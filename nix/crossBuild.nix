{
  nixpkgs,
  rust-overlay,
  crane,
  localSystem,
  target,
}:
let
  inherit (import ./.) mkCrossCraneLib mkFormula mkCrossPkgs;

  pkgs = mkCrossPkgs {
    inherit
      nixpkgs
      rust-overlay
      localSystem
      target
      ;
  };

  craneLib = mkCrossCraneLib {
    inherit crane;
    inherit pkgs target;
  };

  formula = mkFormula {
    inherit pkgs craneLib;
  };

in
craneLib.buildPackage (
  formula
  // {
    CARGO_BUILD_TARGET = target;
    CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
  }
)
