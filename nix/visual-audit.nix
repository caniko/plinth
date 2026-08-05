{
  lib,
  pkgs,
}: let
  json = pkgs.formats.json {};

  defaultViewports = [
    {
      name = "desktop";
      width = 1440;
      height = 900;
    }
    {
      name = "tablet-landscape";
      width = 1024;
      height = 768;
    }
    {
      name = "tablet-portrait";
      width = 768;
      height = 1024;
    }
    {
      name = "mobile";
      width = 390;
      height = 844;
    }
  ];

  normalizeRoute = route:
    if route == "/"
    then "/"
    else let
      trimmed = lib.removeSuffix "/" (lib.removePrefix "/" route);
    in "/${trimmed}/";

  normalizeTarget = target: {
    id = target.id;
    title = target.title;
    kind = target.kind or "static";
    root = target.root or null;
    build_source = target.buildSource or target.build_source or target.siteOutput or target.site_output or null;
    routes = map normalizeRoute (target.routes or ["/"]);
    canonical_url = target.canonicalUrl or target.canonical_url or target.url or null;
    markers = target.markers or [];
    viewports = target.viewports or defaultViewports;
    rubric_preset = target.rubricPreset or target.rubric_preset or "plinth-site-beauty";
    production_gate_mode = target.productionGateMode or target.production_gate_mode or "hard";
  };

  configFromTargets = targets: {
    targets = map normalizeTarget targets;
  };

  shellList = values:
    lib.concatStringsSep " " (map lib.escapeShellArg values);
in rec {
  inherit defaultViewports normalizeRoute normalizeTarget configFromTargets;

  configFile = name: targets:
    json.generate name (configFromTargets targets);

  targetsFromPkl = pklData:
    map normalizeTarget (pklData.targets or []);

  projectTargetFromDefinition = def:
    normalizeTarget {
      id = def.id or def.pname;
      title = def.title;
      kind = "plinth-project";
      root = def.root or ".";
      buildSource = def.siteOutput or ".#site";
      routes = def.routes or ["/"];
      canonicalUrl = def.url or "https://${def.domain}";
      markers = def.markers or [def.title];
      rubricPreset = def.rubricPreset or "plinth-site-beauty";
      productionGateMode = def.productionGateMode or "hard";
    };

  personalTargetFromSite = site:
    normalizeTarget {
      id = site.slug or site.id;
      title = site.name or site.title;
      kind = site.kind or "static";
      root = site.root or null;
      buildSource = site.siteOutput or site.buildSource or null;
      routes = site.routes or ["/"];
      canonicalUrl = site.url or site.canonicalUrl;
      markers = site.markers or [site.name or site.title];
      rubricPreset = site.rubricPreset or "plinth-site-beauty";
      productionGateMode = site.productionGateMode or "hard";
    };

  personalTargetsFromSites = sites:
    map personalTargetFromSite sites;

  mkProjectAuditCommand = {
    config ? "website/plinth-project.toml",
    out ? "website/public",
    report ? "target/site-audit/site-report.json",
    screenshots ? "target/site-audit",
    routes ? [],
    fakeAi ? false,
    skipAi ? false,
    sharedCapture ? true,
    extraArgs ? [],
  }: ''
    plinth-project audit site \
      --config ${lib.escapeShellArg config} \
      --out ${lib.escapeShellArg out} \
      --report ${lib.escapeShellArg report} \
      --screenshots ${lib.escapeShellArg screenshots} \
      ${lib.optionalString sharedCapture "--shared-capture"} \
      ${lib.optionalString fakeAi "--fake-ai"} \
      ${lib.optionalString skipAi "--skip-ai"} \
      ${lib.concatMapStringsSep " " (route: "--route ${lib.escapeShellArg route}") routes} \
      ${shellList extraArgs}
  '';

  mkTargetsMarkerCheck = {
    name,
    targets,
    expectedIds ? map (target: target.id) targets,
  }: let
    config = configFile "${name}.json" targets;
  in
    pkgs.runCommand name {} ''
      grep -q '"rubric_preset": "plinth-site-beauty"' ${config}
      ${lib.concatMapStringsSep "\n" (id: "grep -q '\"id\": \"${id}\"' ${config}") expectedIds}
      cp ${config} $out
    '';
}
