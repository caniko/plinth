{
  config,
  lib,
  pkgs,
  ...
}:
with lib; let
  cfg = config.services.plinth;

  # Helper to escape TOML string values
  tomlStr = s: ''"${s}"'';

  # Generate plinth.toml from Nix options
  configFile = pkgs.writeText "plinth.toml" ''
    [site]
    name = ${tomlStr cfg.site.name}
    tagline = ${tomlStr cfg.site.tagline}
    description = ${tomlStr cfg.site.description}
    lang = ${tomlStr cfg.site.lang}
    default_theme = ${tomlStr cfg.site.defaultTheme}
    base_url = ${tomlStr cfg.site.baseUrl}

    [site.author]
    name = ${tomlStr cfg.site.author.name}
    email = ${tomlStr cfg.site.author.email}

    [site.social]
    github = ${tomlStr cfg.site.social.github}
    gitlab = ${tomlStr cfg.site.social.gitlab}
    codeberg = ${tomlStr cfg.site.social.codeberg}
    mastodon = ${tomlStr cfg.site.social.mastodon}
    bluesky = ${tomlStr cfg.site.social.bluesky}

    [site.footer]
    project_name = ${tomlStr cfg.site.footer.projectName}
    project_url = ${tomlStr cfg.site.footer.projectUrl}

    ${concatMapStringsSep "\n" (item: ''
    [[site.nav]]
    label = ${tomlStr item.label}
    path = ${tomlStr item.path}
    '') cfg.site.nav}

    [pages.home]
    title = ${tomlStr cfg.pages.home.title}
    description = ${tomlStr cfg.pages.home.description}

    [pages.blog]
    title = ${tomlStr cfg.pages.blog.title}
    subtitle = ${tomlStr cfg.pages.blog.subtitle}
    description = ${tomlStr cfg.pages.blog.description}

    [pages.portfolio]
    title = ${tomlStr cfg.pages.portfolio.title}
    subtitle = ${tomlStr cfg.pages.portfolio.subtitle}
    description = ${tomlStr cfg.pages.portfolio.description}

    [pages.about]
    title = ${tomlStr cfg.pages.about.title}
    description = ${tomlStr cfg.pages.about.description}

    [server]
    host = ${tomlStr cfg.host}
    port = ${toString cfg.port}

    [database]
    path = ${tomlStr "${cfg.stateDir}/${cfg.database.path}"}
    namespace = ${tomlStr cfg.database.namespace}
    database = ${tomlStr cfg.database.database}

    [observability]
    service_name = ${tomlStr cfg.observability.serviceName}
    log_level = ${tomlStr cfg.observability.logLevel}
    otlp_endpoint = ${tomlStr (if cfg.observability.enable then cfg.observability.otlpEndpoint else "")}
    otlp_headers = ${tomlStr (if cfg.observability.otlpHeaders != null then cfg.observability.otlpHeaders else "")}

    [search]
    default_limit = ${toString cfg.search.defaultLimit}
    related_limit = ${toString cfg.search.relatedLimit}
    min_similarity = ${toString cfg.search.minSimilarity}

    [content]
    words_per_minute = ${toString cfg.content.wordsPerMinute}
    vector_truncation = ${toString cfg.content.vectorTruncation}

    [immich]
    api_url = ${tomlStr cfg.immich.apiUrl}
    api_key = ${tomlStr cfg.immich.apiKey}

    [images]
    cache_max_age = ${toString cfg.images.cacheMaxAge}

    [feeds]
    blog_limit = ${toString cfg.feeds.blogLimit}
    projects_limit = ${toString cfg.feeds.projectsLimit}
  '';
in {
  options.services.plinth = {
    enable = mkEnableOption "Plinth personal website Leptos SSR application";

    package = mkOption {
      type = types.package;
      default = pkgs.plinth or (throw "plinth package not found. Add the flake overlay to nixpkgs.overlays");
      defaultText = literalExpression "pkgs.plinth";
      description = "The Plinth package to use.";
    };

    user = mkOption {
      type = types.str;
      default = "plinth";
      description = "User account under which the service runs.";
    };

    group = mkOption {
      type = types.str;
      default = "plinth";
      description = "Group under which the service runs.";
    };

    host = mkOption {
      type = types.str;
      default = "127.0.0.1";
      description = "Host address to bind the server to.";
    };

    port = mkOption {
      type = types.port;
      default = 3000;
      description = "Port to bind the server to.";
    };

    stateDir = mkOption {
      type = types.path;
      default = "/var/lib/plinth";
      description = "Directory for stateful data (database, etc).";
    };

    apiKeyFile = mkOption {
      type = types.nullOr types.path;
      default = null;
      description = ''
        Path to file containing the API key for administration.
        This file will be loaded securely using systemd's LoadCredential.
        The API key will be available in the PLINTH_API_KEY environment variable.
      '';
    };

    # Site identity
    site = {
      name = mkOption {
        type = types.str;
        default = "Plinth";
        description = "Site name displayed in the header and page titles.";
      };

      tagline = mkOption {
        type = types.str;
        default = "Welcome to my website";
        description = "Short tagline shown on the home page.";
      };

      description = mkOption {
        type = types.str;
        default = "A personal website";
        description = "Default meta description for the site.";
      };

      lang = mkOption {
        type = types.str;
        default = "en";
        description = "HTML lang attribute.";
      };

      defaultTheme = mkOption {
        type = types.enum' ["dark" "light"];
        default = "dark";
        description = "Default theme (dark or light).";
      };

      baseUrl = mkOption {
        type = types.str;
        default = "";
        example = "https://example.com";
        description = "Public base URL for the site (used in RSS feeds). Empty = auto-detect from host:port.";
      };

      author = {
        name = mkOption {
          type = types.str;
          default = "Admin";
          description = "Default author name for published articles.";
        };

        email = mkOption {
          type = types.str;
          default = "";
          description = "Email address shown in footer (empty = hidden).";
        };
      };

      social = {
        github = mkOption {
          type = types.str;
          default = "";
          description = "GitHub profile URL (empty = hidden).";
        };

        gitlab = mkOption {
          type = types.str;
          default = "";
          description = "GitLab profile URL (empty = hidden).";
        };

        codeberg = mkOption {
          type = types.str;
          default = "";
          description = "Codeberg profile URL (empty = hidden).";
        };

        mastodon = mkOption {
          type = types.str;
          default = "";
          description = "Mastodon profile URL (empty = hidden).";
        };

        bluesky = mkOption {
          type = types.str;
          default = "";
          description = "Bluesky profile URL (empty = hidden).";
        };
      };

      footer = {
        projectName = mkOption {
          type = types.str;
          default = "Plinth";
          description = "Project name shown in footer attribution.";
        };

        projectUrl = mkOption {
          type = types.str;
          default = "https://codeberg.org/caniko/plinth";
          description = "Project URL linked in footer attribution.";
        };
      };

      nav = mkOption {
        type = types.listOf (types.submodule {
          options = {
            label = mkOption {
              type = types.str;
              description = "Navigation label text.";
            };
            path = mkOption {
              type = types.str;
              description = "Navigation link path.";
            };
          };
        });
        default = [
          { label = "Posts"; path = "/posts"; }
          { label = "Projects"; path = "/projects"; }
          { label = "About"; path = "/about"; }
        ];
        description = "Navigation menu items (order matters).";
      };
    };

    # Page-specific configuration
    pages = {
      home = {
        title = mkOption {
          type = types.str;
          default = "";
          description = "Home page title (empty = use site name).";
        };
        description = mkOption {
          type = types.str;
          default = "";
          description = "Home page meta description (empty = use site description).";
        };
      };

      blog = {
        title = mkOption {
          type = types.str;
          default = "Posts";
          description = "Blog page title.";
        };
        subtitle = mkOption {
          type = types.str;
          default = "";
          description = "Blog page subtitle (empty = hidden).";
        };
        description = mkOption {
          type = types.str;
          default = "";
          description = "Blog page meta description.";
        };
      };

      portfolio = {
        title = mkOption {
          type = types.str;
          default = "Projects";
          description = "Portfolio page title.";
        };
        subtitle = mkOption {
          type = types.str;
          default = "";
          description = "Portfolio page subtitle (empty = hidden).";
        };
        description = mkOption {
          type = types.str;
          default = "";
          description = "Portfolio page meta description.";
        };
      };

      about = {
        title = mkOption {
          type = types.str;
          default = "About Me";
          description = "About page title.";
        };
        description = mkOption {
          type = types.str;
          default = "";
          description = "About page meta description.";
        };
      };
    };

    database = {
      path = mkOption {
        type = types.str;
        default = "database.db";
        description = "Relative path to the SurrealDB database file within stateDir.";
      };

      namespace = mkOption {
        type = types.str;
        default = "plinth";
        description = "SurrealDB namespace to use.";
      };

      database = mkOption {
        type = types.str;
        default = "main";
        description = "SurrealDB database name to use.";
      };
    };

    observability = {
      enable = mkEnableOption "OpenObserve observability integration (OTLP push)";

      otlpEndpoint = mkOption {
        type = types.str;
        default = "";
        example = "https://openobserve.example.com:5081";
        description = "OTLP endpoint URL for pushing telemetry.";
      };

      otlpHeaders = mkOption {
        type = types.nullOr types.str;
        default = null;
        example = "Authorization=Basic xxx,organization=default,stream=default";
        description = "OTLP headers for authentication (comma-separated key=value pairs).";
      };

      serviceName = mkOption {
        type = types.str;
        default = "plinth";
        description = "Service name to use in telemetry spans.";
      };

      logLevel = mkOption {
        type = types.str;
        default = "info";
        example = "debug,plinth=trace";
        description = "Rust log level (RUST_LOG environment variable).";
      };
    };

    search = {
      defaultLimit = mkOption {
        type = types.int;
        default = 10;
        description = "Default search result limit.";
      };

      relatedLimit = mkOption {
        type = types.int;
        default = 5;
        description = "Default related articles limit.";
      };

      minSimilarity = mkOption {
        type = types.float;
        default = 0.5;
        description = "Minimum similarity threshold for opinion tracking.";
      };
    };

    content = {
      wordsPerMinute = mkOption {
        type = types.int;
        default = 200;
        description = "Words per minute for reading time calculation.";
      };

      vectorTruncation = mkOption {
        type = types.int;
        default = 5000;
        description = "Character limit before generating embeddings.";
      };
    };

    immich = {
      apiUrl = mkOption {
        type = types.str;
        default = "";
        description = "Immich server URL (empty = disabled).";
      };

      apiKey = mkOption {
        type = types.str;
        default = "";
        description = "Immich API key.";
      };
    };

    images = {
      cacheMaxAge = mkOption {
        type = types.int;
        default = 31536000;
        description = "Cache-Control max-age for proxied images (seconds).";
      };
    };

    feeds = {
      blogLimit = mkOption {
        type = types.int;
        default = 50;
        description = "Maximum number of blog posts in the RSS feed.";
      };

      projectsLimit = mkOption {
        type = types.int;
        default = 50;
        description = "Maximum number of projects in the RSS feed.";
      };
    };

    extraEnv = mkOption {
      type = types.lines;
      default = "";
      description = "Additional environment variables to set (one per line, KEY=value format).";
    };
  };

  config = mkIf cfg.enable {
    # Create user and group
    users.users.${cfg.user} = {
      isSystemUser = true;
      group = cfg.group;
      description = "Plinth service user";
      home = cfg.stateDir;
      createHome = true;
    };

    users.groups.${cfg.group} = {};

    # Systemd service
    systemd.services.plinth = {
      description = "Plinth - Leptos SSR Application";
      after = ["network.target"];
      wantedBy = ["multi-user.target"];

      serviceConfig = {
        Type = "simple";
        User = cfg.user;
        Group = cfg.group;
        Restart = "always";
        RestartSec = "10s";

        # Point server to generated TOML config and Leptos site root
        Environment = [
          "PLINTH_CONFIG=${configFile}"
          "LEPTOS_SITE_ADDR=${cfg.host}:${toString cfg.port}"
          "LEPTOS_SITE_ROOT=${cfg.package}/site"
        ];

        # Load API key securely if provided
        LoadCredential = mkIf (cfg.apiKeyFile != null) [
          "api-key:${cfg.apiKeyFile}"
        ];

        # Set PLINTH_API_KEY from credential if provided
        ExecStartPre = mkIf (cfg.apiKeyFile != null) (
          pkgs.writeShellScript "set-api-key" ''
            export PLINTH_API_KEY=$(cat "''${CREDENTIALS_DIRECTORY}/api-key")
          ''
        );

        # Start the server
        ExecStart = "${cfg.package}/bin/plinth-server";

        # Working directory
        WorkingDirectory = cfg.stateDir;

        # Security hardening
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        PrivateDevices = true;
        ProtectHostname = true;
        ProtectClock = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectKernelLogs = true;
        ProtectControlGroups = true;
        RestrictAddressFamilies = ["AF_UNIX" "AF_INET" "AF_INET6"];
        RestrictNamespaces = true;
        LockPersonality = true;
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        RemoveIPC = true;
        PrivateMounts = true;

        # Allow writing to state directory
        ReadWritePaths = [cfg.stateDir];

        # Logging
        StandardOutput = "journal";
        StandardError = "journal";
        SyslogIdentifier = "plinth";
      };

      # Ensure PLINTH_API_KEY is set from credential
      environment = mkMerge [
        (mkIf (cfg.apiKeyFile != null) {
          PLINTH_API_KEY = "%d/api-key";
        })
        (mkIf (cfg.extraEnv != "") {
          # Extra env vars are passed directly
        })
      ];
    };

    # Create state directory with correct permissions
    systemd.tmpfiles.rules = [
      "d ${cfg.stateDir} 0750 ${cfg.user} ${cfg.group} -"
    ];
  };
}
