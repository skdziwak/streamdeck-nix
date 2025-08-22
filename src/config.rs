use anyhow::Result;
use serde::{Deserialize, Serialize};

// Embed config.yaml at compile time if it exists
const EMBEDDED_CONFIG: &str = include_str!("../config.yaml");

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub menu: Menu,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Menu {
    pub name: String,
    pub buttons: Vec<Button>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Button {
    Command {
        name: String,
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        icon: Option<String>,
    },
    Menu {
        name: String,
        buttons: Vec<Button>,
        #[serde(default)]
        icon: Option<String>,
    },
    Back {
        #[serde(default = "default_back_name")]
        name: String,
        #[serde(default)]
        icon: Option<String>,
    },
    Toggle {
        name: String,
        #[serde(flatten)]
        mode: ToggleMode,
        #[serde(default)]
        probe_command: Option<String>,
        #[serde(default)]
        probe_args: Vec<String>,
        #[serde(default)]
        on_icon: Option<String>,
        #[serde(default)]
        off_icon: Option<String>,
        #[serde(default)]
        icon: Option<String>, // Fallback icon when state is unknown
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum ToggleMode {
    /// Single command that toggles between states
    Single {
        command: String,
        #[serde(default)]
        args: Vec<String>,
    },
    /// Separate commands for on and off states
    Separate {
        on_command: String,
        #[serde(default)]
        on_args: Vec<String>,
        off_command: String,
        #[serde(default)]
        off_args: Vec<String>,
    },
}

fn default_back_name() -> String {
    "Back".to_string()
}

pub fn load_config() -> Result<Config> {
    tracing::info!("Using embedded configuration");
    let config: Config = serde_yaml::from_str(EMBEDDED_CONFIG)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let yaml = r#"
menu:
  name: "Main Menu"
  buttons:
    - type: command
      name: "List Files"
      command: "ls"
      args: ["-la"]
    - type: menu
      name: "Git Commands"
      buttons:
        - type: command
          name: "Git Status"
          command: "git"
          args: ["status"]
        - type: command
          name: "Git Log"
          command: "git"
          args: ["log", "--oneline", "-10"]
        - type: back
    - type: command
      name: "System Info"
      command: "uname"
      args: ["-a"]
    - type: toggle
      name: "WiFi Toggle"
      mode: single
      command: "nmcli"
      args: ["radio", "wifi"]
      probe_command: "nmcli"
      probe_args: ["radio", "wifi"]
      on_icon: "wifi"
      off_icon: "wifi_off"
    - type: toggle
      name: "VPN Toggle" 
      mode: separate
      on_command: "nmcli"
      on_args: ["connection", "up", "vpn"]
      off_command: "nmcli"
      off_args: ["connection", "down", "vpn"]
      probe_command: "nmcli"
      probe_args: ["connection", "show", "--active"]
"#;

        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.menu.name, "Main Menu");
        assert_eq!(config.menu.buttons.len(), 5);
        
        // Check first button
        match &config.menu.buttons[0] {
            Button::Command { name, command, .. } => {
                assert_eq!(name, "List Files");
                assert_eq!(command, "ls");
            }
            _ => panic!("Expected command button"),
        }
        
        // Check nested menu
        match &config.menu.buttons[1] {
            Button::Menu { name, buttons, .. } => {
                assert_eq!(name, "Git Commands");
                assert_eq!(buttons.len(), 3);
            }
            _ => panic!("Expected menu button"),
        }
        
        // Check toggle button with single mode
        match &config.menu.buttons[3] {
            Button::Toggle { name, mode, probe_command, on_icon, off_icon, .. } => {
                assert_eq!(name, "WiFi Toggle");
                match mode {
                    ToggleMode::Single { command, args } => {
                        assert_eq!(command, "nmcli");
                        assert_eq!(args, &vec!["radio".to_string(), "wifi".to_string()]);
                    }
                    _ => panic!("Expected single mode toggle"),
                }
                assert_eq!(probe_command.as_ref().unwrap(), "nmcli");
                assert_eq!(on_icon.as_ref().unwrap(), "wifi");
                assert_eq!(off_icon.as_ref().unwrap(), "wifi_off");
            }
            _ => panic!("Expected toggle button"),
        }
        
        // Check toggle button with separate mode
        match &config.menu.buttons[4] {
            Button::Toggle { name, mode, probe_command, .. } => {
                assert_eq!(name, "VPN Toggle");
                match mode {
                    ToggleMode::Separate { on_command, on_args, off_command, off_args } => {
                        assert_eq!(on_command, "nmcli");
                        assert_eq!(on_args, &vec!["connection".to_string(), "up".to_string(), "vpn".to_string()]);
                        assert_eq!(off_command, "nmcli");
                        assert_eq!(off_args, &vec!["connection".to_string(), "down".to_string(), "vpn".to_string()]);
                    }
                    _ => panic!("Expected separate mode toggle"),
                }
                assert_eq!(probe_command.as_ref().unwrap(), "nmcli");
            }
            _ => panic!("Expected toggle button"),
        }
    }
}