use anyhow::Result;
use crate::AppContext;

use super::{UserResponse, STK_RESPONSE_REGISTRY};

pub async fn resolve_stk_prompt(_: &AppContext, checkout_id: String, result: UserResponse) -> Result<()> {
    let mut reg = STK_RESPONSE_REGISTRY.lock().await;
    if let Some(sender) = reg.remove(&checkout_id) {
        let _ = sender.send(result);
    }
    Ok(())
}
