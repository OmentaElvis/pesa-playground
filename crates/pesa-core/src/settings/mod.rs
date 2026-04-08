use anyhow::Context;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

use crate::settings::models::{AppSettings, EncryptionKeys};

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
            let keys = EncryptionKeys::init()?;
            let default_settings = AppSettings {
                encryption_keys: Some(keys),
                ..Default::default()
            };
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
    pub async fn get(&self) -> models::AppSettings {
        self.settings.read().await.clone()
    }

    /// Updates the settings in memory and atomically saves them to the disk.
    pub async fn set(&self, new_settings: models::AppSettings) -> anyhow::Result<()> {
        *self.settings.write().await = new_settings;
        self.save().await
    }

    /// Atomically saves the current in-memory settings to the disk asynchronously.
    async fn save(&self) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(&*self.settings.read().await)?;

        let dir = self.path.parent().unwrap_or_else(|| Path::new("."));
        let temp_path = dir.join(format!(
            "{}.tmp.{}",
            self.path.file_name().unwrap().to_str().unwrap(),
            nanoid::nanoid!(6)
        ));

        tokio::fs::write(&temp_path, content).await?;
        tokio::fs::rename(&temp_path, &self.path).await?;

        Ok(())
    }

    pub async fn generate_security_credential(&self, password: &str) -> anyhow::Result<String> {
        use base64::{Engine, engine::general_purpose};
        use rsa::{Pkcs1v15Encrypt, pkcs8::DecodePublicKey};

        let mut settings = self.get().await;
        if settings.encryption_keys.is_none() {
            let keys = EncryptionKeys::init()?;
            settings.encryption_keys = Some(keys);
            self.set(settings.clone()).await?;
        }

        let keys = settings
            .encryption_keys
            .as_ref()
            .expect("Keys should not be empty");

        let public_key = rsa::RsaPublicKey::from_public_key_pem(&keys.public_key)
            .context("Failed to decode public key")?;

        let mut rng = rand::thread_rng();
        let enc_password = public_key
            .encrypt(&mut rng, Pkcs1v15Encrypt, password.as_bytes())
            .context("Failed to encrypt password")?;

        let code = general_purpose::STANDARD.encode(enc_password);

        Ok(code)
    }
}

#[cfg(test)]
mod tests {
    use crate::settings::models::{LogLevel, Theme};

    use super::*;
    use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, pkcs8::DecodePrivateKey};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_settings_manager_new_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("settings.json");

        let manager = SettingsManager::new(path.clone()).await.unwrap();

        assert!(path.exists());

        let settings = manager.get().await;
        assert!(settings.encryption_keys.is_some());
    }

    #[tokio::test]
    async fn test_settings_manager_new_loads_existing() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("settings.json");

        let custom_settings = AppSettings {
            theme: Theme::Light,
            server_log_level: LogLevel::Debug,
            encryption_keys: None,
            custom_keymaps: None,
        };
        let content = serde_json::to_string_pretty(&custom_settings).unwrap();
        tokio::fs::write(&path, content).await.unwrap();

        let manager = SettingsManager::new(path.clone()).await.unwrap();

        let settings = manager.get().await;
        assert_eq!(settings.theme, Theme::Light);
        assert_eq!(settings.server_log_level, LogLevel::Debug);
    }

    #[tokio::test]
    async fn test_settings_manager_get() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("settings.json");

        let manager = SettingsManager::new(path).await.unwrap();

        let settings = manager.get().await;
        assert_eq!(settings.theme, Theme::Dark);
        assert_eq!(settings.server_log_level, LogLevel::Info);
    }

    #[tokio::test]
    async fn test_settings_manager_set_updates_and_persists() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("settings.json");

        let manager = SettingsManager::new(path.clone()).await.unwrap();

        let new_settings = AppSettings {
            theme: Theme::Light,
            server_log_level: LogLevel::Warn,
            encryption_keys: None,
            custom_keymaps: None,
        };

        manager.set(new_settings.clone()).await.unwrap();

        let loaded_settings = manager.get().await;
        assert_eq!(loaded_settings.theme, Theme::Light);
        assert_eq!(loaded_settings.server_log_level, LogLevel::Warn);

        let content = tokio::fs::read_to_string(&path).await.unwrap();
        let persisted: AppSettings = serde_json::from_str(&content).unwrap();
        assert_eq!(persisted.theme, Theme::Light);
    }

    #[tokio::test]
    async fn test_encryption_keys_init() {
        let keys = EncryptionKeys::init().unwrap();

        assert!(!keys.public_key.is_empty());
        assert!(!keys.private_key.is_empty());
        assert!(keys.public_key.contains("-----BEGIN PUBLIC KEY-----"));
        assert!(keys.private_key.contains("-----BEGIN PRIVATE KEY-----"));
    }

    #[tokio::test]
    async fn test_generate_security_credential_and_decrypt() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("settings.json");

        let manager = SettingsManager::new(path).await.unwrap();

        let password = "my_secure_password_123";
        let encrypted = manager
            .generate_security_credential(password)
            .await
            .unwrap();

        let settings = manager.get().await;
        let keys = settings.encryption_keys.as_ref().unwrap();

        use base64::Engine;
        let private_key = RsaPrivateKey::from_pkcs8_pem(&keys.private_key).unwrap();

        let encrypted_bytes = base64::engine::general_purpose::STANDARD
            .decode(&encrypted)
            .unwrap();

        let decrypted = private_key
            .decrypt(Pkcs1v15Encrypt, &encrypted_bytes)
            .unwrap();
        let decrypted_password = String::from_utf8(decrypted).unwrap();

        assert_eq!(decrypted_password, password);
    }
}
