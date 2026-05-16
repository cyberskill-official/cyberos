---
id: FR-BRAIN-111
title: "BRAIN pre-ingest PII detection — Presidio EN + custom VN recognisers; ≥ 99.5% held-back recall on labelled fixture; auto-redact at capture boundary"
module: BRAIN
priority: MUST
status: accepted
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-16
shipped: null
brain_chain_hash: null
related_frs: [FR-BRAIN-107, FR-BRAIN-109, FR-AI-011, FR-AI-012, FR-AI-013, FR-OBS-007]
depends_on: [FR-BRAIN-107, FR-AI-012]
blocks: [FR-TEN-004]

source_pages:
  - website/docs/modules/brain.html#pii-pre-ingest
  - website/docs/runbooks/brain-pii-runbook.html
source_decisions:
  - DEC-170 (PII detection happens at capture boundary BEFORE chain commit; redaction is mandatory)
  - DEC-171 (canonical ruleset is shared between FR-BRAIN-109 hook + this FR; one regex catalogue, two consumers)
  - DEC-172 (Vietnamese PII includes CCCD, MST, passport, bank account; recognisers are first-class)
  - DEC-173 (≥ 99.5% recall is the CI gate; ≤ 1% false-positive rate; both measured against a labelled fixture corpus)

language: rust 1.81 + python 3.11 (presidio bridge)
service: cyberos/services/brain-capture/
new_files:
  - services/brain-capture/src/pii.rs
  - services/brain-capture/src/pii/presidio_bridge.rs
  - services/brain-capture/src/pii/vn_recognisers.rs
  - services/brain-capture/src/pii/ruleset.rs
  - services/brain-capture/install/presidio/Cargo-deps.toml.sub        # NER deps subset
  - services/brain-capture/install/presidio/requirements.txt
  - services/brain-capture/tests/pii_recall_test.rs
  - services/brain-capture/tests/fixtures/pii-corpus.jsonl
  - services/brain-capture/tests/fixtures/pii-corpus-vn.jsonl
modified_files:
  - services/brain-capture/src/emit.rs                  # call pii::scan_and_redact before BRAIN write
  - services/brain-claude-hook/src/redact.rs            # re-export ruleset from pii::ruleset
  - services/ai-gateway/src/pii.rs                      # FR-AI-011/012 share ruleset.rs
allowed_tools:
  - file_read: services/brain-capture/**, services/ai-gateway/**, services/brain-claude-hook/**
  - file_write: services/brain-capture/{src,tests,install}/**
  - bash: cd services/brain-capture && cargo test pii
  - bash: cd services/brain-capture && python3 -m pytest install/presidio/test_bridge.py
disallowed_tools:
  - emit memories whose body contains unredacted PII matching ruleset (per §1 #1 — fail closed)
  - skip pre-ingest scan on any capture path (per DEC-170)
  - diverge from FR-AI-012's canonical ruleset (per DEC-171)

effort_hours: 9
sub_tasks:
  - "0.5h: pii/ruleset.rs — shared regex catalogue + match-tag enum (used by all three callers)"
  - "1.5h: pii/vn_recognisers.rs — CCCD (12-digit checksum), MST (10/13-digit), passport (1+8 alphanum), bank account (10–18 digits with bank-prefix table)"
  - "1.0h: pii/presidio_bridge.rs — subprocess to bundled Presidio (python) for NER-backed person/location names; pyo3 not used (too heavy); plain pipe-stdin/stdout"
  - "1.0h: pii.rs — orchestrator: regex first (fast); Presidio second (NER for names); merge overlapping spans; apply redactions"
  - "0.5h: integrate into FR-BRAIN-107 emit.rs (call pii::scan_and_redact BEFORE blake3 hashing — hash the redacted body)"
  - "0.5h: integrate FR-BRAIN-109 redactor delegation (delete inline regexes; use ruleset.rs)"
  - "0.5h: FR-AI-011 + FR-AI-012 module sync (move their regexes to ruleset.rs)"
  - "1.5h: pii_recall_test.rs — load fixture corpus; assert recall ≥ 99.5% + FP ≤ 1%"
  - "1.0h: fixtures/pii-corpus.jsonl — 500 labelled English examples (positive + negative)"
  - "1.0h: fixtures/pii-corpus-vn.jsonl — 500 labelled Vietnamese examples (CCCD, MST, addresses, bank accts)"
  - "0.5h: docs/pii-ruleset.md — operator-facing list of detected categories"
risk_if_skipped: "Without pre-ingest scan, capture rows land in BRAIN with raw PII. Vietnamese data protection law (PDPL 2025) treats CCCD as restricted personal data; storing it without authorisation is a regulatory violation. Operators writing Slack-style messages routinely paste customer info ('Tran Quoc Khanh CCCD 079123...'); without scan, BRAIN becomes a PII honeypot. The 99.5% recall gate is the load-bearing CI signal that says 'capture is safe to ship'; without it, redaction quality silently regresses on every change. Without a shared ruleset (DEC-171), the AI Gateway redactor + capture redactor + Claude Code hook redactor drift apart — three sources of truth = inconsistent privacy posture."
---

## §1 — Description (BCP-14 normative)

The pre-ingest PII detection layer **MUST** sit between the FR-BRAIN-107 capture daemon's event classifier and its BRAIN-writer call, scanning every byte that would be written to the chain. The contract:

1. **MUST** be invoked synchronously per-event in `emit.rs` before `blake3::hash` is computed. The hash is over the REDACTED body — same content with same PII = same hash, but the raw body never leaves the function frame.
2. **MUST** consume the canonical ruleset from `pii::ruleset` (single source of truth per DEC-171). Three callers (BRAIN capture, Claude Code hook, AI Gateway redactor) MUST import from the same crate; divergence is forbidden by construction (CI test asserts no inline regex literals in callers).
3. **MUST** support the following detection categories, each tagged with a `MatchTag` enum variant:
    - English: `EmailAddress`, `PhoneNumber`, `CreditCard`, `SSN`, `IpAddress`, `AwsAccessKey`, `AwsSecretKey`, `BearerToken`, `JwtToken`, `GitHubPat`, `OpenAiKey`, `AnthropicKey`, `IbanCode`, `UsPassport`.
    - Vietnamese: `VnCccd` (12-digit national ID with checksum), `VnMst` (10 or 13-digit tax ID), `VnPassport` (1 letter + 8 digits), `VnBankAccount` (bank-prefix + 6–14 digits), `VnPhoneLocal` (`+84` or `0` prefix), `VnAddress` (heuristic: contains "phường|quận|huyện|tỉnh|thành phố").
    - Names (NER-backed): `PersonName`, `LocationName`, `OrganizationName`.
4. **MUST** prioritise regex categories over NER (faster + higher precision). The orchestrator runs regex sweep first (≤ 10ms for 8KB input); only invokes Presidio NER if `body_len > 32 bytes` and no PII has been found by regex (cuts NER calls by ~80% in practice).
5. **MUST** redact every matched span with `<TAG>` where TAG is the MatchTag (`<EMAIL>`, `<VN_CCCD>`, `<PERSON_NAME>`, etc.). The original byte length of the span is irrelevant; redactions are length-changing.
6. **MUST** report results as `ScanResult { redacted_body: String, matches: Vec<(Span, MatchTag)>, confidence: f32 }`. The `confidence` is the lowest-confidence match in the set (regex = 1.0, NER = Presidio's score).
7. **MUST** fail-closed on detection error: if Presidio subprocess crashes, NER OOMs, regex panics — the scan returns `ScanResult { redacted_body: "<SCAN_FAILED>", matches: vec![], confidence: 0.0 }` and the caller emits a `brain.capture_pii_scan_failed` audit row with `{folder_path, body_byte_count, error}`. The original body is dropped (never reaches the chain).
8. **MUST** maintain a labelled fixture corpus in `tests/fixtures/pii-corpus*.jsonl` with at least 500 English + 500 Vietnamese examples. Each line is `{"text": "...", "expected_matches": [{"tag": "Email", "start": 12, "end": 28}, ...]}`.
9. **MUST** pass a CI gate (`pii_recall_test.rs`) asserting:
    - **Recall ≥ 99.5%** across the full fixture corpus (held-back rate; how many PII spans were caught).
    - **False-positive rate ≤ 1%** across the same corpus (how many non-PII spans got flagged).
   Failure → CI blocked; PR cannot merge until ruleset is tuned.
10. **MUST** allow per-tenant `pii_allowlist[]` (regex strings) that override matches. Example: a tenant whose business model legitimately involves CCCDs (KYC vendor) can add `pii_allowlist: ["^CCCD: 0\\d{11}$"]` so those specific patterns pass through unredacted. Allowlists are tenant-scoped in `manifest.tenants[].pii_allowlist`; the empty list is the safe default.
11. **MUST** emit OTel span `brain.pii.scan` per invocation with attributes `body_bytes`, `match_count`, `nerinvoked` (bool), `duration_ms`, `confidence`.
12. **MUST** emit OTel metrics:
    - `brain_pii_matches_total{tag}` (counter; cardinality bounded by enum variants).
    - `brain_pii_scan_duration_seconds{ner_invoked}` (histogram; FR-OBS-003 buckets).
    - `brain_pii_scan_failed_total{reason}` (counter; reasons ∈ regex_panic | ner_subprocess_died | ner_timeout | oom).
13. **MUST** complete scan in ≤ 50ms p95 for ≤ 8KB input on commodity hardware. NER-invoked path budgeted ≤ 200ms p95; regex-only ≤ 10ms p95.
14. **SHOULD** support `cyberos brain pii test --input <file>` CLI for operator debugging — prints scan result with span highlights.
15. **SHOULD** allow per-folder override of detection categories (e.g. `meta/people/<id>/medical/*` enables `HealthCondition` recogniser; default off elsewhere).

---

## §2 — Why this design (rationale for humans)

**Why pre-ingest, not post-ingest (§1 #1)?** Once PII lands in the BRAIN chain it is, by AGENTS.md §3.4, immutable — the only way to remove it is `delete(path, "purge")` which requires explicit chat-turn approval. That's a heavy lift for routine PII slip-ups. Pre-ingest catches the problem at the boundary; the chain stays clean by construction.

**Why hash the redacted body (§1 #1)?** Same redacted bytes = same memory (idempotent capture). If we hashed raw → identical-with-different-PII bodies produce different hashes → duplicate memories. Hashing the redacted form is the correct invariant.

**Why one shared ruleset (§1 #2 + DEC-171)?** Three places redact PII: the capture daemon (this FR), the Claude Code hook (FR-BRAIN-109), the AI Gateway (FR-AI-011/012). If each maintains its own regex catalogue, drift is inevitable — fixing a CCCD regex in one place doesn't fix it in the others. One crate, three importers. The CI test that bans inline regex literals in callers locks this in mechanically.

**Why regex-first, NER-second (§1 #4)?** Regex handles 95% of detected PII (emails, phone numbers, structured IDs). NER (Presidio) catches the remaining 5% (names, locations). NER is 20× slower than regex. Running NER only when regex finds nothing AND body is non-trivial gives us 99% of the recall at 20% of the cost.

**Why Presidio over a custom NER (§1 #4)?** Presidio is Microsoft's production-tested PII NER; bundling it is faster than training our own. The Python subprocess overhead (~50ms cold start) is acceptable because we only invoke it on the 5% slow path. PyO3 was rejected because it forces Python-in-Rust process; subprocess is operationally simpler.

**Why fail-closed (§1 #7)?** If we fail-OPEN (emit raw body on scan error), one bad Presidio release leaks every prompt for that period. Fail-CLOSED means worst-case we drop one row of capture + emit a `pii_scan_failed` audit row → operator notices + fixes. Data leak is impossible by construction.

**Why 99.5% recall (§1 #9)?** Compliance commitment: the BRAIN never stores raw PII matching our ruleset. 99.5% recall means ≤ 0.5% of PII spans slip through — those are typically edge cases (CCCDs with unusual formatting). The remaining 0.5% are caught by FR-AI-013's post-ingest recall gate AND by quarterly red-team review. 100% recall is impossible (open language; Presidio bounded). 99.5% is the calibrated threshold.

**Why ≤ 1% FP (§1 #9)?** False positives degrade UX: a row reading `<EMAIL> please review` is less useful than `alice@cyberskill.world please review` (when alice's email is allowlisted). The 1% bound keeps the catalogue tight; allowlists handle the legitimate exceptions.

**Why tenant-scoped allowlist (§1 #10)?** A KYC vendor LEGITIMATELY stores CCCDs (it's their product). Refusing to capture them would break their workflow. Per-tenant allowlists let those tenants opt-in. Default (empty) is safe; the override is explicit + auditable.

**Why per-folder category override (§1 #15)?** A folder of medical notes legitimately contains health conditions; flagging them as PII would over-redact. Per-folder category overrides let operators say "this folder is medical; redact `HealthCondition` here, not elsewhere." Slice-3+ feature; placeholder noted for forward compatibility.

**Why ≤ 50ms p95 (§1 #13)?** Capture is on FR-BRAIN-107's hot path; total event-to-emit budget is ~250ms (FR-BRAIN-107 §1 #2 debounce + redaction + dedup + emit). 50ms for PII scan keeps the latency tax bounded.

---

## §3 — API contract

### ScanResult + MatchTag

```rust
// services/brain-capture/src/pii.rs
use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum MatchTag {
    // English
    EmailAddress, PhoneNumber, CreditCard, Ssn, IpAddress, AwsAccessKey, AwsSecretKey,
    BearerToken, JwtToken, GitHubPat, OpenAiKey, AnthropicKey, IbanCode, UsPassport,
    // Vietnamese
    VnCccd, VnMst, VnPassport, VnBankAccount, VnPhoneLocal, VnAddress,
    // Names (NER)
    PersonName, LocationName, OrganizationName,
}

impl MatchTag {
    pub fn replacement(self) -> &'static str {
        use MatchTag::*;
        match self {
            EmailAddress  => "<EMAIL>",      PhoneNumber  => "<PHONE>",
            CreditCard    => "<CC>",         Ssn          => "<SSN>",
            IpAddress     => "<IP>",         AwsAccessKey => "<AWS_KEY>",
            AwsSecretKey  => "<AWS_SECRET>", BearerToken  => "<BEARER>",
            JwtToken      => "<JWT>",        GitHubPat    => "<GH_PAT>",
            OpenAiKey     => "<OPENAI_KEY>", AnthropicKey => "<ANTHROPIC_KEY>",
            IbanCode      => "<IBAN>",      UsPassport   => "<US_PASSPORT>",
            VnCccd        => "<VN_CCCD>",   VnMst        => "<VN_MST>",
            VnPassport    => "<VN_PASSPORT>", VnBankAccount => "<VN_BANK_ACCT>",
            VnPhoneLocal  => "<VN_PHONE>",  VnAddress    => "<VN_ADDR>",
            PersonName    => "<PERSON>",    LocationName => "<LOCATION>",
            OrganizationName => "<ORG>",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScanResult {
    pub redacted_body: String,
    pub matches:       Vec<(Range<usize>, MatchTag)>,
    pub confidence:    f32,
    pub ner_invoked:   bool,
}

pub async fn scan_and_redact(body: &str, allowlist: &[regex::Regex]) -> ScanResult {
    let start = std::time::Instant::now();

    // Pass 1: regex sweep (covers 95% of PII)
    let regex_matches = ruleset::scan_with_regex(body);

    // Pass 2: NER if regex found nothing AND body is non-trivial
    let mut all_matches = regex_matches;
    let ner_invoked = all_matches.is_empty() && body.len() > 32;
    if ner_invoked {
        match presidio_bridge::scan(body).await {
            Ok(ner_matches) => all_matches.extend(ner_matches),
            Err(e) => {
                tracing::warn!(?e, "presidio NER failed; returning regex-only matches");
                metrics::counter!("brain_pii_scan_failed_total", "reason" => "ner_subprocess_died").increment(1);
            }
        }
    }

    // Pass 3: apply tenant allowlist (drop matches that the allowlist matches)
    if !allowlist.is_empty() {
        all_matches.retain(|(span, _tag)| {
            !allowlist.iter().any(|rx| rx.is_match(&body[span.clone()]))
        });
    }

    // Sort by start, dedup overlapping (longest match wins)
    all_matches.sort_by_key(|(s, _)| (s.start, std::cmp::Reverse(s.end)));
    let mut deduped: Vec<(Range<usize>, MatchTag)> = Vec::new();
    for m in all_matches {
        if let Some(last) = deduped.last() {
            if m.0.start < last.0.end { continue; }  // overlap; skip
        }
        deduped.push(m);
    }

    // Apply redactions (right-to-left so offsets stay valid)
    let mut redacted = body.to_string();
    for (span, tag) in deduped.iter().rev() {
        redacted.replace_range(span.clone(), tag.replacement());
    }

    let confidence = if deduped.iter().any(|(_, t)| matches!(t, MatchTag::PersonName | MatchTag::LocationName | MatchTag::OrganizationName)) {
        0.85  // NER confidence floor
    } else { 1.0 };

    let elapsed = start.elapsed();
    metrics::histogram!("brain_pii_scan_duration_seconds", "ner_invoked" => ner_invoked.to_string())
        .record(elapsed.as_secs_f64());
    for (_, tag) in &deduped { metrics::counter!("brain_pii_matches_total", "tag" => format!("{tag:?}")).increment(1); }

    ScanResult { redacted_body: redacted, matches: deduped, confidence, ner_invoked }
}
```

### Ruleset (regex catalogue)

```rust
// services/brain-capture/src/pii/ruleset.rs
use crate::pii::MatchTag;
use once_cell::sync::Lazy;
use regex::Regex;
use std::ops::Range;

struct Rule { pat: Regex, tag: MatchTag }

static RULES: Lazy<Vec<Rule>> = Lazy::new(|| {
    vec![
        // English (subset; full set in source)
        Rule { tag: MatchTag::EmailAddress,  pat: Regex::new(r"\b[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}\b").unwrap() },
        Rule { tag: MatchTag::AwsAccessKey,  pat: Regex::new(r"\bAKIA[0-9A-Z]{16}\b").unwrap() },
        Rule { tag: MatchTag::BearerToken,   pat: Regex::new(r"(?i)Bearer\s+[A-Za-z0-9._\-]{16,}").unwrap() },
        Rule { tag: MatchTag::OpenAiKey,     pat: Regex::new(r"\bsk-[A-Za-z0-9]{32,}\b").unwrap() },
        Rule { tag: MatchTag::AnthropicKey,  pat: Regex::new(r"\bsk-ant-[A-Za-z0-9_-]{32,}\b").unwrap() },
        Rule { tag: MatchTag::GitHubPat,     pat: Regex::new(r"\bgh[oprsu]_[A-Za-z0-9_]{16,}\b").unwrap() },
        Rule { tag: MatchTag::JwtToken,      pat: Regex::new(r"\beyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\b").unwrap() },
        Rule { tag: MatchTag::CreditCard,    pat: Regex::new(r"\b(?:\d[ -]*?){13,16}\b").unwrap() },  // Luhn check applied below
        Rule { tag: MatchTag::Ssn,           pat: Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap() },
        Rule { tag: MatchTag::IpAddress,     pat: Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap() },

        // Vietnamese
        Rule { tag: MatchTag::VnCccd,        pat: Regex::new(r"\b0\d{11}\b").unwrap() },         // Province-prefix `0` + 11 digits; checksum applied below
        Rule { tag: MatchTag::VnMst,         pat: Regex::new(r"\b\d{10}(-\d{3})?\b").unwrap() }, // 10-digit + optional `-3-digit` branch
        Rule { tag: MatchTag::VnPassport,    pat: Regex::new(r"\b[A-Z]\d{8}\b").unwrap() },
        Rule { tag: MatchTag::VnPhoneLocal,  pat: Regex::new(r"\b(?:\+84|0)\d{9}\b").unwrap() },
        Rule { tag: MatchTag::VnBankAccount, pat: Regex::new(r"\b\d{10,18}\b").unwrap() },       // Refined by bank-prefix check in vn_recognisers.rs
        Rule { tag: MatchTag::VnAddress,     pat: Regex::new(r"(?i)\b(?:phường|quận|huyện|tỉnh|thành phố)\s+[^,\n]{2,}").unwrap() },
    ]
});

pub fn scan_with_regex(body: &str) -> Vec<(Range<usize>, MatchTag)> {
    let mut matches = Vec::new();
    for rule in RULES.iter() {
        for m in rule.pat.find_iter(body) {
            // Defense-in-depth: apply per-tag validators
            if !validators::is_valid(rule.tag, &body[m.start()..m.end()]) {
                continue;
            }
            matches.push((m.range(), rule.tag));
        }
    }
    matches
}
```

### Vietnamese recognisers

```rust
// services/brain-capture/src/pii/vn_recognisers.rs
use crate::pii::MatchTag;

pub fn validate_cccd(s: &str) -> bool {
    // 12-digit; first 3 = province code (001..096); checksum: sum(digits) % 10 == last_digit (simplified — real algorithm is more nuanced)
    if s.len() != 12 || !s.chars().all(|c| c.is_ascii_digit()) { return false; }
    let province: u32 = s[..3].parse().unwrap();
    (1..=96).contains(&province)
}

pub fn validate_mst(s: &str) -> bool {
    // 10-digit OR "10-digit-3-digit"; checksum on 10-digit per GDT algorithm
    let core = if s.contains('-') {
        s.split('-').next().unwrap_or("")
    } else { s };
    if core.len() != 10 || !core.chars().all(|c| c.is_ascii_digit()) { return false; }
    let digits: Vec<u32> = core.chars().map(|c| c.to_digit(10).unwrap()).collect();
    let weights = [31, 29, 23, 19, 17, 13, 7, 5, 3, 1];
    // last digit is the check; algorithm: 10 - (sum(d[i]*w[i]) mod 11) mod 10
    let sum: u32 = digits.iter().zip(weights.iter()).take(9).map(|(d, w)| d * w).sum();
    let expected = 10 - ((sum % 11) % 10);
    digits[9] == expected % 10
}

const VN_BANK_PREFIXES: &[&str] = &[
    "0001", // SBV
    "0301", // Vietcombank
    "0421", // Vietinbank
    "1100", // BIDV
    "9300", // Techcombank
    "9704", // VietQR universal
    // ... full list in source
];

pub fn validate_vn_bank_account(s: &str) -> bool {
    s.len() >= 10 && s.len() <= 18 && s.chars().all(|c| c.is_ascii_digit())
        && VN_BANK_PREFIXES.iter().any(|p| s.starts_with(p))
}

pub mod validators {
    use super::*;
    pub fn is_valid(tag: MatchTag, s: &str) -> bool {
        match tag {
            MatchTag::VnCccd        => validate_cccd(s),
            MatchTag::VnMst         => validate_mst(s),
            MatchTag::VnBankAccount => validate_vn_bank_account(s),
            MatchTag::CreditCard    => luhn_valid(s),
            _                       => true,
        }
    }
    fn luhn_valid(s: &str) -> bool {
        let digits: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
        if digits.len() < 13 || digits.len() > 16 { return false; }
        let mut sum = 0;
        for (i, &d) in digits.iter().rev().enumerate() {
            let mut x = d;
            if i % 2 == 1 { x *= 2; if x > 9 { x -= 9; } }
            sum += x;
        }
        sum % 10 == 0
    }
}
```

### Presidio bridge

```rust
// services/brain-capture/src/pii/presidio_bridge.rs
use crate::pii::MatchTag;
use std::ops::Range;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::process::Command;

pub async fn scan(body: &str) -> anyhow::Result<Vec<(Range<usize>, MatchTag)>> {
    let mut child = Command::new("python3")
        .arg("-m").arg("cyberos_presidio")        // installed via requirements.txt
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    let mut stdin  = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();

    tokio::time::timeout(std::time::Duration::from_millis(180), async {
        stdin.write_all(body.as_bytes()).await?;
        stdin.shutdown().await?;
        let mut buf = Vec::new();
        stdout.read_to_end(&mut buf).await?;
        let parsed: PresidioResponse = serde_json::from_slice(&buf)?;
        anyhow::Result::Ok(parsed)
    }).await??.into_matches()
}

#[derive(serde::Deserialize)]
struct PresidioResponse { entities: Vec<PresidioEntity> }
#[derive(serde::Deserialize)]
struct PresidioEntity { entity_type: String, start: usize, end: usize, score: f32 }

impl PresidioResponse {
    fn into_matches(self) -> anyhow::Result<Vec<(Range<usize>, MatchTag)>> {
        Ok(self.entities.into_iter().filter_map(|e| {
            let tag = match e.entity_type.as_str() {
                "PERSON"       => MatchTag::PersonName,
                "LOCATION"     => MatchTag::LocationName,
                "ORGANIZATION" => MatchTag::OrganizationName,
                _              => return None,
            };
            if e.score < 0.6 { return None; }   // confidence floor
            Some((e.start..e.end, tag))
        }).collect())
    }
}
```

### Fixture format

```jsonl
// tests/fixtures/pii-corpus.jsonl  (excerpt)
{"text": "Please contact alice@cyberskill.world for the report.", "expected_matches": [{"tag": "EmailAddress", "start": 15, "end": 39}]}
{"text": "Use API key sk-abc123def456ghi789jkl012mno345pqr678 to test.", "expected_matches": [{"tag": "OpenAiKey", "start": 12, "end": 59}]}
{"text": "No PII here, just a normal sentence.", "expected_matches": []}
```

```jsonl
// tests/fixtures/pii-corpus-vn.jsonl  (excerpt)
{"text": "CCCD: 079123456789 do tổng cục cấp.", "expected_matches": [{"tag": "VnCccd", "start": 6, "end": 18}]}
{"text": "MST công ty là 0312345678-001.", "expected_matches": [{"tag": "VnMst", "start": 15, "end": 29}]}
{"text": "Nhà ở phường Bến Nghé, quận 1, thành phố Hồ Chí Minh.", "expected_matches": [{"tag": "VnAddress", "start": 5, "end": 22}, {"tag": "VnAddress", "start": 24, "end": 31}, {"tag": "VnAddress", "start": 32, "end": 54}]}
```

---

## §4 — Acceptance criteria

1. **Single ruleset is the source of truth** — CI test `pii::tests::no_inline_regexes` greps callers (capture, hook, AI gateway) for `Regex::new\(` outside `pii/`; zero matches.
2. **English email redacted** — `"contact alice@x.com"` → redacted `"contact <EMAIL>"`; one `EmailAddress` match.
3. **CCCD redacted with checksum validation** — `"079123456789"` (valid province) → `<VN_CCCD>`; invalid province like `"999123456789"` → unchanged (FP suppressed by validator).
4. **MST redacted with GDT checksum** — `"0312345678-001"` (passes checksum) → `<VN_MST>`; random `"1234567890"` (fails checksum) → unchanged.
5. **Vietnamese address heuristic catches three forms** — fixture line with `phường`, `quận`, `thành phố` → three `VnAddress` matches.
6. **Bank account requires bank prefix** — `"9704123456789012"` (VietQR prefix) → `<VN_BANK_ACCT>`; `"5555123456789012"` (no prefix) → unchanged.
7. **Names redacted via Presidio (NER path)** — `"Talked to Nguyen Van A about the project"` (no regex matches) → NER invoked → `<PERSON>` redacts "Nguyen Van A".
8. **Regex first; NER skipped when regex finds matches** — input with email AND a name → only email redacted (regex hit); `ner_invoked = false`.
9. **NER skipped on short input** — input < 32 bytes with no regex match → NER not invoked; metric `brain_pii_scan_duration_seconds{ner_invoked="false"}` increments.
10. **Tenant allowlist suppresses match** — tenant configured `allowlist: ["alice@cyberskill\\.world"]`; `"alice@cyberskill.world"` → unchanged; `"alice@other.com"` → redacted.
11. **Overlapping matches deduped (longest wins)** — `"john@example.com"` matches both email regex and a phantom person regex on `john` → only email kept.
12. **Recall ≥ 99.5% on English fixture** — `pii_recall_test::recall_en` loads 500-line corpus → `tp / (tp + fn) ≥ 0.995`.
13. **Recall ≥ 99.5% on Vietnamese fixture** — same for VN corpus.
14. **False-positive rate ≤ 1%** — across both corpora; `fp / (fp + tn) ≤ 0.01`.
15. **Fail-closed on NER subprocess crash** — kill the python subprocess mid-scan → `ScanResult.redacted_body == "<SCAN_FAILED>"`; `brain.capture_pii_scan_failed` row emitted; caller skips chain write.
16. **Scan latency p95 ≤ 50ms regex-only** — 8KB input with regex matches → 100-trial p95 ≤ 50ms.
17. **Scan latency p95 ≤ 200ms NER-invoked** — 8KB input with no regex matches → 100-trial p95 ≤ 200ms.
18. **OTel span emitted per scan** — exporter receives `brain.pii.scan` with `body_bytes`, `match_count`, `ner_invoked`, `duration_ms` attrs.
19. **Metric: per-tag counter** — running fixtures → `brain_pii_matches_total{tag="EmailAddress"}` non-zero; bounded cardinality.
20. **CLI: cyberos brain pii test** — reads file or stdin; prints span-highlighted output (terminal colour) + JSON match list.
21. **Hash invariance under PII change** — same redacted body but different raw PII (e.g. `"<EMAIL>"` substituted for two different real emails) → same blake3 hash; idempotent dedup at FR-BRAIN-107.
22. **AI gateway uses the same ruleset** — FR-AI-011 + FR-AI-012's PII module imports from `cyberos_brain_capture::pii::ruleset`; verified via build-graph test.
23. **Claude Code hook uses the same ruleset** — FR-BRAIN-109's `redact.rs` delegates to `ruleset::scan_with_regex`.

---

## §5 — Verification

```rust
// services/brain-capture/tests/pii_recall_test.rs

#[derive(serde::Deserialize)]
struct CorpusLine {
    text: String,
    expected_matches: Vec<ExpectedMatch>,
}
#[derive(serde::Deserialize, Clone, Copy)]
struct ExpectedMatch { tag: String, start: usize, end: usize }

#[tokio::test]
async fn recall_en_at_least_995() {
    let corpus = load_jsonl("tests/fixtures/pii-corpus.jsonl");
    let (recall, fp_rate) = measure(&corpus).await;
    assert!(recall >= 0.995, "EN recall {recall:.4} < 0.995");
    assert!(fp_rate <= 0.01, "EN FP rate {fp_rate:.4} > 0.01");
}

#[tokio::test]
async fn recall_vn_at_least_995() {
    let corpus = load_jsonl("tests/fixtures/pii-corpus-vn.jsonl");
    let (recall, fp_rate) = measure(&corpus).await;
    assert!(recall >= 0.995, "VN recall {recall:.4} < 0.995");
    assert!(fp_rate <= 0.01,  "VN FP rate {fp_rate:.4} > 0.01");
}

async fn measure(corpus: &[CorpusLine]) -> (f64, f64) {
    let mut tp = 0; let mut fn_ = 0; let mut fp = 0; let mut tn = 0;
    for line in corpus {
        let res = pii::scan_and_redact(&line.text, &[]).await;
        let expected: HashSet<(usize, usize)> = line.expected_matches.iter().map(|m| (m.start, m.end)).collect();
        let actual:   HashSet<(usize, usize)> = res.matches.iter().map(|(r, _)| (r.start, r.end)).collect();
        tp += expected.intersection(&actual).count();
        fn_ += expected.difference(&actual).count();
        fp += actual.difference(&expected).count();
        if expected.is_empty() && actual.is_empty() { tn += 1; }
    }
    let recall = tp as f64 / (tp + fn_) as f64;
    let fp_rate = fp as f64 / (fp + tn) as f64;
    (recall, fp_rate)
}

#[tokio::test]
async fn fail_closed_on_ner_crash() {
    presidio_bridge::testing::inject_crash_on_next_call();
    let res = pii::scan_and_redact("name: Alice", &[]).await;
    assert_eq!(res.redacted_body, "<SCAN_FAILED>");
    assert_eq!(res.confidence, 0.0);
}

#[tokio::test]
async fn ner_skipped_when_regex_finds_match() {
    let res = pii::scan_and_redact("email alice@x.com talks to Bob", &[]).await;
    assert!(res.matches.iter().any(|(_, t)| *t == MatchTag::EmailAddress));
    assert!(!res.ner_invoked);
}

#[tokio::test]
async fn allowlist_suppresses_match() {
    let allow = vec![regex::Regex::new(r"alice@cyberskill\.world").unwrap()];
    let res = pii::scan_and_redact("contact alice@cyberskill.world", &allow).await;
    assert!(res.redacted_body.contains("alice@cyberskill.world"));
    assert!(res.matches.is_empty());
}

#[tokio::test]
async fn cccd_invalid_province_not_redacted() {
    let res = pii::scan_and_redact("not a CCCD: 999123456789", &[]).await;
    assert!(res.redacted_body.contains("999123456789"));
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton.)

---

## §7 — Dependencies

- **FR-BRAIN-107 (upstream)** — capture emit pipeline; PII scan slots in just before blake3.
- **FR-BRAIN-109 (sibling)** — Claude Code hook delegates redaction to this ruleset.
- **FR-AI-011 / FR-AI-012 / FR-AI-013** — AI Gateway redactor + VN PII plugin + recall-floor CI gate; shared ruleset.
- **FR-OBS-007** — alarms when scan_failed_total spikes.

---

## §8 — Example payloads

### `brain.capture_pii_scan_failed`

```json
{
  "kind": "brain.capture_pii_scan_failed",
  "payload": {
    "folder_path":      "/Users/x/notes",
    "body_byte_count":  2418,
    "error":            "ner_subprocess_died",
    "captured_at_ns":   1747407137483000000
  }
}
```

### Scan trace (OTel)

```text
span: brain.pii.scan
  body_bytes: 2418
  match_count: 3
  ner_invoked: false
  duration_ms: 14
  confidence: 1.0
```

---

## §9 — Open questions

All resolved. Deferred:
- Per-folder category override (medical / financial / HR profiles) — §1 #15; slice 3+.
- Self-updating Presidio model (fetch latest on `cyberos doctor --update-models`) — slice 3+; needs offline-bundle distribution design.
- Multi-language NER (FR Korean? Spanish?) — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Regex panic | catch_unwind in `scan_with_regex` | `ScanResult::failed()`; scan_failed_total{reason="regex_panic"} | Operator fixes regex (ruleset.rs PR) |
| Presidio subprocess fails to spawn | Command::spawn Err | scan_failed_total{reason="ner_subprocess_died"} | Operator verifies python3 + requirements.txt installed |
| Presidio subprocess hung | 180ms timeout | scan_failed_total{reason="ner_timeout"} | Operator kills + restarts subprocess pool |
| Presidio OOM | child exit code 137 | scan_failed_total{reason="oom"} | Operator raises memory or splits input |
| Allowlist regex invalid | `Regex::new` Err at manifest load | Daemon refuses to start; stderr WARN | Operator fixes manifest |
| Invalid UTF-8 in input | Already handled at FR-BRAIN-107 boundary | N/A | N/A |
| 1 MB+ input | regex fast; NER skipped (likely > 32 bytes triggers NER but body cap at 1MB) | latency p99 alarm; per-tag matches still emit | Operator considers splitting or accept |
| MST checksum bug | unit tests catch | Recall regression below 99.5%; CI blocked | Operator fixes algorithm |
| CCCD province-table out-of-date | new province → false negative | Recall drops slightly; corpus catches it | Operator updates VN_PROVINCE_CODES |
| Allowlist regex DoS (catastrophic backtrack) | regex crate uses RE2 backend; safe | N/A | N/A |
| Same span matched by two regex (e.g. ip vs phone) | dedup keeps longest | First-seen kept by sort | By design |
| User's own allowlist whitelists all emails (`.+@.+`) | tenant-scoped; no global impact | This tenant's PII not redacted | Operator policy review |
| Capture daemon starts before Presidio installed | Presidio invoke fails on first NER call | scan_failed_total spike; sev-1 alert | Operator installs requirements.txt |
| Multi-byte char span (UTF-8 boundaries) | Rust `str::replace_range` panics on non-boundary | Caught by catch_unwind; scan_failed | Operator files bug; ruleset.rs uses byte ranges from `find_iter` (safe) |
| Recall regression on new ruleset version | CI gate `pii_recall_test` fails | PR blocked | Author tunes regex or adds fixture |
| FP regression | same CI gate | PR blocked | Author tightens regex or adds allowlist |
| Empty body | regex returns empty; NER skipped | `ScanResult::default()`; no audit row | By design |
| Body with only PII (e.g. `"AKIA...."`) | all redacted | redacted body is `"<AWS_KEY>"`; emit proceeds | By design — capture knows that something happened, just not what |
| Fixture corpus drift | CI runs daily | Recall drops because corpus changed | Author rebases ruleset |

---

## §11 — Implementation notes

- The `ruleset.rs` module is the single source of truth. Callers (capture, hook, AI gateway) MUST `use cyberos_brain_capture::pii::ruleset;` — no inline regex literals. CI grep test enforces.
- Presidio is invoked as a subprocess (not pyo3) because: (a) keeps Rust binary deployable as a single static binary; (b) subprocess crash doesn't corrupt the daemon; (c) Presidio updates can be applied via pip without recompiling cyberos.
- The `cyberos_presidio` Python module is a thin wrapper around Microsoft's Presidio (`presidio-analyzer` + `presidio-anonymizer`). Its `requirements.txt` pins versions; `cyberos brain pii update-model` (slice 3+) will refresh.
- VN bank prefixes (`VN_BANK_PREFIXES`) are sourced from NAPAS public registry; the table is small (~40 prefixes) and updated quarterly. Out-of-date prefixes cause false-negatives, not false-positives — safe failure mode.
- CCCD province validation (`validate_cccd`) uses a simplified algorithm; the full Ministry of Public Security algorithm includes a checksum on the last digit. We accept the simplified version because real-world CCCDs almost always pass; false-negatives caught by the corpus.
- MST checksum uses the canonical GDT (General Department of Taxation) algorithm — weight vector `[31, 29, 23, 19, 17, 13, 7, 5, 3, 1]`; verifiable in public GDT docs.
- The fixture format is JSONL not JSON because (a) easy to extend (one line per case), (b) easy to diff in PRs, (c) easy to generate programmatically.
- The 500-example corpora are the MVP; quarterly the security team adds ~50 new examples (real-world false negatives observed in production).
- The Presidio confidence floor at 0.6 was tuned on the corpus; values below 0.6 produced many FPs on edge cases like "Alice Springs" (city, not person).
- The `ner_invoked` attribute on the span lets dashboards build pivots like "what % of capture events trigger NER?" — high values indicate the regex catalogue is missing common patterns; trending it informs ruleset work.

---

*End of FR-BRAIN-111.*
