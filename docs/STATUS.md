# Where Bindle stands

A plain checklist of what is built, what is being built now, and what is left.
Updated at the end of every week of [the roadmap](ROADMAP.md). Last updated: weeks 15 to 17 (14 October 2026).

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

## Being worked on now

- [ ] Weeks 11–12 gate, "a week of email triage from Bindle": connect Gmail (Work → Inbox, with an app password), then triage from Bindle for a week. The cloud session that built this cannot reach Gmail, so this needs the real app
- [ ] Weeks 9–10 gate, "real review requests become promises": connect your own GitHub in the desktop app (Work → Code), make a promise from a real review request, and keep it. The cloud session that built this cannot reach GitHub's search, so this needs the real app

- [ ] Week 8 gate, "all acceptance checks pass": the automated checks pass. The hands-on checks in [ACCEPTANCE.md](ACCEPTANCE.md) still need a person on a real Windows and macOS machine
- [ ] Signed builds need the Apple and Windows certificates added as repository secrets. Only the owner can do this
- [ ] Turn on updates: the owner makes the updater key and adds three secrets (see "Updates inside the app" in [RELEASING.md](RELEASING.md)), then releases v0.0.2. People on v0.0.1 install v0.0.2 by hand once; every later version arrives inside the app

## Left to deal with

### Before Tier 1 is called done
- [ ] **Founder gate:** use Bindle on at least 25 of 30 days (handbook P10)
- [ ] Hands-on test on Windows and macOS: Jot down shortcut, notifications (macOS may ask for permission first), dark theme, and the AI connection from Claude Desktop
- [ ] Add the signing secrets and cut a signed release
- [ ] Hands-on test of an update: install v0.0.2, release v0.0.3, and check each system offers and installs it
- [ ] A screen reader pass (NVDA on Windows, VoiceOver on macOS, Orca on Linux)
- [ ] Some Wayland desktops refuse global shortcuts. Bindle then works with the in-app shortcut only; a desktop-portal shortcut would fix this

### Known rough edges
- [ ] GitHub needs the system keychain. A Linux desktop with no Secret Service (some minimal window managers) cannot keep the token, so Code cannot connect there yet
- [ ] GitHub sign-in uses a personal access token. A "Sign in with GitHub" button needs a registered GitHub app (OAuth device flow), which only the owner can create
- [ ] Each search reads at most 100 items
- [ ] Gmail uses an app password, which needs 2-Step Verification on the Google account. "Sign in with Google" needs a Google Cloud app that only the owner can register (and Google's review for Gmail access)
- [ ] Inbox reads the newest 200 emails of the last two weeks in each folder, and only the start of each email
- [ ] Review waits count from when a pull request was opened, not from when your review was requested; GitHub's search does not say when that was without one more request per pull request
- [ ] The Desk's numbers need a few weeks of history before the comparisons say much
- [ ] Bindle Cloud needs a relay that is running somewhere; none is hosted yet. Until one is, the gate (two people share Ours for two weeks) cannot start
- [ ] Sharing a promise shares its title, state, dates, amount, first step, note, and who added and kept it. Its person and history stay on each device
- [ ] Leaving a space keeps it on the relay for the others. There is no way yet to remove someone or to delete a space for everyone
- [ ] Gmail only for now; Outlook and other IMAP mailboxes come in weeks 18–20
- [ ] Putting back an old safety copy also brings back that day's settings (by design, but it can surprise)
- [ ] When an AI's suggestion mentions an amount, Bindle stores it as the promise's amount
- [ ] A person with no promises left stays on People; there is no archive yet
- [ ] The handbook here is a summary. The full one is in Notion, so some checks were written from the summary
- [ ] `bindle-parse/tests/table.rs` is just over 400 lines; split it when it next grows

### After Tier 1 (the roadmap's next stages)
- [x] Weeks 15–17: Bindle Cloud: accounts, Mine and Ours, invites, recovery kit (sync and end-to-end encryption). Built; the gate waits on a hosted relay
- [ ] Weeks 18–20: Creator (YouTube, Instagram), Outlook and IMAP, phone build
