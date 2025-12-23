use crate::{settings::models::AppSettings, AppContext};

pub async fn get_settings(context: &AppContext) -> anyhow::Result<AppSettings> {
    Ok(context.settings.get())
}

pub async fn set_settings(context: &AppContext, settings: AppSettings) -> anyhow::Result<()> {
    context.settings.set(settings.clone()).await?;
    context
        .event_manager
        .emit_all("settings_updated", serde_json::to_value(settings)?)?;
    Ok(())
}
