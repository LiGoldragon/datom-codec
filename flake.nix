{
  description = "datom-codec — positional typed data over Protos";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-build = {
      url = "github:LiGoldragon/rust-build";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    ethos-zero = {
      url = "github:LiGoldragon/ethos-zero/dc54e3323ae00dc3f88f4d65c2785e6800c06b74";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-build, ethos-zero }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        rust = rust-build.lib.${system}.fromPkgs pkgs;
        inherit (rust) craneLib toolchain;
        src = rust.cleanSource {
          root = ./.;
        };
        commonArguments = { inherit src; strictDeps = true; };
        cargoArtifacts = craneLib.buildDepsOnly commonArguments;
      in
      {
        packages.default = craneLib.buildPackage (commonArguments // { inherit cargoArtifacts; });
        checks = {
          build = craneLib.cargoBuild (commonArguments // { inherit cargoArtifacts; });
          test = craneLib.cargoTest (commonArguments // { inherit cargoArtifacts; });
          no-production-free-functions = pkgs.runCommand "datom-codec-no-production-free-functions" { } ''
            if grep -R -n -E '^(pub(\([^)]*\))? )?fn ' ${src}/src; then
              echo "production Rust must not use module-level free functions" >&2
              exit 1
            fi
            touch $out
          '';
          no-production-inherent-methods = pkgs.runCommand "datom-codec-no-production-inherent-methods" { } ''
            if grep -R -n -E '^[[:space:]]*impl[[:space:]]+[[:alpha:]_][[:alnum:]_:<>]*[[:space:]]*\{' ${src}/src; then
              echo "production Rust must home behavior in traits" >&2
              exit 1
            fi
            touch $out
          '';
          no-zst-behavior = pkgs.runCommand "datom-codec-no-zst-behavior" { } ''
            if grep -R -n -E '^[[:space:]]*(pub[[:space:]]+)?struct[[:space:]]+[[:alpha:]_][[:alnum:]_]*[[:space:]]*;' ${src}/src; then
              echo "behavioral Rust nouns must carry data" >&2
              exit 1
            fi
            touch $out
          '';
          no-forbidden-vocabulary = pkgs.runCommand "datom-codec-no-forbidden-vocabulary" { } ''
            if grep -R -n -i -E 'encode|decode|codec|transcode' ${src}/src; then
              echo "Datomic names must use the ruled form vocabulary" >&2
              exit 1
            fi
            touch $out
          '';
          generated-contract = pkgs.runCommand "datom-codec-generated-contract" {
            generator = ethos-zero.packages.${system}.default;
            declaration = ./datom-codec.ethos;
            committed = ./generated-contract/datom-codec.rs;
          } (builtins.readFile ./checks/generated-contract.sh);
          doc = craneLib.cargoDoc (commonArguments // {
            inherit cargoArtifacts;
            RUSTDOCFLAGS = "-D warnings";
          });
          fmt = craneLib.cargoFmt { inherit src; };
          clippy = craneLib.cargoClippy (commonArguments // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });
        };
        devShells.default = pkgs.mkShell {
          name = "datom-codec";
          packages = [ pkgs.jujutsu toolchain ];
        };
      });
}
