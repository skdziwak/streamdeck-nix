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
"#;

        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.menu.name, "Main Menu");
        assert_eq!(config.menu.buttons.len(), 3);
        
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
    }
}