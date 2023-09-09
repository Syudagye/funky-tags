{
  description = "A thing i did for listing funky usernames found online";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, naersk }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = f: nixpkgs.lib.genAttrs supportedSystems (system: f system);
    in
    {
      packages = forAllSystems
        (system:
          let
            pkgs = import nixpkgs {
              inherit system;
            };
            naersk' = pkgs.callPackage naersk { };
          in
          {
            default = naersk'.buildPackage {
              src = ./.;
            };
          });

      nixosModules.default = { lib, pkgs, config, ... }:
        with lib;
        let
          cfg = config.services.funky-tags;
          pkg = self.packages.${pkgs.system}.default;
        in
        {
          options.services.funky-tags = {
            enable = mkEnableOption "funky-tags web app";
            data = mkOption {
              type = types.str;
              description = "Location of the data directory";
            };
            port = mkOption {
              type = types.int;
              description = "The local port the server will listen to";
              default = 8000;
            };
            secretFile = mkOption {
              type = types.str;
              description = ''
                File containing the secret key for signing JWT tokens.
                Defaults to `$\{services.funky-tags.data\}/secret`
              '';
              default = "${cfg.data}/secret";
            };
            enableNginx = mkOption {
              type = types.bool;
              description = "Enable nginx configuration";
              default = true;
            };
            vhost = mkOption {
              type = types.str;
              description = "nginx virtual host";
            };
          };

          config = mkIf cfg.enable {
            systemd.services.funky-tags = {
              serviceConfig.ExecStart = "${pkg}/bin/funky-tags";
              environment = {
                DATABASE_URL = "sqlite:${cfg.data}/gamertags.db";
                JWT_SECRET_FILE = cfg.secretFile;
                PORT = "${toString cfg.port}";
              };
              serviceConfig.StandardOutput = "journal+console";
            };

            services.nginx.virtualHosts.${cfg.vhost} = mkIf cfg.enableNginx {
              forceSSL = true;
              enableACME = true;
              locations."/" = {
                proxyPass = "http://127.0.0.1:${toString cfg.port}";
              };
            };
          };
        };
    };
}
