// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Bindle Cloud on the device (handbook P15).
//!
//! Everything shared in Ours is encrypted here, before it leaves the
//! device, with a key the relay never sees. The relay only stores sealed
//! batches and hands them back in order. Three secrets exist:
//!
//! - a **space key** for each Ours space. From it come the space's id, the
//!   token that lets a device write to it, and its encryption key;
//! - an **invite secret**, which seals a space key for one person, once, for
//!   seven days. It travels only in the invite link, after the `#`, which
//!   browsers and apps never send to a server;
//! - a **recovery key**, the person's recovery kit. It seals the list of
//!   their spaces so a new device can bring them back.

pub mod client;
pub mod codes;
pub mod crypto;

pub use client::{CloudError, Relay};
pub use crypto::{Key, Sealed};
