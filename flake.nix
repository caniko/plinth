{
  description = "Leptos personal website with SSR";

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
    {
      # NixOS module for declarative deployment
      nixosModules.default = import ./modules/personal-website.nix;
      nixosModules.personal-website = import ./modules/personal-website.nix;

      # Overlay for downstream users to access pre-built packages
      # For custom builds, import the flake and use buildPersonalWebsite directly
      overlays.default = final: prev: {
        inherit (self.packages.${final.system})
          personal-website
          personal-website-dev
          personal-website-minimal;
      };
    }
    // flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };

        inherit (pkgs) lib;

        rustToolchainFor = p:
          p.rust-bin.nightly.latest.default.override {
            # Set the build targets supported by the toolchain
            # wasm32-unknown-unknown is required for Leptos client-side code
            # Using nightly for -Zshare-generics=y flag
            extensions = ["rust-src" "rust-analyzer" "rustfmt" "rustc-codegen-cranelift-preview"];
            targets = ["wasm32-unknown-unknown"];
          };
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchainFor;

        # Parameterized build function for configurability
        buildPersonalWebsite = {
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
          linkerEnvVar = "CARGO_TARGET_${lib.toUpper (lib.replaceStrings ["-"] ["_"] rustTarget)}_LINKER";
          linkerConfig =
            if pkgs.stdenv.isLinux && enableMold
            then {${linkerEnvVar} = "${pkgs.clang}/bin/clang";}
            else {};
        in
          craneLib.buildPackage (commonArgs
            // linkerConfig
            // {
              pname = "personal-website";
              inherit cargoArtifacts;

              # cargo-leptos builds from the server directory
              preBuild = ''
                cd server
              '';

              # Use cargo-leptos to build with appropriate profile
              buildPhaseCargoCommand = "cargo leptos build ${cargoProfileFlag}";

              # Override RUSTFLAGS with profile-specific flags
              RUSTFLAGS =
                if enableMold && pkgs.stdenv.isLinux
                then rustFlags
                else lib.concatStringsSep " " (lib.filter (s: s != "") ["-Zshare-generics=y" extraRustFlags]);

              # Set optimization level for WASM
              CARGO_PROFILE_RELEASE_OPT_LEVEL = profileSettings.rustOptLevel;
              LEPTOS_ENV = profileSettings.leptosEnv;

              # Install the server binary and site assets with wrapper script
              installPhase = ''
                mkdir -p $out/bin
                mkdir -p $out/site

                # Determine binary path based on profile
                if [ "${profileSettings.cargoProfile}" = "dev" ]; then
                  binaryPath="target/debug/server"
                else
                  binaryPath="target/release/server"
                fi

                # Copy server binary
                cp $binaryPath $out/bin/server-unwrapped

                # Create wrapper script that sets LEPTOS_SITE_ROOT
                cat > $out/bin/server <<EOF
                #!/bin/sh
                export LEPTOS_SITE_ROOT="\''${LEPTOS_SITE_ROOT:-$out/site}"
                exec $out/bin/server-unwrapped "\$@"
                EOF
                chmod +x $out/bin/server

                # Copy site assets (includes WASM, JS, CSS)
                cp -r target/site/* $out/site/
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
            (lib.fileset.maybeMissing ./client)
            (lib.fileset.maybeMissing ./server)
            (lib.fileset.maybeMissing ./shared)
            (lib.fileset.maybeMissing ./cli)
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
          ];
        };

        # Common arguments for all builds
        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs =
            [
              # Add additional build inputs here
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
              pkgs.wasm-bindgen-cli
            ]
            ++ lib.optionals pkgs.stdenv.isLinux [
              # Mold linker for faster linking on Linux
              pkgs.mold
              pkgs.clang
            ];

          # Note: RUSTFLAGS and linker config are set in buildPersonalWebsite
        };

        # Build cargo dependencies separately for caching
        cargoArtifacts = craneLib.buildDepsOnly (commonArgs
          // {
            pname = "personal-website-deps";
          });

        # Build variants using the parameterized function
        my-app = buildPersonalWebsite {}; # Default production build
        my-app-dev = buildPersonalWebsite {profile = "dev";};
        my-app-minimal = buildPersonalWebsite {profile = "minimal";};
      in {
        checks = {
          # Build the app as part of `nix flake check` for convenience
          inherit my-app;

          # Run clippy (and deny all warnings) on the crate source
          my-app-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          # Check formatting
          my-app-fmt = craneLib.cargoFmt commonArgs;
        };

        packages = {
          default = my-app;
          personal-website = my-app;
          personal-website-dev = my-app-dev;
          personal-website-minimal = my-app-minimal;
        };

        apps.default = flake-utils.lib.mkApp {
          name = "server";
          drv = my-app;
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from the build
          inputsFrom = [my-app];

          # Extra inputs for development
          packages =
            [
              # cargo-leptos for development server with hot reload
              pkgs.cargo-leptos
              # Tailwind CSS for styling
              pkgs.tailwindcss
              # SurrealDB for database (will be used in Phase 2)
              pkgs.surrealdb
              # wasm-bindgen-cli
              pkgs.wasm-bindgen-cli
            ]
            ++ lib.optionals pkgs.stdenv.isLinux [
              # Mold linker for faster linking
              pkgs.mold
              pkgs.clang
            ];

          # No need for CLIENT_DIST with cargo-leptos
          shellHook = ''
            echo "🦀 Leptos development environment loaded!"
            echo "Run: cd server && cargo leptos watch"
            echo "Or from root: cargo leptos watch --manifest-path server/Cargo.toml"
          '';
        };
      }
    );
}
