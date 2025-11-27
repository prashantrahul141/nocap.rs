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
      in
      {
        packages = {
          # nix build
          default = naersk'.buildPackage {
            src = ./.;
            buildInputs = buildInputs;
            nativeBuildInputs = [ pkgs.makeWrapper ];
            release = false;
            postInstall = ''
              wrapProgram $out/bin/nocap_rs \
                --set LD_LIBRARY_PATH "${pkgs.lib.makeLibraryPath buildInputs}"
            '';
          };

          # nix build .#check
          check = naersk'.buildPackage {
            src = ./.;
            mode = "check";
          };

          # nix build .#test
          test = naersk'.buildPackage {
            src = ./.;
            mode = "test";
          };

          # nix build .#clippy
          clippy = naersk'.buildPackage {
            src = ./.;
            mode = "clippy";
          };

          # nix build .#fmt
          fmt = naersk'.buildPackage {
            src = ./.;
            mode = "fmt";
          };

        };

        # nix run
        apps = {
          default = {
            type = "app";
            program = "${self.packages.${system}.default}/bin/nocap_rs";
          };
        };

        # For `nix develop`:
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustc
            cargo
            clippy
            rustfmt
          ];

          buildInputs = buildInputs;
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        };
      }
    );
}
