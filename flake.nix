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

        buildInputs = with pkgs; [ systemd hidapi udev ];

        nativeBuildInputs = with pkgs; [ pkg-config ];

        rustToolchain = pkgs.rust-bin.stable.latest.default;
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };

        streamdeck-commander = rustPlatform.buildRustPackage {
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
                description =
                  "User account under which StreamDeck Commander runs";
              };

              group = mkOption {
                type = types.str;
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

              systemd.services.streamdeck-commander = {
                description = "StreamDeck Commander";
                after = [ "graphical-session.target" ];
                wantedBy = [ "default.target" ];

                environment = {
                  STREAMDECK_CONFIG = if cfg.configFile != null then
                    cfg.configFile
                  else
                    configFile;
                  RUST_LOG = "debug";
                };

                serviceConfig = {
                  Type = "simple";
                  User = cfg.user;
                  Group = cfg.group;
                  ExecStart = "${package}/bin/streamdeck-commander";
                  Restart = "on-failure";
                  RestartSec = "5s";

                  # Minimal security - disable most hardening for device access
                  SupplementaryGroups = [ cfg.group ];
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
