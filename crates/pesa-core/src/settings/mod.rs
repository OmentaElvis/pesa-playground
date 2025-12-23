use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::fs;

pub mod models;
pub mod ui;

#[derive(Debug, Clone)]
pub struct SettingsManager {
    path: PathBuf,
    settings: Arc<RwLock<models::AppSettings>>,
}

impl SettingsManager {
    /// Creates a new SettingsManager, loading from the given path.
    /// If the file doesn't exist, it creates one with default settings.
    pub async fn new(path: PathBuf) -> anyhow::Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let settings = if fs::try_exists(&path).await? {
            let content = fs::read_to_string(&path).await?;
            serde_json::from_str(&content).unwrap_or_else(|_| models::AppSettings::default())
        } else {
            let default_settings = models::AppSettings::default();
            let content = serde_json::to_string_pretty(&default_settings)?;
            fs::write(&path, content).await?;
            default_settings
        };

        Ok(Self {
            path,
            settings: Arc::new(RwLock::new(settings)),
        })
    }

    /// Returns a clone of the current settings.
    /// This is a fast, in-memory read.
    pub fn get(&self) -> models::AppSettings {
        self.settings.read().unwrap().clone()
    }

    /// Updates the settings in memory and atomically saves them to the disk.
    pub async fn set(&self, new_settings: models::AppSettings) -> anyhow::Result<()> {
        *self.settings.write().unwrap() = new_settings;
        self.save().await
    }

    /// Atomically saves the current in-memory settings to the disk asynchronously.
    async fn save(&self) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(&*self.settings.read().unwrap())?;
        
        let dir = self.path.parent().unwrap_or_else(|| Path::new("."));
        let temp_path = dir.join(format!("{}.tmp.{}", self.path.file_name().unwrap().to_str().unwrap() ,nanoid::nanoid!(6)));

        tokio::fs::write(&temp_path, content).await?;
        tokio::fs::rename(&temp_path, &self.path).await?;

        Ok(())
    }
}
