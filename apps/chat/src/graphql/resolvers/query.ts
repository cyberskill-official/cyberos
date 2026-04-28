import type { CyberOSContext } from "@cyberos/subgraph-kit";

export const Query = {
  chatHealth: (_root: unknown, _args: unknown, _ctx: CyberOSContext) => ({
    ok: true,
    module: "CHAT",
    version: "0.1.0",
  }),
};
