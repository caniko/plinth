{
  pkgs,
  lib,
  plinthProgress,
}: {
  mkProgressPage = {
    src,
    config ? null,
    version ? null,
    pname ? "progress-page",
  }:
    assert builtins.isPath src || builtins.isString src;
    assert version == null || builtins.isString version;
      pkgs.stdenvNoCC.mkDerivation ({
          inherit pname;
          version = "1";
          nativeBuildInputs = [plinthProgress];
          dontUnpack = true;
          progressSource = src;
          phases = ["buildPhase"];
          buildPhase = ''
            ${lib.optionalString (version != null) ''
              plinth-progress check --version "$progressVersion" "$progressSource"
            ''}
            mkdir -p "$out"
            plinth-progress build "$progressSource" --out "$out/index.html" ${lib.optionalString (config != null) ''--config "$progressConfig"''}
          '';
        }
        // lib.optionalAttrs (version != null) {progressVersion = version;}
        // lib.optionalAttrs (config != null) {progressConfig = config;});
}
