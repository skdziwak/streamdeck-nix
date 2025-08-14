# StreamDeck Nix

A Nix flake that provides StreamDeck Commander - a customizable command launcher for Elgato Stream Deck devices using YAML configuration. This flake includes NixOS modules, packages, and development environments.

## Features

- **Nix Flake**: Complete flake with packages, NixOS modules, and dev shells
- **NixOS Module**: Declarative system-wide StreamDeck configuration
- **YAML Configuration**: Define menus and buttons using simple YAML files
- **Nested Menus**: Organize commands into hierarchical menus
- **Shell Command Execution**: Execute any shell command with customizable arguments
- **Automatic udev Rules**: Proper USB device permissions handled automatically

## Quick Start

### Using the Flake

Add this flake as an input to your NixOS configuration:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    streamdeck-nix = {
      url = "github:skdziwak/streamdeck-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, streamdeck-nix, ... }: {
    nixosConfigurations.your-host = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        streamdeck-nix.nixosModules.default
        ./configuration.nix
      ];
    };
  };
}
```

### Direct Installation

Install the package directly:

```bash
# Install temporarily
nix run github:skdziwak/streamdeck-nix

# Install to profile
nix profile install github:skdziwak/streamdeck-nix

# Try it out
nix shell github:skdziwak/streamdeck-nix
```

## NixOS Module Configuration

Enable and configure StreamDeck Commander in your NixOS configuration:

```nix
{
  services.streamdeck-commander = {
    enable = true;
    
    # Optional: specify user (defaults to "streamdeck")
    user = "myuser";
    
    # Configure your menu structure
    menu = {
      name = "My StreamDeck";
      buttons = [
        {
          type = "command";
          name = "Terminal";
          command = "alacritty";
          icon = "terminal";
        }
        {
          type = "menu";
          name = "System";
          icon = "computer";
          buttons = [
            {
              type = "command";
              name = "Top";
              command = "alacritty";
              args = [ "-e" "top" ];
              icon = "play_arrow";
            }
            {
              type = "back";
              name = "← Back";
              icon = "arrow_back";
            }
          ];
        }
      ];
    };
  };
}
```

### Using Custom Configuration File

You can also provide a custom YAML configuration file:

```nix
{
  services.streamdeck-commander = {
    enable = true;
    configFile = ./my-streamdeck-config.yaml;
  };
}
```

## Configuration Examples

### Basic NixOS Configuration

```nix
# /etc/nixos/configuration.nix or in your flake
{
  services.streamdeck-commander = {
    enable = true;
    user = "myuser";  # Run as your user instead of dedicated streamdeck user
    
    menu = {
      name = "Dev Environment";
      buttons = [
        # Quick terminal access
        {
          type = "command";
          name = "Terminal";
          command = "alacritty";
          icon = "terminal";
        }
        
        # Git operations menu
        {
          type = "menu";
          name = "Git";
          icon = "code";
          buttons = [
            {
              type = "command";
              name = "Status";
              command = "alacritty";
              args = [ "-e" "bash" "-c" "cd ~/projects && git status && read" ];
              icon = "network_check";
            }
            {
              type = "command";
              name = "Pull";
              command = "alacritty";
              args = [ "-e" "bash" "-c" "cd ~/projects && git pull && read" ];
              icon = "refresh";
            }
            {
              type = "back";
              name = "← Back";
              icon = "arrow_back";
            }
          ];
        }
        
        # System monitoring
        {
          type = "command";
          name = "System Monitor";
          command = "gnome-system-monitor";
          icon = "memory";
        }
      ];
    };
  };
}
```

### YAML Configuration Format

If using `configFile` option, the YAML structure is:

```yaml
menu:
  name: "Main Menu"
  buttons:
    # Command button - executes a shell command
    - type: command
      name: "Button Label"
      command: "command-to-run"
      args: ["arg1", "arg2"]  # Optional arguments
      icon: "storage"  # Optional Material Design icon name
    
    # Menu button - opens a submenu
    - type: menu
      name: "Submenu"
      buttons:
        - type: command
          name: "Nested Command"
          command: "another-command"
        - type: back  # Returns to parent menu
          name: "← Back"
```

### Button Types

1. **Command Button**: Executes a shell command
   - `type`: "command"
   - `name`: Display name on the button
   - `command`: Command to execute
   - `args`: Optional array of arguments
   - `icon`: Optional Material Design icon name

2. **Menu Button**: Opens a submenu
   - `type`: "menu"
   - `name`: Display name on the button
   - `buttons`: Array of buttons in the submenu
   - `icon`: Optional Material Design icon name

3. **Back Button**: Returns to the parent menu
   - `type`: "back"
   - `name`: Display name (defaults to "Back")
   - `icon`: Optional Material Design icon name

### Icon Configuration

Icons use Material Design icons from the `md-icons` crate. You can specify icons in several ways:

- **Simple name**: `"terminal"` (uses filled style by default)
- **Style prefix**: `"outlined:code"`, `"sharp:arrow_back"`, `"two_tone:memory"`

#### Available Styles:
- `filled` (default) - Solid filled icons
- `outlined` - Outlined icons with transparent fill
- `sharp` - Angular, sharp-cornered icons
- `two_tone` - Two-tone colored icons

#### Available Icons:

**Navigation & Control:**
- `terminal` - Terminal/command prompt
- `home` - Home/house icon
- `arrow_back` - Left arrow (back)
- `arrow_forward` - Right arrow (forward)
- `arrow_upward` - Up arrow
- `arrow_downward` - Down arrow  
- `refresh` - Refresh/reload
- `play_arrow` - Play button
- `stop` - Stop button
- `pause` - Pause button
- `fast_forward` - Fast forward
- `fast_rewind` - Fast rewind
- `skip_next` - Skip to next
- `skip_previous` - Skip to previous

**Files & Folders:**
- `folder` - Closed folder
- `folder_open` - Open folder
- `folder_shared` - Shared folder
- `file_copy` - Copy file
- `description` - Document/file
- `article` - Article/text document
- `note` - Single note
- `notes` - Multiple notes

**System & Hardware:**
- `computer` - Desktop computer
- `laptop` - Laptop computer
- `phone` - Mobile phone
- `tablet` - Tablet device
- `memory` - RAM/memory chip
- `storage` - Storage/hard drive
- `monitor` - Computer monitor
- `keyboard` - Keyboard
- `mouse` - Computer mouse

**Development & Code:**
- `code` - Code brackets
- `build` - Build/compile
- `bug_report` - Bug/error reporting
- `integration_instructions` - Integration guide
- `api` - API interface
- `web` - Web/internet
- `developer_mode` - Developer settings

**Network & Communication:**
- `network_check` - Network status
- `wifi` - WiFi connection
- `bluetooth` - Bluetooth
- `http` - HTTP protocol
- `https` - HTTPS secure protocol
- `vpn_key` - VPN/security key
- `router` - Network router
- `dns` - DNS settings

**Configuration & Settings:**
- `settings` - Settings/preferences
- `tune` - Fine tuning
- `palette` - Color palette
- `build_circle` - Build configuration
- `settings_applications` - Application settings

**Time & Scheduling:**
- `schedule` - Calendar/schedule
- `access_time` - Clock/time
- `timer` - Timer/countdown
- `alarm` - Alarm clock
- `event` - Calendar event
- `today` - Today/current date
- `date_range` - Date range picker

**Media & Entertainment:**
- `music_note` - Music note
- `library_music` - Music library
- `video_library` - Video library
- `movie` - Movie/film
- `photo` - Single photo
- `photo_library` - Photo gallery
- `camera` - Camera
- `videocam` - Video camera

**Utilities & Tools:**
- `search` - Search/magnifying glass
- `edit` - Edit/pencil
- `delete` - Delete/trash
- `add` - Add/plus
- `remove` - Remove/minus
- `save` - Save/disk
- `download` - Download
- `upload` - Upload
- `share` - Share
- `copy` - Copy content
- `cut` - Cut content
- `paste` - Paste content

**Security & Privacy:**
- `lock` - Locked/secure
- `lock_open` - Unlocked
- `key` - Key/password
- `security` - Security shield
- `shield` - Protection shield
- `fingerprint` - Fingerprint authentication

**Status & Indicators:**
- `check` - Checkmark
- `check_circle` - Checkmark in circle
- `warning` - Warning triangle
- `error` - Error/X mark
- `info` - Information
- `help` - Help/question mark
- `notifications` - Bell/notifications

**Workspace & Organization:**
- `dashboard` - Dashboard/grid
- `inbox` - Inbox/mail
- `archive` - Archive box
- `bookmark` - Bookmark
- `favorite` - Heart/favorite
- `star` - Star rating
- `label` - Label/tag
- `tag` - Tag

#### Icon Style Support:
- **All icons** are available in `filled` style
- **Basic icons** (terminal, home, arrow_back, folder, settings, etc.) are available in all styles
- If a style is not available for an icon, it falls back to the terminal icon with a warning

#### Usage Examples:
```yaml
icon: "terminal"              # Default filled style
icon: "filled:code"           # Explicitly filled
icon: "outlined:folder"       # Outlined style
icon: "sharp:arrow_back"      # Sharp style  
icon: "two_tone:settings"     # Two-tone style
```

If an icon name is not found, it falls back to the terminal icon with a warning.

## Example Configuration

See `config.yaml` for a comprehensive example that includes:
- System commands (ls, uname, date)
- Git operations submenu
- Development tools submenu
- System monitoring commands
- Custom scripts placeholder

## Troubleshooting

### No Stream Deck Found
- Ensure your Stream Deck is connected via USB
- Check that you have appropriate permissions to access USB devices
- On Linux, you may need to add udev rules for the Stream Deck

### Commands Not Working
- Verify that the commands exist in your PATH
- Check the terminal output for error messages
- Ensure commands don't require interactive input

## Development

### Icon System

This project uses a build-time code generation system for Material Design icons:

1. **icons.json** - Contains all icon mappings with:
   - Icon name to Material Design constant mappings for each style

2. **build.rs** - Reads icons.json and generates Rust code at compile time
   - Creates optimized match statements for each style
   - Generates the main `resolve_icon` function

3. **src/icons.rs** - Simply includes the generated code

#### Adding New Icons

To add new icons:
1. Edit `icons.json` to add new icon mappings
2. Find the appropriate Material Design constant name from the `md-icons` crate
3. Add it to the appropriate style section
4. Run `cargo build` to regenerate the lookup code

Icon names in JSON should be lowercase with underscores (e.g., "arrow_back"). The build script automatically converts to uppercase for constant lookup.

### Running Tests
```bash
cargo test
```

### Building for Release
```bash
cargo build --release
```

## License

This project is provided as-is for educational and personal use.

## Contributing

Feel free to submit issues and pull requests for bug fixes and new features.
