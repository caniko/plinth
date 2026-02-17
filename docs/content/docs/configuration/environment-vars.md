+++
title = "Environment Variables"
description = "Environment variable reference"
weight = 20
+++

Environment variables override values from `plinth.toml`. This is useful for secrets and deployment-specific overrides.

## Configuration path

| Variable | Default | Description |
|----------|---------|-------------|
| `PLINTH_CONFIG` | `plinth.toml` | Path to the TOML configuration file |

## Server

| Variable | Default | Description |
|----------|---------|-------------|
| `LEPTOS_SITE_ADDR` | `127.0.0.1:3000` | Server bind address (set by Nix wrapper) |
| `LEPTOS_SITE_ROOT` | `target/site` | Path to compiled site assets (set by Nix wrapper) |

## Authentication

| Variable | Default | Description |
|----------|---------|-------------|
| `PLINTH_API_KEY` | `dev_api_key_change_in_production` | Bearer token for admin API endpoints |

## Database

These override the `[database]` section in `plinth.toml`:

| Variable | TOML key | Description |
|----------|----------|-------------|
| `SURREALDB_PATH` | `database.path` | SurrealDB file path |
| `SURREALDB_NAMESPACE` | `database.namespace` | SurrealDB namespace |
| `SURREALDB_DATABASE` | `database.database` | SurrealDB database name |

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

## CLI-only

These are used by `plinth-cli`, not the server:

| Variable | Default | Description |
|----------|---------|-------------|
| `PLINTH_API_URL` | `http://localhost:3000` | Target server URL for CLI operations |

## Precedence

1. Environment variables (highest priority)
2. `plinth.toml` values
3. Compiled defaults (lowest priority)
