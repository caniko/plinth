+++
title = "NixOS Module"
description = "Deploy Plinth declaratively on NixOS"
weight = 10
+++

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
services.plinth = {
  enable = true;
  host = "127.0.0.1";
  port = 3000;
};
```

This starts Plinth on port 3000 with all defaults and a SurrealDB file at `/var/lib/plinth/database.db`.

## Site personalisation

```nix
services.plinth = {
  enable = true;

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
services.plinth = {
  enable = true;
  host = "127.0.0.1";
  port = 3000;
  stateDir = "/var/lib/plinth";
  apiKeyFile = "/run/secrets/plinth-api-key";

  database = {
    namespace = "plinth";
    database = "prod";
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
services.plinth.observability = {
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

services.plinth = {
  enable = true;
  apiKeyFile = config.age.secrets.plinth-api-key.path;
};
```

## Secrets with sops-nix

```nix
sops.secrets."plinth/api-key".owner = "plinth";

services.plinth = {
  enable = true;
  apiKeyFile = config.sops.secrets."plinth/api-key".path;
};
```

## All module options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enable` | bool | `false` | Enable the Plinth service |
| `package` | package | `pkgs.plinth` | Plinth package to use |
| `user` | string | `"plinth"` | System user |
| `group` | string | `"plinth"` | System group |
| `host` | string | `"127.0.0.1"` | Bind address |
| `port` | port | `3000` | Bind port |
| `stateDir` | path | `/var/lib/plinth` | Stateful data directory |
| `apiKeyFile` | path or null | `null` | Path to API key file (loaded via systemd `LoadCredential`) |
| `site.*` | — | — | Site identity (see [plinth.toml](/docs/configuration/plinth-toml/#site)) |
| `pages.*` | — | — | Page-specific config (see [plinth.toml](/docs/configuration/plinth-toml/#pageshome)) |
| `database.path` | string | `"database.db"` | DB file path relative to stateDir |
| `database.namespace` | string | `"plinth"` | SurrealDB namespace |
| `database.database` | string | `"main"` | SurrealDB database name |
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
