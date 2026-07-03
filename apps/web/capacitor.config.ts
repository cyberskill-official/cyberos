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
const config: CapacitorConfig = {
  appId: 'world.cyberskill.cyberos',
  appName: 'CyberOS',
  webDir: '../console/web',
};

export default config;
