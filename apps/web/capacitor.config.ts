import type { CapacitorConfig } from '@capacitor/cli';

// Capacitor wraps the SAME apps/web build (React SPA) as a native iOS / Android app, so there is one UI for
// web + desktop + mobile. `webDir` is the built bundle that `npm run build` writes and `npx cap sync` copies
// into the native projects - the app ships those assets (store-safe), rather than pointing at a live URL.
//
// This config is inert until the one-time init is done (see RELEASE.md):
//   cd apps/web
//   npm i -D @capacitor/core @capacitor/cli @capacitor/ios @capacitor/android
//   npx cap add ios && npx cap add android
// After that, set the repo variable MOBILE_RELEASE=true to turn on the mobile jobs in release.yml.
// CapacitorHttp routes window.fetch/XHR on native through the platform HTTP stack instead of the webview.
// That is what makes the absolute API base in src/lib/api.ts usable at all: the bundle is served from
// capacitor://localhost, so every call to https://os.cyberskill.world is cross-origin, and the server does
// not answer preflight - verified 2026-07-20, `OPTIONS /v1/auth/token` returns 405 with no
// access-control-* headers. A webview fetch would be blocked before it left the device. Native HTTP is not
// subject to CORS, so no server change is needed to unblock mobile.
//
// The alternative was adding CORS for capacitor://localhost and http://localhost at the edge. That remains
// the more conventional fix and is worth doing if the API is ever called from a real third-party origin,
// but it needs a production deploy and the config for the os.cyberskill.world origin is not in this repo.
//
// Known trade-off: CapacitorHttp has a history of mishandling binary request bodies, which is what
// apiUploadRaw() sends. Attachment and avatar upload must be exercised on a device before release; if it
// breaks, exempt that one call rather than disabling the plugin wholesale.
const config: CapacitorConfig = {
  appId: 'os.cyberskill.world',
  appName: 'CyberOS',
  webDir: '../console/web',
  plugins: {
    CapacitorHttp: { enabled: true },
  },
};

export default config;
