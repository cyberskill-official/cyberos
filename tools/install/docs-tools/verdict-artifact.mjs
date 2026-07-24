#!/usr/bin/env node
// verdict-artifact.mjs — TASK-IMP-143: content-addressed HITL verdict artifacts.
// Node stdlib only. Mint or validate attributed verdict JSON that binds actor +
// transition + evidence bytes so --verdict-by is not an honor-system string alone.
//
// Schema (verdict@1):
//   {
//     "schema": "cyberos.verdict@1",
//     "actor": "<string>",
//     "timestamp": "<ISO-8601>",
//     "task_id": "TASK-...",
//     "from": "<status>",
//     "to": "<status>",
//     "evidence_path": "<path as given>",
//     "evidence_sha256": "<hex>",
//     "artifact_sha256": "<hex of canonical payload without artifact_sha256>"
//   }
//
// Usage (library): import { mintVerdictArtifact, validateVerdictArtifact, sha256File } from ...
// CLI:  node verdict-artifact.mjs mint|validate ...
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync, mkdirSync, existsSync, statSync } from "node:fs";
import { dirname, join, resolve, basename } from "node:path";
import { fileURLToPath } from "node:url";

export const VERDICT_SCHEMA = "cyberos.verdict@1";

export function sha256Bytes(buf) {
  return createHash("sha256").update(buf).digest("hex");
}

export function sha256File(path) {
  return sha256Bytes(readFileSync(path));
}

export function canonicalPayload(obj) {
  // Stable key order for hashing (exclude artifact_sha256).
  const keys = ["schema", "actor", "timestamp", "task_id", "from", "to", "evidence_path", "evidence_sha256"];
  const out = {};
  for (const k of keys) out[k] = obj[k];
  return `${JSON.stringify(out)}\n`;
}

export function artifactDigest(obj) {
  return sha256Bytes(Buffer.from(canonicalPayload(obj), "utf8"));
}

export function nowIso() {
  const pinned = process.env.CYBEROS_NOW;
  if (pinned && String(pinned).trim()) return String(pinned).trim();
  return new Date().toISOString().replace(/\.\d{3}Z$/, "Z");
}

export function verdictsDir(root) {
  return join(root, "docs", "tasks", "_verdicts");
}

export function mintVerdictArtifact(root, { actor, task_id, from, to, evidence_path, evidence_abs, timestamp }) {
  const evidence_sha256 = sha256File(evidence_abs);
  const base = {
    schema: VERDICT_SCHEMA,
    actor,
    timestamp: timestamp || nowIso(),
    task_id,
    from,
    to,
    evidence_path,
    evidence_sha256,
  };
  const artifact_sha256 = artifactDigest(base);
  const full = { ...base, artifact_sha256 };
  const dir = verdictsDir(root);
  mkdirSync(dir, { recursive: true });
  const short = evidence_sha256.slice(0, 12);
  const name = `${task_id}--${from}--${to}--${short}.json`;
  const abs = join(dir, name);
  writeFileSync(abs, `${JSON.stringify(full, null, 2)}\n`);
  return { path: abs, relative: join("docs", "tasks", "_verdicts", name), artifact: full };
}

export function validateVerdictArtifact(artifactPath, { actor, task_id, from, to, evidence_abs, evidence_path }) {
  let raw;
  try {
    raw = JSON.parse(readFileSync(artifactPath, "utf8"));
  } catch (e) {
    return { ok: false, error: `verdict artifact unparseable: ${e.message}` };
  }
  if (raw.schema !== VERDICT_SCHEMA) return { ok: false, error: `verdict schema must be ${VERDICT_SCHEMA}` };
  if (raw.actor !== actor) return { ok: false, error: `verdict actor '${raw.actor}' !== --verdict-by '${actor}'` };
  if (raw.task_id !== task_id) return { ok: false, error: `verdict task_id mismatch` };
  if (raw.from !== from || raw.to !== to) return { ok: false, error: `verdict transition mismatch (${raw.from}->${raw.to})` };
  const wantEv = sha256File(evidence_abs);
  if (raw.evidence_sha256 !== wantEv) {
    return { ok: false, error: `verdict evidence_sha256 does not match evidence file bytes` };
  }
  if (evidence_path !== undefined && raw.evidence_path !== evidence_path) {
    return { ok: false, error: `verdict evidence_path mismatch` };
  }
  const expect = artifactDigest(raw);
  if (raw.artifact_sha256 !== expect) {
    return { ok: false, error: `verdict artifact_sha256 self-check failed` };
  }
  return { ok: true, artifact: raw };
}

// ── CLI ──────────────────────────────────────────────────────────────────────
function usage(code = 2) {
  process.stderr.write(`usage:
  node verdict-artifact.mjs mint --root <repo> --actor <a> --task <id> --from <s> --to <s> --evidence <path>
  node verdict-artifact.mjs validate --artifact <path> --actor <a> --task <id> --from <s> --to <s> --evidence <path>
`);
  process.exit(code);
}

function parseArgs(argv) {
  const out = { _: [] };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a.startsWith("--")) {
      const k = a.slice(2);
      const v = argv[i + 1];
      if (v === undefined || v.startsWith("--")) out[k] = true;
      else { out[k] = v; i++; }
    } else out._.push(a);
  }
  return out;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const opts = parseArgs(process.argv.slice(2));
  const cmd = opts._[0];
  if (cmd === "mint") {
    const root = resolve(opts.root || ".");
    const ev = resolve(root, opts.evidence);
    if (!existsSync(ev) || !statSync(ev).isFile() || statSync(ev).size === 0) {
      process.stderr.write("verdict-artifact: evidence must be an existing non-empty file\n");
      process.exit(8);
    }
    const minted = mintVerdictArtifact(root, {
      actor: opts.actor,
      task_id: opts.task,
      from: opts.from,
      to: opts.to,
      evidence_path: opts.evidence,
      evidence_abs: ev,
    });
    process.stdout.write(`${JSON.stringify({ ok: true, ...minted })}\n`);
  } else if (cmd === "validate") {
    const root = resolve(opts.root || ".");
    const ev = resolve(root, opts.evidence);
    const r = validateVerdictArtifact(resolve(opts.artifact), {
      actor: opts.actor,
      task_id: opts.task,
      from: opts.from,
      to: opts.to,
      evidence_abs: ev,
      evidence_path: opts.evidence,
    });
    if (!r.ok) {
      process.stderr.write(`verdict-artifact: ${r.error}\n`);
      process.exit(8);
    }
    process.stdout.write(`${JSON.stringify({ ok: true, artifact: r.artifact })}\n`);
  } else {
    usage(2);
  }
}
