# Google Play submission answer sheet (CyberOS)

Everything the Play Console asks for, answered, so the console work is copy-paste rather than decision-making. Package `os.cyberskill.world`. Track: **internal** first.

The CI side is already done: `ANDROID_RELEASE=true`, the keystore secrets are set, and the android job builds and signs the `.aab` on every `v*` tag. What is left is Play's own paperwork, plus one manual upload that the API is not permitted to do for you (see the bottom of this file).

## Before you start: the two former blockers — both RESOLVED (verified 2026-07-20)

Both items below blocked submission when this runbook was written. Both are now closed. Re-verify
rather than trust this note if significant time has passed.

1. **Privacy and deletion pages — LIVE.** Both return HTTP 200 as of 2026-07-20:
- <https://cyberskill.world/en/cyberos/privacy>
- <https://cyberskill.world/en/cyberos/delete-account>

2. **UGC controls — SHIPPED.** CyberOS has user-generated content because it has chat, so declaring
user-to-user communication truthfully in the content rating questionnaire puts the app under Play's
UGC policy: an in-app **report**, an in-app **block**, and a published content policy. All three now
exist — this runbook previously said "CyberOS has none of these today", which is out of date:

| Requirement | Implementation | Verified |
| --- | --- | --- |
| Report objectionable content | `apps/web/src/components/ReportDialog.tsx`, wired into `pages/Chat.tsx` and `components/ChannelSettings.tsx` | 2026-07-20 |
| Block another user | Block/unblock in `components/ChannelSettings.tsx` + `components/BlockedMessage.tsx` (TASK-CHAT-268) | 2026-07-20 |
| Published content policy | <https://cyberskill.world/en/cyberos/content-policy> (en + vi, both HTTP 200), linked from the report dialog and `pages/Moderation.tsx` | 2026-07-20 |

**Open question, not a blocker: the age declaration disagrees across stores.** Play declares
*18 and over* (App content → Target audience and content, confirmed in console). App Store Connect
declares *4+* across 172 countries. Same app, contradictory answers. Reconcile before either store
review looks closely — a chat app carrying UGC rated 4+ is the kind of thing Apple queries under
guideline 1.2.

## Set up your app

### Set privacy policy

```
https://cyberskill.world/en/cyberos/privacy
```

### Sign in details (app access)

CyberOS is entirely behind a sign-in wall - Google SSO for members, a password form for everyone else. A
reviewer who cannot sign in sees that wall and rejects the app, and this is the single most common cause of
rejection for a workspace tool.

Choose **All or some functionality is restricted**, then add one instruction set:

- Name: `Password sign-in (required)`
- Username: `demo@cyberskill.world` - a password account provisioned into the demo workspace, used for
  store review and nothing else.
- Password: that account's password. Keep it in the password manager; it is deliberately not written
  down in this repo.
- Any other instructions:
  > CyberOS is a private workspace tool. On the sign-in screen, tap "Admin sign-in" - the small link
  > below the Google button - then enter the username and password above.
  > Do NOT tap "Sign in with Google": that path is for workspace members who have a Google account, and
  > it will not accept these credentials.
  > The account is a member of a demo workspace with sample channels and messages. All app functionality
  > is reachable after sign-in. There is no public sign-up: access is granted by a workspace administrator.

A password account rather than a Google one is a deliberate choice. It takes the whole OIDC path out of
the review process: no consent screen, no account chooser, and no dependency on which Google account the
reviewer's device happens to be signed into. The Google path is a member convenience, not the way a
reviewer should be asked to get in.

The "Admin sign-in" label is misleading here - that link is the generic password form and any account
with a password can use it, not just admins. Reviewers have to be told to tap it, or they will only see
the Google button and conclude the credentials do not work.

Keep that account alive and its workspace populated. If it stops working, every future update is rejected until it does.

### Ads

**No**, this app contains no ads.

### Content rating

Category: **Social networking / communication**. Answer honestly:

- Users can interact or exchange content with other users: **Yes** (chat).
- Users can share their content with other users: **Yes** (messages, files).
- App allows users to share their location with other users: **No**.
- Violence, sexuality, profanity, drugs, gambling, in-app purchases: **No** to all.
- Digital purchases: **No**.

Expect a rating around Teen / PEGI 12 driven purely by the user-communication answer. That is normal for a chat app and is not a problem.

### Target audience and content

- Target age group: **18 and over**, only.
- Do not tick any bracket under 18. Doing so puts CyberOS into the Families policy programme, which raises the bar sharply for a tool no child will ever open.
- Appeals to children: **No**.

### Data safety

Answers below match the published privacy policy. If you change one, change both.

Overall:

- Does your app collect or share any of the required user data types? **Yes**.
- Is all of the user data collected by your app encrypted in transit? **Yes** (TLS everywhere).
- Do you provide a way for users to request that their data is deleted? **Yes** - `https://cyberskill.world/en/cyberos/delete-account`.

Data types collected (all: collected = yes, shared = no, processed ephemerally = no, required = yes, purpose = App functionality; add Account management where noted):

| Category | Type | Why |
| --- | --- | --- |
| Personal info | Name | From your Google account at sign-in. App functionality, Account management. |
| Personal info | Email address | From your Google account at sign-in. App functionality, Account management. |
| Personal info | User IDs | The Google account identifier and the CyberOS subject id. App functionality, Account management. |
| Messages | Other in-app messages | Chat messages between workspace members. App functionality. |
| Photos and videos | Photos | Only images the user chooses to attach to a message. App functionality. |
| Files and docs | Files and docs | Only files the user chooses to attach to a message. App functionality. |
| Device or other IDs | Device or other IDs | The push notification token, so notifications can be delivered. App functionality. |

Declare **nothing** under Location, Contacts, Calendar, Financial info, Health, Web browsing, or App activity - CyberOS collects none of it. Do not tick "Crash logs" or "Diagnostics" unless and until you actually ship a crash SDK; server-side logs are not client-collected data and are not declarable here.

Data is not shared with any third party for Play's purposes: Google (identity), Vultr (hosting) and Supabase (database) are service providers processing on our behalf, which Play explicitly excludes from "sharing".

### The AI feature, and why it is still not "sharing"

This needs its own paragraph, because it is the one place the app hands message text to a party that is **not** our service provider, and getting it wrong is an under-declaration.

CyberOS ships AI features (channel summary, action extraction, reply suggestions) in the same `apps/web` bundle that Capacitor wraps as the Android app, so they are in scope for this declaration. Today an employee supplies **their own API key** for an external model provider. That means the provider is *the user's*, not ours - so the **service-provider exemption does not apply to it**. Do not rely on it.

What does apply is Play's **user-initiated transfer** exemption: data sent to a third party because of a specific action the user took, where the user reasonably expects the data to be sent. Every AI call in CyberOS is exactly that:

- `AiPanel` is mounted only when the user opens it (`aiOpen &&` in `pages/Chat.tsx`); it is not rendered, and sends nothing, until then.
- `suggestReplies()` is bound to the composer's sparkle button (`onSuggestReplies`).
- There is **no** background, on-open, or scheduled AI call anywhere in the client.

So: **shared = No** is correct, and the published privacy policy already discloses the transfer ("the text you send to it is passed to the model provider that serves that feature").

**This is a live constraint on the codebase, not a one-off answer.** The exemption holds *only* while every AI call is user-initiated. The moment anything auto-summarises a channel on open, pre-fetches suggestions in the background, or runs AI on a schedule, message text leaves the device without a user action - and "Other in-app messages" must be re-declared with `shared = Yes`, which changes the store listing. If you add a background AI path, change this form in the same PR.

(A future self-hosted model removes the question entirely: nothing leaves our infrastructure.)

### Government apps / Financial features / Health

**No** to all three.

### App category and contact details

- App category: **Business**
- Tags: productivity, team communication
- Email: `info@cyberskill.world`
- Website: `https://cyberskill.world`
- Phone: optional, skip it

### Store listing

App name (30 chars max):

```
CyberOS
```

Short description (80 chars max):

```
Your team's private workspace. Chat, files, and work in one place.
```

Full description (4000 chars max):

```
CyberOS is a private workspace for your organisation. Team chat, file sharing, and the tools your
work actually runs on, in one place, on your phone.

Sign in with your Google work account and you are in the same workspace you use on the desktop and
on the web. Channels, direct messages, attachments, mentions, and read state stay in sync across
every device you are signed in on.

What you get

- Team chat. Channels for the work, direct messages for everything else. Threads, reactions, and
  mentions so nothing gets lost.
- Files where the conversation is. Attach an image or a document to a message and it stays with the
  context that explains it.
- Notifications that respect you. Per-channel settings, so the noisy channel stays quiet and the
  one that matters does not.
- One account, every device. Android, desktop, and the web console at os.cyberskill.world.

Built for organisations

CyberOS is invite-only. There is no public sign-up and no public content - a workspace
administrator grants access, and can revoke it. Your organisation owns its content. We do not sell
your data, we show no advertising, and we do not use your messages to train AI models.

You need an existing CyberOS workspace to use this app. If your organisation does not have one yet,
get in touch at cyberskill.world.

CyberSkill Software Solutions Consultancy and Development JSC
Ho Chi Minh City, Vietnam
```

Graphics you still have to produce:

- App icon: 512 x 512 PNG, 32-bit, no transparency. Use the CyberOS mark from the design system.
- Feature graphic: 1024 x 500 PNG or JPEG. No transparency.
- Phone screenshots: at least 2, up to 8. 16:9 or 9:16, each side between 320 and 3840 px. Take them from the real app - a login screen, the channel list, a channel with messages, a DM.

Do not put a device frame or heavy marketing copy on the screenshots; Play rejects listings whose screenshots misrepresent the app.

## The one manual upload

The Play Developer API **cannot create the first release** for a package. Google requires the first bundle to be uploaded by hand through the console so the app passes one review before API access is permitted. Skip this and `upload-google-play` fails with:

```
Only releases with status draft may be created on draft app.
```

So, once:

1. Tag a release. CI builds and signs the `.aab` in the `android` job.
2. `gh run download --name cyberos-android-aab`
3. Play Console → Testing → Internal testing → Create new release. Accept **Play App Signing** when offered: Google holds the app signing key, your keystore stays the upload key. Upload the `.aab`, add release notes, **Review release**, **Start rollout to internal testing**.
4. Add yourself to the internal tester list first, or there is no one to roll out to.

After that, every tag publishes on its own.

## Turning on automated publishing

Play Console → **Setup → API access** → link a Google Cloud project → create a service account. In Google Cloud: IAM & Admin → Service Accounts → create `play-publisher` (no project roles needed) → Keys → Add key → JSON. Back in Play Console → **Users and permissions** → the service account → **App permissions → CyberOS** → grant only **Release apps to testing tracks** → Invite.

Then:

```sh
gh secret set PLAY_SERVICE_ACCOUNT_JSON < ~/.cyberos-signing/play-publisher.json
gh variable set PLAY_PUBLISH --body true
```

Keep the JSON key outside the repo, next to the Apple `.p12` and the Tauri updater key.

Play permissions take a few minutes to reach the API. If the first automated run 401s, re-run it before you debug anything.

## versionCode

Read from the root `BUILD_NUMBER` file by `scripts/stamp-release-version.mjs`. It is a plain monotonic counter, incremented on every version bump, and it is deliberately NOT derived from `VERSION`.

It used to be derived (`major*10000 + minor*100 + patch`, so 1.2.0 became 10200). That formula quietly couples a marketing decision to a number that can never go backwards. Play remembers every versionCode it has ever accepted - 10700 among them - and refuses anything at or below the highest it has seen, with no appeal and no reset. So when `VERSION` was rolled back to 0.1.0 for the pre-1.0 run-up, the derived code would have become 100, and every Android upload from then on would have been rejected permanently. The stamper now refuses outright to stamp a `BUILD_NUMBER` at or below 10700.

Practical consequences:

- `BUILD_NUMBER` only ever increases. Never hand-edit it downwards, and never "reset" it to match a version. If it is ever lost, recover it from git history rather than guessing.
- Play still rejects any versionCode it has already seen, so **never re-tag a version you have already pushed to Play**. Bump and move on.
