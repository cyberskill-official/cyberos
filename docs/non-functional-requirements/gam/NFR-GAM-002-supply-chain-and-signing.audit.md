---
nfr_id: NFR-GAM-002
audited: 2026-06-24
verdict: PASS (as-built)
score: 10/10
fidelity: as-built
template: as-built-verification@1
---

## §1 — Verdict summary

NFR-GAM-002 is satisfied across both halves: dependency gating (cargo-deny + locked resolution, green) and update signing (minisign, pinned key, rotation completed 2026-06-23). No private signing material is in the tree.

## §2 — Statement to artefact traceability

| §1 Clause | Artefact | Verification | Status |
|---|---|---|---|
| #1 cargo-deny over 4 dimensions | `deny.toml` + gate step | `cargo deny check` green | OK |
| #2 locked resolution | `--locked` cargo, frozen pnpm | gate builds the reviewed graph | OK |
| #3 signed + pinned-key updates | `tauri.conf.json` pubkey | FR-GAM-008 | OK |
| #4 rotatable + re-pinned | rotation 2026-06-23 | new key built into app in green CI | OK |
| #5 no committed signing material | repo scan | no `.env`/`*.key`/`*.pem`/minisign secret under apps/gam | OK |

## §3 — Verification record

```bash
cd apps/gam/src-tauri && cargo deny check     # advisories/licenses/bans/sources
# secret scan:
find apps/gam -type f \( -name '.env' -o -name '*.key' -o -name '*.pem' \)   # empty
grep -rI 'minisign encrypted secret key' apps/gam                           # empty
```

Six transitive advisories (webpki TLS, tar) were cleared by a broad `cargo update` during hardening; gate green afterward.

## §4 — Status

`accepted → shipped`. The rotation is the live proof of the rotatability clause.

*End of NFR-GAM-002 audit.*
