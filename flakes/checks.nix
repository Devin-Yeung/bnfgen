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

      inherit (shared) commonArgs src;

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
    in
    {
      checks = {
        bnfgen-clippy = craneLib.cargoClippy (
          commonArgs
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          }
        );

        bnfgen-doc = craneLib.cargoDoc (
          commonArgs
          // {
            inherit cargoArtifacts;
            # This can be commented out or tweaked as necessary, e.g. set to
            # `--deny rustdoc::broken-intra-doc-links` to only enforce that lint
            env.RUSTDOCFLAGS = "--deny warnings";
          }
        );

        # Check formatting
        bnfgen-fmt = craneLib.cargoFmt {
          inherit src;
        };

        bnfgen-toml-fmt = craneLib.taploFmt {
          src = pkgs.lib.sources.sourceFilesBySuffices src [ ".toml" ];
          # taplo arguments can be further customized below as needed
          # taploExtraArgs = "--config ./taplo.toml";
        };

      };
    };
}
