import type { CyberOSContext } from "@cyberos/subgraph-kit";

export const Query = {
  brainHealth: (_root: unknown, _args: unknown, _ctx: CyberOSContext) => ({
    ok: true,
    module: "BRAIN",
    version: "0.1.0",
  }),
};
