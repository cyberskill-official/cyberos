# Signing and mobile release runbook

How to turn on code signing (macOS desktop) and the mobile builds (Android now, iOS when its project exists). Every signing input is a GitHub secret or repo variable - nothing private is ever committed. This is the companion to `RELEASE.md`; do it once, then every `v*` tag produces signed artifacts.

Set secrets at: GitHub repo -> Settings -> Secrets and variables -> Actions -> Secrets. Set variables on the Variables tab (same page). Never paste a key into a file, a commit, or a chat - only into the GitHub secret box.

## Golden rule for signing keys

A signing key or certificate is the identity of CyberSkill. Keep the originals in a password manager or an offline vault. In this repo they are `.gitignore`d (`*.keystore`, `*.p12`, `*.p8`, ...) so they cannot be committed by accident. If one leaks, revoke and re-issue it - a leaked Android upload key means rotating it with Google, a leaked Apple cert means revoking it in the developer portal.

## A. macOS desktop signing (Apple Developer account)

Removes the "unidentified developer" Gatekeeper warning on the `.dmg`. Optional - the unsigned build works (right-click -> Open the first time). Turn it on:

1. In Keychain Access on your Mac, create the certificate: Xcode -> Settings -> Accounts -> your team -> Manage Certificates -> + -> "Developer ID Application". (Or generate a CSR and download the cert from developer.apple.com -> Certificates.)
2. Export it as a .p12: in Keychain Access, right-click the "Developer ID Application: CyberSkill ..." cert -> Export -> .p12, set a strong export password.
3. Base64 the .p12 and copy it (does not print the key to the screen):

       base64 -i ~/path/to/DeveloperID.p12 | pbcopy

4. Add these secrets:
- `APPLE_CERTIFICATE` - paste (the base64 from step 3).
- `APPLE_CERTIFICATE_PASSWORD` - the export password from step 2.
- `APPLE_SIGNING_IDENTITY` - the exact cert name, e.g. `Developer ID Application: CyberSkill Software ... (TEAMID)`.
- `APPLE_ID` - your Apple ID email.
- `APPLE_PASSWORD` - an app-specific password (appleid.apple.com -> Sign-In and Security -> App-Specific Passwords), NOT your login password.
- `APPLE_TEAM_ID` - the 10-character team id (developer.apple.com -> Membership).
5. Add the repo variable `MACOS_SIGN` = `true`. This is the switch: until it is `true`, `release.yml` forces the Apple env empty and ships unsigned, so a stray or wrong secret can never break the build (that is exactly what broke the first v1.0.0 tag).
6. Re-tag (see section D). The macOS job now signs and notarizes.

## B. Android release (Google Play account)

The `android/` project is already committed and your upload keystore is generated (kept at `~/.cyberos-signing/cyberos-release.keystore`, alias `cyberos`, key password = keystore password).

1. Base64 the keystore and copy it:

       base64 -i ~/.cyberos-signing/cyberos-release.keystore | pbcopy

2. Add these secrets:
- `ANDROID_KEYSTORE_BASE64` - paste (the base64 from step 1).
- `ANDROID_KEYSTORE_PASSWORD` - the keystore password you typed into keytool.
- `ANDROID_KEY_ALIAS` - `cyberos`.
- `ANDROID_KEY_PASSWORD` - the key password (same as the keystore password, since you pressed RETURN at the "key password" prompt).
3. Add the repo variable `ANDROID_RELEASE` = `true`. This turns on ONLY the android job (iOS has its own `IOS_RELEASE` gate, so Android never drags in the not-yet-existing iOS project).
4. Re-tag (section D). The android job assembles a signed `.aab` and uploads it as a release artifact.
5. First upload to Play is manual: Play Console -> your app -> Production (or Internal testing) -> Create release -> upload the `.aab` from the workflow's artifacts. Enroll in Play App Signing when prompted (Google holds the distribution key; your keystore is the upload key). Later automated Play uploads can use a service account JSON (a follow-up).

## C. iOS / TestFlight (Apple Developer account) - one-time project first

The `ios/` project does NOT exist yet, and the workflow's iOS step is a stub. Do this only when you want TestFlight; it does not block the desktop or Android release.

1. One-time, locally, create the native project and commit it:

       cd apps/web
       npm i -D @capacitor/ios
       npx cap add ios
       npx cap sync ios
       git add ios && git commit -m "chore: add Capacitor iOS shell"

2. Add the fastlane lane (`apps/web/ios/App/fastlane/Fastfile`) with a `beta` lane that archives and uploads to TestFlight, and replace the stub `echo` in `release.yml`'s iOS step with `fastlane beta`. (This is a real task - open it as a task, not a rushed edit.)
3. Add these secrets (App Store Connect -> Users and Access -> Integrations -> App Store Connect API -> generate a key):
- `APP_STORE_CONNECT_KEY_ID`
- `APP_STORE_CONNECT_ISSUER_ID`
- `APP_STORE_CONNECT_API_KEY` - the contents of the downloaded `.p8`.
4. Set the repo variable `IOS_RELEASE` = `true`. The next tag runs the iOS job (independent of the Android gate).

## D. Re-tag to produce the signed artifacts

Signing and mobile jobs only run on a `v*` tag. After setting the secrets and variables above, move the tag so `release.yml` re-runs against the configured repo:

    git tag -d v1.0.0                       # delete the local tag
    git push origin :refs/tags/v1.0.0       # delete the remote tag
    git pull                                # make sure you are on the latest main
    git tag v1.0.0
    git push origin v1.0.0                  # re-runs release.yml

Then on GitHub -> Releases, delete the old failed draft and publish the new draft the workflow creates.

## What v1.0.0 can ship today

- Desktop (macOS + Windows + Linux): yes, right now - unsigned by default, or signed once section A is done.
- Android `.aab`: yes, once section B's secrets + `ANDROID_RELEASE=true` are set.
- iOS TestFlight: not until section C's one-time project + fastlane lane land - defer it to a follow-up task; it does not hold up the rest.
