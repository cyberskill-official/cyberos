//! TASK-EVAL-002 - the evaluation rubric built from the three signed employment documents.
//!
//! Turns the Labor Contract, the NDA/non-compete/IP agreement, and the Total Rewards & Career Path Appendix
//! into a structured, versioned, clause-cited framework TASK-EVAL-003 can evaluate evidence against - with a
//! human approving every item before it is effective (DEC-2600..2604). This module is the human-curated
//! authoring + versioning path; the schema is `migrations/0003_rubric.sql`.
//!
//! DISABLED-BY-DEFAULT, HUMAN-ONLY: nothing here scores a person or calls a model. It only lets a human
//! author and publish clause-cited criteria. The GENIE/Lumi draft path (DEC-2602) needs the AI gateway and
//! is a deliberately separate, later slice - see [`draft_genie`] for the seam + TODO.
//!
//! Layout (mirrors the FR's `new_files`):
//! - [`model`] - the five closed enums, the row + draft structs, [`model::validate_item`], and `RubricError`.
//! - [`authoring`] - the human create / add-item flow (validate -> insert -> audit).
//! - [`versioning`] - `resolve_effective(at)`, the HITL `publish_version`, supersede-not-mutate.
//! - [`draft_genie`] - the deferred GENIE proposer (TODO; no model call in this slice).
//!
//! Governance-first (DEC-2601): the HTTP surface in `crate::handlers` access-gates every rubric mutation by
//! the TASK-EVAL-001 grant (founder + designated rubric admins) before calling into here.

pub mod authoring;
pub mod draft_genie;
pub mod model;
pub mod versioning;

pub use model::{
    validate_item, CheckType, ItemKind, ObligationKind, Rubric, RubricError, RubricItem,
    RubricItemDraft, RubricVersion, SourceDoc, VersionState,
};
