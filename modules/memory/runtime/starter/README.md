# `runtime/starter/` — Bootstrap scaffolds for new projects

Templates that get copied when starting a new project that should adopt the CyberOS memory protocol.

## Layout

| Subfolder | Purpose |
| --- | --- |
| [`cyberos-starter/`](cyberos-starter/) | Full project skeleton: empty `.cyberos-memory/` (memory), `manifest.json` with placeholders, AGENTS.md symlink recipe, smoke-test script. |
| [`templates/`](templates/) | Layer-1 starter templates (small `.md` skeletons) loaded by `cyberos init` and `cyberos add <TYPE>`. |

## When to use

**New project from scratch:**
```shell
cp -r /path/to/cyberos/runtime/starter/cyberos-starter ~/Projects/my-thing
cd ~/Projects/my-thing
# Edit .cyberos-memory/manifest.json (project.id, project.name)
ln -s /path/to/cyberos/modules/memory/AGENTS.md AGENTS.md
ln -s /path/to/cyberos/modules/memory/AGENTS.md CLAUDE.md
cyberos onboard
cyberos verify
```

**Existing project adopting memory:**
```shell
cd existing-project/
mkdir -p .cyberos-memory
cp /path/to/cyberos/runtime/starter/cyberos-starter/.cyberos-memory/manifest.json .cyberos-memory/
# Edit the manifest, then:
cyberos onboard
```

## Related

- Starter templates: [`templates/`](templates/)
- Starter rationale: [`README.md`](../../README.md)
