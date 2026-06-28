// Apache AGE removed 2026-06-28 (drop-age). The layer-2 graph is now the relational l2_entity / l2_edge
// tables: doc->entity MENTIONS provenance lives in l2_entity.source_path / source_seq, and entity->entity
// edges (l2_edge) are traversed with recursive CTEs. This file is no longer part of the crate (the
// `pub mod age;` declaration was removed from layer2/mod.rs) and should be deleted on commit:
//   git rm services/memory/src/layer2/age.rs
