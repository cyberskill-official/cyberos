/**
 * @cyberos/mcp-server — common helpers for module MCP tool registration.
 *
 * Each module declares its tools in `apps/{module}/src/mcp/tools.ts`. The
 * central MCP server (apps/mcp) loads them all and exposes a single
 * stdio/SSE transport to LLM agents. RBAC is reused from the GraphQL layer:
 * an MCP call carries the same JWT that a GraphQL query would.
 */

import { z } from "zod";
import type { RequestContext, ModuleCode } from "@cyberos/shared";

export interface McpToolDef<TInput, TOutput> {
  /** Tool name as exposed to the LLM. Must be `{module}.{action}` (snake_case). */
  readonly name: `${Lowercase<string>}.${Lowercase<string>}`;
  /** Module that owns this tool. */
  readonly module: ModuleCode;
  /** Short description shown to the LLM in the tool catalogue. */
  readonly description: string;
  /** Required scopes — gated against the JWT before invocation. */
  readonly scopes: readonly string[];
  /** zod schema for the input payload. */
  readonly input: z.ZodType<TInput>;
  /** zod schema for the output payload (helps the central server type-check). */
  readonly output: z.ZodType<TOutput>;
  /** Implementation. Always receives a hydrated RequestContext. */
  readonly handler: (ctx: RequestContext, input: TInput) => Promise<TOutput>;
}

export function defineTool<TInput, TOutput>(
  def: McpToolDef<TInput, TOutput>,
): McpToolDef<TInput, TOutput> {
  if (!/^[a-z][a-z0-9_]*\.[a-z][a-z0-9_]*$/.test(def.name)) {
    throw new Error(`MCP tool name "${def.name}" must be snake_case {module}.{action}`);
  }
  return def;
}

/** Bundle of tools a module exports. The central MCP server imports one of these per module. */
export interface ModuleToolset {
  readonly module: ModuleCode;
  readonly tools: readonly McpToolDef<unknown, unknown>[];
}

export type { z };
