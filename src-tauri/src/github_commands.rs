use crate::github;

#[tauri::command]
pub async fn get_github_connection_status() -> Result<github::GithubConnectionStatus, String> {
    if let Some(cached) = github::cached_status() {
        // UI reads the already-prewarmed global state immediately. Refresh the
        // authoritative CLI status in the background so later reads stay current.
        let _ = std::thread::Builder::new()
            .name("repotunnel-github-status-refresh".to_string())
            .spawn(|| {
                let _ = github::status();
            });
        return Ok(cached);
    }

    tauri::async_runtime::spawn_blocking(github::status)
        .await
        .map_err(|error| format!("The GitHub status check could not complete: {error}"))
}

#[tauri::command]
pub async fn connect_github() -> Result<github::GithubConnectionStatus, String> {
    tauri::async_runtime::spawn_blocking(github::connect)
        .await
        .map_err(|error| format!("The GitHub connection task could not complete: {error}"))?
}

#[tauri::command]
pub async fn cancel_github_connection() -> Result<github::GithubConnectionStatus, String> {
    tauri::async_runtime::spawn_blocking(github::cancel_connection)
        .await
        .map_err(|error| format!("The GitHub connection cancel task could not complete: {error}"))?
}

#[tauri::command]
pub async fn disconnect_github() -> Result<github::GithubConnectionStatus, String> {
    tauri::async_runtime::spawn_blocking(github::disconnect)
        .await
        .map_err(|error| format!("The GitHub disconnect task could not complete: {error}"))?
}

