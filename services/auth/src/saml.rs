//! TASK-AUTH-103 — SAML 2.0 SSO (SP-initiated flow).
//!
//! Three endpoints under the public router (no JWT required to initiate):
//!   * `GET  /v1/auth/saml/initiate`        — generate AuthnRequest, 302 to IdP
//!   * `POST /v1/auth/saml/acs`             — Assertion Consumer Service; IdP POSTs the SAMLResponse
//!   * `GET  /v1/auth/saml/idp-configs/{id}/sp-metadata` — publishes the SP metadata XML
//!
//! Plus one admin endpoint to create/update IdP config:
//!   * `POST /v1/admin/saml/idp-configs`
//!
//! Slice 1 shipped a working SP-initiated flow with structural validation
//! (RequestID round-trip, audience check, NotOnOrAfter check, idempotency
//! via consumed_at) and a TODO marker for signature verification.
//!
//! Slice 2 (2026-05-18) wires cryptographic XML-DSig verification via the
//! sibling `saml_sig` module — RSA-SHA256 over exclusive-c14n SignedInfo +
//! SHA-256 reference digest. The legacy `AUTH_SAML_ALLOW_UNSIGNED=1` env-var
//! escape hatch is removed; per-IdP fail-open lives in the new
//! `saml_idp_configs.allow_unsigned` column (default FALSE — production safe).

use axum::{
    extract::{Json as JsonInput, Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Json, Response},
    Extension,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::jwt::Claims;
use crate::AppState;

const REQUEST_ID_PREFIX: &str = "_cyberos-";

// ---------------------------------------------------------------------------
// Public — SP-initiated AuthnRequest
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct InitiateQuery {
    pub tenant_slug: String,
    pub idp: String,
    pub relay_state: Option<String>,
}

pub async fn initiate(
    State(state): State<AppState>,
    Query(q): Query<InitiateQuery>,
) -> Result<Response, (StatusCode, Json<Value>)> {
    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000'")
        .execute(&mut *tx)
        .await
        .map_err(internal)?;

    let row: Option<(Uuid, Uuid, String, String, String, String)> = sqlx::query_as(
        "SELECT i.id, i.tenant_id, i.sso_url, i.issuer, i.sp_entity_id, i.sp_acs_url
             FROM saml_idp_configs i
             JOIN tenants t ON t.id = i.tenant_id
            WHERE t.slug = $1 AND i.name = $2 AND i.status = 'active'",
    )
    .bind(&q.tenant_slug)
    .bind(&q.idp)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal)?;
    tx.commit().await.map_err(internal)?;

    let (idp_id, tenant_id, sso_url, _idp_issuer, sp_entity_id, sp_acs_url) =
        row.ok_or_else(|| (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "saml_idp_not_found", "tenant_slug": q.tenant_slug, "idp": q.idp})),
        ))?;

    // Build the AuthnRequest XML.
    let request_id = format!("{REQUEST_ID_PREFIX}{}", Uuid::new_v4().simple());
    let issue_instant = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let authn_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol"
  xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion"
  ID="{request_id}"
  Version="2.0"
  IssueInstant="{issue_instant}"
  Destination="{sso}"
  ProtocolBinding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
  AssertionConsumerServiceURL="{acs}">
  <saml:Issuer>{issuer}</saml:Issuer>
  <samlp:NameIDPolicy Format="urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress" AllowCreate="true"/>
</samlp:AuthnRequest>"#,
        request_id = request_id,
        issue_instant = issue_instant,
        sso = xml_escape(&sso_url),
        acs = xml_escape(&sp_acs_url),
        issuer = xml_escape(&sp_entity_id),
    );

    // Persist the request_id for ACS correlation.
    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    sqlx::query(
        "INSERT INTO saml_authn_request_log (request_id, tenant_id, idp_config_id, relay_state)
              VALUES ($1, $2, $3, $4)",
    )
    .bind(&request_id)
    .bind(tenant_id)
    .bind(idp_id)
    .bind(q.relay_state.as_deref())
    .execute(&mut *tx)
    .await
    .map_err(internal)?;
    tx.commit().await.map_err(internal)?;

    // Two binding options:
    //   HTTP-Redirect: SAMLRequest base64(deflated XML) in query string + 302.
    //   HTTP-POST:    auto-submitting <form> with SAMLRequest base64(XML).
    // Slice 1 ships HTTP-POST because it works without DEFLATE (and the deflate
    // crate adds dependency surface). HTTP-Redirect lands in slice 2.
    let saml_req_b64 = STANDARD.encode(authn_xml.as_bytes());
    let relay = q.relay_state.unwrap_or_default();

    let form = format!(
        r#"<!doctype html><html><head><meta charset="utf-8"><title>Redirecting to SSO…</title></head>
<body onload="document.forms[0].submit()">
  <noscript><p>JavaScript is disabled. Click submit to continue to your SSO provider.</p></noscript>
  <form method="POST" action="{sso}">
    <input type="hidden" name="SAMLRequest" value="{req}"/>
    <input type="hidden" name="RelayState" value="{relay}"/>
    <noscript><button type="submit">Continue</button></noscript>
  </form>
</body></html>"#,
        sso = html_escape(&sso_url),
        req = html_escape(&saml_req_b64),
        relay = html_escape(&relay),
    );

    Ok(Html(form).into_response())
}

// ---------------------------------------------------------------------------
// Public — ACS (Assertion Consumer Service)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AcsBody {
    #[serde(rename = "SAMLResponse")]
    pub saml_response: String,
    #[serde(rename = "RelayState")]
    pub relay_state: Option<String>,
}

pub async fn acs(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    axum::Form(body): axum::Form<AcsBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let caller_ip = crate::handlers::caller_ip(&headers);
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    // Decode the base64 SAMLResponse.
    let xml_bytes = STANDARD
        .decode(&body.saml_response)
        .map_err(|e| bad_req("saml_response_decode_failed", &e.to_string()))?;
    let xml = String::from_utf8(xml_bytes)
        .map_err(|e| bad_req("saml_response_not_utf8", &e.to_string()))?;

    // Extract the fields we care about with simple XML pattern matching.
    // Full XML c14n + signature verify is slice 2.
    let request_id = extract_xml_attr(&xml, "InResponseTo").ok_or_else(|| {
        bad_req(
            "missing_inresponseto",
            "SAMLResponse must carry InResponseTo",
        )
    })?;
    let assertion_issuer = extract_xml_element(&xml, "saml:Issuer")
        .or_else(|| extract_xml_element(&xml, "Issuer"))
        .ok_or_else(|| bad_req("missing_issuer", "SAMLResponse must carry Issuer"))?;
    let name_id = extract_xml_element(&xml, "saml:NameID")
        .or_else(|| extract_xml_element(&xml, "NameID"))
        .ok_or_else(|| bad_req("missing_nameid", "SAMLResponse Subject must carry NameID"))?;
    let email_attr = extract_saml_attribute(&xml, "email")
        .or_else(|| extract_saml_attribute(&xml, "Email"))
        .or_else(|| extract_saml_attribute(&xml, "mail"));

    // TASK-AUTH-111 §1 #6 — SAML carried the identical defect (it bound the email into display_name too), so
    // it is fixed in the same change with the same chain. Fixing one door and not the other guarantees the
    // bug is rediscovered through the other.
    //
    // SAML has no single agreed spelling for these, so map the common ones onto the OIDC claim shape and let
    // ONE resolver decide. The IdP-specific spellings live here; the policy lives in display_name.rs.
    // `http://schemas.xmlsoap.org/...` are the AD FS / Entra names; the bare ones are what most others emit.
    let saml_profile = crate::display_name::Profile {
        name: extract_saml_attribute(&xml, "displayName")
            .or_else(|| extract_saml_attribute(&xml, "display_name"))
            .or_else(|| extract_saml_attribute(&xml, "cn"))
            .or_else(|| {
                extract_saml_attribute(
                    &xml,
                    "http://schemas.microsoft.com/identity/claims/displayname",
                )
            }),
        given_name: extract_saml_attribute(&xml, "givenName").or_else(|| {
            extract_saml_attribute(
                &xml,
                "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/givenname",
            )
        }),
        family_name: extract_saml_attribute(&xml, "sn")
            .or_else(|| extract_saml_attribute(&xml, "surname"))
            .or_else(|| {
                extract_saml_attribute(
                    &xml,
                    "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/surname",
                )
            }),
        preferred_username: extract_saml_attribute(&xml, "uid")
            .or_else(|| extract_saml_attribute(&xml, "preferred_username")),
    };
    let (name_rung, saml_display_name) =
        crate::display_name::resolve(&saml_profile, email_attr.as_deref());
    // The rung, never the name (§1 #8).
    tracing::debug!(target: "cyberos_auth::saml", rung = name_rung, "resolved display_name");

    // Look up the AuthnRequest we issued.
    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000'")
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    let req_row: Option<(Uuid, Uuid, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(
        "SELECT tenant_id, idp_config_id, consumed_at
             FROM saml_authn_request_log
            WHERE request_id = $1 AND expires_at > NOW()",
    )
    .bind(&request_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal)?;
    let (tenant_id, idp_id, consumed_at) = req_row.ok_or_else(|| {
        bad_req(
            "unknown_or_expired_request_id",
            "no matching AuthnRequest within 10-min window",
        )
    })?;
    if consumed_at.is_some() {
        return Err(audit_failure(
            &state,
            tenant_id,
            idp_id,
            &request_id,
            "replay",
            "request_id consumed",
        )
        .await);
    }

    // Load IdP config for issuer + signature validation.
    // Slice-2 (TASK-AUTH-103 hardening): `allow_unsigned` column replaces the
    // legacy `AUTH_SAML_ALLOW_UNSIGNED=1` env-var escape hatch. Default FALSE
    // — operators must explicitly opt-in per IdP. Production deploys ship
    // FALSE everywhere; dev fixtures may set TRUE on the fixture IdP.
    let idp_row: (String, String, String, bool, Vec<String>, bool) = sqlx::query_as(
        "SELECT issuer, signing_cert_pem, sp_entity_id, auto_provision, default_roles, allow_unsigned
             FROM saml_idp_configs WHERE id = $1",
    )
    .bind(idp_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal)?;
    let (idp_issuer, signing_cert_pem, sp_entity_id, auto_provision, default_roles, allow_unsigned) =
        idp_row;

    if assertion_issuer != idp_issuer {
        return Err(audit_failure(
            &state,
            tenant_id,
            idp_id,
            &request_id,
            "audience_mismatch",
            &format!("assertion Issuer {assertion_issuer} ≠ IdP {idp_issuer}"),
        )
        .await);
    }

    // Audience check: the assertion's <AudienceRestriction>/<Audience> MUST match our sp_entity_id.
    let audience = extract_xml_element(&xml, "saml:Audience")
        .or_else(|| extract_xml_element(&xml, "Audience"));
    if let Some(aud) = audience {
        if aud != sp_entity_id {
            return Err(audit_failure(
                &state,
                tenant_id,
                idp_id,
                &request_id,
                "audience_mismatch",
                &format!("Audience {aud} ≠ SP {sp_entity_id}"),
            )
            .await);
        }
    }

    // TASK-AUTH-103 slice-2 — verify the <ds:Signature> with the IdP's configured
    // signing cert. The verifier supports RSA-SHA256 + SHA-256 + exclusive-c14n;
    // these are the algorithms produced by every modern SAML 2.0 IdP. If the
    // verifier fails (or no Signature element is present) we surface a
    // structured audit row and only proceed when `allow_unsigned` is set on
    // this particular IdP config.
    match crate::saml_sig::verify(&xml, &signing_cert_pem) {
        Ok(ok) => {
            tracing::info!(signed_id = %ok.signed_id, idp = %idp_id, "SAML signature verified");
        }
        Err(e) => {
            if !allow_unsigned {
                return Err(audit_failure(
                    &state,
                    tenant_id,
                    idp_id,
                    &request_id,
                    "sig_invalid",
                    &format!("xml-sig verify failed: {e}"),
                )
                .await);
            }
            tracing::warn!(
                idp = %idp_id,
                error = %e,
                "SAML signature verify failed but allow_unsigned=TRUE for this IdP — accepting (dev path)"
            );
        }
    }

    // Mark the AuthnRequest consumed.
    sqlx::query("UPDATE saml_authn_request_log SET consumed_at = NOW() WHERE request_id = $1")
        .bind(&request_id)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    tx.commit().await.map_err(internal)?;

    // Resolve subject (existing link or JIT-provision).
    let subject_id = resolve_subject(
        &state,
        tenant_id,
        idp_id,
        &name_id,
        email_attr.as_deref(),
        auto_provision,
        &default_roles,
        &saml_display_name,
    )
    .await?;

    // TASK-AUTH-111 §1 #4 + #5 — same repair as OIDC, on every path, for the same reason: the existing-link
    // fast path returns before any INSERT, so a rule living in the upsert would never reach an already
    // provisioned person. A failed heal must not fail the login.
    if let Err(e) =
        crate::display_name::heal(&state.pg, tenant_id, subject_id, &saml_display_name).await
    {
        tracing::warn!(target: "cyberos_auth::saml", error = %e, "display_name heal failed; login proceeds");
    }

    // Audit success.
    let _ = sqlx::query(
        "INSERT INTO saml_login_history (tenant_id, idp_config_id, request_id, subject_id, outcome)
              VALUES ($1, $2, $3, $4, 'success')",
    )
    .bind(tenant_id)
    .bind(idp_id)
    .bind(&request_id)
    .bind(subject_id)
    .execute(&state.pg)
    .await;

    // Mint a CyberOS JWT.
    let svc = crate::jwt::JwtService::new(state.pg.clone(), state.jwt_issuer.clone());
    let roles = crate::handlers::load_subject_roles_pub(&state, tenant_id, subject_id, &[]).await;
    let rbac_v = state.role_matrix.read().await.version();
    let tokens = svc
        .issue(
            cyberos_types::TenantId(tenant_id),
            cyberos_types::SubjectId(subject_id),
            "", // TASK-AUTH-004 §1 #2 — SAML callback doesn't pass plaintext email through
            "human",
            vec![],
            roles,
            Some(rbac_v),
            None,
            None,
        )
        .await
        .map_err(|e| internal(e))?;

    // TASK-AUTH-106 slice-3 — apply policy-aware impossible-travel detection.
    let deps = crate::travel::AssessDeps {
        pool: &state.pg,
        geoip: &state.geoip,
        policy_cache: &state.travel_policy,
        sticky_suppress: &state.sticky_suppress,
    };
    let outcome = crate::travel::assess_login(
        &deps,
        tenant_id,
        subject_id,
        "saml",
        caller_ip,
        user_agent.as_deref(),
    )
    .await
    .ok();
    match outcome {
        Some(crate::travel::TravelOutcome::Block { kind, .. }) => Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "impossible_travel_blocked", "kind": kind})),
        )),
        Some(crate::travel::TravelOutcome::Challenge { kind, login_id, .. }) => Ok(Json(json!({
            "access_token": tokens.access_token,
            "refresh_token": tokens.refresh_token,
            "token_type": tokens.token_type,
            "expires_in": tokens.expires_in,
            "subject_id": subject_id,
            "relay_state": body.relay_state,
            "needs_mfa_challenge": true,
            "challenge_reason": kind,
            "challenge_login_id": login_id,
        }))),
        _ => Ok(Json(json!({
            "access_token": tokens.access_token,
            "refresh_token": tokens.refresh_token,
            "token_type": tokens.token_type,
            "expires_in": tokens.expires_in,
            "subject_id": subject_id,
            "relay_state": body.relay_state,
        }))),
    }
}

// ---------------------------------------------------------------------------
// SP metadata publication
// ---------------------------------------------------------------------------

pub async fn sp_metadata(
    State(state): State<AppState>,
    Path(idp_config_id): Path<Uuid>,
) -> Result<Response, (StatusCode, Json<Value>)> {
    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000'")
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT sp_entity_id, sp_acs_url FROM saml_idp_configs WHERE id = $1")
            .bind(idp_config_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(internal)?;
    tx.commit().await.map_err(internal)?;
    let (sp_entity_id, sp_acs_url) = row.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "idp_config_not_found"})),
        )
    })?;

    let metadata = format!(
        r#"<?xml version="1.0"?>
<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata"
  entityID="{eid}">
  <md:SPSSODescriptor protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol"
    AuthnRequestsSigned="false" WantAssertionsSigned="true">
    <md:NameIDFormat>urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress</md:NameIDFormat>
    <md:AssertionConsumerService
      Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
      Location="{acs}"
      index="0"/>
  </md:SPSSODescriptor>
</md:EntityDescriptor>"#,
        eid = xml_escape(&sp_entity_id),
        acs = xml_escape(&sp_acs_url),
    );

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/samlmetadata+xml")],
        metadata,
    )
        .into_response())
}

// ---------------------------------------------------------------------------
// Admin — create/update IdP config
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateIdpConfigBody {
    pub name: String,
    pub sso_url: String,
    pub slo_url: Option<String>,
    pub issuer: String,
    pub signing_cert_pem: String,
    pub sp_entity_id: String,
    pub sp_acs_url: String,
    pub auto_provision: Option<bool>,
    pub default_roles: Option<Vec<String>>,
}

pub async fn create_idp_config(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    JsonInput(body): JsonInput<CreateIdpConfigBody>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id).map_err(internal)?;
    let auto = body.auto_provision.unwrap_or(true);
    let default_roles = body
        .default_roles
        .unwrap_or_else(|| vec!["tenant-member".into()]);

    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal)?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO saml_idp_configs
              (tenant_id, name, sso_url, slo_url, issuer, signing_cert_pem,
               sp_entity_id, sp_acs_url, auto_provision, default_roles)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT (tenant_id, name) DO UPDATE
            SET sso_url           = EXCLUDED.sso_url,
                slo_url           = EXCLUDED.slo_url,
                issuer            = EXCLUDED.issuer,
                signing_cert_pem  = EXCLUDED.signing_cert_pem,
                sp_entity_id      = EXCLUDED.sp_entity_id,
                sp_acs_url        = EXCLUDED.sp_acs_url,
                auto_provision    = EXCLUDED.auto_provision,
                default_roles     = EXCLUDED.default_roles,
                updated_at        = NOW()
       RETURNING id",
    )
    .bind(tenant_id)
    .bind(&body.name)
    .bind(&body.sso_url)
    .bind(body.slo_url.as_deref())
    .bind(&body.issuer)
    .bind(&body.signing_cert_pem)
    .bind(&body.sp_entity_id)
    .bind(&body.sp_acs_url)
    .bind(auto)
    .bind(&default_roles)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal)?;
    tx.commit().await.map_err(internal)?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "id": row.0,
            "tenant_id": tenant_id,
            "name": body.name,
        })),
    ))
}

// ---------------------------------------------------------------------------
// Helpers — XML extraction (slice 1: structural; slice 2 swaps for xml-rs)
// ---------------------------------------------------------------------------

fn extract_xml_attr(xml: &str, attr_name: &str) -> Option<String> {
    // Naive: find `attr_name="..."` anywhere. Acceptable for IDs / well-known attrs.
    let needle = format!("{attr_name}=\"");
    let i = xml.find(&needle)?;
    let after = &xml[i + needle.len()..];
    let end = after.find('"')?;
    Some(after[..end].to_string())
}

fn extract_xml_element(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let i = xml.find(&open)?;
    // Skip until the closing `>` of the open tag.
    let after_open = xml[i..].find('>')? + i + 1;
    let end = xml[after_open..].find(&close)? + after_open;
    Some(xml[after_open..end].trim().to_string())
}

fn extract_saml_attribute(xml: &str, attr_name: &str) -> Option<String> {
    // <saml:Attribute Name="email">...<saml:AttributeValue>VALUE</saml:AttributeValue>...
    let marker = format!(r#"Name="{attr_name}""#);
    let i = xml.find(&marker)?;
    let rest = &xml[i..];
    let av_open = rest
        .find("<saml:AttributeValue")
        .or_else(|| rest.find("<AttributeValue"))?;
    let after_open = rest[av_open..].find('>')? + av_open + 1;
    let close = rest[after_open..].find("</")?;
    Some(rest[after_open..after_open + close].trim().to_string())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn html_escape(s: &str) -> String {
    xml_escape(s).replace('\'', "&#39;")
}

// TASK-AUTH-111: `display_name` is resolved by the caller from the assertion's name attributes, via the one
// shared chain in display_name.rs. Passed in rather than derived here, so OIDC and SAML cannot drift.
#[allow(clippy::too_many_arguments)]
async fn resolve_subject(
    state: &AppState,
    tenant_id: Uuid,
    idp_id: Uuid,
    name_id: &str,
    email: Option<&str>,
    auto_provision: bool,
    default_roles: &[String],
    display_name: &str,
) -> Result<Uuid, (StatusCode, Json<Value>)> {
    // Existing link?
    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    let existing: Option<(Uuid,)> = sqlx::query_as(
        "SELECT subject_id FROM saml_subject_link
          WHERE idp_config_id = $1 AND idp_name_id = $2",
    )
    .bind(idp_id)
    .bind(name_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal)?;
    if let Some((sid,)) = existing {
        let _ = sqlx::query(
            "UPDATE saml_subject_link SET last_login_at = NOW()
              WHERE idp_config_id = $1 AND idp_name_id = $2",
        )
        .bind(idp_id)
        .bind(name_id)
        .execute(&mut *tx)
        .await;
        tx.commit().await.map_err(internal)?;
        return Ok(sid);
    }
    tx.commit().await.map_err(internal)?;

    if !auto_provision {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "subject_not_provisioned"})),
        ));
    }

    let handle = match email {
        Some(e) => format!(
            "@{}",
            e.split('@')
                .next()
                .unwrap_or(&name_id[..name_id.len().min(20)])
        ),
        None => format!("@saml-{}", &name_id[..name_id.len().min(12)]),
    };
    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    // TASK-AUTH-111 — was `email.unwrap_or("")`. Same bug as OIDC, same fix, one resolver (§1 #6). The
    // ON CONFLICT path deliberately does not touch display_name; `display_name::heal` owns that rule.
    let jit_display_name = if display_name.trim().is_empty() {
        handle.as_str()
    } else {
        display_name
    };
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO subjects (tenant_id, handle, display_name, email, kind, status, roles)
              VALUES ($1, $2, $3, $4, 'human', 'active', $5)
         ON CONFLICT (tenant_id, handle) DO UPDATE
            SET email = COALESCE(EXCLUDED.email, subjects.email),
                updated_at = NOW()
       RETURNING id",
    )
    .bind(tenant_id)
    .bind(&handle)
    .bind(jit_display_name)
    .bind(email)
    .bind(default_roles)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal)?;
    let subject_id = row.0;
    sqlx::query(
        "INSERT INTO saml_subject_link (tenant_id, subject_id, idp_config_id, idp_name_id, idp_email)
              VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT DO NOTHING",
    )
    .bind(tenant_id)
    .bind(subject_id)
    .bind(idp_id)
    .bind(name_id)
    .bind(email)
    .execute(&mut *tx).await.map_err(internal)?;
    tx.commit().await.map_err(internal)?;
    Ok(subject_id)
}

async fn audit_failure(
    state: &AppState,
    tenant_id: Uuid,
    idp_id: Uuid,
    request_id: &str,
    outcome: &str,
    detail: &str,
) -> (StatusCode, Json<Value>) {
    let _ = sqlx::query(
        "INSERT INTO saml_login_history (tenant_id, idp_config_id, request_id, outcome, detail)
              VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(tenant_id)
    .bind(idp_id)
    .bind(request_id)
    .bind(outcome)
    .bind(detail)
    .execute(&state.pg)
    .await;
    bad_req(outcome, detail)
}

fn bad_req(err: &str, detail: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({"error": err, "detail": detail})),
    )
}

fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": e.to_string()})),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_xml_attr_finds_quoted_value() {
        let xml = r#"<root InResponseTo="_xyz-123" Foo="bar"/>"#;
        assert_eq!(
            extract_xml_attr(xml, "InResponseTo"),
            Some("_xyz-123".to_string())
        );
        assert_eq!(extract_xml_attr(xml, "Foo"), Some("bar".to_string()));
        assert_eq!(extract_xml_attr(xml, "Missing"), None);
    }

    #[test]
    fn extract_xml_element_finds_inner_text() {
        let xml = r#"<root><saml:Issuer>https://idp.example.com</saml:Issuer></root>"#;
        assert_eq!(
            extract_xml_element(xml, "saml:Issuer"),
            Some("https://idp.example.com".to_string()),
        );
    }

    #[test]
    fn extract_saml_attribute_picks_email() {
        let xml = r#"
<saml:Attribute Name="email">
  <saml:AttributeValue>alice@example.com</saml:AttributeValue>
</saml:Attribute>"#;
        assert_eq!(
            extract_saml_attribute(xml, "email"),
            Some("alice@example.com".to_string())
        );
    }

    #[test]
    fn xml_escape_handles_specials() {
        assert_eq!(
            xml_escape("a & b < c > d \" e"),
            "a &amp; b &lt; c &gt; d &quot; e"
        );
    }
}
