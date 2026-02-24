{
  description = "Bnfgen: A highly customizable BNF based fuzzy tests generator";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain (
          p: p.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml
        );
        craneLibStatic = (crane.mkLib pkgs.pkgsStatic).overrideToolchain (
          p: p.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml
        );
        inherit (pkgs) lib;

        unfilteredRoot = ./.;
        src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            ./Cargo.toml
            ./Cargo.lock
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            (lib.fileset.fileFilter (file: file.hasExt "snap") unfilteredRoot)
            (lib.fileset.fileFilter (file: file.hasExt "lalrpop") unfilteredRoot)
            (lib.fileset.fileFilter (file: file.hasExt "bnfgen") unfilteredRoot)
          ];
        };

        commonArgs = {
          inherit src;
          pname = "bnfgen";
          strictDeps = true;
          buildInputs = lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        cargoArtifactsStatic = craneLibStatic.buildDepsOnly commonArgs;

        bnfgen-cli = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            pname = "bnfgen-cli";
            cargoExtraArgs = "-p bnfgen-cli";
          }
        );

        bnfgen-cli-static = craneLib.buildPackage (
          commonArgs
          // {
            cargoArtifacts = cargoArtifactsStatic;
            pname = "bnfgen-cli";
            cargoExtraArgs = "-p bnfgen-cli";
          }
        );
      in
      {
        checks = {
          bnfgen = bnfgen-cli;

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
          bnfgen = bnfgen-cli;
          bnfgen-static = bnfgen-cli-static;
          default = bnfgen-cli;
        };
      }
    );
}
