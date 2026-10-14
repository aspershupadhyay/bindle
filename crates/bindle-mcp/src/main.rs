// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! `bindle-mcp --db <path>`: Bindle's AI connection on its own, for
//! development. The app does the same when started with `--mcp`.

fn main() -> std::process::ExitCode {
    bindle_mcp::run_from_args(std::env::args().skip(1))
}
