# Example NixOS configuration for personal-website service
#
# This file demonstrates various deployment scenarios:
# 1. Basic deployment with minimal configuration
# 2. Production deployment with Caddy reverse proxy
# 3. Deployment with OpenObserve observability
# 4. Multi-instance deployment
# 5. Secrets management with agenix/sops-nix

{
  config,
  pkgs,
  ...
}: {
  imports = [
    # Import the personal-website flake module
    # Method 1: Via flake inputs
    # inputs.personal-website.nixosModules.default

    # Method 2: Direct import (if using in same repo)
    ./personal-website.nix
  ];

  # ============================================================================
  # Example 1: Basic Deployment
  # ============================================================================
  # Minimal configuration for local testing or development

  services.personal-website = {
    enable = true;
    host = "127.0.0.1";
    port = 3000;

    # Use development build for faster iteration
    # package = pkgs.personal-website-dev;
  };

  # ============================================================================
  # Example 2: Production Deployment with Reverse Proxy
  # ============================================================================
  # Production configuration with Caddy as reverse proxy

  /*
  services.personal-website = {
    enable = true;
    host = "127.0.0.1";  # Bind to localhost
    port = 3000;

    # Use production build
    package = pkgs.personal-website;

    # Custom state directory
    stateDir = "/var/lib/personal-website";

    # Database configuration
    database = {
      namespace = "personal_website";
      database = "prod";
      path = "database.db";
    };

    # API key for blog administration (using systemd credential)
    apiKeyFile = "/run/secrets/blog-api-key";
  };

  # Caddy reverse proxy
  services.caddy = {
    enable = true;
    virtualHosts."example.com" = {
      extraConfig = ''
        reverse_proxy localhost:3000
      '';
    };
  };

  # Firewall configuration
  networking.firewall.allowedTCPPorts = [ 80 443 ];
  */

  # ============================================================================
  # Example 3: Deployment with OpenObserve Observability
  # ============================================================================
  # Enable observability with external OpenObserve instance

  /*
  services.personal-website = {
    enable = true;
    host = "127.0.0.1";
    port = 3000;

    observability = {
      enable = true;
      otlpEndpoint = "https://openobserve.example.com:5081";
      serviceName = "personal-website-prod";
      logLevel = "info,personal_website=debug";

      # IMPORTANT: Store OTLP headers securely using secrets management
      # Don't hardcode credentials! Use agenix or sops-nix
      # otlpHeaders = "Authorization=Basic <base64-encoded-credentials>,organization=default,stream=default";
    };

    # Load API key and OTLP headers from secure sources
    apiKeyFile = config.age.secrets.blog-api-key.path;
  };

  # Example with agenix for secrets management
  age.secrets = {
    blog-api-key = {
      file = ./secrets/blog-api-key.age;
      owner = config.services.personal-website.user;
    };
  };
  */

  # ============================================================================
  # Example 4: Multi-Instance Deployment
  # ============================================================================
  # Run multiple instances (e.g., staging + production)

  /*
  # Production instance
  services.personal-website-prod = {
    enable = true;
    host = "127.0.0.1";
    port = 3000;
    user = "website-prod";
    group = "website-prod";
    stateDir = "/var/lib/personal-website-prod";

    database = {
      namespace = "personal_website_prod";
      database = "main";
    };

    apiKeyFile = "/run/secrets/blog-api-key-prod";
  };

  # Staging instance
  services.personal-website-staging = {
    enable = true;
    host = "127.0.0.1";
    port = 3001;
    user = "website-staging";
    group = "website-staging";
    stateDir = "/var/lib/personal-website-staging";

    # Use dev build for staging
    package = pkgs.personal-website-dev;

    database = {
      namespace = "personal_website_staging";
      database = "main";
    };

    apiKeyFile = "/run/secrets/blog-api-key-staging";
  };
  */

  # ============================================================================
  # Example 5: Advanced Configuration with Extra Environment Variables
  # ============================================================================

  /*
  services.personal-website = {
    enable = true;
    host = "0.0.0.0";  # Bind to all interfaces
    port = 8080;

    # Additional environment variables
    extraEnv = ''
      CUSTOM_FEATURE_FLAG=enabled
      MAX_CONNECTIONS=1000
    '';

    observability = {
      enable = true;
      otlpEndpoint = "https://openobserve.example.com:5081";
      serviceName = "website-${config.networking.hostName}";
      logLevel = "debug";
    };
  };
  */

  # ============================================================================
  # Secrets Management Examples
  # ============================================================================

  # --- Using agenix ---
  /*
  # 1. Add agenix to your flake inputs
  # 2. Generate age key: age-keygen -o /etc/nixos/age-key.txt
  # 3. Encrypt secret: age -r <public-key> -o secrets/blog-api-key.age
  # 4. Configure in NixOS:

  age = {
    secrets = {
      blog-api-key = {
        file = ./secrets/blog-api-key.age;
        owner = "personal-website";
        group = "personal-website";
      };
      otlp-headers = {
        file = ./secrets/otlp-headers.age;
        owner = "personal-website";
        group = "personal-website";
      };
    };
  };

  services.personal-website = {
    enable = true;
    apiKeyFile = config.age.secrets.blog-api-key.path;
    # For OTLP headers, read from file in extraEnv
    extraEnv = ''
      OTEL_EXPORTER_OTLP_HEADERS=$(cat ${config.age.secrets.otlp-headers.path})
    '';
  };
  */

  # --- Using sops-nix ---
  /*
  # 1. Add sops-nix to your flake inputs
  # 2. Create .sops.yaml configuration
  # 3. Encrypt secrets: sops secrets/secrets.yaml
  # 4. Configure in NixOS:

  sops = {
    defaultSopsFile = ./secrets/secrets.yaml;
    age.keyFile = "/etc/nixos/age-key.txt";

    secrets = {
      "personal-website/api-key" = {
        owner = "personal-website";
      };
      "personal-website/otlp-headers" = {
        owner = "personal-website";
      };
    };
  };

  services.personal-website = {
    enable = true;
    apiKeyFile = config.sops.secrets."personal-website/api-key".path;
    observability = {
      enable = true;
      otlpEndpoint = "https://openobserve.example.com:5081";
    };
    extraEnv = ''
      OTEL_EXPORTER_OTLP_HEADERS=$(cat ${config.sops.secrets."personal-website/otlp-headers".path})
    '';
  };
  */

  # ============================================================================
  # Monitoring and Maintenance
  # ============================================================================

  # Automatic database backups
  /*
  systemd.services.personal-website-backup = {
    description = "Backup personal website database";
    serviceConfig = {
      Type = "oneshot";
      User = "personal-website";
      ExecStart = ''
        ${pkgs.bash}/bin/bash -c 'cp ${config.services.personal-website.stateDir}/database.db ${config.services.personal-website.stateDir}/database.db.backup.$(date +%Y%m%d)'
      '';
    };
  };

  systemd.timers.personal-website-backup = {
    description = "Backup personal website database daily";
    wantedBy = ["timers.target"];
    timerConfig = {
      OnCalendar = "daily";
      Persistent = true;
    };
  };
  */

  # Log rotation via journald
  /*
  services.journald.extraConfig = ''
    MaxRetentionSec=7day
  '';
  */
}
