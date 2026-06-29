//! Push notifications (FR-CHAT-101 slice 4). On a new message, the members who are NOT currently connected
//! and who have a registered device are the push targets. Slice 4 emits the intent (logged); the actual
//! APNS/FCM delivery is a deploy-time integration that plugs in here. Best-effort, runs off the hot path.

use uuid::Uuid;

use crate::AppState;

pub async fn notify(st: AppState, channel: Uuid, tenant: Uuid, sender: Uuid, message_id: Uuid) {
    let online = st.presence.online(channel);
    let mut tx = match db_tx(&st, &tenant).await {
        Some(tx) => tx,
        None => return,
    };
    let members: Vec<(Uuid,)> =
        match sqlx::query_as("SELECT subject_id FROM chat_channel_members WHERE channel_id = $1")
            .bind(channel)
            .fetch_all(&mut *tx)
            .await
        {
            Ok(m) => m,
            Err(_) => return,
        };
    let mut intents = 0u32;
    for (member,) in members {
        if member == sender || online.contains(&member) {
            continue;
        }
        let devices: Vec<(String, String)> =
            sqlx::query_as("SELECT platform, token FROM chat_devices WHERE subject_id = $1")
                .bind(member)
                .fetch_all(&mut *tx)
                .await
                .unwrap_or_default();
        for (platform, token) in devices {
            intents += 1;
            tracing::info!(
                target: "cyberos_chat::push",
                channel = %channel,
                subject = %member,
                platform = %platform,
                token = %token,
                message_id = %message_id,
                "push intent (real APNS/FCM delivery is a deploy integration)"
            );
        }
    }
    let _ = tx.commit().await;
    if intents > 0 {
        tracing::info!(target: "cyberos_chat::push", channel = %channel, intents, "push intents emitted");
    }
}

async fn db_tx<'a>(
    st: &'a AppState,
    tenant: &Uuid,
) -> Option<sqlx::Transaction<'a, sqlx::Postgres>> {
    crate::db::tenant_tx(&st.pool, tenant).await.ok()
}
