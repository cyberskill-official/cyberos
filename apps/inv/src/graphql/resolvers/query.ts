import type { CyberOSContext } from "@cyberos/subgraph-kit";

export const Query = {
  invHealth: (_root: unknown, _args: unknown, _ctx: CyberOSContext) => ({
    ok: true,
    module: "INV",
    version: "0.1.0",
  }),
};
