use cyberos_skill_broker::frontmatter::SkillFrontmatter;
use cyberos_skill_broker::{
    indonesia_efaktur_xml, indonesia_npwp_check, invocation_completed, invocation_started,
    memory_capture, memory_sync, plan_publish, singapore_cpf_estimate, singapore_gst_invoice_ref,
    singapore_uen_check, synthesis_author, validate_vietnam_mst, vat_invoice_xml, vietqr_payload,
    CapabilityPolicy, OciError, VatInvoice,
};

#[test]
fn capability_policy_authorizes_tools_and_memory_scopes() {
    let fm: SkillFrontmatter = serde_yaml::from_str(
        r#"
name: memory-capture
description: Capture durable memory rows from workflow outputs. Use when user asks to "remember this" or "capture this decision". Outputs memory writer envelopes with least-privilege scope checks.
metadata: { version: "1.0.0" }
allowed_memory_scopes:
  read: [project:*]
  write: [memories:facts, memories:decisions]
allowed_mcp_tools: [memory.write_memory, audit.*]
"#,
    ).unwrap();
    let policy = CapabilityPolicy::from_frontmatter(&fm);
    assert!(policy.authorize_tool("memory.write_memory").is_ok());
    assert!(policy.authorize_tool("audit.append").is_ok());
    assert!(policy.authorize_tool("shell.exec").is_err());
    assert!(policy.authorize_memory_scope("memories:decisions").is_ok());
    assert!(policy.authorize_memory_scope("client:restricted").is_err());
}

#[test]
fn invocation_rows_have_closed_kinds() {
    let started = invocation_started("memory-capture", "i1");
    let completed = invocation_completed("memory-capture", "i1", "ok");
    assert_eq!(started.row_kind, "skill.invocation_started");
    assert_eq!(completed.row_kind, "skill.invocation_completed");
    assert_eq!(completed.outcome.as_deref(), Some("ok"));
}

#[test]
fn oci_publish_plan_requires_immutable_tag_and_tenant_scope() {
    let plan = plan_publish(
        "registry.local",
        "skill/memory-capture",
        "1.0.0",
        b"bundle",
        "tenant-a",
    )
    .unwrap();
    assert!(plan.bundle.digest.starts_with("sha256:"));
    assert!(plan.cosign_required);
    let err =
        plan_publish("registry.local", "skill/x", "latest", b"bundle", "tenant-a").unwrap_err();
    assert!(matches!(err, OciError::MutableTag(_)));
}

#[test]
fn built_in_bundle_helpers_cover_memory_and_vietnam_pack() {
    assert_eq!(
        memory_capture("facts", "body").row_kind,
        "skill.memory_capture.requested"
    );
    assert!(memory_sync("push", true).dry_run);
    assert!(synthesis_author(&["a", "b"]).contains("2 cluster"));
    assert!(validate_vietnam_mst("0312345678").valid_shape);
    assert_eq!(
        vietqr_payload("970436", "123", 50_000, "INV-1").payload,
        "VQR|970436|123|50000|INV-1"
    );
    let xml = vat_invoice_xml(&VatInvoice {
        seller_mst: "0312345678".into(),
        buyer_mst: "0100109106".into(),
        invoice_no: "A<1>".into(),
        total_vnd: 100_000,
    });
    assert!(xml.contains("A&lt;1&gt;"));
    assert!(singapore_uen_check("201912345A").valid_shape);
    assert_eq!(
        singapore_gst_invoice_ref("201912345a", "INV-9"),
        "SG-GST-201912345A-INV-9"
    );
    assert_eq!(singapore_cpf_estimate(500_000, 1_700), 85_000);
    assert!(indonesia_npwp_check("01.234.567.8-901.000").valid_shape);
    assert!(indonesia_efaktur_xml("012345678901000", "010.001", 10_000)
        .contains("<AmountIDR>10000</AmountIDR>"));
}
