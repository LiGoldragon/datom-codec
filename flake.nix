{
  description = "datom-codec — positional typed data over Protos";

  inputs = {
    base.url = "github:LiGoldragon/build-base";
    ethos-zero.url = "github:LiGoldragon/ethos-zero";
    ethos-zero.inputs.base.follows = "base";
  };

  # Cargo.nix is generated from Cargo.lock by the build base's crate2nix and
  # committed, so evaluation imports no derivation:
  #   nix shell github:LiGoldragon/build-base#toolchain github:LiGoldragon/build-base#crate2nix -c crate2nix generate
  outputs =
    { self, base, ethos-zero }:
    base.lib.workspaceOutputs {
      cargoNix = ./Cargo.nix;

      # Every test runs under the rkyv feature, the archival tests with them.
      memberChecks = { check, ... }: { datom-codec = check "datom-codec" [ "default" "rkyv" ]; };

      extraChecks =
        { pkgs, toolchain, doc, ... }:
        let
          source = ./src;
          generator = ethos-zero.packages.${pkgs.stdenv.hostPlatform.system}.default;
          manifest = builtins.fromTOML (builtins.readFile ./Cargo.toml);
          # The Rust sources and tests the format and lint checks read.
          rust = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.unions [ ./src ./tests ./generated ];
          };
        in
        {
          no-production-free-functions = pkgs.runCommand "datom-codec-no-production-free-functions" { } ''
            if grep -R -n -E '^(pub(\([^)]*\))? )?fn ' ${source}; then
              echo "production Rust must not use module-level free functions" >&2
              exit 1
            fi
            touch $out
          '';
          no-production-inherent-methods = pkgs.runCommand "datom-codec-no-production-inherent-methods" { } ''
            if grep -R -n -E '^[[:space:]]*impl[[:space:]]+[[:alpha:]_][[:alnum:]_:<>]*[[:space:]]*\{' ${source}; then
              echo "production Rust must home behavior in traits" >&2
              exit 1
            fi
            touch $out
          '';
          no-zst-behavior = pkgs.runCommand "datom-codec-no-zst-behavior" { } ''
            if grep -R -n -E '^[[:space:]]*(pub[[:space:]]+)?struct[[:space:]]+[[:alpha:]_][[:alnum:]_]*[[:space:]]*;' ${source}; then
              echo "behavioral Rust nouns must carry data" >&2
              exit 1
            fi
            touch $out
          '';
          no-forbidden-vocabulary = pkgs.runCommand "datom-codec-no-forbidden-vocabulary" { } ''
            if grep -R -n -i -E 'encode|decode|codec|transcode' ${source}; then
              echo "Datomic names must use the ruled form vocabulary" >&2
              exit 1
            fi
            touch $out
          '';
          generated-kinds = pkgs.runCommand "datom-codec-generated-kinds" {
            inherit generator;
            declaration = ./datom-codec-kinds.ethos;
            committed = ./generated/datom-codec-kinds.rs;
          } (builtins.readFile ./checks/generated-kinds.sh);
          checked-anatomy = pkgs.runCommand "datom-codec-checked-anatomy" {
            inherit generator;
            declaration = ./datom-codec.ethos;
          } (builtins.readFile ./checks/checked-anatomy.sh);
          fmt = pkgs.runCommand "datom-codec-fmt" { nativeBuildInputs = [ toolchain ]; } ''
            rustfmt --check --edition ${manifest.package.edition} ${rust}/src/lib.rs ${rust}/tests/*.rs
            touch $out
          '';
          doc = doc "datom-codec" [ "default" "rkyv" ] [ "-D" "warnings" ];
        };
    };
}
