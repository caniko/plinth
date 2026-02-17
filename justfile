# Development
dev:
    cargo leptos watch

# Build production release via Nix
build:
    nix build .#plinth

# Run all checks (build + clippy + fmt + tests) — same as CI
check:
    nix flake check

# Run tests (excludes client crate — it targets WASM)
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
