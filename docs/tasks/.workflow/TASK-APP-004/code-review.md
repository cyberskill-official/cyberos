# TASK-APP-004 — code-review packet (steps 17–18)

Status: `reviewing`. **HALTED at HITL gate 1 (review acceptance).** Diff under review: the `feat(desktop): TASK-APP-004 ...` phase commit (4 new files + 1 lint script; release.yml untouched).

## §1 clause → evidence map (all 8 clauses)

| §1 clause | Requirement | Evidence | Verdict |
|---|---|---|---|
| 1 | MSIX layered over the same build profile, explicit makeappx wrap (Tauri has no native msix target) | Staging step wraps the raw compiled binary (never unpacks NSIS); `makeappx pack` step with exit-code guard | ✅ |
| 2 | Identity declared with unmistakable placeholders, never fabricated real values | `CHANGEME-PENDING-PARTNER-CENTER-RESERVATION` in both fields; lint blocks Store runs while present (proven exit 1) | ✅ |
| 3 | VisualElements references the already-committed tile set; assets not regenerated | All 4 referenced PNGs verified present; AC #8 diff empty; wide tile omitted (asset confirmed absent) | ✅ |
| 4 | `MSSTORE_RELEASE` gate + independent `MSSTORE_SIGNING_MODE` | Job `if:` on the former; both signing steps conditioned only on the latter — never conflated | ✅ |
| 5 | Both Microsoft-supported signing paths, decision deferred | Store-managed = default (no steps run, zero secrets needed); self-managed = opt-in import+sign steps | ✅ |
| 6 | Submission via Store Submission API (client-credentials), inert until Azure AD registration | Token acquisition step per contract; multi-step upload implemented against Microsoft's live docs at first real run (spec's own anti-drift decision, §6/§11) | ✅ contract |
| 7 | No account/cert/registration acquisition by the agent | Nothing acquired; 4-blocker table in the answer sheet; secrets only as `${{ secrets.* }}` | ✅ |
| 8 | Answer sheet incl. IARC (distinct system), privacy URL, identity, API scopes | `microsoft-store-submission.md` — 8 fields + blockers + operational QA notes | ✅ (fields `pending-human`) |

## Implementer-disclosed findings

1. **Skeleton path bug fixed:** §3's staging step copied `src-tauri\...` from repo root — corrected to `apps\desktop\src-tauri\...`.
2. **Binary-name reality:** Cargo produces `cyberos-desktop`; manifest declares `CyberOS.exe`. Tauri v2's rename behavior varies by version → staging resolves either and stages as `CyberOS.exe`, failing loudly with a directory listing otherwise.
3. **Anchor job added** (spec's own AC #4 requires it but §3's skeleton omitted it) + lint runs unconditionally there in inert mode.
4. **Early lint placement:** enforced lint runs before the Rust build — a placeholder manifest fails in seconds.
5. **`modified_files` empty again:** release.yml untouched (standalone workflow), mirroring TASK-APP-003.

## Machine gates

Lint proven in all 3 states; YAML/XML/JSON parses PASS; AC #8 diff empty; AC #1 `makeappx pack` **expected-pending** (needs Windows runner — the workflow step is the standing check); coverage N/A (declared); run-gates floor GREEN.

## Reviewer verdict needed

**"TASK-APP-004 review: approved"** or **"TASK-APP-004 review: rejected — <reason>"**.

*End review packet.*
