//! TASK-TEN-206 — Pg insert/sum when DATABASE_URL is set; otherwise skip.

use chrono::Utc;
use cyberos_metering::axes::MeteringAxis;
use cyberos_metering::drain::drain_to_recorder;
use cyberos_metering::pg::{drain_wal_to_pg, insert_event, period_sum};
use cyberos_metering::recorder::{InMemoryRecorder, MeteringEvent};
use cyberos_metering::usage::utc_month_start;
use cyberos_metering::wal_queue::WalQueue;
use std::sync::Mutex;

#[test]
fn offline_drain_to_memory() {
    let wal = Mutex::new(WalQueue::new(32));
    let rec = InMemoryRecorder::default();
    wal.lock()
        .unwrap()
        .push(MeteringEvent {
            tenant_id: "t".into(),
            axis: MeteringAxis::ApiCalls,
            quantity: 1,
            idempotency_key: "offline-1".into(),
            source_service: "auth".into(),
            occurred_at: Utc::now(),
            extra: serde_json::json!({}),
        })
        .unwrap();
    assert_eq!(drain_to_recorder(&wal, &rec), 1);
}

#[tokio::test]
async fn pg_insert_idempotent_when_database_url() {
    let Ok(url) = std::env::var("DATABASE_URL") else {
        eprintln!("skip: DATABASE_URL unset");
        return;
    };
    let pool = sqlx::PgPool::connect(&url).await.expect("connect");
    let key = format!("pg-idem-{}", uuid::Uuid::new_v4());
    let event = MeteringEvent {
        tenant_id: "t_pg".into(),
        axis: MeteringAxis::ApiCalls,
        quantity: 1,
        idempotency_key: key.clone(),
        source_service: "test".into(),
        occurred_at: Utc::now(),
        extra: serde_json::json!({}),
    };
    assert!(insert_event(&pool, &event).await.unwrap().is_some());
    assert!(insert_event(&pool, &event).await.unwrap().is_none());
    let sum = period_sum(
        &pool,
        "t_pg",
        MeteringAxis::ApiCalls,
        utc_month_start(Utc::now()),
    )
    .await
    .unwrap();
    assert!(sum >= 1);

    let wal = Mutex::new(WalQueue::new(8));
    wal.lock()
        .unwrap()
        .push(MeteringEvent {
            tenant_id: "t_pg".into(),
            axis: MeteringAxis::ApiCalls,
            quantity: 1,
            idempotency_key: format!("drain-{}", uuid::Uuid::new_v4()),
            source_service: "test".into(),
            occurred_at: Utc::now(),
            extra: serde_json::json!({}),
        })
        .unwrap();
    assert_eq!(drain_wal_to_pg(&wal, &pool).await, 1);
}
