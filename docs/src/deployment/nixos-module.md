# NixOS Module

Plinth ships a NixOS module for declarative deployment with systemd hardening.

## Adding the flake

```nix
# flake.nix
{
  inputs.plinth.url = "github:caniko/plinth";

  outputs = { self, nixpkgs, plinth, ... }: {
    nixosConfigurations.myhost = nixpkgs.lib.nixosSystem {
      modules = [
        plinth.nixosModules.default
        # Make packages available via overlay
        { nixpkgs.overlays = [ plinth.overlays.default ]; }
      ];
    };
  };
}
```

## Minimal deployment

```nix
services.plinth.instances.default = {
  host = "127.0.0.1";
  port = 3000;
};
```

This starts Plinth on port 3000 with all defaults. The module also enables PostgreSQL 16, loads pgvector, creates the `plinth` database and `plinth` role, and starts the service after `postgresql.service` with `DATABASE_URL=postgres:///plinth?host=/run/postgresql&user=plinth`.

## Site personalisation

```nix
services.plinth.instances.default = {
  site = {
    name = "Jane's Blog";
    tagline = "Thoughts on systems programming";
    defaultTheme = "dark";

    author = {
      name = "Jane Doe";
      email = "jane@example.com";
    };

    social = {
      github = "https://github.com/janedoe";
      mastodon = "https://fosstodon.org/@janedoe";
    };

    nav = [
      { label = "Blog"; path = "/posts"; }
      { label = "Projects"; path = "/projects"; }
      { label = "About"; path = "/about"; }
    ];
  };

  pages.blog = {
    title = "Blog";
    subtitle = "Notes on software and systems";
  };
};
```

## Production with Caddy

```nix
services.plinth.instances.default = {
  host = "127.0.0.1";
  port = 3000;
  stateDir = "/var/lib/plinth";
  apiKeyFile = "/run/secrets/plinth-api-key";

  database = {
    name = "plinth";
    url = "postgres:///plinth?host=/run/postgresql&user=plinth";
  };
};

services.caddy = {
  enable = true;
  virtualHosts."example.com".extraConfig = ''
    reverse_proxy localhost:3000
  '';
};

networking.firewall.allowedTCPPorts = [ 80 443 ];
```

## Observability

```nix
services.plinth.instances.default.observability = {
  enable = true;
  otlpEndpoint = "https://openobserve.example.com:5081";
  serviceName = "plinth-prod";
  logLevel = "info,plinth=debug";
};
```

## Secrets with agenix

```nix
age.secrets.plinth-api-key = {
  file = ./secrets/plinth-api-key.age;
  owner = "plinth";
  group = "plinth";
};

services.plinth.instances.default = {
  apiKeyFile = config.age.secrets.plinth-api-key.path;
};
```

## Secrets with sops-nix

```nix
sops.secrets."plinth/api-key".owner = "plinth";

services.plinth.instances.default = {
  apiKeyFile = config.sops.secrets."plinth/api-key".path;
};
```

## All module options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `instances` | attrset | `{}` | Named Plinth instances to run |
| `package` | package | `pkgs.plinth` | Plinth package to use |
| `user` | string | The PostgreSQL database name by default | System user; must match `database.name` for local peer authentication |
| `group` | string | The `user` value by default | System group |
| `host` | string | `"127.0.0.1"` | Bind address |
| `port` | port | `3000` | Bind port |
| `stateDir` | path | `/var/lib/plinth` | Stateful data directory |
| `apiKeyFile` | path or null | `null` | Path to API key file (loaded via systemd `LoadCredential`) |
| `site.*` | — | — | Site identity (see [plinth.toml](../configuration/plinth-toml.md#site)) |
| `pages.*` | — | — | Page-specific config (see [plinth.toml](../configuration/plinth-toml.md#pageshome)) |
| `database.name` | string | `"plinth"` for default, otherwise `"plinth_<name>"` | Postgres database name |
| `database.url` | string | `postgres:///plinth?host=/run/postgresql&user=plinth` for default | Postgres connection URL |
| `observability.enable` | bool | `false` | Enable OTLP export |
| `observability.otlpEndpoint` | string | `""` | OTLP endpoint URL |
| `observability.otlpHeaders` | string or null | `null` | OTLP auth headers |
| `observability.serviceName` | string | `"plinth"` | Telemetry service name |
| `observability.logLevel` | string | `"info"` | Log level (RUST_LOG) |
| `search.defaultLimit` | int | `10` | Search result limit |
| `search.relatedLimit` | int | `5` | Related articles limit |
| `search.minSimilarity` | float | `0.5` | Min similarity for opinion tracking |
| `content.wordsPerMinute` | int | `200` | Reading time WPM |
| `content.vectorTruncation` | int | `5000` | Embedding char limit |
| `immich.apiUrl` | string | `""` | Immich URL (empty = disabled) |
| `immich.apiKey` | string | `""` | Immich API key |
| `images.cacheMaxAge` | int | `31536000` | Image cache max-age (seconds) |
| `extraEnv` | lines | `""` | Additional env vars (KEY=value per line) |

## Systemd hardening

The module applies security hardening by default:

- `NoNewPrivileges`, `ProtectSystem=strict`, `ProtectHome`
- `PrivateTmp`, `PrivateDevices`, `PrivateMounts`
- `RestrictAddressFamilies` (AF_UNIX, AF_INET, AF_INET6 only)
- `RestrictNamespaces`, `LockPersonality`, `RestrictRealtime`
- `ReadWritePaths` limited to `stateDir`
