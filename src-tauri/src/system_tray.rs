use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

pub fn install(app: &mut tauri::App) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open RazedTunnel", true, None::<&str>)?;
    let projects = MenuItem::with_id(app, "projects", "Projects", true, None::<&str>)?;
    let connections = MenuItem::with_id(app, "connections", "Connections", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit RazedTunnel", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &projects, &connections, &quit])?;

    let mut tray = TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => show_main_window(app),
            "projects" => open_page(app, "projects"),
            "connections" => open_page(app, "connections"),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::DoubleClick { .. } = event {
                show_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn open_page(app: &tauri::AppHandle, page: &str) {
    show_main_window(app);
    let _ = app.emit("razedtunnel:open-page", page);
}
