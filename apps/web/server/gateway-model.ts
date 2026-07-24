import type {
  LanguageModelV3,
  LanguageModelV3CallOptions,
  LanguageModelV3GenerateResult,
  LanguageModelV3Message,
  LanguageModelV3StreamResult,
} from "@ai-sdk/provider";

type GatewayMessage = { role: string; content: string };

type GatewayChatResponse = {
  id?: string;
  model?: string;
  content: string;
  prompt_tokens?: number;
  completion_tokens?: number;
  finish_reason?: string;
};

function textFromParts(
  content: Array<{ type: string; text?: string }>,
): string {
  return content
    .filter((p) => p.type === "text" && typeof p.text === "string")
    .map((p) => p.text as string)
    .join("\n");
}

function promptToGatewayMessages(prompt: LanguageModelV3Message[]): GatewayMessage[] {
  const out: GatewayMessage[] = [];
  for (const msg of prompt) {
    if (msg.role === "system") {
      out.push({ role: "system", content: msg.content });
      continue;
    }
    if (msg.role === "user") {
      out.push({ role: "user", content: textFromParts(msg.content) });
      continue;
    }
    if (msg.role === "assistant") {
      out.push({ role: "assistant", content: textFromParts(msg.content) });
      continue;
    }
    // tool role is not used by CyberOS chat AI prompts
  }
  return out;
}

/**
 * LanguageModelV3 adapter for CyberOS ai-gateway `POST /v1/chat`
 * (`{ alias, messages }` + `x-tenant-id`). `modelId` is the gateway alias
 * (e.g. `chat.smart`, `chat.fast`).
 */
export function cyberosGatewayModel(alias: string, tenantId: string): LanguageModelV3 {
  return {
    specificationVersion: "v3",
    provider: "cyberos-ai-gateway",
    modelId: alias,
    supportedUrls: {},
    async doGenerate(options: LanguageModelV3CallOptions): Promise<LanguageModelV3GenerateResult> {
      const base = (process.env.AI_GATEWAY_URL || "").trim().replace(/\/$/, "");
      if (!base) {
        throw new Error("AI_GATEWAY_URL is not configured");
      }
      const messages = promptToGatewayMessages(options.prompt);
      const body = {
        alias,
        messages,
        max_tokens: options.maxOutputTokens,
        temperature: options.temperature,
      };
      const res = await fetch(`${base}/v1/chat`, {
        method: "POST",
        headers: {
          "content-type": "application/json",
          "x-tenant-id": tenantId,
        },
        body: JSON.stringify(body),
      });
      if (!res.ok) {
        throw new Error(`ai-gateway returned ${res.status}`);
      }
      const parsed = (await res.json()) as GatewayChatResponse;
      const content = (parsed.content || "").trim();
      if (!content) {
        throw new Error("ai-gateway returned empty content");
      }
      const inputTotal = parsed.prompt_tokens;
      const outputTotal = parsed.completion_tokens;
      return {
        content: [{ type: "text", text: content }],
        finishReason: {
          unified: "stop",
          raw: parsed.finish_reason ?? "stop",
        },
        usage: {
          inputTokens: {
            total: inputTotal,
            noCache: inputTotal,
            cacheRead: undefined,
            cacheWrite: undefined,
          },
          outputTokens: {
            total: outputTotal,
            text: outputTotal,
            reasoning: undefined,
          },
        },
        warnings: [],
        request: { body },
        response: {
          id: parsed.id,
          modelId: parsed.model ?? alias,
          body: parsed,
        },
      };
    },
    async doStream(_options: LanguageModelV3CallOptions): Promise<LanguageModelV3StreamResult> {
      throw new Error("cyberos-ai-gateway model does not support streaming");
    },
  };
}
