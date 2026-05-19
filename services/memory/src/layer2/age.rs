//! Apache AGE graph projection — mirror l2_entity / l2_edge into the
//! `cyberos_graph` AGE graph for fast traversal queries.
//!
//! AGE creates a node label per entity kind (`Person`, `Project`, `Decision`,
//! `Doc`) and a single edge label `MENTIONS` from a source-document node to
//! each extracted entity. Phase-3 will add `CITES` / `IMPLEMENTS` /
//! `SUPERSEDES` edges from typed wiki-link extraction.
//!
//! All AGE calls are best-effort: a failure to write to the graph does NOT
//! roll back the materialized l2_memory / l2_entity row. The graph is a
//! query-side projection of the canonical relational tables.

use sqlx::PgPool;
use thiserror::Error;
use tracing::warn;

#[derive(Debug, Error)]
pub enum AgeError {
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

/// Ensure the AGE graph exists. Call once at boot. Idempotent.
pub async fn ensure_graph(pool: &PgPool) -> Result<(), AgeError> {
    // SELECT create_graph('cyberos_graph') is a one-shot setup. Idempotency is
    // handled by SELECT … WHERE NOT EXISTS guard.
    let stmt = r#"
        LOAD 'age';
        SET search_path = ag_catalog, "$user", public;
        DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM ag_catalog.ag_graph WHERE name = 'cyberos_graph'
            ) THEN
                PERFORM create_graph('cyberos_graph');
            END IF;
        END$$;
    "#;
    sqlx::query(stmt).execute(pool).await?;
    Ok(())
}

/// Upsert an entity node + a `MENTIONS` edge from a doc node to it.
/// Best-effort: graph-write failures are logged + swallowed; the relational
/// projection is the source of truth.
pub async fn mirror_entity(
    pool: &PgPool,
    tenant_id: uuid::Uuid,
    kind: &str,
    name: &str,
    source_path: &str,
) {
    let label = match kind {
        "person" => "Person",
        "project" => "Project",
        "decision" => "Decision",
        "doc" => "Doc",
        _ => "Entity",
    };

    let cypher = format!(
        "SELECT * FROM cypher('cyberos_graph', $$
            MERGE (e:{label} {{tenant_id: '{tenant}', name: '{name_esc}'}})
            MERGE (d:Doc     {{tenant_id: '{tenant}', name: '{path_esc}'}})
            MERGE (d)-[:MENTIONS]->(e)
            RETURN e
         $$) AS (e ag_catalog.agtype);",
        label = label,
        tenant = tenant_id,
        name_esc = escape_cypher_string(name),
        path_esc = escape_cypher_string(source_path),
    );

    if let Err(e) = sqlx::query(&cypher).execute(pool).await {
        warn!(error = %e, %tenant_id, %kind, %name, "AGE mirror_entity failed — relational table still authoritative");
    }
}

/// Trivial Cypher-string escaper. AGE accepts standard SQL-style single-quote
/// doubling. Backslashes and newlines get stripped to keep the query oneliner.
fn escape_cypher_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\'' => out.push_str("''"),
            '\\' | '\n' | '\r' => out.push(' '),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cypher_escape_doubles_apostrophes() {
        assert_eq!(escape_cypher_string("O'Reilly"), "O''Reilly");
        assert_eq!(escape_cypher_string("plain"), "plain");
        assert_eq!(escape_cypher_string("line\nbreak"), "line break");
    }
}
