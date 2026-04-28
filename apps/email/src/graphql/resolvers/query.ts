import type { CyberOSContext } from "@cyberos/subgraph-kit";

export const Query = {
  emailHealth: (_root: unknown, _args: unknown, _ctx: CyberOSContext) => ({
    ok: true,
    module: "EMAIL",
    version: "0.1.0",
  }),
};
