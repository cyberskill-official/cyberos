//! FR-AUTH-004 — JWT issuance + verification round-trip.
//!
//! Requires a live Postgres (boots services/dev/docker-compose.yml).
//! Run with `cargo test -p cyberos-auth -- --ignored`.

#[tokio::test]
#[ignore = "requires Postgres — boot services/dev/docker-compose.yml first"]
async fn issue_then_verify_round_trips_with_correct_claims() {
    use cyberos_auth::{jwt::JwtService, keygen};
    use cyberos_types::{SubjectId, TenantId};
    use sqlx::PgPool;

    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var");
    let pool = PgPool::connect(&url).await.expect("connect");

    // Bootstrap a fresh signing key for this test (kid prefixed to avoid clashes).
    let kid = format!("test-{}", uuid::Uuid::new_v4().simple());
    let key = keygen::generate_rsa_2048().expect("keygen");
    let expires = chrono::Utc::now() + chrono::Duration::days(30);
    sqlx::query(
        "INSERT INTO auth_signing_keys (kid, algorithm, public_pem, private_pem, status, expires_at)
         VALUES ($1, 'RS256', $2, $3, 'active', $4)",
    )
    .bind(&kid)
    .bind(&key.public_pem)
    .bind(&key.private_pem)
    .bind(expires)
    .execute(&pool)
    .await
    .expect("insert key");

    let svc = JwtService::new(pool.clone(), "https://auth.test.cyberos".to_string());

    let tenant = TenantId::new();
    let subject = SubjectId::new();
    let scopes = vec!["admin:tenants".to_string(), "admin:subjects".to_string()];
    let tokens = svc
        .issue(
            tenant,
            subject,
            "alice@test.cyberos",
            "human",
            scopes.clone(),
            vec![],
            None,
            None,
            Some("00-traceparent-x-01".into()),
        )
        .await
        .expect("issue");

    let claims = svc.verify(&tokens.access_token).await.expect("verify");
    assert_eq!(claims.tenant_id, tenant.to_string());
    assert_eq!(claims.sub, subject.to_string());
    assert_eq!(claims.kind, "human");
    assert_eq!(claims.email, "alice@test.cyberos"); // FR-AUTH-004 §1 #2 G-013
    assert_eq!(claims.scope_grants, scopes);
    assert_eq!(claims.iss, "https://auth.test.cyberos");
    assert_eq!(claims.traceparent.as_deref(), Some("00-traceparent-x-01"));

    // Cleanup
    sqlx::query("DELETE FROM auth_signing_keys WHERE kid = $1")
        .bind(&kid)
        .execute(&pool)
        .await
        .ok();
}

#[tokio::test]
#[ignore = "requires Postgres"]
async fn jwks_publishes_active_key() {
    use cyberos_auth::{jwt::JwtService, keygen};
    use sqlx::PgPool;

    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var");
    let pool = PgPool::connect(&url).await.expect("connect");

    let kid = format!("test-jwks-{}", uuid::Uuid::new_v4().simple());
    let key = keygen::generate_rsa_2048().expect("keygen");
    let expires = chrono::Utc::now() + chrono::Duration::days(30);
    sqlx::query(
        "INSERT INTO auth_signing_keys (kid, algorithm, public_pem, private_pem, status, expires_at)
         VALUES ($1, 'RS256', $2, $3, 'active', $4)",
    )
    .bind(&kid)
    .bind(&key.public_pem)
    .bind(&key.private_pem)
    .bind(expires)
    .execute(&pool)
    .await
    .expect("insert");

    let svc = JwtService::new(pool.clone(), "https://auth.test.cyberos".to_string());
    let doc = svc.jwks_for_publication().await.expect("jwks");
    assert!(
        doc.keys
            .iter()
            .any(|k| k.kid == kid && k.kty == "RSA" && k.alg == "RS256"),
        "jwks must publish the test key with RS256/RSA"
    );

    sqlx::query("DELETE FROM auth_signing_keys WHERE kid = $1")
        .bind(&kid)
        .execute(&pool)
        .await
        .ok();
}
