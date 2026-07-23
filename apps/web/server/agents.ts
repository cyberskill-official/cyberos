import { generateText } from "ai";
import { getFog } from "./fog";
import { cyberosGatewayModel } from "./gateway-model";

export type ChatAiContext = {
  tenantId: string;
  channelId: string;
  subjectId: string;
};

const UNAVAILABLE = "AI is unavailable right now";

function customer(tenantId: string) {
  return { id: tenantId };
}

/** Mirror of services/chat/src/ai.rs summarize system prompt. */
const SUMMARIZE_SYSTEM =
  "You summarize a team chat conversation for a bilingual Vietnamese/English software team. \
Write 3-8 short markdown bullet points ('- ') covering the decisions made, open questions, \
and important updates. Answer in the transcript's dominant language. Output only the \
bullets, no preamble.";

/** Mirror of services/chat/src/ai.rs actions system prompt. */
const ACTIONS_SYSTEM =
  "Extract the concrete action items from this team chat transcript. Output one markdown \
bullet per item ('- '), naming the owner when the conversation states one ('- Name: task') \
and keeping each item under 20 words, in the transcript's dominant language (Vietnamese or \
English). If there are no action items, output exactly: (none)";

/** Mirror of services/chat/src/ai.rs replies system prompt. */
const REPLIES_SYSTEM =
  "Suggest 3 short, natural replies the user could send next in this team chat. Match the \
conversation's language (Vietnamese or English) and tone, keep each reply under 15 words, \
and make the three replies meaningfully different. Output exactly 3 lines, one reply per \
line, with no numbering, bullets, or quotes.";

export async function runSummarizer(
  ctx: ChatAiContext,
  transcript: string,
): Promise<string> {
  const fog = getFog();
  const { text } = await generateText({
    model: cyberosGatewayModel("chat.smart", ctx.tenantId),
    system: SUMMARIZE_SYSTEM,
    prompt: transcript,
    telemetry: {
      integrations: [
        fog.integration({
          agentName: "chat-summarizer",
          sessionId: ctx.channelId,
          customer: customer(ctx.tenantId),
          metadata: {
            subjectId: ctx.subjectId,
            alias: "chat.smart",
          },
        }),
      ],
    },
  });
  await fog.flush();
  const out = text.trim();
  if (!out) throw new Error(UNAVAILABLE);
  return out;
}

export async function runActionItems(
  ctx: ChatAiContext,
  transcript: string,
): Promise<string> {
  const fog = getFog();
  const { text } = await generateText({
    model: cyberosGatewayModel("chat.smart", ctx.tenantId),
    system: ACTIONS_SYSTEM,
    prompt: transcript,
    telemetry: {
      integrations: [
        fog.integration({
          agentName: "chat-action-items",
          sessionId: ctx.channelId,
          customer: customer(ctx.tenantId),
          metadata: {
            subjectId: ctx.subjectId,
            alias: "chat.smart",
          },
        }),
      ],
    },
  });
  await fog.flush();
  const out = text.trim();
  if (!out) throw new Error(UNAVAILABLE);
  return out;
}

export function parseSuggestions(raw: string): string[] {
  return raw
    .split(/\r?\n/)
    .map((l) =>
      l
        .trim()
        .replace(/^[-*•]\s*/, "")
        .replace(/^\d+[.)]\s*/, "")
        .trim()
        .replace(/^"|"$/g, ""),
    )
    .filter((l) => l.length > 0)
    .slice(0, 3);
}

export async function runReplySuggest(
  ctx: ChatAiContext,
  transcript: string,
  myName: string,
): Promise<string[]> {
  const fog = getFog();
  const { text } = await generateText({
    model: cyberosGatewayModel("chat.fast", ctx.tenantId),
    system: REPLIES_SYSTEM,
    prompt: `${transcript}\n\n(The person replying is: ${myName})`,
    telemetry: {
      integrations: [
        fog.integration({
          agentName: "chat-reply-suggest",
          sessionId: ctx.channelId,
          customer: customer(ctx.tenantId),
          metadata: {
            subjectId: ctx.subjectId,
            alias: "chat.fast",
          },
        }),
      ],
    },
  });
  await fog.flush();
  const suggestions = parseSuggestions(text);
  if (suggestions.length === 0) throw new Error(UNAVAILABLE);
  return suggestions;
}

export async function runTranslator(
  tenantId: string,
  subjectId: string,
  text: string,
  targetLang: string,
): Promise<string> {
  const fog = getFog();
  const system = `You are a translation engine. Translate the user's message into ${targetLang}, preserving meaning, \
tone, names, and formatting. Output only the translation, with no quotes, labels, or commentary.`;
  const { text: translated } = await generateText({
    model: cyberosGatewayModel("chat.fast", tenantId),
    system,
    prompt: text,
    telemetry: {
      integrations: [
        fog.integration({
          agentName: "chat-translator",
          customer: customer(tenantId),
          metadata: {
            subjectId,
            targetLang,
            alias: "chat.fast",
          },
        }),
      ],
    },
  });
  await fog.flush();
  const out = translated.trim();
  if (!out) throw new Error(UNAVAILABLE);
  return out;
}
