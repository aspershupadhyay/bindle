// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

// The one Node function the tests use (contrast.test.ts reads tokens.css).
// The app itself never runs in Node, so the full Node types are not installed.
declare module "node:fs" {
  export function readFileSync(path: URL, encoding: "utf8"): string;
}
