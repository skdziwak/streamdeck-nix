use crate::config::{Button, Config, Menu};
use crate::icons;
use std::{process::Stdio, sync::Arc};
use tokio::io::{AsyncBufReadExt, BufReader};
use streamdeck_oxide::{
    generic_array::typenum::{U3, U5},
    plugins::{Plugin, PluginContext, PluginNavigation},
    view::{
        customizable::{ClickButton, CustomizableView},
        View,
    },
};
use tokio::process::Command;
use tracing::{debug, error, info, warn};

#[derive(Clone)]
pub struct CommanderPlugin {
    menu: Menu,
    parent: Option<Box<CommanderPlugin>>,
}

pub struct CommanderContext {
    pub config: Arc<Config>,
}

impl CommanderPlugin {
    pub fn new(menu: Menu) -> Self {
        Self { menu, parent: None }
    }
    
    pub fn new_with_parent(menu: Menu, parent: CommanderPlugin) -> Self {
        Self { menu, parent: Some(Box::new(parent)) }
    }


    async fn execute_command(command: &str, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        info!("Executing command: {} {:?}", command, args);
        
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
                
                // Spawn tasks to read stdout and stderr concurrently
                let stdout_task = {
                    let cmd_str = format!("{} {:?}", command, args);
                    tokio::spawn(async move {
                        let mut lines = stdout_reader.lines();
                        while let Ok(Some(line)) = lines.next_line().await {
                            debug!("STDOUT [{}]: {}", cmd_str, line);
                        }
                    })
                };
                
                let stderr_task = {
                    let cmd_str = format!("{} {:?}", command, args);
                    tokio::spawn(async move {
                        let mut lines = stderr_reader.lines();
                        while let Ok(Some(line)) = lines.next_line().await {
                            debug!("STDERR [{}]: {}", cmd_str, line);
                        }
                    })
                };
                
                // Wait for the process to complete
                match child.wait().await {
                    Ok(status) => {
                        // Wait for output reading tasks to complete
                        let _ = tokio::join!(stdout_task, stderr_task);
                        
                        if status.success() {
                            info!("Command executed successfully: {} {:?} (exit code: {})", 
                                  command, args, status.code().unwrap_or(0));
                        } else {
                            warn!("Command exited with non-zero status: {} {:?} (exit code: {})", 
                                  command, args, status.code().unwrap_or(-1));
                        }
                        Ok(())
                    }
                    Err(e) => {
                        error!("Failed to wait for command: {} {:?} - {}", command, args, e);
                        Err(Box::new(e))
                    }
                }
            }
            Err(e) => {
                error!("Failed to execute command: {} {:?} - {}", command, args, e);
                Err(Box::new(e))
            }
        }
    }

    fn create_view_from_menu(
        &self,
    ) -> Result<Box<dyn View<U5, U3, PluginContext, PluginNavigation<U5, U3>>>, Box<dyn std::error::Error>> {
        let mut view = CustomizableView::new();
        
        let mut row = 0;
        let mut col = 0;
        let mut button_index = 0;
        
        for button in &self.menu.buttons {
            // Reserve position 14 (index 14 = row 2, col 4) for the automatic back button
            if button_index == 14 {
                // Skip to next position, leaving space for back button
                button_index += 1;
                col = 0;
                row = 3;
            }
            
            if row >= 3 { // Stream Deck has 3 rows
                break;
            }
            
            match button {
                Button::Command { name, command, args, icon } => {
                    let command_clone = command.clone();
                    let args_clone = args.clone();
                    let name_clone = name.clone();
                    
                    view.set_button(
                        col,
                        row,
                        ClickButton::new(
                            &name_clone,
                            icons::resolve_icon(icon.as_ref()),
                            move |_context: PluginContext| {
                                let cmd = command_clone.clone();
                                let args = args_clone.clone();
                                // Spawn command execution in a separate task to avoid blocking UI
                                tokio::spawn(async move {
                                    if let Err(e) = Self::execute_command(&cmd, &args).await {
                                        error!("Command execution failed: {}", e);
                                    }
                                });
                                async move { Ok(()) }
                            },
                        ),
                    )?;
                }
                Button::Menu { name, buttons, icon } => {
                    let submenu = Menu {
                        name: name.clone(),
                        buttons: buttons.clone(),
                    };
                    
                    view.set_navigation(
                        col,
                        row,
                        PluginNavigation::<U5, U3>::new(CommanderPlugin::new_with_parent(submenu, self.clone())),
                        name,
                        icons::resolve_icon(icon.as_ref()),
                    )?;
                }
                Button::Back { name, icon } => {
                    // Skip user-defined back buttons - we'll add our own automatically
                    debug!("Skipping user-defined back button at position {},{}", col, row);
                    button_index += 1;
                    col += 1;
                    if col >= 5 {
                        col = 0;
                        row += 1;
                    }
                    continue;
                }
            }
            
            button_index += 1;
            col += 1;
            if col >= 5 { // Stream Deck has 5 columns
                col = 0;
                row += 1;
            }
        }
        
        // Always add a back button at position 15 (row 2, col 4) if we have a parent menu
        if self.parent.is_some() {
            if let Some(parent) = &self.parent {
                view.set_navigation(
                    4, // column 5 (0-indexed)
                    2, // row 3 (0-indexed)
                    PluginNavigation::<U5, U3>::new(parent.as_ref().clone()),
                    "Back",
                    icons::resolve_icon(Some(&"arrow_back".to_string())),
                )?;
            }
        }
        
        Ok(Box::new(view))
    }
}

#[async_trait::async_trait]
impl Plugin<U5, U3> for CommanderPlugin {
    fn name(&self) -> &'static str {
        "StreamDeck Commander"
    }

    async fn get_view(&self, _context: PluginContext) -> Result<Box<dyn View<U5, U3, PluginContext, PluginNavigation<U5, U3>>>, Box<dyn std::error::Error>> {
        info!("Creating view for menu: {}", self.menu.name);
        self.create_view_from_menu()
    }
}