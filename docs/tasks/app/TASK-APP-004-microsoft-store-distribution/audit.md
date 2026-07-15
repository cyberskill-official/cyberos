---
task_id: TASK-APP-004
audited: 2026-07-12
verdict: PASS (after revision)
score_pre_revision: 6/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 7
template: engineering-spec@1
---

## §1 — Verdict summary

366 lines, 8 numbered §1 clauses, 8 acceptance criteria, 9 failure-mode rows, 3 verification blocks (bash + YAML assertion), a fully-specified gating script (`msix-identity-lint.sh`) in §6. Initial draft (301 lines) had a real internal inconsistency between its own manifest example and its own acceptance criteria (referencing an icon file it had separately flagged as unconfirmed), a CI signing surface missing the certificate-import step exactly analogous to TASK-APP-003's ISS-002, a hardcoded/fragile SDK tool path, an acceptance-criterion-gating script left unspecified, and a missing forward-reference disclosure. All findings below were resolved in the same authoring pass before this audit was finalized, per the master rule's loop-to-10/10 discipline.

## §2 — Findings (all resolved)

### ISS-001 — Manifest example referenced an icon file its own failure-modes table flagged as unconfirmed
§3's `AppxManifest.xml` skeleton declared `<uap:DefaultTile Wide310x150Logo="Assets\Wide310x150Logo.png" .../>` while §10's failure-modes table separately, correctly, noted that `Wide310x150Logo.png` was not among the confirmed-committed Tauri-generated icon assets — a genuine self-contradiction between the "API contract" and the task's own risk disclosure, and one that would have made AC #2 (icon references resolve) fail on the very manifest this task ships. Resolved: removed the `Wide310x150Logo` attribute from `<uap:DefaultTile>`, kept the confirmed `Square310x310Logo` attribute (schema-valid minimum), and added an inline XML comment explaining the omission; §3, §11.

### ISS-002 — Self-managed signing step assumed the EV certificate was already present in the runner's certificate store
The original `§3` sign step referenced `MSSTORE_EV_CERT_THUMBPRINT` directly against `signtool.exe` with no step showing how that certificate gets into the `windows-2022` runner's certificate store — a real implementer following the skeleton literally would hit "No certificates were found that met all the given criteria" on the first self-managed-signing CI run. This is the Windows-side analogue of TASK-APP-003's ISS-002 (macOS keychain import) and was an equally real gap. Resolved: added an explicit "Import EV signing certificate" step that decodes a base64-encoded PFX secret and imports it via `Import-PfxCertificate` before the sign step runs; §3, and a corresponding failure-mode row; §10.

### ISS-003 — `makeappx.exe`/`signtool.exe` paths were hardcoded to one specific Windows SDK version string
The original CI skeleton hardcoded `...\bin\10.0.22621.0\x64\makeappx.exe` and the same version for `signtool.exe`. Windows SDK versions installed on GitHub-hosted `windows-2022` runner images change across image updates outside CyberOS's control; a hardcoded version string is a latent, silent future breakage. Resolved: replaced with a "Locate Windows SDK tools" step that discovers the installed SDK bin directory dynamically via `Get-ChildItem`/regex + `Sort-Object -Descending` (newest wins), asserts the resolved `makeappx.exe` actually exists, and passes both tool paths to later steps via `$GITHUB_OUTPUT`; §3, and the corresponding failure-mode row was rewritten to describe the now-narrower residual risk (no SDK at all, rather than wrong-version SDK); §10.

### ISS-004 — `tools/msix-identity-lint.sh`, which gates AC #3, was referenced but never specified
AC #3 and §5's verification block both depend on a script (`tools/msix-identity-lint.sh`) whose actual logic was never shown anywhere in the spec — unlike the Microsoft Store Submission API flow (correctly left to WORKER-phase implementation per §11, since it's an external, versioned, well-documented contract), this script is CyberOS-specific gating logic central to keeping the "inert by default" guarantee honest, and leaving it unspecified was a real gap, not a legitimate scope boundary. Resolved: added the full script to §6, including the `MSSTORE_RELEASE` inertness check so the same lint used in local verification (AC #1, unset `MSSTORE_RELEASE`) doesn't false-fail; §6.

### ISS-005 — `MSSTORE_SIGNING_MODE` was used in AC #5 and the CI skeleton but never declared as a config surface in §1
The signing-mode repo variable appeared first in §1 #5's prose and then directly in AC #5/§3's YAML without ever being named alongside `MSSTORE_RELEASE` as a first-class, independent gate a future maintainer needs to know about. Resolved: §1 #4 now explicitly names `MSSTORE_SIGNING_MODE` as a second, independent config surface and states it must not be conflated with `MSSTORE_RELEASE`; §1.

### ISS-006 — §2's "wrap the NSIS output" framing was ambiguous about which artifact gets staged into the MSIX layout
As originally worded, "MSIX-wrap the NSIS output" could be misread as "unpack the NSIS `.exe` installer's contents" rather than the actual mechanism (stage the raw `cargo`/`tauri build` binary directly, using the same build profile NSIS also consumes). This ambiguity could have led a WORKER-phase implementer to attempt extracting files from the NSIS installer rather than building from source, an unnecessary and fragile detour. Resolved: §2's rationale paragraph now states explicitly that the MSIX layout is staged from the raw compiled binary at `target/release/<binary-name>.exe`, never by extracting from the NSIS wrapper, and clarifies what "wrap the NSIS output" means in this task's context (reusing the same build invocation/profile, not the installer artifact itself); §2.

### ISS-007 — `related_tasks` references TASK-APP-005/TASK-APP-006, which don't exist on disk yet, with no forward-reference disclosure
Same category of finding as TASK-APP-003's ISS-006: this task's siblings from the same approved 5-task PLAN hadn't been authored yet at the time this task was written. `depends_on`/`blocks` are both empty (the fields the repo's placeholder-annotation rule actually scopes to), so no inline placeholder comment is mechanically required, but leaving the forward reference undisclosed in §9 would strand a future reader. Resolved: added an explicit §9 note documenting the deliberate same-batch forward reference, matching the disclosure pattern TASK-APP-003 §9 used; §9.

## §3 — Resolution

All 7 findings addressed in the same authoring session that produced them, per the master rule (author → audit → loop to 10/10 before starting the next task). No findings deferred. **Score = 10/10.**

---

*End of TASK-APP-004 audit.*
