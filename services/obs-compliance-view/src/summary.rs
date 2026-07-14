//! Per-view summary block (TASK-OBS-008 §1 #9): the counts an auditor reads before the row detail. Pure -
//! computed from the fetched rows, no I/O - so it is unit-tested directly.

use std::collections::BTreeMap;

use crate::query::AuditRow;

/// The headline an auditor sees: how many rows, broken down by audit kind, and the seq span covered.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Summary {
    pub total_rows: u64,
    /// Count per `event_type`. `BTreeMap` so the JSON key order is stable (the manifest signs the bytes).
    pub by_kind: BTreeMap<String, u64>,
    pub first_seq: Option<i64>,
    pub last_seq: Option<i64>,
}

/// Summarise the rows a view returned. Rows are assumed seq-ordered (the query's `ORDER BY seq`).
pub fn summarize(rows: &[AuditRow]) -> Summary {
    let mut by_kind: BTreeMap<String, u64> = BTreeMap::new();
    for r in rows {
        let kind = r
            .event_type
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        *by_kind.entry(kind).or_default() += 1;
    }
    Summary {
        total_rows: rows.len() as u64,
        by_kind,
        first_seq: rows.first().map(|r| r.seq),
        last_seq: rows.last().map(|r| r.seq),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(seq: i64, kind: Option<&str>) -> AuditRow {
        AuditRow {
            seq,
            event_type: kind.map(|s| s.to_string()),
            op: "put".to_string(),
            path: format!("auth/tenant/x/{seq}"),
            subject_id: None,
            chain_anchor_hex: "00".repeat(32),
            ts_ns: seq * 1000,
        }
    }

    #[test]
    fn empty_rows_summarize_to_zero() {
        let s = summarize(&[]);
        assert_eq!(s.total_rows, 0);
        assert!(s.by_kind.is_empty());
        assert_eq!(s.first_seq, None);
        assert_eq!(s.last_seq, None);
    }

    #[test]
    fn counts_group_by_kind_and_track_seq_span() {
        let rows = vec![
            row(10, Some("auth.login_succeeded")),
            row(11, Some("auth.login_failed")),
            row(12, Some("auth.login_succeeded")),
        ];
        let s = summarize(&rows);
        assert_eq!(s.total_rows, 3);
        assert_eq!(s.by_kind.get("auth.login_succeeded"), Some(&2));
        assert_eq!(s.by_kind.get("auth.login_failed"), Some(&1));
        assert_eq!(s.first_seq, Some(10));
        assert_eq!(s.last_seq, Some(12));
    }

    #[test]
    fn null_event_type_counts_as_unknown() {
        let rows = vec![row(1, None), row(2, None)];
        let s = summarize(&rows);
        assert_eq!(s.by_kind.get("unknown"), Some(&2));
    }
}
