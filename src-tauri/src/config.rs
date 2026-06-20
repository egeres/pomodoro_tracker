use serde::{Deserialize, Serialize};
use std::fs::create_dir_all;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub output_directory: String,
    pub default_pomodoro_time_minutes: i32,
    /// Shell command run when a pomodoro starts. Empty means "do nothing".
    #[serde(default)]
    pub command_start: String,
    /// Shell command run when a pomodoro ends. Empty means "do nothing".
    #[serde(default)]
    pub command_end: String,
}

/// Returns the user's home directory, falling back to the current directory.
pub fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

impl Default for AppConfig {
    fn default() -> Self {
        let output = home_dir().join("Pomodoros");
        AppConfig {
            output_directory: output.to_string_lossy().to_string(),
            default_pomodoro_time_minutes: 25,
            command_start: String::new(),
            command_end: String::new(),
        }
    }
}

/// `HOME/.config/pomodoro_app`
pub fn config_dir() -> PathBuf {
    home_dir().join(".config").join("pomodoro_app")
}

/// `HOME/.config/pomodoro_app/config.json`
pub fn config_file_path() -> PathBuf {
    config_dir().join("config.json")
}

/// Reads the config from disk, creating it with defaults when missing or invalid.
pub fn load_or_create_config() -> AppConfig {
    let path = config_file_path();

    if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str::<AppConfig>(&contents) {
                Ok(config) => return config,
                Err(e) => eprintln!("Error parsing config, using defaults: {:?}", e),
            },
            Err(e) => eprintln!("Error reading config, using defaults: {:?}", e),
        }
    }

    let config = AppConfig::default();
    save_config_to_disk(&config);
    config
}

/// Writes the config as UTF-8 JSON with 4-space indentation.
pub fn save_config_to_disk(config: &AppConfig) {
    let dir = config_dir();
    if !dir.exists() {
        if let Err(e) = create_dir_all(&dir) {
            eprintln!("Error creating config directory: {:?}", e);
            return;
        }
    }

    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut buffer = Vec::new();
    let mut serializer = serde_json::Serializer::with_formatter(&mut buffer, formatter);

    if let Err(e) = config.serialize(&mut serializer) {
        eprintln!("Error serializing config: {:?}", e);
        return;
    }

    let json = match String::from_utf8(buffer) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Error converting config to UTF-8: {:?}", e);
            return;
        }
    };

    if let Err(e) = std::fs::write(config_file_path(), json) {
        eprintln!("Error writing config file: {:?}", e);
    }
}
