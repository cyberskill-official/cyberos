import type { CyberOSContext } from "@cyberos/subgraph-kit";

export const Query = {
  esopHealth: (_root: unknown, _args: unknown, _ctx: CyberOSContext) => ({
    ok: true,
    module: "ESOP",
    version: "0.1.0",
  }),
};
