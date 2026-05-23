use chrono::{Duration, Utc};
use cyberos_proj::billing::{
    resolve_billable, rollup, BillableRules, BillingConfig, BillingMode, TaskClass, TimeEntry,
};
use cyberos_proj::blockers::{detect_blockers, Comment};
use cyberos_proj::crdt::{
    lww_merge, reconnect_state, CollaborativeField, CrdtDocument, LwwValue, YjsUpdate,
};
use cyberos_proj::cycle_review::{draft_cycle_review, summarize_cycle};
use cyberos_proj::drift::{detect_drift, DriftReason, MemoryCitationSnapshot};
use cyberos_proj::estimate::{calibrate, EstimateObservation};
use cyberos_proj::history::{build_history_event, FieldDiff};
use cyberos_proj::memory_link::{create_link, LinkStrength, MemoryLinkType, MemoryTarget};
use cyberos_proj::rate_card::BillingRole;
use cyberos_proj::types::{Issue, IssuePriority, IssueStatus};
use cyberos_proj::views::{
    axe_gate, kanban_columns, timeline_lanes, A11yFinding, GanttTask, TOKENS_PROJ_CSS,
};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

fn issue(status: IssueStatus, assignee: Option<Uuid>) -> Issue {
    Issue {
        id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        engagement_id: Uuid::new_v4(),
        cycle_id: None,
        title: "ship".into(),
        body: None,
        status,
        priority: IssuePriority::Normal,
        assignee_subject_id: assignee,
        estimate_hours: Some(3.0),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

#[test]
fn crdt_updates_are_idempotent_and_lww_scalars_tie_break() {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let mut doc = CrdtDocument::new(id, CollaborativeField::Description);
    let update = YjsUpdate {
        document_id: id,
        field: CollaborativeField::Description,
        client_id: "a".into(),
        clock: 1,
        payload: vec![1, 2, 3],
        received_at: now,
    };
    assert!(doc.apply(update.clone()).unwrap());
    assert!(!doc.apply(update).unwrap());
    assert_eq!(doc.state_vector.get("a"), Some(&1));

    let left = LwwValue {
        value: "old",
        updated_at: now,
        actor_id: Uuid::from_u128(1),
    };
    let right = LwwValue {
        value: "new",
        updated_at: now,
        actor_id: Uuid::from_u128(2),
    };
    assert_eq!(lww_merge(left, right).value, "new");

    let mut peer = BTreeMap::new();
    peer.insert("a".to_string(), 0);
    assert_eq!(doc.missing_for(&peer).len(), 1);
    let (upload, download) = reconnect_state(&doc.state_vector, &peer);
    assert!(upload.contains("a"));
    assert!(download.is_empty());
}

#[test]
fn billable_cascade_and_mode_rollups_are_deterministic() {
    let member = Uuid::new_v4();
    let mut rules = BillableRules {
        fallback: false,
        ..Default::default()
    };
    rules.role_defaults.insert(BillingRole::Engineer, true);
    rules.task_class_defaults.insert(TaskClass::Internal, false);
    rules.member_overrides.insert(member, true);
    assert!(resolve_billable(&rules, member, TaskClass::Internal, BillingRole::Engineer).billable);
    assert!(
        !resolve_billable(
            &rules,
            Uuid::new_v4(),
            TaskClass::Internal,
            BillingRole::Engineer
        )
        .billable
    );

    let entries = [
        TimeEntry {
            minutes: 90,
            billable: true,
        },
        TimeEntry {
            minutes: 30,
            billable: false,
        },
    ];
    let tm = rollup(
        &entries,
        BillingConfig {
            mode: BillingMode::TimeAndMaterials,
            hourly_rate_minor: 10_000,
            fixed_fee_minor: 0,
            retainer_included_hours: 0,
            retainer_fee_minor: 0,
        },
    );
    assert_eq!(tm.amount_minor, 15_000);
    let retainer = rollup(
        &entries,
        BillingConfig {
            mode: BillingMode::Retainer,
            hourly_rate_minor: 10_000,
            fixed_fee_minor: 0,
            retainer_included_hours: 1,
            retainer_fee_minor: 50_000,
        },
    );
    assert_eq!(retainer.overage_minutes, 30);
    assert_eq!(retainer.amount_minor, 55_000);
}

#[test]
fn history_and_memory_link_drift_cover_mutation_chain() {
    let event = build_history_event(
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        "req-1",
        vec![FieldDiff {
            field: "status".into(),
            before: Some("todo".into()),
            after: Some("doing".into()),
        }],
        "abc123",
    );
    assert_eq!(event.event_hash.len(), 64);

    let tenant = Uuid::new_v4();
    let issue_id = Uuid::new_v4();
    let created = Utc::now();
    let target = MemoryTarget {
        path: "memories/decisions/aa/bb/one.md".into(),
        tenant_id: tenant,
        created_at: created - Duration::days(1),
        readable: true,
    };
    let link = create_link(
        tenant,
        issue_id,
        created,
        Some(target),
        MemoryLinkType::Cites,
        &[],
        None,
        Some(LinkStrength::Medium),
    )
    .unwrap();
    let snapshot = MemoryCitationSnapshot {
        existing_paths: BTreeSet::from([link.memory_path.clone()]),
        superseded_paths: BTreeSet::from([link.memory_path.clone()]),
        backlink_paths: BTreeSet::new(),
    };
    let findings = detect_drift(&[link], &snapshot);
    assert!(findings
        .iter()
        .any(|f| f.reason == DriftReason::TargetSuperseded));
    assert!(findings
        .iter()
        .any(|f| f.reason == DriftReason::BrokenBacklink));
}

#[test]
fn blocker_cycle_estimate_and_views_are_available() {
    let now = Utc::now();
    let issue_id = Uuid::new_v4();
    let blockers = detect_blockers(
        &[Comment {
            issue_id,
            body: "blocked by missing API key".into(),
            created_at: now - Duration::hours(26),
        }],
        now,
        Duration::hours(24),
    );
    assert!(blockers[0].notify_cuo);

    let assignee = Uuid::new_v4();
    let issues = vec![
        issue(IssueStatus::Done, Some(assignee)),
        issue(IssueStatus::Doing, Some(assignee)),
    ];
    let stats = summarize_cycle(&issues, blockers.len());
    assert!(draft_cycle_review(&stats).contains("Completion: 50%"));
    assert_eq!(
        kanban_columns(&issues)
            .iter()
            .find(|c| c.status == IssueStatus::Done)
            .unwrap()
            .issue_ids
            .len(),
        1
    );
    assert_eq!(timeline_lanes(&issues)[0].issue_ids.len(), 2);

    let snapshot = calibrate(
        assignee,
        &[EstimateObservation {
            member_id: assignee,
            estimated_hours: 2.0,
            actual_hours: 3.0,
        }],
    );
    assert!(snapshot.bias_ratio > 1.0);

    let mut tasks = vec![
        GanttTask {
            issue_id: issues[0].id,
            starts_at: now.date_naive(),
            ends_at: now.date_naive() + Duration::days(1),
            depends_on: vec![],
            critical: false,
        },
        GanttTask {
            issue_id: issues[1].id,
            starts_at: now.date_naive(),
            ends_at: now.date_naive() + Duration::days(2),
            depends_on: vec![issues[0].id],
            critical: false,
        },
    ];
    cyberos_proj::views::mark_critical_path(&mut tasks);
    assert!(tasks.iter().all(|task| task.critical));
    assert!(TOKENS_PROJ_CSS.contains("--proj-radius: 8px"));
    assert!(axe_gate(&[]).is_ok());
    assert!(axe_gate(&[A11yFinding {
        rule: "button-name".into(),
        selector: "#save".into()
    }])
    .is_err());
}
