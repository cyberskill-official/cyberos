import { foglamp, type Collector } from "foglamp";

let _fog: Collector | null = null;

/** Lazy singleton — created on first AI call, after Vite has loaded .env. */
export function getFog(): Collector {
  if (!_fog) {
    _fog = foglamp({
      // Vite configureServer only — safe; no-op in production/edge builds.
      hud: true,
    });
  }
  return _fog;
}
