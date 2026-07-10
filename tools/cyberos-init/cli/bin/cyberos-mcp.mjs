#!/usr/bin/env node
// npx cyberos-mcp - launch the CyberOS stdio MCP server (passthrough to mcp/cyberos-mcp.mjs).
import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const server = join(here, "..", "..", "mcp", "cyberos-mcp.mjs"); // cli/bin -> payload root -> mcp
const r = spawnSync(process.execPath, [server, ...process.argv.slice(2)], { stdio: "inherit" });
process.exit(r.status == null ? 1 : r.status);
