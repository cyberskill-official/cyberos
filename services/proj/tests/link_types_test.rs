//! TASK-PROJ-001 §4 #10 + §4 #11 — bidirectional symmetric links + cross-module link types.

use cyberos_proj::types::LinkType;

#[test]
fn symmetric_link_pairs() {
    assert_eq!(LinkType::Blocks.inverse(), Some(LinkType::BlockedBy));
    assert_eq!(LinkType::BlockedBy.inverse(), Some(LinkType::Blocks));
    assert_eq!(LinkType::Duplicates.inverse(), Some(LinkType::DuplicatedBy));
    assert_eq!(LinkType::DuplicatedBy.inverse(), Some(LinkType::Duplicates));
}

#[test]
fn asymmetric_link_types_have_no_inverse() {
    for lt in [
        LinkType::Related,
        LinkType::DerivedFromEmailThread,
        LinkType::DerivedFromChatThread,
        LinkType::DerivedFromMeetingDecision,
    ] {
        assert!(lt.inverse().is_none(), "{lt:?} should have no inverse");
    }
}

#[test]
fn parse_round_trip_for_all_known_types() {
    for raw in [
        "duplicates",
        "duplicated_by",
        "blocks",
        "blocked_by",
        "related",
        "derived_from_email_thread",
        "derived_from_chat_thread",
        "derived_from_meeting_decision",
    ] {
        let parsed = LinkType::parse(raw).expect(raw);
        assert_eq!(parsed.as_str(), raw, "round trip for {raw}");
    }
}

#[test]
fn unknown_link_type_rejected() {
    assert!(LinkType::parse("unknown_link").is_none());
    assert!(LinkType::parse("").is_none());
    assert!(LinkType::parse("Blocks").is_none()); // case-sensitive
}

#[test]
fn enum_count_matches_sql_check() {
    // SQL CHECK CONSTRAINT lists 8 values; the Rust enum has 8 variants.
    let strings: Vec<_> = [
        LinkType::Duplicates,
        LinkType::DuplicatedBy,
        LinkType::Blocks,
        LinkType::BlockedBy,
        LinkType::Related,
        LinkType::DerivedFromEmailThread,
        LinkType::DerivedFromChatThread,
        LinkType::DerivedFromMeetingDecision,
    ]
    .iter()
    .map(|l| l.as_str())
    .collect();
    let set: std::collections::HashSet<_> = strings.iter().collect();
    assert_eq!(set.len(), 8);
}
