fn main() {
    // Tauri owns the Windows application manifest. Do not inject a second
    // manifest through global rustc-link-arg settings: those flags also reach
    // the Tauri binary test target and cause CVT1100 duplicate MANIFEST #1.
    tauri_build::build();
}
