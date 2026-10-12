use tauri::AppHandle;
use crate::{activity, browser, projects::approved_workspace, models::{BrowserActionOutcome, BrowserActionRecord, BrowserApplication, BrowserAutomationStatus, BrowserDiagnostics, BrowserDownload, BrowserDownloadSetup, BrowserPageInspection, BrowserScreenshot, BrowserTab, BrowserUploadResult, BrowserVisualSelection}};

#[tauri::command]
pub fn list_automation_browsers(
    app: AppHandle,
    workspace_id: String,
) -> Result<Vec<BrowserApplication>, String> {
    let _workspace = approved_workspace(&app, &workspace_id)?;
    Ok(browser::list_applications())
}

#[tauri::command]
pub fn get_browser_automation_status(
    app: AppHandle,
    workspace_id: String,
) -> Result<BrowserAutomationStatus, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    Ok(browser::status(&app, &scope))
}

#[tauri::command]
pub async fn start_browser_automation(
    app: AppHandle,
    workspace_id: String,
    application_id: String,
) -> Result<BrowserActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::request_start(&app_for_task, &scope, &application_id)
    })
    .await
    .map_err(|error| format!("Browser start task could not complete: {error}"))?
}

#[tauri::command]
pub async fn stop_browser_automation(
    app: AppHandle,
    workspace_id: String,
) -> Result<BrowserActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || browser::request_stop(&app_for_task, &scope))
        .await
        .map_err(|error| format!("Browser stop task could not complete: {error}"))?
}

#[tauri::command]
pub async fn list_browser_tabs(
    app: AppHandle,
    workspace_id: String,
) -> Result<Vec<BrowserTab>, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || browser::list_tabs(&app_for_task, &scope))
        .await
        .map_err(|error| format!("Browser tab task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_open_tab(
    app: AppHandle,
    workspace_id: String,
    url: String,
) -> Result<BrowserActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::request_open_tab(&app_for_task, &scope, &url)
    })
    .await
    .map_err(|error| format!("Browser open-tab task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_activate_tab(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
) -> Result<BrowserActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::request_activate_tab(&app_for_task, &scope, &tab_id)
    })
    .await
    .map_err(|error| format!("Browser activate-tab task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_close_tab(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
) -> Result<BrowserActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::request_close_tab(&app_for_task, &scope, &tab_id)
    })
    .await
    .map_err(|error| format!("Browser close-tab task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_navigate(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
    url: String,
) -> Result<BrowserActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::request_navigate(&app_for_task, &scope, &tab_id, &url)
    })
    .await
    .map_err(|error| format!("Browser navigation task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_click(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
    selector: String,
) -> Result<BrowserActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::request_click(&app_for_task, &scope, &tab_id, &selector)
    })
    .await
    .map_err(|error| format!("Browser click task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_type(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
    selector: String,
    text: String,
    clear_first: bool,
) -> Result<BrowserActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::request_type(
            &app_for_task,
            &scope,
            &tab_id,
            &selector,
            &text,
            clear_first,
        )
    })
    .await
    .map_err(|error| format!("Browser typing task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_scroll(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
    delta_x: i32,
    delta_y: i32,
) -> Result<BrowserActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::request_scroll(&app_for_task, &scope, &tab_id, delta_x, delta_y)
    })
    .await
    .map_err(|error| format!("Browser scroll task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_reload(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
) -> Result<BrowserActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::request_reload(&app_for_task, &scope, &tab_id)
    })
    .await
    .map_err(|error| format!("Browser reload task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_configure_downloads(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
    task_id: String,
) -> Result<BrowserDownloadSetup, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::configure_downloads(&app_for_task, &scope, &tab_id, &task_id)
    })
    .await
    .map_err(|error| format!("Browser download configuration task could not complete: {error}"))?
}

#[tauri::command]
pub fn list_browser_downloads(
    app: AppHandle,
    workspace_id: String,
) -> Result<Vec<BrowserDownload>, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    browser::list_downloads(&app, &scope)
}

#[tauri::command]
pub async fn cancel_browser_download(
    app: AppHandle,
    workspace_id: String,
    guid: String,
) -> Result<BrowserDownload, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::cancel_download(&app_for_task, &scope, &guid)
    })
    .await
    .map_err(|error| format!("Browser download cancel task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_upload_file(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
    selector: String,
    relative_path: String,
) -> Result<BrowserUploadResult, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::upload_file(&app_for_task, &scope, &tab_id, &selector, &relative_path)
    })
    .await
    .map_err(|error| format!("Browser upload task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_inspect_page(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
    selector: Option<String>,
    max_chars: Option<usize>,
) -> Result<BrowserPageInspection, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::inspect_page(
            &app_for_task,
            &scope,
            &tab_id,
            selector.as_deref(),
            max_chars.unwrap_or(12000),
        )
    })
    .await
    .map_err(|error| format!("Browser inspection task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_pick_element(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
    x_ratio: f64,
    y_ratio: f64,
) -> Result<BrowserVisualSelection, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::pick_visual_element(&app_for_task, &scope, &tab_id, x_ratio, y_ratio)
    })
    .await
    .map_err(|error| format!("Browser visual selection task could not complete: {error}"))?
}

#[tauri::command]
pub fn get_browser_visual_selection(
    app: AppHandle,
    workspace_id: String,
) -> Result<Option<BrowserVisualSelection>, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    browser::get_visual_selection(&scope)
}

#[tauri::command]
pub async fn browser_take_screenshot(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
    full_page: bool,
) -> Result<BrowserScreenshot, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::screenshot(&app_for_task, &scope, &tab_id, full_page)
    })
    .await
    .map_err(|error| format!("Browser screenshot task could not complete: {error}"))?
}

#[tauri::command]
pub fn get_browser_diagnostics(
    app: AppHandle,
    workspace_id: String,
    tab_id: Option<String>,
    limit: Option<usize>,
) -> Result<BrowserDiagnostics, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    browser::diagnostics(&app, &scope, tab_id.as_deref(), limit.unwrap_or(40))
}
#[tauri::command]
pub fn list_browser_history(
    app: AppHandle,
    workspace_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<BrowserActionRecord>, String> {
    browser::list_history(&app, workspace_id.as_deref(), limit.unwrap_or(60))
}

#[tauri::command]
pub async fn approve_browser_action(
    app: AppHandle,
    action_id: String,
) -> Result<BrowserActionRecord, String> {
    let record = browser::get_action(&app, &action_id)?;
    let workspace = approved_workspace(&app, &record.workspace_id)?;
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let approved = browser::approve_action(&app_for_task, &workspace, &action_id)?;
        activity::sync_browser(&app_for_task, &approved);
        Ok(approved)
    })
    .await
    .map_err(|error| format!("Browser approval task could not complete: {error}"))?
}

#[tauri::command]
pub fn reject_browser_action(
    app: AppHandle,
    action_id: String,
) -> Result<BrowserActionRecord, String> {
    let record = browser::reject_action(&app, &action_id)?;
    activity::sync_browser(&app, &record);
    Ok(record)
}

