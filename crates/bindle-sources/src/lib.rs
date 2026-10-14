// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Where Bindle reads work from (handbook P14). Each source only reads: it
//! never comments, merges, closes or sends anything (handbook principle 6).
//! What it finds goes to `bindle_store::work`, and becomes a promise only
//! when the person says so.

pub mod github;
pub mod gmail;
pub mod mail;
