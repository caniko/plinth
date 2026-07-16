# Development
dev:
    dx serve --web --fullstack

# Build production release via Nix
build:
    nix build .#plinth

# Run all checks (build + clippy + fmt + tests) — same as CI
check:
    nix flake check

# Run tests (the browser-only Dioxus target is checked separately for WASM)
test:
    cargo test --workspace --exclude plinth-client

# Run a single test by name
test-one name:
    cargo test --package plinth-server {{ name }}

# Format all code
fmt:
    cargo fmt --all

# Check formatting without modifying files
fmt-check:
    cargo fmt --all -- --check

# Run clippy lints
clippy:
    cargo clippy --all-targets -- --deny warnings

# Format + clippy
lint: fmt clippy

# Regenerate favicon PNGs from public/favicon.svg
favicons:
    #!/usr/bin/env bash
    for size in 16 32 48 180 192 512; do
        nix-shell -p inkscape --run \
            "inkscape --export-type=png --export-filename=public/favicon-${size}x${size}.png \
                --export-width=$size --export-height=$size --export-area-page public/favicon.svg"
    done
    cp public/favicon-16x16.png docs/static/favicon-16x16.png
    cp public/favicon-32x32.png docs/static/favicon-32x32.png
    cp public/favicon-180x180.png docs/static/apple-touch-icon.png

# Regenerate checked-in Pkl producers used during pure flake evaluation.
generate-build-contract-artifacts:
    nix develop .#codegen -c ./scripts/generate-build-contract-artifacts.sh
