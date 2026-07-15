# Microsoft Store submission — answer sheet (TASK-APP-004)

Companion to `docs/deploy/RELEASE.md` (GitHub Releases NSIS channel, unchanged) and the per-store sheets (`play-store-submission.md`, `mac-app-store-submission.md`). The Store channel wraps the same Tauri Windows build in an MSIX (`makeappx.exe` over a staged layout + `AppxManifest.xml`) and submits via the Store Submission API — all behind `MSSTORE_RELEASE=true` (off today) plus the identity-placeholder lint (`tools/msix-identity-lint.sh`), which blocks any Store run until the Partner Center identity is real.

## Hard blockers on `MSSTORE_RELEASE=true`

| # | Blocker | Owner | Status |
|---|---|---|---|
| 1 | Partner Center app identity reservation (name "CyberOS"); swap both `Identity Name` and `Publisher` in `AppxManifest.xml` from the CHANGEME placeholder to the reserved values (Partner Center → Product identity page shows the exact strings) | Stephen (Partner Center account already exists per the 2026-07 store push) | pending-human |
| 2 | Signing-mode decision: `store-managed` (default, no cert purchase — Partner Center re-signs at ingestion) vs `self-managed` (own EV cert, required only for sideload/Intune distribution of the same package) | Stephen | pending-human |
| 3 | If self-managed: EV code-signing cert purchase + secrets `MSSTORE_EV_CERT_PFX_BASE64`, `MSSTORE_EV_CERT_PFX_PASSWORD`, `MSSTORE_EV_CERT_THUMBPRINT`; set repo variable `MSSTORE_SIGNING_MODE=self-managed` | Stephen | pending-human (skippable if store-managed) |
| 4 | Azure AD app registration associated with Partner Center; secrets `MSSTORE_TENANT_ID`, `MSSTORE_CLIENT_ID`, `MSSTORE_CLIENT_SECRET`, `MSSTORE_APP_ID` | Stephen | pending-human (fallback: manual Partner Center upload needs none of these) |

## Partner Center submission answer sheet

| Field | Recommended answer | Status |
|---|---|---|
| Package identity (Name / Publisher / PackageFamilyName) | From the Partner Center reservation — never hand-invented; the lint enforces this | pending-human |
| Age rating (IARC questionnaire — Microsoft's system, distinct from Apple's and Google's) | Answer the IARC questionnaire fresh in Partner Center; expected outcome for a productivity app with no user-generated public content, no gambling, no violence: IARC 3+/E equivalent. Do NOT copy Play Store answers field-by-field — the question sets differ | pending-human |
| Privacy policy URL | `https://os.cyberskill.world/privacy` if live, else the canonical CyberSkill privacy page used for the Play listing — must be reachable at review time | pending-human |
| Category | Productivity (primary); Developer tools (secondary) — matches the other store listings | pending-human |
| Pricing / markets | Free, all markets (matches iOS/Play decisions from the 2026-07 push) | pending-human |
| Store listing copy + screenshots | Reuse the desktop marketing captures; capture real builds, never fabricate | pending-human |
| Device family availability | Windows.Desktop only (manifest `TargetDeviceFamily`; min 10.0.17763) | human-confirmed (structural — set by the manifest this task commits) |
| Capability justification (`runFullTrust`) | Structural requirement of packaging any Win32/Tauri/WebView2 app via Desktop Bridge — not optional, not a CyberOS-specific choice (spec §9) | human-confirmed (structural) |

## Operational notes + manual QA checklist

- **Stale `PendingCommit` submission** (interrupted CI run): discard the draft in Partner Center web UI before retrying — the submission flow is deliberately not resume-idempotent (spec §10).
- **Azure AD client secrets expire** (6–24 months typical): rotation of `MSSTORE_CLIENT_SECRET` is routine ops, tracked here.
- **Tile transparency QA (manual, per-submission):** check the Partner Center preview for a visible background box around the logo — if present, the tile PNGs lack alpha; regenerating them is a separate asset task, not this pipeline's job (spec §10 cosmetic row).
- **Local pack-only verification** produces an unsigned `.msix`; SmartScreen warnings on double-clicking it are expected and irrelevant — the verification only checks `makeappx pack` exit 0, never installs (spec §10).
- The MSIX staging uses the raw compiled binary (`target/release/CyberOS.exe`, falling back to the Cargo-named `cyberos-desktop.exe` on older Tauri versions, staged as `CyberOS.exe` either way) — it never unpacks the NSIS installer (spec §2).
- Wide tile (`Wide310x150Logo.png`) deliberately absent: the asset doesn't exist in the committed icon set; adding one later is a pure asset addition + one manifest attribute (spec §11).
