//! Pulls entities out of memory bodies. Wave-1 stub.

/// An extracted entity ready for upsert into l2_entity.
#[derive(Debug, Clone)]
pub struct ExtractedEntity {
    pub kind: String,
    pub name: String,
    pub source_seq: i64,
    pub source_path: String,
}
