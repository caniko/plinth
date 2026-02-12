{
  config,
  lib,
  pkgs,
  ...
}:
with lib; let
  cfg = config.services.personal-website;

  # Environment file for the systemd service
  envFile = pkgs.writeText "personal-website.env" ''
    LEPTOS_SITE_ADDR=${cfg.host}:${toString cfg.port}
    LEPTOS_SITE_ROOT=${cfg.package}/site
    SURREALDB_PATH=${cfg.stateDir}/${cfg.database.path}
    SURREALDB_NAMESPACE=${cfg.database.namespace}
    SURREALDB_DATABASE=${cfg.database.database}
    ${optionalString cfg.observability.enable ''
      OTEL_EXPORTER_OTLP_ENDPOINT=${cfg.observability.otlpEndpoint}
      OTEL_SERVICE_NAME=${cfg.observability.serviceName}
      RUST_LOG=${cfg.observability.logLevel}
      ${optionalString (cfg.observability.otlpHeaders != null) ''
        OTEL_EXPORTER_OTLP_HEADERS=${cfg.observability.otlpHeaders}
      ''}
    ''}
    ${cfg.extraEnv}
  '';
in {
  options.services.personal-website = {
    enable = mkEnableOption "Personal website Leptos SSR application";

    package = mkOption {
      type = types.package;
      default = pkgs.personal-website or (throw "personal-website package not found. Add the flake overlay to nixpkgs.overlays");
      defaultText = literalExpression "pkgs.personal-website";
      description = "The personal-website package to use.";
    };

    user = mkOption {
      type = types.str;
      default = "personal-website";
      description = "User account under which the service runs.";
    };

    group = mkOption {
      type = types.str;
      default = "personal-website";
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
      default = "/var/lib/personal-website";
      description = "Directory for stateful data (database, etc).";
    };

    apiKeyFile = mkOption {
      type = types.nullOr types.path;
      default = null;
      description = ''
        Path to file containing the API key for blog administration.
        This file will be loaded securely using systemd's LoadCredential.
        The API key will be available in the BLOG_API_KEY environment variable.
      '';
    };

    database = {
      path = mkOption {
        type = types.str;
        default = "database.db";
        description = "Relative path to the SurrealDB database file within stateDir.";
      };

      namespace = mkOption {
        type = types.str;
        default = "personal_website";
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
        description = ''
          OTLP endpoint URL for pushing telemetry to external OpenObserve instance.
          If empty and observability is enabled, will use local logging only.
        '';
      };

      otlpHeaders = mkOption {
        type = types.nullOr types.str;
        default = null;
        example = "Authorization=Basic xxx,organization=default,stream=default";
        description = ''
          OTLP headers for authentication to external OpenObserve instance.
          Format: comma-separated key=value pairs.
          Consider using agenix or sops-nix for secrets management.
        '';
      };

      serviceName = mkOption {
        type = types.str;
        default = "personal-website";
        description = "Service name to use in telemetry spans.";
      };

      logLevel = mkOption {
        type = types.str;
        default = "info";
        example = "debug,personal_website=trace";
        description = "Rust log level (RUST_LOG environment variable).";
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
      description = "Personal website service user";
      home = cfg.stateDir;
      createHome = true;
    };

    users.groups.${cfg.group} = {};

    # Systemd service
    systemd.services.personal-website = {
      description = "Personal Website Leptos SSR Application";
      after = ["network.target"];
      wantedBy = ["multi-user.target"];

      serviceConfig = {
        Type = "simple";
        User = cfg.user;
        Group = cfg.group;
        Restart = "always";
        RestartSec = "10s";

        # Load environment variables from file
        EnvironmentFile = envFile;

        # Load API key securely if provided
        LoadCredential = mkIf (cfg.apiKeyFile != null) [
          "api-key:${cfg.apiKeyFile}"
        ];

        # Set BLOG_API_KEY from credential if provided
        ExecStartPre = mkIf (cfg.apiKeyFile != null) (
          pkgs.writeShellScript "set-api-key" ''
            export BLOG_API_KEY=$(cat "''${CREDENTIALS_DIRECTORY}/api-key")
          ''
        );

        # Start the server
        ExecStart = "${cfg.package}/bin/server";

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
        SyslogIdentifier = "personal-website";
      };

      # Ensure BLOG_API_KEY is set from credential
      environment = mkIf (cfg.apiKeyFile != null) {
        BLOG_API_KEY = "%d/api-key";
      };
    };

    # Create state directory with correct permissions
    systemd.tmpfiles.rules = [
      "d ${cfg.stateDir} 0750 ${cfg.user} ${cfg.group} -"
    ];
  };
}
