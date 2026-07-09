//! Tauri commands exposed to the Svelte frontend via `invoke()`.
//!
//! These are first-slice stubs:
//!   - `search_memory`    — POSTs to the local Rust memory service (FR-MEMORY-108),
//!                         which is assumed to be listening on 127.0.0.1:7901.
//!   - `write_quick_note` — writes a markdown file under
//!                         `~/.cyberos/memory/store/{tenant}/captures/` with frontmatter.
//!                         Today the tenant resolves to "default"; FR-MEMORY-105+ wires
//!                         real multi-tenant resolution.
//!   - `get_sync_state`   — reads `~/.cyberos/memory/store/{tenant}/sync/last-status.json`
//!                         (matches `modules/memory/cyberos/core/memory_sync.py::LAST_STATUS_REL`).

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const MEMORY_SEARCH_URL: &str = "http://127.0.0.1:7901/v1/memory/search";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchHit {
    pub id: String,
    pub kind: String,
    pub ts_ns: i64,
    pub preview: String,
    pub score: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncState {
    pub chain_head: Option<String>,
    pub last_sync_at: Option<String>,
    pub last_sync_duration_ms: Option<u64>,
    pub cloud_state: String,
}

// ────────────────────────────────────────────────────────────────────────────
// search_memory
// ────────────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct SearchRequest<'a> {
    query: &'a str,
    limit: u32,
}

#[tauri::command]
pub async fn search_memory(query: String, limit: u32) -> Result<Vec<SearchHit>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| format!("reqwest client: {e}"))?;

    let resp = client
        .post(MEMORY_SEARCH_URL)
        .json(&SearchRequest { query: &query, limit })
        .send()
        .await
        .map_err(|e| format!("memory search request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("memory search HTTP {}", resp.status()));
    }

    resp.json::<Vec<SearchHit>>()
        .await
        .map_err(|e| format!("memory search decode failed: {e}"))
}

// ────────────────────────────────────────────────────────────────────────────
// write_quick_note
// ────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn write_quick_note(text: String, tags: Vec<String>) -> Result<String, String> {
    let dir = memory_dir().join("captures");
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| format!("create captures dir: {e}"))?;

    let ts_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as i64)
        .unwrap_or(0);

    let ts_iso = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%S").to_string();
    let file = dir.join(format!("quick-note-{ts_iso}.md"));

    let yaml_tags = tags
        .iter()
        .map(|t| format!("  - {}", yaml_escape(t)))
        .collect::<Vec<_>>()
        .join("\n");

    let body = format!(
        "---\nkind: quick_note\nts_ns: {ts_ns}\nsource: tray\nsync_class: shareable\ntags:\n{yaml_tags}\n---\n\n{text}\n",
        ts_ns = ts_ns,
        yaml_tags = if yaml_tags.is_empty() { "  []".into() } else { yaml_tags },
        text = text,
    );

    tokio::fs::write(&file, body)
        .await
        .map_err(|e| format!("write quick note: {e}"))?;

    Ok(file.to_string_lossy().into_owned())
}

fn yaml_escape(s: &str) -> String {
    // Minimal YAML-flow escaping for tag values. Full escaping should land in a
    // helper crate; first-slice keeps it simple.
    if s.chars().any(|c| matches!(c, ':' | '#' | '\'' | '"' | '\n')) {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// get_sync_state
// ────────────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_sync_state() -> Result<SyncState, String> {
    let path = memory_dir().join("sync").join("last-status.json");

    let raw = match tokio::fs::read(&path).await {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(SyncState {
                chain_head: None,
                last_sync_at: None,
                last_sync_duration_ms: None,
                cloud_state: "offline".into(),
            });
        }
        Err(e) => return Err(format!("read last-status.json: {e}")),
    };

    let v: serde_json::Value = serde_json::from_slice(&raw)
        .map_err(|e| format!("parse last-status.json: {e}"))?;

    Ok(SyncState {
        chain_head: v.get("chain_head").and_then(|x| x.as_str()).map(str::to_string),
        last_sync_at: v.get("last_sync_at").and_then(|x| x.as_str()).map(str::to_string),
        last_sync_duration_ms: v.get("last_sync_duration_ms").and_then(|x| x.as_u64()),
        cloud_state: v.get("cloud_state").and_then(|x| x.as_str()).unwrap_or("unknown").into(),
    })
}

// ────────────────────────────────────────────────────────────────────────────
// helpers
// ────────────────────────────────────────────────────────────────────────────

/// `~/.cyberos/memory/store/{tenant}/`. For the first slice, tenant is hard-coded to
/// `default`; multi-tenant resolution arrives with FR-MEMORY-105+.
fn memory_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".cyberos/memory/store").join("default")
}
