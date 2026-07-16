//! TASK-AUTH-111 — SSO display names: the self-heal, the no-clobber guarantee, and the visibility view.
//!
//!   cd services/dev && docker compose up -d
//!   DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos \
//!     cargo test -p cyberos-auth --test sso_display_name_test -- --ignored --test-threads=1
//!
//! These tests exercise `display_name::heal` and the `subjects_display_name_unset` view directly against
//! Postgres, rather than driving a full OIDC login. That is deliberate, and worth being explicit about
//! rather than quietly doing.
//!
//! Driving a real sign-in would need a mock IdP: a JWKS endpoint, a token endpoint, and a signed ID token
//! per case. That harness does not exist in this repo, and building it would test `jsonwebtoken` — a library
//! that is not the thing at risk here. What IS at risk is a three-line rule about when a stored name may be
//! overwritten, and that rule is what these tests pin.
//!
//! AC coverage (§4):
//!   AC 7   a_returning_user_whose_name_is_their_email_is_repaired
//!   AC 8   a_deliberately_set_name_is_never_clobbered      <- the guarantee the task turns on
//!   AC 9   a_blank_resolution_never_blanks_a_stored_name
//!   AC 13  the_visibility_view_drains
//!   AC 6   heal_never_writes_the_email_back                 (belt and braces; the resolver already refuses)
//!
//! AC 1-5, 9, 11 are unit-tested in `src/display_name.rs` — the resolution chain is a pure function and a
//! compile-time-checked test there is stronger than a round-trip through Postgres.
//!
//! NOT covered here, and named rather than skipped:
//!   AC 10  (the name never reaches the log stream) — needs a tracing subscriber capturing at DEBUG. The
//!          enforcement is that `resolve` returns the RUNG as a `&'static str` and both call sites log that
//!          and only that; there is no `?name` anywhere to find. Reviewable by grep, not by assertion.
//!   AC 12  (SAML fixed identically) — structural: saml.rs calls the same `display_name::resolve` and the
//!          same `display_name::heal` as oidc.rs. There is one chain, so there is nothing that can drift
//!          apart to test. Driving a SAML assertion end-to-end would test the XML extractor, not the rule.

use cyberos_auth::display_name;
use sqlx::PgPool;
use uuid::Uuid;

/// Connects, then drops to `cyberos_app` — the runtime app's identity.
///
/// Superusers bypass RLS entirely, so a superuser connection would make the tenant scoping in `heal`
/// vacuous. `cyberos_app` is what production runs as.
async fn pool() -> PgPool {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var");
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("SET ROLE cyberos_app").execute(conn).await?;
                Ok(())
            })
        })
        .connect(&url)
        .await
        .expect("connect — is Postgres up? cd services/dev && docker compose up -d")
}

/// Seeds a tenant as root.
///
/// The nil-UUID GUC is not decoration. `tenants_modify` (migrations/0005) is `WITH CHECK
/// (current_setting('app.current_tenant_id') = '000…000')` — only the root tenant may create tenants. With no
/// GUC set at all the policy simply refuses the INSERT, and `cyberos_app` is subject to it.
///
/// The first version of this helper did `let _ = sqlx::query(...)`, which discarded exactly that rejection.
/// The tenant was never created, the test sailed on, and the failure surfaced one call later as a foreign-key
/// violation on the SUBJECT insert — pointing at the wrong line and describing the wrong problem. Errors in
/// test fixtures get `.expect()`, not `let _`: a fixture that fails silently produces a test that lies.
async fn seed_tenant(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();
    let mut tx = pool.begin().await.expect("begin");
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(Uuid::nil().to_string())
        .execute(&mut *tx)
        .await
        .expect("root guc");
    sqlx::query(
        "INSERT INTO tenants (id, slug, display_name, country, plan_tier, status, residency)
              VALUES ($1, $2, 'TASK-AUTH-111 Tenant', 'VN', 'starter', 'active', 'vn-1')",
    )
    .bind(id)
    .bind(format!("task111-{}", id.simple()))
    .execute(&mut *tx)
    .await
    .expect("seed tenant");
    tx.commit().await.expect("commit tenant");
    id
}

/// A subject in whatever state we need — including the damaged state production is in today.
async fn seed_subject(pool: &PgPool, tenant: Uuid, display_name: &str, email: &str) -> Uuid {
    let mut tx = pool.begin().await.expect("begin");
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant.to_string())
        .execute(&mut *tx)
        .await
        .expect("guc");
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO subjects (tenant_id, handle, display_name, email, kind, status, password_hash)
              VALUES ($1, $2, $3, $4, 'human', 'active', 'bcrypt:fake')
         RETURNING id",
    )
    .bind(tenant)
    .bind(format!("@u-{}", Uuid::new_v4().simple()))
    .bind(display_name)
    .bind(email)
    .fetch_one(&mut *tx)
    .await
    .expect("seed subject");
    tx.commit().await.unwrap();
    id
}

async fn display_name_of(pool: &PgPool, tenant: Uuid, id: Uuid) -> Option<String> {
    let mut tx = pool.begin().await.expect("begin");
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant.to_string())
        .execute(&mut *tx)
        .await
        .expect("guc");
    let row: (Option<String>,) = sqlx::query_as("SELECT display_name FROM subjects WHERE id = $1")
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .expect("read back");
    tx.commit().await.unwrap();
    row.0
}

async fn unset_rows_for(pool: &PgPool, tenant: Uuid) -> i64 {
    let mut tx = pool.begin().await.expect("begin");
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant.to_string())
        .execute(&mut *tx)
        .await
        .expect("guc");
    let (n,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM subjects_display_name_unset WHERE tenant_id = $1")
            .bind(tenant)
            .fetch_one(&mut *tx)
            .await
            .expect("view");
    tx.commit().await.unwrap();
    n
}

// ── Tests ────────────────────────────────────────────────────────────────────────────────────────

/// AC 7 — the self-heal. This is the case the task's own §3 skeleton would have MISSED: it put the rule in the
/// JIT upsert's ON CONFLICT clause, which a returning SSO user never reaches, because `resolve_subject`
/// short-circuits on the existing-link fast path. Everyone damaged today is a returning user. The rule has to
/// live somewhere that runs on every path, and this test is what proves it does.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn a_returning_user_whose_name_is_their_email_is_repaired() {
    let pool = pool().await;
    let tenant = seed_tenant(&pool).await;

    // Exactly the shape production has today.
    let email = "van-anh.vu@cyberskill.world";
    let s = seed_subject(&pool, tenant, email, email).await;

    display_name::heal(&pool, tenant, s, "Vũ Vân Anh")
        .await
        .expect("heal");

    // Repaired — and byte-identical. A "helpful" transformation appearing in the resolver later fails here.
    assert_eq!(
        display_name_of(&pool, tenant, s).await.as_deref(),
        Some("Vũ Vân Anh")
    );
}

/// AC 8 — the no-clobber guarantee, and the reason §1 #4 is a MUST rather than a nicety.
///
/// `PATCH /v1/auth/me` ships TODAY: a person can set their own display name. So can an administrator, by
/// hand — which is exactly what was done to make a Play Store screenshot presentable. The naive fix (always
/// refresh from the ID token on login) would silently revert both, on the next sign-in, with no error and no
/// trace. That is a worse bug than the one being fixed, because it is invisible and it undoes a human's
/// decision.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn a_deliberately_set_name_is_never_clobbered() {
    let pool = pool().await;
    let tenant = seed_tenant(&pool).await;

    // Someone put this here on purpose. It differs from the email, so it was never the bug's doing.
    let s = seed_subject(&pool, tenant, "Play Review", "play-review@cyberskill.world").await;

    // The IdP would resolve them to something else entirely. It does not get a vote.
    display_name::heal(&pool, tenant, s, "play-review")
        .await
        .expect("heal");

    assert_eq!(
        display_name_of(&pool, tenant, s).await.as_deref(),
        Some("Play Review"),
        "a display_name that differs from the email was set deliberately and must survive sign-in"
    );
}

/// AC 9 — a blank resolution must never blank a stored name. A person who renders as nothing at all is worse
/// off than one who renders as their email address, so the failure mode has to be inert, not destructive.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn a_blank_resolution_never_blanks_a_stored_name() {
    let pool = pool().await;
    let tenant = seed_tenant(&pool).await;

    let email = "a.b@c.com";
    let damaged = seed_subject(&pool, tenant, email, email).await;
    let named = seed_subject(&pool, tenant, "Trịnh Thái Anh", "t.a@c.com").await;

    for blank in ["", "   ", "\t\n"] {
        display_name::heal(&pool, tenant, damaged, blank)
            .await
            .expect("heal");
        display_name::heal(&pool, tenant, named, blank)
            .await
            .expect("heal");
    }

    // The damaged row is still damaged — which is correct. It is not repaired, but nor is it made worse, and
    // it will repair itself the moment a sign-in carries a real name.
    assert_eq!(
        display_name_of(&pool, tenant, damaged).await.as_deref(),
        Some(email)
    );
    assert_eq!(
        display_name_of(&pool, tenant, named).await.as_deref(),
        Some("Trịnh Thái Anh")
    );
}

/// AC 6 — belt and braces. The resolver already refuses to yield a full email address on any rung, but the
/// heal is the last gate before the column, and this is the exact bug we are removing. Pin it at both ends.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn the_email_is_never_written_back_as_a_name() {
    let pool = pool().await;
    let tenant = seed_tenant(&pool).await;

    let email = "van-anh.vu@cyberskill.world";
    let s = seed_subject(&pool, tenant, email, email).await;

    // What the chain actually produces when the ID token carries no name claims at all.
    let profile = display_name::Profile::default();
    let (rung, resolved) = display_name::resolve(&profile, Some(email));
    assert_eq!(rung, "email_local_part");

    display_name::heal(&pool, tenant, s, &resolved)
        .await
        .expect("heal");

    let stored = display_name_of(&pool, tenant, s).await.unwrap();
    assert_eq!(stored, "van-anh.vu");
    assert_ne!(stored, email, "the bug: display_name bound from the email");
}

/// AC 13 — the view drains. A self-healing fix is invisible: without this, there is no way to know whether
/// the repair is working or how many people are still affected. The view makes the damage countable.
#[tokio::test]
#[ignore = "requires Postgres (DATABASE_URL)"]
async fn the_visibility_view_drains() {
    let pool = pool().await;
    let tenant = seed_tenant(&pool).await;

    let email = "a.b@c.com";
    let s = seed_subject(&pool, tenant, email, email).await;
    assert_eq!(
        unset_rows_for(&pool, tenant).await,
        1,
        "a subject wearing their email as a name must be visible in the view"
    );

    display_name::heal(&pool, tenant, s, "A B")
        .await
        .expect("heal");

    assert_eq!(
        unset_rows_for(&pool, tenant).await,
        0,
        "and must leave it once repaired — this is how we watch the fix drain to zero"
    );
}
