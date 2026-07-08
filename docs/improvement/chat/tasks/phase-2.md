# Phase 2 - push + desktop GA (T-023..T-032)

Exit bar (report section 6): signed desktop builds auto-updating, notifications with the app closed, the
team defaults to desktop daily. This phase is the first go-live moment for web + desktop.

## T-023 Push relay worker (FCM v1 + APNs) + collapse + badges

- C-refs C38, C40 | P0/L | depends: T-008; blocked:input until Stephen provides FCM service account +
  APNs .p8 key (see RELEASE.md additions)
- Touch: new services/chat-push (or a module inside chat behind a feature flag - prefer separate worker
  binary sharing the crate); consumes push intents; devices.rs tokens.
- Spec: FCM HTTP v1 (OAuth2 service account) for Android + web fallback; APNs token-based (.p8) for
  iOS/macOS; payload privacy-preserving per FR-CHAT-104 (title + sender, no body); collapse key per
  channel so bursts update one notification; badge = unread total (APNs badge field, FCM count); retries
  with backoff; dead tokens pruned on provider 404/410; metrics: sent/failed/pruned per platform.
- Accept: staging device receives a push with app closed (Android emulator acceptable for FCM; APNs
  sandbox for iOS once T-033 exists - web push via T-024 counts for the first pass); intent -> delivery
  p95 logged; token prune test.
- Review: verify payload contains no message body (screenshot the notification).

## T-024 Web push: VAPID + declarative payload

- C-refs C39, C90 | P1/S | depends: T-023
- Touch: relay (webpush target), apps/web (subscription UI in settings, sw push handler), manifest.
- Spec: standard VAPID web push; payload uses the declarative web push JSON shape ({web_push: 8030,
  notification: {title, navigate, app_badge...}}) so Safari/iOS 18.4+ installed PWAs display without SW
  execution, and the classic SW push handler renders the same JSON elsewhere. Subscription lifecycle:
  store per device row, unsubscribe on logout.
- Accept: Chrome + Firefox notification with tab closed; payload validates against the declarative
  shape; iOS PWA verified when hardware available (note in ledger if deferred).
- Review: click-through lands on the exact channel/message (navigate URL correctness).

## T-025 Quiet hours / DND schedule

- C-refs C42 | P1/S | depends: none
- Touch: prefs.rs (+ migration for schedule fields), notify.rs fan-out check, relay respect, apps/web
  settings UI.
- Spec: per-user weekly schedule + timezone + "until" snooze; evaluated server-side at fan-out so every
  surface obeys; mentions can optionally break through (flag); default template = VN work calendar
  (Mon-Fri 08:30-18:00 active), user-editable.
- Accept: unit tests across timezone boundaries + DST-free VN case; push suppressed in quiet hours in
  staging test; UI writes schedule.
- Review: pick the default breakthrough policy (mentions pierce quiet hours: yes/no).

## T-026 Desktop: bundle SPA + CSP both surfaces

- C-refs C92, C99, C50 | P0/M | depends: none
- Touch: apps/desktop/src-tauri/tauri.conf.json (frontendDist -> built apps/web bundle), build wiring in
  release.yml; CSP for web (Caddy header) and shell (tauri.conf security.csp, null today); capability
  files narrowed.
- Spec: app ships its own assets (API base points at prod; env-switchable for staging); remote-URL mode
  kept behind a menu/flag as fallback; CSP: default-src 'self', connect-src app origin + wss, img-src
  'self' data: blob:, frame-ancestors 'none'; enumerate exact Tauri capabilities used (window, sql,
  notification, tray, deep-link, updater, single-instance) and remove the rest.
- Accept: app opens with network cable pulled (shell + cached store from T-027 renders); CSP violations
  absent in console on the happy path; capabilities file diff reviewed.
- Review: launch it offline yourself - the blank-window failure should be gone.

## T-027 Desktop: sql adapter + native notifications

- C-refs C93, C94 | P0/M | depends: T-026, T-015
- Touch: apps/desktop (tauri-plugin-sql registration), packages/chat-core adapters/tauri-sql; notification
  plugin + focus routing.
- Spec: chat-core detects Tauri and uses the sql plugin adapter (same schema/migrations as web); native
  notifications with click -> focus window + open channel; respect T-025 quiet hours (server already
  filters; client honors OS focus-assist state where readable).
- Accept: adapter passes the same conformance tests as web adapters; notification click lands on the
  channel with the window restored from background.
- Review: daily-drive it one afternoon; note anything that feels worse than the browser.

## T-028 Desktop: tray, deep links, window basics

- C-refs C95, C96, C98 | P1/M | depends: T-026
- Spec: tray icon with unread badge + context menu (open, mute 1h, quit); close-to-tray option; deep
  links cyberos://channel/<id> and cyberos://message/<id> via deep-link plugin; single-instance plugin
  routes second launches + deep links into the running window; window-state persistence; global shortcut
  (default Cmd/Ctrl+Shift+K) opening the quick switcher; autostart opt-in toggle in settings.
- Accept: clicking a cyberos:// link from a browser focuses the running app on the right channel; badge
  count matches sidebar; state survives restart.
- Review: choose the default for close-to-tray (VN team habit: suggest ON for Windows, OFF for macOS).

## T-029 Desktop: updater, crash reporting, CI matrix

- C-refs C97, C100, C101 | P1/M | depends: T-026; blocked:input (Tauri updater signing keys per
  RELEASE.md, stored offline)
- Spec: finish the scaffolded updater: generate + store signing keypair offline, wire pubkey into
  tauri.conf, release.yml signs artifacts and publishes the update manifest (stable + beta channels);
  update-available UX reuses UpdateBanner; crash reporting via sentry-tauri (or equivalent) into the
  T-031 project, release-tagged; CI matrix macOS/Windows/Linux on tags: build + launch smoke + login
  against staging + send/receive.
- Accept: v-tag produces signed installers for 3 OSes; an old build updates itself from the manifest in
  a test channel; a forced crash appears in the tracker with the right release.
- Review: keep the private key custody note in RELEASE.md accurate (who holds it, where).

## T-030 Version-skew policy

- C-refs C142 | P1/S | depends: none
- Touch: /version endpoint (min_supported_client, current), apps/web UpdateBanner blocking mode, ws
  handshake version check.
- Spec: clients send their build id on connect/sync; below min_supported -> blocking upgrade banner (web
  reload, desktop triggers updater); ws frames already carry v (T-019) so old clients degrade to
  sync-only rather than misparse; policy doc: how long old protocols stay supported (suggest: one phase).
- Accept: staging flip of min_supported blocks an old build and the banner path works on web + desktop.
- Review: agree the support window statement.

## T-031 Error tracking both sides

- C-refs C73 | P1/S | depends: none
- Spec: self-hosted GlitchTip (compose service) or Sentry SaaS - default GlitchTip for residency; Rust
  panic + error-level hook in chat service; browser SDK in apps/web with release tag = build id; desktop
  via T-029; scrub PII (no message bodies in events); alert route to the team channel/email.
- Accept: forced client error + forced server 500 both visible, release-tagged, PII-free; noise budget
  note (rate limits on event send).
- Review: decide GlitchTip vs Sentry once traffic shows volume (default stands unless you object).

## T-032 Feature flags + canary habit

- C-refs C140, C141 | P1/S | depends: none
- Touch: migration chat_feature_flags(tenant_id, flag, value, updated_at); chat service cached reader
  (30 s TTL); apps/web bootstrap exposure; console admin toggle later (T-065 scope).
- Spec: every group-N/phase-4 feature and the T-018 transport migration read a flag; canary playbook doc:
  enable for cyberskill tenant -> watch T-008/T-031 dashboards for a day -> enable globally; release
  pause rule tied to the T-047 error budget once it exists.
- Accept: flag flip changes behavior without deploy (test flag wired to a visible dev feature); playbook
  committed.
- Review: none.
