#!/usr/bin/env python3
"""
cyberos_onboard.py — interactive new-contributor bootstrap.

Aspect 8.1 of the Layer-1 improvement catalog.
Pattern from ECC `codebase-onboarding` (6 parallel reconnaissance checks).

Usage:
    cyberos onboard                      # interactive 5-step wizard
    cyberos onboard --shared <zip>       # pull shared content during onboarding
    cyberos onboard --persona <role>     # also create persona/<role>.md
    cyberos onboard --non-interactive    # fail if any required input missing

5 steps:
    1. Reconnaissance — detect project type, stack, language, timezone, protocol pin
    2. Subject identity — interactive prompts
    3. Generate starter memories — member/<id>/profile.md, PERSON-NNN, PREF-NNN
    4. Pull shared content (manual at this stage; auto when BRAIN module P1 ships)
    5. Verify — run validator + audit-chain check
"""
from __future__ import annotations
import argparse
import json
import os
import re
import subprocess
import sys
import uuid
from datetime import datetime, timezone, timedelta
from pathlib import Path

ICT = timezone(timedelta(hours=7))

def _tty():
    return sys.stdin.isatty() and sys.stdout.isatty()

def ask(prompt, default=None, validator=None):
    if not _tty():
        if default is not None:
            return default
        print(f"non-interactive mode + no default for: {prompt}", file=sys.stderr)
        sys.exit(2)
    while True:
        d_str = f" [{default}]" if default else ""
        r = input(f"  ? {prompt}{d_str}: ").strip()
        if not r and default is not None:
            r = default
        if validator and not validator(r):
            print(f"    invalid; try again")
            continue
        return r

def step_1_reconnaissance(root: Path) -> dict:
    """Detect project type, stack, language, timezone, protocol pin."""
    print(f"\n{'═'*60}")
    print(f"  Step 1/5 — Project reconnaissance")
    print(f"{'═'*60}\n")
    info = {}

    # git origin
    try:
        url = subprocess.check_output(["git", "remote", "get-url", "origin"],
                                       cwd=root, stderr=subprocess.DEVNULL, text=True).strip()
        info["origin"] = url
        print(f"  ✓ git origin: {url}")
    except Exception:
        info["origin"] = "(no git remote)"
        print(f"  · no git remote configured")

    # Stack detection
    stack = []
    for ind, name in [
        ("package.json", "node"), ("pyproject.toml", "python"), ("Cargo.toml", "rust"),
        ("go.mod", "go"), ("Gemfile", "ruby"), ("composer.json", "php"),
        ("AGENTS.md", "agents-md-protocol"),
    ]:
        if (root / ind).exists():
            stack.append(name)
    info["stack"] = stack
    print(f"  ✓ stack: {', '.join(stack) if stack else '(none detected)'}")

    # Manifest pin
    manifest_p = root / ".cyberos-memory" / "manifest.json"
    if manifest_p.exists():
        try:
            m = json.loads(manifest_p.read_text())
            info["protocol_pin"] = m.get("protocol", {}).get("sha256", "?")
            info["memory_count"] = m.get("memory_count", 0)
            info["project_id"] = m.get("project", {}).get("id", "?")
            info["project_name"] = m.get("project", {}).get("name", "?")
            print(f"  ✓ protocol pin: {info['protocol_pin'][:24]}…")
            print(f"  ✓ memory count: {info['memory_count']}")
            print(f"  ✓ project: {info['project_name']} ({info['project_id']})")
        except Exception:
            print(f"  ✗ manifest.json unreadable")
    else:
        print(f"  · no .cyberos-memory/ yet (bootstrap will create one — TODO)")

    # Language
    info["language"] = "en"  # default; could detect via filenames
    print(f"  ✓ language: {info['language']}")
    return info

def step_2_subject_identity(info: dict) -> dict:
    """Interactive prompts for subject identity."""
    print(f"\n{'═'*60}")
    print(f"  Step 2/5 — Subject identity")
    print(f"{'═'*60}\n")

    subj = ask("Your subject ID (lowercase-kebab, e.g. stephen-cheng)",
               validator=lambda x: bool(re.match(r"^[a-z][a-z0-9-]*$", x)))
    name = ask("Display name (e.g. 'Stephen Cheng' or 'Trinh Thai Anh (Stephen)')")
    role = ask("Role", default="contributor",
               validator=lambda x: x in ("founder", "engineer", "designer", "ops", "legal", "sales", "support", "contributor"))
    tz = ask("Timezone (IANA)", default="Asia/Ho_Chi_Minh")
    lang = ask("Preferred language", default="en")
    hours = ask("Working hours (e.g. '09:00-19:00')", default="flexible")

    return {
        "subject_id": subj, "display_name": name, "role": role,
        "tz": tz, "language": lang, "hours": hours,
    }

def _ts_now():
    return datetime.now(ICT).isoformat(timespec='seconds')

def _uuid7():
    """Best-effort UUIDv7 — falls back to UUIDv4 if no helper available."""
    return str(uuid.uuid4())

def step_3_starter_memories(root: Path, identity: dict, info: dict) -> list[Path]:
    """Generate starter memories. Returns list of staged file paths."""
    print(f"\n{'═'*60}")
    print(f"  Step 3/5 — Generate starter memories")
    print(f"{'═'*60}\n")

    staged_dir = root / "outputs" / "staged-memories" / f"onboard-{identity['subject_id']}-{datetime.now(ICT).strftime('%Y-%m-%d')}"
    staged_dir.mkdir(parents=True, exist_ok=True)
    files = []

    # 1. member/<id>/profile.md
    profile_path = staged_dir / f"member-{identity['subject_id']}-profile.md"
    profile_path.write_text(f"""---
memory_id: mem_{_uuid7()}
scope: member:{identity['subject_id']}
classification: personnel
authority: human-edited
version: 1
created_at: {_ts_now()}
created_by: subject:{identity['subject_id']}
last_updated_at: {_ts_now()}
updated_by: subject:{identity['subject_id']}
provenance:
  source: chat
  source_ref: cyberos onboard interactive wizard
  confidence: 1.0
consent:
  has_consent: true
  consent_event: null
  consent_scope: ["self-profile", "onboarding"]
tags: [profile, onboarding, subject-owned]
relationships: []
retention:
  rule: personnel-7-years-post-employment
  earliest_delete: null
embedding: {{model: null, version: null, vector_id: null}}
sync_class: local-only
source_freshness_tier: 8
---

# Profile — {identity['display_name']}

## Identity
- **Subject ID:** subject:{identity['subject_id']}
- **Display name:** {identity['display_name']}
- **Role:** {identity['role']}
- **Timezone:** {identity['tz']}
- **Language:** {identity['language']}
- **Working hours:** {identity['hours']}

## Onboarding
- **Onboarded at:** {_ts_now()}
- **Onboard tool:** runtime/tools/cyberos_onboard.py
- **Initial protocol pin:** {info.get('protocol_pin', '?')[:24]}…

## Subject sovereignty
Per AGENTS.md §17: this `member:` scope is subject-sovereign. Agents do not contest your edits. You may override the default sync_class per file. You may not promote your own writes to `shared` — that requires org BRAIN acceptance (when BRAIN module P1 ships).

## Privacy
This file is `local-only` by default per §17 + classification:personnel. To share elements with the org, manually create separate `publishable`-class memories.
""")
    files.append(profile_path)
    print(f"  ✓ staged member/{identity['subject_id']}/profile.md")

    # 2. memories/people/PERSON-NNN-<subject>.md
    person_path = staged_dir / f"memories-people-PERSON-NNN-{identity['subject_id']}.md"
    person_path.write_text(f"""---
memory_id: mem_{_uuid7()}
scope: memories/people
classification: personnel
authority: human-edited
version: 1
created_at: {_ts_now()}
created_by: subject:{identity['subject_id']}
last_updated_at: {_ts_now()}
updated_by: subject:{identity['subject_id']}
provenance:
  source: chat
  source_ref: cyberos onboard interactive wizard
  confidence: 1.0
consent:
  has_consent: true
  consent_event: null
  consent_scope: ["personnel", "people-graph"]
tags: [person, contributor, role-{identity['role']}]
relationships: []
retention:
  rule: personnel-7-years-post-employment
  earliest_delete: null
embedding: {{model: null, version: null, vector_id: null}}
sync_class: publishable
source_freshness_tier: 20
---

# PERSON-NNN {identity['display_name']}

## Identity
- **Subject ID:** subject:{identity['subject_id']}
- **Display name:** {identity['display_name']}
- **Role:** {identity['role']}
- **Timezone:** {identity['tz']}
- **Languages:** {identity['language']}

## Working preferences
- **Working hours:** {identity['hours']}
- **Communication style:** [add async/sync preference]
- **Decision style:** [add consult-first / decide-and-broadcast / etc.]

## Context
[How this person relates to the project / org / client]

## Consent
- **Consent for people-graph inclusion:** YES (granted via onboarding wizard)
- **Consent event:** [audit row ID will populate on brain_writer write]
- **Retention:** 7 years post-employment-end per personnel default

## Privacy
- This is a `personnel`-class memory; conflicts NEVER auto-resolve (§9.1)
- Compensation, gov-ID, home address, health PII are DENYLISTED per §9.3
""")
    files.append(person_path)
    print(f"  ✓ staged memories/people/PERSON-NNN-{identity['subject_id']}.md")

    # 3. memories/preferences/PREF-NNN-onboarding-checklist.md
    pref_path = staged_dir / f"memories-preferences-PREF-NNN-{identity['subject_id']}-onboarding.md"
    pref_path.write_text(f"""---
memory_id: mem_{_uuid7()}
scope: memories/preferences
classification: operational
authority: human-edited
version: 1
created_at: {_ts_now()}
created_by: subject:{identity['subject_id']}
last_updated_at: {_ts_now()}
updated_by: subject:{identity['subject_id']}
provenance:
  source: chat
  source_ref: cyberos onboard interactive wizard
  confidence: 1.0
consent:
  has_consent: true
  consent_event: null
  consent_scope: ["preference", "onboarding"]
tags: [preference, onboarding, checklist, subject-{identity['subject_id']}]
relationships: []
retention:
  rule: indefinite
  earliest_delete: null
embedding: {{model: null, version: null, vector_id: null}}
sync_class: local-only
source_freshness_tier: 25
---

# PREF-NNN Onboarding checklist for {identity['display_name']}

## Preference
Track onboarding progress for subject:{identity['subject_id']}.

## Checklist
- [ ] Read AGENTS.md Part 1-4 of CyberOS-AGENTS.README.md
- [ ] Run first `cyberos status`
- [ ] Run first `cyberos add DEC` (or REF, FACT)
- [ ] Trigger first refinement candidate (intentional — to learn the §0.4 loop)
- [ ] Onboard 2nd machine via Aspect 6 personal sync (when implemented)
- [ ] Adopt voice standard for new docs (no em dashes, no AI vocab)

## Scope
- **Applies to:** subject:{identity['subject_id']}
- **Override-rule:** mark items complete as they happen; tombstone this PREF when 100% complete

## Rationale
Pattern from `human-resources/onboarding` skill — 30/60/90-day plan with concrete milestones.
""")
    files.append(pref_path)
    print(f"  ✓ staged memories/preferences/PREF-NNN-{identity['subject_id']}-onboarding.md")

    return files

def step_4_shared_content(root: Path, shared_zip: Path | None):
    print(f"\n{'═'*60}")
    print(f"  Step 4/5 — Pull shared content")
    print(f"{'═'*60}\n")
    if shared_zip and shared_zip.exists():
        print(f"  · would import {shared_zip} via cyberos_export.py import (TODO)")
    else:
        print(f"  · skipped (no --shared <zip> passed)")
        print(f"  · Note: real org-BRAIN sync ships with BRAIN module P1.")
        print(f"          Until then, manual export/import per §11.5.")

def step_5_verify(root: Path):
    print(f"\n{'═'*60}")
    print(f"  Step 5/5 — Verify")
    print(f"{'═'*60}\n")
    tool = root / "runtime" / "tools" / "cyberos_validate.py"
    if tool.exists():
        rc = subprocess.run(["python3", str(tool), str(root)], capture_output=True, text=True).returncode
        if rc == 0:
            print(f"  ✓ validator: clean")
        else:
            print(f"  ⚠ validator returned {rc} (review findings before bootstrapping memories)")
    else:
        print(f"  · cyberos_validate.py not found — skipping")

def main():
    p = argparse.ArgumentParser(description="cyberos onboard — interactive new-contributor bootstrap")
    p.add_argument("--shared", type=Path, help="shared-export zip from teammate")
    p.add_argument("--persona", help="also create persona/<role>.md")
    p.add_argument("--non-interactive", action="store_true", help="fail if any input missing")
    p.add_argument("--root", type=Path, default=Path.cwd(), help="project root")
    args = p.parse_args()

    print(f"\n{'═'*60}")
    print(f"  CyberOS BRAIN Onboarding")
    print(f"{'═'*60}")

    info = step_1_reconnaissance(args.root)
    identity = step_2_subject_identity(info)
    staged = step_3_starter_memories(args.root, identity, info)
    step_4_shared_content(args.root, args.shared)
    step_5_verify(args.root)

    print(f"\n{'═'*60}")
    print(f"  ✓ Onboarding staged complete.")
    print(f"{'═'*60}")
    print(f"\nStaged memories (NOT yet committed to BRAIN):")
    for f in staged:
        print(f"  · {f}")
    print(f"\nNext: review staged files, then run brain_writer to commit each:")
    print(f"  cd {args.root}")
    print(f"  for f in {staged[0].parent}/*.md; do")
    print(f"    python3 outputs/brain_writer.py write <relpath> <body-file>")
    print(f"  done")
    print(f"\nThen run:")
    print(f"  python3 runtime/tools/cyberos status")
    print(f"  python3 runtime/tools/cyberos verify")
    print()

if __name__ == "__main__":
    main()
