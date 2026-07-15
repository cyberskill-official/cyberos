//! TASK-AI-018 §1 #11 — Per-case Redis isolation via UUID prefix + Drop cleanup.

use uuid::Uuid;

pub struct RedisTestNamespace {
    pub prefix: String,
    conn_str: String,
}

impl RedisTestNamespace {
    pub fn new() -> Self {
        let prefix = format!("test_{}_", Uuid::new_v4().simple());
        let conn_str =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        Self { prefix, conn_str }
    }

    /// Wraps a tenant_id with the per-case prefix.
    pub fn tenant(&self, original: &str) -> String {
        format!("{}{}", self.prefix, original)
    }

    /// Delete all keys matching the given pattern.
    pub fn delete_keys_matching(&self, pattern: &str) {
        let client = match redis::Client::open(self.conn_str.as_str()) {
            Ok(c) => c,
            Err(_) => return,
        };
        let mut conn = match client.get_connection() {
            Ok(c) => c,
            Err(_) => return,
        };

        let mut cursor: u64 = 0;
        loop {
            let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(pattern)
                .arg("COUNT")
                .arg(100)
                .query(&mut conn)
                .unwrap_or_default();

            if !keys.is_empty() {
                for key in &keys {
                    let _: Result<(), _> = redis::cmd("DEL").arg(key).query(&mut conn);
                }
            }

            cursor = next_cursor;
            if cursor == 0 {
                break;
            }
        }
    }
}

impl Drop for RedisTestNamespace {
    fn drop(&mut self) {
        let pattern = format!("ai_cache:v1:{}*", self.prefix);
        self.delete_keys_matching(&pattern);
    }
}

/// Whether a Redis server is reachable at `REDIS_URL` (default `redis://127.0.0.1:6379`).
///
/// Probed once and cached for the lifetime of the test binary. Redis-backed isolation tests call
/// this first and skip (return early) when it is false, so the no-Redis `lint + test` job stays
/// green and fast while the integration job and the awh gate - which both provide Redis - still
/// execute them. This mirrors how the cost tests skip when `DATABASE_URL` is unset: an absent
/// backend is a "cannot run here", not a failure. Without the guard, these tests either panic on
/// connect (`unwrap`), misread a connect timeout as a cross-tenant leak, or hang retrying every op.
#[allow(dead_code)]
pub fn redis_available() -> bool {
    use std::sync::OnceLock;
    static AVAILABLE: OnceLock<bool> = OnceLock::new();
    *AVAILABLE.get_or_init(|| {
        let url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        redis::Client::open(url.as_str())
            .and_then(|client| client.get_connection())
            .is_ok()
    })
}
