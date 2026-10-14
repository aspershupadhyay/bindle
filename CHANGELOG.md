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
- Jot down window: Alt+Shift+J opens it over any app; Enter saves and shows "Saved", Escape or clicking away closes it. Every window refreshes when a promise is saved.
- `bindle-parse` reads days (today, kal, parso, weekdays in English and Hindi, "next week", "in 3 days", "2 din mein", written dates day first), deadlines ("by Friday", "Friday tak"), times ("8:30 pm", "at 4", "shaam 7 baje", "in the evening"), money ("₹2,340", "Rs 500", "1.5 lakh", "$12"), people ("to Rohan", "@aman", "Rohan ko", names already in Bindle), who owes it ("Rohan will", "bhejega", "I'll", "just for me") and soft promises ("sometime", "kabhi"). Words that could mean two things are flagged so Sort can ask.
- Jot down keeps exactly what was typed as the promise's source, saves the day or deadline, time, amount and person, and adds new people.
- Chips on Home show the day, time, amount, person, who owes it and soft promises.
