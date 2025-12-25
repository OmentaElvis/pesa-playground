use anyhow::Context;
use base64::{Engine, engine::general_purpose};
use rsa::{Pkcs1v15Encrypt, pkcs8::DecodePublicKey};

use crate::{
    AppContext,
    settings::models::{AppSettings, EncryptionKeys},
};

pub async fn get_settings(context: &AppContext) -> anyhow::Result<AppSettings> {
    Ok(context.settings.get().await)
}

pub async fn set_settings(context: &AppContext, settings: AppSettings) -> anyhow::Result<()> {
    context.settings.set(settings.clone()).await?;
    context
        .event_manager
        .emit_all("settings_updated", serde_json::to_value(settings)?)?;
    Ok(())
}

pub async fn generate_security_credential(
    context: &AppContext,
    password: String,
) -> anyhow::Result<String> {
    let mut settings = get_settings(context).await?;
    if settings.encryption_keys.is_none() {
        let keys = EncryptionKeys::init()?;
        settings.encryption_keys = Some(keys);
        set_settings(context, settings.clone()).await?;
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
