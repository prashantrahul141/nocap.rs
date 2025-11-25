{
  craneLib,
  lib,
  src,
  pkgs,
  buildInputs,
  withCleanup ? false,
}:
let
  cargoArtifacts = craneLib.buildDepsOnly { inherit src; };
in
craneLib.buildPackage {
  inherit cargoArtifacts src;
  doCheck = true;

  LD_LIBRARY_PATH = builtins.foldl' (a: b: "${a}:${b}/lib") "${pkgs.vulkan-loader}/lib" buildInputs;

  cargoExtraArgs = lib.optionalString withCleanup "--features cleanup";
  buildInputs = buildInputs;

}
