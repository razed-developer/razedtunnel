use tauri::AppHandle;
use crate::desktop_control::{self, DesktopControlApplication};

#[tauri::command]
pub(crate) fn get_visual_access_status(app: AppHandle) -> Result<bool, String> {
    desktop_control::is_enabled(&app, "")
}

#[tauri::command]
pub(crate) fn set_visual_access_enabled(app: AppHandle, enabled: bool) -> Result<bool, String> {
    desktop_control::set_global_enabled(&app, "", enabled)
}

#[tauri::command]
pub(crate) async fn list_visual_access_applications(app: AppHandle) -> Result<Vec<DesktopControlApplication>, String> {
    // UI Automation needs a fresh COM apartment, and enumeration must not block
    // the Tauri UI thread while Connections mounts.
    tauri::async_runtime::spawn_blocking(move || {
        let ignored = read_ignored(&app)?;
        let apps = desktop_control::list(&app, "")?;
        Ok(apps.into_iter().filter(|candidate| {
            match executable_identity(&candidate.id) {
                Ok(path) => !ignored.iter().any(|entry| entry.executable_path.eq_ignore_ascii_case(&path)),
                Err(_) => true, // Unknown identities remain visible; never hide by PID/title.
            }
        }).collect())
    })
        .await
        .map_err(|error| format!("UI Automation discovery worker failed: {error}"))?
}

#[tauri::command]
pub(crate) async fn capture_visual_access_window(
    app: AppHandle,
    application_id: String,
) -> Result<desktop_control::DesktopScreenshot, String> {
    // The existing screenshot service enforces local Desktop Access and validates
    // the selected application/window. UIA must initialize on a fresh COM thread.
    tauri::async_runtime::spawn_blocking(move || {
        desktop_control::screenshot(&app, "", &application_id, None)
    })
    .await
    .map_err(|error| format!("Window capture worker failed: {error}"))?
}

#[cfg(windows)]
fn executable_identity(application_id: &str) -> Result<String, String> {
    use std::ffi::c_void;
    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut c_void;
        fn QueryFullProcessImageNameW(process: *mut c_void, flags: u32, name: *mut u16, size: *mut u32) -> i32;
        fn CloseHandle(handle: *mut c_void) -> i32;
    }
    let pid = application_id.strip_prefix("uia-pid-")
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| *value != 0)
        .ok_or_else(|| "Invalid application identity.".to_string())?;
    unsafe {
        let process = OpenProcess(0x1000, 0, pid); // PROCESS_QUERY_LIMITED_INFORMATION
        if process.is_null() {
            return Err("Cannot inspect the selected application's executable.".to_string());
        }
        let mut buffer = vec![0u16; 32768];
        let mut length = buffer.len() as u32;
        let ok = QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut length);
        CloseHandle(process);
        if ok == 0 || length == 0 {
            return Err("Cannot resolve the selected application's executable path.".to_string());
        }
        let path = String::from_utf16_lossy(&buffer[..length as usize]);
        if path.is_empty() {
            return Err("Application executable identity is empty.".to_string());
        }
        Ok(path)
    }
}

#[tauri::command]
pub(crate) async fn get_visual_access_application_identity(application_id: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        #[cfg(windows)]
        { executable_identity(&application_id) }
        #[cfg(not(windows))]
        { let _ = application_id; Err("Executable identity is currently supported on Windows only.".to_string()) }
    })
    .await
    .map_err(|error| format!("Application identity worker failed: {error}"))?
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IgnoredVisualApplication {
    pub(crate) executable_path: String,
    pub(crate) name: String,
}

fn ignored_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    use tauri::Manager;
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("visual-access-ignored.json"))
        .map_err(|error| format!("Cannot resolve Visual Access settings: {error}"))
}

fn read_ignored(app: &AppHandle) -> Result<Vec<IgnoredVisualApplication>, String> {
    let path = ignored_path(app)?;
    if !path.exists() { return Ok(Vec::new()); }
    let bytes = std::fs::read(&path).map_err(|error| format!("Cannot read ignored applications: {error}"))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("Invalid ignored applications settings: {error}"))
}

#[tauri::command]
pub(crate) fn list_ignored_visual_applications(app: AppHandle) -> Result<Vec<IgnoredVisualApplication>, String> {
    read_ignored(&app)
}

#[tauri::command]
pub(crate) async fn set_visual_application_ignored(
    app: AppHandle,
    application_id: Option<String>,
    executable_path: Option<String>,
    name: Option<String>,
    ignored: bool,
) -> Result<Vec<IgnoredVisualApplication>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = if ignored {
            let id = application_id.ok_or("Select a currently running application to ignore.")?;
            #[cfg(windows)]
            { executable_identity(&id)? }
            #[cfg(not(windows))]
            { let _ = id; return Err("Application identity is supported on Windows only.".to_string()); }
        } else {
            executable_path.ok_or("Executable path is required to restore an ignored application.")?
        };
        let mut items = read_ignored(&app)?;
        items.retain(|entry| !entry.executable_path.eq_ignore_ascii_case(&path));
        if ignored {
            items.push(IgnoredVisualApplication {
                executable_path: path,
                name: name.unwrap_or_else(|| "Application".to_string()).chars().take(256).collect(),
            });
        }
        items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        let file = ignored_path(&app)?;
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent).map_err(|error| format!("Cannot create settings directory: {error}"))?;
        }
        let data = serde_json::to_vec_pretty(&items).map_err(|error| error.to_string())?;
        std::fs::write(file, data).map_err(|error| format!("Cannot save ignored applications: {error}"))?;
        Ok(items)
    })
    .await
    .map_err(|error| format!("Ignored applications worker failed: {error}"))?
}
