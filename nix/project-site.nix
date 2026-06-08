{
  pkgs,
  lib,
  plinthProject,
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
in {
  mkProjectSite = {
    pname,
    domain,
    configPath ? "website/plinth-project.toml",
    staticPaths ? [],
    docsPackage ? null,
    version ? "0.1.0",
  }: let
    normalizedStatic = map normalizeStatic staticPaths;
    staticAttrs = lib.listToAttrs (lib.imap0 (index: path: {
        name = "static_${toString index}";
        value = path.source;
      })
      normalizedStatic);
  in
    pkgs.stdenvNoCC.mkDerivation ({
      inherit pname version;
      nativeBuildInputs = [plinthProject];
      config = configPath;
      dontUnpack = true;
      phases = ["buildPhase" "installPhase"];
      buildPhase = ''
        mkdir -p source/website
        cp "$config" source/website/plinth-project.toml
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
