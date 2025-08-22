use crate::config::ToggleMode;
use crate::probe::execute_probe_command;
use crate::toggle_state::{ToggleState, ToggleStateManager};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{debug, error, info, warn};

/// Result of executing a toggle command
#[derive(Debug, Clone)]
pub struct ToggleCommandResult {
    pub success: bool,
    pub new_state: ToggleState,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub error_message: Option<String>,
}

impl ToggleCommandResult {
    /// Creates a successful toggle command result
    pub fn success(new_state: ToggleState, exit_code: i32, stdout: String, stderr: String) -> Self {
        Self {
            success: true,
            new_state,
            exit_code: Some(exit_code),
            stdout,
            stderr,
            error_message: None,
        }
    }

    /// Creates a failed toggle command result
    pub fn failure(
        current_state: ToggleState,
        exit_code: Option<i32>,
        stdout: String,
        stderr: String,
        error_message: String,
    ) -> Self {
        Self {
            success: false,
            new_state: current_state,
            exit_code,
            stdout,
            stderr,
            error_message: Some(error_message),
        }
    }
}

/// Executes a toggle command and updates state accordingly
pub async fn execute_toggle_command(
    button_name: &str,
    mode: &ToggleMode,
    probe_command: Option<&str>,
    probe_args: &[String],
    state_manager: &ToggleStateManager,
) -> ToggleCommandResult {
    info!("Executing toggle command for '{}'", button_name);

    // Get current state - either from probe or from state manager
    let current_state = if let Some(probe_cmd) = probe_command {
        // Execute probe to get current state
        let probe_result = execute_probe_command(probe_cmd, probe_args, button_name).await;
        let probed_state = if probe_result.is_success() {
            ToggleState::On
        } else if probe_result.is_command_failure() {
            ToggleState::Off
        } else {
            ToggleState::Unknown
        };
        
        // Update state manager with probed state
        state_manager.set_state(button_name, probed_state);
        probed_state
    } else {
        // Use state from state manager
        state_manager.get_state(button_name)
    };

    debug!("Current state for '{}': {:?}", button_name, current_state);

    // Determine what command to execute based on mode and current state
    let (command, args, expected_new_state) = match (mode, current_state) {
        (ToggleMode::Single { command, args }, state) => {
            // For single command mode, always execute the same command
            let new_state = match state {
                ToggleState::On => ToggleState::Off,
                ToggleState::Off => ToggleState::On,
                ToggleState::Unknown => {
                    // If state is unknown, we assume we're turning it on
                    debug!("State unknown for '{}', assuming we're turning it on", button_name);
                    ToggleState::On
                }
            };
            (command.clone(), args.clone(), new_state)
        }
        (ToggleMode::Separate { on_command, on_args, off_command, off_args }, state) => {
            // For separate command mode, choose command based on desired state
            match state {
                ToggleState::On => {
                    // Currently on, turn off
                    (off_command.clone(), off_args.clone(), ToggleState::Off)
                }
                ToggleState::Off => {
                    // Currently off, turn on
                    (on_command.clone(), on_args.clone(), ToggleState::On)
                }
                ToggleState::Unknown => {
                    // If state is unknown, default to turning on
                    debug!("State unknown for '{}', defaulting to turn on", button_name);
                    (on_command.clone(), on_args.clone(), ToggleState::On)
                }
            }
        }
    };

    info!(
        "Executing {} command for '{}': {} {:?} (expecting state: {:?})",
        match mode {
            ToggleMode::Single { .. } => "single",
            ToggleMode::Separate { .. } => "separate",
        },
        button_name,
        command,
        args,
        expected_new_state
    );

    // Execute the command
    match execute_command_with_output(&command, &args, button_name).await {
        Ok((exit_code, stdout, stderr)) => {
            if exit_code == 0 {
                // Command succeeded, update state
                state_manager.set_state(button_name, expected_new_state);
                
                // Optionally verify the new state with a probe
                let final_state = if let Some(probe_cmd) = probe_command {
                    debug!("Verifying new state for '{}' with probe", button_name);
                    let verify_probe = execute_probe_command(probe_cmd, probe_args, button_name).await;
                    let verified_state = if verify_probe.is_success() {
                        ToggleState::On
                    } else if verify_probe.is_command_failure() {
                        ToggleState::Off
                    } else {
                        // Probe failed, keep expected state but warn
                        warn!("Failed to verify new state for '{}', keeping expected state", button_name);
                        expected_new_state
                    };
                    
                    if verified_state != expected_new_state {
                        warn!(
                            "State verification mismatch for '{}': expected {:?}, probed {:?}",
                            button_name, expected_new_state, verified_state
                        );
                    }
                    
                    state_manager.set_state(button_name, verified_state);
                    verified_state
                } else {
                    expected_new_state
                };

                info!("Toggle command for '{}' succeeded, new state: {:?}", button_name, final_state);
                ToggleCommandResult::success(final_state, exit_code, stdout, stderr)
            } else {
                // Command failed
                let error_msg = format!("Toggle command failed with exit code {}", exit_code);
                warn!("Toggle command for '{}' failed: {}", button_name, error_msg);
                ToggleCommandResult::failure(current_state, Some(exit_code), stdout, stderr, error_msg)
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to execute toggle command: {}", e);
            error!("Toggle command execution error for '{}': {}", button_name, error_msg);
            ToggleCommandResult::failure(current_state, None, String::new(), String::new(), error_msg)
        }
    }
}

/// Executes a command and captures all output
async fn execute_command_with_output(
    command: &str,
    args: &[String],
    button_name: &str,
) -> Result<(i32, String, String), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Executing command for '{}': {} {:?}", button_name, command, args);

    let mut cmd = Command::new(command);
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    match cmd.spawn() {
        Ok(mut child) => {
            // Get stdout and stderr handles
            let stdout = child.stdout.take().expect("Failed to capture stdout");
            let stderr = child.stderr.take().expect("Failed to capture stderr");

            // Create async readers
            let stdout_reader = BufReader::new(stdout);
            let stderr_reader = BufReader::new(stderr);

            // Read all output
            let stdout_task = {
                tokio::spawn(async move {
                    let mut lines = stdout_reader.lines();
                    let mut output = String::new();
                    while let Ok(Some(line)) = lines.next_line().await {
                        if !output.is_empty() {
                            output.push('\n');
                        }
                        output.push_str(&line);
                    }
                    output
                })
            };

            let stderr_task = {
                tokio::spawn(async move {
                    let mut lines = stderr_reader.lines();
                    let mut output = String::new();
                    while let Ok(Some(line)) = lines.next_line().await {
                        if !output.is_empty() {
                            output.push('\n');
                        }
                        output.push_str(&line);
                    }
                    output
                })
            };

            // Wait for the process to complete
            match child.wait().await {
                Ok(status) => {
                    // Wait for output reading tasks to complete
                    let (stdout_result, stderr_result) = tokio::join!(stdout_task, stderr_task);
                    let stdout = stdout_result.unwrap_or_default();
                    let stderr = stderr_result.unwrap_or_default();

                    let exit_code = status.code().unwrap_or(-1);
                    
                    if !stdout.is_empty() {
                        debug!("Command STDOUT for '{}': {}", button_name, stdout);
                    }
                    if !stderr.is_empty() {
                        debug!("Command STDERR for '{}': {}", button_name, stderr);
                    }

                    Ok((exit_code, stdout, stderr))
                }
                Err(e) => {
                    error!("Failed to wait for command for '{}': {}", button_name, e);
                    Err(Box::new(e))
                }
            }
        }
        Err(e) => {
            error!("Failed to spawn command for '{}': {} {:?} - {}", button_name, command, args, e);
            Err(Box::new(e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_command_result_creation() {
        let success = ToggleCommandResult::success(
            ToggleState::On,
            0,
            "output".to_string(),
            "".to_string(),
        );
        assert!(success.success);
        assert_eq!(success.new_state, ToggleState::On);
        assert_eq!(success.exit_code, Some(0));
        assert!(success.error_message.is_none());

        let failure = ToggleCommandResult::failure(
            ToggleState::Off,
            Some(1),
            "".to_string(),
            "error".to_string(),
            "Command failed".to_string(),
        );
        assert!(!failure.success);
        assert_eq!(failure.new_state, ToggleState::Off);
        assert_eq!(failure.exit_code, Some(1));
        assert!(failure.error_message.is_some());
    }

    #[tokio::test]
    async fn test_execute_command_with_output_success() {
        let result = execute_command_with_output("echo", &["test".to_string()], "test-button").await;
        
        assert!(result.is_ok());
        let (exit_code, stdout, stderr) = result.unwrap();
        assert_eq!(exit_code, 0);
        assert!(stdout.contains("test"));
        assert!(stderr.is_empty());
    }

    #[tokio::test]
    async fn test_execute_command_with_output_failure() {
        let result = execute_command_with_output("false", &[], "test-button").await;
        
        assert!(result.is_ok());
        let (exit_code, _stdout, _stderr) = result.unwrap();
        assert_ne!(exit_code, 0);
    }

    #[tokio::test]
    async fn test_execute_toggle_command_single_mode() {
        let state_manager = ToggleStateManager::new();
        let mode = ToggleMode::Single {
            command: "echo".to_string(),
            args: vec!["toggle".to_string()],
        };

        // Set initial state to Off
        state_manager.set_state("test", ToggleState::Off);

        let result = execute_toggle_command("test", &mode, None, &[], &state_manager).await;

        assert!(result.success);
        assert_eq!(result.new_state, ToggleState::On);
        assert_eq!(state_manager.get_state("test"), ToggleState::On);
    }

    #[tokio::test]
    async fn test_execute_toggle_command_separate_mode() {
        let state_manager = ToggleStateManager::new();
        let mode = ToggleMode::Separate {
            on_command: "echo".to_string(),
            on_args: vec!["turn_on".to_string()],
            off_command: "echo".to_string(),
            off_args: vec!["turn_off".to_string()],
        };

        // Set initial state to Off
        state_manager.set_state("test", ToggleState::Off);

        let result = execute_toggle_command("test", &mode, None, &[], &state_manager).await;

        assert!(result.success);
        assert_eq!(result.new_state, ToggleState::On);
        assert!(result.stdout.contains("turn_on"));
    }

    #[tokio::test]
    async fn test_execute_toggle_command_with_probe() {
        let state_manager = ToggleStateManager::new();
        let mode = ToggleMode::Single {
            command: "echo".to_string(),
            args: vec!["toggle".to_string()],
        };

        // Use a probe that should succeed
        let result = execute_toggle_command(
            "test",
            &mode,
            Some("true"), // Always succeeds
            &[],
            &state_manager,
        ).await;

        assert!(result.success);
        // Since probe always succeeds ("true"), the final state after verification will be On
        // This is expected behavior - the probe determines the final state
        assert_eq!(result.new_state, ToggleState::On);
    }
}