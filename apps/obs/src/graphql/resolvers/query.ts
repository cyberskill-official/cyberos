import type { CyberOSContext } from "@cyberos/subgraph-kit";

export const Query = {
  obsHealth: (_root: unknown, _args: unknown, _ctx: CyberOSContext) => ({
    ok: true,
    module: "OBS",
    version: "0.1.0",
  }),
};
