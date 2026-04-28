/**
 * MCP tools exposed by {{PACKAGE}}.
 *
 * Naming: `{{NAMESPACE}}.{action}` — snake_case, single dot. The central MCP
 * server (apps/mcp) imports the `toolset` export from every module and
 * publishes the union to LLM agents.
 *
 * Adding a tool:
 *   1. Define it with `defineTool({ ... })`.
 *   2. Append to the `tools` array.
 *   3. Add the corresponding required scope to AUTH if it doesn't exist.
 */

import { defineTool, type ModuleToolset } from "@cyberos/mcp-server";

// Example placeholder — remove when you wire your first real tool.
const ping = defineTool({
  name: "{{NAMESPACE}}.ping",
  module: "{{MODULE}}",
  description: "Liveness probe for the {{NAMESPACE}} module.",
  scopes: [],
  input: (await import("zod")).z.object({}).strict(),
  output: (await import("zod")).z.object({ ok: (await import("zod")).z.literal(true) }),
  handler: async () => ({ ok: true as const }),
});

export const toolset: ModuleToolset = {
  module: "{{MODULE}}",
  tools: [ping],
};
