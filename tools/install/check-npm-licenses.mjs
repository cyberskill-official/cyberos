#!/usr/bin/env node
import { readFileSync, existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
const __dirname = dirname(fileURLToPath(import.meta.url));
const DEFAULT_ALLOW = join(__dirname, "npm-license-allowlist.txt");
const GPL_RE = /\b(AGPL|GPL)(-|\b)/i;
function usage() { process.stderr.write("usage: node check-npm-licenses.mjs [--allowlist <path>] <package.json> [...]\n"); process.exit(2); }
function loadAllowlist(path) {
  const set = new Set();
  for (const line of readFileSync(path, "utf8").split(/\r?\n/)) {
    const t = line.replace(/#.*$/, "").trim();
    if (t) set.add(t);
  }
  return set;
}
function depsOf(pkg) {
  const out = [];
  for (const field of ["dependencies", "devDependencies", "optionalDependencies", "peerDependencies"]) {
    const block = pkg[field];
    if (!block || typeof block !== "object") continue;
    for (const [name, ver] of Object.entries(block)) out.push({ name, version: String(ver) });
  }
  return { deps: out, licenses: pkg.dependencyLicenses || {} };
}
function licenseAllowed(lic, allow) {
  if (allow.has(lic)) return true;
  const parts = lic.split(/\s+OR\s+/i).map((s) => s.trim()).filter(Boolean);
  return parts.length > 1 && parts.every((part) => allow.has(part));
}
function main(argv) {
  let allowPath = DEFAULT_ALLOW; const pkgs = [];
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === "--allowlist") { allowPath = argv[++i]; continue; }
    if (argv[i].startsWith("-")) usage();
    pkgs.push(resolve(argv[i]));
  }
  if (!pkgs.length) usage();
  if (!existsSync(allowPath)) { process.stderr.write(`check-npm-licenses: allowlist missing: ${allowPath}\n`); process.exit(2); }
  const allow = loadAllowlist(allowPath);
  let failed = 0;
  for (const p of pkgs) {
    if (!existsSync(p)) { process.stderr.write(`check-npm-licenses: missing ${p}\n`); failed++; continue; }
    const pkg = JSON.parse(readFileSync(p, "utf8"));
    const { deps, licenses } = depsOf(pkg);
    if (!deps.length) { process.stdout.write(`check-npm-licenses: ${p}: zero deps — ok\n`); continue; }
    for (const d of deps) {
      const lic = licenses[d.name] || "UNKNOWN";
      if (GPL_RE.test(lic) || /UNKNOWN/i.test(lic) || lic === "") {
        process.stderr.write(`check-npm-licenses: FAIL ${p}: ${d.name}@${d.version} license='${lic}' (GPL-family or unknown)\n`);
        failed++; continue;
      }
      if (!licenseAllowed(lic, allow)) {
        process.stderr.write(`check-npm-licenses: FAIL ${p}: ${d.name}@${d.version} license='${lic}' not in allowlist\n`);
        failed++;
      }
    }
  }
  process.exit(failed === 0 ? 0 : 1);
}
main(process.argv.slice(2));
