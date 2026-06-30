//! FR-EVAL-002 §1 #9 - the GENIE/Lumi DRAFT path (DEC-2602). NOT IMPLEMENTED IN THIS SLICE.
//!
//! TODO(FR-EVAL-002 GENIE slice): wire Lumi (the FR-EVAL-001-governed analysis path) to read the three
//! signed documents and PROPOSE `rubric_item` rows into `state='draft'`. This is intentionally deferred
//! because it requires the AI gateway (services/ai-gateway, FR-AI-022) and the FR-EVAL-001 document-access
//! basis, neither of which this slice depends on. Keeping the seam here, with no model call compiled in,
//! keeps the module structure aligned with the FR while honouring "disabled-by-default, human-only" for now.
//!
//! When implemented, this path MUST hold to the §1 #9 contract:
//!   * it MAY create items in `state='draft'` ONLY - it has NO path to approve or publish (that is the human
//!     HITL gate in [`super::versioning::publish_version`], §1 #8);
//!   * every proposed item carries `authored_by='genie'` and a `genie_confidence`; a later human edit adds
//!     `edited_by_subject_id` but NEVER rewrites `authored_by`, so a GENIE origin stays visible forever;
//!   * it MUST NOT fabricate a `clause_ref`. When the model cannot ground an item in a specific clause it
//!     leaves `clause_ref` empty and sets `needs_clause_ref=true`; a human supplies the real clause before
//!     approval. `publish_version` already refuses to make a `needs_clause_ref` item operative (§1 #13), so
//!     an invented citation can never become the standard. This is the anti-fabrication mechanism, modelled
//!     on the obs-triage local-model precedent where an unconstrained model invented a runbook URL.
//!
//! The insert + audit machinery this path will reuse already exists in [`super::authoring`]
//! (`add_item`-style insert) and [`crate::audit`] (`eval.rubric_drafted`), so the GENIE slice is additive:
//! it adds the Lumi call and an `authored_by='genie'` insert variant; it changes nothing built here.

// No public functions yet - this slice ships the human-only authoring path. The GENIE proposer lands in a
// later slice per the TODO above.
