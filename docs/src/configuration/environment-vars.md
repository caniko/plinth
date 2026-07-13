# Environment Variables

Environment variables override values from `plinth.toml`. This is useful for secrets and deployment-specific overrides.

## Configuration path

| Variable | Default | Description |
|----------|---------|-------------|
| `PLINTH_CONFIG` | `plinth.toml` | Path to the TOML configuration file |

## Server

| Variable | Default | Description |
|----------|---------|-------------|
| `PLINTH_SITE_ADDR` | `127.0.0.1:3000` | Dioxus server bind address |
| `DIOXUS_PUBLIC_PATH` | executable sibling `public/` | Path to compiled site assets (set by Nix wrapper) |
| `PLINTH_RENDER_CACHE_DIR` | _(disabled)_ | Optional writable directory for completed Dioxus HTML responses; use a state directory, never the immutable asset tree |
| `LEPTOS_SITE_ADDR` / `LEPTOS_SITE_ROOT` | legacy | Accepted only by the rollback Leptos binary |

## Authentication

| Variable | Default | Description |
|----------|---------|-------------|
| `PLINTH_API_KEY` | `dev_api_key_change_in_production` | Bearer token for admin API endpoints |

## Database

These override the `[database]` section in `plinth.toml`:

| Variable | TOML key | Description |
|----------|----------|-------------|
| `PLINTH_DATABASE_URL` | `database.database_url` | Postgres connection URL |
| `DATABASE_URL` | `database.database_url` | Postgres connection URL |

## Observability

These override the `[observability]` section:

| Variable | TOML key | Description |
|----------|----------|-------------|
| `RUST_LOG` | `observability.log_level` | Log level filter (e.g. `info,plinth=debug`) |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | `observability.otlp_endpoint` | OTLP endpoint URL |
| `OTEL_EXPORTER_OTLP_HEADERS` | `observability.otlp_headers` | OTLP auth headers |
| `OTEL_SERVICE_NAME` | `observability.service_name` | Telemetry service name |

## Immich

| Variable | TOML key | Description |
|----------|----------|-------------|
| `IMMICH_API_URL` | `immich.api_url` | Immich server URL (enables image proxy) |
| `IMMICH_API_KEY` | — | Immich API key (env-only, not in TOML) |

## Forge tokens

Optional tokens for fetching activity from code forges. Public data works without them but is rate-limited. These tokens are read only from the environment, have no TOML equivalent in `[forge]`, and are never sent to the browser.

| Variable | Description |
|----------|-------------|
| `GITHUB_TOKEN` | GitHub personal access token; raises the rate limit for GitHub API requests |
| `CODEBERG_TOKEN` | Codeberg or Forgejo access token |

Activity metadata is served from the database immediately. When an entry's `fetched_at` is older than `forge.refresh_ttl_secs`, the server starts a background refresh and continues serving the cached entry. Configure the TTL and failed-refresh backoff in the [`[forge]`](plinth-toml.md#forge) table.

## Analytics

These override the `[analytics]` section:

| Variable | TOML key | Description |
|----------|----------|-------------|
| `PLAUSIBLE_DOMAIN` | `analytics.plausible_domain` | Site domain tracked by Plausible |
| `PLAUSIBLE_SCRIPT_URL` | `analytics.plausible_script_url` | URL to self-hosted Plausible script |

## CLI-only

These are used by `plinth-cli`, not the server:

| Variable | Default | Description |
|----------|---------|-------------|
| `PLINTH_API_URL` | `http://localhost:3000` | Target server URL for CLI operations |

## Precedence

1. Environment variables (highest priority)
2. `plinth.toml` values
3. Compiled defaults (lowest priority)
