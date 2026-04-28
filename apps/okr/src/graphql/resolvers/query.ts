import type { CyberOSContext } from "@cyberos/subgraph-kit";

export const Query = {
  okrHealth: (_root: unknown, _args: unknown, _ctx: CyberOSContext) => ({
    ok: true,
    module: "OKR",
    version: "0.1.0",
  }),
};
