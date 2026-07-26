# TASK-DOCS-021 — Mothership violation triage (timeboxed)

Date: 2026-07-27
Source feed: `docs/status/data/status-feed.json` @ `51041133` (`fp-40f196d86b41`, VERSION 1.11.0)
Cutoff (gate): `14e4ee5551d19af743739ed7ab9a0727ec89ef48` (PR #171 merge)
Task: TASK-DOCS-021

## Counts

| Bucket | Count | Gate impact |
| --- | ---: | --- |
| Current-epoch unlinked (`lg` false) | 148 | None — all are **at/before cutoff** by construction |
| First-epoch unlinked (`lg` true) | 296 | None — folded history |
| Total unlinked on page | 444 | Page truth only; CI range after cutoff is clean |

## Operator disposition (LOCKED 2026-07-27)

**ACCEPT ALL 148 current-epoch rows as pre-cutoff history.**  
No mass recovery via `docs/tasks/_state/commit-links.yaml`.

Rationale (confirmed by operator Stephen):

1. Traceability CI/hooks only fail commits **newer than** the cutoff; these 148 never trip the gate.
2. Plan Phase 4 marks ledger backfill as **optional truth-recovery**, not a release blocker.
3. Timebox (~agent pass): full human mapping of 148 subjects → tasks is low ROI vs P2/P6/P7 gates.
4. Release target is **1.12.0** (not 2.0.0); accept-all still holds — page truth recovery remains optional later.

### Optional follow-up (not blocking)

| Disposition tag | Count | Meaning |
| --- | ---: | --- |
| `optional_ledger_if_task_known` | 75 | heuristic from subject only |
| `accept_pre_cutoff` | 70 | heuristic from subject only |
| `accept_pre_cutoff_style` | 3 | heuristic from subject only |

If Stephen wants page-truth recovery later: pick `optional_ledger_if_task_known` rows that still matter,
confirm the TASK id exists, and append to `docs/tasks/_state/commit-links.yaml` via normal PR review.
Never rewrite git history.

## Subject-prefix histogram (current-epoch 148)

| Prefix | n |
| --- | ---: |
| `feat` | 8 |
| `docs(release)` | 8 |
| `docs(deploy)` | 7 |
| `fix(rename)` | 7 |
| `fix(ci)` | 5 |
| `Fix` | 4 |
| `Add` | 4 |
| `docs(audit)` | 4 |
| `style` | 3 |
| `fix(install)` | 3 |
| `docs` | 3 |
| `fix(web)` | 3 |
| `fix(release)` | 3 |
| `ci(npm)` | 3 |
| `refactor(rename)` | 3 |
| `feat(cyberos-init)` | 3 |
| `feat(ten)` | 2 |
| `Enhance` | 2 |
| `fix(auth)` | 2 |
| `fix(tests)` | 2 |

## Full current-epoch ledger (148)

| SHA | Date | Release | Subject | Heuristic |
| --- | --- | --- | --- | --- |
| `bb2b974e` | 2026-07-26 | 1.9.0 | feat(ten): host-b — auth api_calls metering emit at verify_jwt (#166) | `optional_ledger_if_task_known` |
| `ee62fb3a` | 2026-07-26 | 1.8.0 | feat(ten): host-a — plan-change HTTP + ai_tokens metering emit (#165) | `optional_ledger_if_task_known` |
| `b65a1a5e` | 2026-07-25 | 1.6.0 | docs(batch-8): close Wave 0 residual in parent ledger (#149) | `optional_ledger_if_task_known` |
| `ac1f5e31` | 2026-07-24 | 1.5.0 | style: cargo fmt for obs-router alertmanager wiring test | `accept_pre_cutoff_style` |
| `64a763e4` | 2026-07-24 | 1.5.0 | style: cargo fmt for sep986 residual tests | `accept_pre_cutoff_style` |
| `5c75769d` | 2026-07-24 | 1.5.0 | docs(deploy): Foglamp handoff — local done, production options | `optional_ledger_if_task_known` |
| `dcc32385` | 2026-07-23 | 1.4.0 | feat(web): instrument chat AI with Foglamp (AI SDK v7) | `optional_ledger_if_task_known` |
| `4a3a34d5` | 2026-07-23 | 1.3.0 | chore(web): add Vercel AI SDK (ai@7) | `accept_pre_cutoff` |
| `7120a598` | 2026-07-23 | 1.3.0 | fix(test): pin full rollout SHA256SUMS regex in t08 | `optional_ledger_if_task_known` |
| `daee97a7` | 2026-07-23 | 1.3.0 | fix(ci): fold batch/9 notes into 1.2.0 so top-entry token checks pass | `optional_ledger_if_task_known` |
| `33b461f0` | 2026-07-23 | 1.2.0 | fix(install): point scaffold README at TASK-TEMPLATE.md | `optional_ledger_if_task_known` |
| `46ceb8b4` | 2026-07-23 | 1.2.0 | fix(caf): restore per-line CONFIG blocks in eval fixtures | `optional_ledger_if_task_known` |
| `7d488f31` | 2026-07-23 | 1.2.0 | docs: sweep post-1.1.0 cs rename leftovers | `accept_pre_cutoff` |
| `4ed4a109` | 2026-07-22 | 1.1.0 | Fix grok sync-host-plugins failing with repo already installed | `accept_pre_cutoff` |
| `d077654e` | 2026-07-22 | 1.1.0 | Point /install command at tools/install/README.md as the single reference | `accept_pre_cutoff` |
| `f8a8dbcf` | 2026-07-22 | 1.1.0 | Consolidate install instructions; fix dead GUIDE.md links | `accept_pre_cutoff` |
| `5fb23e92` | 2026-07-22 | 1.1.0 | Fix stale docs-site domain links in docs/README | `accept_pre_cutoff` |
| `7e1b2a5f` | 2026-07-22 | 1.1.0 | Fix stale docs-site domain link in README | `accept_pre_cutoff` |
| `95b210d8` | 2026-07-22 | 1.1.0 | Fix stale docs-site domain and consolidate install instructions | `accept_pre_cutoff` |
| `29609be2` | 2026-07-22 | 1.1.0 | Revise plan: public CLI renames to cs (not cyberos-memory) | `accept_pre_cutoff` |
| `f116b0a9` | 2026-07-22 | 1.1.0 | Add plan: CLI module namespacing (fix cyberos name collision) | `accept_pre_cutoff` |
| `3205812a` | 2026-07-22 | 1.1.0 | Remove application identifier from entitlements file | `accept_pre_cutoff` |
| `f1d8f95a` | 2026-07-22 | 1.0.9 | Add application identifier to Entitlements.mas.plist | `accept_pre_cutoff` |
| `656182a5` | 2026-07-22 | 1.0.9 | Add LSApplicationCategoryType to Info.plist | `accept_pre_cutoff` |
| `fbe96c91` | 2026-07-22 | 1.0.9 | Update application identifier in tauri.conf.json | `accept_pre_cutoff` |
| `db9dd182` | 2026-07-22 | 1.0.9 | Clean up debug output in release-mas.yml | `accept_pre_cutoff` |
| `5bb7ce5c` | 2026-07-22 | 1.0.9 | Enhance debug output for keychain identities | `accept_pre_cutoff` |
| `53e09cd3` | 2026-07-22 | 1.0.9 | Enhance debug output for mas-build.keychain certificates | `accept_pre_cutoff` |
| `1701c059` | 2026-07-22 | 1.0.9 | Add debug identity dump in MAS build keychain | `accept_pre_cutoff` |
| `9987d362` | 2026-07-22 | 1.0.8 | fix: release-mas | `accept_pre_cutoff` |
| `658695dc` | 2026-07-22 | 1.0.8 | Set default keychain in release workflow | `accept_pre_cutoff` |
| `01d66263` | 2026-07-22 | 1.0.8 | Delete embedded.provisionprofile in release workflow | `accept_pre_cutoff` |
| `d1c774f2` | 2026-07-22 | 1.0.8 | Write MAS provisioning profile in release workflow | `accept_pre_cutoff` |
| `486629d8` | 2026-07-22 | 1.0.8 | Add embedded provision profile to macOS config | `accept_pre_cutoff` |
| `a25bd486` | 2026-07-21 | 1.0.8 | chore(winget): stamp manifests to v1.0.7, bump schema to 1.12.0 | `accept_pre_cutoff` |
| `a15c288a` | 2026-07-21 | 1.0.8 | chore(homebrew): stamp Cask draft to v1.0.7 | `accept_pre_cutoff` |
| `5a95b2d8` | 2026-07-21 | 1.0.7 | fix(web): stop the message action bar covering text on touch devices | `optional_ledger_if_task_known` |
| `8ae081e4` | 2026-07-21 | 1.0.6 | fix(web): separate the sign-in footer links | `optional_ledger_if_task_known` |
| `7a165fd7` | 2026-07-21 | 1.0.6 | docs(deploy): switch the store review account to demo@cyberskill.world | `optional_ledger_if_task_known` |
| `83170b2f` | 2026-07-20 | 1.0.5 | chore(mobile): register @capacitor/app in the native projects | `accept_pre_cutoff` |
| `06334c94` | 2026-07-20 | 1.0.5 | fix(auth): let users choose which Google account to sign in with | `optional_ledger_if_task_known` |
| `adb65b20` | 2026-07-20 | 1.0.4 | fix(auth): make Google sign-in work in the native shells | `optional_ledger_if_task_known` |
| `43371328` | 2026-07-20 | 1.0.3 | chore(ios): declare export compliance in Info.plist | `accept_pre_cutoff` |
| `08fa364c` | 2026-07-20 | 1.0.3 | fix(web): correct mobile layout on notched viewports | `optional_ledger_if_task_known` |
| `08fd7e9e` | 2026-07-20 | 1.0.2 | fix(mobile): give Capacitor builds an absolute API origin | `optional_ledger_if_task_known` |
| `56202557` | 2026-07-20 | 1.0.2 | docs(deploy): correct stale go-live checklist and Play UGC blockers | `optional_ledger_if_task_known` |
| `44893a05` | 2026-07-20 | 1.0.2 | fix(release): rebuild latest.json after the desktop matrix, not inside it | `optional_ledger_if_task_known` |
| `05046815` | 2026-07-20 | 1.0.2 | feat: ready to release | `accept_pre_cutoff` |
| `41b3db36` | 2026-07-20 | 1.0.2 | feat: ready to release | `accept_pre_cutoff` |
| `cb318fe0` | 2026-07-20 | 1.0.2 | fix(release): linked assets, docs-first install, first-release changelog; quarantine legacy 1.x changelog | `optional_ledger_if_task_known` |
| `182bb756` | 2026-07-20 | 1.0.2 | docs(audit): record build-payload contamination breaking rules_sha reproducibility | `optional_ledger_if_task_known` |
| `069d4dff` | 2026-07-20 | 1.0.2 | docs: unwrap hard-wrapped markdown to one line per paragraph | `accept_pre_cutoff` |
| `2bf61900` | 2026-07-20 | 1.0.2 | docs(release): check B4 - 20/20, release pipeline verified end to end | `optional_ledger_if_task_known` |
| `76090919` | 2026-07-19 | 1.0.2 | fix(install): an orphaned status hook stands down instead of blocking commits | `optional_ledger_if_task_known` |
| `61d87349` | 2026-07-19 | 1.0.2 | docs(release): record B3 evidence, row stays open | `optional_ledger_if_task_known` |
| `40689b91` | 2026-07-19 | 1.0.2 | chore(machine): re-vendor .cyberos to the shipped payload rules | `accept_pre_cutoff` |
| `83a52299` | 2026-07-19 | 1.0.2 | docs(release): check D1 + D2 - changelog block final, GUIDE documents all 8 CLI commands | `optional_ledger_if_task_known` |
| `0ed94852` | 2026-07-19 | 1.0.2 | docs(release): check C1 - fresh channel-matrix research pass (2026-07-19) | `optional_ledger_if_task_known` |
| `fc83f6dc` | 2026-07-19 | 1.0.2 | docs(release): check A1 (operator confirmed) + draft the 1.0.0 hardening changelog block | `optional_ledger_if_task_known` |
| `1ce2fb90` | 2026-07-19 | 1.0.2 | docs(release): check D3 fresh-clone consumer test (22/22, 100% coverage) - 14/20 checked, all agent rows done | `optional_ledger_if_task_known` |
| `545279c6` | 2026-07-19 | 1.0.2 | docs(release): check A6, B1, B2 with measured evidence (13/20 checked) | `optional_ledger_if_task_known` |
| `6d91eefe` | 2026-07-19 | 1.0.2 | fix(tests): make the suite runnable on macOS (bash 3.2 + BSD userland) | `optional_ledger_if_task_known` |
| `7c0a726c` | 2026-07-19 | 1.0.2 | docs(audit): open post-1.0.0 improvement backlog; park TRACE-006 findings out of the release path | `optional_ledger_if_task_known` |
| `8c50523a` | 2026-07-19 | 1.0.2 | docs(audit): TRACE-006 remediation plan (453 findings -> ~50 tasks + 6 dispositions) | `optional_ledger_if_task_known` |
| `cd3a0f36` | 2026-07-19 | 1.0.2 | docs(audit): TRACE-006 emit sweep results (355 clauses; 16 PASS / 54 WEAK / 256 INSUFFICIENT / 29 N/A) | `optional_ledger_if_task_known` |
| `481a4be6` | 2026-07-19 | 1.0.2 | audit: TRACE-006 acute-verb sweep results (234 clauses; 64 PASS / 31 WEAK / 112 INSUFFICIENT / 27 N/A) | `accept_pre_cutoff` |
| `06778c6b` | 2026-07-19 | 1.0.2 | Finding #9 rules_sha (stored-not-recomputed, docs-tools-excluded): recorded as accepted by-design per 2026-07-19 PLAN gate. Both h | `accept_pre_cutoff` |
| `9873d389` | 2026-07-19 | 1.0.2 | Goal 3: fleet drift scan - 24 installs, all behind platform source (070bcaa5); 3 generations, 22/24 on oldest 2-tool machine; sach | `accept_pre_cutoff` |
| `6d8d4af0` | 2026-07-17 | 1.0.2 | feat: cleanup | `accept_pre_cutoff` |
| `b1b4b2db` | 2026-07-17 | 1.0.2 | PR #53: document the one thing the reviewers found that was not a defect | `accept_pre_cutoff` |
| `0c7ca5a1` | 2026-07-17 | 1.0.2 | PR #53: my suites asserted a gitignored artifact, so they passed only on my machine | `accept_pre_cutoff` |
| `36309ecf` | 2026-07-17 | 1.0.2 | feat: cleanup | `accept_pre_cutoff` |
| `02a97cc1` | 2026-07-17 | 1.0.2 | feat: cleanup | `accept_pre_cutoff` |
| `ca6caca5` | 2026-07-17 | 1.0.2 | review round 6: two overstated claims, one of them made by the fix for the last review | `accept_pre_cutoff` |
| `41854cae` | 2026-07-17 | 1.0.2 | review round 4: the trap released a lock it no longer owned; my fix lived in a heredoc | `accept_pre_cutoff` |
| `eb3fe2e1` | 2026-07-17 | 1.0.2 | feat: fix comment | `accept_pre_cutoff` |
| `d31d2ef1` | 2026-07-17 | 1.0.2 | feat: push | `accept_pre_cutoff` |
| `13cae6cd` | 2026-07-17 | 1.0.2 | fix(review): batch-4 Devin findings - lease reboot-wedge guards, clean nested --out, doc truthfulness | `optional_ledger_if_task_known` |
| `81ac11a3` | 2026-07-16 | 1.0.2 | fix(payload): vendor per-type rubric families (contracts/task/rubrics) into cuo/rubrics/ | `optional_ledger_if_task_known` |
| `a882e705` | 2026-07-16 | 1.0.2 | docs(ship-tasks): §11a swarm — shared-tree gates belong to the parent; sub-agents verify per-cone only (v2.6.1) | `optional_ledger_if_task_known` |
| `feff8cef` | 2026-07-16 | 1.0.2 | fix(install): summary reports only rendered status-page parts; truthful else-branch cause | `optional_ledger_if_task_known` |
| `e370b5d4` | 2026-07-16 | 1.0.2 | fix(ci): ignore optional semantic_dedup tests without embed model | `optional_ledger_if_task_known` |
| `dbd3ac7e` | 2026-07-16 | 1.0.2 | fix(ci): regenerate memory schema; add pytest-asyncio for voice CI | `optional_ledger_if_task_known` |
| `6bbb5fce` | 2026-07-16 | 1.0.2 | fix(ci): point voice+doc-consistency at modules/memory | `optional_ledger_if_task_known` |
| `10ae9c65` | 2026-07-16 | 1.0.2 | fix(ci): voice numpy dep + soft-fail Play publish flake | `optional_ledger_if_task_known` |
| `17cef40a` | 2026-07-16 | 1.0.2 | test(channels): compare the npm job's Node version instead of matching its spelling | `accept_pre_cutoff` |
| `5f9f8526` | 2026-07-16 | 1.0.2 | chore(ci): upgrade Actions to Node-24 majors (checkout/setup-node@v7) | `accept_pre_cutoff` |
| `bd0588b2` | 2026-07-16 | 1.0.2 | chore(ci): pin Node 24.18.0; upgrade Actions to Node-24 majors | `accept_pre_cutoff` |
| `eda3e078` | 2026-07-16 | 1.0.2 | chore: require Node.js >=24; remove dangling agent skill links | `accept_pre_cutoff` |
| `24fe4f34` | 2026-07-16 | 1.0.2 | ci(npm): drop a stale comment the OIDC gate could not see, and widen it so it can | `accept_pre_cutoff` |
| `81caf281` | 2026-07-16 | 1.0.2 | ci(npm): trusted publishing via OIDC — delete the token, pin the trust to this workflow | `accept_pre_cutoff` |
| `873d4950` | 2026-07-16 | 1.0.2 | chore(status): regenerate the status pages after the vocabulary purge | `accept_pre_cutoff` |
| `c4cd4ca6` | 2026-07-16 | 1.0.2 | ci(npm): --provenance needs id-token:write; the publish never reached the registry | `accept_pre_cutoff` |
| `3a318ee7` | 2026-07-16 | 1.0.2 | fix(rename): the shipped plugin told users to run /init, a command that does not exist | `optional_ledger_if_task_known` |
| `acb147a0` | 2026-07-16 | 1.0.2 | ci(channels): prove docker + github-action + npx-cli at release; publish the npm package | `accept_pre_cutoff` |
| `2f17070e` | 2026-07-16 | 1.0.2 | feat(channels): gate the 13 declared channels; the MCP one advertised a tool that did not exist | `optional_ledger_if_task_known` |
| `27be3dad` | 2026-07-16 | 1.0.2 | rename: last three init/fr filenames the sweeps could not see | `accept_pre_cutoff` |
| `0bfaa5a1` | 2026-07-16 | 1.0.2 | fix(fr): the status page's row-click was dead in every repo; 4 JSON keys + a CLI flag | `optional_ledger_if_task_known` |
| `b96e720d` | 2026-07-16 | 1.0.2 | docs: two stale 'init' copy strings; `init` survives only as another tool's command | `accept_pre_cutoff` |
| `8e72ce55` | 2026-07-16 | 1.0.2 | fix(residue): rollout's plugin probe was always NO; 40+ comments named dead identifiers | `optional_ledger_if_task_known` |
| `8170ecd8` | 2026-07-16 | 1.0.2 | fix(channels): the desktop app and the docker image were both dead on init.sh | `optional_ledger_if_task_known` |
| `c02e9085` | 2026-07-16 | 1.0.2 | chore(1.0.0): retire 5 spent one-shots; keep the 2 the installer actually runs | `accept_pre_cutoff` |
| `c217763a` | 2026-07-16 | 1.0.2 | feat(migrate): one-shot fr->task for the fleet; 24/24 repos green | `optional_ledger_if_task_known` |
| `409cdcc0` | 2026-07-16 | 1.0.2 | refactor(cli): one bin `cyberos <command>`; children stop repeating the repo name | `accept_pre_cutoff` |
| `f563e438` | 2026-07-16 | 1.0.2 | refactor(rename): cyberos-init -> cyberos-install; finish the init->install verb | `accept_pre_cutoff` |
| `58ca5264` | 2026-07-16 | 1.0.2 | fix(rename): close the residue class — the vocabulary gate could not see renames | `optional_ledger_if_task_known` |
| `697ee23e` | 2026-07-16 | 1.0.2 | feat(cuo): ship-tasks 2.6.0 — swarm batching, OS control, batch branches | `optional_ledger_if_task_known` |
| `db07f29a` | 2026-07-15 | 1.0.2 | style: cargo fmt | `accept_pre_cutoff_style` |
| `fe57714a` | 2026-07-15 | 1.0.2 | fix(macos): 3 real macOS bugs the newly-gated tests exposed | `optional_ledger_if_task_known` |
| `b466f354` | 2026-07-15 | 1.0.2 | fix(init): payload shipped no per-type templates; both fr contracts renamed | `optional_ledger_if_task_known` |
| `eb930a96` | 2026-07-15 | 1.0.2 | fix(tests): portable sed + timeout; drop doubled data/data tree | `optional_ledger_if_task_known` |
| `0d6125f2` | 2026-07-15 | 1.0.2 | fix(rename): 3 live regressions + un-orphan every shell test suite | `optional_ledger_if_task_known` |
| `c9e05833` | 2026-07-15 | 1.0.2 | refactor(rename): complete fr->task migration | `accept_pre_cutoff` |
| `f3e17e9f` | 2026-07-15 | 1.0.2 | fix(rename): idempotent BRAIN applier + verify exemptions; wire type discriminator | `optional_ledger_if_task_known` |
| `34b46d7c` | 2026-07-15 | 1.0.2 | feat: updates | `accept_pre_cutoff` |
| `e37dc420` | 2026-07-14 | 1.0.2 | fix(rename): rewrite filename references for every moved script | `optional_ledger_if_task_known` |
| `40c6c385` | 2026-07-14 | 1.0.2 | fix(rename): un-nest task contract; finish agent symlinks; guard one-shot path rename | `optional_ledger_if_task_known` |
| `6bba768b` | 2026-07-14 | 1.0.2 | fix(rename): skip tracked symlinks; retarget agent skill links; disarm one-shot rules | `optional_ledger_if_task_known` |
| `11628138` | 2026-07-14 | 1.0.2 | refactor(rename): feature-request -> task, task -> subtask | `accept_pre_cutoff` |
| `81322b72` | 2026-07-14 | 1.0.2 | chore(rename): deny-list sealed held-out tests; report via --sealed | `accept_pre_cutoff` |
| `8bc5e30e` | 2026-07-14 | 1.0.2 | chore(rename): add fr->task codemod + impact analysis | `accept_pre_cutoff` |
| `ba22529b` | 2026-07-14 | 1.0.2 | docs(release): CHANGELOG 1.0.0 + asset-guide notes; status hub sync | `optional_ledger_if_task_known` |
| `7fb1f709` | 2026-07-14 | 1.0.2 | feat(cyberos-init): final CLI — install, uninstall, version, status, help | `optional_ledger_if_task_known` |
| `bb0f2392` | 2026-07-14 | 1.0.2 | feat(cyberos-init): 1.0.0 CLI surface — install/uninstall/update/status | `optional_ledger_if_task_known` |
| `0a7553f2` | 2026-07-14 | 1.0.2 | Drop migrate-frs shim; wire update-check on every .cyberos entrypoint | `accept_pre_cutoff` |
| `b67e8618` | 2026-07-14 | 1.0.2 | feat(init): combine FR migrate into init; fleet-init-test; status-hook v2 | `optional_ledger_if_task_known` |
| `d361a1d1` | 2026-07-14 | 1.0.2 | fix(init): status-hook v2 — block commit if docs/status regen fails | `optional_ledger_if_task_known` |
| `f023e67f` | 2026-07-14 | 1.0.2 | fix(deploy): fail-fast preflight if VPS_SSH_KEY doesn't parse (deploy #132 retry) | `optional_ledger_if_task_known` |
| `d563de8a` | 2026-07-14 | 1.0.2 | fix(deploy): force ssh -4 in ship.sh (ENETUNREACH on docs deploy #132) | `optional_ledger_if_task_known` |
| `696334eb` | 2026-07-13 | 1.0.2 | docs(deploy): update GO-LIVE-CHECKLIST.md after v1.0.0 re-tag | `optional_ledger_if_task_known` |
| `461cc87e` | 2026-07-13 | 1.0.2 | fix(release): rename deprecated upload-google-play track: to tracks: | `optional_ledger_if_task_known` |
| `24931641` | 2026-07-13 | 1.0.2 | chore(fleet): commit-fleet.sh - stage ONLY CyberOS-owned paths, never a repo's own work | `accept_pre_cutoff` |
| `3bfa80e6` | 2026-07-13 | 1.0.2 | fix(status): pre-commit auto-sync was silently dead; pin the provenance stamp | `optional_ledger_if_task_known` |
| `ac33beb5` | 2026-07-13 | 1.0.2 | feat(docs): status-hub@2 - one page, three lenses, full-spec drawer | `optional_ledger_if_task_known` |
| `fc12ef8f` | 2026-07-13 | 1.0.2 | docs(deploy): draft MS Store listing copy (GO-LIVE §5.4) | `optional_ledger_if_task_known` |
| `8a9c0c12` | 2026-07-13 | 1.0.2 | docs(deploy): rework GO-LIVE-CHECKLIST.md as full todo with agent-task labels | `optional_ledger_if_task_known` |
| `03101108` | 2026-07-13 | 1.0.2 | docs(deploy): trim GO-LIVE-CHECKLIST.md to Stephen-only manual actions | `optional_ledger_if_task_known` |
| `1b6ffcc8` | 2026-07-13 | 1.0.2 | test(fr): testing phase green for all 5 FRs - halted at final acceptance gate (HITL gate 2) | `accept_pre_cutoff` |
| `90633bb0` | 2026-07-13 | 1.0.2 | docs(fr): backlog counters - 5 FRs at reviewing (batch parked at HITL gate 1) | `optional_ledger_if_task_known` |
| `a6a2f3db` | 2026-07-13 | 1.0.2 | feat(cyberos-init): per-repo docs/status page, auto-sync hooks, migration hardening | `optional_ledger_if_task_known` |
| `28590db6` | 2026-07-12 | 1.0.2 | ci(release): select newest stable Xcode for ios + build the tag on dispatch | `accept_pre_cutoff` |
| `33dad99d` | 2026-07-12 | 1.0.2 | ci(ios): split archive and export into two gym calls - no flag bleed into exportArchive | `accept_pre_cutoff` |
| `c13cad80` | 2026-07-12 | 1.0.2 | ci(deploy+ios): restore deploy.yml (docs job swallowed by over-greedy patch) + archive-unsigned/export-signed iOS lane | `accept_pre_cutoff` |
| `223b07dc` | 2026-07-12 | 1.0.2 | ci(deploy): stream the docs site over ssh - the scp+/tmp path truncated the grown site (1.0.0 docs failures) | `accept_pre_cutoff` |
| `0e4d70bb` | 2026-07-12 | 1.0.0 | fix(cyberos-init): hardening closeout - t04 at-rest guard + init.sh --check redirect wart (13/13 suites) | `optional_ledger_if_task_known` |
| `12a22f9d` | 2026-07-12 | 0.4.0 | fix(ios): remove manual CODE_SIGN_IDENTITY overrides - automatic signing + ASC API key owns identity | `optional_ledger_if_task_known` |
| `2b88f11c` | 2026-07-12 | 0.4.0 | chore | `accept_pre_cutoff` |
| `e8acf8cf` | 2026-07-12 | 0.1.1 | fix(ios): authenticate xcodebuild with the ASC key and sign Release as distribution | `optional_ledger_if_task_known` |

## First-epoch (296) — deferred

Not enumerated here. Folded into the first release epoch on the status page (`lg: true`).
Same disposition: accept as history; no gate impact; no ledger work planned.

## Verify gate after cutoff

```bash
bash scripts/check_task_link.sh --range 14e4ee5551d19af743739ed7ab9a0727ec89ef48..HEAD
```

