//! Shared FFmpeg/FFprobe lookup used by media inspection and phone workflows.
//! The implementation currently delegates to the legacy video helper until its
//! provisioning logic can be extracted without changing runtime behavior.

use std::path::PathBuf;
use tauri::AppHandle;

pub(crate) fn available_ffmpeg_program(app: &AppHandle) -> Option<PathBuf> {
    crate::video::available_ffmpeg_program(app)
}

pub(crate) fn available_ffprobe_program(app: &AppHandle) -> Option<PathBuf> {
    crate::video::available_ffprobe_program(app)
}

pub(crate) fn ensure_ffmpeg_program(app: &AppHandle) -> Result<PathBuf, String> {
    crate::video::ensure_ffmpeg_program(app)
}
