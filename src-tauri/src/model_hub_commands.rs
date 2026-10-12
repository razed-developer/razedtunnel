use tauri::AppHandle;
use crate::model_hub::{self, ModelHubSnapshot, ModelProviderId, ModelSelection, ModelTestResult, RuntimeStatus};

#[tauri::command]
pub async fn get_model_hub(app: AppHandle) -> Result<ModelHubSnapshot, String> {
    model_hub::snapshot(&app).await
}

#[tauri::command]
pub async fn refresh_model_runtime(
    app: AppHandle,
    provider: ModelProviderId,
) -> Result<RuntimeStatus, String> {
    let endpoint = model_hub::configured_endpoint(&app, provider)?;
    Ok(model_hub::discover_provider(provider, endpoint).await)
}

#[tauri::command]
pub fn update_model_runtime_endpoint(
    app: AppHandle,
    provider: ModelProviderId,
    endpoint: String,
) -> Result<String, String> {
    model_hub::update_endpoint(&app, provider, endpoint)
}

#[tauri::command]
pub fn set_selected_local_model(
    app: AppHandle,
    selection: Option<ModelSelection>,
) -> Result<Option<ModelSelection>, String> {
    model_hub::set_selected_model(&app, selection)
}

#[tauri::command]
pub async fn test_local_model(
    app: AppHandle,
    selection: ModelSelection,
) -> Result<ModelTestResult, String> {
    let configured = model_hub::configured_endpoint(&app, selection.provider)?;
    let requested = model_hub::validate_loopback_endpoint(&selection.endpoint)?;
    if requested != configured {
        return Err("This model no longer matches the configured local runtime. Refresh Model Hub and try again.".to_string());
    }
    model_hub::test_model(selection).await
}

