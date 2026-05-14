"""Catalog discovery tests."""

from __future__ import annotations

from pathlib import Path

from cuo.core.catalog import SkillEntry, discover

REPO = Path(__file__).resolve().parents[2]
SKILLS_ROOT = REPO / "skill" / "skills"


def test_discover_returns_skill_entries():
    entries = discover(SKILLS_ROOT)
    assert len(entries) > 0
    assert all(isinstance(e, SkillEntry) for e in entries)


def test_discover_includes_cyberskill_vn():
    entries = discover(SKILLS_ROOT)
    names = {e.name for e in entries}
    expected = {
        "vn-mst-validate",
        "vn-vat-invoice",
        "vn-bank-transfer",
        "vneid-integration",
        "vn-tax-filing",
        "vn-legal-compliance",
    }
    assert expected.issubset(names), f"missing: {expected - names}"


def test_discover_parses_metadata():
    entries = discover(SKILLS_ROOT)
    by_name = {e.name: e for e in entries}
    mst = by_name.get("vn-mst-validate")
    assert mst is not None
    assert mst.region == "VN"
    assert mst.collection == "cyberskill-vn"
    assert "validate" in mst.description.lower() or "vietnamese" in mst.description.lower()


def test_discover_returns_empty_on_missing_root(tmp_path):
    assert discover(tmp_path / "no-such-dir") == []


def test_discover_skips_files_without_frontmatter(tmp_path):
    # Create a fake skill root with one valid and one broken SKILL.md.
    good = tmp_path / "good"
    good.mkdir()
    (good / "SKILL.md").write_text(
        "---\nname: fake-skill\ndescription: a thing\n---\n# body\n",
        encoding="utf-8",
    )
    bad = tmp_path / "bad"
    bad.mkdir()
    (bad / "SKILL.md").write_text("# no frontmatter\n", encoding="utf-8")

    entries = discover(tmp_path)
    names = {e.name for e in entries}
    assert names == {"fake-skill"}
