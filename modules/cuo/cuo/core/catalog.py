"""Catalog scanner — discovers persona folders and their workflow files.

Filesystem layout (per `cuo/MODULE.md` §0.1 — flat persona layout):

    cuo/
    ├── MODULE.md                  ← canonical catalog
    ├── _template/                 ← scaffolds (skip during discovery)
    ├── docs/                      ← protocol docs (skip)
    ├── <persona-slug>/            ← one per active C-role
    │   ├── README.md              ← 9-block-schema persona spec
    │   └── workflows/
    │       └── <workflow>.md      ← skill-chain declarations

A persona is "shipped" if it has at least one workflow file at `cuo/<slug>/workflows/*.md`.
A persona is "planned" if its folder exists but `workflows/` is empty.
A persona is "extinct" if it's tagged as such in its README (e.g. chief-metaverse-officer
intentionally preserved per C-Suite Reference §8 rule 4).
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path

import yaml

# Directories under cuo/ that are NOT personas.
_NON_PERSONA_DIRS = frozenset(["_template", "docs", "cuo", "tests", "__pycache__", ".pytest_cache"])

# Pattern matching workflow files: cuo/<persona>/workflows/<slug>.md
_WORKFLOW_GLOB = "workflows/*.md"

# Pattern matching the frontmatter block at the top of a workflow file.
_FRONTMATTER_RE = re.compile(r"\A---\n(.*?)\n---\n", re.DOTALL)


@dataclass
class PersonaEntry:
    """A discovered persona folder."""

    slug: str
    persona_dir: Path
    readme_path: Path
    workflows_dir: Path
    has_workflows: bool
    is_extinct: bool = False
    # Lazy-loaded fields populated on demand from README.md frontmatter / body
    disambiguated_title: str = ""
    section: str = ""

    def __repr__(self) -> str:
        flag = "EXTINCT" if self.is_extinct else ("shipped" if self.has_workflows else "planned")
        return f"PersonaEntry({self.slug!r}, {flag})"


@dataclass
class WorkflowEntry:
    """A discovered workflow file with parsed frontmatter."""

    workflow_id: str
    workflow_version: str
    purpose: str
    persona: str
    cadence: str
    status: str
    inputs: list[dict] = field(default_factory=list)
    outputs: list[dict] = field(default_factory=list)
    skill_chain: list[dict] = field(default_factory=list)
    escalates_to: list[dict] = field(default_factory=list)
    consults: list[dict] = field(default_factory=list)
    audit_hooks: list[str] = field(default_factory=list)
    workflow_file: Path = field(default_factory=Path)
    # Phase 4 (FR-CUO-106): preserve the full raw frontmatter dict so the
    # handler dispatcher can read pattern-specific fields (pattern, sla_minutes,
    # instance_descriptor, output_recipients, gates, peer_persona, etc.)
    frontmatter: dict = field(default_factory=dict)

    @property
    def slug(self) -> str:
        """Workflow slug — last segment of workflow_id, e.g. 'architect-new-system'."""
        return self.workflow_id.rsplit("/", 1)[-1]

    @property
    def persona_slug(self) -> str:
        """Persona slug — first segment of workflow_id, e.g. 'cto'."""
        return self.workflow_id.split("/", 1)[0]

    def __repr__(self) -> str:
        return f"WorkflowEntry({self.workflow_id!r}, status={self.status!r}, chain_len={len(self.skill_chain)})"


def _is_persona_dir(path: Path) -> bool:
    """True if `path` is a candidate persona folder."""
    if not path.is_dir():
        return False
    if path.name in _NON_PERSONA_DIRS or path.name.startswith("."):
        return False
    # A persona folder MUST have a README.md per MODULE.md §0.3
    if not (path / "README.md").is_file():
        return False
    return True


def _detect_extinct(readme_path: Path) -> bool:
    """Detect if a persona is intentionally extinct (e.g. chief-metaverse-officer)."""
    if not readme_path.is_file():
        return False
    try:
        head = readme_path.read_text(encoding="utf-8")[:2000]
    except (OSError, UnicodeDecodeError):
        return False
    return "EXTINCT" in head or "extinct cautionary-tale" in head.lower()


def _parse_persona_title(readme_path: Path) -> tuple[str, str]:
    """Extract (disambiguated_title, section) from persona README.

    Looks for §1 Identity & scope block per MODULE.md §0.3 nine-block schema.
    Returns ("", "") if not found.
    """
    try:
        txt = readme_path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        return "", ""

    title = ""
    section = ""
    m = re.search(r"\*\*Full disambiguated title:\*\*\s+(.+?)(?:\.|$)", txt, re.MULTILINE)
    if m:
        title = m.group(1).strip()
    m = re.search(r"§(5\.\d+)", txt)
    if m:
        section = m.group(1)
    return title, section


def discover_personas(cuo_root: Path) -> list[PersonaEntry]:
    """Scan `cuo_root` for persona folders.

    Returns a list of `PersonaEntry` records sorted by slug. Extinct personas
    are included but flagged via `is_extinct=True`. Personas without a
    `workflows/` subdirectory still appear but with `has_workflows=False`.

    Args:
        cuo_root: path to the `cuo/` directory (must contain MODULE.md).

    Raises:
        FileNotFoundError: if `cuo_root` or `cuo_root/MODULE.md` is missing.
    """
    cuo_root = Path(cuo_root).resolve()
    if not cuo_root.is_dir():
        raise FileNotFoundError(f"cuo_root does not exist: {cuo_root}")
    if not (cuo_root / "MODULE.md").is_file():
        raise FileNotFoundError(f"cuo_root missing MODULE.md (not a CUO module): {cuo_root}")

    personas: list[PersonaEntry] = []
    for entry in sorted(cuo_root.iterdir()):
        if not _is_persona_dir(entry):
            continue
        workflows_dir = entry / "workflows"
        has_workflows = workflows_dir.is_dir() and any(workflows_dir.glob("*.md"))
        readme = entry / "README.md"
        title, section = _parse_persona_title(readme)
        personas.append(
            PersonaEntry(
                slug=entry.name,
                persona_dir=entry,
                readme_path=readme,
                workflows_dir=workflows_dir,
                has_workflows=has_workflows,
                is_extinct=_detect_extinct(readme),
                disambiguated_title=title,
                section=section,
            )
        )
    return personas


def _parse_workflow_frontmatter(workflow_file: Path) -> dict | None:
    """Parse the YAML frontmatter block at the top of a workflow file.

    Returns the parsed dict or None if no frontmatter found / parse error.
    """
    try:
        content = workflow_file.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        return None
    m = _FRONTMATTER_RE.match(content)
    if not m:
        return None
    try:
        data = yaml.safe_load(m.group(1))
    except yaml.YAMLError:
        return None
    return data if isinstance(data, dict) else None


def discover_workflows(persona: PersonaEntry) -> list[WorkflowEntry]:
    """Discover all workflows belonging to a persona.

    Returns a list of `WorkflowEntry` sorted by `workflow_id`. Files that don't
    parse as valid YAML frontmatter are skipped silently.
    """
    if not persona.workflows_dir.is_dir():
        return []

    workflows: list[WorkflowEntry] = []
    for workflow_file in sorted(persona.workflows_dir.glob("*.md")):
        fm = _parse_workflow_frontmatter(workflow_file)
        if fm is None:
            continue
        wf = WorkflowEntry(
            workflow_id=str(fm.get("workflow_id", "")),
            workflow_version=str(fm.get("workflow_version", "")),
            purpose=str(fm.get("purpose", "")),
            persona=str(fm.get("persona", "")),
            cadence=str(fm.get("cadence", "")),
            status=str(fm.get("status", "")),
            inputs=list(fm.get("inputs") or []),
            outputs=list(fm.get("outputs") or []),
            skill_chain=list(fm.get("skill_chain") or []),
            escalates_to=list(fm.get("escalates_to") or []),
            consults=list(fm.get("consults") or []),
            audit_hooks=list(fm.get("audit_hooks") or []),
            workflow_file=workflow_file,
            frontmatter=dict(fm),  # Phase 4: preserve full frontmatter for handler dispatch
        )
        workflows.append(wf)
    return workflows


def discover_all(cuo_root: Path) -> dict[str, list[WorkflowEntry]]:
    """Discover all workflows across all personas in cuo_root.

    Returns a dict mapping persona_slug → list[WorkflowEntry]. Personas with
    no workflows are omitted from the dict.
    """
    result: dict[str, list[WorkflowEntry]] = {}
    for persona in discover_personas(cuo_root):
        if not persona.has_workflows:
            continue
        wfs = discover_workflows(persona)
        if wfs:
            result[persona.slug] = wfs
    return result
