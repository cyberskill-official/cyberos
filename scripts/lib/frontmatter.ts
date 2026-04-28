/**
 * Minimal YAML frontmatter parser/serializer for feature_request@1 markdown.
 * No external deps — keeps the generator runnable with a single tsx call.
 *
 * Frontmatter spec: opening `---` on line 1, closing `---` on its own line,
 * keys are snake_case, values are scalars (string, number, boolean) or simple lists.
 */

const FENCE = "---";

export interface ParsedDoc {
  frontmatter: Record<string, unknown>;
  body: string;
}

export function parseDoc(text: string): ParsedDoc {
  if (!text.startsWith(FENCE + "\n") && !text.startsWith(FENCE + "\r\n")) {
    throw new Error("Document missing leading frontmatter fence");
  }
  const lines = text.split(/\r?\n/);
  // First line is "---". Find next standalone "---".
  let endIdx = -1;
  for (let i = 1; i < lines.length; i++) {
    if (lines[i] === FENCE) {
      endIdx = i;
      break;
    }
  }
  if (endIdx === -1) throw new Error("Frontmatter never closes");
  const fmLines = lines.slice(1, endIdx);
  const body = lines.slice(endIdx + 1).join("\n");
  return { frontmatter: parseFrontmatterLines(fmLines), body };
}

export function parseFrontmatterLines(lines: string[]): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  for (const raw of lines) {
    const line = raw.trim();
    if (!line || line.startsWith("#")) continue;
    const colon = line.indexOf(":");
    if (colon === -1) continue;
    const key = line.slice(0, colon).trim();
    const rest = line.slice(colon + 1).trim();
    out[key] = parseScalar(rest);
  }
  return out;
}

function parseScalar(raw: string): unknown {
  if (raw === "" || raw === "null" || raw === "~") return null;
  if (raw === "true") return true;
  if (raw === "false") return false;
  // Comment-stripped quoted string
  if (
    (raw.startsWith('"') && raw.endsWith('"')) ||
    (raw.startsWith("'") && raw.endsWith("'"))
  ) {
    return raw.slice(1, -1);
  }
  // Strip trailing comment (`# ...`) when value is bare
  const hash = raw.indexOf("#");
  const body = hash >= 0 ? raw.slice(0, hash).trim() : raw;
  // Number
  if (/^-?\d+(\.\d+)?$/.test(body)) return Number(body);
  // Bare string
  return body;
}

/** Serialise a frontmatter block in a fixed key order. */
export function serializeFrontmatter(
  fm: Record<string, unknown>,
  keyOrder: readonly string[],
): string {
  const lines: string[] = [];
  const seen = new Set<string>();
  for (const key of keyOrder) {
    if (key in fm) {
      lines.push(formatLine(key, fm[key]));
      seen.add(key);
    }
  }
  // Append any extra keys deterministically
  for (const key of Object.keys(fm).sort()) {
    if (!seen.has(key)) lines.push(formatLine(key, fm[key]));
  }
  return [FENCE, ...lines, FENCE, ""].join("\n");
}

function formatLine(key: string, value: unknown): string {
  if (value === null || value === undefined) return `${key}: ""`;
  if (typeof value === "boolean") return `${key}: ${value ? "true" : "false"}`;
  if (typeof value === "number") return `${key}: ${value}`;
  if (Array.isArray(value)) {
    if (value.length === 0) return `${key}: []`;
    const parts = value.map((v) => quoteString(String(v))).join(", ");
    return `${key}: [${parts}]`;
  }
  // String — always quote; preserves whitespace and disambiguates
  return `${key}: ${quoteString(String(value))}`;
}

function quoteString(s: string): string {
  return `"${s.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
}
