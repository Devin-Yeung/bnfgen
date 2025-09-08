{
  nixpkgs,
  rust-overlay,
  crane,
  localSystem,
  target,
}:
let
  inherit (import ./.) mkCrossCraneLib mkShared mkCrossPkgs;

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

  shared = mkShared {
    inherit pkgs craneLib;
  };

  inherit (shared) commonArgs;

in
craneLib.buildPackage (
  commonArgs
  // {
    CARGO_BUILD_TARGET = target;
    CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
  }
)
