use crate::config::{Button, Config, Menu};
use crate::icons;
use std::{process::Stdio, sync::Arc};
use streamdeck_oxide::{
    generic_array::typenum::{U3, U5},
    plugins::{Plugin, PluginContext, PluginNavigation},
    view::{
        customizable::{ClickButton, CustomizableView},
        View,
    },
};
use tokio::process::Command;
use tracing::{error, info, warn};

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
                match child.wait().await {
                    Ok(status) => {
                        if status.success() {
                            info!("Command executed successfully");
                        } else {
                            info!("Command exited with status: {}", status);
                        }
                        Ok(())
                    }
                    Err(e) => {
                        error!("Failed to wait for command: {}", e);
                        Err(Box::new(e))
                    }
                }
            }
            Err(e) => {
                error!("Failed to execute command: {}", e);
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
        
        for button in &self.menu.buttons {
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
                                async move {
                                    Self::execute_command(&cmd, &args).await?;
                                    Ok(())
                                }
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
                    if let Some(parent) = &self.parent {
                        // Create navigation back to parent menu
                        view.set_navigation(
                            col,
                            row,
                            PluginNavigation::<U5, U3>::new(parent.as_ref().clone()),
                            name,
                            icons::resolve_icon(icon.as_ref()),
                        )?;
                    } else {
                        // No parent, just show a disabled button or skip
                        view.set_button(
                            col,
                            row,
                            ClickButton::new(
                                name,
                                icons::resolve_icon(icon.as_ref()),
                                |_context: PluginContext| async move {
                                    info!("Back button pressed (no parent to go back to)");
                                    Ok(())
                                },
                            ),
                        )?;
                    }
                }
            }
            
            col += 1;
            if col >= 5 { // Stream Deck has 5 columns
                col = 0;
                row += 1;
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