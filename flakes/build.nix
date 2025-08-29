{ inputs, ... }:

{
  perSystem =
    {
      pkgs,
      ...
    }:
    let
      inherit (import ../nix) mkFormula;

      craneLib = inputs.crane.mkLib pkgs;

      # Common arguments for building
      formula = mkFormula {
        inherit pkgs craneLib;
      };

      bnfgen = craneLib.buildPackage (
        formula
        // {
          cargoArtifacts = craneLib.buildDepsOnly formula;
        }
      );
    in
    {
      packages.bnfgen = bnfgen;
      packages.default = bnfgen;
    };
}
