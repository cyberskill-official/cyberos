//! TASK-MEMORY-121 §1 #4 — `content_ref`: a closed union that references content by pointer or hash, never
//! by raw body. This is the privacy spine (DEC-2701): the audit chain is retained for years, so message
//! bodies, document/IP text, email bodies, and file contents are referenced here — never inlined.
//!
//! - `Pointer { store, id }` — the raw content stays in the owning store under that store's own RLS (e.g.
//!   chat's `chat_messages`); the BRAIN holds only the pointer, so there is one source of truth.
//! - `Hash { sha256, bytes, preview_len }` — for content that lives nowhere durable but where "did it
//!   change / are these the same" still matters; we keep a digest, NOT the content. `preview_len` is 0 by
//!   default; a non-zero preview is opt-in per module and MUST be a short, non-sensitive prefix only.
//! - `None` — the interaction carried no content (sign-in, presence, module-open).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// A pointer or hash of the content an interaction carried. Serialises as an internally-tagged object
/// (`{kind:"pointer"|"hash"|"none", ...}`), matching the contract's `content_ref.oneOf`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ContentRef {
    /// The raw content lives in the owning store under its own RLS; the BRAIN holds only this pointer.
    Pointer { store: String, id: String },
    /// No durable store; keep a digest only. `preview_len` is 0 unless a module opts into a short,
    /// non-sensitive prefix.
    Hash {
        sha256: String,
        bytes: u64,
        preview_len: u32,
    },
    /// The interaction carried no content.
    None,
}

impl ContentRef {
    /// Build a pointer into an owning store (e.g. `pointer("chat_messages", msg_id)`). The store names
    /// the contract permits are `chat_messages | proj_documents | email_objects | memory | attachments`.
    pub fn pointer(store: impl Into<String>, id: impl Into<String>) -> Self {
        ContentRef::Pointer {
            store: store.into(),
            id: id.into(),
        }
    }

    /// Build a content-hash reference from the raw content bytes WITHOUT storing the content: the bytes
    /// are hashed (lowercase-hex SHA-256) and their length recorded, then dropped. `preview_len` is 0 —
    /// no prefix is kept. This is the safe default for the hash arm; a module that genuinely needs a
    /// short, non-sensitive preview constructs `ContentRef::Hash { .. }` directly and owns that choice.
    pub fn hash_of(content: &[u8]) -> Self {
        let digest = Sha256::digest(content);
        let sha256 = digest.iter().map(|b| format!("{b:02x}")).collect();
        ContentRef::Hash {
            sha256,
            bytes: content.len() as u64,
            preview_len: 0,
        }
    }

    /// The union arm tag — convenience for logging / assertions.
    pub fn kind(&self) -> &'static str {
        match self {
            ContentRef::Pointer { .. } => "pointer",
            ContentRef::Hash { .. } => "hash",
            ContentRef::None => "none",
        }
    }
}

/// Redaction-safe `Display`: shows the arm + non-sensitive identifiers (store/id, the hash digest +
/// length) but never any raw content — there is none to leak by construction, and a non-zero
/// `preview_len` is reported as a count, not a value. Safe to put in logs.
impl fmt::Display for ContentRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentRef::Pointer { store, id } => write!(f, "pointer({store}:{id})"),
            ContentRef::Hash {
                sha256,
                bytes,
                preview_len,
            } => write!(
                f,
                "hash(sha256={sha256} bytes={bytes} preview_len={preview_len})"
            ),
            ContentRef::None => write!(f, "none"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_serialises_with_kind_store_id() {
        let r = ContentRef::pointer("chat_messages", "msg-7");
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["kind"], "pointer");
        assert_eq!(v["store"], "chat_messages");
        assert_eq!(v["id"], "msg-7");
    }

    #[test]
    fn hash_of_records_digest_and_length_not_content() {
        let r = ContentRef::hash_of(b"hello");
        match &r {
            ContentRef::Hash {
                sha256,
                bytes,
                preview_len,
            } => {
                // SHA-256("hello") — the same vector the chain pins elsewhere.
                assert_eq!(
                    sha256,
                    "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
                );
                assert_eq!(*bytes, 5);
                assert_eq!(*preview_len, 0, "default hash arm keeps no preview");
            }
            _ => panic!("expected a hash arm"),
        }
        // The Display must not contain the raw content.
        assert!(!format!("{r}").contains("hello"));
    }

    #[test]
    fn none_serialises_as_kind_none() {
        let v = serde_json::to_value(ContentRef::None).unwrap();
        assert_eq!(v["kind"], "none");
        assert_eq!(v.as_object().unwrap().len(), 1);
    }

    #[test]
    fn display_is_redaction_safe() {
        assert_eq!(
            ContentRef::pointer("proj_documents", "doc-1").to_string(),
            "pointer(proj_documents:doc-1)"
        );
        assert_eq!(ContentRef::None.to_string(), "none");
    }
}
