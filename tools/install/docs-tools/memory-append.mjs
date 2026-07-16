#!/usr/bin/env node
// memory-append.mjs — doc-driven appender for the Layer-1 BRAIN audit chain (TASK-IMP-093).
//
// Every ship-tasks phase declares memory rows (workflow_phase_complete, workflow_complete,
// task_routed_back, artefact_write), but a doc-driven run has no MCP writer — the real
// consumer run parked its payloads in a tracked _audits file, chain-shaped data with no
// chain (IMPROVEMENT_HANDOFF.md IMP-05). This tool is the minimal protocol-honoring
// writer for EXACTLY those four kinds, plus a verify mode, so a governed run can keep
// the chain truthful from any environment that has node.
//
// Usage:  node memory-append.mjs [--json] [--actor <name>] [--now <ISO-8601>] <command> ...
//
//   append <store-root> <kind> <payload.json|->
//       Validate kind + payload (refusals happen BEFORE any write), acquire the §4.2
//       lease lock, bootstrap a fresh store (HEAD=0, canonical dirs) when needed, clean
//       stale two-phase tmp files, walk + re-verify the whole chain, then append ONE
//       framed record to audit/current.binlog and advance HEAD — both via §4.1 two-phase
//       writes (tmp + fsync + rename + parent-dir fsync). Payload '-' reads stdin.
//   verify <store-root>
//       Recompute every frame (crc32c, seq continuity, prev_chain linkage, §6.3 chain
//       hash) across all audit/*.binlog segments and compare the tip to HEAD. Exits
//       non-zero NAMING THE FIRST BAD ORDINAL on any mismatch. Verify only reports —
//       it never rewrites (§6.5: recovery belongs to the canonical writer).
//
// ON-DISK MAPPING (matches the canonical store, not a private format) ---------------
// The layout is the one install.sh scaffolds at .cyberos/memory/store/ and the one the
// canonical python writer (modules/memory/cyberos/core/writer.py) appends to:
//   <store-root>/HEAD                  8-byte LE u64 — last committed seq (struct '<Q')
//   <store-root>/.lock                 §4.2 lease record (JSON) — empty when unheld
//   <store-root>/audit/current.binlog  active segment of §6.2 frames (see below)
//   <store-root>/audit/*.binlog        sealed segments, walked in name order first
// Frame (writer.py _FRAME_HDR, struct '>IIQQ' = 24 bytes, then payload):
//   [u32 length BE][u32 crc32c BE][u64 seq BE][u64 ts_ns BE][payload]
// Payload is canonical JSON of the AuditRecord — sorted keys, compact separators,
// UTF-8, no NaN/Infinity (msgspec order='sorted'; RFC 8785-conforming for this closed
// schema). Record shape (memory.schema.json#/definitions/AuditRecord):
//   { actor, chain, content_sha256, extra, op, path, prev_chain, ts_ns }
// The four workflow kinds map onto that shape the same way session rows do
// (cyberos/core/session.py: op="session.start", extra={...}):
//   op            = <kind>                      (the closed four-kind set)
//   path          = meta/workflow/<task>.json   (from payload.task_id|task, validated
//                   against the MemoryPath segment charset; meta/workflow/run.json
//                   when the payload names no task)
//   actor         = --actor | $CYBEROS_ACTOR | "doc-driven"
//   content_sha256= ""                          (no file body — same as session rows)
//   extra         = the payload object, verbatim
// Chain (§6.3, exactly writer.py._chain_hash): the record is serialized WITH its chain
// field set to "" (msgspec omit_defaults=False keeps the key), then
//   chain = SHA-256( canonical(record_minus_chain) || bytes.fromhex(prev_chain) )
// — prev_chain concatenated as RAW 32 bytes, not hex text. Genesis prev_chain is 64
// zeros. Verify recomputes from the RAW stored bytes (the top-level '"chain":"<hex>"'
// span is blanked in place), so no float/bigint reserialization can perturb the check.
//
// CAVEATS DOCUMENTED PER SPEC --------------------------------------------------------
// * Darwin durability (§4.1): the protocol requires fcntl(F_BARRIERFSYNC) per batch and
//   F_FULLFSYNC for checkpoints on macOS — "plain fsync() is insufficient on Darwin".
//   Node's stdlib exposes fsync(2) only (no fcntl passthrough), so this tool fsyncs the
//   tmp file and the parent directory and DOCUMENTS the gap: rename atomicity gives
//   crash consistency (a torn append can never be observed), but power-loss durability
//   on macOS is weaker than the canonical writer's barrier mode. Doc-driven runs accept
//   this; the MCP writer remains the arbiter where the stronger barrier matters.
// * Lock (§4.2): node stdlib has no flock(2). The lease RECORD is enforced instead:
//   a non-empty .lock whose expiry_ns is in the future (vs the shared monotonic clock)
//   fails fast; an expired lease is reaped loudly (python StoreLock force-breaks the
//   same way); unparseable .lock bytes fail fast — never a guess. The check-then-write
//   window is a documented tolerance for single-operator doc-driven runs.
// * Append implementation: the segment is rewritten via tmp+rename with the ONE new
//   frame appended after the old bytes — a §4.1 two-phase write of the row. Prior bytes
//   are never modified (append-only, §6.5); O(segment) per append is fine at doc-driven
//   scale. Every append re-verifies the full chain first and refuses to extend a store
//   that does not verify. The one benign divergence — rows exactly one ahead of HEAD
//   (crash between segment rename and HEAD publish) — is healed by re-publishing HEAD,
//   mirroring writer.py._recover_tail; verify REPORTS it (verify never rewrites).
// * Cross-implementation canonical JSON: sorted-key compact JSON here matches msgspec
//   order='sorted' for the ASCII workflow payloads this tool writes. Exotic payloads
//   (non-ASCII keys, floats needing shortest-repr, huge ints beyond 2^53) may serialize
//   differently across implementations; verify is immune (raw-byte recompute), and the
//   canonical writer remains authoritative for anything beyond the four kinds.
//
// Exit codes:
//   0  ok / chain verifies clean
//   2  usage error: unknown kind (refused BEFORE any write), non-JSON or non-object
//      payload, unreadable payload/store, unsafe task token, bad --now
//   3  lock held (or .lock unreadable/unparseable) — fail fast, nothing written
//   4  integrity failure: verify names the first bad ordinal; append refuses a chain
//      that does not verify, a HEAD that disagrees with the rows, or a compacted
//      (.binlog.zst) store this tool cannot walk
//
// Clock: ts_ns comes from --now <ISO-8601> or $CYBEROS_NOW (the injectable clock that
// keeps test runs deterministic — same convention as ship-manifest.mjs), else the wall
// clock. Identical store + args + clock + actor = byte-identical frames, HEAD, and
// --json output. Node stdlib only (docs-tools convention).

import {
  readFileSync, writeFileSync, renameSync, existsSync, mkdirSync, readdirSync,
  openSync, fsyncSync, closeSync, unlinkSync,
} from "node:fs";
import { createHash, randomBytes } from "node:crypto";
import { join, resolve, basename } from "node:path";
import { hostname } from "node:os";

const KINDS = ["workflow_phase_complete", "workflow_complete", "task_routed_back", "artefact_write"];
const GENESIS = "0".repeat(64);              // writer.py _GENESIS_CHAIN — the null root
const FRAME_HDR = 24;                        // struct '>IIQQ'
const LEASE_TTL_NS = 10n * 1000000000n;      // §4.2: TTL 10 s
const SAFE_TOKEN = /^[A-Za-z0-9_][A-Za-z0-9_.-]*$/;  // MemoryPath segment charset
// install.sh's canonical v2 top-level scaffold (layout-root-canonical invariant).
const STORE_DIRS = ["memories", "meta", "company", "module", "member", "client",
  "project", "persona", "conflicts", "exports", "index", "audit"];

class UsageError extends Error {}
class Refusal extends Error { constructor(code, msg) { super(msg); this.code = code; } }

// ── CRC-32C (Castagnoli, reflected 0x82F63B78) ───────────────────────────────
// The frame checksum the canonical writer uses (writer.py: crc32c wheel; its zlib
// fallback is explicitly "NOT CRC-32C ... fine for development"). This table-based
// implementation IS CRC-32C, so frames interoperate with the production walker.
const CRC_TABLE = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? (0x82f63b78 ^ (c >>> 1)) >>> 0 : c >>> 1;
    t[n] = c >>> 0;
  }
  return t;
})();
function crc32c(buf) {
  let c = 0xffffffff;
  for (let i = 0; i < buf.length; i++) c = (CRC_TABLE[(c ^ buf[i]) & 0xff] ^ (c >>> 8)) >>> 0;
  return (c ^ 0xffffffff) >>> 0;
}

// ── canonical JSON (msgspec order='sorted' equivalent for this closed schema) ─
function canonicalJSON(v) {
  if (v === null) return "null";
  const t = typeof v;
  if (t === "string") return JSON.stringify(v);
  if (t === "boolean") return v ? "true" : "false";
  if (t === "bigint") return v.toString();
  if (t === "number") {
    if (!Number.isFinite(v)) throw new UsageError("payload numbers must be finite (canonical JSON has no NaN/Infinity)");
    return JSON.stringify(v);
  }
  if (Array.isArray(v)) return "[" + v.map(canonicalJSON).join(",") + "]";
  if (t === "object") {
    return "{" + Object.keys(v).sort().map((k) => JSON.stringify(k) + ":" + canonicalJSON(v[k])).join(",") + "}";
  }
  throw new UsageError(`payload carries an unserializable ${t} value`);
}

// ── deterministic --json envelope (shared docs-tools idiom) ──────────────────
function stableStringify(v, indent = 0) {
  const pad = "  ".repeat(indent), pad2 = "  ".repeat(indent + 1);
  if (v === null || typeof v !== "object") return JSON.stringify(v);
  if (Array.isArray(v)) {
    if (v.length === 0) return "[]";
    return "[\n" + v.map((x) => pad2 + stableStringify(x, indent + 1)).join(",\n") + "\n" + pad + "]";
  }
  const keys = Object.keys(v).sort();
  if (keys.length === 0) return "{}";
  return "{\n" + keys.map((k) => pad2 + JSON.stringify(k) + ": " + stableStringify(v[k], indent + 1)).join(",\n") + "\n" + pad + "}";
}

// ── §4.1 two-phase write: tmp + fsync + rename + parent-dir fsync ────────────
// (Darwin F_BARRIERFSYNC caveat: see the header — node exposes fsync only.)
function fsyncDirBestEffort(dir) {
  try {
    const fd = openSync(dir, "r");
    try { fsyncSync(fd); } finally { closeSync(fd); }
  } catch { /* directory fsync unsupported on this platform — rename atomicity still holds */ }
}
function atomicWriteBytes(path, bytes) {
  const dir = resolve(path, "..");
  const tmp = `${path}.tmp.${randomBytes(6).toString("hex")}`;
  writeFileSync(tmp, bytes);
  const fd = openSync(tmp, "r");
  try { fsyncSync(fd); } finally { closeSync(fd); }
  renameSync(tmp, path);
  fsyncDirBestEffort(dir);
}

// ── injectable clock (ship-manifest.mjs convention) ──────────────────────────
function nowNs(opts) {
  const v = opts.now || process.env.CYBEROS_NOW;
  if (v) {
    const ms = Date.parse(v);
    if (Number.isNaN(ms)) throw new UsageError(`--now/CYBEROS_NOW is not ISO-8601: '${v}'`);
    return BigInt(ms) * 1000000n;
  }
  return BigInt(Date.now()) * 1000000n;
}

// ── §4.2 lease lock (no flock in node stdlib — see header caveat) ────────────
function acquireLease(store) {
  const lockPath = join(store, ".lock");
  let cur = null;
  try { cur = readFileSync(lockPath, "utf8"); } catch { cur = null; }
  if (cur !== null && cur.trim() !== "") {
    let lease;
    try { lease = JSON.parse(cur); }
    catch {
      throw new Refusal(3, `.lock holds unparseable bytes - unknown lock state, failing fast (clear ${lockPath} by hand only if no writer is alive)`);
    }
    const nowN = Number(process.hrtime.bigint());
    const expN = Number(lease.expiry_ns);
    if (Number.isFinite(expN) && expN > nowN) {
      throw new Refusal(3, `store is locked (lease pid=${lease.pid ?? "?"} host=${lease.host ?? "?"}, ~${((expN - nowN) / 1e9).toFixed(1)}s left on the §4.2 TTL) - failing fast, nothing written`);
    }
    process.stderr.write(`memory-append: note: reaping stale lease (pid=${lease.pid ?? "?"} host=${lease.host ?? "?"}) - §4.2 expiry passed\n`);
  }
  const now = process.hrtime.bigint();
  const lease = {
    pid: process.pid, host: hostname(),
    monotonic_ns: Number(now), expiry_ns: Number(now + LEASE_TTL_NS),
    version: 1, writer: "memory-append.mjs",
  };
  writeFileSync(lockPath, JSON.stringify(lease));
  return () => { try { writeFileSync(lockPath, ""); } catch { /* release is best-effort; TTL heals */ } };
}

// ── store plumbing ────────────────────────────────────────────────────────────
function readHead(store) {
  const p = join(store, "HEAD");
  if (!existsSync(p)) return null;
  const b = readFileSync(p);
  if (b.length !== 8) throw new Refusal(4, `HEAD is ${b.length} byte(s) - the protocol requires exactly 8 (LE u64)`);
  return b.readBigUInt64LE(0);
}

function listSegments(store) {
  const auditDir = join(store, "audit");
  if (!existsSync(auditDir)) return [];
  const names = readdirSync(auditDir);
  const zst = names.filter((n) => n.endsWith(".binlog.zst")).sort();
  if (zst.length > 0) {
    throw new Refusal(4, `compacted segment ${zst[0]} present - this doc-driven tool cannot walk zstd segments; use the canonical cyberos walker`);
  }
  const segs = names.filter((n) => n.endsWith(".binlog")).sort();
  const i = segs.indexOf("current.binlog");
  if (i >= 0) { segs.splice(i, 1); segs.push("current.binlog"); }  // active segment walks last
  return segs.map((n) => join(auditDir, n));
}

// Walk every frame across every segment, recomputing crc32c, seq continuity,
// prev_chain linkage, and the §6.3 chain hash. Throws Refusal(4) naming the
// FIRST bad ordinal. Readers open exact paths only — tmp litter is invisible here.
function walkChain(store) {
  let prevChain = GENESIS;
  let expected = 1n;
  for (const seg of listSegments(store)) {
    const buf = readFileSync(seg);
    let off = 0;
    while (off < buf.length) {
      const ordinal = expected;
      const at = `${basename(seg)}+${off}`;
      if (off + FRAME_HDR > buf.length) {
        throw new Refusal(4, `first bad ordinal ${ordinal}: truncated frame header at ${at} (${buf.length - off} byte(s) left; §6.5 forbids tail rewrites - recovery belongs to the canonical writer)`);
      }
      const len = buf.readUInt32BE(off);
      const crc = buf.readUInt32BE(off + 4);
      const seq = buf.readBigUInt64BE(off + 8);
      const start = off + FRAME_HDR;
      if (start + len > buf.length) {
        throw new Refusal(4, `first bad ordinal ${ordinal}: truncated frame payload at ${at} (frame claims ${len} bytes, ${buf.length - start} left)`);
      }
      const payload = buf.subarray(start, start + len);
      if (crc32c(payload) !== crc) {
        throw new Refusal(4, `first bad ordinal ${ordinal}: crc32c mismatch at ${at} (row bytes tampered or torn)`);
      }
      if (seq !== expected) {
        throw new Refusal(4, `first bad ordinal ${ordinal}: frame header carries seq ${seq}, expected ${expected} (rows re-ordered or dropped)`);
      }
      let rec;
      try { rec = JSON.parse(payload.toString("utf8")); }
      catch (e) { throw new Refusal(4, `first bad ordinal ${ordinal}: payload is not JSON (${e.message})`); }
      if (typeof rec.chain !== "string" || !/^[0-9a-f]{64}$/.test(rec.chain)) {
        throw new Refusal(4, `first bad ordinal ${ordinal}: record carries no 64-hex chain field`);
      }
      if (rec.prev_chain !== prevChain) {
        throw new Refusal(4, `first bad ordinal ${ordinal}: prev_chain does not match the previous record's chain (link broken)`);
      }
      // Recompute on the RAW bytes: blank the top-level '"chain":"<hex>"' span in place
      // (keys are sorted, so the first occurrence is the record's own field — any
      // payload copy inside extra serializes later and escaped).
      const needle = Buffer.from(`"chain":"${rec.chain}"`, "utf8");
      const idx = payload.indexOf(needle);
      if (idx < 0) {
        throw new Refusal(4, `first bad ordinal ${ordinal}: canonical form drifted (chain field not found in raw payload bytes)`);
      }
      const minus = Buffer.concat([payload.subarray(0, idx), Buffer.from('"chain":""', "utf8"), payload.subarray(idx + needle.length)]);
      const want = createHash("sha256").update(minus).update(Buffer.from(prevChain, "hex")).digest("hex");
      if (want !== rec.chain) {
        throw new Refusal(4, `first bad ordinal ${ordinal}: chain hash mismatch (§6.3 recompute disagrees - row tampered or mis-linked)`);
      }
      prevChain = rec.chain;
      expected += 1n;
      off = start + len;
    }
  }
  return { rows: expected - 1n, lastSeq: expected - 1n, lastChain: prevChain };
}

// Stale two-phase leftovers from an interrupted append: our '<final>.tmp.<nonce>'
// pattern plus the python writer's 'HEAD.tmp'. A tmp file is NEVER state (readers
// and the walker open exact final paths), but the next locked append cleans them
// so litter cannot accumulate. Loud, per docs-tools convention.
function cleanStaleTmp(store) {
  const cleaned = [];
  const sweep = (dir, re) => {
    if (!existsSync(dir)) return;
    for (const n of readdirSync(dir)) {
      if (re.test(n)) { unlinkSync(join(dir, n)); cleaned.push(join(basename(dir) === "audit" ? "audit" : ".", n)); }
    }
  };
  sweep(join(store, "audit"), /^current\.binlog\.tmp\./);
  sweep(store, /^HEAD\.tmp/);
  if (cleaned.length > 0) {
    process.stderr.write(`memory-append: note: cleaned ${cleaned.length} stale tmp file(s) from an interrupted append: ${cleaned.join(", ")}\n`);
  }
  return cleaned;
}

// ── append ────────────────────────────────────────────────────────────────────
function cmdAppend(storeArg, kind, payloadArg, opts) {
  // Refusals BEFORE any write (spec §1 #1.2, edge "store byte-untouched"):
  if (!storeArg || !kind || !payloadArg) throw new UsageError("append requires <store-root> <kind> <payload.json|->");
  if (!KINDS.includes(kind)) {
    throw new UsageError(`kind '${kind}' refused - this appender writes exactly [${KINDS.join(", ")}]; nothing was written (the MCP writer owns every other kind)`);
  }
  let raw;
  try { raw = payloadArg === "-" ? readFileSync(0, "utf8") : readFileSync(resolve(payloadArg), "utf8"); }
  catch (e) { throw new UsageError(`payload unreadable: ${payloadArg} (${e.message}) - nothing written`); }
  let payload;
  try { payload = JSON.parse(raw); }
  catch (e) { throw new UsageError(`payload is not JSON (${e.message}) - refused before any write`); }
  if (payload === null || typeof payload !== "object" || Array.isArray(payload)) {
    throw new UsageError("payload must be a JSON object (the record's extra field is an object in the closed schema) - refused before any write");
  }
  const actor = opts.actor || process.env.CYBEROS_ACTOR || "doc-driven";
  const taskTok = payload.task_id ?? payload.task;
  let memPath = "meta/workflow/run.json";
  if (taskTok !== undefined) {
    if (typeof taskTok !== "string" || !SAFE_TOKEN.test(taskTok)) {
      throw new UsageError(`payload task/task_id ${JSON.stringify(taskTok)} is not a safe MemoryPath token (${SAFE_TOKEN}) - refused before any write`);
    }
    memPath = `meta/workflow/${taskTok}.json`;
  }
  const tsNs = nowNs(opts);
  const store = resolve(storeArg);

  // Fresh-store bootstrap part 1: the root must exist before the lease can land.
  mkdirSync(store, { recursive: true });
  const release = acquireLease(store);
  try {
    // Bootstrap (spec §1 #1.3): no HEAD -> HEAD=0 + canonical dirs, deterministic.
    let head = readHead(store);
    if (head === null) {
      if (listSegments(store).some((s) => readFileSync(s).length > 0)) {
        throw new Refusal(4, "store has audit rows but no HEAD - inconsistent; refusing to bootstrap over data (run the canonical walker)");
      }
      for (const d of STORE_DIRS) mkdirSync(join(store, d), { recursive: true });
      atomicWriteBytes(join(store, "HEAD"), Buffer.alloc(8)); // 8-byte LE u64 = 0
      head = 0n;
      process.stderr.write(`memory-append: note: bootstrapped fresh store at ${store} (HEAD=0, null-root prev_chain)\n`);
    }

    cleanStaleTmp(store);

    // Never extend a chain that does not verify.
    const { lastSeq, lastChain } = walkChain(store);
    if (lastSeq !== head) {
      if (lastSeq === head + 1n) {
        // The one benign crash window: segment renamed, HEAD publish lost.
        // writer.py._recover_tail brings HEAD forward; mirror it, loudly.
        atomicWriteBytes(join(store, "HEAD"), leU64(lastSeq));
        head = lastSeq;
        process.stderr.write(`memory-append: note: HEAD was one behind the intact rows (interrupted publish) - re-published HEAD=${lastSeq}\n`);
      } else {
        throw new Refusal(4, `HEAD says ${head} but the rows end at seq ${lastSeq} - inconsistent store, refusing to append (run verify)`);
      }
    }

    const seq = lastSeq + 1n;
    const recMinus = {
      actor, chain: "", content_sha256: "", extra: payload,
      op: kind, path: memPath, prev_chain: lastChain, ts_ns: tsNs,
    };
    const minusBytes = Buffer.from(canonicalJSON(recMinus), "utf8");
    const chain = createHash("sha256").update(minusBytes).update(Buffer.from(lastChain, "hex")).digest("hex");
    const payloadBytes = Buffer.from(canonicalJSON({ ...recMinus, chain }), "utf8");
    const hdr = Buffer.alloc(FRAME_HDR);
    hdr.writeUInt32BE(payloadBytes.length, 0);
    hdr.writeUInt32BE(crc32c(payloadBytes), 4);
    hdr.writeBigUInt64BE(seq, 8);
    hdr.writeBigUInt64BE(tsNs, 16);

    // §4.1 two-phase: segment durable FIRST, then HEAD publish (writer.py order).
    const segPath = join(store, "audit", "current.binlog");
    const old = existsSync(segPath) ? readFileSync(segPath) : Buffer.alloc(0);
    atomicWriteBytes(segPath, Buffer.concat([old, hdr, payloadBytes]));
    atomicWriteBytes(join(store, "HEAD"), leU64(seq));

    return {
      code: 0, seq: Number(seq), chain, prev_chain: lastChain, kind, path: memPath,
      actor, store,
      message: `append: seq ${seq} (${kind}) chained at ${chain.slice(0, 12)}... path ${memPath} HEAD=${seq}`,
    };
  } finally {
    release();
  }
}

function leU64(v) { const b = Buffer.alloc(8); b.writeBigUInt64LE(v, 0); return b; }

// ── verify ────────────────────────────────────────────────────────────────────
function cmdVerify(storeArg) {
  if (!storeArg) throw new UsageError("verify requires <store-root>");
  const store = resolve(storeArg);
  if (!existsSync(store)) throw new UsageError(`no store at ${store}`);
  const head = readHead(store);
  if (head === null) throw new UsageError(`no HEAD at ${join(store, "HEAD")} - not a bootstrapped store`);
  const { rows, lastSeq, lastChain } = walkChain(store);
  if (lastSeq !== head) {
    const detail = lastSeq < head
      ? `first missing ordinal ${lastSeq + 1n} (HEAD claims ${head})`
      : `rows run to seq ${lastSeq} but HEAD stopped at ${head} (interrupted publish - the next locked append re-publishes)`;
    throw new Refusal(4, `tip/HEAD mismatch: ${detail}`);
  }
  return {
    code: 0, rows: Number(rows), head: Number(head), tip_chain: rows > 0n ? lastChain : null,
    message: `verify: OK - ${rows} row(s), HEAD=${head}${rows > 0n ? `, tip chain ${lastChain.slice(0, 12)}...` : " (empty store)"}`,
  };
}

// ── CLI shell ────────────────────────────────────────────────────────────────
const HELP = `memory-append.mjs - doc-driven appender for the Layer-1 BRAIN audit chain (TASK-IMP-093)

usage: node memory-append.mjs [--json] [--actor <name>] [--now <ISO-8601>] <command> ...

commands
  append <store-root> <kind> <payload.json|->
      append ONE chained record to <store-root>/audit/current.binlog and advance HEAD.
      kind is closed: workflow_phase_complete | workflow_complete | task_routed_back |
      artefact_write - anything else is refused BEFORE any write. payload must be a JSON
      object ('-' reads stdin); payload.task_id|task names the record's meta/workflow/
      path. A fresh store bootstraps deterministically (HEAD=0, null-root prev_chain,
      canonical dirs). Stale two-phase tmp files are cleaned; a held §4.2 lease fails
      fast. The whole chain is re-verified before every append.
  verify <store-root>
      recompute every link (crc32c, seq continuity, prev_chain, §6.3 chain hash) across
      all audit/*.binlog segments and compare the tip to HEAD; exits non-zero naming the
      FIRST bad ordinal. Verify only reports - it never rewrites (§6.5).

exit codes
  0  ok / chain verifies clean
  2  usage error: unknown kind (refused before any write), non-JSON or non-object
     payload, unreadable input, unsafe task token
  3  lock held or .lock unparseable - fail fast, nothing written
  4  integrity failure: the first bad ordinal is named; append refuses inconsistent or
     compacted stores

format  frames are writer.py-identical: [u32 len BE][u32 crc32c BE][u64 seq BE]
        [u64 ts_ns BE] + canonical-JSON AuditRecord (sorted keys); HEAD is an 8-byte
        LE u64; chain = SHA-256(canonical(record_minus_chain) || raw(prev_chain)).
clock   --now / CYBEROS_NOW pins ts_ns for deterministic runs; unset = wall clock.
darwin  node exposes fsync only - no F_BARRIERFSYNC/F_FULLFSYNC (documented §4.1 gap;
        rename atomicity still guarantees crash consistency).
writes  two-phase (.tmp.<nonce> + fsync + rename + parent-dir fsync); readers and the
        walker open exact final paths, so tmp litter is never state.
--json  prints a stable-stringified result envelope (sorted keys) instead of prose.
`;

function main(argv) {
  const flags = new Set(["json", "help"]);
  const valued = new Set(["actor", "now"]);
  const opts = {};
  const positionals = [];
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "-h" || a === "--help") { opts.help = true; continue; }
    if (a.startsWith("--")) {
      const name = a.slice(2);
      if (flags.has(name)) { opts[name] = true; continue; }
      if (valued.has(name)) {
        if (i + 1 >= argv.length) { process.stderr.write(`memory-append: --${name} needs a value\n`); return 2; }
        opts[name] = argv[++i]; continue;
      }
      process.stderr.write(`memory-append: unknown flag '${a}'\n${HELP}`);
      return 2;
    }
    positionals.push(a);
  }
  if (opts.help) { process.stdout.write(HELP); return 0; }
  const [command, ...rest] = positionals;
  const emit = (r) => {
    if (opts.json) {
      const env = { command, ok: r.code === 0, exit_code: r.code, ...r };
      delete env.code;
      process.stdout.write(stableStringify(env) + "\n");
    } else if (r.message) {
      process.stdout.write(r.message + "\n");
    }
    return r.code;
  };
  try {
    if (command === "append") return emit(cmdAppend(rest[0], rest[1], rest[2], opts));
    if (command === "verify") return emit(cmdVerify(rest[0]));
    throw new UsageError(command ? `unknown command '${command}'` : "no command given");
  } catch (e) {
    if (e instanceof Refusal || e instanceof UsageError) {
      const code = e instanceof Refusal ? e.code : 2;
      if (opts.json) {
        process.stdout.write(stableStringify({ command: command ?? null, ok: false, exit_code: code, error: e.message }) + "\n");
      } else {
        process.stderr.write(`memory-append: ${e.message}\n`);
        if (code === 2) process.stderr.write("usage: node memory-append.mjs [--json] [--actor <name>] [--now <ISO>] <append|verify> ... (--help for details)\n");
      }
      return code;
    }
    throw e;
  }
}

process.exitCode = main(process.argv.slice(2));
