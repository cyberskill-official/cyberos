# `runtime/completions/` — Shell tab-completion (Aspect 1.4)

Static completion scripts for the `cyberos` umbrella binary. Loaded by your shell of choice; gives `<TAB>` completion for all 63+ subcommands, their flags, and (where applicable) live BRAIN values like memory IDs.

## Files

| File | Shell |
| --- | --- |
| `cyberos.bash` | bash |
| `cyberos.zsh` | zsh |
| `cyberos.fish` | fish |

## Installation

**bash:**
```shell
echo 'source /abs/path/to/cyberos/runtime/completions/cyberos.bash' >> ~/.bashrc
```

**zsh:**
```shell
echo 'fpath=(/abs/path/to/cyberos/runtime/completions $fpath)' >> ~/.zshrc
echo 'autoload -Uz compinit && compinit' >> ~/.zshrc
```

**fish:**
```shell
ln -s /abs/path/to/cyberos/runtime/completions/cyberos.fish ~/.config/fish/completions/
```

## What's completed

- All subcommand names (`cyberos <TAB>` → `add`, `audit`, `chain`, `doctor`, …).
- Per-subcommand flags (`cyberos chain run --<TAB>` → `--pitch`, `--profile`, `--prd`, `--srs`, `--with-llm`, …).
- Dynamic values where cheap to enumerate: persona slugs, memory types, FR IDs from `cyberos fr list`.

## Regenerating

The completion scripts are hand-maintained today. The `cyberos completions emit` subcommand re-emits them from a single source-of-truth declaration in `runtime/tools/cyberos`:

```shell
cyberos completions emit bash > runtime/completions/cyberos.bash
cyberos completions emit zsh  > runtime/completions/cyberos.zsh
cyberos completions emit fish > runtime/completions/cyberos.fish
```

## Related

- Aspect 1.4 in the operator manual: [`../../memory/docs/README.md`](../../memory/docs/README.md) Part 26.1.4
