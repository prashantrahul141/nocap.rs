{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    {
      self,
      flake-utils,
      naersk,
      nixpkgs,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk { };

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
            src = ./.;
            buildInputs = buildInputs;
            nativeBuildInputs =
              [ pkgs.makeWrapper ]
              ++ pkgs.lib.optionals pkgs.stdenv.isLinux [ pkgs.mold ];
            inherit release;

            CARGO_BUILD_RUSTFLAGS = pkgs.lib.optionalString pkgs.stdenv.isLinux "-C linker=clang -C link-arg=--ld-path=${pkgs.mold}/bin/mold";

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
          nativeBuildInputs = with pkgs; [
            rustc
            cargo
            clippy
            rustfmt
          ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [ mold ];

          buildInputs = buildInputs;
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        };
      }
    );
}
