{
  self,
  nixpkgs,
  pkgs,
}: let
  inherit (pkgs) lib;
  system = pkgs.stdenv.hostPlatform.system;
  manifests = pkgs.writeText "test-manifests.json" "[]";

  evalInstance = instanceCfg:
    nixpkgs.lib.nixosSystem {
      inherit system;
      modules = [
        self.nixosModules.plinth
        {
          system.stateVersion = "25.11";
          services.plinth.instances.can =
            {
              package = pkgs.hello;
              user = "plinth_can";
              group = "plinth_can";
              port = 3010;
              site.baseUrl = "https://example.invalid";
            }
            // instanceCfg;
        }
      ];
    };

  moduleFailures = eval:
    builtins.filter (a: !a.assertion && lib.hasPrefix "services.plinth." a.message) eval.config.assertions;

  synced = evalInstance {
    apiKeyFile = "/run/agenix/plinth-can-api-key";
    immich.apiKeyFile = "/run/agenix/immich-api-key";
    autoPublishPortfolio = true;
    portfolioManifestsJson = manifests;
  };
  syncedFailed = moduleFailures synced;
  serverUnit = synced.config.systemd.services.plinth-can;
  syncUnit = synced.config.systemd.services.plinth-can-portfolio-sync;

  unsynced = evalInstance {
    apiKeyFile = "/run/agenix/plinth-can-api-key";
    autoPublishPortfolio = false;
  };
  unsyncedFailed = moduleFailures unsynced;

  storeApiKey = evalInstance {
    apiKeyFile = "/nix/store/00000000000000000000000000000000-secret";
  };
  storeApiKeyFailed = moduleFailures storeApiKey;

  storeImmichKey = evalInstance {
    immich.apiKeyFile = "/nix/store/00000000000000000000000000000000-secret";
  };
  storeImmichKeyFailed = moduleFailures storeImmichKey;
in
  assert lib.assertMsg (syncedFailed == []) "valid Plinth instance failed assertions";
  assert lib.assertMsg (unsyncedFailed == []) "unsynced Plinth instance failed assertions";
  assert lib.assertMsg (serverUnit.serviceConfig.ExecStart == "${pkgs.hello}/bin/plinth-server")
  "plinth-can must execute plinth-server directly without a shell wrapper";
  assert lib.assertMsg (!(serverUnit.serviceConfig ? ExecStartPost))
  "plinth-can must not define ExecStartPost on Type=simple";
  assert lib.assertMsg (lib.elem "plinth-can-portfolio-sync.service" serverUnit.wants)
  "plinth-can must want plinth-can-portfolio-sync.service when autoPublishPortfolio is true";
  assert lib.assertMsg (serverUnit.serviceConfig.LoadCredential
    == [
      "api-key:/run/agenix/plinth-can-api-key"
      "immich-api-key:/run/agenix/immich-api-key"
    ]) "plinth-can LoadCredential mismatch";
  assert lib.assertMsg (lib.elem "PLINTH_API_KEY_FILE=%d/api-key" serverUnit.serviceConfig.Environment)
  "plinth-can missing PLINTH_API_KEY_FILE=%d/api-key";
  assert lib.assertMsg (lib.elem "IMMICH_API_KEY_FILE=%d/immich-api-key" serverUnit.serviceConfig.Environment)
  "plinth-can missing IMMICH_API_KEY_FILE=%d/immich-api-key";
  assert lib.assertMsg (syncUnit.after == ["plinth-can.service"])
  "plinth-can-portfolio-sync must run after plinth-can.service";
  assert lib.assertMsg (syncUnit.serviceConfig.Type == "oneshot")
  "plinth-can-portfolio-sync must be Type=oneshot";
  assert lib.assertMsg (syncUnit.serviceConfig.LoadCredential == ["api-key:/run/agenix/plinth-can-api-key"])
  "plinth-can-portfolio-sync LoadCredential mismatch";
  assert lib.assertMsg (lib.elem "PLINTH_API_URL=http://127.0.0.1:3010" syncUnit.serviceConfig.Environment)
  "plinth-can-portfolio-sync must talk to local listener";
  assert lib.assertMsg (lib.elem "PLINTH_API_KEY_FILE=%d/api-key" syncUnit.serviceConfig.Environment)
  "plinth-can-portfolio-sync missing PLINTH_API_KEY_FILE=%d/api-key";
  assert lib.assertMsg (lib.hasPrefix "-${pkgs.hello}/bin/plinth portfolio sync --retry-attempts 30 --retry-delay-secs 2 " syncUnit.serviceConfig.ExecStart)
  "plinth-can-portfolio-sync must invoke plinth portfolio sync directly with non-fatal prefix and retries";
  assert lib.assertMsg (!(unsynced.config.systemd.services ? plinth-can-portfolio-sync))
  "unsynced instance must not create portfolio-sync unit";
  assert lib.assertMsg (storeApiKeyFailed != []) "store apiKeyFile must fail assertions";
  assert lib.assertMsg (storeImmichKeyFailed != []) "store immich.apiKeyFile must fail assertions";
    pkgs.runCommand "plinth-module-eval" {} "touch $out"
