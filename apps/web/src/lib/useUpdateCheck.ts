import { useEffect, useRef, useState } from "react";

// Cross-platform "a new build was deployed" detector. Every CyberOS surface - browser, installed PWA, the
// Tauri desktop shell, and the Capacitor mobile shell - loads the same /web/ bundle, so this one web-layer
// check covers them all; there is no per-platform update code to maintain.
//
// How it works: each `npm run build` writes /web/version.json with a unique build id (scripts/stamp-sw.mjs,
// the same id that stamps the service-worker cache). On mount we record the id we are running, then re-fetch
// version.json on an interval and whenever the tab regains focus or the network returns. When the served id
// differs from the one we started with, a newer build is live, so we surface a reload prompt. Reloading is
// what applies it: the service worker is network-first, so a reload pulls the fresh index + hashed bundles
// and the new worker activates and purges the old caches. version.json is excluded from the SW cache (see
// public/sw.js) and fetched no-store, so the poll always sees the truth.

const VERSION_URL = `${import.meta.env.BASE_URL}version.json`;
const POLL_MS = 5 * 60 * 1000; // 5 minutes

async function fetchBuild(): Promise<string | null> {
  try {
    const res = await fetch(`${VERSION_URL}?t=${Date.now()}`, { cache: "no-store" });
    if (!res.ok) return null;
    const body: unknown = await res.json();
    const build = (body as { build?: unknown }).build;
    return typeof build === "string" && build.length > 0 ? build : null;
  } catch {
    return null; // offline or not deployed yet - just try again next tick
  }
}

// Returns true once a newer build than the one this tab loaded with is live.
export function useUpdateCheck(): boolean {
  const [updateReady, setUpdateReady] = useState(false);
  const runningBuild = useRef<string | null>(null);

  useEffect(() => {
    let alive = true;

    const check = async () => {
      const build = await fetchBuild();
      if (!alive || build === null) return;
      if (runningBuild.current === null) {
        runningBuild.current = build; // baseline: the build this tab is running
      } else if (build !== runningBuild.current) {
        setUpdateReady(true);
      }
    };

    check();
    const timer = window.setInterval(check, POLL_MS);
    const onVisible = () => {
      if (document.visibilityState === "visible") check();
    };
    document.addEventListener("visibilitychange", onVisible);
    window.addEventListener("online", check);

    return () => {
      alive = false;
      window.clearInterval(timer);
      document.removeEventListener("visibilitychange", onVisible);
      window.removeEventListener("online", check);
    };
  }, []);

  return updateReady;
}
