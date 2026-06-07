//! FR-AI-017 §3 — Redis backend for per-tenant cache.

use std::future::Future;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use redis::AsyncCommands;
use tokio::time::timeout;

use super::{CacheError, CacheInsertOutcome, CacheLookupOutcome, CachedResponse, REDIS_TIMEOUT_MS};

static REDIS_URL: OnceLock<String> = OnceLock::new();
static REDIS_MANAGER: Mutex<Option<redis::aio::ConnectionManager>> = Mutex::new(None);

/// Initialize the Redis connection URL. Call once at startup.
pub fn init(url: &str) {
    REDIS_URL.set(url.to_string()).ok();
    if let Ok(mut manager) = REDIS_MANAGER.lock() {
        *manager = None;
    }
}

async fn get_conn() -> Result<redis::aio::ConnectionManager, redis::RedisError> {
    if let Some(manager) = REDIS_MANAGER
        .lock()
        .expect("redis manager mutex should not be poisoned")
        .as_ref()
        .cloned()
    {
        return Ok(manager);
    }

    let url = REDIS_URL
        .get()
        .cloned()
        .or_else(|| std::env::var("REDIS_URL").ok())
        .unwrap_or_else(|| "redis://127.0.0.1:6379".to_string());
    let client = redis::Client::open(url.as_str())?;
    let manager = redis::aio::ConnectionManager::new(client).await?;

    let mut guard = REDIS_MANAGER
        .lock()
        .expect("redis manager mutex should not be poisoned");
    if guard.is_none() {
        *guard = Some(manager.clone());
        return Ok(manager);
    }
    Ok(guard.as_ref().expect("manager was just checked").clone())
}

async fn command_with_retry<T, F, Fut>(mut command: F) -> Result<T, redis::RedisError>
where
    F: FnMut(redis::aio::ConnectionManager) -> Fut,
    Fut: Future<Output = Result<T, redis::RedisError>>,
{
    let conn = get_conn().await?;
    match command(conn).await {
        Ok(value) => Ok(value),
        Err(_) => {
            if let Ok(mut manager) = REDIS_MANAGER.lock() {
                *manager = None;
            }
            let conn = get_conn().await?;
            command(conn).await
        }
    }
}

/// §1 #1: lookup with timeout, schema-version check, and metrics.
pub async fn lookup(key: &super::CacheKey) -> CacheLookupOutcome {
    let t0 = std::time::Instant::now();
    let redis_key = key.redis_key();
    let raw: Result<Option<Vec<u8>>, _> = match timeout(
        Duration::from_millis(REDIS_TIMEOUT_MS),
        command_with_retry(|mut conn| {
            let redis_key = redis_key.clone();
            async move { conn.get(redis_key).await }
        }),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => {
            super::error_metric("timeout");
            return CacheLookupOutcome::Error(CacheError::Timeout);
        }
    };
    let bytes = match raw {
        Ok(Some(b)) => b,
        Ok(None) => {
            super::lookup_metric(&key.tenant_id, "miss", t0.elapsed());
            return CacheLookupOutcome::Miss;
        }
        Err(e) => {
            super::error_metric("unreachable");
            return CacheLookupOutcome::Error(CacheError::Unreachable(e.to_string()));
        }
    };

    // §1 #13: check schema_version before full deserialisation so stale
    // payloads with old enum shapes degrade as misses, not backend errors.
    let value = match serde_json::from_slice::<serde_json::Value>(&bytes) {
        Ok(value) => value,
        Err(e) => {
            super::error_metric("deserialisation");
            return CacheLookupOutcome::Error(CacheError::Deserialisation(e.to_string()));
        }
    };
    if value.get("schema_version").and_then(|v| v.as_str()) != Some(super::CACHE_SCHEMA_VERSION) {
        super::lookup_metric(&key.tenant_id, "schema_mismatch", t0.elapsed());
        return CacheLookupOutcome::SchemaMismatch;
    }

    match serde_json::from_value::<CachedResponse>(value) {
        Ok(cr) => {
            super::lookup_metric(&key.tenant_id, "hit", t0.elapsed());
            CacheLookupOutcome::Hit(Box::new(cr), t0.elapsed())
        }
        Err(e) => {
            super::error_metric("deserialisation");
            CacheLookupOutcome::Error(CacheError::Deserialisation(e.to_string()))
        }
    }
}

/// §1 #4 + §1 #5: insert with jittered TTL.
pub async fn insert(
    key: &super::CacheKey,
    bytes: &[u8],
    jittered_ttl: Duration,
) -> CacheInsertOutcome {
    let redis_key = key.redis_key();
    let payload = bytes.to_vec();
    let ttl_secs = jittered_ttl.as_secs();
    let result: Result<(), _> = match timeout(
        Duration::from_millis(REDIS_TIMEOUT_MS),
        command_with_retry(|mut conn| {
            let redis_key = redis_key.clone();
            let payload = payload.clone();
            async move { conn.set_ex(redis_key, payload, ttl_secs).await }
        }),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => {
            super::error_metric("timeout");
            return CacheInsertOutcome::Error(CacheError::Timeout);
        }
    };
    if let Err(e) = result {
        super::error_metric("unreachable");
        return CacheInsertOutcome::Error(CacheError::Unreachable(e.to_string()));
    }

    CacheInsertOutcome::Inserted {
        ttl: jittered_ttl,
        jittered_ttl,
    }
}
