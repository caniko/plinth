{
  description = "Plinth - personal website with Dioxus SSR";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    nix-pklx = {
      url = "git+https://codeberg.org/caniko/nix-pklx.git";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.crane.follows = "crane";
      inputs.flake-utils.follows = "flake-utils";
      inputs.rust-overlay.follows = "rust-overlay";
    };

    rs-harbor = {
      url = "git+https://codeberg.org/caniko/rs-harbor.git?ref=trunk&rev=9bfa8bdb0ecb22d7bc11448665f7fbaebae7a759";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.crane.follows = "crane";
      inputs.rust-overlay.follows = "rust-overlay";
    };
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    rust-overlay,
    nix-pklx,
    rs-harbor,
    ...
  }: let
    topPkgs = import nixpkgs {
      system = "x86_64-linux";
      overlays = [(import rust-overlay)];
    };
    topProjectSiteLib = import ./nix/project-site.nix {
      pkgs = topPkgs;
      lib = nixpkgs.lib;
    };
    topSiteChecksLib = import ./nix/site-checks.nix {
      pkgs = topPkgs;
      lib = nixpkgs.lib;
    };
    topVisualAuditLib = import ./nix/visual-audit.nix {
      pkgs = topPkgs;
      lib = nixpkgs.lib;
    };
    topPlinthProjects = topProjectSiteLib.mkProjectRegistry {
      plinth = topProjectSiteLib.mkProjectDefinition {
        id = "plinth";
        pname = "plinth-site";
        title = "Plinth";
        description = "A self-hosted personal website platform built with Dioxus, Postgres, semantic search, and Nix.";
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
        techStack = ["Rust" "Dioxus" "Postgres" "Nix"];
      };
    };
    topProjectReferences = nixpkgs.lib.mapAttrs (_: topProjectSiteLib.projectReferenceFromDefinition) topPlinthProjects;
    topPortfolioManifests = nixpkgs.lib.mapAttrs (_: topProjectSiteLib.portfolioManifestFromDefinition) topPlinthProjects;

    # Read the checked-in producer output.  Evaluation must not invoke Pkl or
    # import a derivation; generated-data-check below verifies freshness.
    portfolioFromPkl = import ./website/portfolio.generated.nix;
    portfolioItems = portfolioFromPkl.portfolio or [];

    # Reusable lib attr for both top-level and per-system exposure
    plinthLib =
      topProjectSiteLib
      // {
      plinthProjects = topPlinthProjects;
      projectReferences = topProjectReferences;
      portfolioManifests = topPortfolioManifests;
      siteChecks = topSiteChecksLib;
      visualAudit = topVisualAuditLib;
      inherit portfolioFromPkl portfolioItems;
    };
  in
    {
      lib = plinthLib;

      # NixOS module for declarative deployment
      nixosModules.default = import ./modules/plinth.nix;
      nixosModules.plinth = import ./modules/plinth.nix;
      nixosModules.site-checks = import ./modules/site-checks-nixos.nix;

      homeModules.site-checks = import ./modules/site-checks-home.nix;

      # Overlay for downstream users to access pre-built packages
      # For custom builds, import the flake and use buildPlinth directly
      overlays.default = final: prev: {
        inherit
          (self.packages.${final.system})
          plinth
          plinth-csr
          plinth-cli
          plinth-person
          plinth-project
          pcomfy
          plinth-dev
          plinth-minimal
          ;
        plinth-dioxus-helper = self.packages.${final.system}.plinth-dioxus-helper;
      };
      crossPackages."x86_64-linux"."aarch64-linux".plinth = self.packages."x86_64-linux"."plinth-aarch64-linux";
    }
    # Nixpkgs 26.11 dropped x86_64-darwin; do not evaluate that unsupported
    # package set while consumers evaluate this flake on Linux.
    // flake-utils.lib.eachSystem (builtins.filter (system: system != "x86_64-darwin") flake-utils.lib.defaultSystems) (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };

        inherit (pkgs) lib;

        # Build versions have one producer each:
        #   rust-toolchain.toml -> Rust channel/components/targets
        #   Cargo.lock -> Dioxus and wasm-bindgen versions
        #   rs-harbor -> compiler-cache policy and sccache client
        # Keep the Nix builders derived from those producers so local Cargo,
        # native Nix, Dioxus, and Crossbow cannot silently drift apart.
        rustToolchainSpec = builtins.fromTOML (builtins.readFile ./rust-toolchain.toml);
        rustToolchainConfig = rustToolchainSpec.toolchain;
        cargoLockSpec = builtins.fromTOML (builtins.readFile ./Cargo.lock);
        cargoPackageVersion = name: let
          matches = lib.filter (package: package.name == name) cargoLockSpec.package;
        in
          if matches == []
          then throw "Cargo.lock has no package named ${name}"
          else (builtins.head matches).version;
        dioxusVersion = cargoPackageVersion "dioxus";
        wasmBindgenVersion = cargoPackageVersion "wasm-bindgen";
        rustChannel = rustToolchainConfig.channel;
        rustDate = lib.removePrefix "nightly-" rustChannel;

        # rs-harbor owns the compiler-cache executable, wrapper, namespace,
        # and sandbox admission policy.  The product only selects the shared
        # fleet namespace; Atlas supplies the writable mount at build time.
        sccachePackage = rs-harbor.packages.${system}.sccache;
        buildCache = rs-harbor.lib.mkBuildCachePolicy {
          inherit pkgs sccachePackage;
          buildPackageSet = pkgs.buildPackages;
          namespaceScope = "canix-rust";
          namespaceGeneration = 5;
        };

        dioxusCliContractAssertion =
          lib.assertMsg
          (pkgs.dioxus-cli.version == dioxusVersion)
          "Dioxus CLI drift: Cargo.lock requires ${dioxusVersion}, nixpkgs provides ${pkgs.dioxus-cli.version}";

        wasm-bindgen-cli = pkgs.buildWasmBindgenCli {
          version = wasmBindgenVersion;
          src = pkgs.fetchCrate {
            pname = "wasm-bindgen-cli";
            version = wasmBindgenVersion;
            hash = "sha256-H6Is3fiZVxZCfOMWK5dWMSrtn50VGv0sfdnsT+cTtyk=";
          };
          cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
            name = "wasm-bindgen-cli-${wasmBindgenVersion}-vendor";
            src = pkgs.fetchCrate {
              pname = "wasm-bindgen-cli";
              version = wasmBindgenVersion;
              hash = "sha256-H6Is3fiZVxZCfOMWK5dWMSrtn50VGv0sfdnsT+cTtyk=";
            };
            hash = "sha256-VucqkXbCi4qtQzY/HrXiDnbSURsagPsdNVMn1Tw3UiY=";
          };
        };

        rustToolchainFor = p:
          p.rust-bin.nightly.${rustDate}.default.override {
            extensions = rustToolchainConfig.components;
            targets = rustToolchainConfig.targets;
          };
        canonicalNativeRustFlags = lib.concatStringsSep " " (lib.filter (flag: flag != "") [
          (lib.optionalString pkgs.stdenv.isLinux "-C link-arg=-fuse-ld=mold")
          "-Zshare-generics=y"
        ]);
        canonicalNativeLinkerConfig = let
          target = pkgs.stdenv.hostPlatform.rust.rustcTarget;
          targetUpper = lib.toUpper (lib.replaceStrings ["-"] ["_"] target);
        in
          {
            "CARGO_TARGET_${targetUpper}_RUSTFLAGS" = canonicalNativeRustFlags;
          }
          // lib.optionalAttrs (pkgs.stdenv.isLinux) {
            "CARGO_TARGET_${targetUpper}_LINKER" = "${pkgs.clang}/bin/clang";
          };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchainFor;
        cross = rs-harbor.lib.mkCross {
          inherit pkgs system;
          enableOsxcross = false;
        };
        postgresqlWithPgvector = pkgs.postgresql_17.withPackages (ps: [ps.pgvector]);

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
              dioxusEnv = "DEV";
              rustOptLevel = "0";
            }
            else if profile == "minimal"
            then {
              cargoProfile = "release";
              dioxusEnv = "PROD";
              rustOptLevel = "z"; # Optimize for size
            }
            else {
              # prod profile (default)
              cargoProfile = "release";
              dioxusEnv = "PROD";
              rustOptLevel = "3";
            };

          # Build RUSTFLAGS based on configuration
          rustFlags = lib.concatStringsSep " " (
              lib.filter (s: s != "") [
                (lib.optionalString enableMold "-C link-arg=-fuse-ld=mold")
                "-Zshare-generics=y"
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
              ${rustflagsEnvVar} = lib.concatStringsSep " " (lib.filter (s: s != "") ["-Zshare-generics=y" extraRustFlags]);
            };
        in
          craneLib.buildPackage (commonArgs
            // linkerConfig
            // {
              pname = "plinth";
              cargoArtifacts =
                if profileSettings.cargoProfile == "dev"
                then cargoArtifactsDev
                else if profile == "minimal"
                then cargoArtifactsMinimal
                else cargoArtifactsRelease;

              # Build the Dioxus server and browser targets explicitly.  Keeping
              # these commands visible makes the WASM feature graph and the
              # wasm-bindgen ABI pin auditable in CI.
              buildPhaseCargoCommand = ''
                cargo build --locked --package plinth-web --bin plinth-web --no-default-features --features server,brick-blog,brick-portfolio,brick-todo,brick-activity ${cargoProfileFlag}
                cargo build --locked --package plinth-cli --bin plinth ${cargoProfileFlag}
                cargo build --locked --package plinth-web --bin plinth-web --target wasm32-unknown-unknown --no-default-features --features web,brick-blog,brick-portfolio,brick-todo,brick-activity ${cargoProfileFlag}
                mkdir -p target/site/pkg
                wasm-bindgen target/wasm32-unknown-unknown/${
                  if profileSettings.cargoProfile == "dev"
                  then "debug"
                  else "release"
                }/plinth-web.wasm --target web --out-dir target/site/pkg --out-name plinth
                tailwindcss --input input.css --output target/site/plinth.css --minify
                cp -r public/* target/site/
              '';

              # Tests run in the dedicated plinth-test check, which starts
              # Postgres for SQLx integration tests.
              doCheck = false;

              doNotPostBuildInstallCargoBinaries = true;

              # Set optimization level for WASM
              ${
                if profileSettings.cargoProfile == "dev"
                then "CARGO_PROFILE_DEV_OPT_LEVEL"
                else "CARGO_PROFILE_RELEASE_OPT_LEVEL"
              } =
                profileSettings.rustOptLevel;
          DIOXUS_ENV = profileSettings.dioxusEnv;

              # Install the server binary and site assets with wrapper script
              installPhase = ''
                mkdir -p $out/bin
                mkdir -p $out/site

                # Determine binary path based on profile
                if [ "${profileSettings.cargoProfile}" = "dev" ]; then
                  binaryPath="target/debug/plinth-web"
                else
                  binaryPath="target/release/plinth-web"
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

                # Create the compatibility service name used by existing NixOS
                # modules while the executable itself is now Dioxus-owned.
                cat > $out/bin/plinth-server <<EOF
                #!/bin/sh
                export DIOXUS_PUBLIC_PATH="\''${DIOXUS_PUBLIC_PATH:-$out/site}"
                if [ -z "\''${PLINTH_RENDER_CACHE_DIR:-}" ] && [ -n "\''${STATE_DIRECTORY:-}" ]; then
                  # Namespace rendered HTML by the package profile. A
                  # deployment may override this with a revision-specific
                  # path through PLINTH_RENDER_CACHE_DIR.
                  export PLINTH_RENDER_CACHE_DIR="\''${STATE_DIRECTORY}/render-cache/plinth-${profileSettings.cargoProfile}"
                fi
                exec $out/bin/plinth-server-unwrapped "\$@"
                EOF
                chmod +x $out/bin/plinth-server

                # Copy site assets (WASM, JS, CSS, and public files).
                cp -r target/site/* $out/site/

                # Install example configuration
                mkdir -p $out/share/plinth
                if [ -f plinth.toml ]; then
                  cp plinth.toml $out/share/plinth/plinth.toml
                fi
              '';
            });

        # When filtering sources, include all files needed for Dioxus + Tailwind.
        unfilteredRoot = ./.; # The original, unfiltered source
        src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            # Default files from crane (Rust and cargo files)
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            # Workspace members
            (lib.fileset.maybeMissing ./crates/client)
            (lib.fileset.maybeMissing ./crates/dioxus-ui)
            (lib.fileset.maybeMissing ./crates/server)
            (lib.fileset.maybeMissing ./crates/shared)
            (lib.fileset.maybeMissing ./crates/cli)
            (lib.fileset.maybeMissing ./crates/forge)
            (lib.fileset.maybeMissing ./crates/person)
            (lib.fileset.maybeMissing ./crates/project)
            (lib.fileset.maybeMissing ./crates/pcomfy)
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
            (lib.fileset.maybeMissing ./Dioxus.toml)
            # Static CSR shell
            (lib.fileset.maybeMissing ./csr)
            # Default configuration file
            (lib.fileset.maybeMissing ./plinth.toml)
            # Canonical local toolchain contract
            (lib.fileset.maybeMissing ./rust-toolchain.toml)
            # Development helper scripts used by Nix checks
            (lib.fileset.maybeMissing ./scripts)
          ];
        };

        # Common arguments for all builds
        commonArgs =
          {
          inherit src;
          strictDeps = true;
            # The wrapper and environment come from rs-harbor.  This keeps
            # dependency builds and final packages on the same cache contract.
            inherit (buildCache.rustEnv) RUSTC_WRAPPER CARGO_INCREMENTAL;

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
              # Dioxus CLI is used for local fullstack routing and asset checks;
              # release builds still use the explicit commands above.
              pkgs.dioxus-cli
              # Tailwind CSS standalone binary (v4 with native @plugin support)
              pkgs.tailwindcss_4
              # wasm-bindgen-cli version must match Cargo.lock
              wasm-bindgen-cli
              # libclang needed by bindgen-based dependencies
              pkgs.llvmPackages.libclang.lib
              # pkg-config needed by openssl-sys
              pkgs.pkg-config
              # wasm-opt for optimizing WASM output
              pkgs.binaryen
              # Compiler-object cache wrapper supplied by rs-harbor. Atlas
              # supplies SCCACHE_DIR through the Nix daemon; local builds use
              # the wrapper's sandbox fallback.
              buildCache.wrapper
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

            # Keep the dependency artifact under the same target-specific flags
            # used by native package/check derivations. Cargo fingerprints these
            # flags, so applying them after buildDepsOnly defeats reuse.
          }
          // canonicalNativeLinkerConfig;

        crossPlinthArgs =
          commonArgs
          // {
          buildPhaseCargoCommand = ''
            cargo build --locked --package plinth-web --bin plinth-web --no-default-features --features server,brick-blog,brick-portfolio,brick-todo,brick-activity --release
            cargo build --locked --package plinth-cli --bin plinth --release
            cargo build --locked --package plinth-web --bin plinth-web --target wasm32-unknown-unknown --no-default-features --features web,brick-blog,brick-portfolio,brick-todo,brick-activity --release
            if [ "\''\${CRANE_BUILD_DEPS_ONLY:-0}" != "1" ]; then
              mkdir -p target/site/pkg
              wasm-bindgen target/wasm32-unknown-unknown/release/plinth-web.wasm --target web --out-dir target/site/pkg --out-name plinth
              tailwindcss --input input.css --output target/site/plinth.css --minify
              cp -r public/* target/site/
            fi
          '';
          doCheck = false;
          doNotPostBuildInstallCargoBinaries = true;
          CARGO_PROFILE_RELEASE_OPT_LEVEL = "3";
          DIOXUS_ENV = "PROD";
            CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS = "-Zshare-generics=y";
          installPhase = ''
            mkdir -p $out/bin
            mkdir -p $out/site
            cp target/release/plinth-web $out/bin/plinth-server-unwrapped
            cp target/release/plinth $out/bin/plinth
            cat > $out/bin/plinth-server <<EOF
            #!/bin/sh
            export DIOXUS_PUBLIC_PATH="\''${DIOXUS_PUBLIC_PATH:-$out/site}"
            if [ -z "\''${PLINTH_RENDER_CACHE_DIR:-}" ] && [ -n "\''${STATE_DIRECTORY:-}" ]; then
              # Namespace rendered HTML by the package profile. A deployment
              # may override this with a revision-specific path.
              export PLINTH_RENDER_CACHE_DIR="\''${STATE_DIRECTORY}/render-cache/plinth-release"
            fi
            exec $out/bin/plinth-server-unwrapped "\$@"
            EOF
            chmod +x $out/bin/plinth-server
            cp -r target/site/* $out/site/
            mkdir -p $out/share/plinth
            if [ -f plinth.toml ]; then
              cp plinth.toml $out/share/plinth/plinth.toml
            fi
          '';
        };
        crossPackageSet = rs-harbor.lib.mkCrossPackages ({
          inherit pkgs craneLib cross;
          pname = "plinth";
          commonArgs = crossPlinthArgs;
          targets = ["native" "aarch64-linux"];
          targetArgs."aarch64-linux" = {
            buildInputs = [
              cross.linuxAarch64.pkgsCross.openssl
              # OpenVINO is an optional execution provider for ONNX Runtime.
              # It pulls a large native C++/OpenBLAS closure and currently
              # enables a missing ARMV9SME kernel when Crossbow prepares the
              # aarch64 target under QEMU.  Fastembed uses ONNX Runtime's CPU
              # provider, so keep the target closure portable and omit the
              # optional provider for production cross builds.
              (cross.linuxAarch64.pkgsCross.onnxruntime.override {
                openvinoSupport = false;
              })
            ];
          };
          }
          // lib.optionalAttrs (builtins.hasAttr "toolchainArgs" (builtins.functionArgs rs-harbor.lib.mkCrossPackages)) {
          toolchainArgs = {
              channel = lib.head (lib.splitString "-" rustChannel);
              date = rustDate;
              extensions = rustToolchainConfig.components;
              crossTargets = rustToolchainConfig.targets;
          };
        });

        # Linker configuration for cargo test and other non-build checks uses
        # the same target-specific flags as buildDepsOnly.
        baseLinkerConfig = canonicalNativeLinkerConfig;

        # Build each Cargo compilation class against matching dependency
        # artifacts. Sharing a release deps derivation with a debug or
        # size-optimized build changes Cargo's profile fingerprints and
        # causes the entire dependency graph to compile again under a
        # different key.
        cargoArtifactsFor = {
          profile,
          optLevel,
        }:
          craneLib.buildDepsOnly (commonArgs
          // {
              pname = "plinth-${profile}-deps";
              ${
                if profile == "dev"
                then "CARGO_PROFILE"
                else "CARGO_PROFILE_RELEASE_OPT_LEVEL"
              } =
                if profile == "dev"
                then profile
                else optLevel;
          });
        cargoArtifactsRelease = cargoArtifactsFor {
          profile = "release";
          optLevel = "3";
        };
        cargoArtifactsMinimal = cargoArtifactsFor {
          profile = "minimal";
          optLevel = "z";
        };
        cargoArtifactsDev = cargoArtifactsFor {
          profile = "dev";
          optLevel = "0";
        };
        # All release checks and single-binary packages use this canonical
        # release class; profile-specific bundles select their own above.
        cargoArtifacts = cargoArtifactsRelease;

        # Canonical Dioxus fullstack bundle. rs-harbor owns the offline
        # Dioxus/Cargo/WASM mechanics; Plinth keeps its Tailwind pipeline and
        # runtime inputs in this product flake.
        plinth-dioxus-helper = rs-harbor.lib.mkDioxusFullstackPackage ({
          inherit pkgs src craneLib;
          cargoLock = ./Cargo.lock;
          pname = "plinth-dioxus-helper";
          package = "plinth-web";
          binary = "plinth-web";
          serverInstallName = "plinth-server";
          serverBinary = "server";
          wrapServer = false;
          rustToolchain = rustToolchainFor pkgs;
          inherit wasm-bindgen-cli;
          profile = "release";
          debugSymbols = false;
          noDefaultFeatures = true;
          webFeatures = ["web" "brick-blog" "brick-portfolio" "brick-todo" "brick-activity"];
          serverFeatures = ["server" "brick-blog" "brick-portfolio" "brick-todo" "brick-activity"];
          publicSubdir = "site";
          strictDeps = true;
          inherit buildCache;
          buildInputs = commonArgs.buildInputs;
          nativeBuildInputs = [pkgs.tailwindcss_4];
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          ORT_LIB_LOCATION = "${pkgs.onnxruntime}/lib";
          ORT_PREFER_DYNAMIC_LINK = "1";
          CARGO_PROFILE_RELEASE_OPT_LEVEL = "3";
          DIOXUS_ENV = "PROD";
          postBuild = ''
            tailwindcss --input input.css --output "$TMPDIR/dioxus-out/public/plinth.css" --minify
          '';
          postInstall = ''
            mv "$out/bin/plinth-server" "$out/bin/plinth-server-unwrapped"
            cat > "$out/bin/plinth-server" <<EOF
            #!/bin/sh
            export DIOXUS_PUBLIC_PATH="\''${DIOXUS_PUBLIC_PATH:-$out/site}"
            if [ -z "\''${PLINTH_RENDER_CACHE_DIR:-}" ] && [ -n "\''${STATE_DIRECTORY:-}" ]; then
              export PLINTH_RENDER_CACHE_DIR="\''${STATE_DIRECTORY}/render-cache/plinth-release"
            fi
            exec "$out/bin/plinth-server-unwrapped" "\$@"
            EOF
            chmod +x "$out/bin/plinth-server"
            mkdir -p "$out/share/plinth"
            if [ -f plinth.toml ]; then
              cp plinth.toml "$out/share/plinth/plinth.toml"
            fi
          '';
          }
          // canonicalNativeLinkerConfig);

        # Build variants using the parameterized function. Production now
        # composes the shared rs-harbor Dioxus bundle with the product CLI;
        # the legacy parameterized builder remains for dev/minimal rollback
        # profiles until their independent cutover canaries are complete.
        plinth = pkgs.symlinkJoin {
          name = "plinth";
          paths = [plinth-dioxus-helper plinth-cli];
        };
        plinth-dev = buildPlinth {profile = "dev";};
        plinth-minimal = buildPlinth {profile = "minimal";};

        plinth-csr = craneLib.buildPackage (commonArgs
          // {
            pname = "plinth-csr";
            inherit cargoArtifacts;

            buildPhaseCargoCommand = ''
              cargo build --locked --package plinth-web --bin plinth-web \
                --target wasm32-unknown-unknown \
                --no-default-features \
                --features web,brick-blog,brick-portfolio,brick-todo,brick-activity \
                --release
            '';

            doCheck = false;
            doNotPostBuildInstallCargoBinaries = true;

            installPhase = ''
              mkdir -p $out/pkg

              wasm-bindgen \
                target/wasm32-unknown-unknown/release/plinth-web.wasm \
                --target web \
                --out-dir $out/pkg \
                --out-name plinth

              tailwindcss \
                --input input.css \
                --output $out/plinth.css \
                --minify

              cp public/index.html $out/index.html
              cp -r public/* $out/
            '';
          });

        plinth-cli = craneLib.buildPackage (commonArgs
          // baseLinkerConfig
          // {
            inherit cargoArtifacts;
            pname = "plinth-cli";
            cargoExtraArgs = "--locked --package plinth-cli --bin plinth";
            doCheck = false;
            postInstall = ''
              ln -s $out/bin/plinth $out/bin/plinth-cli
            '';
          });

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

        pcomfy = craneLib.buildPackage (commonArgs
          // baseLinkerConfig
          // {
            inherit cargoArtifacts;
            pname = "pcomfy";
            cargoExtraArgs = "--locked --package pcomfy --bin pcomfy";
            doCheck = false;
          });
        projectSiteLib = import ./nix/project-site.nix {
          inherit pkgs lib;
          plinthProject = plinth-project;
        };
        siteChecksLib = import ./nix/site-checks.nix {
          inherit pkgs lib;
        };
        visualAuditLib = import ./nix/visual-audit.nix {
          inherit pkgs lib;
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
            description = "A self-hosted personal website platform built with Dioxus, Postgres, semantic search, and Nix.";
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
            techStack = ["Rust" "Dioxus" "Postgres" "Nix"];
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

        siteChecksModuleMarkers = let
          target = {
            id = "example";
            title = "Example";
            url = "https://example.com";
            kind = "static";
            markers = ["Example"];
          };
          stubHomeOptions = {lib, ...}: {
            options = {
              home.packages = lib.mkOption {
                type = lib.types.listOf lib.types.package;
                default = [];
              };
              home.sessionVariables = lib.mkOption {
                type = lib.types.attrsOf lib.types.str;
                default = {};
              };
              xdg.configFile = lib.mkOption {
                type = lib.types.attrsOf (lib.types.submodule {
                  options.source = lib.mkOption {
                    type = lib.types.path;
                  };
                });
                default = {};
              };
              assertions = lib.mkOption {
                type = lib.types.listOf lib.types.attrs;
                default = [];
              };
            };
          };
          stubNixosOptions = {lib, ...}: {
            options = {
              environment.systemPackages = lib.mkOption {
                type = lib.types.listOf lib.types.package;
                default = [];
              };
              environment.sessionVariables = lib.mkOption {
                type = lib.types.attrsOf lib.types.str;
                default = {};
              };
              environment.etc = lib.mkOption {
                type = lib.types.attrsOf (lib.types.submodule {
                  options.source = lib.mkOption {
                    type = lib.types.path;
                  };
                });
                default = {};
              };
              assertions = lib.mkOption {
                type = lib.types.listOf lib.types.attrs;
                default = [];
              };
            };
          };
          homeEval = lib.evalModules {
            specialArgs = {inherit pkgs;};
            modules = [
              stubHomeOptions
              self.homeModules.site-checks
              {
                programs.plinth.siteChecks = {
                  enable = true;
                  package = plinth-cli;
                  targets = [target];
                };
              }
            ];
          };
          nixosEval = lib.evalModules {
            specialArgs = {inherit pkgs;};
            modules = [
              stubNixosOptions
              self.nixosModules.site-checks
              {
                services.plinth.siteChecks = {
                  enable = true;
                  package = plinth-cli;
                  targets = [target];
                };
              }
            ];
          };
          homeConfig = homeEval.config.xdg.configFile."plinth/site-checks.toml".source;
          nixosConfig = nixosEval.config.environment.etc."plinth/site-checks.toml".source;
        in
          pkgs.runCommand "plinth-site-checks-module-markers" {} ''
            grep -q 'id = "example"' ${homeConfig}
            grep -q 'kind = "static"' ${homeConfig}
            grep -q 'expected_status = 200' ${homeConfig}
            grep -q 'id = "example"' ${nixosConfig}
            grep -q 'kind = "static"' ${nixosConfig}
            grep -q 'expected_status = 200' ${nixosConfig}
            touch $out
          '';

        visualAuditPklFixture = builtins.fromJSON (builtins.readFile ./pkl/VisualAudit.fixture.json);
        visualAuditTargets = visualAuditLib.targetsFromPkl visualAuditPklFixture;
        visualAuditHelperMarkers = visualAuditLib.mkTargetsMarkerCheck {
          name = "plinth-visual-audit-helper-markers";
          targets = visualAuditTargets;
          expectedIds = ["project" "personal"];
        };

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
      in
        assert dioxusCliContractAssertion; {
          lib =
            projectSiteLib
            // {
          plinthProjects = topPlinthProjects;
          projectReferences = topProjectReferences;
          portfolioManifests = topPortfolioManifests;
          siteChecks = topSiteChecksLib;
          visualAudit = visualAuditLib;
          portfolioFromPkl = plinthLib.portfolioFromPkl;
          portfolioItems = plinthLib.portfolioItems;
        };
        checks = {
          # Build the app as part of `nix flake check` for convenience
          inherit plinth plinth-csr;

            generated-data-check =
              pkgs.runCommand "plinth-generated-data-check" {
                nativeBuildInputs = [
                  nix-pklx.packages.${system}.pklx
                  pkgs.pkl
                ];
                # pklx constructs a reqwest client even for local
                # evaluation; provide the sandbox CA bundle so reqwest does
                # not panic before the offline producer runs.
                SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
                NIX_SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
                portfolioSource = ./website/portfolio.pkl;
                portfolioGenerated = ./website/portfolio.generated.nix;
                visualAuditSource = ./pkl/VisualAudit.fixture.pkl;
                visualAuditModule = ./pkl/VisualAudit.pkl;
                visualAuditGenerated = ./pkl/VisualAudit.fixture.json;
              } ''
                pklx eval "$portfolioSource" > "$TMPDIR/portfolio.generated.nix"
                tail -n +2 "$portfolioGenerated" > "$TMPDIR/portfolio.expected.nix"
                cmp -s "$TMPDIR/portfolio.generated.nix" "$TMPDIR/portfolio.expected.nix"
                cp "$visualAuditModule" "$TMPDIR/VisualAudit.pkl"
                cp "$visualAuditSource" "$TMPDIR/VisualAudit.fixture.pkl"
                (cd "$TMPDIR" && pkl eval -f json VisualAudit.fixture.pkl) > "$TMPDIR/VisualAudit.fixture.json"
                cmp -s "$TMPDIR/VisualAudit.fixture.json" "$visualAuditGenerated"
                touch "$out"
              '';

            build-contract-version-check = pkgs.runCommand "plinth-build-contract-version-check" {} ''
              test "${pkgs.dioxus-cli.version}" = "${dioxusVersion}"
              test "${wasmBindgenVersion}" = "${wasm-bindgen-cli.version}"
              test "${rustChannel}" = "${rustToolchainConfig.channel}"
              touch "$out"
            '';

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
                export DATABASE_URL="postgres://$(id -un)@localhost/plinth?host=$PGHOST"
                # Site-check unit tests use loopback wiremock servers.  Keep
                # them local even when the builder environment provides an
                # outbound HTTP proxy.
                export NO_PROXY="''${NO_PROXY:-},127.0.0.1,localhost"
                export no_proxy="$NO_PROXY"
                unset HTTP_PROXY HTTPS_PROXY ALL_PROXY http_proxy https_proxy all_proxy

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
          site-checks-modules = siteChecksModuleMarkers;
          visual-audit-helper-markers = visualAuditHelperMarkers;
        };

          formatter = pkgs.alejandra;

        packages = {
          default = plinth;
          inherit plinth plinth-csr plinth-cli plinth-person plinth-project pcomfy plinth-dev plinth-minimal plinth-dioxus-helper;
          "plinth-aarch64-linux" = crossPackageSet."plinth-aarch64-linux";
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

          devShells.codegen = pkgs.mkShell {
            packages = [
              nix-pklx.packages.${system}.pklx
              pkgs.pkl
            ];
          };

          devShells.default = craneLib.devShell {
            # Keep the shell independent from release derivations.  Pulling
            # `plinth` through inputsFrom realizes the entire server/WASM
            # closure before an interactive shell (and before generators
            # such as the checked-in Pkl artifacts) can start.
          packages =
            [
              # Dioxus development server with hot reload
              pkgs.dioxus-cli
              # Tailwind CSS for styling (v4)
              pkgs.tailwindcss_4
              # Postgres with pgvector for local development
              postgresqlWithPgvector
              pkgs.sqlx-cli
              # wasm-bindgen-cli
              wasm-bindgen-cli
              # Shared rs-harbor compiler-cache wrapper for interactive builds.
              buildCache.wrapper
              # OpenSSL for reqwest/other crates
              pkgs.pkg-config
              pkgs.openssl
                pkgs.onnxruntime
              # libclang for bindgen-based dependencies
              pkgs.llvmPackages.libclang.lib
              # mdBook for documentation development
              pkgs.mdbook
              plinth-project
              # nix-pklx for Pkl-based portfolio evaluation
              nix-pklx.packages.${system}.pklx
                # Pkl compiler for the checked-in visual-audit fixture
                pkgs.pkl
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
            ORT_LIB_LOCATION = "${pkgs.onnxruntime}/lib";
            ORT_PREFER_DYNAMIC_LINK = "1";
          CHROMIUM_PATH = "${pkgs.chromium}/bin/chromium";
          PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = "1";

          shellHook = ''
            export RUSTC_WRAPPER="${buildCache.wrapper}/bin/rs-harbor-sandbox-sccache"
            export SCCACHE_DIR="''${SCCACHE_DIR:-$PWD/.cache/sccache}"
            export CARGO_INCREMENTAL=0
            export PGDATA="$PWD/.dev-pgdata"
            export PGHOST="$PWD/.dev-pgsocket"
            export DATABASE_URL="postgres://$(id -un)@localhost/plinth?host=$PGHOST"

            echo "Dioxus development environment loaded"
            echo "Run: dx serve --web --fullstack"
            echo "Local Postgres: ./scripts/dev-db.sh start|stop|reset"
            echo ""
            echo "Documentation: cd docs && mdbook serve"
          '';
        };
      }
    );
}
