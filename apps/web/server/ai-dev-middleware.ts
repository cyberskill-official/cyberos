import type { ServerResponse } from "node:http";
import type { Connect, PreviewServer, ViteDevServer } from "vite";
import {
  runActionItems,
  runReplySuggest,
  runSummarizer,
  runTranslator,
} from "./agents";

type ChatMessage = {
  sender_subject_id?: string;
  body?: string;
  deleted_at?: string | null;
};

const MAX_TRANSCRIPT_BYTES = 24 * 1024;
const UNAVAILABLE = "AI is unavailable right now";

function decodeJwt(token: string): Record<string, unknown> | null {
  try {
    let p = token.split(".")[1].replace(/-/g, "+").replace(/_/g, "/");
    p += "====".slice(p.length % 4 || 4);
    return JSON.parse(Buffer.from(p, "base64").toString("utf8"));
  } catch {
    return null;
  }
}

function bearer(req: Connect.IncomingMessage): string | null {
  const h = req.headers.authorization;
  if (!h || !h.startsWith("Bearer ")) return null;
  return h.slice("Bearer ".length).trim() || null;
}

function claimsFrom(req: Connect.IncomingMessage): {
  tenantId: string;
  subjectId: string;
} | null {
  const token = bearer(req);
  if (!token) return null;
  const c = decodeJwt(token);
  if (!c) return null;
  const tenantId = typeof c.tenant_id === "string" ? c.tenant_id : "";
  const subjectId = typeof c.sub === "string" ? c.sub : "";
  if (!tenantId || !subjectId) return null;
  return { tenantId, subjectId };
}

function speaker(names: Record<string, string>, id: string): string {
  const n = names[id];
  if (!n) return id.slice(0, 8);
  const clean = n
    .replace(/[\n\r:]/g, " ")
    .trim()
    .slice(0, 40);
  return clean || id.slice(0, 8);
}

function formatTranscript(
  rows: Array<{ sender: string; body: string }>,
  names: Record<string, string>,
): string {
  const lines = rows.map((r) => `${speaker(names, r.sender)}: ${r.body.trim()}`);
  let start = 0;
  let total = lines.reduce((n, l) => n + l.length + 1, 0);
  while (total > MAX_TRANSCRIPT_BYTES && start < lines.length) {
    total -= lines[start].length + 1;
    start += 1;
  }
  return lines.slice(start).join("\n");
}

async function readJson(req: Connect.IncomingMessage): Promise<unknown> {
  const chunks: Buffer[] = [];
  for await (const chunk of req) {
    chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
  }
  if (chunks.length === 0) return {};
  return JSON.parse(Buffer.concat(chunks).toString("utf8") || "{}");
}

function sendJson(res: ServerResponse, status: number, body: unknown): void {
  res.statusCode = status;
  res.setHeader("content-type", "application/json");
  res.end(JSON.stringify(body));
}

function sendText(res: ServerResponse, status: number, msg: string): void {
  res.statusCode = status;
  res.setHeader("content-type", "text/plain; charset=utf-8");
  res.end(msg);
}

function chatOrigin(): string {
  return (process.env.CHAT_URL || "http://127.0.0.1:7720").replace(/\/$/, "");
}

async function loadTranscript(
  req: Connect.IncomingMessage,
  channelId: string,
  limit: number,
  names: Record<string, string>,
): Promise<{ rows: Array<{ sender: string; body: string }>; transcript: string }> {
  const auth = req.headers.authorization || "";
  const url = `${chatOrigin()}/v1/chat/channels/${channelId}/messages?limit=${limit}`;
  const res = await fetch(url, { headers: { authorization: auth } });
  if (res.status === 401) throw Object.assign(new Error("unauthorized"), { status: 401 });
  if (res.status === 403) throw Object.assign(new Error("not a channel member"), { status: 403 });
  if (!res.ok) throw Object.assign(new Error(UNAVAILABLE), { status: 502 });
  const msgs = (await res.json()) as ChatMessage[];
  const rows = msgs
    .filter((m) => !m.deleted_at && (m.body || "").trim())
    .map((m) => ({
      sender: m.sender_subject_id || "",
      body: (m.body || "").trim(),
    }))
    .filter((r) => r.sender && r.body)
    // list is newest-first; transcript wants oldest-first
    .reverse();
  if (rows.length === 0) {
    throw Object.assign(
      new Error("nothing to work with yet - this conversation has no messages"),
      { status: 400 },
    );
  }
  return { rows, transcript: formatTranscript(rows, names) };
}

const AI_CHANNEL =
  /^\/v1\/chat\/channels\/([^/]+)\/ai\/(summarize|actions|replies)\/?$/;
const TRANSLATE = /^\/v1\/chat\/translate\/?$/;

export function attachFoglampAiMiddleware(
  server: ViteDevServer | PreviewServer,
): void {
  server.middlewares.use(async (req, res, next) => {
    try {
      const url = req.url || "";
      const path = url.split("?")[0] || "";
      if (req.method !== "POST") {
        next();
        return;
      }

      const channelMatch = path.match(AI_CHANNEL);
      if (channelMatch) {
        const channelId = channelMatch[1];
        const action = channelMatch[2] as "summarize" | "actions" | "replies";
        const claims = claimsFrom(req);
        if (!claims) {
          sendText(res, 401, "unauthorized");
          return;
        }
        if (!(process.env.AI_GATEWAY_URL || "").trim()) {
          sendText(res, 502, UNAVAILABLE);
          return;
        }
        const body = (await readJson(req)) as {
          limit?: number;
          names?: Record<string, string>;
        };
        const names = body.names || {};
        const limit =
          action === "replies"
            ? 12
            : Math.min(200, Math.max(10, body.limit ?? 100));
        const { rows, transcript } = await loadTranscript(
          req,
          channelId,
          limit,
          names,
        );
        const ctx = {
          tenantId: claims.tenantId,
          channelId,
          subjectId: claims.subjectId,
        };

        if (action === "summarize") {
          const text = await runSummarizer(ctx, transcript);
          sendJson(res, 200, { text, message_count: rows.length });
          return;
        }
        if (action === "actions") {
          const text = await runActionItems(ctx, transcript);
          sendJson(res, 200, { text, message_count: rows.length });
          return;
        }
        const myName = speaker(names, claims.subjectId);
        const suggestions = await runReplySuggest(ctx, transcript, myName);
        sendJson(res, 200, { suggestions });
        return;
      }

      if (TRANSLATE.test(path)) {
        const claims = claimsFrom(req);
        if (!claims) {
          sendText(res, 401, "unauthorized");
          return;
        }
        if (!(process.env.AI_GATEWAY_URL || "").trim()) {
          sendText(
            res,
            502,
            "translation is unavailable (ai-gateway not configured)",
          );
          return;
        }
        const body = (await readJson(req)) as {
          text?: string;
          target_lang?: string;
        };
        const text = (body.text || "").trim();
        const target = (body.target_lang || "").trim();
        if (!text) {
          sendText(res, 400, "text is required");
          return;
        }
        if (text.length > 8 * 1024) {
          sendText(res, 400, "text is too long to translate");
          return;
        }
        if (!target) {
          sendText(res, 400, "target_lang is required");
          return;
        }
        const translated = await runTranslator(
          claims.tenantId,
          claims.subjectId,
          text,
          target,
        );
        sendJson(res, 200, { translated });
        return;
      }

      next();
    } catch (e) {
      const status =
        e && typeof e === "object" && "status" in e
          ? Number((e as { status: number }).status)
          : 502;
      const msg = e instanceof Error ? e.message : UNAVAILABLE;
      if (status === 400 || status === 401 || status === 403) {
        sendText(res, status, msg);
        return;
      }
      console.warn("[foglamp-ai]", msg);
      sendText(res, 502, UNAVAILABLE);
    }
  });
}
