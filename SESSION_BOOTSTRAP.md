# CyberOS — Session Bootstrap Prompt

Paste the prompt below into a new Claude Code (or Cowork) session opened on `/Users/stephencheng/Projects/CyberSkill/cyberos/`. It tells Claude where everything is, what's already built, what's coming next, and how to load the memory protocol cleanly after you've wiped `.cyberos-memory/`.

---

## Pre-bootstrap checklist (you, on the host shell)

```bash
# 1. Delete the duplicate sources left in place during consolidation (see CHANGELOG.md 2026-05-14)
cd /Users/stephencheng/Projects/CyberSkill
rm -rf design-system landing-page
rm -rf workbench/CyberOS-docs workbench/cyberskill-vn-skills
rm -f workbench/CYBEROS_STRATEGY.md

# 2. Wipe the old BRAIN (per your request — fresh start)
cd /Users/stephencheng/Projects/CyberSkill/cyberos
rm -rf .cyberos-memory

# 3. Tag the current commit as a stable checkpoint
git add -A
git commit -m "feat: umbrella consolidation + Liquid Glass + Pagefind"
git tag cyberos-bootstrap-$(date +%Y%m%d)

# 4. Open this folder in a new Claude Code or Cowork session
```

Now paste the prompt below.

---

## The Bootstrap Prompt

```
You are continuing development of CyberOS, an AI-native internal operations platform built by CyberSkill (Stephen Cheng, Ho Chi Minh City, Vietnam, 10-person consultancy). The repo at /Users/stephencheng/Projects/CyberSkill/cyberos/ is now the single source of truth for everything CyberOS-related.

PROJECT CONTEXT

CyberOS has THREE shipped modules and NINETEEN designed-but-not-built modules.

Shipped modules (read each module's README.md first):
- memory/   — append-only audit-chained personal memory store. 245 tests passing. 30 CLI subcommands via `python -m cyberos`. MMR + Ed25519 STH + crypto-mode. Cross-platform automation (launchd/systemd/Task Scheduler). All 12 audit proposals (P1-P12 + P2 Stage 3) shipped.
- skill/    — Anthropic Agent Skills open-standard compliant catalog + Rust host + Bun toolchain. 20 SKILL.md bundles (14 CUO + 6 cyberskill-vn). All 7 audit phases done. Phase 5 WASM execution path is feature-gated behind --features wasm + wasm32-wasi target.
- cuo/      — natural-language router. Phase 1 rule-based (15/15 routing fixtures + 15/15 pytest). Phases 2-4 (LLM router, multi-skill chains, persona switching) designed but not built.

Designed-not-yet-built modules (all in PRD + SRS):
- P0 cross-cutting: AUTH, AI Gateway, MCP Gateway, OBS
- P0 priority: CHAT
- P1: EMAIL, PROJ, TIME, CRM, KB, HR, REW, LEARN
- P2: INV, ESOP
- P3: RES, OKR
- P4: DOC, PORTAL, TEN

REPO LAYOUT

cyberos/
├── memory/                  Python — shipped
├── skill/                   Rust workspace — shipped
├── cuo/                     Python — Phase 1 shipped
├── docs/{prd,srs}/          Markdown PRD + SRS (433KB + 591KB — authoritative spec)
├── design-system/           CyberSkill design doctrine (DESIGN.md, 30k+ lines, v1.1.0 — Liquid Glass default added Part 21)
├── website/
│   ├── docs/                Multi-page interactive docs site (32 pages, 226 Mermaid diagrams, 341 FRs, 100 NFRs, 199 glossary terms, 42 risks). Pagefind site-wide search wired.
│   └── landing/             cyberskill.world landing page source
├── strategy/
│   └── CYBEROS_STRATEGY.md  Ecosystem-as-a-service playbook (3000 words, 7 sections, 5 productization levels)
├── public-skills/           Public-repo scaffold for the cyberskill-vn collection
├── runtime/                 Legacy leftovers (gradually retiring)
├── .cyberos-memory/         The BRAIN — will be RE-INITIALIZED in step 1 below
├── AGENTS.md, CLAUDE.md     Symlinks to memory/docs/AGENTS.md (memory protocol RFC)
├── README.md                Umbrella overview
├── CHANGELOG.md             Top-level dated entries
└── SESSION_BOOTSTRAP.md     This file

YOUR FIRST FIVE STEPS

1. Initialize a fresh BRAIN.
   cd memory
   pip install -e .
   mkdir -p ../.cyberos-memory/audit
   echo '{}' > ../.cyberos-memory/manifest.json
   cd ..
   python -m cyberos --store .cyberos-memory doctor
   Expected: 16 invariants reported (12 pass / 3 warn / 1 layout-no-sandbox-path error since this is a real path).
   The 'layout-no-sandbox-path' error on macOS may need CYBEROS_HOST_MOUNT_PREFIX=/Users/stephencheng/Projects/CyberSkill set if it complains about the path.

2. Skim the strategic playbook.
   Read strategy/CYBEROS_STRATEGY.md. It frames the 12-month arc and explains why CyberOS is positioned as ecosystem-as-a-service, not a horizontal SaaS. Section 5 (Concrete next-session priorities) is the work queue.

3. Skim the docs site.
   Open website/docs/index.html in a browser, or:
   cd website/docs && python3 -m http.server 8765
   The 22 module pages each carry: 5W1H2C5M analysis, architecture diagram, data model ERD, GraphQL/MCP/CLI API surface, sequence diagrams, state machines, FR catalog, NFR matrix, compliance traceability, risk entries, KPIs, RACI, CLI examples. Pagefind search is in the top-right nav.

4. Read the per-module READMEs.
   memory/README.md       — module overview + status table + place in CyberOS
   skill/README.md        — same shape
   cuo/README.md          — same shape

5. Pick the next work item.
   The strategic playbook §5 prioritizes:
   - Deploy docs site publicly (Cloudflare Pages) — see DEPLOYMENT.md
   - Build AUTH module (keystone for everything else; spec in website/docs/modules/auth.html)
   - Per-module UI mockups using design-system + Claude Design (next major UX work)
   - Comparison matrices + migration guides (demand-gen)

MEMORY PROTOCOL CONVENTIONS

The memory protocol is documented in memory/docs/AGENTS.md (symlinked from root AGENTS.md). Before any work that mutates memory state:
- verify cyberos doctor is green
- writes go through the Writer (ops.put, ops.move, ops.delete)
- every operation emits an audit row
- the chain is verified via cyberos verify

DESIGN SYSTEM CONVENTIONS

- Anchor colors are IMMUTABLE: Umber #45210E + Ochre #F4BA17
- Liquid Glass is the DEFAULT surface treatment (Part 21 of design-system/DESIGN.md)
- Vietnamese-first commitment (Be Vietnam Pro font listed before Inter)
- WCAG 2.2 AA + APCA Lc ≥ 75 for body text — never compromised
- Voice axes: warm + direct + honest + respectful — IMMUTABLE

WHAT TO DO IN THIS SESSION

Tell me what you want to focus on. Suggested first conversations:
- "Walk me through what's shipped so I can verify my mental model."
- "Start building the AUTH module per website/docs/modules/auth.html spec."
- "Deploy website/docs/ to Cloudflare Pages at docs.cyberskill.world."
- "Build module UI mockups using the design system."
- "Show me the per-page state of the docs site and help me decide which pages need deeper content."

Run cyberos doctor + git status as your first two commands to confirm the workspace is healthy.
```

---

## What this prompt does

1. **Loads context** — explains the 3 shipped + 19 designed modules, the umbrella layout, the strategic posture
2. **Reinitializes the BRAIN** — Step 1 creates a clean `.cyberos-memory/` since you wiped the old one
3. **Sets reading order** — strategy → docs site → per-module READMEs → pick next item
4. **Reinforces protocols** — memory protocol writes through Writer, design system anchors are immutable
5. **Suggests starting points** — concrete next-session work items

## Notes

- The prompt is ~600 words. Long enough to land context, short enough to fit in any chat-turn budget.
- It tells Claude to run `cyberos doctor` first — this is the canonical "is everything healthy" check.
- It points at the docs site (`website/docs/index.html`) which is now your interactive reference.
- It explicitly mentions the Liquid Glass default + Umber/Ochre anchors so any new UI work follows the system.
- The "next conversations" list is a menu — Claude will read it and either pick one or wait for you to choose.

## When to use a different prompt

- **First-time Claude Code on a fresh laptop**: same prompt, but also run `which cargo bun python3` first to confirm the toolchain.
- **Resuming mid-feature**: skip steps 2-3 and tell Claude the specific module + the last commit message.
- **Auditing rather than building**: tell Claude "Audit only — don't modify anything. Walk through the 22 module pages and tell me what looks inconsistent or incomplete."

## Source

Generated 2026-05-14 alongside the consolidation pass. See `CHANGELOG.md` for that day's entries.
