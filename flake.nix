{
  description = "Bnfgen: A highly customizable BNF based fuzzy tests generator";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      crane,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;
        inherit (pkgs) lib;

        unfilteredRoot = ./.;
        src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            (lib.fileset.fileFilter (file: file.hasExt "snap") unfilteredRoot)
            (lib.fileset.fileFilter (file: file.hasExt "lalrpop") unfilteredRoot)
            (lib.fileset.maybeMissing ./examples)
          ];
        };

        commonArgs = {
          inherit src;
          strictDeps = true;
          buildInputs = lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        bnfgen = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );
      in
      {
        checks = {
          inherit bnfgen;

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
              env.RUSTDOCFLAGS = "--deny warnings";
            }
          );

          bnfgen-fmt = craneLib.cargoFmt {
            inherit src;
          };

          bnfgen-toml-fmt = craneLib.taploFmt {
            src = lib.sources.sourceFilesBySuffices src [ ".toml" ];
          };

          bnfgen-nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
              cargoNextestPartitionsExtraArgs = "--no-tests=pass";
            }
          );
        };

        packages = {
          inherit bnfgen;
          default = bnfgen;
        };
      }
    );
}
