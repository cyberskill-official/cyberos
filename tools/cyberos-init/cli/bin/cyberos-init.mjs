#!/usr/bin/env node
// npx cyberos-init [target-dir] - thin wrapper over the payload's install.sh.
// Vendors the CyberOS machine + wires every popular agent into the target repo (default cwd).
import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const init = join(here, "..", "..", "install.sh"); // cli/bin -> payload root
const args = process.argv.slice(2);
const r = spawnSync("bash", [init, ...(args.length ? args : [process.cwd()])], { stdio: "inherit" });
process.exit(r.status == null ? 1 : r.status);
