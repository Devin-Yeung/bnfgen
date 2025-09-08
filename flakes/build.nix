{ inputs, ... }:

{
  perSystem =
    {
      pkgs,
      ...
    }:
    let
      inherit (import ../nix) mkShared;

      craneLib = inputs.crane.mkLib pkgs;

      # Common arguments for building
      shared = mkShared {
        inherit pkgs craneLib;
      };

      inherit (shared) commonArgs;

      bnfgen = craneLib.buildPackage (
        commonArgs
        // {
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        }
      );
    in
    {
      checks = {
        inherit bnfgen;
      };
      packages.bnfgen = bnfgen;
      packages.default = bnfgen;
    };
}
