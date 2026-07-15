//! TASK-EVAL-002 rubric domain types and write-time validation.
//!
//! The five closed enums (`SourceDoc`, `ItemKind`, `ObligationKind`, `CheckType`, `VersionState`) mirror
//! the migration's CHECK constraints exactly, so the Rust type system and the database agree on the legal
//! values. Each derives `sqlx::Type` over the underlying `text` column in the same style as
//! `cyberos_proj::types` (a `type_name = "text"` enum plus a manual `as_str()` for audit payloads and OTel
//! labels). [`validate_item`] is the single write-time gate every authoring path runs before an item may be
//! inserted into a draft version: it rejects an uncited item (§1 #2), a missing Vietnamese title (§1 #5), an
//! obligation with no obligation_kind (§1 #3), and a check_params shape that does not match its check_type
//! (§1 #4). It performs NO scoring and reads NO model - it only checks that a human-authored standard cites
//! a real clause and is shaped for TASK-EVAL-003 to evaluate.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The closed set of the three signed employment documents (DEC-2600). A rubric item MUST cite exactly one
/// of these and nothing else; the migration's `source_doc` CHECK is the database-side mirror of this enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SourceDoc {
    /// "1. Hợp Đồng Lao Động - Labor Contract".
    LaborContract,
    /// "2. Thoả Thuận Bảo Mật, Không Cạnh Tranh & Sở Hữu Trí Tuệ - NDNCA & IP".
    NdaIp,
    /// "3. Phụ Lục Đãi Ngộ Toàn Diện & Lộ Trình Phát Triển - Total Rewards & Career Path Appendix".
    TotalRewards,
}

impl SourceDoc {
    pub fn as_str(self) -> &'static str {
        match self {
            SourceDoc::LaborContract => "labor_contract",
            SourceDoc::NdaIp => "nda_ip",
            SourceDoc::TotalRewards => "total_rewards",
        }
    }
}

/// What kind of thing an item checks (§1 #3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    Obligation,
    WorkingTerm,
    Kpi,
    CareerMilestone,
}

impl ItemKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ItemKind::Obligation => "obligation",
            ItemKind::WorkingTerm => "working_term",
            ItemKind::Kpi => "kpi",
            ItemKind::CareerMilestone => "career_milestone",
        }
    }
}

/// The three NDA obligation families called out in the brain-evaluation plan (§1 #3). Required for an
/// `obligation` item, null otherwise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ObligationKind {
    Confidentiality,
    NonCompete,
    IpAssignment,
}

impl ObligationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ObligationKind::Confidentiality => "confidentiality",
            ObligationKind::NonCompete => "non_compete",
            ObligationKind::IpAssignment => "ip_assignment",
        }
    }
}

/// How an item is evaluated (§1 #4). A small closed set keeps TASK-EVAL-003's evaluation logic bounded: is
/// there evidence of X; is a number at/above a threshold; is there a signed attestation; was a review done
/// on cadence; was a milestone reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CheckType {
    EvidencePresence,
    ThresholdNumeric,
    Attestation,
    PeriodicReview,
    MilestoneReached,
}

impl CheckType {
    pub fn as_str(self) -> &'static str {
        match self {
            CheckType::EvidencePresence => "evidence_presence",
            CheckType::ThresholdNumeric => "threshold_numeric",
            CheckType::Attestation => "attestation",
            CheckType::PeriodicReview => "periodic_review",
            CheckType::MilestoneReached => "milestone_reached",
        }
    }
}

/// A version's lifecycle state (§1 #6). draft -> approved -> published -> superseded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum VersionState {
    Draft,
    Approved,
    Published,
    Superseded,
}

impl VersionState {
    pub fn as_str(self) -> &'static str {
        match self {
            VersionState::Draft => "draft",
            VersionState::Approved => "approved",
            VersionState::Published => "published",
            VersionState::Superseded => "superseded",
        }
    }
}

/// A rubric framework row (§1 #1).
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Rubric {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: Uuid,
}

/// One effective-dated cut of a rubric (§1 #6 #7).
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct RubricVersion {
    pub id: Uuid,
    pub rubric_id: Uuid,
    pub tenant_id: Uuid,
    pub version_no: i32,
    pub state: VersionState,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub approver_subject_id: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub published_by_subject_id: Option<Uuid>,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: Uuid,
}

/// One persisted, clause-cited item within a version (§1 #2). Every field a reviewer or TASK-EVAL-003 needs:
/// the citation, the classification, the machine-usable check descriptor, the bilingual text, and the
/// provenance. No per-employee / score / evidence field exists by design (§1 #14).
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct RubricItem {
    pub id: Uuid,
    pub rubric_version_id: Uuid,
    pub tenant_id: Uuid,
    pub source_doc: SourceDoc,
    pub clause_ref: String,
    pub source_quote_vi: Option<String>,
    pub source_quote_en: Option<String>,
    pub item_kind: ItemKind,
    pub obligation_kind: Option<ObligationKind>,
    pub check_type: CheckType,
    pub check_params: serde_json::Value,
    pub weight: rust_decimal::Decimal,
    pub title_vi: String,
    pub title_en: Option<String>,
    pub description_vi: Option<String>,
    pub description_en: Option<String>,
    pub authored_by: String,
    pub genie_confidence: Option<rust_decimal::Decimal>,
    pub needs_clause_ref: bool,
    pub edited_by_subject_id: Option<Uuid>,
}

/// A candidate item before it is written into a draft version. The authoring HTTP surface deserializes this
/// from the request body; [`validate_item`] gates it; [`super::authoring::add_item`] inserts it. `weight`
/// defaults to 0 (relative within a version; the roll-up is TASK-EVAL-003's, §1 #4 #14).
#[derive(Debug, Clone, Deserialize)]
pub struct RubricItemDraft {
    pub source_doc: SourceDoc,
    pub clause_ref: String,
    #[serde(default)]
    pub source_quote_vi: Option<String>,
    #[serde(default)]
    pub source_quote_en: Option<String>,
    pub item_kind: ItemKind,
    #[serde(default)]
    pub obligation_kind: Option<ObligationKind>,
    pub check_type: CheckType,
    #[serde(default)]
    pub check_params: serde_json::Value,
    #[serde(default)]
    pub weight: Option<rust_decimal::Decimal>,
    pub title_vi: String,
    #[serde(default)]
    pub title_en: Option<String>,
    #[serde(default)]
    pub description_vi: Option<String>,
    #[serde(default)]
    pub description_en: Option<String>,
}

/// The rubric error type, mapped to an HTTP status + stable error code by the handler layer. Mirrors the
/// `thiserror`-derived `AccessError` in `crate::access`. The `*_code()` strings are the §1 / §4 contract
/// codes (e.g. `rubric_item_uncited`) the acceptance tests assert on.
#[derive(Debug, thiserror::Error)]
pub enum RubricError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    /// An item with no citable clause - §1 #2, 422 `rubric_item_uncited`.
    #[error("rubric item must cite a clause (source_doc + clause_ref)")]
    Uncited,
    /// An item missing the legally-operative Vietnamese title - §1 #5, 422 `rubric_item_missing_vi`.
    #[error("rubric item must carry a Vietnamese title (_vi is required)")]
    MissingVi,
    /// An obligation item without its obligation_kind - §1 #3.
    #[error("an obligation item must set obligation_kind (confidentiality / non_compete / ip_assignment)")]
    ObligationKindRequired,
    /// A non-obligation item that set obligation_kind anyway - §1 #3 (kept clean for TASK-EVAL-003).
    #[error("obligation_kind is only valid on an obligation item")]
    ObligationKindNotAllowed,
    /// check_params does not match the shape required for its check_type - §1 #4, 422.
    #[error("check_params shape is invalid for check_type {0}")]
    CheckParamsInvalid(&'static str),
    /// A negative weight - the migration's CHECK (weight >= 0) mirror, surfaced early.
    #[error("weight must be >= 0")]
    NegativeWeight,
    /// Publishing or transitioning a version with no human approver - §1 #8, 403
    /// `rubric_requires_human_approver`.
    #[error("a rubric version requires a human approver")]
    RequiresHumanApprover,
    /// Publishing a version with no items - §1 #13, 422 `rubric_version_empty`.
    #[error("cannot publish an empty rubric version")]
    VersionEmpty,
    /// Publishing a version whose effective_from overlaps a live published version - §1 #13, 409.
    #[error("effective_from overlaps an existing published version")]
    EffectiveOverlap,
    /// Mutating a version that is not in a state that allows it (e.g. publishing a superseded one) - §1 #6.
    #[error("rubric version is not in a state that allows this transition")]
    NotMutable,
    /// resolve_effective found no published version in force on the date - surfaced to TASK-EVAL-003.
    #[error("no rubric version is effective on the requested date")]
    NoEffectiveVersion,
    /// The named rubric / version was not found in this tenant.
    #[error("rubric or version not found")]
    NotFound,
}

impl RubricError {
    /// Stable error code for the audit payload / API body (the §1 / §4 contract strings).
    pub fn code(&self) -> &'static str {
        match self {
            RubricError::Db(_) => "internal",
            RubricError::Uncited => "rubric_item_uncited",
            RubricError::MissingVi => "rubric_item_missing_vi",
            RubricError::ObligationKindRequired => "rubric_item_obligation_kind_required",
            RubricError::ObligationKindNotAllowed => "rubric_item_obligation_kind_not_allowed",
            RubricError::CheckParamsInvalid(_) => "rubric_item_check_params_invalid",
            RubricError::NegativeWeight => "rubric_item_weight_negative",
            RubricError::RequiresHumanApprover => "rubric_requires_human_approver",
            RubricError::VersionEmpty => "rubric_version_empty",
            RubricError::EffectiveOverlap => "rubric_version_effective_overlap",
            RubricError::NotMutable => "rubric_version_not_mutable",
            RubricError::NoEffectiveVersion => "rubric_no_effective_version",
            RubricError::NotFound => "rubric_not_found",
        }
    }
}

/// Validate a draft item before it may be written into a draft version (§1 #2 #3 #4 #5). This is the
/// write-time gate every authoring path runs; it never scores and never calls a model.
pub fn validate_item(it: &RubricItemDraft) -> Result<(), RubricError> {
    // §1 #2 - an item that cannot name its clause is not a rubric item.
    if it.clause_ref.trim().is_empty() {
        return Err(RubricError::Uncited);
    }
    // §1 #5 - Vietnamese is the legally-operative text; it is required.
    if it.title_vi.trim().is_empty() {
        return Err(RubricError::MissingVi);
    }
    // §1 #3 - obligation_kind pairs with item_kind exactly.
    match it.item_kind {
        ItemKind::Obligation if it.obligation_kind.is_none() => {
            return Err(RubricError::ObligationKindRequired);
        }
        ItemKind::Obligation => {}
        _ if it.obligation_kind.is_some() => {
            return Err(RubricError::ObligationKindNotAllowed);
        }
        _ => {}
    }
    // The migration enforces weight >= 0; surface a negative weight as a typed error before the DB round
    // trip so the handler returns a clean 422 rather than a constraint 500.
    if let Some(w) = it.weight {
        if w < rust_decimal::Decimal::ZERO {
            return Err(RubricError::NegativeWeight);
        }
    }
    // §1 #4 - check_params must match the shape its check_type requires.
    check_params_shape_for(it.check_type, &it.check_params)?;
    Ok(())
}

/// The shape of `check_params` keyed by `check_type` (§1 #4). An unknown / missing shape for a given
/// check_type is rejected at write time so TASK-EVAL-003 can consume the descriptor without per-item
/// interpretation. The schemas are intentionally minimal in this slice; a formal JSON-Schema registry per
/// check_type is the deferred §9 item.
pub fn check_params_shape_for(
    check_type: CheckType,
    params: &serde_json::Value,
) -> Result<(), RubricError> {
    // An empty object {} is the "no parameters" shape; treat a JSON null the same as {}.
    let is_empty_obj =
        params.is_null() || params.as_object().map(|m| m.is_empty()).unwrap_or(false);

    match check_type {
        // A numeric threshold KPI needs the metric, the comparison operator, and the target number.
        CheckType::ThresholdNumeric => {
            let obj = params
                .as_object()
                .ok_or(RubricError::CheckParamsInvalid("threshold_numeric"))?;
            let metric_ok = obj.get("metric").and_then(|v| v.as_str()).is_some();
            let operator_ok = obj
                .get("operator")
                .and_then(|v| v.as_str())
                .map(|op| matches!(op, ">=" | ">" | "<=" | "<" | "=="))
                .unwrap_or(false);
            let target_ok = obj.get("target").map(|v| v.is_number()).unwrap_or(false);
            if metric_ok && operator_ok && target_ok {
                Ok(())
            } else {
                Err(RubricError::CheckParamsInvalid("threshold_numeric"))
            }
        }
        // A periodic review needs a cadence string (e.g. "annual", "quarterly").
        CheckType::PeriodicReview => {
            let cadence_ok = params
                .as_object()
                .and_then(|m| m.get("cadence"))
                .and_then(|v| v.as_str())
                .is_some();
            if cadence_ok {
                Ok(())
            } else {
                Err(RubricError::CheckParamsInvalid("periodic_review"))
            }
        }
        // Evidence-presence, attestation, and milestone-reached carry an optional signal/marker, but an
        // empty {} is a valid shape (the item title is the standard; TASK-EVAL-003 looks for any evidence).
        // If params are present they must be a JSON object, never a scalar/array.
        CheckType::EvidencePresence | CheckType::Attestation | CheckType::MilestoneReached => {
            if is_empty_obj || params.is_object() {
                Ok(())
            } else {
                Err(RubricError::CheckParamsInvalid(
                    "evidence_or_attestation_or_milestone",
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// A well-formed obligation draft - the baseline the negative cases mutate one field of.
    fn good_obligation() -> RubricItemDraft {
        RubricItemDraft {
            source_doc: SourceDoc::NdaIp,
            clause_ref: "art.2(a)".into(),
            source_quote_vi: Some(
                "Bên Nhận Thông Tin cam kết chỉ sử dụng Thông Tin Bảo Mật...".into(),
            ),
            source_quote_en: Some(
                "RECEIVING PARTY undertakes to use the Confidential Information...".into(),
            ),
            item_kind: ItemKind::Obligation,
            obligation_kind: Some(ObligationKind::Confidentiality),
            check_type: CheckType::EvidencePresence,
            check_params: json!({}),
            weight: Some(rust_decimal::Decimal::new(1000, 2)), // 10.00
            title_vi: "Bảo mật thông tin".into(),
            title_en: Some("Confidentiality".into()),
            description_vi: None,
            description_en: None,
        }
    }

    #[test]
    fn well_formed_item_validates() {
        assert!(validate_item(&good_obligation()).is_ok());
    }

    #[test]
    fn uncited_item_is_rejected() {
        // §1 #2 / AC #2 - an empty clause_ref is rejected with the uncited code.
        let mut it = good_obligation();
        it.clause_ref = "   ".into();
        let err = validate_item(&it).unwrap_err();
        assert!(matches!(err, RubricError::Uncited));
        assert_eq!(err.code(), "rubric_item_uncited");
    }

    #[test]
    fn missing_vietnamese_title_is_rejected() {
        // §1 #5 / AC #6 - _vi is required; _en absent is allowed (good_obligation already proves that path).
        let mut it = good_obligation();
        it.title_vi = "".into();
        let err = validate_item(&it).unwrap_err();
        assert!(matches!(err, RubricError::MissingVi));
        assert_eq!(err.code(), "rubric_item_missing_vi");
    }

    #[test]
    fn obligation_without_obligation_kind_is_rejected() {
        // §1 #3 / AC #4.
        let mut it = good_obligation();
        it.obligation_kind = None;
        assert!(matches!(
            validate_item(&it).unwrap_err(),
            RubricError::ObligationKindRequired
        ));
    }

    #[test]
    fn non_obligation_with_obligation_kind_is_rejected() {
        // §1 #3 - obligation_kind only belongs on an obligation item.
        let mut it = good_obligation();
        it.item_kind = ItemKind::Kpi;
        it.check_type = CheckType::ThresholdNumeric;
        it.check_params = json!({"metric": "on_time_delivery", "operator": ">=", "target": 0.9});
        // obligation_kind still set -> rejected.
        assert!(matches!(
            validate_item(&it).unwrap_err(),
            RubricError::ObligationKindNotAllowed
        ));
    }

    #[test]
    fn threshold_numeric_requires_full_shape() {
        // §1 #4 / AC #5 - a complete {metric, operator, target} is accepted; {} is rejected.
        let mut it = good_obligation();
        it.item_kind = ItemKind::Kpi;
        it.obligation_kind = None;
        it.check_type = CheckType::ThresholdNumeric;

        it.check_params = json!({"metric": "on_time_delivery", "operator": ">=", "target": 0.9});
        assert!(validate_item(&it).is_ok());

        it.check_params = json!({});
        let err = validate_item(&it).unwrap_err();
        assert!(matches!(err, RubricError::CheckParamsInvalid(_)));
        assert_eq!(err.code(), "rubric_item_check_params_invalid");

        // A bad operator is rejected too.
        it.check_params = json!({"metric": "x", "operator": "~=", "target": 1});
        assert!(matches!(
            validate_item(&it).unwrap_err(),
            RubricError::CheckParamsInvalid(_)
        ));
    }

    #[test]
    fn evidence_presence_accepts_empty_params() {
        // The default {} shape is valid for evidence_presence; a scalar is not.
        let mut it = good_obligation();
        it.check_params = json!({});
        assert!(validate_item(&it).is_ok());

        it.check_params = json!({"signal": "no_unauthorized_disclosure"});
        assert!(validate_item(&it).is_ok());

        it.check_params = json!("not-an-object");
        assert!(matches!(
            validate_item(&it).unwrap_err(),
            RubricError::CheckParamsInvalid(_)
        ));
    }

    #[test]
    fn negative_weight_is_rejected() {
        let mut it = good_obligation();
        it.weight = Some(rust_decimal::Decimal::new(-5, 0));
        assert!(matches!(
            validate_item(&it).unwrap_err(),
            RubricError::NegativeWeight
        ));
    }

    #[test]
    fn enum_wire_forms_match_the_migration_check_constraints() {
        // The as_str() forms must match the SQL CHECK literals exactly, or a row round-trip would fail.
        assert_eq!(SourceDoc::LaborContract.as_str(), "labor_contract");
        assert_eq!(SourceDoc::NdaIp.as_str(), "nda_ip");
        assert_eq!(SourceDoc::TotalRewards.as_str(), "total_rewards");
        assert_eq!(ObligationKind::NonCompete.as_str(), "non_compete");
        assert_eq!(ObligationKind::IpAssignment.as_str(), "ip_assignment");
        assert_eq!(CheckType::ThresholdNumeric.as_str(), "threshold_numeric");
        assert_eq!(VersionState::Superseded.as_str(), "superseded");
        assert_eq!(ItemKind::CareerMilestone.as_str(), "career_milestone");
    }
}
