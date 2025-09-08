{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

{
  # https://devenv.sh/packages/
  packages = [ ];

  # https://devenv.sh/languages/
  languages.rust.enable = true;

  enterShell = ''
    rustc --version
  '';

  # https://devenv.sh/git-hooks/
  git-hooks.hooks = {
    rustfmt.enable = true;
    yamlfmt.enable = true;
    treefmt.enable = true;
  };

  # See full reference at https://devenv.sh/reference/options/
}
