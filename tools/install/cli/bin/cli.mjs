#!/usr/bin/env node
// npx cs <command> [args] — the single CLI entry point.
//
// `cs` is the trigger keyword; everything after it is a command. Naming a bin after one
// of its own verbs (`cs-install`) made `cs-install --help` read as nonsense, and it
// meant the bin name had to change every time the verb did — which is exactly how `init`
// survived the init->install rename. The commands below mirror the payload's own surface
// (help.sh) and the plugin's slash commands 1:1, so the three channels cannot drift apart.
// Renamed from `cyberos` to `cs` (TASK-IMP-130) — the old name collided on $PATH with the
// unrelated, PyPI-unpublished modules/memory console script of the same name.
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const payload = join(here, "..", ".."); // cli/bin -> payload root

// command -> how to run it. Shell commands take [repo]; mcp is a node passthrough; gates
// resolves inside the TARGET repo's vendored machine, not the payload.
const SCRIPTS = {
  install: "install.sh",
  uninstall: "uninstall.sh",
  version: "version.sh",
  status: "status.sh",
  help: "help.sh",
  create: "create.sh",
};

function usage(stream = process.stdout) {
  const v = existsSync(join(payload, "VERSION"))
    ? readFileSync(join(payload, "VERSION"), "utf8").trim()
    : "unknown";
  stream.write(`CyberOS ${v} — npx cs <command> [args]

  install [repo]     install / re-vendor CyberOS into a repo
  uninstall [repo]   remove the vendored machine (keeps tasks + BRAIN by default)
  version [repo]     check for a newer CyberOS; if stale, ask to run install
  status [repo]      open docs/status/index.html in your default browser
  create [dir]       scaffold a new repo with CyberOS already installed
  gates [repo]       run the machine gates for an installed repo
  mcp                launch the stdio MCP server
  help               this text

  -h, --help         this text
  -v, --version      print the payload version

[repo] defaults to the current directory. A bare \`npx cs\` prints this and
changes nothing — installing is always explicit.
Docs: https://os.cyberskill.world/docs
`);
}

const argv = process.argv.slice(2);
const cmd = argv[0];
const rest = argv.slice(1);

// A bare invocation describes itself; it does not write to whatever repo you are standing in.
if (!cmd || cmd === "help" || cmd === "-h" || cmd === "--help") {
  usage();
  process.exit(0);
}
if (cmd === "-v" || cmd === "--version") {
  const f = join(payload, "VERSION");
  process.stdout.write(existsSync(f) ? readFileSync(f, "utf8").trim() + "\n" : "unknown\n");
  process.exit(0);
}

let r;
if (cmd === "mcp") {
  r = spawnSync(process.execPath, [join(payload, "mcp", "cyberos-mcp.mjs"), ...rest], { stdio: "inherit" });
} else if (cmd === "gates") {
  // gates live in the TARGET repo's vendored machine — the payload has no gates to run.
  const repo = resolve(rest[0] || process.cwd());
  const gates = join(repo, ".cyberos", "cuo", "gates", "run-gates.sh");
  if (!existsSync(gates)) {
    process.stderr.write(`cs gates: no gates at ${gates}. Run \`npx cs install\` first.\n`);
    process.exit(2);
  }
  r = spawnSync("bash", [gates], { cwd: repo, stdio: "inherit" });
} else if (SCRIPTS[cmd]) {
  const script = join(payload, SCRIPTS[cmd]);
  // Only the repo-scoped commands get a cwd default; help/create take their own args.
  const args = rest.length || cmd === "help" || cmd === "create" ? rest : [process.cwd()];
  r = spawnSync("bash", [script, ...args], { stdio: "inherit" });
} else {
  process.stderr.write(`cs: unknown command '${cmd}'\n\n`);
  usage(process.stderr);
  process.exit(2);
}
process.exit(r.status == null ? 1 : r.status);
