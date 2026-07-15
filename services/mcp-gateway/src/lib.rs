//! cyberos-mcp-gateway — Model Context Protocol 2025-11-25 federation door.
//!
//! Implements [TASK-MCP-001..008](../../docs/tasks/mcp/). External MCP clients
//! (Claude Desktop, IDE plugins, third-party agents) connect here; the gateway holds the
//! federated tool catalog and dispatches `tools/call` to the owning module server.
//!
//! ## Module map
//!
//! - [`protocol`] — JSON-RPC 2.0 request/response/error/batch types · `initialize` ·
//!   `tools/list` · `tools/call` · closed error-code map.
//! - [`federation`] — in-memory tool registry; module servers register via TASK-MCP-002.
//! - [`annotations`] — `ToolAnnotations` struct (`destructiveHint`/`readOnlyHint`/
//!   `idempotentHint`/`openWorldHint`).
//! - [`router`] — Axum router mounting `POST /mcp` + `GET /mcp/healthz`.
//!
//! ## Status (2026-05-19 slice 1)
//!
//! - JSON-RPC parser + closed error-code map: **shipped** (full unit tests).
//! - `initialize` capability advertisement: **shipped** (returns `protocolVersion`,
//!   capabilities, `serverInfo`, `instructions`).
//! - `tools/list` from in-memory registry: **shipped** (stub registry; TASK-MCP-002 lands
//!   the live registration handler).
//! - `tools/call` dispatch: **scaffolded** (returns `-32004 module_unreachable` until
//!   TASK-MCP-002 wires the registry to live module endpoints).
//! - Bearer-token + scope check: **scaffolded** (TASK-MCP-004 PKCE flow lands the full
//!   token issuance + audience-bound verification; this scaffold accepts any valid
//!   JWT per TASK-AUTH-004 + asserts the `mcp:tools` scope).
//! - Rate-limit + audit emission: **deferred** to follow-on tasks.

#![deny(missing_debug_implementations)]
#![warn(missing_docs)]

pub mod annotations;
pub mod elicitation;
pub mod elicitation_pg;
pub mod federation;
pub mod gating;
pub mod kms;
pub mod naming;
pub mod oauth;
pub mod protocol;
pub mod router;
pub mod tasks;
pub mod tasks_pg;

#[cfg(test)]
mod db_slice_test;

/// Banner emitted by the binary on startup.
pub const SERVICE_BANNER: &str = concat!(
    "cyberos-mcp-gateway v",
    env!("CARGO_PKG_VERSION"),
    " — Model Context Protocol 2025-11-25 federation door (TASK-MCP-001..008)"
);

/// Pinned MCP protocol version. See DEC-260.
pub const MCP_PROTOCOL_VERSION: &str = "2025-11-25";
