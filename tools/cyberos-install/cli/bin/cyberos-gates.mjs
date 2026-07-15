#!/usr/bin/env node
// npx cyberos-gates [repo-dir] - run the CyberOS machine gates for a repo (default cwd).
import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { join, resolve } from "node:path";

const repo = resolve(process.argv[2] || process.cwd());
const gates = join(repo, ".cyberos", "cuo", "gates", "run-gates.sh");
if (!existsSync(gates)) {
  process.stderr.write(`cyberos-gates: no gates at ${gates}. Run cyberos-install first.\n`);
  process.exit(2);
}
const r = spawnSync("bash", [gates], { cwd: repo, stdio: "inherit" });
process.exit(r.status == null ? 1 : r.status);
