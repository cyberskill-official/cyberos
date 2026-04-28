import type { CyberOSContext } from "@cyberos/subgraph-kit";

export const Query = {
  cpHealth: (_root: unknown, _args: unknown, _ctx: CyberOSContext) => ({
    ok: true,
    module: "CP",
    version: "0.1.0",
  }),
};
