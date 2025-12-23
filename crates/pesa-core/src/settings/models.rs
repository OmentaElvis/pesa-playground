use anyhow::Context;
use rsa::RsaPrivateKey;
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncryptionKeys {
    pub public_key: String,
    pub private_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppSettings {
    #[serde(default = "default_theme")]
    pub theme: Theme,
    #[serde(default)]
    pub server_log_level: LogLevel,
    pub encryption_keys: Option<EncryptionKeys>,
}

fn default_theme() -> Theme {
    Theme::Dark
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            server_log_level: LogLevel::Info,
            encryption_keys: None,
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

impl EncryptionKeys {
    pub fn init() -> anyhow::Result<Self> {
        let mut rng = rand::thread_rng();
        let private_key =
            RsaPrivateKey::new(&mut rng, 2048).context("Failed to generate private key")?;
        let public_key = private_key.to_public_key();

        let public_key = public_key
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .context("Failed to encode public key")?;

        let private_key = private_key
            .to_pkcs8_der()
            .context("Failed to encode private key")?
            .to_pem("PRIVATE KEY", rsa::pkcs8::LineEnding::LF)
            .context("Failed to encode private key to PEM")?
            .to_string();

        Ok(Self {
            public_key,
            private_key,
        })
    }
}
