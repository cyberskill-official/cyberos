# Install

Generic install for any Agent-Skills-compatible host.

## Step 1 — Get the bundle

```bash
git clone <this-repository-url> cyberskill-vn-skills
```

Or download and extract a release tarball.

## Step 2 — Locate your AI client's skills directory

Consult your AI client's documentation for the path it watches for skill bundles. Each skill is a directory containing a `SKILL.md` file at its root.

## Step 3 — Copy or symlink each skill folder

```bash
for d in cyberskill-vn-skills/vn-* cyberskill-vn-skills/vneid-*; do
  cp -r "$d" <your-skills-directory>/
done
```

## Step 4 — Restart your AI client

So it picks up the new bundles.

## Step 5 — Verify

List loaded skills via your client's skill-management command, or trigger one with a relevant prompt (for example: "Is `0312345678-001` a valid Vietnamese MST?").
