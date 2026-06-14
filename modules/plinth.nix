{
  config,
  lib,
  pkgs,
  ...
}:
with lib; let
  cfg = config.services.plinth;
  postgresqlPackage = pkgs.postgresql_16.withPackages (ps: [ps.pgvector]);
  projectSiteLib = import ../nix/project-site.nix {
    inherit pkgs lib;
  };

  # Helper to escape TOML string values
  tomlStr = s: ''"${s}"'';

  # Generate plinth.toml from per-instance config
  mkConfigFile = name: icfg:
    pkgs.writeText "plinth-${name}.toml" ''
      [site]
      name = ${tomlStr icfg.site.name}
      tagline = ${tomlStr icfg.site.tagline}
      description = ${tomlStr icfg.site.description}
      lang = ${tomlStr icfg.site.lang}
      default_theme = ${tomlStr icfg.site.defaultTheme}
      base_url = ${tomlStr icfg.site.baseUrl}

      [site.author]
      name = ${tomlStr icfg.site.author.name}
      email = ${tomlStr icfg.site.author.email}

      [site.social]
      github = ${tomlStr icfg.site.social.github}
      gitlab = ${tomlStr icfg.site.social.gitlab}
      codeberg = ${tomlStr icfg.site.social.codeberg}
      mastodon = ${tomlStr icfg.site.social.mastodon}
      bluesky = ${tomlStr icfg.site.social.bluesky}

      [site.footer]
      project_name = ${tomlStr icfg.site.footer.projectName}
      project_url = ${tomlStr icfg.site.footer.projectUrl}

      ${concatMapStringsSep "\n" (item: ''
      [[site.nav]]
      label = ${tomlStr item.label}
      path = ${tomlStr item.path}
      '') icfg.site.nav}

      [pages.home]
      title = ${tomlStr icfg.pages.home.title}
      description = ${tomlStr icfg.pages.home.description}

      [pages.blog]
      title = ${tomlStr icfg.pages.blog.title}
      subtitle = ${tomlStr icfg.pages.blog.subtitle}
      description = ${tomlStr icfg.pages.blog.description}

      [pages.portfolio]
      title = ${tomlStr icfg.pages.portfolio.title}
      subtitle = ${tomlStr icfg.pages.portfolio.subtitle}
      description = ${tomlStr icfg.pages.portfolio.description}

      [pages.about]
      title = ${tomlStr icfg.pages.about.title}
      description = ${tomlStr icfg.pages.about.description}

      [server]
      host = ${tomlStr icfg.host}
      port = ${toString icfg.port}

      [database]
      database_url = ${tomlStr icfg.database.url}

      [observability]
      service_name = ${tomlStr icfg.observability.serviceName}
      log_level = ${tomlStr icfg.observability.logLevel}
      otlp_endpoint = ${tomlStr (if icfg.observability.enable then icfg.observability.otlpEndpoint else "")}
      otlp_headers = ${tomlStr (if icfg.observability.otlpHeaders != null then icfg.observability.otlpHeaders else "")}

      [search]
      default_limit = ${toString icfg.search.defaultLimit}
      related_limit = ${toString icfg.search.relatedLimit}
      min_similarity = ${toString icfg.search.minSimilarity}

      [content]
      words_per_minute = ${toString icfg.content.wordsPerMinute}
      vector_truncation = ${toString icfg.content.vectorTruncation}

      [immich]
      api_url = ${tomlStr icfg.immich.apiUrl}
      api_key = ${tomlStr icfg.immich.apiKey}

      [images]
      cache_max_age = ${toString icfg.images.cacheMaxAge}

      [feeds]
      blog_limit = ${toString icfg.feeds.blogLimit}
      projects_limit = ${toString icfg.feeds.projectsLimit}
    '';

  # Build a content directory derivation for declarative articles
  mkArticlesDir = name: icfg: let
    articleEntries = mapAttrsToList (slug: art: let
      hasSource = art.source != null;
      hasContent = art.content != null;

      # Determine format
      detectedFormat =
        if art.format != null then art.format
        else if hasSource then
          let ext = lib.last (lib.splitString "." (toString art.source));
          in if ext == "typ" then "typst" else "markdown"
        else "markdown";

      ext = if detectedFormat == "typst" then "typ" else "md";

      # Determine source file
      sourceFile =
        if hasSource then art.source
        else pkgs.writeText "${slug}.${ext}" art.content;

    in {
      inherit slug sourceFile detectedFormat ext;
      published = art.published;
    }) icfg.articles;

    # Build the content directory
    buildScript = pkgs.writeShellScript "build-articles-${name}" (''
      set -euo pipefail
      mkdir -p $out/articles

    '' + (concatMapStringsSep "\n" (entry: ''
      # Article: ${entry.slug}
      cp ${entry.sourceFile} $out/articles/${entry.slug}.${entry.ext}
    '' + (optionalString (entry.detectedFormat == "typst") ''
      ${pkgs.typst}/bin/typst compile --format html \
        $out/articles/${entry.slug}.${entry.ext} \
        $out/articles/${entry.slug}.html
    '')) articleEntries) + ''

      # Write manifest.json
      cat > $out/manifest.json <<'MANIFEST_EOF'
      ${builtins.toJSON (listToAttrs (map (entry: {
        name = entry.slug;
        value = {
          slug = entry.slug;
          filename = "${entry.slug}.${entry.ext}";
          format = entry.detectedFormat;
          published = entry.published;
          content_hash = builtins.hashFile "sha256" entry.sourceFile;
        } // (optionalAttrs (entry.detectedFormat == "typst") {
          html_filename = "${entry.slug}.html";
        });
      }) articleEntries))}
      MANIFEST_EOF
    '');
  in pkgs.runCommand "plinth-articles-${name}" {
    nativeBuildInputs = [ pkgs.typst ];
  } ''
    ${buildScript}
  '';

  # Per-instance option definitions
  instanceModule = {name, ...}: {
    options = {
      package = mkOption {
        type = types.package;
        default = pkgs.plinth or (throw "plinth package not found. Add the flake overlay to nixpkgs.overlays");
        defaultText = literalExpression "pkgs.plinth";
        description = "The Plinth package to use.";
      };

      user = mkOption {
        type = types.str;
        default = if name == "default" then "plinth" else "plinth-${name}";
        description = "User account under which the service runs.";
      };

      group = mkOption {
        type = types.str;
        default = if name == "default" then "plinth" else "plinth-${name}";
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
        default = "/var/lib/plinth-${name}";
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
          type = types.enum ["dark" "light"];
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
        name = mkOption {
          type = types.str;
          default = if name == "default" then "plinth" else "plinth_${name}";
          description = "Postgres database name to use.";
        };

        url = mkOption {
          type = types.str;
          default = "postgres:///${if name == "default" then "plinth" else "plinth_${name}"}?host=/run/postgresql";
          defaultText = literalExpression ''"postgres:///plinth?host=/run/postgresql"'';
          description = "Postgres connection URL for Plinth.";
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
          default = "plinth-${name}";
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

      articles = mkOption {
        type = types.attrsOf (types.submodule {
          options = {
            source = mkOption {
              type = types.nullOr types.path;
              default = null;
              description = ''
                Path to article source file (.md or .typ).
                Exactly one of `source` or `content` must be set.
              '';
            };

            content = mkOption {
              type = types.nullOr types.lines;
              default = null;
              description = ''
                Inline article content (markdown or typst with frontmatter).
                Exactly one of `source` or `content` must be set.
              '';
            };

            format = mkOption {
              type = types.nullOr (types.enum ["markdown" "typst"]);
              default = null;
              description = ''
                Content format. Auto-detected from source file extension if null.
                Must be set when using inline `content`.
              '';
            };

            published = mkOption {
              type = types.bool;
              default = true;
              description = "Whether this article should be published.";
            };
          };
        });
        default = {};
        description = ''
          Declarative blog articles, keyed by slug.
          Articles are loaded into Postgres at server startup and coexist
          with API-published articles. Removing an article from this set
          deletes it from the database on the next restart.
        '';
        example = literalExpression ''
          {
            "hello-world" = {
              source = ./posts/hello-world.md;
            };
            "typst-post" = {
              source = ./posts/typst-post.typ;
            };
            "inline-post" = {
              content = "---\ntitle: Quick Note\ntags: [\"meta\"]\n---\n\nHello world.";
              format = "markdown";
            };
          }
        '';
      };
    };
  };

  projectLinkModule = {
    options = {
      label = mkOption {
        type = types.str;
        description = "Project link label.";
      };
      href = mkOption {
        type = types.str;
        description = "Project link URL.";
      };
      kind = mkOption {
        type = types.enum ["person" "project_site" "source" "demo" "docs" "contact" "other"];
        default = "other";
        description = "Standardized link kind.";
      };
    };
  };

  projectDefinitionModule = {name, ...}: {
    options = {
      id = mkOption {
        type = types.str;
        default = name;
        description = "Stable project registry identifier.";
      };
      pname = mkOption {
        type = types.str;
        description = "Nix package name for the generated project site.";
      };
      title = mkOption {
        type = types.str;
        description = "Public project title.";
      };
      description = mkOption {
        type = types.str;
        description = "Public project summary.";
      };
      domain = mkOption {
        type = types.str;
        description = "Canonical project-site domain.";
      };
      url = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Canonical project URL. Defaults to https://<domain>.";
      };
      configPath = mkOption {
        type = types.path;
        description = "Path to the plinth-project TOML definition.";
      };
      staticPaths = mkOption {
        type = types.listOf (types.either types.path (types.submodule {
          options = {
            source = mkOption {
              type = types.path;
              description = "Static source path to copy into the project-site build.";
            };
            target = mkOption {
              type = types.str;
              description = "Target path inside the generated site source tree.";
            };
          };
        }));
        default = [];
        description = "Static files or directories copied into the project-site source tree.";
      };
      docsPackage = mkOption {
        type = types.nullOr types.package;
        default = null;
        description = "Optional documentation package copied under /docs.";
      };
      sourceUrl = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Source repository URL.";
      };
      demoUrl = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Demo URL.";
      };
      links = mkOption {
        type = types.listOf (types.submodule projectLinkModule);
        default = [];
        description = "Additional standardized project links.";
      };
      docsUrl = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Documentation URL used for generated project-site footer links.";
      };
      authorSiteUrl = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Author or personal site URL used for generated project-site footer links.";
      };
      portfolioUrl = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Personal-site portfolio URL for this project.";
      };
      footerLinks = mkOption {
        type = types.listOf (types.submodule {
          options = {
            label = mkOption {
              type = types.str;
              description = "Footer link label.";
            };
            href = mkOption {
              type = types.str;
              description = "Footer link URL.";
            };
          };
        });
        default = [];
        description = "Extra footer links appended when building project sites from this definition.";
      };
      appendStandardFooterLinks = mkOption {
        type = types.bool;
        default = false;
        description = "Whether to append Source, Documentation, Author site, and Portfolio footer links from registry metadata.";
      };
      portfolioSlug = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Portfolio slug. Defaults to the project registry key.";
      };
      portfolioDate = mkOption {
        type = types.nullOr types.str;
        default = null;
        example = "2026-06-07T00:00:00Z";
        description = "Portfolio publication date used by portfolio manifest generation.";
      };
      techStack = mkOption {
        type = types.nullOr (types.listOf types.str);
        default = null;
        description = "Portfolio technology stack used by portfolio manifest generation.";
      };
      imageUrl = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Hosted portfolio image URL.";
      };
      content = mkOption {
        type = types.nullOr types.lines;
        default = null;
        description = "Optional markdown content for generated portfolio manifests.";
      };
      featured = mkOption {
        type = types.bool;
        default = false;
        description = "Whether generated portfolio manifests mark this project as featured.";
      };
      order = mkOption {
        type = types.int;
        default = 0;
        description = "Portfolio display order for generated portfolio manifests.";
      };
      version = mkOption {
        type = types.str;
        default = "0.1.0";
        description = "Version for generated project-site derivations.";
      };
    };
  };

  projectReferenceModule = {
    options = {
      title = mkOption {
        type = types.str;
        description = "Public project title.";
      };
      url = mkOption {
        type = types.str;
        description = "Canonical project URL.";
      };
      source_url = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Source repository URL.";
      };
      demo_url = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Demo URL.";
      };
      links = mkOption {
        type = types.listOf (types.submodule projectLinkModule);
        default = [];
        description = "Additional standardized project links.";
      };
    };
  };
in {
  options.services.plinth = {
    projectRegistry = {
      definitions = mkOption {
        type = types.attrsOf (types.submodule projectDefinitionModule);
        default = {};
        description = "Registry of Plinth project definitions keyed by stable project id.";
      };

      references = mkOption {
        type = types.attrsOf (types.submodule projectReferenceModule);
        default = {};
        description = "Project references generated from services.plinth.projectRegistry.definitions.";
      };
    };

    instances = mkOption {
      type = types.attrsOf (types.submodule instanceModule);
      default = {};
      description = "Named Plinth instances to run. Each instance gets its own systemd service, user, and state directory.";
      example = literalExpression ''
        {
          prod = {
            port = 3000;
            site.name = "My Blog";
            apiKeyFile = "/run/secrets/plinth-api-key";
          };
          staging = {
            port = 3001;
            package = pkgs.plinth-dev;
            site.name = "Staging";
          };
        }
      '';
    };
  };

  config = {
    services.plinth.projectRegistry.references =
      lib.mapAttrs (_: projectSiteLib.projectReferenceFromDefinition)
      (projectSiteLib.mkProjectRegistry cfg.projectRegistry.definitions);

    services.postgresql = mkIf (cfg.instances != {}) {
      enable = true;
      package = postgresqlPackage;
      extensions = ps: [ps.pgvector];
      ensureDatabases = mapAttrsToList (_: icfg: icfg.database.name) cfg.instances;
      ensureUsers = mapAttrsToList (_: icfg: {
        name = icfg.user;
        ensureDBOwnership = true;
      }) cfg.instances;
    };

    assertions = concatLists (mapAttrsToList (name: icfg:
      mapAttrsToList (slug: art: {
        assertion = (art.source != null) != (art.content != null);
        message = "services.plinth.instances.${name}.articles.\"${slug}\": exactly one of `source` or `content` must be set.";
      }) icfg.articles
      ++ mapAttrsToList (slug: art: {
        assertion = art.source != null || art.format != null;
        message = "services.plinth.instances.${name}.articles.\"${slug}\": `format` must be set when using inline `content` (cannot auto-detect).";
      }) (filterAttrs (_: art: art.content != null) icfg.articles)
    ) cfg.instances);

    users.users = mapAttrs' (name: icfg:
      nameValuePair icfg.user {
        isSystemUser = true;
        group = icfg.group;
        description = "Plinth ${name} service user";
        home = icfg.stateDir;
        createHome = true;
      })
    cfg.instances;

    users.groups = mapAttrs' (_: icfg: nameValuePair icfg.group {}) cfg.instances;

    systemd.services = mkMerge (mapAttrsToList (name: icfg: let
      configFile = mkConfigFile name icfg;
      serviceName = if name == "default" then "plinth" else "plinth-${name}";
      postgresInitServiceName = "${serviceName}-postgres-init";
      runtimeSiteRoot = "${icfg.stateDir}/site";
      packageSiteRoot = "${icfg.package}/site";
      prepareSiteRoot = pkgs.writeShellScript "prepare-plinth-site-root-${name}" ''
        set -eu

        install -d -m 0750 -o ${escapeShellArg icfg.user} -g ${escapeShellArg icfg.group} ${escapeShellArg runtimeSiteRoot}

        for asset in \
          pkg \
          favicon.svg \
          favicon-16x16.png \
          favicon-32x32.png \
          favicon-48x48.png \
          favicon-180x180.png \
          favicon-192x192.png \
          favicon-512x512.png \
          htmx.min.js \
          plinth-logo.svg \
          robots.txt
        do
          if [ -e ${escapeShellArg packageSiteRoot}/"$asset" ]; then
            rm -rf ${escapeShellArg runtimeSiteRoot}/"$asset"
            ln -s ${escapeShellArg packageSiteRoot}/"$asset" ${escapeShellArg runtimeSiteRoot}/"$asset"
          fi
        done
      '';
      startScript = pkgs.writeShellScript "start-plinth-${name}" ''
        set -eu

        if [ -n "''${CREDENTIALS_DIRECTORY:-}" ] && [ -r "''${CREDENTIALS_DIRECTORY}/api-key" ]; then
          PLINTH_API_KEY="$(${pkgs.coreutils}/bin/tr -d '\n' < "''${CREDENTIALS_DIRECTORY}/api-key")"
          export PLINTH_API_KEY
        fi

        exec ${icfg.package}/bin/plinth-server "$@"
      '';
      extraEnvFile = pkgs.writeText "plinth-${name}-extra-env" icfg.extraEnv;
    in {
      ${postgresInitServiceName} = {
        description = "Initialize Plinth ${name} Postgres extensions";
        after = ["postgresql.service"];
        wants = ["postgresql.service"];

        serviceConfig = {
          Type = "oneshot";
          User = "postgres";
          Group = "postgres";
          ExecStart = "${postgresqlPackage}/bin/psql ${escapeShellArg icfg.database.name} -c 'CREATE EXTENSION IF NOT EXISTS vector'";
        };
      };

      ${serviceName} = {
        description = "Plinth ${name} - Leptos SSR Application";
        after = ["network.target" "postgresql.service" "${postgresInitServiceName}.service"];
        wants = ["postgresql.service" "${postgresInitServiceName}.service"];
        wantedBy = ["multi-user.target"];

        serviceConfig = {
          Type = "simple";
          User = icfg.user;
          Group = icfg.group;
          Restart = "always";
          RestartSec = "10s";

          # Point server to generated TOML config and Leptos site root
          Environment = [
            "PLINTH_CONFIG=${configFile}"
            "DATABASE_URL=${icfg.database.url}"
            "LEPTOS_SITE_ADDR=${icfg.host}:${toString icfg.port}"
            "LEPTOS_SITE_ROOT=${runtimeSiteRoot}"
          ] ++ lib.optional (icfg.articles != {}) "PLINTH_CONTENT_DIR=${mkArticlesDir name icfg}";
          EnvironmentFile = mkIf (icfg.extraEnv != "") extraEnvFile;

          # Load API key securely if provided
          LoadCredential = mkIf (icfg.apiKeyFile != null) [
            "api-key:${icfg.apiKeyFile}"
          ];

          # Prepare a writable site root for Leptos static route generation.
          ExecStartPre = prepareSiteRoot;

          # Start the server
          ExecStart = startScript;

          # Working directory
          WorkingDirectory = icfg.stateDir;

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
          ReadWritePaths = [icfg.stateDir];

          # Logging
          StandardOutput = "journal";
          StandardError = "journal";
          SyslogIdentifier = "plinth-${name}";
        };

      };
    }) cfg.instances);

    systemd.tmpfiles.rules = mapAttrsToList (_: icfg:
      "d ${icfg.stateDir} 0750 ${icfg.user} ${icfg.group} -")
    cfg.instances;
  };
}
