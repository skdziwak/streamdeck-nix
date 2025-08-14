use streamdeck_oxide::md_icons;
use tracing::warn;

/// Resolves icon name from YAML config to actual md-icons SVG string
pub fn resolve_icon(icon_name: Option<&String>) -> Option<&'static str> {
    let icon_name = icon_name?;
    
    // Parse icon specification: "style:name" or just "name" (defaults to filled)
    let (style, name) = if let Some(colon_pos) = icon_name.find(':') {
        let style = &icon_name[..colon_pos];
        let name = &icon_name[colon_pos + 1..];
        (style, name)
    } else {
        ("filled", icon_name.as_str())
    };
    
    // Convert name to uppercase for constant lookup
    let const_name = name.to_uppercase();
    
    // Match against available icons by style
    match style {
        "filled" => resolve_filled_icon(&const_name),
        "outlined" => resolve_outlined_icon(&const_name),
        "sharp" => resolve_sharp_icon(&const_name),
        "two_tone" => resolve_two_tone_icon(&const_name),
        _ => {
            warn!("Unknown icon style: {}, using filled:terminal", style);
            Some(md_icons::filled::ICON_TERMINAL)
        }
    }
}

/// Resolves filled style Material Design icons
fn resolve_filled_icon(const_name: &str) -> Option<&'static str> {
    match const_name {
        // Navigation & Control
        "TERMINAL" => Some(md_icons::filled::ICON_TERMINAL),
        "HOME" => Some(md_icons::filled::ICON_HOME),
        "ARROW_BACK" => Some(md_icons::filled::ICON_ARROW_BACK),
        "ARROW_FORWARD" => Some(md_icons::filled::ICON_ARROW_FORWARD),
        "ARROW_UPWARD" => Some(md_icons::filled::ICON_ARROW_UPWARD),
        "ARROW_DOWNWARD" => Some(md_icons::filled::ICON_ARROW_DOWNWARD),
        "REFRESH" => Some(md_icons::filled::ICON_REFRESH),
        "PLAY_ARROW" => Some(md_icons::filled::ICON_PLAY_ARROW),
        "STOP" => Some(md_icons::filled::ICON_STOP),
        "PAUSE" => Some(md_icons::filled::ICON_PAUSE),
        "FAST_FORWARD" => Some(md_icons::filled::ICON_FAST_FORWARD),
        "FAST_REWIND" => Some(md_icons::filled::ICON_FAST_REWIND),
        "SKIP_NEXT" => Some(md_icons::filled::ICON_SKIP_NEXT),
        "SKIP_PREVIOUS" => Some(md_icons::filled::ICON_SKIP_PREVIOUS),
        
        // Files & Folders
        "FOLDER" => Some(md_icons::filled::ICON_FOLDER),
        "FOLDER_OPEN" => Some(md_icons::filled::ICON_FOLDER_OPEN),
        "FOLDER_SHARED" => Some(md_icons::filled::ICON_FOLDER_SHARED),
        "FILE_COPY" => Some(md_icons::filled::ICON_FILE_COPY),
        "DESCRIPTION" => Some(md_icons::filled::ICON_DESCRIPTION),
        "ARTICLE" => Some(md_icons::filled::ICON_ARTICLE),
        "NOTE" => Some(md_icons::filled::ICON_NOTE),
        "NOTES" => Some(md_icons::filled::ICON_NOTES),
        
        // System & Hardware
        "COMPUTER" => Some(md_icons::filled::ICON_COMPUTER),
        "LAPTOP" => Some(md_icons::filled::ICON_LAPTOP),
        "PHONE" => Some(md_icons::filled::ICON_PHONE),
        "TABLET" => Some(md_icons::filled::ICON_TABLET),
        "MEMORY" => Some(md_icons::filled::ICON_MEMORY),
        "STORAGE" => Some(md_icons::filled::ICON_STORAGE),
        "HARD_DRIVE" => Some(md_icons::filled::ICON_STORAGE), // Alias
        "CPU" => Some(md_icons::filled::ICON_MEMORY), // Alias
        "MONITOR" => Some(md_icons::filled::ICON_MONITOR),
        "KEYBOARD" => Some(md_icons::filled::ICON_KEYBOARD),
        "MOUSE" => Some(md_icons::filled::ICON_MOUSE),
        
        // Development & Code
        "CODE" => Some(md_icons::filled::ICON_CODE),
        "BUILD" => Some(md_icons::filled::ICON_BUILD),
        "BUG_REPORT" => Some(md_icons::filled::ICON_BUG_REPORT),
        "INTEGRATION_INSTRUCTIONS" => Some(md_icons::filled::ICON_INTEGRATION_INSTRUCTIONS),
        "API" => Some(md_icons::filled::ICON_API),
        "WEB" => Some(md_icons::filled::ICON_WEB),
        "DEVELOPER_MODE" => Some(md_icons::filled::ICON_DEVELOPER_MODE),
        
        // Network & Communication
        "NETWORK_CHECK" => Some(md_icons::filled::ICON_NETWORK_CHECK),
        "WIFI" => Some(md_icons::filled::ICON_WIFI),
        "BLUETOOTH" => Some(md_icons::filled::ICON_BLUETOOTH),
        "HTTP" => Some(md_icons::filled::ICON_HTTP),
        "HTTPS" => Some(md_icons::filled::ICON_HTTPS),
        "VPN_KEY" => Some(md_icons::filled::ICON_VPN_KEY),
        "ROUTER" => Some(md_icons::filled::ICON_ROUTER),
        "DNS" => Some(md_icons::filled::ICON_DNS),
        
        // Configuration & Settings  
        "SETTINGS" => Some(md_icons::filled::ICON_SETTINGS),
        "TUNE" => Some(md_icons::filled::ICON_TUNE),
        "PALETTE" => Some(md_icons::filled::ICON_PALETTE),
        "BUILD_CIRCLE" => Some(md_icons::filled::ICON_BUILD_CIRCLE),
        "SETTINGS_APPLICATIONS" => Some(md_icons::filled::ICON_SETTINGS_APPLICATIONS),
        
        // Time & Scheduling
        "SCHEDULE" => Some(md_icons::filled::ICON_SCHEDULE),
        "ACCESS_TIME" => Some(md_icons::filled::ICON_ACCESS_TIME),
        "TIMER" => Some(md_icons::filled::ICON_TIMER),
        "ALARM" => Some(md_icons::filled::ICON_ALARM),
        "EVENT" => Some(md_icons::filled::ICON_EVENT),
        "TODAY" => Some(md_icons::filled::ICON_TODAY),
        "DATE_RANGE" => Some(md_icons::filled::ICON_DATE_RANGE),
        
        // Media & Entertainment
        "MUSIC_NOTE" => Some(md_icons::filled::ICON_MUSIC_NOTE),
        "LIBRARY_MUSIC" => Some(md_icons::filled::ICON_LIBRARY_MUSIC),
        "VIDEO_LIBRARY" => Some(md_icons::filled::ICON_VIDEO_LIBRARY),
        "MOVIE" => Some(md_icons::filled::ICON_MOVIE),
        "PHOTO" => Some(md_icons::filled::ICON_PHOTO),
        "PHOTO_LIBRARY" => Some(md_icons::filled::ICON_PHOTO_LIBRARY),
        "CAMERA" => Some(md_icons::filled::ICON_CAMERA),
        "VIDEOCAM" => Some(md_icons::filled::ICON_VIDEOCAM),
        
        // Utilities & Tools
        "SEARCH" => Some(md_icons::filled::ICON_SEARCH),
        "EDIT" => Some(md_icons::filled::ICON_EDIT),
        "DELETE" => Some(md_icons::filled::ICON_DELETE),
        "ADD" => Some(md_icons::filled::ICON_ADD),
        "REMOVE" => Some(md_icons::filled::ICON_REMOVE),
        "SAVE" => Some(md_icons::filled::ICON_SAVE),
        "DOWNLOAD" => Some(md_icons::filled::ICON_DOWNLOAD),
        "UPLOAD" => Some(md_icons::filled::ICON_UPLOAD),
        "SHARE" => Some(md_icons::filled::ICON_SHARE),
        "COPY" => Some(md_icons::filled::ICON_CONTENT_COPY),
        "CUT" => Some(md_icons::filled::ICON_CONTENT_CUT),
        "PASTE" => Some(md_icons::filled::ICON_CONTENT_PASTE),
        
        // Security & Privacy
        "LOCK" => Some(md_icons::filled::ICON_LOCK),
        "LOCK_OPEN" => Some(md_icons::filled::ICON_LOCK_OPEN),
        "KEY" => Some(md_icons::filled::ICON_KEY),
        "SECURITY" => Some(md_icons::filled::ICON_SECURITY),
        "SHIELD" => Some(md_icons::filled::ICON_SHIELD),
        "FINGERPRINT" => Some(md_icons::filled::ICON_FINGERPRINT),
        
        // Status & Indicators
        "CHECK" => Some(md_icons::filled::ICON_CHECK),
        "CHECK_CIRCLE" => Some(md_icons::filled::ICON_CHECK_CIRCLE),
        "WARNING" => Some(md_icons::filled::ICON_WARNING),
        "ERROR" => Some(md_icons::filled::ICON_ERROR),
        "INFO" => Some(md_icons::filled::ICON_INFO),
        "HELP" => Some(md_icons::filled::ICON_HELP),
        "NOTIFICATIONS" => Some(md_icons::filled::ICON_NOTIFICATIONS),
        
        // Workspace & Organization
        "DASHBOARD" => Some(md_icons::filled::ICON_DASHBOARD),
        "INBOX" => Some(md_icons::filled::ICON_INBOX),
        "ARCHIVE" => Some(md_icons::filled::ICON_ARCHIVE),
        "BOOKMARK" => Some(md_icons::filled::ICON_BOOKMARK),
        "FAVORITE" => Some(md_icons::filled::ICON_FAVORITE),
        "STAR" => Some(md_icons::filled::ICON_STAR),
        "LABEL" => Some(md_icons::filled::ICON_LABEL),
        "TAG" => Some(md_icons::filled::ICON_LOCAL_OFFER), // Using local_offer as tag
        
        // Third-party Services (using reasonable fallbacks)
        "DOCKER" => Some(md_icons::filled::ICON_COMPUTER), // Docker uses computer icon as fallback
        "GIT" => Some(md_icons::filled::ICON_CODE), // Git uses code icon as fallback
        "GITHUB" => Some(md_icons::filled::ICON_CODE), // GitHub uses code icon as fallback
        "GITLAB" => Some(md_icons::filled::ICON_CODE), // GitLab uses code icon as fallback
        "JENKINS" => Some(md_icons::filled::ICON_BUILD), // Jenkins uses build icon as fallback
        "AWS" => Some(md_icons::filled::ICON_COMPUTER), // AWS uses computer icon as fallback
        "KUBERNETES" => Some(md_icons::filled::ICON_COMPUTER), // K8s uses computer icon as fallback
        
        _ => {
            warn!("Unknown filled icon: {}, using default terminal icon", const_name);
            Some(md_icons::filled::ICON_TERMINAL)
        }
    }
}

/// Resolves outlined style Material Design icons
fn resolve_outlined_icon(const_name: &str) -> Option<&'static str> {
    match const_name {
        // Navigation & Control
        "TERMINAL" => Some(md_icons::outlined::ICON_TERMINAL),
        "HOME" => Some(md_icons::outlined::ICON_HOME),
        "ARROW_BACK" => Some(md_icons::outlined::ICON_ARROW_BACK),
        "ARROW_FORWARD" => Some(md_icons::outlined::ICON_ARROW_FORWARD),
        "ARROW_UPWARD" => Some(md_icons::outlined::ICON_ARROW_UPWARD),
        "ARROW_DOWNWARD" => Some(md_icons::outlined::ICON_ARROW_DOWNWARD),
        "REFRESH" => Some(md_icons::outlined::ICON_REFRESH),
        
        // Files & Folders
        "FOLDER" => Some(md_icons::outlined::ICON_FOLDER),
        "FOLDER_OPEN" => Some(md_icons::outlined::ICON_FOLDER_OPEN),
        "DESCRIPTION" => Some(md_icons::outlined::ICON_DESCRIPTION),
        
        // System & Hardware
        "COMPUTER" => Some(md_icons::outlined::ICON_COMPUTER),
        "LAPTOP" => Some(md_icons::outlined::ICON_LAPTOP),
        "MEMORY" => Some(md_icons::outlined::ICON_MEMORY),
        "STORAGE" => Some(md_icons::outlined::ICON_STORAGE),
        
        // Configuration & Settings
        "SETTINGS" => Some(md_icons::outlined::ICON_SETTINGS),
        
        // Development & Code
        "CODE" => Some(md_icons::outlined::ICON_CODE),
        "BUILD" => Some(md_icons::outlined::ICON_BUILD),
        
        _ => {
            warn!("Unknown outlined icon: {}, using default terminal icon", const_name);
            Some(md_icons::outlined::ICON_TERMINAL)
        }
    }
}

/// Resolves sharp style Material Design icons
fn resolve_sharp_icon(const_name: &str) -> Option<&'static str> {
    match const_name {
        // Navigation & Control
        "TERMINAL" => Some(md_icons::sharp::ICON_TERMINAL),
        "HOME" => Some(md_icons::sharp::ICON_HOME),
        "ARROW_BACK" => Some(md_icons::sharp::ICON_ARROW_BACK),
        "ARROW_FORWARD" => Some(md_icons::sharp::ICON_ARROW_FORWARD),
        "ARROW_UPWARD" => Some(md_icons::sharp::ICON_ARROW_UPWARD),
        "ARROW_DOWNWARD" => Some(md_icons::sharp::ICON_ARROW_DOWNWARD),
        
        // Files & Folders
        "FOLDER" => Some(md_icons::sharp::ICON_FOLDER),
        "FOLDER_OPEN" => Some(md_icons::sharp::ICON_FOLDER_OPEN),
        
        // Configuration & Settings
        "SETTINGS" => Some(md_icons::sharp::ICON_SETTINGS),
        
        _ => {
            warn!("Unknown sharp icon: {}, using default terminal icon", const_name);
            Some(md_icons::sharp::ICON_TERMINAL)
        }
    }
}

/// Resolves two-tone style Material Design icons
fn resolve_two_tone_icon(const_name: &str) -> Option<&'static str> {
    match const_name {
        // Navigation & Control
        "TERMINAL" => Some(md_icons::two_tone::ICON_TERMINAL),
        "HOME" => Some(md_icons::two_tone::ICON_HOME),
        "ARROW_BACK" => Some(md_icons::two_tone::ICON_ARROW_BACK),
        "ARROW_FORWARD" => Some(md_icons::two_tone::ICON_ARROW_FORWARD),
        "ARROW_UPWARD" => Some(md_icons::two_tone::ICON_ARROW_UPWARD),
        "ARROW_DOWNWARD" => Some(md_icons::two_tone::ICON_ARROW_DOWNWARD),
        
        // Files & Folders
        "FOLDER" => Some(md_icons::two_tone::ICON_FOLDER),
        "FOLDER_OPEN" => Some(md_icons::two_tone::ICON_FOLDER_OPEN),
        
        // Configuration & Settings
        "SETTINGS" => Some(md_icons::two_tone::ICON_SETTINGS),
        
        _ => {
            warn!("Unknown two_tone icon: {}, using default terminal icon", const_name);
            Some(md_icons::two_tone::ICON_TERMINAL)
        }
    }
}