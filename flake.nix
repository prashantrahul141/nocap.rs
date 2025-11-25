{
  description = "nocap.rs";

  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/release-25.05";
    };

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      crane,
      fenix,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ fenix.overlays.default ];
        };

        inherit (pkgs) lib;

        toolchain = pkgs.fenix.fromToolchainFile {
          file = ./rust-toolchain;
          sha256 = "sha256-SDu4snEWjuZU475PERvu+iO50Mi39KVjqCeJeNvpguU=";
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
        src = ./.;

        commonArgs = { inherit src; };

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

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        nocap-rs = pkgs.callPackage ./nix/nocap-rs.nix {
          inherit
            craneLib
            lib
            src
            buildInputs
            pkgs
            ;
        };
      in
      {
        packages = {
          inherit nocap-rs;
          nocap-rs-all = nocap-rs.override { withCleanup = true; };
          default = nocap-rs;
          image = pkgs.dockerTools.buildImage {
            name = "nocap_rs";
            config = {
              Cmd = [ "${nocap-rs}/bin/nocap-rs" ];
            };
          };

          # not using flake checks to run them individually
          checks = {
            clippy = craneLib.cargoClippy (
              commonArgs
              // {
                inherit cargoArtifacts;
              }
            );

            fmt = craneLib.cargoFmt {
              inherit src;
            };
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = buildInputs;
          nativeBuildInputs = [
            toolchain
          ];

          LD_LIBRARY_PATH = builtins.foldl' (a: b: "${a}:${b}/lib") "${pkgs.vulkan-loader}/lib" buildInputs;
        };

        formatter = pkgs.nixfmt-tree;
      }
    );
}
