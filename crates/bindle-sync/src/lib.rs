// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! The heart of Bindle Cloud sync (handbook P15).
//!
//! Every change is a small operation stamped with a hybrid logical clock (HLC).
//! An HLC is wall-clock time plus a counter, so stamps keep their order even
//! when two devices' clocks disagree a little. When two devices edit the same
//! field, the later stamp wins and the other value stays in history.
//!
//! Encryption and the network client are added in later steps of the plan.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// A hybrid logical clock stamp. Compared by time, then counter, then device,
/// so two different devices can never produce equal stamps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stamp {
    /// Milliseconds since the Unix epoch, never going backwards on a device.
    pub millis: u64,
    /// Breaks ties when several changes happen in the same millisecond.
    pub counter: u32,
    /// The device that made the change.
    pub device: String,
}

impl Ord for Stamp {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.millis, self.counter, &self.device).cmp(&(other.millis, other.counter, &other.device))
    }
}

impl PartialOrd for Stamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// One device's clock.
#[derive(Debug, Clone)]
pub struct Clock {
    device: String,
    last: Option<Stamp>,
}

impl Clock {
    pub fn new(device: impl Into<String>) -> Self {
        Self {
            device: device.into(),
            last: None,
        }
    }

    /// Makes a stamp for a local change. `now_millis` is the wall clock,
    /// passed in so tests can control time.
    pub fn tick(&mut self, now_millis: u64) -> Stamp {
        let stamp = match &self.last {
            Some(last) if last.millis >= now_millis => Stamp {
                millis: last.millis,
                counter: last.counter + 1,
                device: self.device.clone(),
            },
            _ => Stamp {
                millis: now_millis,
                counter: 0,
                device: self.device.clone(),
            },
        };
        self.last = Some(stamp.clone());
        stamp
    }

    /// Moves this clock past a stamp received from another device, so the
    /// next local change is always ordered after everything already seen.
    pub fn observe(&mut self, remote: &Stamp, now_millis: u64) {
        let newest = self
            .last
            .iter()
            .chain(std::iter::once(remote))
            .max()
            .cloned();
        if let Some(newest) = newest {
            if newest.millis >= now_millis {
                self.last = Some(Stamp {
                    device: self.device.clone(),
                    ..newest
                });
            }
        }
    }
}

/// A single field change on one record, as it travels between devices.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldOp {
    /// "table:id", for example "promise:0192f3...".
    pub record: String,
    pub field: String,
    /// JSON text of the new value; `None` clears the field.
    pub value: Option<String>,
    pub stamp: Stamp,
}

/// Picks the winning value for one field: the operation with the later stamp.
/// The loser is not thrown away by callers; it goes into promise history.
pub fn merge<'a>(current: &'a FieldOp, incoming: &'a FieldOp) -> &'a FieldOp {
    if incoming.stamp > current.stamp {
        incoming
    } else {
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stamps_keep_order_when_the_wall_clock_stands_still() {
        let mut clock = Clock::new("laptop");
        let first = clock.tick(1_000);
        let second = clock.tick(1_000);
        assert!(second > first);
    }

    #[test]
    fn stamps_keep_order_when_the_wall_clock_goes_backwards() {
        let mut clock = Clock::new("laptop");
        let first = clock.tick(2_000);
        let second = clock.tick(1_500);
        assert!(second > first, "a clock adjustment must not reorder edits");
    }

    #[test]
    fn after_seeing_a_newer_remote_stamp_local_changes_come_after_it() {
        let mut laptop = Clock::new("laptop");
        let mut phone = Clock::new("phone");
        let remote = phone.tick(9_000);
        laptop.observe(&remote, 5_000);
        let local = laptop.tick(5_000);
        assert!(local > remote);
    }

    #[test]
    fn the_later_edit_to_a_field_wins() {
        let mut laptop = Clock::new("laptop");
        let mut phone = Clock::new("phone");
        let a = FieldOp {
            record: "promise:p1".into(),
            field: "state".into(),
            value: Some("\"open\"".into()),
            stamp: laptop.tick(1_000),
        };
        let b = FieldOp {
            record: "promise:p1".into(),
            field: "state".into(),
            value: Some("\"done\"".into()),
            stamp: phone.tick(1_200),
        };
        assert_eq!(merge(&a, &b).value.as_deref(), Some("\"done\""));
        assert_eq!(
            merge(&b, &a).value.as_deref(),
            Some("\"done\""),
            "merge order must not matter"
        );
    }
}
