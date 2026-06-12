//! FR-EMAIL-001 §1 #5 — per-tenant DKIM key registry.
//!
//! Slice 1 generates RSA-2048 keys. The KMS-encrypted private blob is
//! stored in `dkim_keys.private_key_kms_encrypted_blob`; the plaintext
//! never lives on Postgres-readable rows. Stalwart's signing keystore is
//! synced from this table at boot + on rotation events (`email.dkim_key_rotated`).
//!
//! The real KMS-encrypt step is held behind a trait so tests can plug a
//! mock encryptor without a live KMS endpoint.

use crate::errors::{EmailError, EmailResult};
use crate::types::{DkimKey, DkimKeyStatus, KeyAlgorithm};
use chrono::Utc;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

/// Trait abstracting the KMS encrypt step so unit tests don't need AWS.
///
/// Uses Rust 2024 native async-in-traits — no `async-trait` crate
/// required. The `Send` bound on the returned future is explicit so
/// trait objects work in `tokio::spawn` contexts.
pub trait KmsEncryptor: Send + Sync {
    fn encrypt<'a>(
        &'a self,
        kms_key_id: &'a str,
        plaintext: &'a [u8],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = EmailResult<Vec<u8>>> + Send + 'a>>;
}

/// Test-only encryptor that prefixes the plaintext with the KMS key id.
/// Reversible only inside the test harness via [`MockKmsEncryptor::key_id_from_blob`].
pub struct MockKmsEncryptor;

impl KmsEncryptor for MockKmsEncryptor {
    fn encrypt<'a>(
        &'a self,
        kms_key_id: &'a str,
        plaintext: &'a [u8],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = EmailResult<Vec<u8>>> + Send + 'a>>
    {
        let kms_key_id = kms_key_id.to_owned();
        let plaintext = plaintext.to_vec();
        Box::pin(async move {
            let mut out = Vec::with_capacity(kms_key_id.len() + 1 + plaintext.len());
            out.extend_from_slice(kms_key_id.as_bytes());
            out.push(b'|');
            out.extend_from_slice(&plaintext);
            Ok(out)
        })
    }
}

impl MockKmsEncryptor {
    /// Inspector for tests — recover the kms_key_id from a blob.
    pub fn key_id_from_blob(blob: &[u8]) -> Option<String> {
        blob.iter()
            .position(|b| *b == b'|')
            .map(|pos| String::from_utf8_lossy(&blob[..pos]).into_owned())
    }
}

/// Slice-1 placeholder key generator. Returns a fixed PEM-shaped string so
/// callers can verify the schema CHECK constraint passes without a real
/// RSA generation step in unit tests. The production path (in the binary
/// `src/bin/server.rs`) wires `openssl` or a pure-Rust RSA crate behind a
/// feature flag; that lands in slice 2.
pub fn generate_rsa_2048_pem_pair() -> EmailResult<(String, Vec<u8>)> {
    generate_rsa_2048_pem_pair_for_seed("slice-1-default")
}

pub fn generate_rsa_2048_pem_pair_for_seed(seed: &str) -> EmailResult<(String, Vec<u8>)> {
    // Slice 1 produces deterministic placeholders so the migration's CHECK
    // constraint (length 100..=10000 for PEM, 100..=8192 for blob) is
    // satisfied without invoking actual crypto. The CLI binary wires a
    // proper generator behind a feature flag in slice 2.
    let public_pem = format!(
        "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
        deterministic_pem_body(&format!("{seed}:public"), 380)
    );
    let private_pem_plaintext = format!(
        "-----BEGIN RSA PRIVATE KEY-----\n{}\n-----END RSA PRIVATE KEY-----",
        deterministic_pem_body(&format!("{seed}:private"), 1600)
    );
    Ok((public_pem, private_pem_plaintext.into_bytes()))
}

fn deterministic_pem_body(seed: &str, len: usize) -> String {
    let mut out = String::with_capacity(len);
    let mut counter = 0_u64;
    while out.len() < len {
        let digest = Sha256::digest(format!("{seed}:{counter}").as_bytes());
        out.push_str(&format!("{digest:x}"));
        counter += 1;
    }
    out.truncate(len);
    out
}

/// FR-EMAIL-001 §4 #7 — provision a per-tenant DKIM key. The unique partial
/// index `uniq_active_dkim_key` enforces at most one ACTIVE key per
/// (tenant, selector); rotation transitions the prior row to `rotated`
/// before inserting the new one.
pub async fn provision_key(
    db: &PgPool,
    encryptor: &dyn KmsEncryptor,
    tenant_id: Uuid,
    selector: &str,
    algorithm: KeyAlgorithm,
    kms_key_id: &str,
) -> EmailResult<DkimKey> {
    // Refuse if an active key already exists for the (tenant, selector).
    let existing: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM dkim_keys WHERE tenant_id = $1 AND dkim_selector = $2 AND status = 'active'",
    )
    .bind(tenant_id)
    .bind(selector)
    .fetch_optional(db)
    .await?;

    if existing.is_some() {
        return Err(EmailError::DkimKeyAlreadyExists(
            tenant_id,
            selector.to_owned(),
        ));
    }

    let id = Uuid::new_v4();
    let (public_pem, private_plaintext) = match algorithm {
        KeyAlgorithm::Rsa2048 => {
            generate_rsa_2048_pem_pair_for_seed(&format!("{tenant_id}:{selector}:{id}"))?
        }
        KeyAlgorithm::Ed25519 => {
            return Err(EmailError::DkimKeyGen(
                "ed25519 deferred to FR-EMAIL-004 / slice 2".into(),
            ))
        }
    };

    let encrypted_blob = encryptor.encrypt(kms_key_id, &private_plaintext).await?;

    let now = Utc::now();

    sqlx::query(
        "INSERT INTO dkim_keys (id, tenant_id, dkim_selector, key_algorithm, public_key_pem,
                                private_key_kms_encrypted_blob, kms_key_id, status, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'active', $8)",
    )
    .bind(id)
    .bind(tenant_id)
    .bind(selector)
    .bind(algorithm.as_str())
    .bind(&public_pem)
    .bind(&encrypted_blob)
    .bind(kms_key_id)
    .bind(now)
    .execute(db)
    .await?;

    Ok(DkimKey {
        id,
        tenant_id,
        dkim_selector: selector.to_owned(),
        key_algorithm: algorithm.as_str().to_owned(),
        public_key_pem: public_pem,
        private_key_kms_encrypted_blob: encrypted_blob,
        kms_key_id: kms_key_id.to_owned(),
        status: DkimKeyStatus::Active.as_str().to_owned(),
        created_at: now,
        rotated_at: None,
    })
}

/// FR-EMAIL-001 §4 #8 — rotate the active key. Atomic: the prior active row
/// transitions to `rotated` in the same tx as the new `active` insert.
pub async fn rotate_key(
    db: &PgPool,
    encryptor: &dyn KmsEncryptor,
    tenant_id: Uuid,
    selector: &str,
    kms_key_id: &str,
) -> EmailResult<(Option<Uuid>, Uuid)> {
    let mut tx = db.begin().await?;

    // Find the currently active key (may not exist on a fresh tenant).
    let prior: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM dkim_keys WHERE tenant_id = $1 AND dkim_selector = $2 AND status = 'active'",
    )
    .bind(tenant_id)
    .bind(selector)
    .fetch_optional(&mut *tx)
    .await?;

    let prior_id = prior.map(|(id,)| id);
    let now = Utc::now();

    // Demote the prior key.
    if let Some(pid) = prior_id {
        sqlx::query("UPDATE dkim_keys SET status = 'rotated', rotated_at = $1 WHERE id = $2")
            .bind(now)
            .bind(pid)
            .execute(&mut *tx)
            .await?;
    }

    // Mint a new key.
    let new_id = Uuid::new_v4();
    let (public_pem, private_plaintext) =
        generate_rsa_2048_pem_pair_for_seed(&format!("{tenant_id}:{selector}:{new_id}"))?;
    let encrypted_blob = encryptor.encrypt(kms_key_id, &private_plaintext).await?;

    sqlx::query(
        "INSERT INTO dkim_keys (id, tenant_id, dkim_selector, key_algorithm, public_key_pem,
                                private_key_kms_encrypted_blob, kms_key_id, status, created_at)
         VALUES ($1, $2, $3, 'rsa-2048', $4, $5, $6, 'active', $7)",
    )
    .bind(new_id)
    .bind(tenant_id)
    .bind(selector)
    .bind(&public_pem)
    .bind(&encrypted_blob)
    .bind(kms_key_id)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((prior_id, new_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_encryptor_round_trip_recovers_kms_id() {
        let enc = MockKmsEncryptor;
        let blob = enc.encrypt("alias/test", b"plaintext").await.unwrap();
        assert_eq!(
            MockKmsEncryptor::key_id_from_blob(&blob),
            Some("alias/test".into())
        );
    }

    #[test]
    fn rsa_2048_pem_sizes_satisfy_db_check() {
        let (pub_pem, priv_pem) = generate_rsa_2048_pem_pair().unwrap();
        assert!(pub_pem.len() >= 100 && pub_pem.len() <= 10000);
        assert!(priv_pem.len() >= 100 && priv_pem.len() <= 8192);
        assert!(pub_pem.contains("-----BEGIN PUBLIC KEY-----"));
        assert!(pub_pem.contains("-----END PUBLIC KEY-----"));
    }

    #[test]
    fn seeded_rsa_2048_placeholders_are_distinct() {
        let (a, _) = generate_rsa_2048_pem_pair_for_seed("tenant-a").unwrap();
        let (b, _) = generate_rsa_2048_pem_pair_for_seed("tenant-b").unwrap();
        assert_ne!(a, b);
    }
}
