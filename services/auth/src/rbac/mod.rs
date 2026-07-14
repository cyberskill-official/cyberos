//! TASK-AUTH-101 — closed 22-role RBAC catalogue + permission matrix + assignment REST.
//!
//! Per DEC-121 + DEC-122, the catalogue is **closed**: adding a 23rd role requires
//! an ADR. Per DEC-123 the 5 stub roles from TASK-AUTH-002 (root-admin / tenant-admin /
//! tenant-member / service-account / agent-persona) are a strict prefix; their
//! permission sets are additive only.
//!
//! Wave 2 slice ships:
//!   * `catalogue` — closed `Role` enum + Display/FromStr + reserved + requires_webauthn
//!   * `permissions` — closed `Resource` and `Action` enums
//!   * `matrix` — in-memory `RoleMatrix` snapshot loaded at boot
//!   * `assignment` — `POST /v1/admin/subjects/{id}/roles` + `DELETE /…/{role}`
//!   * `catalogue_endpoint` — `GET /v1/admin/roles`
//!
//! Deferred to follow-up: 60s background refresher, SQL `auth.has_role()` function,
//! scope-grant narrowing layer, ADR-gate CI test, OTel metrics, perf test.

pub mod adr;
pub mod assignment;
pub mod catalogue;
pub mod catalogue_endpoint;
pub mod matrix;
pub mod permissions;
pub mod refresher;

pub use catalogue::{Role, RoleParseError};
pub use matrix::RoleMatrix;
pub use permissions::{Action, Resource};
