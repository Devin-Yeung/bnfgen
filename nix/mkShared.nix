{
  pkgs,
  craneLib,
}:
let
  inherit (pkgs) lib;

  unfilteredRoot = ../.; # The original, unfiltered source
  src = lib.fileset.toSource {
    root = unfilteredRoot;
    fileset = lib.fileset.unions [
      # Default files from crane (Rust and cargo files)
      (craneLib.fileset.commonCargoSources unfilteredRoot)
      # Keep any snapshots files
      (lib.fileset.fileFilter (file: file.hasExt "snap") unfilteredRoot)
      # Keep any lalrpop grammar
      (lib.fileset.fileFilter (file: file.hasExt "lalrpop") unfilteredRoot)
      # Folder that stores all the bnfgen example
      (lib.fileset.maybeMissing ../examples)
    ];
  };
in
{
  inherit src;
  commonArgs = {
    inherit src;
    strictDeps = true;
    buildInputs = [
      # Add extra build inputs if needed
    ]
    ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
      pkgs.libiconv
    ];
  };
}
