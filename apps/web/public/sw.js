// Minimal service worker: enables PWA installability and an offline app shell. API and websocket traffic is
// never cached, and the hashed bundle filenames change every deploy, so updates always land on next load.
const CACHE = "cyberos-shell-v1";

self.addEventListener("install", () => self.skipWaiting());
self.addEventListener("activate", (e) => e.waitUntil(self.clients.claim()));

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
  event.respondWith(
    fetch(req)
      .then((res) => {
        if (res && res.ok) {
          const copy = res.clone();
          caches.open(CACHE).then((c) => c.put(req, copy)).catch(() => {});
        }
        return res;
      })
      .catch(() => caches.match(req).then((r) => r || caches.match("/web/"))),
  );
});
