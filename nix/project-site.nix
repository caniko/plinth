{
  pkgs,
  lib,
  plinthProject ? null,
}: let
  shellQuote = lib.escapeShellArg;

  normalizeStatic = path:
    if builtins.isAttrs path
    then path
    else {
      source = path;
      target = baseNameOf path;
    };

  copyStaticCommands = paths:
    lib.concatImapStringsSep "\n" (
      index: path: let
        item = normalizeStatic path;
        target = item.target;
        input = "$static_${toString (index - 1)}";
      in ''
        mkdir -p source/${shellQuote (dirOf target)}
        if [ -d ${input} ]; then
          mkdir -p source/${shellQuote target}
          cp -rL --no-preserve=mode ${input}/. source/${shellQuote target}/
        else
          cp -L --no-preserve=mode ${input} source/${shellQuote target}
        fi
      ''
    )
    paths;

  canonicalUrl = def:
    if def ? url && def.url != null
    then def.url
    else "https://${def.domain}";

  compactAttrs = attrs: lib.filterAttrs (_: value: value != null) attrs;

  required = def: field:
    if builtins.hasAttr field def && def.${field} != null && def.${field} != []
    then def.${field}
    else throw "project definition `${def.id or def.pname}` is missing required portfolio field `${field}`";

  tomlScalar = value:
    if builtins.isString value
    then builtins.toJSON value
    else if builtins.isBool value
    then
      if value
      then "true"
      else "false"
    else if builtins.isInt value
    then toString value
    else if builtins.isList value
    then "[${lib.concatMapStringsSep ", " tomlScalar value}]"
    else throw "unsupported portfolio TOML value: ${builtins.typeOf value}";

  tomlAttrs = attrs:
    lib.concatStringsSep "\n" (lib.mapAttrsToList (name: value: "${name} = ${tomlScalar value}") attrs);

  footerLinksToml = links:
    lib.concatMapStringsSep "\n" (link: ''
      [[footer_links]]
      label = ${tomlScalar link.label}
      href = ${tomlScalar link.href}
    '')
    links;

  portfolioLinksToml = links:
    lib.concatMapStringsSep "\n" (link: ''
      [[links]]
      label = ${tomlScalar link.label}
      href = ${tomlScalar link.href}
      kind = ${tomlScalar (link.kind or "other")}
    '')
    links;
in rec {
  mkProjectDefinition = {
    id,
    pname,
    title,
    description,
    domain,
    configPath,
    staticPaths ? [],
    docsPackage ? null,
    sourceUrl ? null,
    demoUrl ? null,
    links ? [],
    version ? "0.1.0",
    url ? null,
    docsUrl ? null,
    authorSiteUrl ? null,
    portfolioUrl ? null,
    footerLinks ? [],
    appendStandardFooterLinks ? false,
    portfolioSlug ? null,
    portfolioDate ? null,
    techStack ? null,
    imageUrl ? null,
    content ? null,
    featured ? false,
    order ? 0,
  }:
    compactAttrs {
      inherit
        id
        pname
        title
        description
        domain
        configPath
        staticPaths
        docsPackage
        sourceUrl
        demoUrl
        links
        version
        url
        docsUrl
        authorSiteUrl
        portfolioUrl
        footerLinks
        appendStandardFooterLinks
        portfolioSlug
        portfolioDate
        techStack
        imageUrl
        content
        featured
        order
        ;
    };

  mkProjectRegistry = definitions:
    lib.mapAttrs (
      name: definition:
        definition
        // {
          id = definition.id or name;
          url = canonicalUrl definition;
          staticPaths = definition.staticPaths or [];
          docsPackage = definition.docsPackage or null;
          sourceUrl = definition.sourceUrl or null;
          demoUrl = definition.demoUrl or null;
          links = definition.links or [];
          version = definition.version or "0.1.0";
          docsUrl =
            if definition ? docsUrl
            then definition.docsUrl
            else if definition ? docsPackage && definition.docsPackage != null
            then "/docs/"
            else null;
          footerLinks = definition.footerLinks or [];
          appendStandardFooterLinks = definition.appendStandardFooterLinks or false;
          authorSiteUrl = definition.authorSiteUrl or null;
          portfolioUrl = definition.portfolioUrl or null;
          portfolioSlug = definition.portfolioSlug or name;
          portfolioDate = definition.portfolioDate or null;
          techStack = definition.techStack or null;
          imageUrl = definition.imageUrl or null;
          content = definition.content or null;
          featured = definition.featured or false;
          order = definition.order or 0;
        }
    )
    definitions;

  projectReferenceFromDefinition = def:
    compactAttrs {
      title = def.title;
      url = canonicalUrl def;
      source_url = def.sourceUrl or null;
      demo_url = def.demoUrl or null;
    }
    // lib.optionalAttrs ((def.links or []) != []) {
      links = def.links;
    };

  standardFooterLinksFromDefinition = def:
    lib.optional ((def.sourceUrl or null) != null) {
      label = "Source";
      href = def.sourceUrl;
    }
    ++ lib.optional ((def.docsUrl or null) != null) {
      label = "Documentation";
      href = def.docsUrl;
    }
    ++ lib.optional ((def.authorSiteUrl or null) != null) {
      label = "Author site";
      href = def.authorSiteUrl;
    }
    ++ lib.optional ((def.portfolioUrl or null) != null) {
      label = "Portfolio";
      href = def.portfolioUrl;
    };

  portfolioManifestFromDefinition = def:
    compactAttrs {
      slug = def.portfolioSlug or def.id;
      title = def.title;
      description = def.description;
      tech_stack = required def "techStack";
      date = required def "portfolioDate";
      content = def.content or null;
      link = def.sourceUrl or null;
      demo = def.demoUrl or null;
      project_url = canonicalUrl def;
      links = def.links or [];
      image_url = def.imageUrl or null;
      featured = def.featured or false;
      order = def.order or 0;
      content_format = "markdown";
    };

  portfolioManifestTomlFromDefinition = def: let
    manifest = portfolioManifestFromDefinition def;
    links = manifest.links or [];
    scalarManifest = removeAttrs manifest ["links"];
  in ''
    ${tomlAttrs scalarManifest}
    ${portfolioLinksToml links}
  '';

  portfolioManifestFileFromDefinition = def:
    pkgs.writeText "${def.id or def.pname}-portfolio.toml" (portfolioManifestTomlFromDefinition def);

  mkProjectSite = {
    pname,
    domain,
    configPath ? "website/plinth-project.toml",
    staticPaths ? [],
    docsPackage ? null,
    footerLinks ? [],
    version ? "0.1.0",
  }: let
    normalizedStatic = map normalizeStatic staticPaths;
    staticAttrs = lib.listToAttrs (lib.imap0 (index: path: {
        name = "static_${toString index}";
        value = path.source;
      })
      normalizedStatic);
  in
    assert plinthProject != null;
    pkgs.stdenvNoCC.mkDerivation ({
      inherit pname version;
      nativeBuildInputs = [plinthProject];
      config = configPath;
      dontUnpack = true;
      phases = ["buildPhase" "installPhase"];
      buildPhase = ''
        mkdir -p source/website
        cp "$config" source/website/plinth-project.toml
        ${lib.optionalString (footerLinks != []) ''
          chmod u+w source/website/plinth-project.toml
          cat >> source/website/plinth-project.toml <<'PLINTH_PROJECT_FOOTER_LINKS'

          ${footerLinksToml footerLinks}
          PLINTH_PROJECT_FOOTER_LINKS
        ''}
        ${copyStaticCommands normalizedStatic}
        plinth-project build \
          --config source/website/plinth-project.toml \
          --out public
      '';
      installPhase = ''
        mkdir -p $out
        cp -rL --no-preserve=mode public/. $out/
        ${lib.optionalString (docsPackage != null) ''
          mkdir -p $out/docs
          cp -rL --no-preserve=mode ${docsPackage}/. $out/docs/
        ''}
        printf '%s\n' ${shellQuote domain} > $out/.domains
      '';
    }
    // staticAttrs);

  mkProjectSiteFromDefinition = def:
    assert plinthProject != null;
      mkProjectSite {
        inherit (def) pname domain configPath;
        staticPaths = def.staticPaths or [];
        docsPackage = def.docsPackage or null;
        footerLinks =
          (def.footerLinks or [])
          ++ lib.optionals (def.appendStandardFooterLinks or false) (standardFooterLinksFromDefinition def);
        version = def.version or "0.1.0";
      };

  mkDeployPagesApp = {
    domain,
    finalUrl ? "https://${domain}/",
    flakeRef ? "#site",
    branch ? "pages",
  }: let
    script = pkgs.writeShellApplication {
      name = "deploy-pages";
      runtimeInputs = [
        pkgs.coreutils
        pkgs.findutils
        pkgs.git
        pkgs.nix
      ];
      text = ''
        set -euo pipefail

        repo_root="$(git rev-parse --show-toplevel)"
        remote="''${DEPLOY_REMOTE:-origin}"
        branch=${shellQuote branch}
        commit_msg="Deploy site $(date -u +%Y-%m-%dT%H:%M:%SZ)"

        echo ":: Building site..."
        site_path="$(nix build "$repo_root${flakeRef}" --no-link --print-out-paths)"
        echo "   Built: $site_path"
        grep -qx ${shellQuote domain} "$site_path/.domains"

        work_dir="$(mktemp -d)"
        trap 'rm -rf "$work_dir"' EXIT

        echo ":: Preparing $branch branch..."
        if git ls-remote --exit-code "$remote" "refs/heads/$branch" >/dev/null 2>&1; then
          git clone --depth 1 --branch "$branch" --single-branch \
            "$(git remote get-url "$remote")" "$work_dir" --quiet
          if [ "$remote" != "origin" ]; then
            git -C "$work_dir" remote rename origin "$remote"
          fi
        else
          git init "$work_dir" --quiet
          git -C "$work_dir" checkout --orphan "$branch"
          git -C "$work_dir" remote add "$remote" "$(git remote get-url "$remote")"
        fi

        echo ":: Copying site output..."
        find "$work_dir" -mindepth 1 -maxdepth 1 ! -name .git -exec rm -rf {} +
        cp -rL --no-preserve=mode "$site_path/." "$work_dir/"

        echo ":: Committing and pushing..."
        cd "$work_dir"
        git config user.name "''${GIT_AUTHOR_NAME:-forgejo-actions}"
        git config user.email "''${GIT_AUTHOR_EMAIL:-forgejo-actions@noreply.codeberg.org}"
        git add --all
        if git diff --cached --quiet; then
          echo "   No changes to deploy."
          exit 0
        fi

        git commit -m "$commit_msg" --quiet
        git push "$remote" "HEAD:$branch" --force --quiet

        echo ":: Deployed to ${finalUrl}"
      '';
    };
  in {
    type = "app";
    program = "${script}/bin/deploy-pages";
  };
}
