import { Fragment, useMemo } from "react";
import type { ReactNode } from "react";
import type { Block, Inline, MentionCandidate } from "./richtext";
import { isPlainText, parseRich, prepareMentions } from "./richtext";

// Renders a message body as rich text (see richtext.ts for the grammar). Everything user-authored lands in
// React text nodes - no dangerouslySetInnerHTML anywhere - so the renderer is XSS-safe by construction.
// Links open in a new tab with rel=noopener; mentions of the viewer get the stronger "me" tint.

function renderInline(nodes: Inline[], keyBase: string): ReactNode {
  return nodes.map((n, idx) => {
    const key = `${keyBase}.${idx}`;
    switch (n.t) {
      case "text":
        return <Fragment key={key}>{n.text}</Fragment>;
      case "code":
        return (
          <code key={key} className="rt-code">
            {n.text}
          </code>
        );
      case "bold":
        return <strong key={key}>{renderInline(n.children, key)}</strong>;
      case "italic":
        return <em key={key}>{renderInline(n.children, key)}</em>;
      case "strike":
        return <s key={key}>{renderInline(n.children, key)}</s>;
      case "link":
        return (
          <a key={key} className="rt-link" href={n.href} target="_blank" rel="noopener noreferrer">
            {n.label}
          </a>
        );
      case "mention":
        return (
          <span key={key} className={"rt-mention" + (n.me ? " me" : "")}>
            @{n.text}
          </span>
        );
    }
  });
}

function renderLines(lines: Inline[][], keyBase: string): ReactNode {
  return lines.map((ln, idx) => (
    <Fragment key={`${keyBase}.${idx}`}>
      {idx > 0 && <br />}
      {renderInline(ln, `${keyBase}.${idx}`)}
    </Fragment>
  ));
}

function renderBlock(b: Block, idx: number): ReactNode {
  const key = `b${idx}`;
  switch (b.t) {
    case "p":
      return <p key={key} className="rt-p">{renderLines(b.lines, key)}</p>;
    case "codeblock":
      return (
        <pre key={key} className="rt-pre">
          {b.lang && <span className="rt-lang">{b.lang}</span>}
          <code>{b.text}</code>
        </pre>
      );
    case "quote":
      return (
        <blockquote key={key} className="rt-quote">
          {renderLines(b.lines, key)}
        </blockquote>
      );
    case "ul":
      return (
        <ul key={key} className="rt-list">
          {b.items.map((it, j) => (
            <li key={`${key}.${j}`}>{renderInline(it, `${key}.${j}`)}</li>
          ))}
        </ul>
      );
    case "ol":
      return (
        <ol key={key} className="rt-list" start={b.start}>
          {b.items.map((it, j) => (
            <li key={`${key}.${j}`}>{renderInline(it, `${key}.${j}`)}</li>
          ))}
        </ol>
      );
  }
}

export function RichText({ text, mentions }: { text: string; mentions?: MentionCandidate[] }) {
  // Most messages are plain sentences; skip parsing entirely for those and let `.m-body`'s pre-wrap
  // handle the newlines, keeping the hot path as cheap as the old `{m.body}` render.
  const plain = isPlainText(text);
  const blocks = useMemo(
    () => (plain ? [] : parseRich(text, prepareMentions(mentions || []))),
    [plain, text, mentions],
  );
  if (plain) return <>{text}</>;
  return <>{blocks.map(renderBlock)}</>;
}
