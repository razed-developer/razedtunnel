use tauri::{AppHandle, Manager, State};
use crate::{app_state::AppState, phone};

#[tauri::command]
pub async fn get_phone_discovery() -> Result<phone::PhoneDiscoveryStatus, String> {
    tauri::async_runtime::spawn_blocking(phone::discover)
        .await
        .map_err(|error| format!("Phone discovery worker could not complete: {error}"))
}

#[tauri::command]
pub async fn pair_phone_wirelessly(code: String) -> Result<phone::PhoneDiscoveryStatus, String> {
    tauri::async_runtime::spawn_blocking(move || phone::pair_wirelessly(&code))
        .await
        .map_err(|error| format!("Phone pairing worker could not complete: {error}"))?
}

#[tauri::command]
pub fn get_phone_access_status(app: AppHandle) -> Result<phone::PhoneAccessStatus, String> {
    phone::access_status(&app)
}

#[tauri::command]
pub async fn select_phone_device(
    app: AppHandle,
    device_id: Option<String>,
) -> Result<phone::PhoneAccessStatus, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let status = phone::select_device(&app, device_id)?;
        let state = app.state::<AppState>();
        if state.phone.status()?.device_id != status.selected_device_id {
            let _ = state.phone.clear()?;
        }
        Ok(status)
    })
    .await
    .map_err(|error| format!("Phone selection worker could not complete: {error}"))?
}

#[tauri::command]
pub async fn set_phone_access_mode(
    app: AppHandle,
    device_id: String,
    mode: phone::PhoneAccessMode,
    limited_capabilities: Vec<phone::PhoneCapability>,
) -> Result<phone::PhoneAccessStatus, String> {
    tauri::async_runtime::spawn_blocking(move || {
        phone::set_access_mode(&app, device_id, mode, limited_capabilities)
    })
    .await
    .map_err(|error| format!("Phone access worker could not complete: {error}"))?
}

#[tauri::command]
pub fn set_phone_access_paused(
    app: AppHandle,
    paused: bool,
) -> Result<phone::PhoneAccessStatus, String> {
    phone::set_access_paused(&app, paused)
}

#[tauri::command]
pub fn get_phone_runtime_status(
    state: State<'_, AppState>,
) -> Result<phone::PhoneRuntimeStatus, String> {
    state.phone.status()
}

#[tauri::command]
pub async fn ensure_phone_runtime(
    app: AppHandle,
    device_id: String,
    preferred_transport: Option<String>,
) -> Result<phone::PhoneRuntimeStatus, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        state
            .phone
            .ensure_preferred(&device_id, preferred_transport.as_deref())
    })
    .await
    .map_err(|error| format!("Phone runtime worker could not complete: {error}"))?
}

#[tauri::command]
pub fn clear_phone_runtime(
    state: State<'_, AppState>,
) -> Result<phone::PhoneRuntimeStatus, String> {
    state.phone.clear()
}

#[tauri::command]
pub async fn probe_phone_runtime(
    app: AppHandle,
    device_id: String,
) -> Result<phone::PhoneRuntimeProbe, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        state.phone.probe_display(&app, &device_id)
    })
    .await
    .map_err(|error| format!("Phone runtime probe could not complete: {error}"))?
}

#[tauri::command]
pub async fn get_phone_screen_frame(
    app: AppHandle,
    device_id: String,
    after_captured_at: Option<u64>,
) -> Result<phone::PhoneScreenFrame, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        state
            .phone
            .local_ui_screen_frame(&app, &device_id, after_captured_at)
    })
    .await
    .map_err(|error| format!("Phone screen capture could not complete: {error}"))?
}

#[tauri::command]
pub async fn phone_tap(
    app: AppHandle,
    device_id: String,
    x_ratio: f64,
    y_ratio: f64,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        state.phone.local_ui_tap(&app, &device_id, x_ratio, y_ratio)
    })
    .await
    .map_err(|error| format!("Phone tap worker could not complete: {error}"))?
}

#[tauri::command]
pub async fn phone_swipe(
    app: AppHandle,
    device_id: String,
    start_x_ratio: f64,
    start_y_ratio: f64,
    end_x_ratio: f64,
    end_y_ratio: f64,
    duration_ms: u32,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        state.phone.local_ui_swipe(
            &app,
            &device_id,
            phone::PhoneSwipeGesture {
                start_x_ratio,
                start_y_ratio,
                end_x_ratio,
                end_y_ratio,
                duration_ms,
            },
        )
    })
    .await
    .map_err(|error| format!("Phone swipe worker could not complete: {error}"))?
}

#[tauri::command]
pub async fn phone_key_event(
    app: AppHandle,
    device_id: String,
    key: phone::PhoneKey,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        state.phone.local_ui_key_event(&app, &device_id, key)
    })
    .await
    .map_err(|error| format!("Phone key worker could not complete: {error}"))?
}

#[tauri::command]
pub async fn phone_type_text(
    app: AppHandle,
    device_id: String,
    text: String,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        state.phone.local_ui_type_text(&app, &device_id, &text)
    })
    .await
    .map_err(|error| format!("Phone text worker could not complete: {error}"))?
}

