"""Example migration — adds a 'migrated-001' tag to every FACT memory."""
APPLIES_TO = "memories/facts/*.md"
DESCRIPTION = "Tag every FACT with `migrated-001` (sample migration for E.1)"


def transform(fm, body, rel):
    tags = fm.get("tags") or []
    if "migrated-001" not in tags:
        tags = list(tags) + ["migrated-001"]
        fm["tags"] = tags
    return fm, body
