use super::{UserResponse, STK_RESPONSE_REGISTRY};

#[tauri::command]
pub async fn resolve_stk_prompt(checkout_id: String, result: UserResponse) {
    let mut reg = STK_RESPONSE_REGISTRY.lock().await;
    if let Some(sender) = reg.remove(&checkout_id) {
        let _ = sender.send(result);
    }
}
