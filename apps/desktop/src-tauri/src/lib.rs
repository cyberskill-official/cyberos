//! CyberOS desktop - a Tauri app that triggers CyberOS workflows and skills from the desktop by driving
//! the existing gateway HTTP surface. The Rust backend makes every network and keychain call; the webview
//! invokes these commands, so there is no browser CORS and the token never enters the webview.
//!
//! This first slice covers the chat-trigger path (the proven gateway `/v1/chat`) plus keychain-backed
//! token storage and a health check. The workflow/skill picker driven by the mcp-gateway `tools/list`
//! surface is the next iteration (see TASK-APP-002 clause 5); the structure here is ready for it.

mod gateway_client;
mod keychain;
mod mcp_client;

use gateway_client::{ChatTurn, GatewayClient};
use mcp_client::{McpClient, ToolInfo};

/// GET /healthz against the configured gateway.
#[tauri::command]
async fn health(gateway: String) -> bool {
    GatewayClient::new(gateway).health().await
}

/// Send a chat turn list to the gateway, attaching the keychain token when present.
#[tauri::command]
async fn chat(
    gateway: String,
    tenant: String,
    alias: String,
    messages: Vec<ChatTurn>,
) -> Result<serde_json::Value, String> {
    let token = keychain::get_token().ok();
    GatewayClient::new(gateway)
        .chat(&tenant, &alias, &messages, token.as_deref())
        .await
}

#[tauri::command]
fn save_token(token: String) -> Result<(), String> {
    keychain::set_token(&token).map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_token() -> Result<(), String> {
    keychain::clear_token().map_err(|e| e.to_string())
}

#[tauri::command]
fn has_token() -> bool {
    keychain::get_token().is_ok()
}

/// GET /mcp/healthz against the configured mcp-gateway.
#[tauri::command]
async fn mcp_health(mcp: String) -> bool {
    McpClient::new(mcp).health().await
}

/// List the workflows and skills the mcp-gateway exposes (tools/list, paginated).
#[tauri::command]
async fn list_tools(mcp: String) -> Result<Vec<ToolInfo>, String> {
    McpClient::new(mcp).list_tools().await
}

/// Trigger a tool by name (tools/call). The gateway forwards the call to the owning module's
/// registered MCP endpoint (TASK-MCP-002) and returns its result, or module_unreachable if that
/// endpoint is down.
#[tauri::command]
async fn call_tool(mcp: String, name: String, arguments: serde_json::Value) -> Result<serde_json::Value, String> {
    McpClient::new(mcp).call_tool(&name, arguments).await
}

/// Desktop-only: on launch, check for a newer signed build and, if one exists, download + install it, then
/// restart into it. Best-effort by design - if the updater has no `plugins.updater` config yet (no pubkey /
/// endpoint), or the check fails, or we are offline, it logs and does nothing. The desktop shell just loads
/// the live /web/, so its content already updates on its own; this keeps the installed binary current too.
///
/// TASK-IMP-075: compiled OUT of the Mac App Store target (`--features mas`) - a sandboxed MAS
/// bundle must not self-update (App Sandbox violation + App Store policy). Default builds keep
/// the updater exactly as before.
#[cfg(all(desktop, not(feature = "mas")))]
fn spawn_update_check(app: tauri::AppHandle) {
    use tauri_plugin_updater::UpdaterExt;
    tauri::async_runtime::spawn(async move {
        let updater = match app.updater() {
            Ok(u) => u,
            Err(e) => {
                eprintln!("cyberos updater: not configured, skipping ({e})");
                return;
            }
        };
        match updater.check().await {
            Ok(Some(update)) => match update.download_and_install(|_downloaded, _total| {}, || {}).await {
                Ok(()) => app.restart(),
                Err(e) => eprintln!("cyberos updater: install failed ({e})"),
            },
            Ok(None) => {}
            Err(e) => eprintln!("cyberos updater: check skipped ({e})"),
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();
    #[cfg(all(desktop, not(feature = "mas")))]
    {
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }
    builder
        .setup(|app| {
            #[cfg(all(desktop, not(feature = "mas")))]
            spawn_update_check(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            health, chat, save_token, clear_token, has_token, mcp_health, list_tools, call_tool
        ])
        .run(tauri::generate_context!())
        .expect("error while running the CyberOS desktop app");
}
