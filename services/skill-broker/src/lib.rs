//! `cyberos-skill-broker` — SKILL.md frontmatter validator + broker.
//!
//! Implements the Rust runtime side of:
//! - TASK-SKILL-103 — frontmatter schema + parser + validators
//! - TASK-SKILL-111 — description trigger enrichment (SKB-020..023)
//! - TASK-SKILL-113 — XML-free frontmatter (SKB-040..042)
//!
//! See `cyberos/modules/skill/SKILL_BUNDLE_RUBRIC.md` for the rule corpus.
//!
//! The runtime broker now includes policy extraction, tool/scope authorization,
//! invocation audit row builders, OCI bundle publish plans, and first-party
//! bundle helper contracts.

pub mod bundles;
pub mod capability;
pub mod frontmatter;
pub mod oci;
pub mod transpilers;

pub use bundles::{
    indonesia_efaktur_xml, indonesia_npwp_check, memory_capture, memory_sync,
    singapore_cpf_estimate, singapore_gst_invoice_ref, singapore_uen_check, synthesis_author,
    validate_vietnam_mst, vat_invoice_xml, vietqr_payload, IndonesiaTaxCheck, MemoryCapture,
    MemorySyncRequest, MstValidation, SingaporeCompanyCheck, VatInvoice, VietQrPayload,
};
pub use capability::{
    invocation_completed, invocation_started, CapabilityError, CapabilityPolicy, InvocationAuditRow,
};
pub use frontmatter::{
    load_and_validate, validate_description, validate_marker, FrontmatterError, MarkerName,
    SkillFrontmatter,
};
pub use oci::{digest_bytes, plan_publish, BundleRef, OciError, PublishPlan};
pub use transpilers::{transpile_anthropic, AnthropicSkill};
