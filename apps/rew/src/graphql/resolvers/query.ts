import type { CyberOSContext } from "@cyberos/subgraph-kit";

export const Query = {
  rewHealth: (_root: unknown, _args: unknown, _ctx: CyberOSContext) => ({
    ok: true,
    module: "REW",
    version: "0.1.0",
  }),
};
