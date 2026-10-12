//! Shared FFmpeg/FFprobe discovery for media inspection and phone workflows.
//! Provisioning remains delegated to legacy video code until safely extracted.

use std::{env, fs, path::PathBuf};
use tauri::{path::BaseDirectory, AppHandle, Manager};

fn executable_name(name: &str) -> String {
    if cfg!(windows) { format!("{name}.exe") } else { name.to_string() }
}

fn discover_program(app: &AppHandle, name: &str) -> Option<PathBuf> {
    // Preserve the legacy precedence: private managed helpers before PATH.
    // The managed directory is created even on a lookup, as in video::tools_root.
    let managed_root = app.path().resolve("video-tools", BaseDirectory::AppData).ok();
    if let Some(root) = managed_root {
        if fs::create_dir_all(&root).is_ok() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).is_ok() {
                    let candidate = root.join(executable_name(name));
                    if candidate.is_file() { return Some(candidate); }
                }
            }
            #[cfg(not(unix))]
            {
                let candidate = root.join(executable_name(name));
                if candidate.is_file() { return Some(candidate); }
            }
        }
    }
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .map(|directory| directory.join(executable_name(name)))
            .find(|candidate| candidate.is_file())
    })
}

pub(crate) fn available_ffmpeg_program(app: &AppHandle) -> Option<PathBuf> {
    discover_program(app, "ffmpeg")
}

pub(crate) fn available_ffprobe_program(app: &AppHandle) -> Option<PathBuf> {
    discover_program(app, "ffprobe")
}

pub(crate) fn ensure_ffmpeg_program(app: &AppHandle) -> Result<PathBuf, String> {
    crate::video::ensure_ffmpeg_program(app)
}
