import type { CyberOSContext } from "@cyberos/subgraph-kit";

export const Query = {
  hrHealth: (_root: unknown, _args: unknown, _ctx: CyberOSContext) => ({
    ok: true,
    module: "HR",
    version: "0.1.0",
  }),
};
