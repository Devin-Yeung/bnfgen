{
  pkgs,
  craneLib,
}:
{
  src = craneLib.cleanCargoSource ../.;
  strictDeps = true;
  buildInputs = [
    # Add extra build inputs if needed
  ]
  ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
    pkgs.libiconv
  ];
}
