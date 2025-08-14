use anyhow::Result;
use std::{any::{Any, TypeId}, collections::BTreeMap, sync::Arc};
use streamdeck_oxide::{
    button::RenderConfig,
    elgato_streamdeck,
    generic_array::typenum::{U3, U5},
    plugins::{PluginContext, PluginNavigation},
    run_with_external_triggers,
    theme::Theme,
    ExternalTrigger,
};
use tracing::{error, info};
use tracing_subscriber;

mod button;
mod config;
mod icons;

use crate::button::{CommanderContext, CommanderPlugin};
use crate::config::{Config, load_config};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("Starting StreamDeck Commander");
    
    // Load configuration
    let config_path = std::env::var("STREAMDECK_CONFIG")
        .unwrap_or_else(|_| "config.yaml".to_string());
    
    let config: Config = load_config(&config_path)?;
    let config = Arc::new(config);
    
    info!("Configuration loaded from {}", config_path);
    info!("Main menu: {}", config.menu.name);
    info!("Number of buttons: {}", config.menu.buttons.len());
    
    // Connect to Stream Deck
    let hid = elgato_streamdeck::new_hidapi()?;
    let devices = elgato_streamdeck::list_devices(&hid);
    
    if devices.is_empty() {
        error!("No Stream Deck devices found!");
        return Err(anyhow::anyhow!("No Stream Deck devices found"));
    }
    
    info!("Found {} Stream Deck device(s)", devices.len());
    
    // Use the first available device (preferably Mk2, but fall back to others)
    let (kind, serial) = devices
        .into_iter()
        .find(|(kind, _)| matches!(kind, elgato_streamdeck::info::Kind::Mk2))
        .or_else(|| {
            // Fall back to any device if Mk2 not found
            elgato_streamdeck::list_devices(&hid).into_iter().next()
        })
        .ok_or_else(|| anyhow::anyhow!("No Stream Deck found"))?;
    
    info!("Using Stream Deck: {:?} (Serial: {})", kind, serial);
    
    let deck = Arc::new(elgato_streamdeck::AsyncStreamDeck::connect(
        &hid, kind, &serial,
    )?);
    
    info!("Connected to Stream Deck successfully!");
    
    // Create configuration
    let render_config = RenderConfig::default();
    let theme = Theme::light();
    
    // Create plugin context
    let commander_context = CommanderContext {
        config: config.clone(),
    };
    
    let context = PluginContext::new(BTreeMap::from([
        (TypeId::of::<CommanderContext>(), Box::new(Arc::new(commander_context)) as Box<dyn Any + Send + Sync>)
    ]));
    
    // Create external trigger channel
    let (sender, receiver) = tokio::sync::mpsc::channel::<ExternalTrigger<PluginNavigation<U5, U3>, U5, U3, PluginContext>>(1);
    
    // Send initial navigation to main menu
    sender.send(ExternalTrigger::new(
        PluginNavigation::<U5, U3>::new(CommanderPlugin::new(config.menu.clone())),
        true
    )).await?;
    
    info!("Starting Stream Deck application...");
    info!("Press Ctrl+C to exit");
    
    // Run the application
    run_with_external_triggers::<PluginNavigation<U5, U3>, U5, U3, PluginContext>(
        theme,
        render_config,
        deck,
        context,
        receiver,
    )
    .await
    .map_err(|e| anyhow::anyhow!("StreamDeck application error: {}", e))?;
    
    Ok(())
}
