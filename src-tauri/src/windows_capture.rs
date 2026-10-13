//! Window-only Windows capture. Never falls back to desktop/screen BitBlt.
#![cfg(windows)]

use base64::{engine::general_purpose::STANDARD, Engine as _};
use windows_sys::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Gdi::{
        CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits,
        SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    },
    UI::WindowsAndMessaging::{GetDC, GetWindowRect, GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible, PrintWindow, ReleaseDC},
};

use crate::{desktop_control::DesktopScreenshot, windows_uia};

const MAX_DIMENSION: i32 = 4096;
const MAX_PIXELS: i64 = 8_388_608;
const MAX_PNG_BYTES: usize = 16 * 1024 * 1024;

pub(crate) fn capture(application_id: &str, window_id: Option<&str>) -> Result<DesktopScreenshot, String> {
    let (handle, resolved_id) = windows_uia::resolve_capture_window(application_id, window_id)?;
    let pid: u32 = application_id.strip_prefix("uia-pid-")
        .and_then(|v| v.parse().ok()).filter(|v| *v > 0)
        .ok_or_else(|| "Invalid approved Windows application identity.".to_string())?;
    let hwnd = handle as HWND;
    unsafe {
        let mut actual_pid = 0u32;
        if IsWindow(hwnd) == 0 || IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 {
            return Err("Approved window is hidden, minimized, or no longer present.".into());
        }
        GetWindowThreadProcessId(hwnd, &mut actual_pid);
        if actual_pid != pid || actual_pid == std::process::id() {
            return Err("Window no longer belongs to the approved application.".into());
        }
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect) == 0 {
            return Err("Could not read approved window dimensions.".into());
        }
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        if width <= 0 || height <= 0 || width > MAX_DIMENSION || height > MAX_DIMENSION ||
            i64::from(width) * i64::from(height) > MAX_PIXELS {
            return Err("Window dimensions exceed capture limits.".into());
        }
        let window_dc = GetDC(hwnd);
        if window_dc.is_null() { return Err("Could not open window device context.".into()); }
        let dc = CreateCompatibleDC(window_dc);
        if dc.is_null() { ReleaseDC(hwnd, window_dc); return Err("Could not create capture device context.".into()); }
        let bitmap = CreateCompatibleBitmap(window_dc, width, height);
        if bitmap.is_null() { DeleteDC(dc); ReleaseDC(hwnd, window_dc); return Err("Could not allocate window bitmap.".into()); }
        let previous = SelectObject(dc, bitmap as _);
        if previous.is_null() {
            DeleteObject(bitmap as _); DeleteDC(dc); ReleaseDC(hwnd, window_dc);
            return Err("Could not select window bitmap.".into());
        }
        // PrintWindow requests the approved window to draw into our private bitmap.
        // Do not substitute screen capture: that could expose unrelated windows.
        let printed = PrintWindow(hwnd, dc, 2);
        let mut pixels = vec![0u8; width as usize * height as usize * 4];
        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width, biHeight: -height, biPlanes: 1, biBitCount: 32,
                biCompression: BI_RGB, ..std::mem::zeroed()
            },
            ..std::mem::zeroed()
        };
        SelectObject(dc, previous);
        let rows = if printed != 0 {
            GetDIBits(dc, bitmap, 0, height as u32, pixels.as_mut_ptr() as _, &mut info, DIB_RGB_COLORS)
        } else { 0 };
        DeleteObject(bitmap as _);
        DeleteDC(dc);
        ReleaseDC(hwnd, window_dc);
        if printed == 0 || rows != height as i32 {
            return Err("Window declined or failed isolated capture; no desktop fallback is permitted.".into());
        }
        // BI_RGB 32-bit DIB uses BGRA; PNG expects RGBA.
        for pixel in pixels.chunks_exact_mut(4) {
            pixel.swap(0, 2);
            pixel[3] = 255;
        }
        let mut png_bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png_bytes, width as u32, height as u32);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().map_err(|e| format!("PNG header: {e}"))?;
            writer.write_image_data(&pixels).map_err(|e| format!("PNG encoding: {e}"))?;
        }
        if png_bytes.len() > MAX_PNG_BYTES { return Err("Captured PNG exceeds size limit.".into()); }
        Ok(DesktopScreenshot {
            application_id: application_id.to_owned(),
            window_id: resolved_id,
            mime_type: "image/png".into(),
            size_bytes: png_bytes.len() as u64,
            width: width as u32,
            height: height as u32,
            data_base64: STANDARD.encode(png_bytes),
        })
    }
}
