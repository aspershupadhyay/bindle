# Where Bindle stands

A plain checklist of what is built, what is being built now, and what is left.
Updated at the end of every week of [the roadmap](ROADMAP.md). Last updated: week 8 (14 October 2026).

## Built

### Core (Rust)
- [x] `bindle-core`: promise states and allowed changes, "close the loop" (a kept promise says who owes it, a firm one has a day), running late, waiting on others, the week and day capacity (5), and the reminder rules (at most 3 a day, none 10 PM to 7 AM, never late)
- [x] `bindle-parse`: English and Hinglish days, times, money, people, who owes it and soft promises, with a test table. Words that could mean two things are flagged for Sort. Festivals, months and places are never read as people
- [x] `bindle-store`: SQLite (WAL, FTS5) with migrations 0001 to 0006, history and Undo for every change, Home, People, search, settings, Wrap up, the reminder log, AI rules, export and safety copies
- [x] `bindle-sync`: hybrid logical clocks and field merges (ready for Bindle Cloud; not wired to anything yet)
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

### Project
- [x] Handbook mirror, roadmap, changelog, contributing guide, DCO and licence headers
- [x] CI on Windows, macOS and Linux: format, lint, tests, views and licence headers. It runs on pushes to `main`, to working branches, and on pull requests. First full run passed on all three on 14 October 2026
- [x] Release workflow (week 8): a version tag builds installers for all three systems. They are signed once the signing secrets are added (see [RELEASING.md](RELEASING.md))

## Being worked on now

- [ ] Week 8 gate, "all acceptance checks pass": the automated checks pass. The hands-on checks in [ACCEPTANCE.md](ACCEPTANCE.md) still need a person on a real Windows and macOS machine
- [ ] Signed builds need the Apple and Windows certificates added as repository secrets. Only the owner can do this

## Left to deal with

### Before Tier 1 is called done
- [ ] **Founder gate:** use Bindle on at least 25 of 30 days (handbook P10)
- [ ] Hands-on test on Windows and macOS: Jot down shortcut, notifications (macOS may ask for permission first), dark theme, and the AI connection from Claude Desktop
- [ ] Add the signing secrets and cut the first tagged release
- [ ] A screen reader pass (NVDA on Windows, VoiceOver on macOS, Orca on Linux)
- [ ] Some Wayland desktops refuse global shortcuts. Bindle then works with the in-app shortcut only; a desktop-portal shortcut would fix this

### Known rough edges
- [ ] Putting back an old safety copy also brings back that day's settings (by design, but it can surprise)
- [ ] When an AI's suggestion mentions an amount, Bindle stores it as the promise's amount
- [ ] A person with no promises left stays on People; there is no archive yet
- [ ] The handbook here is a summary. The full one is in Notion, so some checks were written from the summary
- [ ] `bindle-parse/tests/table.rs` is just over 400 lines; split it when it next grows

### After Tier 1 (the roadmap's next stages)
- [ ] Weeks 9–10: mode switch, GitHub source, Code screen, Make it a promise
- [ ] Weeks 11–12: Gmail source, Inbox, Board
- [ ] Weeks 13–14: Insights and the Work Desk
- [ ] Weeks 15–17: Bindle Cloud: accounts, Mine and Ours, invites, recovery kit (sync and end-to-end encryption)
- [ ] Weeks 18–20: Creator (YouTube, Instagram), Outlook and IMAP, phone build
