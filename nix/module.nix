{ self }:
{
  lib,
  pkgs,
  config,
  ...
}:
let
  inherit (lib)
    types
    boolToString
    mkOption
    mkEnableOption
    mkIf
    ;

  cfg = config.services.dioxus-fs-demo;

  system = pkgs.stdenv.system;
  dioxus-fs-demo = self.packages.${system}.default;

  wrapper = pkgs.writeShellScriptBin "dioxus-fs-demo" ''
    export RUST_LOG=info
    export PORT=${toString cfg.port}
    exec "${dioxus-fs-demo}/bin/dioxus-fs-demo"
  '';
in
{
  options.services.dioxus-fs-demo = {
    enable = mkEnableOption "dioxus-fs-demo service";
    data_dir = mkOption {
      type = types.str;
      default = "/var/lib/dioxus";
    };
    port = mkOption {
      type = types.int;
      default = 8080;
    };
    secretsFile = mkOption {
      type = types.str;
      example = "/run/secrets/dioxus-fs-demo.env";
      description = lib.mdDoc ''
        Path to an env file containing the secrets used by dioxus-fs-demo.

        Must contain at least:
        - `DATABASE_URL` - The URL to the database.
      '';
    };
  };

  config = mkIf cfg.enable {
    users.users.dioxus = {
      isSystemUser = true;
      description = "Robotica user";
      group = "dioxus";
      createHome = true;
      home = "${cfg.data_dir}";
    };

    users.groups.dioxus = { };

    systemd.services.dioxus-fs-demo = {
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        User = "dioxus";
        ExecStart = "${wrapper}/bin/dioxus-fs-demo";
        EnvironmentFile = cfg.secretsFile;
      };
    };
  };
}
