use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, error, info, warn};

/// Result of a probe command execution
#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl ProbeResult {
    /// Creates a new successful probe result
    pub fn success(exit_code: i32, stdout: String, stderr: String) -> Self {
        Self {
            success: true,
            exit_code: Some(exit_code),
            stdout,
            stderr,
        }
    }

    /// Creates a new failed probe result
    pub fn failure(exit_code: Option<i32>, stdout: String, stderr: String) -> Self {
        Self {
            success: false,
            exit_code,
            stdout,
            stderr,
        }
    }

    /// Creates a probe result indicating execution error
    pub fn execution_error(error_message: String) -> Self {
        Self {
            success: false,
            exit_code: None,
            stdout: String::new(),
            stderr: error_message,
        }
    }

    /// Returns true if the command executed successfully (exit code 0)
    pub fn is_success(&self) -> bool {
        self.success && self.exit_code == Some(0)
    }

    /// Returns true if the command failed but was executed (non-zero exit code)
    pub fn is_command_failure(&self) -> bool {
        !self.success && self.exit_code.is_some()
    }

    /// Returns true if the command could not be executed
    pub fn is_execution_error(&self) -> bool {
        !self.success && self.exit_code.is_none()
    }
}

/// Executes a probe command to determine the current state of a toggle
pub async fn execute_probe_command(
    command: &str,
    args: &[String],
    button_name: &str,
) -> ProbeResult {
    info!("Executing probe command for '{}': {} {:?}", button_name, command, args);

    let mut cmd = Command::new(command);
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null()); // Ensure no interactive input

    match cmd.output().await {
        Ok(output) => {
            let exit_code = output.status.code();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            let success = output.status.success();
            
            debug!(
                "Probe command for '{}' completed: exit_code={:?}, success={}, stdout_len={}, stderr_len={}",
                button_name, exit_code, success, stdout.len(), stderr.len()
            );

            // Log stdout/stderr at trace level to avoid noise
            if !stdout.is_empty() {
                debug!("Probe STDOUT for '{}': {}", button_name, stdout.trim());
            }
            if !stderr.is_empty() {
                debug!("Probe STDERR for '{}': {}", button_name, stderr.trim());
            }

            if success {
                ProbeResult::success(exit_code.unwrap_or(0), stdout, stderr)
            } else {
                ProbeResult::failure(exit_code, stdout, stderr)
            }
        }
        Err(e) => {
            error!("Failed to execute probe command for '{}': {} {:?} - {}", 
                   button_name, command, args, e);
            ProbeResult::execution_error(format!("Command execution failed: {}", e))
        }
    }
}

/// Configuration for probe behavior
#[derive(Debug, Clone)]
pub struct ProbeConfig {
    /// Timeout for probe commands in milliseconds
    pub timeout_ms: u64,
    /// Whether to consider empty stdout as success or failure
    pub empty_stdout_is_success: bool,
    /// Custom success indicators in stdout (if any of these are found, consider success)
    pub success_indicators: Vec<String>,
    /// Custom failure indicators in stdout (if any of these are found, consider failure)  
    pub failure_indicators: Vec<String>,
}

impl Default for ProbeConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 5000, // 5 seconds default timeout
            empty_stdout_is_success: true,
            success_indicators: Vec::new(),
            failure_indicators: Vec::new(),
        }
    }
}

/// Advanced probe execution with custom configuration
pub async fn execute_probe_command_with_config(
    command: &str,
    args: &[String],
    button_name: &str,
    config: &ProbeConfig,
) -> ProbeResult {
    info!(
        "Executing probe command with config for '{}': {} {:?} (timeout: {}ms)",
        button_name, command, args, config.timeout_ms
    );

    let mut cmd = Command::new(command);
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    // Use tokio timeout for command execution
    let timeout_duration = std::time::Duration::from_millis(config.timeout_ms);
    
    match tokio::time::timeout(timeout_duration, cmd.output()).await {
        Ok(Ok(output)) => {
            let exit_code = output.status.code();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            let exit_success = output.status.success();
            
            // Apply custom success/failure logic
            let custom_success = evaluate_custom_indicators(&stdout, config);
            let final_success = match custom_success {
                Some(success) => success,
                None => exit_success,
            };

            debug!(
                "Probe command for '{}' completed: exit_code={:?}, exit_success={}, custom_success={:?}, final_success={}",
                button_name, exit_code, exit_success, custom_success, final_success
            );

            if final_success {
                ProbeResult::success(exit_code.unwrap_or(0), stdout, stderr)
            } else {
                ProbeResult::failure(exit_code, stdout, stderr)
            }
        }
        Ok(Err(e)) => {
            error!("Failed to execute probe command for '{}': {} {:?} - {}", 
                   button_name, command, args, e);
            ProbeResult::execution_error(format!("Command execution failed: {}", e))
        }
        Err(_) => {
            warn!("Probe command for '{}' timed out after {}ms: {} {:?}", 
                  button_name, config.timeout_ms, command, args);
            ProbeResult::execution_error(format!("Command timed out after {}ms", config.timeout_ms))
        }
    }
}

/// Evaluates custom success/failure indicators in command output
fn evaluate_custom_indicators(stdout: &str, config: &ProbeConfig) -> Option<bool> {
    // Check failure indicators first (they take precedence)
    for indicator in &config.failure_indicators {
        if stdout.contains(indicator) {
            return Some(false);
        }
    }

    // Check success indicators
    for indicator in &config.success_indicators {
        if stdout.contains(indicator) {
            return Some(true);
        }
    }

    // Handle empty stdout case
    if stdout.trim().is_empty() {
        return Some(config.empty_stdout_is_success);
    }

    // No custom indicators matched, let caller use exit code
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_result_creation() {
        let success = ProbeResult::success(0, "output".to_string(), "".to_string());
        assert!(success.is_success());
        assert!(!success.is_command_failure());
        assert!(!success.is_execution_error());

        let failure = ProbeResult::failure(Some(1), "".to_string(), "error".to_string());
        assert!(!failure.is_success());
        assert!(failure.is_command_failure());
        assert!(!failure.is_execution_error());

        let exec_error = ProbeResult::execution_error("command not found".to_string());
        assert!(!exec_error.is_success());
        assert!(!exec_error.is_command_failure());
        assert!(exec_error.is_execution_error());
    }

    #[test]
    fn test_evaluate_custom_indicators() {
        let mut config = ProbeConfig::default();
        config.success_indicators = vec!["enabled".to_string(), "active".to_string()];
        config.failure_indicators = vec!["disabled".to_string(), "inactive".to_string()];

        // Test success indicators
        assert_eq!(evaluate_custom_indicators("Service is enabled", &config), Some(true));
        assert_eq!(evaluate_custom_indicators("Status: active", &config), Some(true));

        // Test failure indicators (should take precedence)
        assert_eq!(evaluate_custom_indicators("Service is disabled", &config), Some(false));
        assert_eq!(evaluate_custom_indicators("Status: inactive", &config), Some(false));

        // Test mixed (failure takes precedence)
        assert_eq!(evaluate_custom_indicators("Service enabled but disabled", &config), Some(false));

        // Test no indicators
        assert_eq!(evaluate_custom_indicators("unknown status", &config), None);

        // Test empty stdout
        config.empty_stdout_is_success = true;
        assert_eq!(evaluate_custom_indicators("", &config), Some(true));
        assert_eq!(evaluate_custom_indicators("   ", &config), Some(true));

        config.empty_stdout_is_success = false;
        assert_eq!(evaluate_custom_indicators("", &config), Some(false));
    }

    #[tokio::test]
    async fn test_execute_probe_command_success() {
        // Test with a command that should succeed on most systems
        let result = execute_probe_command("echo", &["test".to_string()], "test-button").await;
        
        assert!(result.is_success());
        assert_eq!(result.exit_code, Some(0));
        assert!(result.stdout.contains("test"));
    }

    #[tokio::test]
    async fn test_execute_probe_command_failure() {
        // Test with a command that should fail
        let result = execute_probe_command("false", &[], "test-button").await;
        
        assert!(!result.is_success());
        assert!(result.is_command_failure());
        assert_eq!(result.exit_code, Some(1));
    }

    #[tokio::test]
    async fn test_execute_probe_command_not_found() {
        // Test with a command that doesn't exist
        let result = execute_probe_command("nonexistent_command_xyz123", &[], "test-button").await;
        
        assert!(!result.is_success());
        assert!(result.is_execution_error());
        assert_eq!(result.exit_code, None);
    }

    #[tokio::test]
    async fn test_execute_probe_command_with_timeout() {
        let config = ProbeConfig {
            timeout_ms: 100, // Very short timeout
            ..Default::default()
        };

        // Test with a command that should timeout
        let result = execute_probe_command_with_config(
            "sleep", 
            &["1".to_string()], 
            "test-button",
            &config
        ).await;
        
        assert!(!result.is_success());
        assert!(result.is_execution_error());
        assert!(result.stderr.contains("timed out"));
    }
}