// Minimal service worker: enables PWA installability and an offline app shell. API and websocket traffic is
// never cached; the network-first fetch below means a reload always gets the fresh index + hashed bundles.
// 20260720104229 is stamped by scripts/stamp-sw.mjs on every `npm run build`, so each deploy gets its own cache
// name and activation below deletes every older cache (stale hashed assets no longer accumulate forever).
const CACHE = "cyberos-shell-20260720104229";

self.addEventListener("install", () => self.skipWaiting());
self.addEventListener("activate", (e) =>
  e.waitUntil(
    caches
      .keys()
      .then((keys) => Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k))))
      .then(() => self.clients.claim()),
  ),
);

self.addEventListener("fetch", (event) => {
  const req = event.request;
  if (req.method !== "GET") return;
  let url;
  try {
    url = new URL(req.url);
  } catch {
    return;
  }
  if (url.origin !== self.location.origin) return; // only our own origin
  if (url.pathname.startsWith("/v1/") || url.pathname === "/healthz") return; // never the API
  if (url.pathname.endsWith("/version.json")) return; // update-check probe: always hit the network, never cache
  event.respondWith(
    fetch(req)
      .then((res) => {
        if (res && res.ok) {
          const copy = res.clone();
          caches.open(CACHE).then((c) => c.put(req, copy)).catch(() => {});
        }
        return res;
      })
      .catch(() => caches.match(req).then((r) => r || caches.match("/"))),
  );
});
