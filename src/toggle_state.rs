use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, warn};

/// Represents the state of a toggle button
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleState {
    On,
    Off,
    Unknown, // Used when probe fails or state cannot be determined
}

impl ToggleState {
    /// Returns the opposite state for toggling
    pub fn toggle(self) -> ToggleState {
        match self {
            ToggleState::On => ToggleState::Off,
            ToggleState::Off => ToggleState::On,
            ToggleState::Unknown => ToggleState::Unknown,
        }
    }

    /// Returns true if the state is definitively known
    pub fn is_known(self) -> bool {
        matches!(self, ToggleState::On | ToggleState::Off)
    }
}

/// Manages the state of all toggle buttons in the application
#[derive(Debug)]
pub struct ToggleStateManager {
    states: Arc<RwLock<HashMap<String, ToggleState>>>,
}

impl Clone for ToggleStateManager {
    fn clone(&self) -> Self {
        Self {
            states: Arc::clone(&self.states),
        }
    }
}

impl Default for ToggleStateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ToggleStateManager {
    /// Creates a new toggle state manager
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Gets the current state of a toggle button
    pub fn get_state(&self, button_name: &str) -> ToggleState {
        match self.states.read() {
            Ok(states) => {
                let state = states.get(button_name).copied().unwrap_or(ToggleState::Unknown);
                debug!("Retrieved state for '{}': {:?}", button_name, state);
                state
            }
            Err(e) => {
                warn!("Failed to read toggle state for '{}': {}", button_name, e);
                ToggleState::Unknown
            }
        }
    }

    /// Sets the state of a toggle button
    pub fn set_state(&self, button_name: &str, state: ToggleState) {
        match self.states.write() {
            Ok(mut states) => {
                let previous = states.insert(button_name.to_string(), state);
                debug!(
                    "Set state for '{}': {:?} -> {:?}",
                    button_name, previous.unwrap_or(ToggleState::Unknown), state
                );
            }
            Err(e) => {
                warn!("Failed to set toggle state for '{}': {}", button_name, e);
            }
        }
    }

    /// Toggles the state of a button and returns the new state
    pub fn toggle_state(&self, button_name: &str) -> ToggleState {
        let current_state = self.get_state(button_name);
        let new_state = current_state.toggle();
        self.set_state(button_name, new_state);
        new_state
    }

    /// Updates the state based on probe results
    pub fn update_from_probe(&self, button_name: &str, probe_success: bool) {
        let new_state = if probe_success {
            ToggleState::On
        } else {
            ToggleState::Off
        };
        self.set_state(button_name, new_state);
    }

    /// Clears all states (useful for resetting)
    pub fn clear_all(&self) {
        match self.states.write() {
            Ok(mut states) => {
                let count = states.len();
                states.clear();
                debug!("Cleared {} toggle states", count);
            }
            Err(e) => {
                warn!("Failed to clear toggle states: {}", e);
            }
        }
    }

    /// Gets all current states (for debugging/monitoring)
    pub fn get_all_states(&self) -> HashMap<String, ToggleState> {
        match self.states.read() {
            Ok(states) => states.clone(),
            Err(e) => {
                warn!("Failed to read all toggle states: {}", e);
                HashMap::new()
            }
        }
    }

    /// Returns the number of buttons being tracked
    pub fn button_count(&self) -> usize {
        match self.states.read() {
            Ok(states) => states.len(),
            Err(_) => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_state_toggle() {
        assert_eq!(ToggleState::On.toggle(), ToggleState::Off);
        assert_eq!(ToggleState::Off.toggle(), ToggleState::On);
        assert_eq!(ToggleState::Unknown.toggle(), ToggleState::Unknown);
    }

    #[test]
    fn test_toggle_state_is_known() {
        assert!(ToggleState::On.is_known());
        assert!(ToggleState::Off.is_known());
        assert!(!ToggleState::Unknown.is_known());
    }

    #[test]
    fn test_toggle_state_manager_basic() {
        let manager = ToggleStateManager::new();
        
        // Initial state should be unknown
        assert_eq!(manager.get_state("test"), ToggleState::Unknown);
        
        // Set and get state
        manager.set_state("test", ToggleState::On);
        assert_eq!(manager.get_state("test"), ToggleState::On);
        
        // Toggle state
        let new_state = manager.toggle_state("test");
        assert_eq!(new_state, ToggleState::Off);
        assert_eq!(manager.get_state("test"), ToggleState::Off);
    }

    #[test]
    fn test_toggle_state_manager_multiple_buttons() {
        let manager = ToggleStateManager::new();
        
        manager.set_state("wifi", ToggleState::On);
        manager.set_state("bluetooth", ToggleState::Off);
        manager.set_state("vpn", ToggleState::Unknown);
        
        assert_eq!(manager.get_state("wifi"), ToggleState::On);
        assert_eq!(manager.get_state("bluetooth"), ToggleState::Off);
        assert_eq!(manager.get_state("vpn"), ToggleState::Unknown);
        assert_eq!(manager.button_count(), 3);
        
        let all_states = manager.get_all_states();
        assert_eq!(all_states.len(), 3);
        assert_eq!(all_states.get("wifi"), Some(&ToggleState::On));
        assert_eq!(all_states.get("bluetooth"), Some(&ToggleState::Off));
        assert_eq!(all_states.get("vpn"), Some(&ToggleState::Unknown));
    }

    #[test]
    fn test_toggle_state_manager_probe_update() {
        let manager = ToggleStateManager::new();
        
        // Simulate successful probe
        manager.update_from_probe("service", true);
        assert_eq!(manager.get_state("service"), ToggleState::On);
        
        // Simulate failed probe
        manager.update_from_probe("service", false);
        assert_eq!(manager.get_state("service"), ToggleState::Off);
    }

    #[test]
    fn test_toggle_state_manager_clear() {
        let manager = ToggleStateManager::new();
        
        manager.set_state("test1", ToggleState::On);
        manager.set_state("test2", ToggleState::Off);
        assert_eq!(manager.button_count(), 2);
        
        manager.clear_all();
        assert_eq!(manager.button_count(), 0);
        assert_eq!(manager.get_state("test1"), ToggleState::Unknown);
        assert_eq!(manager.get_state("test2"), ToggleState::Unknown);
    }

    #[test]
    fn test_toggle_state_manager_clone() {
        let manager1 = ToggleStateManager::new();
        manager1.set_state("test", ToggleState::On);
        
        let manager2 = manager1.clone();
        assert_eq!(manager2.get_state("test"), ToggleState::On);
        
        // Changes through one should be visible through the other
        manager2.set_state("test", ToggleState::Off);
        assert_eq!(manager1.get_state("test"), ToggleState::Off);
    }
}