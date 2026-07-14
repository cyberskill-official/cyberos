# TASK-APP-005 — code-review packet (steps 17–18)

Status: `reviewing`. **HALTED at HITL gate 1 (review acceptance).** Diff under review: the `feat(desktop): TASK-APP-005 ...` phase commit (4 new files; `modified_files: []` honored).

## §1 clause → evidence map (all 8 clauses)

| §1 clause | Requirement | Evidence | Verdict |
|---|---|---|---|
| 1 | Two independent Linux paths layered over existing GH-Releases artifacts | snap recipe + Flathub manifest; deb pipeline untouched, reused as the snap's dump source | ✅ |
| 2 | Snap vs Flathub treated as architecturally distinct | Snap: CI job; Flathub: manifest + Stephen-gated PR, **no CI gate anywhere** — structural, not just documented | ✅ |
| 3 | `confinement: strict`, gnome/core22, webkit2gtk-4.1 generation | recipe fields + standing AC #2 lint (proven locally); runtime lib names match release.yml's dev-package generation | ✅ |
| 4 | Plugs = exactly the justified six | recipe + standing AC #3 exact-set lint (proven locally, want/got output on drift) | ✅ |
| 5 | Flatpak app-id an explicit decision, not silent reuse | PROVISIONAL banner in-manifest; hard-blocker section in the answer sheet; rename cost contained to this FR's own files | ✅ |
| 6 | `SNAP_RELEASE` gate per repo idiom; no Flathub CI flag | job `if:`; AC #6 standing grep guard (self-match found + fixed) | ✅ |
| 7 | No name registration / credential minting / Flathub PR by the agent | none performed; per-instance approval requirement restated in 3 places (manifest header, answer sheet, this packet) | ✅ |
| 8 | Two-section answer sheet | `linux-store-submission.md`: Snap metadata + Flathub review checklist + smoke-test gate | ✅ (`pending-human` fields for you) |

## Implementer-disclosed findings

1. **Skeleton staging-path bug fixed:** §3's CI skeleton extracted the deb into `snap/dist`, but snapcraft resolves `source: dist/` from the PROJECT root — the recipe would never have seen the payload. Extraction corrected to `apps/desktop/src-tauri/dist`.
2. **AC #6 guard self-match:** the in-repo guard's own grep pattern was the string it forbids; fixed with the split-pattern idiom (chosen over an exclusion flag, which would blind the guard to future additions in the same file). The spec's §5 check assumed an external runner; an in-repo standing check needs the idiom.
3. **YAML scalar bug caught by verification:** unquoted `confinement: strict` inside a plain `run:` scalar; block-scalar fix.
4. **Binary-name normalization added:** deb layout/binary name unconfirmed until a real build (spec §10 row 3) — staging accepts cyberos/cyberos-desktop/CyberOS and fails loudly with the tree listing, upgrading a smoke-test-only catch to a hard CI failure.
5. **Version-drift disclosure:** `snapcraft.yaml` carries a literal `version: '1.0.0'` not yet wired into the stamper — manual bump per release until a follow-up lands (answer sheet ops note). Not silently wired in: stamper changes are outside this FR's scope.
6. **`.desktop` entry deliberately absent** per spec §6 (depends on the app-id decision) — absence is spec-compliant, not an oversight.

## Machine gates

YAML ×3 PASS; AC #2 + AC #3 lints proven locally; AC #6 proven after fix; AC #1 `snapcraft pack` + AC #4 `flatpak-builder` **expected-pending** (tools unavailable in this container — snapd can't run here; both are standing steps/documented pre-PR requirements); coverage N/A (declared); run-gates floor GREEN.

## Reviewer verdict needed

**"TASK-APP-005 review: approved"** or **"TASK-APP-005 review: rejected — <reason>"**.

*End review packet.*
