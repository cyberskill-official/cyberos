import type { CyberOSContext } from "@cyberos/subgraph-kit";

export const Query = {
  mcpHealth: (_root: unknown, _args: unknown, _ctx: CyberOSContext) => ({
    ok: true,
    module: "MCP",
    version: "0.1.0",
  }),
};
