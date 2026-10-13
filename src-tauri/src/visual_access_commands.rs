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
    tauri::async_runtime::spawn_blocking(move || desktop_control::list(&app, ""))
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
