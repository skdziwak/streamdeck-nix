{
  description =
    "StreamDeck Commander - YAML-configured Stream Deck command launcher";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        buildInputs = with pkgs; [ systemd hidapi ];

        nativeBuildInputs = with pkgs; [ pkg-config ];

        streamdeck-commander = pkgs.rustPlatform.buildRustPackage {
          pname = "streamdeck-commander";
          version = "0.1.0";

          src = ./.;

          cargoLock = { lockFile = ./Cargo.lock; };

          inherit buildInputs nativeBuildInputs;

          meta = with pkgs.lib; {
            description = "YAML-configured Stream Deck command launcher";
            homepage = "https://github.com/yourusername/streamdeck-commander";
            license = licenses.mit;
            maintainers = [ ];
          };
        };
      in {
        packages = {
          default = streamdeck-commander;
          streamdeck-commander = streamdeck-commander;
        };

        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;

          packages = with pkgs; [
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" "rust-analyzer" ];
            })
            cargo-watch
            cargo-edit
          ];

          shellHook = ''
            echo "StreamDeck Commander development environment"
            echo "Run 'cargo build' to build the project"
            echo "Run 'cargo run' to run the application"
          '';
        };
      }) // {
        nixosModules.default = { config, lib, pkgs, ... }:
          with lib;
          let
            cfg = config.services.streamdeck-commander;

            configFormat = pkgs.formats.yaml { };

            configFile =
              configFormat.generate "streamdeck-commander-config.yaml" {
                menu = cfg.menu;
              };

            package = self.packages.${pkgs.system}.streamdeck-commander;
          in {
            options.services.streamdeck-commander = {
              enable = mkEnableOption "StreamDeck Commander service";

              user = mkOption {
                type = types.str;
                default = "streamdeck";
                description =
                  "User account under which StreamDeck Commander runs";
              };

              group = mkOption {
                type = types.str;
                default = "streamdeck";
                description = "Group under which StreamDeck Commander runs";
              };

              menu = mkOption {
                type = types.submodule {
                  options = {
                    name = mkOption {
                      type = types.str;
                      default = "Main Menu";
                      description = "Name of the main menu";
                    };

                    buttons = mkOption {
                      type = types.listOf (types.attrs);
                      default = [ ];
                      example = literalExpression ''
                        [
                          {
                            type = "command";
                            name = "List Files";
                            command = "ls";
                            args = [ "-la" ];
                          }
                          {
                            type = "menu";
                            name = "Git Commands";
                            buttons = [
                              {
                                type = "command";
                                name = "Git Status";
                                command = "git";
                                args = [ "status" ];
                              }
                              {
                                type = "back";
                              }
                            ];
                          }
                        ]
                      '';
                      description = "List of buttons in the menu";
                    };
                  };
                };
                default = {
                  name = "Main Menu";
                  buttons = [ ];
                };
                description = "StreamDeck Commander menu configuration";
              };

              extraConfig = mkOption {
                type = types.attrs;
                default = { };
                description =
                  "Extra configuration to merge with the generated config";
              };

              configFile = mkOption {
                type = types.nullOr types.path;
                default = null;
                description =
                  "Path to a custom configuration file. If set, this overrides the menu option.";
              };
            };

            config = mkIf cfg.enable {
              # Create user and group if they don't exist
              users.users = optionalAttrs (cfg.user == "streamdeck") {
                streamdeck = {
                  isSystemUser = true;
                  group = cfg.group;
                  description = "StreamDeck Commander service user";
                  extraGroups = [ "input" "plugdev" ];
                };
              };

              users.groups =
                optionalAttrs (cfg.group == "streamdeck") { streamdeck = { }; };

              # udev rules for Stream Deck access
              services.udev.extraRules = ''
                # Elgato Stream Deck Original
                SUBSYSTEM=="usb", ATTRS{idVendor}=="0fd9", ATTRS{idProduct}=="0060", MODE="0660", GROUP="${cfg.group}", TAG+="uaccess"
                # Elgato Stream Deck Original V2
                SUBSYSTEM=="usb", ATTRS{idVendor}=="0fd9", ATTRS{idProduct}=="006d", MODE="0660", GROUP="${cfg.group}", TAG+="uaccess"
                # Elgato Stream Deck Mini
                SUBSYSTEM=="usb", ATTRS{idVendor}=="0fd9", ATTRS{idProduct}=="0063", MODE="0660", GROUP="${cfg.group}", TAG+="uaccess"
                # Elgato Stream Deck XL
                SUBSYSTEM=="usb", ATTRS{idVendor}=="0fd9", ATTRS{idProduct}=="006c", MODE="0660", GROUP="${cfg.group}", TAG+="uaccess"
                # Elgato Stream Deck MK.2
                SUBSYSTEM=="usb", ATTRS{idVendor}=="0fd9", ATTRS{idProduct}=="0080", MODE="0660", GROUP="${cfg.group}", TAG+="uaccess"
                # Elgato Stream Deck Pedal
                SUBSYSTEM=="usb", ATTRS{idVendor}=="0fd9", ATTRS{idProduct}=="0086", MODE="0660", GROUP="${cfg.group}", TAG+="uaccess"
                # Elgato Stream Deck Plus
                SUBSYSTEM=="usb", ATTRS{idVendor}=="0fd9", ATTRS{idProduct}=="0084", MODE="0660", GROUP="${cfg.group}", TAG+="uaccess"
              '';

              systemd.services.streamdeck-commander = {
                description = "StreamDeck Commander";
                after = [ "graphical-session.target" ];
                wantedBy = [ "default.target" ];

                environment = {
                  STREAMDECK_CONFIG = if cfg.configFile != null then
                    cfg.configFile
                  else
                    configFile;
                  RUST_LOG = "info";
                };

                serviceConfig = {
                  Type = "simple";
                  User = cfg.user;
                  Group = cfg.group;
                  ExecStart = "${package}/bin/streamdeck-commander";
                  Restart = "on-failure";
                  RestartSec = "5s";

                  # Security hardening
                  NoNewPrivileges = true;
                  PrivateTmp = true;
                  ProtectSystem = "strict";
                  ProtectHome = "read-only";
                  ReadWritePaths = [ ];

                  # Device access
                  DeviceAllow = [ "/dev/hidraw*" "/dev/usb/*" ];
                  SupplementaryGroups = [ "input" "plugdev" ];
                };
              };
            };
          };

        overlays.default = final: prev: {
          streamdeck-commander =
            self.packages.${prev.system}.streamdeck-commander;
        };
      };
}
