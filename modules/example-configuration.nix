# Example NixOS configuration for Plinth service
#
# This file demonstrates various deployment scenarios:
# 1. Basic deployment with minimal configuration
# 2. Site personalization (name, social links, nav, pages)
# 3. Production deployment with Caddy reverse proxy
# 4. Deployment with OpenObserve observability
# 5. Multi-instance deployment (staging + production)
# 6. Secrets management with agenix/sops-nix

{
  config,
  pkgs,
  ...
}: {
  imports = [
    # Import the Plinth flake module
    # Method 1: Via flake inputs
    # inputs.plinth.nixosModules.default

    # Method 2: Direct import (if using in same repo)
    ./plinth.nix
  ];

  # ============================================================================
  # Example 1: Basic Deployment
  # ============================================================================
  # Minimal configuration for local testing or development

  services.plinth.instances.default = {
    host = "127.0.0.1";
    port = 3000;

    # Use development build for faster iteration
    # package = pkgs.plinth-dev;
  };

  # ============================================================================
  # Example 2: Site Personalization
  # ============================================================================
  # Customize your site identity, navigation, and page content.
  # All site.* and pages.* options end up in plinth.toml and are served
  # to the client via a Dioxus fullstack loader.

  /*
  services.plinth.instances.default = {
    # Site identity
    site = {
      name = "Jane's Blog";
      tagline = "Thoughts on systems programming and open source";
      description = "Jane Doe's personal website about Rust, NixOS, and more";
      lang = "en";
      defaultTheme = "dark";  # "dark" or "light"

      author = {
        name = "Jane Doe";
        email = "jane@example.com";
      };

      # Social links — only non-empty values render in the footer
      social = {
        github = "https://github.com/janedoe";
        mastodon = "https://fosstodon.org/@janedoe";
        codeberg = "https://codeberg.org/janedoe";
        gitlab = "";  # empty = hidden
      };

      footer = {
        projectName = "Plinth";
        projectUrl = "https://codeberg.org/caniko/plinth";
      };

      # Navigation items (order matters)
      nav = [
        { label = "Blog"; path = "/posts"; }
        { label = "Projects"; path = "/projects"; }
        { label = "About"; path = "/about"; }
      ];
    };

    # Page-specific titles and descriptions
    pages = {
      home = {
        title = "";             # empty = use site.name
        description = "";       # empty = use site.description
      };
      blog = {
        title = "Blog";
        subtitle = "Notes on software and systems";
        description = "Jane's blog posts about Rust, NixOS, and software development";
      };
      portfolio = {
        title = "Projects";
        subtitle = "Open source and personal work";
        description = "A collection of my open source projects";
      };
      about = {
        title = "About Me";
        description = "Learn more about Jane Doe";
      };
    };
  };
  */

  # ============================================================================
  # Example 3: Production Deployment with Reverse Proxy
  # ============================================================================
  # Production configuration with Caddy as reverse proxy

  /*
  services.plinth.instances.prod = {
    host = "127.0.0.1";  # Bind to localhost
    port = 3000;

    # Use production build
    package = pkgs.plinth;

    # Database configuration
    database = {
      name = "plinth";
      url = "postgres:///plinth?host=/run/postgresql";
    };

    # API key for administration (using systemd credential)
    apiKeyFile = "/run/secrets/plinth-api-key";
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
  # Example 4: Deployment with OpenObserve Observability
  # ============================================================================
  # Enable observability with external OpenObserve instance

  /*
  services.plinth.instances.default = {
    host = "127.0.0.1";
    port = 3000;

    observability = {
      enable = true;
      otlpEndpoint = "https://openobserve.example.com:5081";
      logLevel = "info,plinth=debug";

      # IMPORTANT: Store OTLP headers securely using secrets management
      # Don't hardcode credentials! Use agenix or sops-nix
      # otlpHeaders = "Authorization=Basic <base64-encoded-credentials>,organization=default,stream=default";
    };

    # Load API key from secure source
    apiKeyFile = config.age.secrets.plinth-api-key.path;
  };

  # Example with agenix for secrets management
  age.secrets = {
    plinth-api-key = {
      file = ./secrets/plinth-api-key.age;
      owner = "plinth-default";
    };
  };
  */

  # ============================================================================
  # Example 5: Multi-Instance Deployment
  # ============================================================================
  # Run multiple instances (e.g., staging + production)
  # Each instance gets its own systemd service, user, state directory, and database.
  # Defaults are name-aware: user=plinth-<name>, stateDir=/var/lib/plinth-<name>, etc.

  /*
  services.plinth.instances = {
    # Production instance — systemd service: plinth-prod, user: plinth-prod
    prod = {
      port = 3000;
      site.name = "My Website";
      site.baseUrl = "https://example.com";
      apiKeyFile = "/run/secrets/plinth-api-key-prod";
    };

    # Staging instance — systemd service: plinth-staging, user: plinth-staging
    staging = {
      port = 3001;
      package = pkgs.plinth-dev;
      site.name = "My Website (Staging)";
      site.baseUrl = "https://staging.example.com";
      apiKeyFile = "/run/secrets/plinth-api-key-staging";
    };
  };

  # Caddy reverse proxy for both instances
  services.caddy = {
    enable = true;
    virtualHosts."example.com".extraConfig = "reverse_proxy localhost:3000";
    virtualHosts."staging.example.com".extraConfig = "reverse_proxy localhost:3001";
  };
  */

  # ============================================================================
  # Example 6: Advanced Configuration with Extra Environment Variables
  # ============================================================================

  /*
  services.plinth.instances.default = {
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
  # 3. Encrypt secret: age -r <public-key> -o secrets/plinth-api-key.age
  # 4. Configure in NixOS:

  age = {
    secrets = {
      plinth-api-key = {
        file = ./secrets/plinth-api-key.age;
        owner = "plinth-default";
        group = "plinth-default";
      };
      otlp-headers = {
        file = ./secrets/otlp-headers.age;
        owner = "plinth-default";
        group = "plinth-default";
      };
    };
  };

  services.plinth.instances.default = {
    apiKeyFile = config.age.secrets.plinth-api-key.path;
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
      "plinth/api-key" = {
        owner = "plinth-default";
      };
      "plinth/otlp-headers" = {
        owner = "plinth-default";
      };
    };
  };

  services.plinth.instances.default = {
    apiKeyFile = config.sops.secrets."plinth/api-key".path;
    observability = {
      enable = true;
      otlpEndpoint = "https://openobserve.example.com:5081";
    };
    extraEnv = ''
      OTEL_EXPORTER_OTLP_HEADERS=$(cat ${config.sops.secrets."plinth/otlp-headers".path})
    '';
  };
  */

  # ============================================================================
  # Monitoring and Maintenance
  # ============================================================================

  # Automatic database backups (adjust instance name as needed)
  /*
  systemd.services.plinth-backup = {
    description = "Backup Plinth database";
    serviceConfig = {
      Type = "oneshot";
      User = "plinth-default";
      ExecStart = ''
        ${pkgs.bash}/bin/bash -c 'cp /var/lib/plinth-default/database.db /var/lib/plinth-default/database.db.backup.$(date +%Y%m%d)'
      '';
    };
  };

  systemd.timers.plinth-backup = {
    description = "Backup Plinth database daily";
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
