use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppSettings {
    #[serde(default = "default_theme")]
    pub theme: Theme,
    #[serde(default)]
    pub server_log_level: LogLevel,
}

fn default_theme() -> Theme {
    Theme::Dark
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            server_log_level: LogLevel::Info,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Dark,
    Light,
}
