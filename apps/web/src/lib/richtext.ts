// Chat-grade rich text: a small hand-written parser from a message body to a block/inline tree. The tree is
// rendered as React elements by RichText.tsx, so the pipeline is XSS-safe by construction - raw HTML is never
// parsed or injected, and every piece of user text stays a text node. Grammar (Slack-scale, intentionally not
// full CommonMark): fenced code blocks, blockquotes, unordered/ordered lists, paragraphs; inline code, bold,
// italic, strikethrough, [label](url) links, bare-URL autolink, and @-mention highlighting against the
// directory. This module is pure TS (no React import) so scripts/richtext-smoke.ts can run it under node.

export interface MentionCandidate {
  /// A name this person can be @-mentioned as (display name, handle, or email local-part).
  name: string;
  /// Whether the name refers to the viewing user (renders with the stronger "mentions me" tint).
  me: boolean;
}

export type Inline =
  | { t: "text"; text: string }
  | { t: "code"; text: string }
  | { t: "bold"; children: Inline[] }
  | { t: "italic"; children: Inline[] }
  | { t: "strike"; children: Inline[] }
  | { t: "link"; href: string; label: string }
  | { t: "mention"; text: string; me: boolean };

export type Block =
  | { t: "p"; lines: Inline[][] }
  | { t: "codeblock"; lang: string; text: string }
  | { t: "quote"; lines: Inline[][] }
  | { t: "ul"; items: Inline[][] }
  | { t: "ol"; items: Inline[][]; start: number };

const FENCE_OPEN = /^```([A-Za-z0-9+#._-]{0,24})\s*$/;
const FENCE_CLOSE = /^```\s*$/;
const QUOTE = /^>\s?(.*)$/;
const UL_ITEM = /^\s{0,3}[-*]\s+(.*)$/;
const OL_ITEM = /^\s{0,3}(\d{1,4})[.)]\s+(.*)$/;
// [label](https://url) - label has no ] or newline; url has no whitespace or ).
const LINK = /^\[([^\]\n]{1,300})\]\((https?:\/\/[^\s)]{1,1500})\)/;
const URL_AT = /^(https?:\/\/[^\s<>"'`]{1,1500}|www\.[^\s<>"'`]{2,1500})/;

const isWordChar = (ch: string | undefined): boolean => (ch === undefined ? false : /[\p{L}\p{N}_]/u.test(ch));

// A boundary suitable to start a token like *bold*, @mention, or a bare URL: start-of-line or a preceding
// character that is not a letter/digit/underscore.
const boundaryBefore = (s: string, i: number): boolean => i === 0 || !isWordChar(s[i - 1]);

/// Normalized candidates, longest name first so "Anna Vu" wins over "Anna". Precomputed once per render.
export function prepareMentions(candidates: MentionCandidate[]): MentionCandidate[] {
  return candidates
    .filter((c) => c.name && c.name.trim().length > 0)
    .map((c) => ({ name: c.name.trim(), me: c.me }))
    .sort((a, b) => b.name.length - a.name.length);
}

// Trailing characters that are almost always sentence punctuation, not part of a pasted URL.
function trimUrlTail(url: string): string {
  let u = url;
  for (;;) {
    const last = u[u.length - 1];
    if (last && ".,!?;:'\"".includes(last)) {
      u = u.slice(0, -1);
      continue;
    }
    // A trailing ")" is kept only when the URL itself contains a "(" (e.g. wiki links).
    if (last === ")" && !u.includes("(")) {
      u = u.slice(0, -1);
      continue;
    }
    return u;
  }
}

function matchMention(
  s: string,
  at: number,
  mentions: MentionCandidate[],
): { text: string; me: boolean; len: number } | null {
  // `at` points at "@". Names may contain spaces, so compare against each candidate wholesale.
  const rest = s.slice(at + 1);
  const restLower = rest.toLowerCase();
  for (const c of mentions) {
    if (restLower.startsWith(c.name.toLowerCase())) {
      const after = rest[c.name.length];
      if (!isWordChar(after)) {
        return { text: rest.slice(0, c.name.length), me: c.me, len: c.name.length + 1 };
      }
    }
  }
  return null;
}

// Find a closing delimiter (e.g. "**") from `from`, requiring the char just before it to be non-space (so
// "** not bold **" stays literal). Returns -1 when absent.
function findCloser(s: string, delim: string, from: number): number {
  let i = from;
  for (;;) {
    const j = s.indexOf(delim, i);
    if (j === -1) return -1;
    if (j > from - 1 && s[j - 1] !== " " && s[j - 1] !== "\t") return j;
    i = j + 1;
  }
}

const MAX_DEPTH = 4;

/// Inline parse of one line of text (no newlines inside).
export function parseInline(s: string, mentions: MentionCandidate[] = [], depth = 0): Inline[] {
  const out: Inline[] = [];
  let text = "";
  const flush = () => {
    if (text) {
      out.push({ t: "text", text });
      text = "";
    }
  };
  let i = 0;
  while (i < s.length) {
    const ch = s[i];

    // Inline code: `code` (no nesting inside; backticks win over everything else).
    if (ch === "`") {
      const close = s.indexOf("`", i + 1);
      if (close > i + 1) {
        flush();
        out.push({ t: "code", text: s.slice(i + 1, close) });
        i = close + 1;
        continue;
      }
    }

    // Bold / strikethrough (two-char delimiters), italic (single char) - all require tight content edges.
    if (depth < MAX_DEPTH && (s.startsWith("**", i) || s.startsWith("~~", i))) {
      const delim = s.slice(i, i + 2);
      if (s[i + 2] && s[i + 2] !== " " && s[i + 2] !== "\t") {
        const close = findCloser(s, delim, i + 2);
        if (close !== -1 && close > i + 2) {
          flush();
          const kids = parseInline(s.slice(i + 2, close), mentions, depth + 1);
          out.push({ t: delim === "**" ? "bold" : "strike", children: kids });
          i = close + 2;
          continue;
        }
      }
    }
    if (depth < MAX_DEPTH && (ch === "*" || ch === "_") && boundaryBefore(s, i)) {
      const inner = s[i + 1];
      if (inner && inner !== " " && inner !== "\t" && inner !== ch) {
        let close = findCloser(s, ch, i + 2);
        // For "_", also require a word boundary after the closer (protects snake_case mid-word).
        while (close !== -1 && ch === "_" && isWordChar(s[close + 1])) close = findCloser(s, ch, close + 1);
        if (close !== -1 && close > i + 1) {
          flush();
          out.push({ t: "italic", children: parseInline(s.slice(i + 1, close), mentions, depth + 1) });
          i = close + 1;
          continue;
        }
      }
    }

    // [label](url) links, then bare URLs.
    if (ch === "[") {
      const m = LINK.exec(s.slice(i));
      if (m) {
        flush();
        out.push({ t: "link", href: m[2], label: m[1] });
        i += m[0].length;
        continue;
      }
    }
    if ((ch === "h" || ch === "w") && boundaryBefore(s, i)) {
      const m = URL_AT.exec(s.slice(i));
      if (m) {
        const url = trimUrlTail(m[1]);
        if (url.length >= 8) {
          flush();
          const href = url.startsWith("www.") ? `https://${url}` : url;
          out.push({ t: "link", href, label: url });
          i += url.length;
          continue;
        }
      }
    }

    // @-mentions against the directory (longest name wins; names may contain spaces).
    if (ch === "@" && boundaryBefore(s, i) && mentions.length > 0) {
      const m = matchMention(s, i, mentions);
      if (m) {
        flush();
        out.push({ t: "mention", text: m.text, me: m.me });
        i += m.len;
        continue;
      }
    }

    text += ch;
    i += 1;
  }
  flush();
  return out;
}

/// Full parse: body text to a list of blocks. Never throws; unknown syntax degrades to plain text.
export function parseRich(body: string, mentions: MentionCandidate[] = []): Block[] {
  const lines = (body || "").replace(/\r\n?/g, "\n").split("\n");
  const blocks: Block[] = [];
  let i = 0;
  const inline = (s: string) => parseInline(s, mentions);

  while (i < lines.length) {
    const line = lines[i];

    const fence = FENCE_OPEN.exec(line);
    if (fence) {
      const lang = fence[1] || "";
      const buf: string[] = [];
      i += 1;
      while (i < lines.length && !FENCE_CLOSE.test(lines[i])) {
        buf.push(lines[i]);
        i += 1;
      }
      i += 1; // skip the closing fence (or run past EOF on an unclosed fence)
      blocks.push({ t: "codeblock", lang, text: buf.join("\n") });
      continue;
    }

    if (QUOTE.test(line)) {
      const qlines: Inline[][] = [];
      while (i < lines.length) {
        const q = QUOTE.exec(lines[i]);
        if (!q) break;
        qlines.push(inline(q[1]));
        i += 1;
      }
      blocks.push({ t: "quote", lines: qlines });
      continue;
    }

    if (UL_ITEM.test(line)) {
      const items: Inline[][] = [];
      while (i < lines.length) {
        const m = UL_ITEM.exec(lines[i]);
        if (!m) break;
        items.push(inline(m[1]));
        i += 1;
      }
      blocks.push({ t: "ul", items });
      continue;
    }

    const olFirst = OL_ITEM.exec(line);
    if (olFirst) {
      const items: Inline[][] = [];
      const start = parseInt(olFirst[1], 10) || 1;
      while (i < lines.length) {
        const m = OL_ITEM.exec(lines[i]);
        if (!m) break;
        items.push(inline(m[2]));
        i += 1;
      }
      blocks.push({ t: "ol", items, start });
      continue;
    }

    if (line.trim() === "") {
      i += 1; // paragraph separator; consecutive blanks collapse
      continue;
    }

    // Paragraph: consecutive plain lines, kept as separate rendered lines (the client composes with
    // Shift+Enter newlines and expects them preserved).
    const plines: Inline[][] = [];
    while (
      i < lines.length &&
      lines[i].trim() !== "" &&
      !FENCE_OPEN.test(lines[i]) &&
      !QUOTE.test(lines[i]) &&
      !UL_ITEM.test(lines[i]) &&
      !OL_ITEM.test(lines[i])
    ) {
      plines.push(inline(lines[i]));
      i += 1;
    }
    blocks.push({ t: "p", lines: plines });
  }
  return blocks;
}

/// True when the body would render with any formatting at all (used to skip the tree for plain messages -
/// the overwhelmingly common case - and render the string directly).
export function isPlainText(body: string): boolean {
  return !/[`*_~@[\]>]|https?:\/\/|www\./.test(body || "") && !/^\s{0,3}(?:[-*]|\d{1,4}[.)])\s/m.test(body || "");
}
