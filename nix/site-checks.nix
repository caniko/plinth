{
  lib,
  pkgs,
}: let
  toml = pkgs.formats.toml {};

  normalizeUrl = target:
    target
    // {
      url =
        target.url
        or (if target ? domain then "https://${target.domain}" else null);
    };

  renderTarget = target: let
    normalized = normalizeUrl target;
  in
    {
      inherit (normalized) id title url kind;
      routes = normalized.routes or [];
      markers = normalized.markers or [];
      expected_status = normalized.expectedStatus or normalized.expected_status or 200;
      follow_redirects = normalized.followRedirects or normalized.follow_redirects or true;
    };

  targetModule = {name, ...}: {
    options = {
      id = lib.mkOption {
        type = lib.types.str;
        default = name;
        description = "Stable site-check target identifier.";
      };
      title = lib.mkOption {
        type = lib.types.str;
        description = "Human-readable site title.";
      };
      url = lib.mkOption {
        type = lib.types.str;
        description = "Canonical base URL to check.";
      };
      kind = lib.mkOption {
        type = lib.types.enum ["plinth" "static"];
        description = "Site checker target kind.";
      };
      routes = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [];
        description = "Additional or overriding route paths to check.";
      };
      markers = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [];
        description = "Text markers that must appear in checked route responses.";
      };
      expectedStatus = lib.mkOption {
        type = lib.types.ints.between 100 599;
        default = 200;
        description = "Expected HTTP status for route checks.";
      };
      followRedirects = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = "Whether the checker should follow redirects for this target.";
      };
    };
  };
in rec {
  inherit targetModule;

  renderConfig = targets: {
    targets = map renderTarget targets;
  };

  configFile = name: targets:
    toml.generate name (renderConfig targets);

  projectTargetFromDefinition = def: {
    id = def.id or def.pname;
    title = def.title;
    url = def.url or "https://${def.domain}";
    kind = "static";
    routes = ["/"];
    markers = [def.title];
  };

  projectTargetsFromDefinitions = definitions:
    lib.mapAttrsToList (_: projectTargetFromDefinition) definitions;

  plinthTargetFromInstance = name: instance:
    if ((instance.site.baseUrl or "") != "")
    then {
      id = name;
      title = instance.site.name;
      url = instance.site.baseUrl;
      kind = "plinth";
      markers = [instance.site.name];
    }
    else {};

  plinthTargetsFromInstances = instances:
    lib.filter (target: target != {})
    (lib.mapAttrsToList plinthTargetFromInstance instances);
}
