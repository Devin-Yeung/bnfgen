{ inputs, ... }:

{
  perSystem =
    {
      system,
      ...
    }:
    let
      inherit (import ../nix) crossBuild;
      crossBuild' =
        target:
        crossBuild {
          inherit (inputs) nixpkgs rust-overlay crane;
          localSystem = system;
          inherit target;
        };
    in
    {
      packages = {
        "bnfgen-static-x86_64" = crossBuild' "x86_64-unknown-linux-musl";
        "bnfgen-static-aarch64" = crossBuild' "aarch64-unknown-linux-musl";
      };
    };
}
