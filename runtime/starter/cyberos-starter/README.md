# cyberos-starter

Skeleton for bootstrapping a new project with a fresh `.cyberos-memory/`
BRAIN. Drop-in template; replace placeholders, run `cyberos onboard`.

## Layout

```
cyberos-starter/
├── README.md                      ← this file
├── .cyberos-memory/
│   ├── manifest.json              ← per-project pointer; fill in your project_id + name
│   ├── audit/
│   │   └── (empty — populated on first session)
│   ├── meta/
│   │   ├── retention-rules.md     ← default retention policy
│   │   └── validators/
│   │       └── README.md          ← drop check-*.py plugins here
│   ├── memories/
│   │   ├── decisions/             ← (empty)
│   │   ├── refinements/           ← (empty)
│   │   ├── facts/                 ← (empty)
│   │   ├── people/                ← (empty)
│   │   ├── projects/              ← (empty)
│   │   ├── preferences/           ← (empty)
│   │   └── drift/                 ← (empty)
│   └── persona/                   ← (empty; example via `cyberos onboard --persona founder`)
└── tours/
    └── onboarding.tour            ← VS Code CodeTour walkthrough
```

## Bootstrap

```bash
# 1. Clone the starter into a new project
cp -r runtime/starter/cyberos-starter ~/Projects/my-new-thing

cd ~/Projects/my-new-thing

# 2. Edit manifest.json — set project.id, project.name
$EDITOR .cyberos-memory/manifest.json

# 3. Symlink CLAUDE.md + AGENTS.md → docs/memory/AGENTS.md (single source of truth)
ln -s /path/to/cyberos/docs/memory/AGENTS.md AGENTS.md
ln -s /path/to/cyberos/docs/memory/AGENTS.md CLAUDE.md

# 4. Run onboard wizard
/path/to/cyberos/runtime/tools/cyberos onboard

# 5. Verify
/path/to/cyberos/runtime/tools/cyberos verify
```

## What `cyberos onboard` does

1. Prompts for your subject id (`subject:<slug>`)
2. Optionally creates `.cyberos-memory/persona/<role>.md`
3. Optionally seeds `memories/people/PERSON-001-<subject>.md`
4. Drops a starter checklist memory at
   `memories/preferences/PREF-onboarding-checklist-<subject>.md`
5. Runs `cyberos verify` and reports

## Convention reminders

- Memory IDs must be UUIDv7 (`mem_<uuid7>`) — `cyberos add` handles this
- Frontmatter is required on every memory — see `meta/templates/`
- Sync-class defaults: `publishable` for `decisions/refinements`,
  `local-only` for `people`
- `cyberos verify` is your safety net — run it after every batch of writes

## Tour

Open `tours/onboarding.tour` in VS Code with the CodeTour extension to
walk through what each starter file does.
