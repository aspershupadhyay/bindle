# Changelog

## Unreleased

- Project renamed from Life OS to Bindle.
- Rust workspace with four tested crates: `bindle-core` (promise rules), `bindle-parse` (English and Hinglish day words), `bindle-store` (SQLite schema and migrations, including Mine and Ours spaces), `bindle-sync` (hybrid logical clocks and field merges).
- Design tokens and the first pixel art sprites.
- Screen designs for Life Home, Work Desk, Inbox, Board, Creator, Ours and the brand sheet.
- CI for formatting, linting and tests.
- Desktop shell (Tauri 2, React, Vite): the v2 sidebar with the Life and Work switch, Home with a working Jot down field and what waits in Sort, Settings with the build and data version, and a page with its own pixel scene for each screen still to come.
- `bindle-core`: the `Promise` type and the exact words the database stores for each state, direction, strength and mode.
- `bindle-store`: Jot down saves an unsorted promise and its `created` event in one transaction; list and count promises by state.
- Typed commands between the views and Rust, with an in-memory preview for running the views in a browser.
- App icons drawn from the logo sprite at whole-number scales.
- CI builds the views and the desktop app on all three systems, and checks TypeScript files for the licence header.
