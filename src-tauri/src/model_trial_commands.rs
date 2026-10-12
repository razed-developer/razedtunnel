use tauri::AppHandle;
use crate::{model_hub::ModelSelection, model_trial::{self, ModelTrialSnapshot, TrialMode}};

#[tauri::command]
pub async fn get_model_trial(app: AppHandle) -> Result<ModelTrialSnapshot, String> {
    model_trial::snapshot(&app).await
}

#[tauri::command]
pub async fn run_model_trial(
    app: AppHandle,
    mode: TrialMode,
    selections: Vec<ModelSelection>,
) -> Result<ModelTrialSnapshot, String> {
    model_trial::run_trial(&app, mode, selections).await
}

#[tauri::command]
pub fn cancel_model_trial(app: AppHandle) -> Result<bool, String> {
    model_trial::cancel_trial(&app)
}

