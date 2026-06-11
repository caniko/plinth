{
  description = "Plinth - personal website with Leptos SSR";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

  };

  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    rust-overlay,
    ...
  }:
  let
    topPkgs = import nixpkgs {
      system = "x86_64-linux";
      overlays = [(import rust-overlay)];
    };
    topProjectSiteLib = import ./nix/project-site.nix {
      pkgs = topPkgs;
      lib = nixpkgs.lib;
    };
    topPlinthProjects = topProjectSiteLib.mkProjectRegistry {
      plinth = topProjectSiteLib.mkProjectDefinition {
        id = "plinth";
        pname = "plinth-site";
        title = "Plinth";
        description = "A self-hosted personal website platform built with Leptos, Postgres, semantic search, and Nix.";
        domain = "plinth.tartanoglu.com";
        configPath = ./website/plinth-project.toml;
        staticPaths = [
          {
            source = ./logo/plinth-logo.svg;
            target = "logo/plinth-logo.svg";
          }
        ];
        sourceUrl = "https://codeberg.org/caniko/plinth";
        appendStandardFooterLinks = true;
        portfolioDate = "2026-06-07T00:00:00Z";
        techStack = ["Rust" "Leptos" "Postgres" "Nix"];
      };
    };
    topProjectReferences = nixpkgs.lib.mapAttrs (_: topProjectSiteLib.projectReferenceFromDefinition) topPlinthProjects;
    topPortfolioManifests = nixpkgs.lib.mapAttrs (_: topProjectSiteLib.portfolioManifestFromDefinition) topPlinthProjects;
  in
    {
      lib = topProjectSiteLib // {
        plinthProjects = topPlinthProjects;
        projectReferences = topProjectReferences;
        portfolioManifests = topPortfolioManifests;
      };

      # NixOS module for declarative deployment
      nixosModules.default = import ./modules/plinth.nix;
      nixosModules.plinth = import ./modules/plinth.nix;

      # Overlay for downstream users to access pre-built packages
      # For custom builds, import the flake and use buildPlinth directly
      overlays.default = final: prev: {
        inherit (self.packages.${final.system})
          plinth
          plinth-csr
          plinth-cli
          plinth-person
          plinth-project
          plinth-dev
          plinth-minimal;
      };
    }
    // flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };

        inherit (pkgs) lib;

        # wasm-bindgen-cli version — must match Cargo.lock.
        # Update this when wasm-bindgen changes in Cargo.lock, then fix the hashes.
        wasmBindgenVersion = "0.2.114";

        wasm-bindgen-cli = pkgs.buildWasmBindgenCli {
          version = wasmBindgenVersion;
          src = pkgs.fetchCrate {
            pname = "wasm-bindgen-cli";
            version = wasmBindgenVersion;
            hash = "sha256-xrCym+rFY6EUQFWyWl6OPA+LtftpUAE5pIaElAIVqW0=";
          };
          cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
            name = "wasm-bindgen-cli-${wasmBindgenVersion}-vendor";
            src = pkgs.fetchCrate {
              pname = "wasm-bindgen-cli";
              version = wasmBindgenVersion;
              hash = "sha256-xrCym+rFY6EUQFWyWl6OPA+LtftpUAE5pIaElAIVqW0=";
            };
            hash = "sha256-Z8+dUXPQq7S+Q7DWNr2Y9d8GMuEdSnq00quUR0wDNPM=";
          };
        };

        rustToolchainFor = p:
          p.rust-bin.nightly."2026-02-28".default.override {
            # Set the build targets supported by the toolchain
            # wasm32-unknown-unknown is required for Leptos client-side code
            # Using nightly for -Zshare-generics=y flag
            # Pinned to specific date to prevent surprise breakage from nightly changes.
            # To update: change the date, run `nix flake check`, and commit as a deliberate PR.
            extensions = ["rust-src" "rust-analyzer" "rustfmt" "rustc-codegen-cranelift-preview"];
            targets = ["wasm32-unknown-unknown"];
          };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchainFor;
        postgresqlWithPgvector = pkgs.postgresql_16.withPackages (ps: [ps.pgvector]);

        # Parameterized build function for configurability
        buildPlinth = {
          profile ? "prod",
          wasmOptLevel ? null, # Reserved for future WASM optimization configuration
          enableMold ? pkgs.stdenv.isLinux,
          extraRustFlags ? "",
        }: let
          # Profile-specific settings
          profileSettings =
            if profile == "dev"
            then {
              cargoProfile = "dev";
              leptosEnv = "DEV";
              rustOptLevel = "0";
            }
            else if profile == "minimal"
            then {
              cargoProfile = "release";
              leptosEnv = "PROD";
              rustOptLevel = "z"; # Optimize for size
            }
            else {
              # prod profile (default)
              cargoProfile = "release";
              leptosEnv = "PROD";
              rustOptLevel = "3";
            };

          # Build RUSTFLAGS based on configuration
          rustFlags =
            lib.concatStringsSep " " (
              lib.filter (s: s != "") [
                (lib.optionalString enableMold "-C link-arg=-fuse-ld=mold")
                "-Zshare-generics=y"
                "--cfg tokio_unstable"
                extraRustFlags
              ]
            );

          # Cargo profile flag for build command
          cargoProfileFlag =
            if profileSettings.cargoProfile == "dev"
            then ""
            else "--release";

          # Conditional linker configuration
          # Dynamically set linker for the current architecture
          rustTarget = pkgs.stdenv.hostPlatform.rust.rustcTarget;
          rustTargetUpper = lib.toUpper (lib.replaceStrings ["-"] ["_"] rustTarget);
          linkerEnvVar = "CARGO_TARGET_${rustTargetUpper}_LINKER";
          # Use target-specific RUSTFLAGS so mold flags don't leak to wasm32
          rustflagsEnvVar = "CARGO_TARGET_${rustTargetUpper}_RUSTFLAGS";
          linkerConfig =
            if pkgs.stdenv.isLinux && enableMold
            then {
              ${linkerEnvVar} = "${pkgs.clang}/bin/clang";
              ${rustflagsEnvVar} = rustFlags;
            }
            else {
              ${rustflagsEnvVar} = lib.concatStringsSep " " (lib.filter (s: s != "") ["-Zshare-generics=y" "--cfg tokio_unstable" extraRustFlags]);
            };
        in
          craneLib.buildPackage (commonArgs
            // linkerConfig
            // {
              pname = "plinth";
              inherit cargoArtifacts;

              # Use cargo-leptos to build with appropriate profile (workspace-level config)
              buildPhaseCargoCommand = "cargo leptos build ${cargoProfileFlag}";

              # Tests run in the dedicated plinth-test check, which starts
              # Postgres for SQLx integration tests.
              doCheck = false;

              # cargo-leptos doesn't produce a cargo build log, so skip crane's auto-install
              doNotPostBuildInstallCargoBinaries = true;

              # Set optimization level for WASM
              CARGO_PROFILE_RELEASE_OPT_LEVEL = profileSettings.rustOptLevel;
              LEPTOS_ENV = profileSettings.leptosEnv;

              # Install the server binary and site assets with wrapper script
              installPhase = ''
                mkdir -p $out/bin
                mkdir -p $out/site

                # Determine binary path based on profile
                if [ "${profileSettings.cargoProfile}" = "dev" ]; then
                  binaryPath="target/debug/plinth-server"
                else
                  binaryPath="target/release/plinth-server"
                fi

                # Copy server binary
                cp $binaryPath $out/bin/plinth-server-unwrapped

                # Copy CLI binary
                if [ "${profileSettings.cargoProfile}" = "dev" ]; then
                  cliBinaryPath="target/debug/plinth"
                else
                  cliBinaryPath="target/release/plinth"
                fi
                cp $cliBinaryPath $out/bin/plinth

                # Create wrapper script that sets LEPTOS_SITE_ROOT
                cat > $out/bin/plinth-server <<EOF
                #!/bin/sh
                export LEPTOS_SITE_ROOT="\''${LEPTOS_SITE_ROOT:-$out/site}"
                exec $out/bin/plinth-server-unwrapped "\$@"
                EOF
                chmod +x $out/bin/plinth-server

                # Copy site assets (includes WASM, JS, CSS)
                cp -r target/site/* $out/site/

                # Install example configuration
                mkdir -p $out/share/plinth
                if [ -f plinth.toml ]; then
                  cp plinth.toml $out/share/plinth/plinth.toml
                fi
              '';
            });

        # When filtering sources, we include all necessary files for Leptos + Tailwind
        unfilteredRoot = ./.; # The original, unfiltered source
        src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            # Default files from crane (Rust and cargo files)
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            # Workspace members
            (lib.fileset.maybeMissing ./crates/client)
            (lib.fileset.maybeMissing ./crates/server)
            (lib.fileset.maybeMissing ./crates/shared)
            (lib.fileset.maybeMissing ./crates/cli)
            (lib.fileset.maybeMissing ./crates/forge)
            (lib.fileset.maybeMissing ./crates/person)
            (lib.fileset.maybeMissing ./crates/project)
            # Tailwind configuration
            (lib.fileset.fileFilter (
                file:
                  lib.any file.hasExt [
                    "js" # tailwind.config.js
                    "css" # input.css
                  ]
              )
              unfilteredRoot)
            # Public assets directory
            (lib.fileset.maybeMissing ./public)
            # Static CSR shell
            (lib.fileset.maybeMissing ./csr)
            # Default configuration file
            (lib.fileset.maybeMissing ./plinth.toml)
            # Development helper scripts used by Nix checks
            (lib.fileset.maybeMissing ./scripts)
          ];
        };

        # Common arguments for all builds
        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs =
            [
              pkgs.openssl
              pkgs.onnxruntime
            ]
            ++ lib.optionals pkgs.stdenv.isDarwin [
              # Additional darwin specific inputs can be set here
              pkgs.libiconv
            ];

          nativeBuildInputs =
            [
              # cargo-leptos for building Leptos apps with SSR
              pkgs.cargo-leptos
              # Tailwind CSS standalone binary
              pkgs.tailwindcss
              # wasm-bindgen-cli version must match Cargo.lock
              wasm-bindgen-cli
              # libclang needed by bindgen-based dependencies
              pkgs.llvmPackages.libclang.lib
              # pkg-config needed by openssl-sys
              pkgs.pkg-config
              # wasm-opt for optimizing WASM output
              pkgs.binaryen
            ]
            ++ lib.optionals pkgs.stdenv.isLinux [
              # Mold linker for faster linking on Linux
              pkgs.mold
              pkgs.clang
            ];

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          # Tell ort-sys to use the system ONNX Runtime instead of downloading
          ORT_LIB_LOCATION = "${pkgs.onnxruntime}/lib";
          ORT_PREFER_DYNAMIC_LINK = "1";

          # Note: RUSTFLAGS and linker config are set in buildPlinth
        };

        # Linker configuration for cargo test and other non-build checks
        # (buildPlinth computes its own internally for full configurability)
        rustTarget = pkgs.stdenv.hostPlatform.rust.rustcTarget;
        rustTargetUpper = lib.toUpper (lib.replaceStrings ["-"] ["_"] rustTarget);
        linkerEnvVar = "CARGO_TARGET_${rustTargetUpper}_LINKER";
        rustflagsEnvVar = "CARGO_TARGET_${rustTargetUpper}_RUSTFLAGS";
        baseLinkerConfig =
          if pkgs.stdenv.isLinux
          then {
            ${linkerEnvVar} = "${pkgs.clang}/bin/clang";
            ${rustflagsEnvVar} = "-C link-arg=-fuse-ld=mold -Zshare-generics=y --cfg tokio_unstable";
          }
          else {
            ${rustflagsEnvVar} = "-Zshare-generics=y --cfg tokio_unstable";
          };

        # Build cargo dependencies separately for caching
        cargoArtifacts = craneLib.buildDepsOnly (commonArgs
          // {
            pname = "plinth-deps";
          });

        # Build variants using the parameterized function
        plinth = buildPlinth {}; # Default production build
        plinth-dev = buildPlinth {profile = "dev";};
        plinth-minimal = buildPlinth {profile = "minimal";};

        plinth-csr = craneLib.buildPackage (commonArgs
          // {
            pname = "plinth-csr";
            inherit cargoArtifacts;

            buildPhaseCargoCommand = ''
              cargo build --package plinth-client --lib \
                --target wasm32-unknown-unknown \
                --no-default-features \
                --features csr,brick-blog,brick-portfolio,brick-todo,brick-activity \
                --release
            '';

            doCheck = false;
            doNotPostBuildInstallCargoBinaries = true;

            installPhase = ''
              mkdir -p $out/pkg

              wasm-bindgen \
                target/wasm32-unknown-unknown/release/plinth_client.wasm \
                --target web \
                --out-dir $out/pkg \
                --out-name plinth

              tailwindcss \
                --input input.css \
                --config tailwind.config.js \
                --output $out/pkg/plinth.css \
                --minify

              cp csr/index.html $out/index.html
              cp -r public/* $out/
            '';
          });

        # Standalone CLI package (extracts just the CLI from the main build)
        plinth-cli = pkgs.runCommand "plinth-cli" {} ''
          mkdir -p $out/bin
          cp ${plinth}/bin/plinth $out/bin/plinth
        '';

        plinth-project = craneLib.buildPackage (commonArgs
          // baseLinkerConfig
          // {
            inherit cargoArtifacts;
            pname = "plinth-project";
            cargoExtraArgs = "--locked --package plinth-project --bin plinth-project";
            doCheck = false;
          });

        plinth-person = craneLib.buildPackage (commonArgs
          // baseLinkerConfig
          // {
            inherit cargoArtifacts;
            pname = "plinth-person";
            cargoExtraArgs = "--locked --package plinth-person";
            doCheck = true;
          });
        projectSiteLib = import ./nix/project-site.nix {
          inherit pkgs lib;
          plinthProject = plinth-project;
        };

        # Documentation built with mdBook and published as the Codeberg Pages site.
        docs = pkgs.stdenv.mkDerivation {
          pname = "plinth-docs";
          version = "0.1.0";
          src = lib.fileset.toSource {
            root = unfilteredRoot;
            fileset = lib.fileset.maybeMissing ./docs;
          };
          nativeBuildInputs = [pkgs.mdbook];
          phases = ["buildPhase" "installPhase"];
          buildPhase = ''
            if [ -d "$src/docs" ]; then
              cp -r --no-preserve=mode "$src/docs" docs
            else
              cp -r --no-preserve=mode "$src" docs
            fi
            mdbook build docs
          '';
          installPhase = ''
            cp -r docs/book $out
          '';
        };

        plinthProjects = projectSiteLib.mkProjectRegistry {
          plinth = projectSiteLib.mkProjectDefinition {
            id = "plinth";
            pname = "plinth-site";
            title = "Plinth";
            description = "A self-hosted personal website platform built with Leptos, Postgres, semantic search, and Nix.";
            domain = "plinth.tartanoglu.com";
            configPath = ./website/plinth-project.toml;
            staticPaths = [
              {
                source = ./logo/plinth-logo.svg;
                target = "logo/plinth-logo.svg";
              }
            ];
            docsPackage = docs;
            sourceUrl = "https://codeberg.org/caniko/plinth";
            appendStandardFooterLinks = true;
            portfolioDate = "2026-06-07T00:00:00Z";
            techStack = ["Rust" "Leptos" "Postgres" "Nix"];
          };
        };

        projectReferences = lib.mapAttrs (_: projectSiteLib.projectReferenceFromDefinition) plinthProjects;
        portfolioManifests = lib.mapAttrs (_: projectSiteLib.portfolioManifestFromDefinition) plinthProjects;
        projectReferencesJson = pkgs.writeText "plinth-project-references.json" (builtins.toJSON projectReferences);
        portfolioManifestsJson = pkgs.writeText "plinth-portfolio-manifests.json" (builtins.toJSON portfolioManifests);
        portfolioManifestFiles = lib.mapAttrs (_: projectSiteLib.portfolioManifestFileFromDefinition) plinthProjects;
        site = projectSiteLib.mkProjectSiteFromDefinition plinthProjects.plinth;
        website = site;
        mdbook = docs;

        websiteMarkers = pkgs.runCommand "plinth-website-markers" {} ''
          grep -q 'Plinth' ${website}/index.html
          grep -q 'href="/docs/"' ${website}/index.html
          grep -q 'From clone to local site' ${website}/index.html
          grep -q 'feature-card' ${website}/index.html
          grep -q 'workflow-steps' ${website}/index.html
          grep -q 'trust-panel' ${website}/index.html
          test -f ${website}/plinth-logo.svg
          touch $out
        '';

        # Rustdoc API documentation
        rustdoc = craneLib.cargoDoc (commonArgs
          // baseLinkerConfig
          // {
            inherit cargoArtifacts;
            cargoDocExtraArgs = "--workspace --exclude plinth-client --no-deps";
          });

        # Combined output: mdBook docs + rustdoc API reference
        docs-full = pkgs.runCommand "plinth-docs-full" {} ''
          mkdir -p $out
          cp -r ${docs}/* $out/
          mkdir -p $out/api/rustdoc
          cp -r ${rustdoc}/share/doc/* $out/api/rustdoc/
        '';
      in {
        checks = {
          # Build the app as part of `nix flake check` for convenience
          inherit plinth plinth-csr;

          # Run clippy (and deny all warnings) on the crate source
          plinth-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          # Check formatting
          plinth-fmt = craneLib.cargoFmt commonArgs;

          # Run cargo tests against a sandbox-local Postgres instance.
          plinth-test = craneLib.cargoTest (
            commonArgs
            // baseLinkerConfig
            // {
              inherit cargoArtifacts;
              nativeBuildInputs = commonArgs.nativeBuildInputs ++ [postgresqlWithPgvector];
              cargoTestExtraArgs = "--workspace --all-targets -- --test-threads=1";
              preCheck = ''
                export PGDATA="$TMPDIR/pgdata"
                export PGHOST="$TMPDIR/pgsocket"
                export DATABASE_URL="postgres://localhost/plinth?host=$PGHOST"

                mkdir -p "$PGHOST"
                initdb -D "$PGDATA" --auth=trust --no-locale --encoding=UTF8
                pg_ctl -D "$PGDATA" -l "$TMPDIR/postgres.log" -o "-k $PGHOST" start
                createdb -h "$PGHOST" plinth
                psql -h "$PGHOST" -d plinth -v ON_ERROR_STOP=1 -c "CREATE EXTENSION IF NOT EXISTS vector"
              '';
              postCheck = ''
                pg_ctl -D "$PGDATA" stop
              '';
            }
          );

          # Verify wasm-bindgen-cli version matches Cargo.lock
          wasm-bindgen-version-check = pkgs.runCommand "wasm-bindgen-version-check" {} ''
            LOCK_VERSION=$(${pkgs.python3}/bin/python3 -c "
            import json, sys
            with open('${./Cargo.lock}') as f:
                content = f.read()
            # Parse TOML manually — find wasm-bindgen version (not wasm-bindgen-cli)
            in_pkg = False
            for line in content.split('\n'):
                if line.strip() == '[[package]]':
                    in_pkg = True
                    pkg_name = None
                    pkg_version = None
                elif in_pkg and line.startswith('name = '):
                    pkg_name = line.split('\"')[1]
                elif in_pkg and line.startswith('version = '):
                    pkg_version = line.split('\"')[1]
                    if pkg_name == 'wasm-bindgen':
                        print(pkg_version)
                        sys.exit(0)
                    in_pkg = True
            print('NOT_FOUND')
            ")
            FLAKE_VERSION="${wasmBindgenVersion}"
            if [ "$LOCK_VERSION" != "$FLAKE_VERSION" ]; then
              echo ""
              echo "ERROR: wasm-bindgen version mismatch!"
              echo "  Cargo.lock has: $LOCK_VERSION"
              echo "  flake.nix has:  $FLAKE_VERSION"
              echo ""
              echo "To fix: update wasmBindgenVersion in flake.nix to \"$LOCK_VERSION\""
              echo "        then update the SRI hashes (build will show expected hash on first failure)."
              echo ""
              exit 1
            fi
            echo "wasm-bindgen version check passed: $FLAKE_VERSION"
            mkdir -p $out
          '';
          website = website;
          website-markers = websiteMarkers;
        };

        packages = {
          default = plinth;
          inherit plinth plinth-csr plinth-cli plinth-person plinth-project plinth-dev plinth-minimal;
          inherit docs website site mdbook rustdoc docs-full projectReferencesJson portfolioManifestsJson;
          portfolio-manifest-plinth = portfolioManifestFiles.plinth;
        };

        apps.default = flake-utils.lib.mkApp {
          name = "plinth-server";
          drv = plinth;
        };
        apps.deploy-pages = projectSiteLib.mkDeployPagesApp {
          domain = "plinth.tartanoglu.com";
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from the build
          inputsFrom = [plinth];

          # Extra inputs for development
          packages =
            [
              # cargo-leptos for development server with hot reload
              pkgs.cargo-leptos
              # Tailwind CSS for styling
              pkgs.tailwindcss
              # Postgres with pgvector for local development
              postgresqlWithPgvector
              pkgs.sqlx-cli
              # wasm-bindgen-cli
              wasm-bindgen-cli
              # OpenSSL for reqwest/other crates
              pkgs.pkg-config
              pkgs.openssl
              # libclang for bindgen-based dependencies
              pkgs.llvmPackages.libclang.lib
              # mdBook for documentation development
              pkgs.mdbook
              plinth-project
              # Node.js + Chromium for Playwright E2E tests
              pkgs.nodejs
              pkgs.chromium
            ]
            ++ lib.optionals pkgs.stdenv.isLinux [
              # Mold linker for faster linking
              pkgs.mold
              pkgs.clang
            ];

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          CHROMIUM_PATH = "${pkgs.chromium}/bin/chromium";
          PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = "1";

          # No need for CLIENT_DIST with cargo-leptos
          shellHook = ''
            export PGDATA="$PWD/.dev-pgdata"
            export PGHOST="$PWD/.dev-pgsocket"
            export DATABASE_URL="postgres://localhost/plinth?host=$PGHOST"

            echo "Leptos development environment loaded"
            echo "Run: cargo leptos watch"
            echo "Local Postgres: ./scripts/dev-db.sh start|stop|reset"
            echo ""
            echo "Documentation: cd docs && mdbook serve"
          '';
        };
      }
    );
}
