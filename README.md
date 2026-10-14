<p align="center">
  <img src="docs/design/logo-large.png" alt="Bindle logo: a cloth bag tied to a stick, in pixel art" width="120">
</p>

<h1 align="center">Bindle</h1>

<p align="center"><b>Everything that needs you, in one bag.</b><br>
A calm, local-first app that carries your promises, from your life and your work, so your head doesn't have to.</p>

<p align="center">
  <img src="docs/design/hero-hills.png" alt="Pixel art: a traveller with a bindle walking over green hills at sunset" width="768">
</p>

> **Status: early.** The handbook and designs are done, the Rust core has its first tested crates, and the desktop app opens, and Jot down works from anywhere with Alt+Shift+J, understanding days, times, money and people in English and Hinglish. Sort turns each jot into a real promise, every change can be undone, and Home shows your top 3, what is running late, what others owe you and how full the week is. Promises lists everything by state with search across all of it (Ctrl+K), and People shows what you and each person owe each other, with a recap you can send. Wrap up, notifications and first run come next.

<p align="center">
  <img src="docs/screens/week-4-home.png" alt="Life Home in the Bindle desktop app: top 3 with one done, Waiting on others with a nudge, Coming up, and the week strip with a full Friday in honey" width="768">
</p>
<p align="center">
  <img src="docs/screens/week-2-jot-down.png" alt="The Jot down window reading 'Pay electricity bill ₹2,340 kal shaam 7 baje' as Tomorrow 7 PM, with a question mark, and ₹2,340" width="480">
</p>
<p align="center">
  <img src="docs/screens/week-3-sort.png" alt="Sort: one promise at a time, with who will do it, when, a deadline box, a first step, and an Undo bar after the last one was kept" width="768">
</p>
<p align="center">
  <img src="docs/screens/week-5-people.png" alt="People: a list of people with what each owes, and one person's page with what they will do and a recap ready to copy" width="768">
</p>

## What Bindle is

Bindle remembers every promise in your life, the ones you make and the ones made to you. You jot things down in two seconds, sort them once a day, and see what matters on one calm Home screen.

It has two modes and two kinds of space:

| | |
|---|---|
| **Life** | The calm promise ledger: Home, Sort, Promises, People, Wrap up. No scores, no red, no streaks. |
| **Work** | A focused desk: what needs you across Gmail, GitHub and your channels, a board, and real analysis of how long people wait on you. |
| **Mine** | Private. Only you, on your devices. |
| **Ours** | Shared with your partner or family through Bindle Cloud, end-to-end encrypted. |

## Designs

Hand-crafted screens with a warm paper palette and a hand-made pixel art layer. Every page has its own small pixel scene in its header. These are from v2 of the design; v1 is kept unchanged next to it.

| Work Desk | Code |
|---|---|
| <img src="docs/design/v2/desk.png" alt="Work Desk dashboard with a railway platform pixel scene" width="420"> | <img src="docs/design/v2/code.png" alt="Code page with an arcade pixel scene" width="420"> |
| **Creator, all accounts** | **Creator, one account** |
| <img src="docs/design/v2/creator.png" alt="Creator overview with every account" width="420"> | <img src="docs/design/v2/creator-cte.png" alt="Creator detail for one account" width="420"> |
| **Inbox** | **Board** |
| <img src="docs/design/v2/inbox.png" alt="Inbox with a post office pixel scene" width="420"> | <img src="docs/design/v2/board.png" alt="Board with a workshop pixel scene" width="420"> |
| **Life Home** | **Ours** |
| <img src="docs/design/v2/main.png" alt="Life Home screen" width="420"> | <img src="docs/design/v2/ours.png" alt="Ours shared space with a two-room house pixel scene" width="420"> |

The brand sheet is in [docs/design/v2/brand.png](docs/design/v2/brand.png). The design source files, sprites and render script are in [design/](design/README.md).

## Tech stack

| Layer | Choice |
|---|---|
| App shell | Tauri 2 (desktop first, iOS and Android later) |
| Core | Rust workspace: `bindle-core`, `bindle-parse`, `bindle-store`, `bindle-sync` |
| Local data | SQLite (bundled, WAL, FTS5); SQLCipher once cloud is on |
| UI | React, TypeScript, Vite, Radix, cmdk, TanStack Query, Zustand, Motion |
| Charts and board | Custom SVG on d3-scale and d3-shape; dnd-kit |
| Work sources | Gmail, Microsoft Graph, GitHub, YouTube, Instagram Graph (OAuth with PKCE, tokens in the OS keychain) |
| AI connection | MCP server in Rust (`rmcp`), off by default |
| Bindle Cloud | Rust (Axum) sync server, Postgres on Supabase, Fly.io, Cloudflare R2 backups |
| Sync | Hybrid logical clocks, field-level merges, XChaCha20-Poly1305 and X25519 end-to-end encryption |

## Repository layout

```
bindle/
  app/
    src/             React views: shell, Home, Sort, Promises, People, Settings, the Jot down window
    src/art/         pixel sprites at 1x (scale by whole numbers only)
    src/tokens.css   design tokens
    src-tauri/       the Tauri desktop app and its typed commands
  crates/
    bindle-core/     promise rules: states, allowed changes, running late
    bindle-parse/    understands what you type: days, times, money, people, English and Hinglish
    bindle-store/    SQLite schema, migrations, Sort, history, Undo, Home, People and search
    bindle-sync/     hybrid logical clocks and merges for Bindle Cloud
  design/            design sources, sprites and render tools
  docs/              handbook mirror and designs
```

Coming next: Wrap up, notifications, first run and Settings (week 6 of the plan in [docs/ROADMAP.md](docs/ROADMAP.md)).

Alt+Shift+J is claimed system-wide where the desktop allows it. Some Wayland sessions do not let apps claim global shortcuts; there it works inside Bindle, and the Jot down button is always there.

## Build and test

You need Rust (stable), Node 22, and on Linux the WebKitGTK libraries (`sudo apt install libwebkit2gtk-4.1-dev librsvg2-dev` on Debian and Ubuntu; see the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for other systems).

```sh
cd app && npm install

npm run tauri dev        # run the desktop app
npm run dev              # views only, in a browser, with an in-memory preview (nothing saved)
```

Checks, the same as CI:

```sh
(cd app && npm run typecheck && npm test && npx vite build)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The views must be built (`npx vite build`) before the Rust workspace compiles, because the desktop app embeds them.

## Handbook

The full product handbook (product guide, design system, data model, screens, AI connection, cloud and build plan) lives in Notion and is mirrored in [docs/HANDBOOK.md](docs/HANDBOOK.md).

## Ownership and licence

Bindle is created, owned and led by **Sparsh Upadhyay** (Aspersh).
The code is licensed under the [Apache License 2.0](LICENSE); every copy must keep the [NOTICE](NOTICE) file.
The Bindle name and logo are not covered by the code licence; see [TRADEMARKS.md](TRADEMARKS.md).
