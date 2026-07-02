// Smoke assertions for the rich-text parser (apps/web/src/lib/richtext.ts). The web app has no test runner
// (gate = tsc + vite), so this runs the pure parser directly under node's TS type-stripping:
//   node scripts/richtext-smoke.ts        (from apps/web; node >= 23.6)
// Keep every case here boring and explicit - this is the contract the message renderer relies on.

import assert from "node:assert/strict";
import { isPlainText, parseInline, parseRich } from "../src/lib/richtext.ts";
import type { Block, Inline, MentionCandidate } from "../src/lib/richtext.ts";

let n = 0;
function ok(name: string, fn: () => void) {
  fn();
  n += 1;
  console.log(`ok ${n} - ${name}`);
}

const MENTIONS: MentionCandidate[] = [
  { name: "Anna Vu", me: false },
  { name: "Anna", me: false },
  { name: "Stephen", me: true },
  { name: "stephen.cheng", me: true },
];

const types = (nodes: Inline[]) => nodes.map((x) => x.t).join(",");

ok("plain text stays a single text node", () => {
  const r = parseInline("xin chao ca nha");
  assert.equal(types(r), "text");
});

ok("isPlainText fast path", () => {
  assert.equal(isPlainText("hello world"), true);
  assert.equal(isPlainText("has `code`"), false);
  assert.equal(isPlainText("see https://x.vn"), false);
  assert.equal(isPlainText("- a list"), false);
  assert.equal(isPlainText("ping @Anna"), false);
});

ok("bold", () => {
  const r = parseInline("a **bold** b");
  assert.equal(types(r), "text,bold,text");
  const bold = r[1] as Extract<Inline, { t: "bold" }>;
  assert.equal((bold.children[0] as { text: string }).text, "bold");
});

ok("italic star and underscore", () => {
  assert.equal(types(parseInline("an *it* b")), "text,italic,text");
  assert.equal(types(parseInline("an _it_ b")), "text,italic,text");
});

ok("snake_case and math stay literal", () => {
  assert.equal(types(parseInline("chat_message_created")), "text");
  assert.equal(types(parseInline("2*3*4 = 24")), "text");
  assert.equal(types(parseInline("2 * 3 * 4")), "text");
});

ok("strike", () => {
  assert.equal(types(parseInline("x ~~gone~~ y")), "text,strike,text");
});

ok("inline code wins over bold and mentions", () => {
  const r = parseInline("run `cargo **test** @Anna` now", MENTIONS);
  assert.equal(types(r), "text,code,text");
  assert.equal((r[1] as { text: string }).text, "cargo **test** @Anna");
});

ok("nested bold+italic", () => {
  const r = parseInline("**bold *both***");
  assert.equal(r[0].t, "bold");
});

ok("markdown link", () => {
  const r = parseInline("see [docs](https://os.cyberskill.world/web/) ok");
  assert.equal(types(r), "text,link,text");
  const l = r[1] as Extract<Inline, { t: "link" }>;
  assert.equal(l.href, "https://os.cyberskill.world/web/");
  assert.equal(l.label, "docs");
});

ok("bare url autolink trims trailing punctuation", () => {
  const r = parseInline("go to https://cyberskill.world/a?b=1, then rest");
  const l = r[1] as Extract<Inline, { t: "link" }>;
  assert.equal(l.href, "https://cyberskill.world/a?b=1");
  assert.equal((r[2] as { text: string }).text.startsWith(","), true);
});

ok("www url gets https href", () => {
  const r = parseInline("www.cyberskill.world is us");
  const l = r[0] as Extract<Inline, { t: "link" }>;
  assert.equal(l.href, "https://www.cyberskill.world");
  assert.equal(l.label, "www.cyberskill.world");
});

ok("url inside a word stays literal", () => {
  assert.equal(types(parseInline("xhttps://nope")), "text");
});

ok("mention basic + me flag", () => {
  const r = parseInline("cc @Stephen please", MENTIONS);
  assert.equal(types(r), "text,mention,text");
  const m = r[1] as Extract<Inline, { t: "mention" }>;
  assert.equal(m.me, true);
  assert.equal(m.text, "Stephen");
});

ok("longest mention wins and names may contain spaces", () => {
  const r = parseInline("ask @Anna Vu about it", MENTIONS);
  const m = r[1] as Extract<Inline, { t: "mention" }>;
  assert.equal(m.text, "Anna Vu");
});

ok("mention is case-insensitive but keeps typed text", () => {
  const r = parseInline("hi @anna!", MENTIONS);
  const m = r[1] as Extract<Inline, { t: "mention" }>;
  assert.equal(m.text, "anna");
  assert.equal(m.me, false);
});

ok("email-style @ stays literal", () => {
  assert.equal(types(parseInline("mail info@cyberskill.world ok", MENTIONS)), "text");
});

ok("unknown mention stays literal", () => {
  assert.equal(types(parseInline("hey @Nobody", MENTIONS)), "text");
});

ok("vietnamese text is untouched", () => {
  const r = parseInline("Chuc mung nam moi Đặng Thị Hồng Nhung nhé");
  assert.equal(types(r), "text");
});

const btypes = (bs: Block[]) => bs.map((b) => b.t).join(",");

ok("code block with language", () => {
  const bs = parseRich("before\n```rust\nfn main() {}\n```\nafter");
  assert.equal(btypes(bs), "p,codeblock,p");
  const cb = bs[1] as Extract<Block, { t: "codeblock" }>;
  assert.equal(cb.lang, "rust");
  assert.equal(cb.text, "fn main() {}");
});

ok("unclosed fence swallows the rest as code", () => {
  const bs = parseRich("```\nlet a = 1;\nlet b = 2;");
  assert.equal(btypes(bs), "codeblock");
  assert.equal((bs[0] as { text: string }).text, "let a = 1;\nlet b = 2;");
});

ok("blockquote groups consecutive lines", () => {
  const bs = parseRich("> one\n> two\nplain");
  assert.equal(btypes(bs), "quote,p");
  assert.equal((bs[0] as Extract<Block, { t: "quote" }>).lines.length, 2);
});

ok("unordered and ordered lists", () => {
  const bs = parseRich("- a\n- b\n\n1. x\n2. y");
  assert.equal(btypes(bs), "ul,ol");
  assert.equal((bs[1] as Extract<Block, { t: "ol" }>).start, 1);
});

ok("paragraph keeps shift+enter lines", () => {
  const bs = parseRich("line one\nline two");
  assert.equal(btypes(bs), "p");
  assert.equal((bs[0] as Extract<Block, { t: "p" }>).lines.length, 2);
});

ok("crlf bodies normalize", () => {
  const bs = parseRich("a\r\n```\r\nx\r\n```");
  assert.equal(btypes(bs), "p,codeblock");
});

ok("empty body yields no blocks", () => {
  assert.equal(parseRich("").length, 0);
});

console.log(`richtext-smoke: ${n} checks passed`);
