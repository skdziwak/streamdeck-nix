use crate::config::Button;
use crate::icons::resolve_icon;
use crate::toggle_state::{ToggleState, ToggleStateManager};
use tracing::debug;

/// Resolves the appropriate icon for a toggle button based on its current state
pub fn resolve_toggle_icon(
    button: &Button,
    state_manager: &ToggleStateManager,
) -> Option<&'static str> {
    match button {
        Button::Toggle { name, on_icon, off_icon, icon, .. } => {
            let current_state = state_manager.get_state(name);
            
            debug!("Resolving icon for toggle '{}' in state {:?}", name, current_state);
            
            match current_state {
                ToggleState::On => {
                    // Try on_icon first, then fallback to general icon, then default
                    if let Some(resolved) = on_icon.as_ref().and_then(|i| resolve_icon(Some(i))) {
                        debug!("Using on_icon for '{}': resolved", name);
                        Some(resolved)
                    } else if let Some(resolved) = icon.as_ref().and_then(|i| resolve_icon(Some(i))) {
                        debug!("Using fallback icon for '{}' (on state): resolved", name);
                        Some(resolved)
                    } else {
                        debug!("No icon specified for '{}' (on state), using default", name);
                        resolve_icon(Some(&"toggle_on".to_string()))
                    }
                }
                ToggleState::Off => {
                    // Try off_icon first, then fallback to general icon, then default
                    if let Some(resolved) = off_icon.as_ref().and_then(|i| resolve_icon(Some(i))) {
                        debug!("Using off_icon for '{}': resolved", name);
                        Some(resolved)
                    } else if let Some(resolved) = icon.as_ref().and_then(|i| resolve_icon(Some(i))) {
                        debug!("Using fallback icon for '{}' (off state): resolved", name);
                        Some(resolved)
                    } else {
                        debug!("No icon specified for '{}' (off state), using default", name);
                        resolve_icon(Some(&"toggle_off".to_string()))
                    }
                }
                ToggleState::Unknown => {
                    // For unknown state, prefer fallback icon, then a question mark or similar
                    if let Some(resolved) = icon.as_ref().and_then(|i| resolve_icon(Some(i))) {
                        debug!("Using fallback icon for '{}' (unknown state): resolved", name);
                        Some(resolved)
                    } else {
                        debug!("No icon specified for '{}' (unknown state), using default", name);
                        resolve_icon(Some(&"help".to_string()))
                    }
                }
            }
        }
        // For non-toggle buttons, use the standard icon resolution
        Button::Command { icon, .. }
        | Button::Menu { icon, .. }
        | Button::Back { icon, .. } => {
            resolve_icon(icon.as_ref())
        }
    }
}

/// Gets the display name for a toggle button, potentially with state indicators
pub fn get_toggle_display_name(button: &Button, state_manager: &ToggleStateManager) -> String {
    match button {
        Button::Toggle { name, .. } => {
            let current_state = state_manager.get_state(name);
            match current_state {
                ToggleState::On => format!("{} ●", name),      // Green dot indicator
                ToggleState::Off => format!("{} ○", name),     // Empty circle indicator
                ToggleState::Unknown => format!("{} ?", name), // Question mark for unknown
            }
        }
        Button::Command { name, .. }
        | Button::Menu { name, .. }
        | Button::Back { name, .. } => name.clone(),
    }
}

/// Gets a simple display name without state indicators
pub fn get_simple_display_name(button: &Button) -> &str {
    match button {
        Button::Command { name, .. }
        | Button::Menu { name, .. }
        | Button::Back { name, .. }
        | Button::Toggle { name, .. } => name,
    }
}

/// Checks if a button is a toggle button
pub fn is_toggle_button(button: &Button) -> bool {
    matches!(button, Button::Toggle { .. })
}

/// Gets the state description for a toggle button
pub fn get_toggle_state_description(button: &Button, state_manager: &ToggleStateManager) -> Option<String> {
    match button {
        Button::Toggle { name, .. } => {
            let state = state_manager.get_state(name);
            Some(match state {
                ToggleState::On => "Currently enabled".to_string(),
                ToggleState::Off => "Currently disabled".to_string(),
                ToggleState::Unknown => "State unknown".to_string(),
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ToggleMode;

    fn create_test_toggle_button() -> Button {
        Button::Toggle {
            name: "Test Toggle".to_string(),
            mode: ToggleMode::Single {
                command: "test".to_string(),
                args: vec![],
            },
            probe_command: None,
            probe_args: vec![],
            on_icon: Some("wifi".to_string()),
            off_icon: Some("wifi_off".to_string()),
            icon: Some("settings".to_string()),
        }
    }

    fn create_test_command_button() -> Button {
        Button::Command {
            name: "Test Command".to_string(),
            command: "echo".to_string(),
            args: vec![],
            icon: Some("terminal".to_string()),
        }
    }

    #[test]
    fn test_is_toggle_button() {
        let toggle = create_test_toggle_button();
        let command = create_test_command_button();

        assert!(is_toggle_button(&toggle));
        assert!(!is_toggle_button(&command));
    }

    #[test]
    fn test_get_simple_display_name() {
        let toggle = create_test_toggle_button();
        let command = create_test_command_button();

        assert_eq!(get_simple_display_name(&toggle), "Test Toggle");
        assert_eq!(get_simple_display_name(&command), "Test Command");
    }

    #[test]
    fn test_get_toggle_display_name() {
        let button = create_test_toggle_button();
        let state_manager = ToggleStateManager::new();

        // Test different states
        state_manager.set_state("Test Toggle", ToggleState::On);
        assert_eq!(get_toggle_display_name(&button, &state_manager), "Test Toggle ●");

        state_manager.set_state("Test Toggle", ToggleState::Off);
        assert_eq!(get_toggle_display_name(&button, &state_manager), "Test Toggle ○");

        state_manager.set_state("Test Toggle", ToggleState::Unknown);
        assert_eq!(get_toggle_display_name(&button, &state_manager), "Test Toggle ?");

        // Test non-toggle button
        let command = create_test_command_button();
        assert_eq!(get_toggle_display_name(&command, &state_manager), "Test Command");
    }

    #[test]
    fn test_get_toggle_state_description() {
        let button = create_test_toggle_button();
        let command = create_test_command_button();
        let state_manager = ToggleStateManager::new();

        // Test toggle button descriptions
        state_manager.set_state("Test Toggle", ToggleState::On);
        assert_eq!(
            get_toggle_state_description(&button, &state_manager),
            Some("Currently enabled".to_string())
        );

        state_manager.set_state("Test Toggle", ToggleState::Off);
        assert_eq!(
            get_toggle_state_description(&button, &state_manager),
            Some("Currently disabled".to_string())
        );

        state_manager.set_state("Test Toggle", ToggleState::Unknown);
        assert_eq!(
            get_toggle_state_description(&button, &state_manager),
            Some("State unknown".to_string())
        );

        // Test non-toggle button
        assert_eq!(get_toggle_state_description(&command, &state_manager), None);
    }

    #[test]
    fn test_resolve_toggle_icon_fallbacks() {
        let state_manager = ToggleStateManager::new();

        // Button with all icons specified
        let full_button = create_test_toggle_button();
        state_manager.set_state("Test Toggle", ToggleState::On);
        
        // We can't actually test icon resolution without the generated icons,
        // but we can test the logic flow
        let _result = resolve_toggle_icon(&full_button, &state_manager);
        
        // Button with no specific icons
        let minimal_button = Button::Toggle {
            name: "Minimal Toggle".to_string(),
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
        
        state_manager.set_state("Minimal Toggle", ToggleState::Unknown);
        let _result = resolve_toggle_icon(&minimal_button, &state_manager);
        
        // Test with command button (should use standard resolution)
        let command = create_test_command_button();
        let _result = resolve_toggle_icon(&command, &state_manager);
    }
}