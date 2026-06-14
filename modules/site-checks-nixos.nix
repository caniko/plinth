{
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.services.plinth.siteChecks;
  siteChecksLib = import ../nix/site-checks.nix {
    inherit lib pkgs;
  };
  configFile = siteChecksLib.configFile "plinth-site-checks.toml" cfg.targets;
  wrappedPackage = pkgs.symlinkJoin {
    name = "plinth-cli-site-checks";
    paths = [cfg.package];
    nativeBuildInputs = [pkgs.makeWrapper];
    postBuild = ''
      wrapProgram "$out/bin/plinth" \
        --set-default PLINTH_SITE_CHECK_CONFIG ${lib.escapeShellArg configFile}
    '';
  };
in {
  options.services.plinth.siteChecks = {
    enable = lib.mkEnableOption "Plinth site check registry";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.plinth-cli or (throw "plinth-cli package not found. Set services.plinth.siteChecks.package.");
      defaultText = lib.literalExpression "pkgs.plinth-cli";
      description = "Plinth CLI package to wrap with the site-check config path.";
    };

    targets = lib.mkOption {
      type = lib.types.listOf (lib.types.submodule siteChecksLib.targetModule);
      default = [];
      description = "Registered Plinth and static site check targets.";
    };
  };

  config = lib.mkIf cfg.enable {
    assertions = [
      {
        assertion = cfg.targets != [];
        message = "services.plinth.siteChecks.targets must not be empty when site checks are enabled.";
      }
    ];

    environment.systemPackages = [wrappedPackage];
    environment.etc."plinth/site-checks.toml".source = configFile;
    environment.sessionVariables.PLINTH_SITE_CHECK_CONFIG = toString configFile;
  };
}
