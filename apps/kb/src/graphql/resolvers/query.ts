import type { CyberOSContext } from "@cyberos/subgraph-kit";

export const Query = {
  kbHealth: (_root: unknown, _args: unknown, _ctx: CyberOSContext) => ({
    ok: true,
    module: "KB",
    version: "0.1.0",
  }),
};
