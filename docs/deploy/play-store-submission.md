# Google Play submission answer sheet (CyberOS)

Everything the Play Console asks for, answered, so the console work is copy-paste rather than
decision-making. Package `os.cyberskill.world`. Track: **internal** first.

The CI side is already done: `ANDROID_RELEASE=true`, the keystore secrets are set, and the android
job builds and signs the `.aab` on every `v*` tag. What is left is Play's own paperwork, plus one
manual upload that the API is not permitted to do for you (see the bottom of this file).

## Before you start: the two real blockers

1. **The privacy and deletion pages must be deployed.** They are at
   `landing-page/app/[lang]/cyberos/{privacy,delete-account}`. Play fetches both URLs and will
   reject a 404. Ship the landing page first, then confirm:
   - <https://cyberskill.world/en/cyberos/privacy>
   - <https://cyberskill.world/en/cyberos/delete-account>

2. **CyberOS has user-generated content**, because it has chat. You must declare user-to-user
   communication truthfully in the content rating questionnaire, and doing so puts the app under
   Play's UGC policy: an in-app way to **report** objectionable content, an in-app way to **block**
   another user, and a published content policy. CyberOS has none of these today. The workspace is
   invite-only and org-scoped, which makes the moderation load light, but the mechanisms still have
   to exist. Raise this as an FR against the chat module before you submit. Declaring UGC without
   the controls gets you rejected; not declaring it gets you pulled later, which is worse.

## Set up your app

### Set privacy policy

```
https://cyberskill.world/en/cyberos/privacy
```

### Sign in details (app access)

CyberOS is entirely behind Google SSO. A reviewer who cannot sign in sees a login wall and rejects
the app - this is the single most common cause of rejection for a workspace tool.

Choose **All or some functionality is restricted**, then add one instruction set:

- Name: `Google sign-in (required)`
- Username: a real Google account you control, provisioned into a demo workspace. Create
  `play-review@cyberskill.world` for this and nothing else.
- Password: that account's password.
- Any other instructions:
  > CyberOS is a private workspace tool. Tap "Sign in with Google" and use the credentials above.
  > The account is already a member of a demo workspace with sample channels and messages. All app
  > functionality is reachable after sign-in. There is no public sign-up: access is granted by a
  > workspace administrator.

Keep that account alive and its workspace populated. If it stops working, every future update is
rejected until it does.

### Ads

**No**, this app contains no ads.

### Content rating

Category: **Social networking / communication**. Answer honestly:

- Users can interact or exchange content with other users: **Yes** (chat).
- Users can share their content with other users: **Yes** (messages, files).
- App allows users to share their location with other users: **No**.
- Violence, sexuality, profanity, drugs, gambling, in-app purchases: **No** to all.
- Digital purchases: **No**.

Expect a rating around Teen / PEGI 12 driven purely by the user-communication answer. That is
normal for a chat app and is not a problem.

### Target audience and content

- Target age group: **18 and over**, only.
- Do not tick any bracket under 18. Doing so puts CyberOS into the Families policy programme, which
  raises the bar sharply for a tool no child will ever open.
- Appeals to children: **No**.

### Data safety

Answers below match the published privacy policy. If you change one, change both.

Overall:

- Does your app collect or share any of the required user data types? **Yes**.
- Is all of the user data collected by your app encrypted in transit? **Yes** (TLS everywhere).
- Do you provide a way for users to request that their data is deleted? **Yes** -
  `https://cyberskill.world/en/cyberos/delete-account`.

Data types collected (all: collected = yes, shared = no, processed ephemerally = no, required = yes,
purpose = App functionality; add Account management where noted):

| Category | Type | Why |
| --- | --- | --- |
| Personal info | Name | From your Google account at sign-in. App functionality, Account management. |
| Personal info | Email address | From your Google account at sign-in. App functionality, Account management. |
| Personal info | User IDs | The Google account identifier and the CyberOS subject id. App functionality, Account management. |
| Messages | Other in-app messages | Chat messages between workspace members. App functionality. |
| Photos and videos | Photos | Only images the user chooses to attach to a message. App functionality. |
| Files and docs | Files and docs | Only files the user chooses to attach to a message. App functionality. |
| Device or other IDs | Device or other IDs | The push notification token, so notifications can be delivered. App functionality. |

Declare **nothing** under Location, Contacts, Calendar, Financial info, Health, Web browsing, or
App activity - CyberOS collects none of it. Do not tick "Crash logs" or "Diagnostics" unless and
until you actually ship a crash SDK; server-side logs are not client-collected data and are not
declarable here.

Data is not shared with any third party for Play's purposes: Google (identity), Vultr (hosting) and
Supabase (database) are service providers processing on our behalf, which Play explicitly excludes
from "sharing".

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
- Phone screenshots: at least 2, up to 8. 16:9 or 9:16, each side between 320 and 3840 px. Take
  them from the real app - a login screen, the channel list, a channel with messages, a DM.

Do not put a device frame or heavy marketing copy on the screenshots; Play rejects listings whose
screenshots misrepresent the app.

## The one manual upload

The Play Developer API **cannot create the first release** for a package. Google requires the first
bundle to be uploaded by hand through the console so the app passes one review before API access is
permitted. Skip this and `upload-google-play` fails with:

```
Only releases with status draft may be created on draft app.
```

So, once:

1. Tag a release. CI builds and signs the `.aab` in the `android` job.
2. `gh run download --name cyberos-android-aab`
3. Play Console → Testing → Internal testing → Create new release. Accept **Play App Signing** when
   offered: Google holds the app signing key, your keystore stays the upload key. Upload the
   `.aab`, add release notes, **Review release**, **Start rollout to internal testing**.
4. Add yourself to the internal tester list first, or there is no one to roll out to.

After that, every tag publishes on its own.

## Turning on automated publishing

Play Console → **Setup → API access** → link a Google Cloud project → create a service account.
In Google Cloud: IAM & Admin → Service Accounts → create `play-publisher` (no project roles needed)
→ Keys → Add key → JSON. Back in Play Console → **Users and permissions** → the service account →
**App permissions → CyberOS** → grant only **Release apps to testing tracks** → Invite.

Then:

```sh
gh secret set PLAY_SERVICE_ACCOUNT_JSON < ~/.cyberos-signing/play-publisher.json
gh variable set PLAY_PUBLISH --body true
```

Keep the JSON key outside the repo, next to the Apple `.p12` and the Tauri updater key.

Play permissions take a few minutes to reach the API. If the first automated run 401s, re-run it
before you debug anything.

## versionCode

Derived from `VERSION` by `scripts/stamp-release-version.mjs`: `major*10000 + minor*100 + patch`.
So 1.2.0 is 10200 and 1.2.1 is 10201. Play rejects any upload with a versionCode it has already
seen, so **never re-tag a version you have already pushed to Play**. Bump and move on.
