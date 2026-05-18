// Hide the Windows console in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod sync_supervisor;
mod tray;

use tauri::{Manager, WindowEvent};
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Spawn the brain-sync supervisor on a background tokio task.
            // The supervisor owns its own restart-with-circuit-breaker policy.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                sync_supervisor::start(handle).await;
            });

            // Build the system tray (Open / Force Sync / Quit).
            tray::build_tray(&app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search_brain,
            commands::write_quick_note,
            commands::get_sync_state,
        ])
        .on_window_event(|window, event| {
            // Minimise-to-tray: intercept the close button on the main window.
            // (Real "quit" path goes through the tray menu's Quit item.)
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running CyberOS BRAIN app");
}
