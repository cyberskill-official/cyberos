import type { CyberOSContext } from "@cyberos/subgraph-kit";

export const Query = {
  learnHealth: (_root: unknown, _args: unknown, _ctx: CyberOSContext) => ({
    ok: true,
    module: "LEARN",
    version: "0.1.0",
  }),
};
