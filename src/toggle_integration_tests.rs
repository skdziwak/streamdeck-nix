//! Integration tests for toggle button functionality
//! 
//! This module contains comprehensive tests that validate the entire toggle button
//! implementation including state management, command execution, probing, and UI integration.

use crate::config::{Button, Menu, ToggleMode};
use crate::probe::{execute_probe_command, ProbeConfig, execute_probe_command_with_config};
use crate::toggle_command::execute_toggle_command;
use crate::toggle_icons::{resolve_toggle_icon, get_toggle_display_name, is_toggle_button};
use crate::toggle_state::{ToggleState, ToggleStateManager};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_single_mode_toggle() -> Button {
        Button::Toggle {
            name: "WiFi".to_string(),
            mode: ToggleMode::Single {
                command: "nmcli".to_string(),
                args: vec!["radio".to_string(), "wifi".to_string()],
            },
            probe_command: Some("nmcli".to_string()),
            probe_args: vec!["radio".to_string(), "wifi".to_string()],
            on_icon: Some("wifi".to_string()),
            off_icon: Some("wifi_off".to_string()),
            icon: Some("settings".to_string()),
        }
    }

    fn create_separate_mode_toggle() -> Button {
        Button::Toggle {
            name: "VPN".to_string(),
            mode: ToggleMode::Separate {
                on_command: "systemctl".to_string(),
                on_args: vec!["start".to_string(), "openvpn".to_string()],
                off_command: "systemctl".to_string(),
                off_args: vec!["stop".to_string(), "openvpn".to_string()],
            },
            probe_command: Some("systemctl".to_string()),
            probe_args: vec!["is-active".to_string(), "openvpn".to_string()],
            on_icon: Some("vpn_key".to_string()),
            off_icon: Some("vpn_key_off".to_string()),
            icon: None,
        }
    }

    fn create_test_menu() -> Menu {
        Menu {
            name: "Test Menu".to_string(),
            buttons: vec![
                Button::Command {
                    name: "Test Command".to_string(),
                    command: "echo".to_string(),
                    args: vec!["hello".to_string()],
                    icon: Some("terminal".to_string()),
                },
                create_single_mode_toggle(),
                create_separate_mode_toggle(),
                Button::Menu {
                    name: "Submenu".to_string(),
                    buttons: vec![create_single_mode_toggle()],
                    icon: Some("folder".to_string()),
                },
            ],
        }
    }

    #[test]
    fn test_toggle_button_identification() {
        let single_toggle = create_single_mode_toggle();
        let separate_toggle = create_separate_mode_toggle();
        let command_button = Button::Command {
            name: "Test".to_string(),
            command: "echo".to_string(),
            args: vec![],
            icon: None,
        };

        assert!(is_toggle_button(&single_toggle));
        assert!(is_toggle_button(&separate_toggle));
        assert!(!is_toggle_button(&command_button));
    }

    #[test]
    fn test_toggle_state_management_integration() {
        let state_manager = ToggleStateManager::new();
        let button = create_single_mode_toggle();

        // Initial state should be unknown
        assert_eq!(state_manager.get_state("WiFi"), ToggleState::Unknown);

        // Test state transitions
        state_manager.set_state("WiFi", ToggleState::Off);
        assert_eq!(state_manager.get_state("WiFi"), ToggleState::Off);

        let new_state = state_manager.toggle_state("WiFi");
        assert_eq!(new_state, ToggleState::On);
        assert_eq!(state_manager.get_state("WiFi"), ToggleState::On);

        // Test probe-based updates
        state_manager.update_from_probe("WiFi", false);
        assert_eq!(state_manager.get_state("WiFi"), ToggleState::Off);

        state_manager.update_from_probe("WiFi", true);
        assert_eq!(state_manager.get_state("WiFi"), ToggleState::On);
    }

    #[test]
    fn test_toggle_display_names() {
        let state_manager = ToggleStateManager::new();
        let button = create_single_mode_toggle();

        // Test different state displays
        state_manager.set_state("WiFi", ToggleState::On);
        assert_eq!(get_toggle_display_name(&button, &state_manager), "WiFi ●");

        state_manager.set_state("WiFi", ToggleState::Off);
        assert_eq!(get_toggle_display_name(&button, &state_manager), "WiFi ○");

        state_manager.set_state("WiFi", ToggleState::Unknown);
        assert_eq!(get_toggle_display_name(&button, &state_manager), "WiFi ?");
    }

    #[test]
    fn test_icon_resolution_logic() {
        let state_manager = ToggleStateManager::new();
        let button = create_single_mode_toggle();

        // Test icon resolution for different states
        state_manager.set_state("WiFi", ToggleState::On);
        let _on_icon = resolve_toggle_icon(&button, &state_manager);

        state_manager.set_state("WiFi", ToggleState::Off);
        let _off_icon = resolve_toggle_icon(&button, &state_manager);

        state_manager.set_state("WiFi", ToggleState::Unknown);
        let _unknown_icon = resolve_toggle_icon(&button, &state_manager);

        // Test with button that has no specific icons
        let minimal_button = Button::Toggle {
            name: "Minimal".to_string(),
            mode: ToggleMode::Single {
                command: "test".to_string(),
                args: vec![],
            },
            probe_command: None,
            probe_args: vec![],
            on_icon: None,
            off_icon: None,
            icon: None,
        };

        state_manager.set_state("Minimal", ToggleState::On);
        let _minimal_icon = resolve_toggle_icon(&minimal_button, &state_manager);
    }

    #[tokio::test]
    async fn test_probe_command_execution() {
        // Test successful probe
        let result = execute_probe_command("true", &[], "test-probe").await;
        assert!(result.is_success());
        assert_eq!(result.exit_code, Some(0));

        // Test failed probe
        let result = execute_probe_command("false", &[], "test-probe").await;
        assert!(!result.is_success());
        assert!(result.is_command_failure());
        assert_eq!(result.exit_code, Some(1));

        // Test nonexistent command
        let result = execute_probe_command("nonexistent_command_xyz", &[], "test-probe").await;
        assert!(!result.is_success());
        assert!(result.is_execution_error());
    }

    #[tokio::test]
    async fn test_probe_with_config() {
        let config = ProbeConfig {
            timeout_ms: 1000,
            empty_stdout_is_success: true,
            success_indicators: vec!["active".to_string()],
            failure_indicators: vec!["inactive".to_string()],
        };

        // Test with custom success indicator
        let result = execute_probe_command_with_config(
            "echo", 
            &["Service is active".to_string()], 
            "test-probe",
            &config
        ).await;
        assert!(result.is_success());

        // Test with custom failure indicator
        let result = execute_probe_command_with_config(
            "echo", 
            &["Service is inactive".to_string()], 
            "test-probe",
            &config
        ).await;
        assert!(!result.is_success());
    }

    #[tokio::test]
    async fn test_single_mode_toggle_execution() {
        let state_manager = ToggleStateManager::new();
        let mode = ToggleMode::Single {
            command: "echo".to_string(),
            args: vec!["toggling".to_string()],
        };

        // Test toggle from unknown state
        let result = execute_toggle_command("test", &mode, None, &[], &state_manager).await;
        assert!(result.success);
        assert_eq!(result.new_state, ToggleState::On);
        assert!(result.stdout.contains("toggling"));

        // Test toggle from known state
        state_manager.set_state("test", ToggleState::On);
        let result = execute_toggle_command("test", &mode, None, &[], &state_manager).await;
        assert!(result.success);
        assert_eq!(result.new_state, ToggleState::Off);
    }

    #[tokio::test]
    async fn test_separate_mode_toggle_execution() {
        let state_manager = ToggleStateManager::new();
        let mode = ToggleMode::Separate {
            on_command: "echo".to_string(),
            on_args: vec!["turning_on".to_string()],
            off_command: "echo".to_string(),
            off_args: vec!["turning_off".to_string()],
        };

        // Test turning on from off state
        state_manager.set_state("test", ToggleState::Off);
        let result = execute_toggle_command("test", &mode, None, &[], &state_manager).await;
        assert!(result.success);
        assert_eq!(result.new_state, ToggleState::On);
        assert!(result.stdout.contains("turning_on"));

        // Test turning off from on state
        state_manager.set_state("test", ToggleState::On);
        let result = execute_toggle_command("test", &mode, None, &[], &state_manager).await;
        assert!(result.success);
        assert_eq!(result.new_state, ToggleState::Off);
        assert!(result.stdout.contains("turning_off"));
    }

    #[tokio::test]
    async fn test_toggle_with_probe_verification() {
        let state_manager = ToggleStateManager::new();
        let mode = ToggleMode::Single {
            command: "echo".to_string(),
            args: vec!["toggle".to_string()],
        };

        // Test with probe that succeeds (indicating "on" state)
        let result = execute_toggle_command(
            "test",
            &mode,
            Some("true"),
            &[],
            &state_manager,
        ).await;
        assert!(result.success);
        // Since probe "true" always succeeds, final state will be "on" after verification
        assert_eq!(result.new_state, ToggleState::On);

        // Test with probe that fails (indicating "off" state)
        let result = execute_toggle_command(
            "test2",
            &mode,
            Some("false"),
            &[],
            &state_manager,
        ).await;
        assert!(result.success);
        // Since probe "false" always fails, final state will be "off" after verification
        assert_eq!(result.new_state, ToggleState::Off);
    }

    #[tokio::test]
    async fn test_toggle_command_failure_handling() {
        let state_manager = ToggleStateManager::new();
        let mode = ToggleMode::Single {
            command: "false".to_string(), // Command that always fails
            args: vec![],
        };

        state_manager.set_state("test", ToggleState::Off);
        let result = execute_toggle_command("test", &mode, None, &[], &state_manager).await;
        
        assert!(!result.success);
        assert_eq!(result.new_state, ToggleState::Off); // Should remain in original state
        assert!(result.error_message.is_some());
    }

    #[test]
    fn test_configuration_parsing_compatibility() {
        let yaml = r#"
menu:
  name: "Integration Test Menu"
  buttons:
    - type: command
      name: "Test Command"
      command: "echo"
      args: ["test"]
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
      name: "Service Toggle"
      mode: separate
      on_command: "systemctl"
      on_args: ["start", "service"]
      off_command: "systemctl"
      off_args: ["stop", "service"]
      probe_command: "systemctl"
      probe_args: ["is-active", "service"]
    - type: menu
      name: "Submenu"
      buttons:
        - type: back
"#;

        let result = serde_yaml::from_str::<crate::config::Config>(yaml);
        assert!(result.is_ok(), "Configuration parsing should succeed: {:?}", result.err());
        
        let config = result.unwrap();
        assert_eq!(config.menu.buttons.len(), 4);
        
        // Verify toggle buttons parsed correctly
        match &config.menu.buttons[1] {
            Button::Toggle { name, mode, .. } => {
                assert_eq!(name, "WiFi Toggle");
                match mode {
                    ToggleMode::Single { command, .. } => assert_eq!(command, "nmcli"),
                    _ => panic!("Expected single mode"),
                }
            }
            _ => panic!("Expected toggle button"),
        }
        
        match &config.menu.buttons[2] {
            Button::Toggle { name, mode, .. } => {
                assert_eq!(name, "Service Toggle");
                match mode {
                    ToggleMode::Separate { on_command, off_command, .. } => {
                        assert_eq!(on_command, "systemctl");
                        assert_eq!(off_command, "systemctl");
                    }
                    _ => panic!("Expected separate mode"),
                }
            }
            _ => panic!("Expected toggle button"),
        }
    }

    #[test]
    fn test_menu_with_toggles_creation() {
        let menu = create_test_menu();
        assert_eq!(menu.buttons.len(), 4);
        
        let toggle_count = menu.buttons.iter()
            .filter(|b| is_toggle_button(b))
            .count();
        assert_eq!(toggle_count, 2);
    }

    #[test]
    fn test_state_manager_thread_safety() {
        use std::sync::Arc;
        use std::thread;
        
        let state_manager = Arc::new(ToggleStateManager::new());
        let mut handles = vec![];
        
        // Spawn multiple threads that modify state
        for i in 0..10 {
            let manager = Arc::clone(&state_manager);
            let handle = thread::spawn(move || {
                let button_name = format!("button_{}", i);
                manager.set_state(&button_name, ToggleState::On);
                manager.toggle_state(&button_name);
                manager.get_state(&button_name)
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            let final_state = handle.join().unwrap();
            assert_eq!(final_state, ToggleState::Off);
        }
        
        // Verify all buttons were created
        assert_eq!(state_manager.button_count(), 10);
    }
}