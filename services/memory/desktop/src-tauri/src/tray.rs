//! System-tray with quick actions (Open / Force Sync / Quit).
//!
//! First-slice tray: just three items wired to log lines and a window-show.
//! "Force Sync" emits an event the supervisor will eventually listen for;
//! today it only logs. "Quick Capture" and "Recent Memories" land in slice 2+.

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

pub fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let open_item = MenuItem::with_id(app, "open", "Open memory", true, None::<&str>)?;
    let force_sync_item = MenuItem::with_id(app, "force_sync", "Force Sync Now", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&open_item, &force_sync_item, &quit_item])?;

    let _tray = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("CyberOS memory")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "force_sync" => {
                tracing::info!("tray: force_sync requested (no-op in first slice)");
                // TODO: emit event consumed by sync_supervisor::start.
                let _ = app.emit("memory://force-sync", ());
            }
            "quit" => {
                tracing::info!("tray: quit");
                app.exit(0);
            }
            other => {
                tracing::warn!(id = other, "tray: unknown menu id");
            }
        })
        .build(app)?;

    Ok(())
}
