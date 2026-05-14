# Publishing CyberOS Skills

## Where skills live

Three distribution channels, all resolved through `cyberos-skill-resolver`:

1. **Local filesystem** — `~/.cyberos/skills/` (user-global) or `<project>/.cyberos/skills/` (project-scoped).
2. **OCI registry** — `oci://ghcr.io/cyberskill/<name>:<version>` (Phase 5+).
3. **HTTPS URL** — direct `.skill.tar.gz` download (Phase 5+).
4. **agentskills.io** — the open Agent Skills registry. CyberSkill publishes the `cyberskill-vn` collection here.

## Publish workflow (Phase 1 — local + tarball)

```bash
# 1. Validate the skill
cyberos-skill validate skill/skills/cyberskill-vn/vn-vat-invoice/SKILL.md

# 2. Run the test fixtures (your own logic, not done here yet)
python skill/skills/cyberskill-vn/vn-vat-invoice/tests/run_fixtures.py

# 3. Package
bash skill/tools/package.sh skill/skills/cyberskill-vn/vn-vat-invoice --out dist/

# Output:
#   dist/vn-vat-invoice-0.1.0.skill.tar.gz
#   dist/vn-vat-invoice-0.1.0.skill.tar.gz.sha256
```

## Publish workflow (Phase 5+ — agentskills.io)

```bash
# Sign the bundle with cosign
cosign sign-blob dist/vn-vat-invoice-0.1.0.skill.tar.gz \
    --output-signature dist/vn-vat-invoice-0.1.0.skill.tar.gz.sig

# Push to OCI registry
cyberos-skill push dist/vn-vat-invoice-0.1.0.skill.tar.gz \
    --registry ghcr.io/cyberskill

# Submit to the agentskills.io directory (HTTP form / API call)
cyberos-skill publish \
    --bundle dist/vn-vat-invoice-0.1.0.skill.tar.gz \
    --target agentskills.io
```

## Install (consumer side)

```bash
# From the published registry
cyberos-skill install agentskills.io/cyberskill/vn-vat-invoice@0.1.0

# Or from a local tarball
cyberos-skill install file://./dist/vn-vat-invoice-0.1.0.skill.tar.gz

# Or from any compatible host: drop the unpacked directory into
# ~/.cyberos/skills/   (or wherever the host scans)
```

## CyberSkill collection conventions

Skills in the `cyberskill-vn` collection carry:

- `metadata.region: VN`
- `metadata.collection: cyberskill-vn`
- `metadata.author: cyberskill`
- License: Apache-2.0 or MIT
- All references to Vietnamese government documents include the decree/circular number for traceability (e.g. "Nghị định 126/2020/NĐ-CP")
