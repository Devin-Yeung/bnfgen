{
  pkgs,
  craneLib,
}:
let
  inherit (pkgs) lib;
  lalrpopFilter = path: _type: builtins.match ".*lalrpop$" path != null;
  bnfgenFilter = path: _type: builtins.match ".*bnfgen$" path != null;
  snapshotFilter = path: _type: builtins.match ".*snap$" path != null;
  filter =
    path: type:
    (craneLib.filterCargoSources path type)
    || (lalrpopFilter path type)
    || (bnfgenFilter path type)
    || (snapshotFilter path type);
in
{
  src = lib.cleanSourceWith {
    src = ../.; # The original, unfiltered source
    name = "source"; # Be reproducible, regardless of the directory name
    inherit filter;
  };
  strictDeps = true;
  buildInputs = [
    # Add extra build inputs if needed
  ]
  ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
    pkgs.libiconv
  ];
}
