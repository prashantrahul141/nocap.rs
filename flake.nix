{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      flake-utils,
      naersk,
      fenix,
      nixpkgs,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ fenix.overlays.default ];
        };

        toolchain = pkgs.fenix.fromToolchainFile {
          file = ./rust-toolchain;
          sha256 = "sha256-zC8E38iDVJ1oPIzCqTk/Ujo9+9kx9dXq7wAwPMpkpg0=";
        };

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };

        buildInputs = with pkgs; [
          wayland
          libxkbcommon
          wayland.dev
          libGL

          # we might need these later:
          # expat
          # jfontconfig
          # freetype
          # freetype.dev
          # pkg-config
          # xorg.libX11
          # xorg.libXcursor
          # xorg.libXi
          # xorg.libXrandr
        ];

        binaryName = "nocap_rs";
        nocap_rs =
          { release }:
          naersk'.buildPackage {
            inherit release;
            src = ./.;
            buildInputs = buildInputs;
            nativeBuildInputs = [
              pkgs.makeWrapper
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.isLinux [ pkgs.mold ];
            RUSTFLAGS = pkgs.lib.optionalString pkgs.stdenv.isLinux "-C link-arg=-fuse-ld=mold";
            postInstall = ''
              path="${pkgs.lib.makeLibraryPath buildInputs}"
              wrapProgram "$out/bin/${binaryName}" \
                --set LD_LIBRARY_PATH "$path"
            '';
          };

      in
      {
        packages = {
          # nix build
          default = nocap_rs { release = true; };

          # nix build .#debug
          debug = nocap_rs { release = false; };

          # nix build .#check
          check = naersk'.buildPackage {
            src = ./.;
            release = false;
            mode = "check";
          };

          # nix build .#test
          test = naersk'.buildPackage {
            src = ./.;
            release = false;
            mode = "test";
          };

          # nix build .#clippy
          clippy = naersk'.buildPackage {
            src = ./.;
            release = false;
            mode = "clippy";
          };

          # nix build .#fmt
          fmt = naersk'.buildPackage {
            src = ./.;
            release = false;
            nativeBuildInputs = [ pkgs.rustfmt ];
            mode = "fmt";
          };

        };

        # nix run
        apps = {
          default = {
            type = "app";
            program = "${self.packages.${system}.default}/bin/${binaryName}";
          };

          debug = {
            type = "app";
            program = "${self.packages.${system}.debug}/bin/${binaryName}";
          };
        };

        # For `nix develop`:
        devShell = pkgs.mkShell {
          inherit buildInputs;
          nativeBuildInputs =
            with pkgs;
            [
              toolchain
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.isLinux [ mold ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        };
      }
    );
}
