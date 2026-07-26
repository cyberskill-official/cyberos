//! TASK-INV-012 — Wise HTTP host: cache, rotation, handler, stale, unknown type.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use base64::Engine;
use chrono::{Duration as ChronoDuration, Utc};
use cyberos_inv::wise::{
    router, verify_signature, CachedPublicKeys, PublicKeyError, PublicKeySource, StaticPemSource,
    WiseHostState, WiseProcessor, WiseWalQueue,
};
use http_body_util::BodyExt;
use rand::rngs::OsRng;
use rsa::pkcs1v15::SigningKey;
use rsa::pkcs8::{EncodePublicKey, LineEnding};
use rsa::signature::{SignatureEncoding, Signer};
use rsa::{RsaPrivateKey, RsaPublicKey};
use sha2::Sha256;
use tower::ServiceExt;

#[derive(Debug)]
struct SharedRotate {
    /// (first_pem, second_pem, advanced, fetch_count)
    inner: Arc<Mutex<(String, String, bool, usize)>>,
}

impl PublicKeySource for SharedRotate {
    fn fetch(&self, _profile_id: i64) -> Result<String, PublicKeyError> {
        let mut g = self.inner.lock().unwrap();
        g.3 += 1;
        if g.2 {
            Ok(g.1.clone())
        } else {
            Ok(g.0.clone())
        }
    }
}

fn keypair() -> (String, SigningKey<Sha256>) {
    let mut rng = OsRng;
    let private = RsaPrivateKey::new(&mut rng, 2048).unwrap();
    let public = RsaPublicKey::from(&private);
    let pem = public.to_public_key_pem(LineEnding::LF).unwrap();
    (pem, SigningKey::<Sha256>::new(private))
}

fn sign(signing: &SigningKey<Sha256>, body: &[u8]) -> String {
    let sig = signing.sign(body);
    base64::engine::general_purpose::STANDARD.encode(sig.to_bytes())
}

fn credit_body(profile_id: i64, event_id: &str, occurred_at: &str) -> Vec<u8> {
    format!(
        r#"{{
          "event_type":"balances#credit",
          "event_id":"{event_id}",
          "data":{{
            "resource":{{"id":"res-1","profile_id":{profile_id}}},
            "occurred_at":"{occurred_at}"
          }}
        }}"#
    )
    .into_bytes()
}

fn host(
    pem: &str,
) -> (
    Arc<WiseHostState<StaticPemSource>>,
    Arc<WiseProcessor>,
    Arc<Mutex<WiseWalQueue>>,
) {
    let keys = Arc::new(CachedPublicKeys::new(StaticPemSource::new(pem.to_string())));
    let wal = Arc::new(Mutex::new(WiseWalQueue::default()));
    let processor = Arc::new(WiseProcessor::new(Arc::clone(&wal)));
    let state = Arc::new(WiseHostState {
        keys,
        wal: Arc::clone(&wal),
        processor: Arc::clone(&processor),
    });
    (state, processor, wal)
}

#[tokio::test]
async fn valid_signature_200_and_receipt() {
    let (pem, signing) = keypair();
    let (state, processor, wal) = host(&pem);
    let app = router(Arc::clone(&state));
    let now = Utc::now().to_rfc3339();
    let body = credit_body(42, "11111111-1111-1111-1111-111111111111", &now);
    let sig = sign(&signing, &body);

    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/webhooks/wise/42")
                .header("X-Signature-SHA256", sig)
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    assert!(bytes.is_empty());

    assert_eq!(wal.lock().unwrap().depth(), 1);
    processor.drain_all();
    assert!(processor.has_receipt(42, "11111111-1111-1111-1111-111111111111"));
    assert_eq!(processor.receipt_count(), 1);
}

#[tokio::test]
async fn invalid_signature_401() {
    let (pem, _) = keypair();
    let (state, processor, wal) = host(&pem);
    let app = router(state);
    let now = Utc::now().to_rfc3339();
    let body = credit_body(1, "22222222-2222-2222-2222-222222222222", &now);

    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/webhooks/wise/1")
                .header("X-Signature-SHA256", sig_placeholder())
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(wal.lock().unwrap().depth(), 0);
    assert_eq!(processor.receipt_count(), 0);
}

fn sig_placeholder() -> &'static str {
    "bm90LXZhbGlk"
}

#[tokio::test]
async fn rotation_retry_accepts_refreshed_pem() {
    let (pem_old, _) = keypair();
    let (pem_new, signing_new) = keypair();
    let cell = Arc::new(Mutex::new((pem_old, pem_new.clone(), false, 0usize)));
    let keys = Arc::new(CachedPublicKeys::new(SharedRotate {
        inner: Arc::clone(&cell),
    }));
    keys.get_or_fetch(7).unwrap();
    assert_eq!(cell.lock().unwrap().3, 1);

    let wal = Arc::new(Mutex::new(WiseWalQueue::default()));
    let processor = Arc::new(WiseProcessor::new(Arc::clone(&wal)));
    let state = Arc::new(WiseHostState {
        keys,
        wal: Arc::clone(&wal),
        processor: Arc::clone(&processor),
    });
    let app = router(state);

    cell.lock().unwrap().2 = true;
    let now = Utc::now().to_rfc3339();
    let body = credit_body(7, "33333333-3333-3333-3333-333333333333", &now);
    let sig = sign(&signing_new, &body);
    verify_signature(&pem_new, &body, &sig).unwrap();

    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/webhooks/wise/7")
                .header("X-Signature-SHA256", sig)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(cell.lock().unwrap().3 >= 2);
    processor.drain_all();
    assert!(processor.has_receipt(7, "33333333-3333-3333-3333-333333333333"));
}

#[tokio::test]
async fn stale_event_200_without_wal() {
    let (pem, signing) = keypair();
    let (state, processor, wal) = host(&pem);
    let app = router(state);
    let old = (Utc::now() - ChronoDuration::days(6)).to_rfc3339();
    let body = credit_body(3, "44444444-4444-4444-4444-444444444444", &old);
    let sig = sign(&signing, &body);

    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/webhooks/wise/3")
                .header("X-Signature-SHA256", sig)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(wal.lock().unwrap().depth(), 0);
    processor.drain_all();
    assert_eq!(processor.receipt_count(), 0);
}

#[tokio::test]
async fn unknown_event_type_dead_letter_no_receipt() {
    let (pem, signing) = keypair();
    let (state, processor, wal) = host(&pem);
    let app = router(state);
    let now = Utc::now().to_rfc3339();
    let body = format!(
        r#"{{"event_type":"unknown#thing","event_id":"55555555-5555-5555-5555-555555555555","data":{{"resource":{{"id":"1","profile_id":9}},"occurred_at":"{now}"}}}}"#
    )
    .into_bytes();
    let sig = sign(&signing, &body);

    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/webhooks/wise/9")
                .header("X-Signature-SHA256", sig)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(wal.lock().unwrap().depth(), 0);
    assert_eq!(processor.dead_letter_count(), 1);
    assert_eq!(processor.receipt_count(), 0);
}

#[test]
fn cache_ttl_skips_second_fetch() {
    let cell = Arc::new(Mutex::new((
        "pem".to_string(),
        "pem".to_string(),
        false,
        0usize,
    )));
    let cache = CachedPublicKeys::with_ttl(
        SharedRotate {
            inner: Arc::clone(&cell),
        },
        Duration::from_secs(3600),
    );
    cache.get_or_fetch(1).unwrap();
    cache.get_or_fetch(1).unwrap();
    assert_eq!(cell.lock().unwrap().3, 1);
}
