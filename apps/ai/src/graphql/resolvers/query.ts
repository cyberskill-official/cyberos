import type { CyberOSContext } from "@cyberos/subgraph-kit";

export const Query = {
  aiHealth: (_root: unknown, _args: unknown, _ctx: CyberOSContext) => ({
    ok: true,
    module: "AI",
    version: "0.1.0",
  }),
};
