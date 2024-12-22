{
  description = "Dioxus Full Stack Demo";

  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
  inputs.nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.devenv.url = "github:cachix/devenv";
  inputs.crane.url = "github:ipetkov/crane";
  inputs.flockenzeit.url = "github:balsoft/flockenzeit";

  outputs =
    inputs@{
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
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
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

        # This currently doesn't work, but it's a good idea.
        # The tests fail due to network access, so we need to skip them.
        # dioxus-cli = pkgs-unstable.dioxus-cli.overrideAttrs (old: rec {
        #   version = "0.6.1";
        #   src = pkgs.fetchCrate {
        #     inherit version;
        #     pname = old.pname;
        #     hash = "sha256-mQnSduf8SHYyUs6gHfI+JAvpRxYQA1DiMlvNofImElU=";
        #   };
        #   cargoDeps = old.cargoDeps.overrideAttrs (pkgs.lib.const {
        #     name = "${old.pname}-vendor.tar.gz";
        #     inherit src;
        #     outputHash = "sha256-7jNOdlX9P9yxIfHTY32IXnT6XV5/9WDEjxhvHvT7bms=";
        #     # outputHash = pkgs.lib.fakeHash;
        #   });
        # });
        dioxus-cli =
          pkgs.callPackage ./nix/dioxus-cli.nix
            {
            };
        rustPlatform = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
          extensions = [ "rust-src" ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustPlatform;

        nodejs = pkgs.nodejs_20;

        build_env = {
          BUILD_DATE = with flockenzeit.lib.splitSecondsSinceEpoch { } self.lastModified; "${F}T${T}${Z}";
          VCS_REF = "${self.rev or "dirty"}";
        };

        nodePackages = pkgs.buildNpmPackage {
          name = "node-packages";
          src = ./.;
          npmDepsHash = builtins.readFile ./npm-deps-hash;
          dontNpmBuild = true;
          inherit nodejs;

          installPhase = ''
            mkdir $out
            cp -r node_modules $out
            ln -s $out/node_modules/.bin $out/bin
          '';
        };

        frontend =
          let
            common = {
              src = ./.;
              pname = "dioxus-fs-demo-frontend";
              version = "0.0.0";
              cargoExtraArgs = "--features web";
              # nativeBuildInputs = with pkgs; [ pkg-config ];
              # buildInputs = with pkgs; [
              #   protobuf
              # ];
              CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
              doCheck = false;
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

            # Build the actual crate itself.
            pkg = craneLib.buildPackage (
              {
                inherit cargoArtifacts;
                doCheck = false;
              }
              // common
              // build_env
            );
          in
          {
            clippy = clippy;
            pkg = pkg;
          };

        frontend-bindgen = pkgs.stdenv.mkDerivation {
          name = "dioxus-fs-demo-frontend-bindgen";
          src = ./.;

          buildPhase = ''
            ${wasm-bindgen-cli}/bin/wasm-bindgen \
              --target bundler \
              --out-dir pkg \
              --omit-default-module-path \
              ${frontend.pkg}/bin/dioxus-fs-demo.wasm

            ln -s ${nodePackages}/node_modules ./node_modules
            export PATH="${nodejs}/bin:${nodePackages}/bin:$PATH"
            webpack
          '';

          installPhase = ''
            copy_hashed() {
                local filename
                local hash
                local dst
                local extension
                local name
                filename="$(basename "$1")"
                hash="''$(${pkgs.b3sum}/bin/b3sum --raw "$1" | head --bytes 6 | base64)"
                extension="''${filename##*.}"
                name="''${filename%.*}"
                dst="$out/$name-$hash.$extension"
                cp "$1" "$dst"
            }

            mkdir $out
            cp -rv dist/* $out/

            copy_hashed "assets/header.svg"
            copy_hashed "assets/main.css"
            copy_hashed "assets/favicon.ico"
          '';
        };

        backend =
          let
            common = {
              src = ./.;
              pname = "phone_db-backend";
              version = "0.0.0";
              cargoExtraArgs = "--features server";
              # nativeBuildInputs = with pkgs; [ pkg-config ];
              buildInputs = with pkgs; [
                pkgs.postgresql_15
                #   openssl
                #   python3
                #   protobuf
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
            coverage = craneLib.cargoTarpaulin ({ cargoArtifacts = clippy; } // common);

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
          in
          {
            inherit clippy coverage pkg;
          };

        combined = pkgs.runCommand "dioxus-fs-demo" { } ''
          mkdir -p $out
          mkdir -p $out/bin/public
          cp -r ${backend.pkg}/. $out
          cp -r ${frontend-bindgen}/. $out/bin/public
        '';

        test_module = pkgs.nixosTest {
          name = "dioxus-fs-demo-test";
          nodes.machine =
            { ... }:
            {
              imports = [
                self.nixosModules.default
              ];
              services.dioxus-fs-demo = {
                enable = true;
                port = 4000;
                secretsFile = builtins.toFile "dioxus-fs-demo.env" ''
                  DATABASE_URL="postgresql://dioxus_fs_demo:your_secure_password_here@localhost/dioxus_fs_demo"
                '';

              };
              system.stateVersion = "24.11";

              services.postgresql = {
                enable = true;
                package = pkgs.postgresql_15;
                extensions = ps: [ ps.postgis ];
                initialScript = pkgs.writeText "init.psql" ''
                  CREATE DATABASE dioxus_fs_demo;
                  CREATE USER dioxus_fs_demo with encrypted password 'your_secure_password_here';
                  ALTER DATABASE dioxus_fs_demo OWNER TO dioxus_fs_demo;
                  ALTER USER dioxus_fs_demo WITH SUPERUSER;
                '';
              };
            };

          testScript = ''
            machine.wait_for_unit("dioxus-fs-demo.service")
            machine.wait_for_open_port(4000)
            machine.succeed("${pkgs.curl}/bin/curl --fail -v http://localhost:4000/_health")
          '';
        };

        port = 4000;
        postgres_port = 6201;

        devShell = devenv.lib.mkShell {
          inherit inputs pkgs;
          modules = [
            {
              packages = [
                rustPlatform
                pkgs.rust-analyzer
                wasm-bindgen-cli
                nodejs
                pkgs.cargo-watch
                pkgs.sqlx-cli
                # pkgs.jq
                # pkgs.openssl
                pkgs.prefetch-npm-deps
                dioxus-cli
                pkgs.b3sum
                pkgs.diesel-cli
                pkgs.postgresql_15
              ];
              enterShell = ''
                # export DIOXUS_ASSET_ROOT="dist"
                export PORT="${toString port}"
                export DATABASE_URL="postgresql://dioxus_fs_demo:your_secure_password_here@localhost:${toString postgres_port}/dioxus_fs_demo"
              '';
              services.postgres = {
                enable = true;
                package = pkgs.postgresql_15.withPackages (ps: [ ps.postgis ]);
                listen_addresses = "127.0.0.1";
                port = postgres_port;
                initialDatabases = [ { name = "dioxus_fs_demo"; } ];
                initialScript = ''
                  \c dioxus_fs_demo;
                  CREATE USER dioxus_fs_demo with encrypted password 'your_secure_password_here';
                  GRANT ALL PRIVILEGES ON DATABASE dioxus_fs_demo TO dioxus_fs_demo;
                  -- GRANT ALL ON SCHEMA public TO dioxus_fs_demo;
                  ALTER USER dioxus_fs_demo WITH SUPERUSER;
                '';
              };

            }
          ];
        };
      in
      {
        checks = {
          brian-backend = backend.clippy;
          frontend-bindgen = frontend.clippy;
          test_module = test_module;
        };
        packages = {
          devenv-up = devShell.config.procfileScript;
          backend = backend.pkg;
          frontend = frontend-bindgen;
          combined = combined;
          default = combined;
        };
        devShells.default = devShell;
      }
    )
    // {
      nixosModules.default = import ./nix/module.nix { inherit self; };
    };
}
