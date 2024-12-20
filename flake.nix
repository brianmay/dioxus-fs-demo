{
  description = "Time tracking application";

  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
  inputs.nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.devenv.url = "github:cachix/devenv";
  inputs.crane.url = "github:ipetkov/crane";
  inputs.flockenzeit.url = "github:balsoft/flockenzeit";

  outputs = inputs @ {
    self,
    nixpkgs,
    nixpkgs-unstable,
    flake-utils,
    rust-overlay,
    devenv,
    crane,
    flockenzeit,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };
        pkgs-unstable = nixpkgs-unstable.legacyPackages.${system};
        wasm-bindgen-cli = pkgs.wasm-bindgen-cli.override (old: {
          version = "0.2.99";
          hash = "sha256-1AN2E9t/lZhbXdVznhTcniy+7ZzlaEp/gwLEAucs6EA=";
          cargoHash = "sha256-DbwAh8RJtW38LJp+J9Ht8fAROK9OabaJ85D9C/Vkve4=";
          # version = "0.2.97";
          # hash = "sha256-DDUdJtjCrGxZV84QcytdxrmS5qvXD8Gcdq4OApj5ktI=";
          # cargoHash = "sha256-Zfc2aqG7Qi44dY2Jz1MCdpcL3lk8C/3dt7QiE0QlNhc=";
          # hash = pkgs.lib.fakeHash;
          # cargoHash = pkgs.lib.fakeHash;
        });
        rustPlatform = pkgs.rust-bin.stable.latest.default.override {
          targets = ["wasm32-unknown-unknown"];
          extensions = ["rust-src"];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustPlatform;

        build_env = {
          BUILD_DATE = with flockenzeit.lib.splitSecondsSinceEpoch {} self.lastModified; "${F}T${T}${Z}";
          VCS_REF = "${self.rev or "dirty"}";
        };

        backend = let
          common = {
            src = ./.;
            pname = "phone_db-backend";
            version = "0.0.0";
            cargoExtraArgs = "-p backend";
            nativeBuildInputs = with pkgs; [pkg-config];
            buildInputs = with pkgs; [
              openssl
              python3
              protobuf
            ];
            # See https://github.com/ipetkov/crane/issues/414#issuecomment-1860852084
            # for possible work around if this is required in the future.
            # installCargoArtifactsMode = "use-zstd";
          };

          # Build *just* the cargo dependencies, so we can reuse
          # all of that work (e.g. via cachix) when running in CI
          cargoArtifacts = craneLib.buildDepsOnly common;

          # Run clippy (and deny all warnings) on the crate source.
          clippy = craneLib.cargoClippy (
            {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "-- --deny warnings";
            }
            // common
          );

          # Next, we want to run the tests and collect code-coverage, _but only if
          # the clippy checks pass_ so we do not waste any extra cycles.
          coverage = craneLib.cargoTarpaulin ({cargoArtifacts = clippy;} // common);

          # Build the actual crate itself.
          pkg = craneLib.buildPackage (
            {
              inherit cargoArtifacts;
              doCheck = true;
              # CARGO_LOG = "cargo::core::compiler::fingerprint=info";
            }
            // common
            // build_env
          );
        in {
          inherit clippy coverage pkg;
        };

        port = 4000;

        devShell = devenv.lib.mkShell {
          inherit inputs pkgs;
          modules = [
            {
              packages = [
                rustPlatform
                pkgs.rust-analyzer
                wasm-bindgen-cli
                pkgs.nodejs_20
                pkgs.cargo-watch
                pkgs.sqlx-cli
                pkgs.jq
                pkgs.openssl
                pkgs-unstable.dioxus-cli
                pkgs.b3sum
              ];
              enterShell = ''
                export HTTP_LISTEN="localhost:${toString port}"
                export STATIC_PATH="frontend/dist"
              '';
            }
          ];
        };
      in {
        checks = {
          brian-backend = backend.clippy;
        };
        packages = {
          devenv-up = devShell.config.procfileScript;
          backend = backend.pkg;
        };
        devShells.default = devShell;
      }
    );
}
