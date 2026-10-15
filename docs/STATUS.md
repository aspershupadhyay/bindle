# Where Bindle stands

A plain checklist of what is built, what is being built now, and what is left.
Updated at the end of every week of [the roadmap](ROADMAP.md). Last updated: desktop polish after weeks 18 to 20 (15 October 2026).

## Built

### Core (Rust)
- [x] `bindle-core`: promise states and allowed changes, "close the loop" (a kept promise says who owes it, a firm one has a day), running late, waiting on others, the week and day capacity (5), and the reminder rules (at most 3 a day, none 10 PM to 7 AM, never late)
- [x] `bindle-parse`: English and Hinglish days, times, money, people, who owes it and soft promises, with a test table. Words that could mean two things are flagged for Sort. Festivals, months and places are never read as people
- [x] `bindle-store`: SQLite (WAL, FTS5) with migrations 0001 to 0006, history and Undo for every change, Home, People, search, settings, Wrap up, the reminder log, AI rules, export and safety copies
- [x] `bindle-sync`: hybrid logical clocks and field merges, used by Bindle Cloud
- [x] `bindle-mcp`: the AI connection over MCP. It reads Home, lists and searches promises, reads people, and suggests promises. It never changes anything

### Desktop app
- [x] Tauri shell with the v2 sidebar, the Life and Work switch, and a pixel scene for each Work screen still to come
- [x] Jot down from anywhere (Alt+Shift+J), with live chips while typing
- [x] Sort, one promise at a time, with Keep it, Later and Let go, and AI suggestions marked "Suggested by Claude"
- [x] Home: greeting, top 3, Running late, Waiting on others with Nudge, Coming up, the week strip, and the Wrap up card
- [x] Promises by state, with search (Ctrl+K or "/")
- [x] People: both sides, kept lately, Send recap, pictures, pronouns, rename, and Keep from AI
- [x] Wrap up in about a minute
- [x] First run in four skippable steps
- [x] Settings: name and picture, Wrap up, reminders, theme, AI connection, Your data (export, safety copies), shortcuts, and about
- [x] Reminders through the system's notifications
- [x] Undo bar and Ctrl+Z for every change
- [x] Light and dark themes, with the logo's "on dark" version
- [x] Pixel art: the logo, Home hills, Work scenes, icons for Sort, Promises, People and Wrap up, and nine people's pictures
- [x] Accessibility (week 8): skip link, a main landmark on every screen, live announcements, visible focus, colour contrast checked by 48 tests (primary buttons were darkened to pass), and an audit script that finds unnamed buttons, missing alt text and unlabelled fields (none left)
- [x] Linux (week 8): the blank window on some graphics drivers is avoided, a shortcut that cannot be claimed never stops the app, and the .deb package builds, installs with a menu entry, and runs the AI connection (.rpm and AppImage are configured)

### Work mode (weeks 9 and 10)
- [x] Mode switch: Bindle opens in the mode you last chose; Jot down saves into it and says "Work" in its window; work promises carry a Work chip
- [x] `bindle-sources`: GitHub, read only. Reviews you owe, your pull requests and issues assigned to you, from three searches; plain errors for a refused token, missing permissions, rate limits and no internet; tested against a local stand-in server
- [x] The token lives in the system keychain (Keychain, Credential Manager or Secret Service), never in the database, exports or safety copies. Bindle looks again every 15 minutes, and on "Look now"
- [x] Code screen (v2 design): a summary that names who is waiting, tiles, three lists, "Open" on GitHub, and Disconnect
- [x] Make it a promise: a work promise in Sort, titled "Review latch #42: …" with the author as the person; once GitHub stops listing it, Code asks "Finished on GitHub?" and offers Mark done (nothing is closed by itself)
- [x] Migration 0007 (`work_item`), and the accessibility audit now covers Code

### Work mode (weeks 11 and 12)
- [x] Gmail, read only: IMAP with an app password in the system keychain. Inbox and sent mail are opened read-only and fetched with `BODY.PEEK`, so nothing is marked as read; Bindle never sends email. Tested against a local stand-in IMAP server that also checks no mailbox-changing command is ever sent
- [x] Triage: threads waiting on your reply (To or Cc you), your questions with no answer after a day, and promises you wrote, read with Jot down's parser from the day you wrote them. Newsletters, receipts, alerts and no-reply senders never count. Only the last two weeks are read
- [x] Inbox screen (v2 design): a summary, a list in three groups, a reading pane with Make it a promise, Not needed (with Undo; Gmail is not touched) and Open in Gmail, and "Bindle noticed a promise" with Add to promises. Answered threads move to "Answered already?" when their promise is still open
- [x] Board (v2 design): Ideas, Next, Making, Waiting and Shipped this week, filtered by Code, Inbox or Other. Start, Not now, Done and Plan it are buttons, so it works from the keyboard. The Making limit (2 by default, in Settings → Work) is a gentle note, never a block
- [x] Migration 0008 (`mail_item`, the Board's Making lane); the accessibility audit covers Inbox and Board

### Work mode (weeks 13 and 14)
- [x] Wait history: each look records how long people waited. Email replies run from the first email someone wrote you after your last word to your next email; reviews run from opening a pull request to your first review of it (at most 20 new pull requests per look). Samples are never changed, so history grows by itself (migration 0009)
- [x] Desk (v2 design): what needs you from GitHub and Gmail, oldest first, with Make it a promise and Open; tiles for needs you now, email reply time and review wait (medians, compared with your own last four weeks), and shipped this week; how each source is doing; this week's work as bars (done and planned); "Who waits longer" with eight weeks of medians and a table view
- [x] Desk checks: 7 tests in `bindle-store` (medians, counting each answer once, comparing with the four weeks before, shipped by week and source, the week's work, and merging what needs you). Chart colours checked with the dataviz validator in both themes

### Bindle Cloud and Ours (weeks 15 to 17)
- [x] `bindle-cloud`: one random key per space. From it come the space's id on the relay, its write token and its encryption key (HKDF-SHA256); every batch is sealed with XChaCha20-Poly1305 before it leaves the device. The key lives in the system keychain
- [x] `bindle-relay`: a small server (axum, SQLite) that keeps sealed batches in order and hands them back. It knows no names, emails or passwords; the first device to write a space sets its token, and only a hash of it is kept. Invites wait 7 days and are handed out once. A Dockerfile is included
- [x] Sync: each shared field is compared with the last value both sides agreed on; changes go out in batches, and changes coming in win only when their clock is later (last writer wins per field, with hybrid logical clocks). Works offline, and catches up every 30 seconds or on "Sync now" (migration 0010)
- [x] Invites: a link that carries its secret after the `#`, so no server sees it. The person joining pastes it in Settings, Bindle Cloud
- [x] Recovery kit: the recovery key written as 14 groups of 4 letters, with a check so a typo is caught. It unlocks a sealed list of your spaces on the relay, so a new device can bring them back
- [x] Ours screen (v2 design): who carries the space, shared promises with who added them, Jot down to the space, share one of your promises, kept lately and kept together this month (no scores), the invite card, and Leave. Spaces are listed in the sidebar
- [x] Checks: 8 sync tests in `bindle-store`, crypto and code tests, relay tests, and an end-to-end test where two databases share a space through a real relay (`crates/bindle-cloud/tests/ours.rs`). The accessibility audit covers the recovery kit and Ours

### Desktop polish (after weeks 18–20)
- [x] Desktop checks drive the real app on Linux (tauri-driver and WebKitWebDriver on a virtual display): first run, Sort, People, every Life and Work screen and Settings with no error text or thrown errors, Bindle Cloud on two devices, and an Ours space with an invite and a shared promise. In CI as "Desktop app (Linux)", with screenshots kept
- [x] Linux without a Secret Service: tokens and keys fall back to `secrets.json` in the app's folder, readable only by the person's account. The keychain is always tried first. Checked in the real app: Bindle Cloud starts where it used to fail
- [x] GitHub searches read page after page, up to GitHub's 1,000, instead of stopping at 100
- [x] People: someone with nothing open can be archived, and brought back from Archived or by naming them in a new promise (their history stays)
- [x] Fixed: whoever made a shared space did not see themselves under "Who carries" until someone joined
- [x] Jot down reads "on the 1st", "by the 20th" and "15th tak" as the next such day of the month (`crates/bindle-parse/tests/month_days.rs`). The global Alt+Shift+J shortcut and the Jot window were checked by hand in the real app on X11
- [x] Fixed: Home's Waiting list and Nudge lowercased names ("Rohan · rohan will pay 500"); the Desk said Gmail with no mailbox connected

### Project
- [x] Handbook mirror, roadmap, changelog, contributing guide, DCO and licence headers
- [x] CI on Windows, macOS and Linux: format, lint, tests, views and licence headers. It runs on pushes to `main`, to working branches, and on pull requests. First full run passed on all three on 14 October 2026
- [x] Release workflow (week 8): a version tag builds installers for all three systems. They are signed once the signing secrets are added (see [RELEASING.md](RELEASING.md))

### Release and website
- [x] Download page in Bindle's pixel style (`site/`), published by the Website workflow to https://aspershupadhyay.github.io/bindle/ (Pages switched on)
- [x] Release workflow: start it from the Actions tab with a version; builds Windows (.exe, .msi), macOS (Apple silicon and Intel .dmg) and Linux (.AppImage, .deb, .rpm), unsigned for now
- [x] First release v0.0.1 published on 14 October 2026 (the Release run passed on all four builds)
- [x] Updates inside the app: Bindle looks for a new version on GitHub after opening and every six hours, offers "Install and restart", and checks the signature first. Releases take their version from the tag, so nothing is bumped by hand
- [x] GitHub's language bar counts design mockups as documentation

## Picking this up in a new session

Read this first if you are continuing the work (another person or a new Claude session).

- **Branch:** `claude/awesome-mccarthy-vtojde`, started from `main` after [#5](https://github.com/aspershupadhyay/bindle/pull/5) (weeks 9–20) was merged. It holds the desktop polish below; no pull request is open for it yet. Only the owner merges.
- **Focus now:** the desktop app. The owner asked to leave Android until the desktop works well.
- **Checks before every push** (same as CI): `(cd app && npm run typecheck && npm test && npx vite build)`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`. The views must be built before the Rust workspace compiles.
- **Desktop checks (the real app, not the preview):** build with `(cd app && npx vite build) && cargo build -p bindle-app --features tauri/custom-protocol && cargo build -p bindle-relay`, install `cargo install tauri-driver --locked` and `apt-get install webkit2gtk-driver xvfb`, then `dbus-run-session -- xvfb-run -a app/scripts/desktop/run.sh`. Two copies of the app share a space through a local relay; screenshots go to `$BINDLE_SHOTS`. CI runs the same in the "Desktop app (Linux)" job.
- **Accessibility audit and screenshots:** start `npx vite --port 1420` in `app/`, then `PLAYWRIGHT_DIR=$(npm root -g) node app/scripts/a11y-audit.mjs`. In the cloud container, Chromium is at `/opt/pw-browsers/chromium` (pass it as `executablePath`). Screenshots for each week go in `docs/screens/`.
- **Where things live:** all SQL is in `crates/bindle-store` (one migration per change, numbered; the latest is 0010). Sources (GitHub, Gmail) are in `crates/bindle-sources`; Bindle Cloud is in `crates/bindle-cloud` and `crates/bindle-relay`. Tauri commands are in `app/src-tauri/src/commands/`, one file per area, and every command needs a stand-in in `app/src/lib/preview*.ts` so the views run in a browser.
- **House rules:** sources are read only; Bindle never sends email or posts anything; suggestions wait in Sort; no scores or shaming; tokens and keys live in the system keychain (`app/src-tauri/src/secrets.rs`), never in the database; errors in plain words; tests for everything; every source file starts with the SPDX licence header.
- **Cannot be done from a cloud session:** reaching GitHub search, Gmail or other mailboxes with real accounts, hosting the relay, building for a phone (no Android SDK or Xcode), signing. These are listed as gates below.
- **The owner has added** the updater secrets (`TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, `TAURI_UPDATER_PUBKEY`). The v0.0.2 release waits until #5 is merged.

## Being worked on now

### Weeks 18–20: Creator, Outlook and IMAP, phone build (gate: Ours works on a phone)
- [x] Any IMAP mailbox, not only Gmail (done): Inbox's connect card has Gmail, iCloud Mail, Fastmail, Yahoo Mail, Zoho Mail, Outlook and "Another mailbox" (server and port). `crates/bindle-sources/src/imap.rs` (was `gmail.rs`) holds the presets; the sent folder is found by its \\Sent mark or its usual name, and a mailbox without one is read without it. Labels say the service's name. Still read only. Internally the source is still called "gmail" (setting key, `mail_item.source`, keychain name, command names) so connected accounts stay connected; renaming needs a migration
- [ ] Outlook "Sign in with Microsoft": Microsoft turned off app passwords for most Outlook.com accounts, so Outlook needs OAuth (XOAUTH2 over IMAP). That needs an app registered by the owner in Microsoft Entra (Azure), then a device-code sign-in in Bindle and `AUTHENTICATE XOAUTH2` in `imap.rs`. Not started
- [x] Creator, YouTube, read only (done): migration 0011 (`creator_snapshot`, `creator_post`, `creator_comment`) in `crates/bindle-store/src/creator.rs`; the client in `crates/bindle-sources/src/youtube.rs` (a YouTube Data API key the person pastes, kept in the keychain as `youtube-api-key`; channel by @handle or id, latest 50 uploads, Shorts by length, comments from the last two weeks the channel has not answered); commands in `app/src-tauri/src/commands/creator.rs` (a look every hour); the screen `app/src/screens/Creator.tsx` (v2 design: tiles, 12-week growth chart with a table view, comments waiting with Make it a promise / Not needed / Open, top posts of the last 30 days, long videos and Shorts mix) and a sidebar badge. Audit and screenshots (`docs/screens/week-19-*.png`) done
- [ ] YouTube Analytics (watch time, click-through, returning viewers, "when people are around", "who sees your posts" in the design): needs "Sign in with Google" with a Google Cloud OAuth app the owner registers (and Google's verification for the Analytics scope). Not started
- [ ] Instagram: needs a Meta app and its review, which only the owner can register. The Creator screen says "not in Bindle yet"
- [ ] Several channels at once (the design's account switcher): one channel for now
- [x] Phone-ready app (done, not yet run on a phone): `#[cfg_attr(mobile, tauri::mobile_entry_point)]` on `run()`; desktop-only parts (Jot window, global shortcut, updater, process restart, keychain) are left out on phones; `capabilities/desktop.json` holds the updater permissions for desktops only; `tauri.android.conf.json` and `tauri.ios.conf.json` keep one window (identifier `app.bindle.mobile`). On phones, secrets are kept in `secrets.json` in the app's private folder (`app/src-tauri/src/secrets.rs`, `phone` module)
- [x] Phone layout (done): under 600 px the sidebar becomes a bar across the top with the screens and spaces in sideways-scrolling rows, Jot down floats bottom right (it focuses Home's field, as there is no second window), and Undo sits above it. No sideways page scroll at 390 px. Screenshots `docs/screens/week-20-phone-*.png`
- [x] Android build: `.github/workflows/android.yml` runs `tauri android init` and builds a debug arm64 APK as an artifact. It runs on pushes that change `app/src-tauri/**`, `app/package*.json` or the workflow, on version tags, and by hand once it is on main. First run passed on 14 October 2026 in about 5 minutes: https://github.com/aspershupadhyay/bindle/actions/runs/38652153535 (the APK is the `bindle-android-debug` artifact). Next: install it on a phone and try Ours there
- [ ] Phone keychain: use the Android Keystore and iOS Keychain (a Tauri plugin such as a stronghold or keystore plugin) instead of the private-folder file
- [ ] iOS build: needs a Mac with Xcode and an Apple developer account (owner)
- [ ] Gate: Ours works on a phone. Needs a hosted relay and the Android build on a real phone

- [ ] Weeks 11–12 gate, "a week of email triage from Bindle": connect Gmail (Work → Inbox, with an app password), then triage from Bindle for a week. The cloud session that built this cannot reach Gmail, so this needs the real app
- [ ] Weeks 9–10 gate, "real review requests become promises": connect your own GitHub in the desktop app (Work → Code), make a promise from a real review request, and keep it. The cloud session that built this cannot reach GitHub's search, so this needs the real app

- [ ] Week 8 gate, "all acceptance checks pass": the automated checks pass. The hands-on checks in [ACCEPTANCE.md](ACCEPTANCE.md) still need a person on a real Windows and macOS machine
- [ ] Signed builds need the Apple and Windows certificates added as repository secrets. Only the owner can do this
- [ ] Turn on updates: the three updater secrets are added and #5 is merged, so v0.0.2 can be released (see "Updates inside the app" in [RELEASING.md](RELEASING.md)). People on v0.0.1 install v0.0.2 by hand once; every later version arrives inside the app

## Left to deal with

### Before Tier 1 is called done
- [ ] **Founder gate:** use Bindle on at least 25 of 30 days (handbook P10)
- [ ] Hands-on test on Windows and macOS: Jot down shortcut, notifications (macOS may ask for permission first), dark theme, and the AI connection from Claude Desktop
- [ ] Add the Apple and Windows signing certificates and cut a signed release
- [ ] Hands-on test of an update: install v0.0.2, release v0.0.3, and check each system offers and installs it
- [ ] A screen reader pass (NVDA on Windows, VoiceOver on macOS, Orca on Linux)
- [ ] Some Wayland desktops refuse global shortcuts. Bindle then works with the in-app shortcut only; a desktop-portal shortcut would fix this

### Known rough edges
- [ ] GitHub sign-in uses a personal access token. A "Sign in with GitHub" button needs a registered GitHub app (OAuth device flow), which only the owner can create
- [ ] Gmail uses an app password, which needs 2-Step Verification on the Google account. "Sign in with Google" needs a Google Cloud app that only the owner can register (and Google's review for Gmail access)
- [ ] Inbox reads the newest 200 emails of the last two weeks in each folder, and only the start of each email
- [ ] Review waits count from when a pull request was opened, not from when your review was requested; GitHub's search does not say when that was without one more request per pull request
- [ ] The Desk's numbers need a few weeks of history before the comparisons say much
- [ ] Bindle Cloud needs a relay that is running somewhere; none is hosted yet. Until one is, the gate (two people share Ours for two weeks) cannot start
- [ ] Sharing a promise shares its title, state, dates, amount, first step, note, and who added and kept it. Its person and history stay on each device
- [ ] Leaving a space keeps it on the relay for the others. There is no way yet to remove someone or to delete a space for everyone
- [ ] Putting back an old safety copy also brings back that day's settings (by design, but it can surprise)
- [ ] When an AI's suggestion mentions an amount, Bindle stores it as the promise's amount
- [ ] The handbook here is a summary. The full one is in Notion, so some checks were written from the summary
- [ ] `bindle-parse/tests/table.rs` is just over 400 lines; split it when it next grows

### After Tier 1 (the roadmap's next stages)
- [x] Weeks 15–17: Bindle Cloud: accounts, Mine and Ours, invites, recovery kit (sync and end-to-end encryption). Built; the gate waits on a hosted relay
- [x] Weeks 18–20: Creator (YouTube), any IMAP mailbox, phone-ready app and layout, Android build workflow. Built; Instagram, Outlook sign-in and YouTube Analytics wait on owner-registered apps; the gate waits on a hosted relay and a phone
- [ ] Next on the roadmap: see [ROADMAP.md](ROADMAP.md) (nothing is planned after week 20 yet; ask the owner)
