# Acceptance checks (Tier 1, week 8)

The checks Bindle must pass before Tier 1 is called done (handbook P10). **Automated** checks run in CI or with one command. **By hand** checks need a person at a real machine; record the date and system when done.

## Automated

| Check | How | Status |
|---|---|---|
| Rules, parsing, store, AI connection, GitHub and Gmail sources: 182 Rust tests | `cargo test --workspace` | Passing on Linux, macOS and Windows in CI |
| Views: typecheck, 89 tests, build | `cd app && npm run typecheck && npm test && npx vite build` | Passing |
| Format and lint, no warnings | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` | Passing |
| Every source file has the licence header | CI job "Licence headers" | Passing |
| Colour contrast, WCAG AA (4.5:1 text, 3:1 focus ring), light and dark | `app/src/lib/contrast.test.ts` (48 checks) | Passing |
| Every button and link has a name, every image has alt text, every field has a label, one `h1` and a `main` landmark per screen, and the skip link comes first | `node app/scripts/a11y-audit.mjs` against `npm run dev` (needs Playwright) | Passing on first run, Home, Sort, Promises, People, Wrap up, Settings, Code and Inbox (before and after connecting) and Board |
| Search under 100 ms on 10,000 promises | `bindle-store` search test | Passing (about 5 ms in release) |
| Capture: "Saved" only after the write commits | `bindle-store` and `JotField` tests | Passing |
| Migrations 0001 to 0008 upgrade a real older database | Done by hand on week 6 data; migration tests in `bindle-store` | Passing |
| Linux `.deb` package builds | `cd app && npx tauri build --bundles deb` | Passing (week 8) |

## By hand

| Check | Linux | Windows | macOS |
|---|---|---|---|
| The app opens with the v2 sidebar | Done (week 1 onward) | To do | To do |
| Alt+Shift+J opens Jot down over another app; Enter saves | Done (X11) | To do | To do |
| A new person reaches Home in under 3 minutes | Done: 41 seconds | To do | To do |
| A reminder arrives at a promise's time | Done (week 6) | To do | To do (may ask for permission first) |
| Dark theme, including the logo | Done (week 7) | To do | To do |
| Claude reads Home and suggests a promise that lands in Sort | Done with Claude Code (week 7) | To do with Claude Desktop | To do with Claude Desktop |
| Export JSON and CSV; make a safety copy and put it back | Done (week 7) | To do | To do |
| Weeks 9–10: connect GitHub, and a real review request becomes a promise in Sort | To do | To do | To do |
| Weeks 11–12: connect Gmail with an app password, and triage email from Bindle for a week | To do | To do | To do |
| Keyboard only: every screen can be reached and used with Tab, Enter, Space and Escape | Done in the browser preview | To do | To do |
| Screen reader reads each screen sensibly | To do (Orca) | To do (NVDA) | To do (VoiceOver) |
| Text zoom to 200% keeps every screen usable | To do | To do | To do |
| Signed installer opens without a warning | Not signed (Linux packages are not) | To do: needs the certificate | To do: needs the certificate |

**Founder gate:** Bindle used on at least 25 of 30 days. Not started.
