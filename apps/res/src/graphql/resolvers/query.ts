import type { CyberOSContext } from "@cyberos/subgraph-kit";

export const Query = {
  resHealth: (_root: unknown, _args: unknown, _ctx: CyberOSContext) => ({
    ok: true,
    module: "RES",
    version: "0.1.0",
  }),
};
