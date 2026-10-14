# Roadmap

One developer, a working build every week. Each stage has a gate before the next starts.

## Tier 1: the core loop (8 weeks)

| Week | Build | Done when |
|---|---|---|
| 1 | Tauri shell, design tokens, migrations, core types, typed commands, CI on three OSes | Empty app opens on Windows, macOS and Linux |
| 2 | Jot down window, global shortcut, parser with the test table | All parsing tests pass |
| 3 | Sort, lifecycle, history, Undo | Founder uses it daily |
| 4 | Home: top 3, running late, waiting on others, week strip | Home checks pass |
| 5 | Promises, People with Send recap, search | Search under 100 ms on 10,000 promises |
| 6 | Wrap up, notifications, first run, Settings | New user reaches Home in under 3 minutes |
| 7 | AI connection (MCP), export, safety copies | Claude reads Home and suggests a promise |
| 8 | Accessibility, Linux fixes, signed builds | All acceptance checks pass |

**Gate:** the founder uses Bindle on at least 25 of 30 days.

## Then: Work and Ours

| Weeks | Build | Done when |
|---|---|---|
| 9–10 | Mode switch, GitHub source, Code screen, Make it a promise | Real review requests become promises |
| 11–12 | Gmail source, Inbox, Board | A week of email triage from Bindle |
| 13–14 | Insights and the Work Desk | Desk checks pass |
| 15–17 | Bindle Cloud: accounts, Mine and Ours, invites, recovery kit | Two people share Ours for two weeks |
| 18–20 | Creator (YouTube, Instagram), Outlook and IMAP, phone build | Ours works on a phone |

## Done so far

- [x] Handbook and designs
- [x] `bindle-core`: states, allowed changes, running late, waiting on others
- [x] `bindle-parse`: English and Hinglish day words
- [x] `bindle-store`: schema and migrations 0001 and 0002
- [x] `bindle-sync`: hybrid logical clocks and field merges
- [x] Week 1: Tauri shell with the v2 sidebar, Life and Work switch, Home, Settings and placeholder screens; `Promise` core type; store reads and writes with history; typed commands (`app_info`, `jot_down`, `list_promises`, `count_promises`); CI building the app on Windows, macOS and Linux
- [x] Week 2: Jot down window on Alt+Shift+J from anywhere, live chips while typing, and `bindle-parse` reading days, times, money, people, who owes it and how firm it is, in English and Hinglish, with its test table
- [x] Week 3: Sort one promise at a time (who, when, deadline, first step; keep, later or let go), the "close the loop" rule, Done and reopen, Move it, full history, and Undo for every change (Undo bar and Ctrl+Z); migration 0003
- [x] Week 4: Home from the v2 design: top 3 with suggestions, Running late (Move it, Let go), Waiting on others with Nudge, Coming up, the week strip with day capacity, and this week's kept, moved and let go; 13 Home checks in `bindle-store`
- [x] Week 5: Promises by state with search across everything (Ctrl+K or "/"), People with both sides, kept lately, rename and Send recap; migration 0004 keeps the search index in step. Search on 10,000 promises: about 9 ms in a debug build, 5 ms in release (gate: under 100 ms)
- [x] Week 6: first run in four skippable steps (41 seconds end to end in a test run; gate: under 3 minutes), Wrap up, Settings with themes, and reminders: quiet by default, at most 3 a day, none from 10 PM to 7 AM, never late, never about being late; migration 0005
- [x] Week 7: the AI connection (`bindle --mcp`): off, read only, or read and suggest; suggestions wait in Sort; people can be kept from AI; every call is logged. Gate: Claude read Home and suggested a promise, which landed in Sort. Export to JSON and CSV, daily safety copies (the last 14) that can be put back, people's pictures and pronouns, pixel icons for Sort, Promises, People and Wrap up, the logo on dark, and CI on branch pushes; migration 0006
- [x] Week 8 (automated part): accessibility (skip link, landmarks, contrast tests in both themes, an audit of names, alt text and labels on every screen), Linux fixes (blank window on some drivers, package settings; the .deb builds and installs), and a release workflow that signs when certificates are added. CI passed on all three systems for the first time. Hands-on checks on Windows and macOS, a screen reader pass and the signing certificates remain: see [ACCEPTANCE.md](ACCEPTANCE.md) and [STATUS.md](STATUS.md)
- [x] Weeks 9–10 (built, gate pending): the mode switch (Bindle remembers Life or Work, and Jot down saves into the mode you are in, marked Work in its window and on each promise), the GitHub source (a personal access token kept in the system keychain; reviews you owe, your pull requests and issues assigned to you, looked at every 15 minutes), the Code screen from the v2 design, and Make it a promise (a work promise in Sort, with who it is for; when GitHub stops listing the item, Code asks whether the promise is done); migration 0007. Gate: real review requests become promises, to be checked with the founder's own GitHub account
- [x] Weeks 11–12 (built, gate pending): Gmail, read only over IMAP with an app password kept in the system keychain (folders opened read-only, nothing marked as read; an email is never sent); Inbox finds threads waiting on your reply, your questions waiting on others, and promises you wrote ("I'll send it by Friday", read with Jot down's parser from the day it was written), skipping newsletters, receipts and alerts; Make it a promise, Add to promises and Not needed; the Board with Ideas, Next, Making (with a limit), Waiting and Shipped; migration 0008. Gate: a week of email triage from Bindle, to be done by the founder
- [x] Weeks 13–14: Insights and the Work Desk. Each look at Gmail and GitHub records how long people waited for your replies and reviews (migration 0009, `wait_sample`); the Desk shows what needs you from every source, oldest first, the median waits against your own last four weeks, shipped this week (with "best in 8 weeks"), each source's state, this week's work, and eight weeks of waits with a table view. Gate: the Desk checks in `bindle-store` (`insights_tests.rs`) pass
