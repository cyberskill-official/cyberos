//! TASK-AI-017 §3 — Redis backend for per-tenant cache.

use std::sync::OnceLock;
use std::time::Duration;

use redis::AsyncCommands;
use tokio::sync::Mutex;
use tokio::time::timeout;

use super::{CacheError, CacheInsertOutcome, CacheLookupOutcome, CachedResponse, REDIS_TIMEOUT_MS};

static REDIS_URL: OnceLock<String> = OnceLock::new();

/// Initialize the Redis connection URL. Call once at startup.
pub fn init(url: &str) {
    REDIS_URL.set(url.to_string()).ok();
}

/// Process-wide multiplexed connection.
///
/// `ConnectionManager` multiplexes concurrent commands over a single auto-reconnecting socket and
/// is cheap to clone (Arc-backed), so thousands of concurrent operations share one connection
/// instead of each opening its own. Building a fresh `ConnectionManager` per call created a
/// connection-per-operation storm that exhausted Redis and timed out under load (TASK-AI-018 §1 #8).
///
/// It is guarded by a `Mutex<Option<..>>` rather than a `OnceCell` so a dead connection can be
/// rebuilt. A `ConnectionManager`'s background driver is bound to the Tokio runtime that created
/// it; once that runtime is gone - a real Redis restart, or one `#[tokio::test]` finishing before
/// the next starts its own runtime - the socket yields "broken pipe". On an operation error we
/// drop the cached connection and rebuild it once (see `fetch_bytes` / `store_bytes`).
static CONN: Mutex<Option<redis::aio::ConnectionManager>> = Mutex::const_new(None);

async fn build_conn() -> Result<redis::aio::ConnectionManager, redis::RedisError> {
    let url = REDIS_URL.get().ok_or_else(|| {
        redis::RedisError::from((
            redis::ErrorKind::InvalidClientConfig,
            "redis URL not initialized",
        ))
    })?;
    let client = redis::Client::open(url.as_str())?;
    redis::aio::ConnectionManager::new(client).await
}

/// Return a live multiplexed connection, building it once and caching the clone.
async fn get_conn() -> Result<redis::aio::ConnectionManager, redis::RedisError> {
    let mut guard = CONN.lock().await;
    if let Some(c) = guard.as_ref() {
        return Ok(c.clone());
    }
    let mgr = build_conn().await?;
    *guard = Some(mgr.clone());
    Ok(mgr)
}

/// Drop the cached connection so the next `get_conn` rebuilds it.
async fn invalidate_conn() {
    *CONN.lock().await = None;
}

/// GET with one transparent rebuild-and-retry if the cached connection is dead.
async fn fetch_bytes(redis_key: &str) -> Result<Option<Vec<u8>>, redis::RedisError> {
    let mut conn = get_conn().await?;
    let first: Result<Option<Vec<u8>>, redis::RedisError> = conn.get(redis_key).await;
    match first {
        Ok(v) => Ok(v),
        Err(_) => {
            invalidate_conn().await;
            let mut conn = get_conn().await?;
            conn.get(redis_key).await
        }
    }
}

/// SET (with TTL) with one transparent rebuild-and-retry if the cached connection is dead.
async fn store_bytes(
    redis_key: &str,
    bytes: &[u8],
    ttl_secs: u64,
) -> Result<(), redis::RedisError> {
    let mut conn = get_conn().await?;
    let first: Result<(), redis::RedisError> = conn.set_ex(redis_key, bytes, ttl_secs).await;
    match first {
        Ok(()) => Ok(()),
        Err(_) => {
            invalidate_conn().await;
            let mut conn = get_conn().await?;
            conn.set_ex(redis_key, bytes, ttl_secs).await
        }
    }
}

/// §1 #1: lookup with timeout, schema-version check, and metrics.
pub async fn lookup(key: &super::CacheKey) -> CacheLookupOutcome {
    let t0 = std::time::Instant::now();
    let redis_key = key.redis_key();

    let raw = match timeout(
        Duration::from_millis(REDIS_TIMEOUT_MS),
        fetch_bytes(&redis_key),
    )
    .await
    {
        Ok(Ok(v)) => v,
        Ok(Err(e)) => {
            super::error_metric("unreachable");
            return CacheLookupOutcome::Error(CacheError::Unreachable(e.to_string()));
        }
        Err(_) => {
            super::error_metric("timeout");
            return CacheLookupOutcome::Error(CacheError::Timeout);
        }
    };

    let bytes = match raw {
        Some(b) => b,
        None => {
            super::lookup_metric(&key.tenant_id, "miss", t0.elapsed());
            return CacheLookupOutcome::Miss;
        }
    };

    // §1 #13: schema-version check on load. Peek at the version BEFORE full deserialization:
    // an entry written under an older schema may not deserialize into the current CachedResponse
    // at all (e.g. a renamed enum variant), and such a stale entry must be treated as a
    // SchemaMismatch (a miss), not a hard deserialization error.
    #[derive(serde::Deserialize)]
    struct VersionPeek {
        schema_version: String,
    }
    if let Ok(peek) = serde_json::from_slice::<VersionPeek>(&bytes) {
        if peek.schema_version != super::CACHE_SCHEMA_VERSION {
            super::lookup_metric(&key.tenant_id, "schema_mismatch", t0.elapsed());
            return CacheLookupOutcome::SchemaMismatch;
        }
    }
    match serde_json::from_slice::<CachedResponse>(&bytes) {
        Ok(cr) if cr.schema_version == super::CACHE_SCHEMA_VERSION => {
            super::lookup_metric(&key.tenant_id, "hit", t0.elapsed());
            CacheLookupOutcome::Hit(Box::new(cr), t0.elapsed())
        }
        Ok(_) => {
            super::lookup_metric(&key.tenant_id, "schema_mismatch", t0.elapsed());
            CacheLookupOutcome::SchemaMismatch
        }
        Err(e) => {
            super::error_metric("deserialisation");
            CacheLookupOutcome::Error(CacheError::Deserialisation(e.to_string()))
        }
    }
}

/// §1 #4 + §1 #5: insert with jittered TTL.
///
/// A failed write is surfaced as `CacheInsertOutcome::Error` rather than silently reported as
/// `Inserted`: a dropped write means the entry is NOT cached, and callers (and the isolation
/// property test, which counts keys) must be able to tell.
pub async fn insert(
    key: &super::CacheKey,
    bytes: &[u8],
    jittered_ttl: Duration,
) -> CacheInsertOutcome {
    let redis_key = key.redis_key();
    match timeout(
        Duration::from_millis(REDIS_TIMEOUT_MS),
        store_bytes(&redis_key, bytes, jittered_ttl.as_secs()),
    )
    .await
    {
        Ok(Ok(())) => CacheInsertOutcome::Inserted {
            ttl: jittered_ttl,
            jittered_ttl,
        },
        Ok(Err(e)) => {
            super::error_metric("unreachable");
            CacheInsertOutcome::Error(CacheError::Unreachable(e.to_string()))
        }
        Err(_) => {
            super::error_metric("timeout");
            CacheInsertOutcome::Error(CacheError::Timeout)
        }
    }
}
