# Dev Environment

## Using Nix (recommended)

```bash
git clone https://codeberg.org/caniko/plinth.git
cd plinth
nix develop
```

The dev shell provides:
- Rust nightly with `wasm32-unknown-unknown` target
- dioxus-cli, wasm-bindgen-cli, binaryen
- Tailwind CSS standalone binary
- PostgreSQL 16 with pgvector
- sqlx-cli
- OpenSSL, ONNX Runtime, libclang
- Mold linker (Linux)
- mdBook (for documentation development)

## Local database

Start the development Postgres before running the server or integration tests:

```bash
./scripts/dev-db.sh start
```

The dev shell exports `PGDATA=$PWD/.dev-pgdata`, `PGHOST=$PWD/.dev-pgsocket`, and `DATABASE_URL=postgres://localhost/plinth?host=$PWD/.dev-pgsocket`. The lifecycle script initializes the cluster if needed, creates the `plinth` database, and installs the `vector` extension.

Other database commands:

```bash
./scripts/dev-db.sh stop
./scripts/dev-db.sh reset
```

`pg_ctl` requires `PGDATA`; run these commands inside `nix develop` unless you set the Postgres environment variables yourself.

## Development server

```bash
dx serve --web --fullstack
```

This starts the Axum server with Dioxus hot reload at `http://127.0.0.1:3000`. Changes to Rust source files trigger recompilation of both the server and the WASM client.

## Running checks

```bash
# Full CI check (build + clippy + fmt + tests)
nix flake check

# Individual commands
cargo fmt --all
cargo clippy --all-targets -- --deny warnings
cargo test --workspace --exclude plinth-client
```

The client crate is excluded from `cargo test` because it targets `wasm32-unknown-unknown`.

## Building documentation locally

```bash
cd docs
mdbook serve
```

This starts a local preview server with live reload.

## Important build notes

- **New files must be `git add`-ed** before `nix flake check` or `nix build` can see them (Nix uses the git index)
- **`reqwest::Client::new()` panics in Nix sandbox** — use `Client::builder().build()` and handle errors
- **`fastembed::TextEmbedding::try_new()`** downloads models at runtime and fails in the Nix sandbox — all tests must avoid it
- **Raw string literals**: use `r##"..."##` when content contains `"#` (common with Markdown headings)
