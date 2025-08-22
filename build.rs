use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Config {
    menu: Menu,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Menu {
    name: String,
    buttons: Vec<Button>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Button {
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

fn default_back_name() -> String {
    "Back".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
enum ToggleMode {
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

#[derive(Debug)]
struct IconSpec {
    style: String,
    name: String,
}

// Recursively extract icons from menu configuration
fn extract_icons_from_menu(menu: &Menu) -> Vec<String> {
    let mut icons = Vec::new();
    extract_icons_from_buttons(&menu.buttons, &mut icons);
    icons
}

fn extract_icons_from_buttons(buttons: &[Button], icons: &mut Vec<String>) {
    for button in buttons {
        match button {
            Button::Command { icon, .. }
            | Button::Menu { icon, .. }
            | Button::Back { icon, .. } => {
                if let Some(icon_name) = icon {
                    icons.push(icon_name.clone());
                }
            }
            Button::Toggle { icon, on_icon, off_icon, .. } => {
                if let Some(icon_name) = icon {
                    icons.push(icon_name.clone());
                }
                if let Some(icon_name) = on_icon {
                    icons.push(icon_name.clone());
                }
                if let Some(icon_name) = off_icon {
                    icons.push(icon_name.clone());
                }
            }
        }

        // Recurse into submenus
        if let Button::Menu { buttons, .. } = button {
            extract_icons_from_buttons(buttons, icons);
        }
    }
}

// Parse icon specification (e.g., "terminal" or "sharp:home")
fn parse_icon_spec(spec: &str) -> IconSpec {
    if let Some(colon_pos) = spec.find(':') {
        IconSpec {
            style: spec[..colon_pos].to_string(),
            name: spec[colon_pos + 1..].to_string(),
        }
    } else {
        IconSpec {
            style: "filled".to_string(),
            name: spec.to_string(),
        }
    }
}

// Convert snake_case to ICON_SNAKE_CASE with special cases
fn icon_name_to_constant(name: &str) -> String {
    match name {
        "copy" => "ICON_CONTENT_COPY".to_string(),
        "cut" => "ICON_CONTENT_CUT".to_string(),
        "paste" => "ICON_CONTENT_PASTE".to_string(),
        "tag" => "ICON_LOCAL_OFFER".to_string(),
        _ => format!("ICON_{}", name.to_uppercase()),
    }
}

fn main() {
    println!("cargo:rerun-if-changed=config.yaml");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("icons_generated.rs");

    // Read and parse config.yaml
    let config_yaml = fs::read_to_string("config.yaml")
        .expect("Failed to read config.yaml - ensure it exists in the project root");

    let config: Config = serde_yaml::from_str(&config_yaml).expect("Failed to parse config.yaml");

    // Extract all icons from the menu
    let icon_strings = extract_icons_from_menu(&config.menu);
    let icon_specs: Vec<IconSpec> = icon_strings.iter().map(|s| parse_icon_spec(s)).collect();

    // Group icons by style and collect unique names
    let mut icons_by_style: HashMap<String, HashSet<String>> = HashMap::new();
    for spec in &icon_specs {
        icons_by_style
            .entry(spec.style.clone())
            .or_insert_with(HashSet::new)
            .insert(spec.name.clone());
    }

    // Add default icons to ensure they're always available
    let default_icons = vec![
        "terminal", "home", "arrow_back", "settings",
        "toggle_on", "toggle_off", "help", "wifi", "wifi_off"
    ];
    for icon in default_icons {
        icons_by_style
            .entry("filled".to_string())
            .or_insert_with(HashSet::new)
            .insert(icon.to_string());
    }

    let mut generated = String::new();

    generated.push_str("// This file is automatically generated by build.rs\n");
    generated.push_str("// DO NOT EDIT MANUALLY\n\n");
    generated.push_str("use streamdeck_oxide::md_icons;\n\n");

    // Generate resolve functions for each style
    for (style, icon_names) in &icons_by_style {
        let fn_name = format!("resolve_{}_icon", style);
        generated.push_str(&format!(
            "pub fn {}(const_name: &str) -> Option<&'static str> {{\n",
            fn_name
        ));
        generated.push_str("    match const_name {\n");

        // Process all icons for this style
        let mut sorted_icons: Vec<_> = icon_names.iter().collect();
        sorted_icons.sort();

        for icon_name in sorted_icons {
            let const_name = icon_name.to_uppercase();
            let icon_const = icon_name_to_constant(icon_name);

            // Check if the icon constant exists by trying to use it
            // This will cause a compile error if the icon doesn't exist
            generated.push_str(&format!(
                "        \"{}\" => Some(md_icons::{}::{}),\n",
                const_name, style, icon_const
            ));
        }

        // Add default case
        generated.push_str(&format!(
            "        _ => {{\n            tracing::warn!(\"Unknown {} icon: {{}}, using default terminal icon\", const_name);\n",
            style
        ));

        // Use terminal as default fallback
        generated.push_str(&format!(
            "            Some(md_icons::{}::ICON_TERMINAL)\n",
            style
        ));

        generated.push_str("        }\n");
        generated.push_str("    }\n");
        generated.push_str("}\n\n");
    }

    // Generate the main resolve_icon function
    generated
        .push_str("pub fn resolve_icon(icon_name: Option<&String>) -> Option<&'static str> {\n");
    generated.push_str("    let icon_name = icon_name?;\n");
    generated.push_str("    \n");
    generated.push_str(
        "    // Parse icon specification: \"style:name\" or just \"name\" (defaults to filled)\n",
    );
    generated.push_str("    let (style, name) = if let Some(colon_pos) = icon_name.find(':') {\n");
    generated.push_str("        let style = &icon_name[..colon_pos];\n");
    generated.push_str("        let name = &icon_name[colon_pos + 1..];\n");
    generated.push_str("        (style, name)\n");
    generated.push_str("    } else {\n");
    generated.push_str("        (\"filled\", icon_name.as_str())\n");
    generated.push_str("    };\n");
    generated.push_str("    \n");
    generated.push_str("    // Convert name to uppercase for constant lookup\n");
    generated.push_str("    let const_name = name.to_uppercase();\n");
    generated.push_str("    \n");
    generated.push_str("    // Match against available icons by style\n");
    generated.push_str("    match style {\n");

    for style in icons_by_style.keys() {
        generated.push_str(&format!(
            "        \"{}\" => resolve_{}_icon(&const_name),\n",
            style, style
        ));
    }

    generated.push_str("        _ => {\n");
    generated.push_str(
        "            tracing::warn!(\"Unknown icon style: {}, using filled:terminal\", style);\n",
    );
    generated.push_str("            Some(md_icons::filled::ICON_TERMINAL)\n");

    generated.push_str("        }\n");
    generated.push_str("    }\n");
    generated.push_str("}\n");

    fs::write(dest_path, generated).expect("Failed to write generated file");
}
