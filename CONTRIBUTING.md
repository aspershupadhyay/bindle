# Contributing to Bindle

Thank you for helping. A few rules keep the project healthy and the credit clear.

## Before you start

- Read the matching handbook chapter in [docs/HANDBOOK.md](docs/HANDBOOK.md). Every rule in the code points back to one.
- For anything bigger than a small fix, open an issue first so we agree on the approach.

## Code standards

- Anyone should understand any file in five minutes. Clear names first, comments for the why.
- Every public item has a doc comment. Functions stay short (about 40 lines), files under about 400.
- Tests are named as sentences, for example `a_soft_promise_is_never_running_late`.
- Every source file starts with:

  ```
  // SPDX-License-Identifier: Apache-2.0
  // Copyright 2026 Sparsh Upadhyay and the Bindle contributors.
  ```

- Use Bindle's words: promise, Jot down, Sort, Running late, Move it, Let go. Never "overdue".

## Checks

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI runs the same checks on every pull request.

## Commits and sign-off

Use short, clear messages like `feat(core): add running-late rule`.

Every commit must be signed off under the [Developer Certificate of Origin](https://developercertificate.org/):

```sh
git commit -s
```

This confirms you have the right to submit the change under the project licence.

## Pull requests

Say what changed, why, which handbook chapter it follows, and how you tested it. Screenshots for anything visible.
