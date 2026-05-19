//! Frontmatter module — parse + validate SKILL.md YAML frontmatter.

pub mod description_validator;
pub mod marker_validator;
pub mod parser;
pub mod schema;

pub use description_validator::{validate as validate_description, DescriptionViolation};
pub use marker_validator::{validate as validate_marker, MarkerViolation};
pub use parser::{load_and_validate, FrontmatterError};
pub use schema::{MarkerName, SkillFrontmatter, UntrustedInputs};
