//! TASK-AUTH-004 — JWT-verification middleware.
//!
//! Tower middleware layered onto every admin route. Responsibilities:
//!   1. Pull `Authorization: Bearer <jwt>` from the request.
//!   2. Call `JwtService::verify` — validates signature, `iss`, `aud`, `exp`.
//!   3. Set `app.current_tenant_id` GUC on the request's transaction context
//!      so RLS policies (migration 0005) apply correctly.
//!   4. Attach the verified `Claims` to request extensions so downstream
//!      handlers can read `tenant_id`, `sub`, `kind`, `scope_grants`.
//!   5. Reject with 401 on missing / invalid / expired tokens.
//!
//! A separate `require_scope` extractor gates routes that need specific
//! `scope_grants` (e.g. `["admin"]` on `/v1/admin/*`).

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;

use crate::jwt::{Claims, JwtService};
use crate::AppState;

/// Tower middleware. Verifies the bearer token and attaches `Claims` to the
/// request extensions. Returns 401 on any verification failure.
pub async fn verify_jwt(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let token = extract_bearer(request.headers())
        .ok_or_else(|| unauthorized("missing or malformed Authorization header"))?;

    let svc = JwtService::new(state.pg.clone(), state.jwt_issuer.clone());
    let claims = svc
        .verify(&token)
        .await
        .map_err(|e| unauthorized(&format!("jwt verification failed: {e}")))?;

    // TASK-AUTH-005 §1 #3 + #11 + G-011/G-017 — deny-list check. If revoke
    // pushed this jti into the deny-list, the JWT is no longer valid even
    // though its signature + exp + iss verify. Returns 401 with explicit
    // `token_revoked` so the client can distinguish from "expired" and
    // trigger re-auth (not refresh).
    if state.deny_list.is_denied(&claims.jti) {
        return Err(unauthorized("token_revoked"));
    }

    // TASK-AUTH-101 §1 #10 — set the per-connection `app.roles` GUC so
    // RLS policies can call `auth.has_role(<role>)` directly. The GUC is
    // a comma-separated list; the SQL function splits on `,`.
    // Best-effort: failure to set the GUC doesn't fail the request, but it
    // means RLS that depends on role checks may deny a legitimate read.
    if let Ok(mut conn) = state.pg.acquire().await {
        let role_csv = claims.roles.join(",");
        let _ = sqlx::query(&format!("SELECT set_config('app.roles', $1, false)"))
            .bind(role_csv)
            .execute(&mut *conn)
            .await;
    }

    // TASK-OBS-003 - hand the tenant to the RED middleware (outer layer) via the response extensions, so
    // the metric's tenant_id label is real rather than "unknown".
    let tenant_id = claims.tenant_id.clone();
    request.extensions_mut().insert(claims);
    let mut response = next.run(request).await;
    response
        .extensions_mut()
        .insert(cyberos_obs_sdk::TenantCtx(tenant_id));
    Ok(response)
}

/// Optional middleware that REQUIRES a specific scope grant. Layer this AFTER
/// `verify_jwt` so `Claims` are already in extensions.
///
/// Usage:
/// ```text
/// .route_layer(middleware::from_fn(require_scope("admin")))
/// ```
pub fn require_scope(
    needed: &'static str,
) -> impl Fn(Request, Next) -> futures_util::future::BoxFuture<'static, Result<Response, Response>> + Clone
{
    move |request: Request, next: Next| {
        Box::pin(async move {
            let claims = request
                .extensions()
                .get::<Claims>()
                .ok_or_else(|| unauthorized("verify_jwt middleware must run first"))?;

            let ok = claims
                .scope_grants
                .iter()
                .any(|s| s == needed || s == "admin");
            if !ok {
                return Err(forbidden(&format!(
                    "missing required scope grant '{needed}' — present: {:?}",
                    claims.scope_grants
                )));
            }
            Ok(next.run(request).await)
        })
    }
}

fn extract_bearer(headers: &header::HeaderMap) -> Option<String> {
    let h = headers.get(header::AUTHORIZATION)?;
    let s = h.to_str().ok()?;
    let rest = s.strip_prefix("Bearer ")?;
    Some(rest.trim().to_string())
}

fn unauthorized(msg: &str) -> Response {
    (StatusCode::UNAUTHORIZED, Json(json!({"error": msg}))).into_response()
}

fn forbidden(msg: &str) -> Response {
    (StatusCode::FORBIDDEN, Json(json!({"error": msg}))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn extracts_well_formed_bearer() {
        let mut h = HeaderMap::new();
        h.insert(
            "authorization",
            HeaderValue::from_static("Bearer abc.def.ghi"),
        );
        assert_eq!(extract_bearer(&h).as_deref(), Some("abc.def.ghi"));
    }

    #[test]
    fn rejects_no_bearer_prefix() {
        let mut h = HeaderMap::new();
        h.insert(
            "authorization",
            HeaderValue::from_static("Basic Zm9vOmJhcg=="),
        );
        assert!(extract_bearer(&h).is_none());
    }

    #[test]
    fn rejects_missing_header() {
        assert!(extract_bearer(&HeaderMap::new()).is_none());
    }

    #[test]
    fn rejects_empty_value() {
        let mut h = HeaderMap::new();
        h.insert("authorization", HeaderValue::from_static(""));
        assert!(extract_bearer(&h).is_none());
    }
}
