//! User blocking (TASK-CHAT-268). "A no longer wishes to receive B's content."
//!
//! The one thing to understand before changing anything here: **B is never told.** Not by a status code, not
//! by an error string, not by a missing read receipt. B posts, the message is persisted, B sees it in their
//! own client — and it simply never arrives. Refusing B's message with a 403 would be honest and simple and
//! dangerous: it converts someone who was being ignored into someone who knows they were rejected, and the
//! documented pattern is that they escalate through another channel. Signal, WhatsApp and iMessage all let
//! the blocked sender believe the message went out. So do we (§1 #7, #8; rationale in the FR's §2).
//!
//! Enforcement is server-side at FOUR fan-out points (§1 #4), because there are four ways content reaches a
//! person and filtering three of them is filtering none:
//!
//! 1. the message list — `messages::list`
//! 2. the realtime socket — `realtime::ws_loop`
//! 3. notification + push — `notify::fanout`
//! 4. the DM list — `channels::list`
//!
//! Point 3 is the one that was silently broken before this FR: the fan-out selected channel members and did
//! not know blocks existed, so a blocked person's name and the first line of their message would still have
//! landed on the blocker's lock screen.
//!
//! A block implemented in the client is not a block; it is a CSS rule anyone can defeat with the network tab.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{audit, auth, db, AppState};

/// Blocker -> the set of subjects they have blocked.
///
/// Why a cache at all: three of the four enforcement points sit on the hot path, and the realtime one is the
/// worst — a 200-member channel means 200 open sockets, and re-querying per socket per frame would be an
/// N*M of the busiest thing in the service. The set is small (a handful of uuids), so it is read once and
/// held behind an `Arc`.
///
/// Correctness: the cache is invalidated *synchronously* by `block` / `unblock` in this process, so the
/// blocker's own open tabs stop receiving immediately (§10 row 5 — an open socket that keeps delivering
/// after the block lands is a live, observable failure of §1 #4).
///
/// KNOWN LIMIT — multi-replica. The chat service is horizontally scaled, and this cache is per-process. A
/// block placed on replica 1 does not invalidate a socket held on replica 2, so that socket can keep
/// delivering until it reconnects. The DB-backed paths (list, notify, DM list) are always correct on every
/// replica; only the live socket is affected. This is called out explicitly rather than hidden: closing it
/// needs a cross-replica invalidation bus (Redis pub/sub or LISTEN/NOTIFY) and is a decision, not a detail.
/// It is raised in the review packet.
#[derive(Clone, Default)]
pub struct BlockCache {
    inner: Arc<RwLock<HashMap<Uuid, Arc<HashSet<Uuid>>>>>,
}

impl BlockCache {
    fn get(&self, blocker: Uuid) -> Option<Arc<HashSet<Uuid>>> {
        self.inner.read().ok()?.get(&blocker).cloned()
    }

    fn put(&self, blocker: Uuid, set: Arc<HashSet<Uuid>>) {
        if let Ok(mut m) = self.inner.write() {
            m.insert(blocker, set);
        }
    }

    /// Drop the entry so the next read re-queries. Called on every mutation.
    pub fn invalidate(&self, blocker: Uuid) {
        if let Ok(mut m) = self.inner.write() {
            m.remove(&blocker);
        }
    }
}

/// Every subject `blocker` has blocked, memoised. The enforcement points call this once per request (or once
/// per frame, off the cache) and thread the set through — never a per-message query.
pub async fn blocked_by(st: &AppState, tenant: Uuid, blocker: Uuid) -> Arc<HashSet<Uuid>> {
    if let Some(hit) = st.blocks.get(blocker) {
        return hit;
    }
    let set = load_blocked(st, tenant, blocker).await;
    st.blocks.put(blocker, set.clone());
    set
}

/// Fails OPEN to an empty set on a DB error, deliberately and with a warning.
///
/// This is the one place the choice is uncomfortable. Failing closed (treat everyone as blocked) would black
/// out the whole channel on a transient DB blip — a self-inflicted outage. Failing open means a blocked
/// person's message could slip through during that blip. The blip is loud (it logs) and the leak is one
/// message, whereas failing closed silently breaks chat for everyone. Same call `prefs::modes_for_channel`
/// already makes for notify overrides.
async fn load_blocked(st: &AppState, tenant: Uuid, blocker: Uuid) -> Arc<HashSet<Uuid>> {
    let mut tx = match db::tenant_tx(&st.pool, &tenant).await {
        Ok(tx) => tx,
        Err(e) => {
            tracing::warn!(target: "cyberos_chat::blocks", error = %e, "block lookup failed; failing open");
            return Arc::new(HashSet::new());
        }
    };
    let rows: Result<Vec<(Uuid,)>, _> =
        sqlx::query_as("SELECT blocked_subject_id FROM chat_blocks WHERE blocker_subject_id = $1")
            .bind(blocker)
            .fetch_all(&mut *tx)
            .await;
    let _ = tx.commit().await;
    match rows {
        Ok(r) => Arc::new(r.into_iter().map(|(id,)| id).collect()),
        Err(e) => {
            tracing::warn!(target: "cyberos_chat::blocks", error = %e, "block lookup failed; failing open");
            Arc::new(HashSet::new())
        }
    }
}

/// The REVERSE question, and the only one the notification fan-out asks: of these recipients, which have
/// blocked this sender? Returns the recipients to SKIP.
///
/// A candidate list rather than a global scan: in a large channel the fan-out already knows exactly who it
/// is about to notify, so "of these 200, which blocked the sender" is one indexed lookup against
/// `chat_blocks_blocked_idx`. Asking "who has blocked this sender" globally returns a set we would have to
/// intersect anyway.
pub async fn blockers_of(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    sender: Uuid,
    candidates: &[Uuid],
) -> HashSet<Uuid> {
    if candidates.is_empty() {
        return HashSet::new();
    }
    let rows: Result<Vec<(Uuid,)>, _> = sqlx::query_as(
        "SELECT blocker_subject_id FROM chat_blocks
          WHERE blocked_subject_id = $1 AND blocker_subject_id = ANY($2)",
    )
    .bind(sender)
    .bind(candidates)
    .fetch_all(&mut **tx)
    .await;
    match rows {
        Ok(r) => r.into_iter().map(|(id,)| id).collect(),
        Err(e) => {
            // Fails open, as above — a notify hiccup must not silence the whole channel.
            tracing::warn!(target: "cyberos_chat::blocks", error = %e, "blockers_of failed; failing open");
            HashSet::new()
        }
    }
}

// ── Handlers ─────────────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct BlockRequest {
    pub subject_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct BlockEntry {
    pub subject_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// POST /v1/chat/blocks — block a person. Idempotent: blocking twice is a no-op and still 204.
///
/// Idempotent by design (§3). A distinguishable second response (409, or 404 on unblock) would let a caller
/// enumerate their own block state through side effects, and — more prosaically — invites a client to render
/// an error for a state the user does not care about.
pub async fn block(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<BlockRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let blocker = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    // §1 #3. Also enforced by chat_blocks_not_self in the DB.
    if req.subject_id == blocker {
        return Err((StatusCode::BAD_REQUEST, "cannot block yourself".to_string()));
    }

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    let res = sqlx::query(
        "INSERT INTO chat_blocks (tenant_id, blocker_subject_id, blocked_subject_id)
         VALUES ($1,$2,$3) ON CONFLICT DO NOTHING",
    )
    .bind(tenant)
    .bind(blocker)
    .bind(req.subject_id)
    .execute(&mut *tx)
    .await
    .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;

    // Invalidate BEFORE returning, so the blocker's own open sockets stop delivering by the time their
    // client sees the 204 (§10 row 5).
    st.blocks.invalidate(blocker);

    // §1 #14 / AC 15 — exactly one audit row per real mutation. The idempotent no-op emits nothing: a second
    // block is not an event, and a chain full of no-ops is a chain nobody reads.
    if res.rows_affected() > 0 {
        audit::emit(
            &st,
            tenant,
            blocker,
            "chat.subject_blocked",
            serde_json::json!({ "blocked_subject_id": req.subject_id }),
        )
        .await;
    }
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /v1/chat/blocks/:subject_id — unblock. Idempotent: unblocking someone you never blocked is 204.
///
/// §1 #11 — unblocking restores everything, in place, immediately. Nothing was ever deleted: the block
/// filters READS only, so the messages sent during the block are still sitting in the table in their
/// original positions and simply become visible again on the next fetch.
pub async fn unblock(
    State(st): State<AppState>,
    Path(subject): Path<Uuid>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let blocker = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    let res = sqlx::query(
        "DELETE FROM chat_blocks WHERE blocker_subject_id = $1 AND blocked_subject_id = $2",
    )
    .bind(blocker)
    .bind(subject)
    .execute(&mut *tx)
    .await
    .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;

    st.blocks.invalidate(blocker);

    if res.rows_affected() > 0 {
        audit::emit(
            &st,
            tenant,
            blocker,
            "chat.subject_unblocked",
            serde_json::json!({ "blocked_subject_id": subject }),
        )
        .await;
    }
    Ok(StatusCode::NO_CONTENT)
}

/// GET /v1/chat/blocks — the caller's OWN block list, and only ever their own (§1 #1, #2). There is no
/// surface anywhere that lets B discover A blocked them.
pub async fn list(
    State(st): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<BlockEntry>>, (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let blocker = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    let rows: Vec<(Uuid, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT blocked_subject_id, created_at FROM chat_blocks
          WHERE blocker_subject_id = $1 ORDER BY created_at DESC",
    )
    .bind(blocker)
    .fetch_all(&mut *tx)
    .await
    .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;

    Ok(Json(
        rows.into_iter()
            .map(|(subject_id, created_at)| BlockEntry {
                subject_id,
                created_at,
            })
            .collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uid(n: u8) -> Uuid {
        Uuid::from_bytes([n; 16])
    }

    #[test]
    fn cache_round_trips_and_invalidates() {
        let c = BlockCache::default();
        let a = uid(1);
        assert!(c.get(a).is_none(), "cold cache is a miss");

        let set: Arc<HashSet<Uuid>> = Arc::new([uid(2), uid(3)].into_iter().collect());
        c.put(a, set);
        let hit = c.get(a).expect("warm");
        assert!(hit.contains(&uid(2)) && hit.contains(&uid(3)));

        // The mutation path MUST make the next read re-query — an open socket that keeps delivering after
        // the block lands is a live failure of §1 #4.
        c.invalidate(a);
        assert!(c.get(a).is_none(), "invalidate forces a re-read");
    }

    #[test]
    fn cache_is_per_blocker_not_global() {
        // A's block list must never leak into B's view. Blocks are directional and private (§1 #2).
        let c = BlockCache::default();
        c.put(uid(1), Arc::new([uid(9)].into_iter().collect()));
        assert!(c.get(uid(1)).unwrap().contains(&uid(9)));
        assert!(
            c.get(uid(2)).is_none(),
            "B has no entry just because A does"
        );
    }
}
