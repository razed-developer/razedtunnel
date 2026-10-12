//! Shared FFmpeg/FFprobe discovery for media inspection and phone workflows.
//! Provisioning remains delegated to legacy video code until safely extracted.

use std::{env, fs, io::{Cursor, Read}, path::{Path, PathBuf}, process::{Command, Stdio}, sync::atomic::{AtomicBool, Ordering}, time::Duration};
use zip::ZipArchive;
use reqwest::blocking::Client;
use url::Url;
use sha2::{Digest, Sha256};
use tauri::{path::BaseDirectory, AppHandle, Manager};

pub(crate) fn wheel_matches_platform(filename: &str) -> bool {
    let lower = filename.to_ascii_lowercase();
    if !lower.ends_with(".whl") {
        return false;
    }
    match (env::consts::OS, env::consts::ARCH) {
        ("linux", "x86_64") => lower.contains("manylinux") && lower.contains("x86_64"),
        ("linux", "aarch64") => {
            lower.contains("manylinux") && (lower.contains("aarch64") || lower.contains("arm64"))
        }
        ("windows", "x86_64") => lower.contains("win_amd64"),
        ("windows", "aarch64") => lower.contains("win_arm64"),
        ("macos", "x86_64") => lower.contains("macosx") && lower.contains("x86_64"),
        ("macos", "aarch64") => lower.contains("macosx") && lower.contains("arm64"),
        _ => false,
    }
}

pub(crate) fn trusted_helper_host(host: &str) -> bool {
    matches!(
        host,
        "github.com"
            | "release-assets.githubusercontent.com"
            | "objects.githubusercontent.com"
            | "pypi.org"
            | "files.pythonhosted.org"
    )
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(crate) fn download_client() -> Result<Client, String> {
    Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(180))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.url().host_str().is_some_and(crate::media_tools::trusted_helper_host) {
                attempt.follow()
            } else {
                attempt.stop()
            }
        }))
        .user_agent("RepoTunnel/0.3.1 Video Intelligence")
        .build()
        .map_err(|error| format!("Could not initialize the video helper downloader: {error}"))
}

pub(crate) fn download_bytes(
    client: &Client,
    url: &str,
    limit: u64,
    cancel: Option<&AtomicBool>,
) -> Result<Vec<u8>, String> {
    let parsed =
        Url::parse(url).map_err(|_| "Video helper download URL is invalid.".to_string())?;
    if parsed.scheme() != "https" {
        return Err("Video helpers may only be downloaded over HTTPS.".to_string());
    }
    let host = parsed.host_str().unwrap_or_default();
    if !crate::media_tools::trusted_helper_host(host) {
        return Err("Video helper download host is not allowlisted.".to_string());
    }

    let mut response = client
        .get(url)
        .send()
        .map_err(|error| format!("Could not download a video helper: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Video helper download failed with HTTP {}.",
            response.status()
        ));
    }
    if response.content_length().is_some_and(|size| size > limit) {
        return Err("Video helper download exceeds RepoTunnel's safety limit.".to_string());
    }

    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        if cancel.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
            return Err("Video analysis cancelled.".to_string());
        }
        let read = response
            .read(&mut buffer)
            .map_err(|error| format!("Could not read a video helper download: {error}"))?;
        if read == 0 {
            break;
        }
        if bytes.len().saturating_add(read) > usize::try_from(limit).unwrap_or(usize::MAX) {
            return Err("Video helper download exceeds RepoTunnel's safety limit.".to_string());
        }
        bytes.extend_from_slice(&buffer[..read]);
    }
    Ok(bytes)
}

pub(crate) fn extract_ffmpeg_wheel_binary(wheel: Vec<u8>, filename: &str) -> Result<Vec<u8>, String> {
    let mut archive = ZipArchive::new(Cursor::new(wheel))
        .map_err(|error| format!("Could not open the verified FFmpeg package: {error}"))?;
    let mut binary = None;
    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|error| format!("Could not inspect the FFmpeg package: {error}"))?;
        let name = file.name().replace('\\', "/");
        let lower = name.to_ascii_lowercase();
        let looks_like_binary = lower.contains("/binaries/ffmpeg-")
            && !lower.ends_with(".txt")
            && !lower.ends_with(".md");
        if looks_like_binary {
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).map_err(|error| {
                format!("Could not extract FFmpeg from its verified package: {error}")
            })?;
            if !bytes.is_empty() {
                binary = Some(bytes);
                break;
            }
        }
    }
    let bytes = binary.ok_or_else(|| {
        format!(
            "The verified FFmpeg wheel {} did not contain its expected executable.",
            filename
        )
    })?;
    Ok(bytes)
}

pub(crate) fn program_version(path: &Path, name: &str) -> Option<String> {
    let arg = if name == "ffmpeg" {
        "-version"
    } else {
        "--version"
    };
    let output = Command::new(path)
        .arg(arg)
        .stdin(Stdio::null())
        .output()
        .ok()?;
    let text = if output.stdout.is_empty() {
        String::from_utf8_lossy(&output.stderr)
    } else {
        String::from_utf8_lossy(&output.stdout)
    };
    text.lines().next().map(|line| line.trim().to_string())
}

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
