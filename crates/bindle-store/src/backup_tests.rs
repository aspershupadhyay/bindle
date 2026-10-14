// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

use super::*;
use crate::open;
use crate::promises::{jot_down, Jot};
use chrono::TimeZone;

fn at(hour: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 14, hour, 0, 0).unwrap()
}

/// A fresh folder for one test, removed when the test ends.
struct Folder(PathBuf);

impl Drop for Folder {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

impl std::ops::Deref for Folder {
    type Target = PathBuf;
    fn deref(&self) -> &PathBuf {
        &self.0
    }
}

/// A fresh folder with a database file in it.
fn folder() -> (Folder, Connection) {
    let dir = std::env::temp_dir().join(format!("bindle-test-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir_all(&dir).expect("dir");
    let conn = open(&dir.join("bindle.db")).expect("open");
    (Folder(dir), conn)
}

#[test]
fn one_copy_is_made_a_day() {
    let (dir, conn) = folder();
    let copies = dir.join("safety");
    assert!(daily_copy(&conn, &copies, at(9)).expect("copy").is_some());
    assert!(daily_copy(&conn, &copies, at(20)).expect("copy").is_none());
    let tomorrow = at(9) + chrono::Duration::hours(25);
    assert!(daily_copy(&conn, &copies, tomorrow)
        .expect("copy")
        .is_some());
    assert_eq!(list(&copies).expect("list").len(), 2);
}

#[test]
fn only_the_newest_copies_are_kept() {
    let (dir, conn) = folder();
    let copies = dir.join("safety");
    for day in 0..(COPIES_KEPT as i64 + 3) {
        make_copy(&conn, &copies, at(9) + chrono::Duration::days(day)).expect("copy");
    }
    let kept = list(&copies).expect("list");
    assert_eq!(kept.len(), COPIES_KEPT);
    assert!(kept[0].made_at > kept[1].made_at, "newest first");
}

#[test]
fn a_restore_brings_back_the_old_data_and_keeps_todays() {
    let (dir, mut conn) = folder();
    let copies = dir.join("safety");
    jot_down(&mut conn, &Jot::plain("Call mom"), at(9)).expect("jot");
    let morning = make_copy(&conn, &copies, at(9)).expect("copy");
    jot_down(&mut conn, &Jot::plain("Pay rent"), at(10)).expect("jot");

    let before = restore(&mut conn, &copies, &morning.file_name, at(11)).expect("restore");
    let count: u32 = conn
        .query_row("SELECT count(*) FROM promise", [], |r| r.get(0))
        .expect("count");
    assert_eq!(count, 1, "back to the morning");

    restore(&mut conn, &copies, &before.file_name, at(12)).expect("undo the restore");
    let count: u32 = conn
        .query_row("SELECT count(*) FROM promise", [], |r| r.get(0))
        .expect("count");
    assert_eq!(count, 2, "the restore itself can be undone");
}

#[test]
fn a_restore_only_reads_listed_copies() {
    let (dir, mut conn) = folder();
    let copies = dir.join("safety");
    make_copy(&conn, &copies, at(9)).expect("copy");
    for name in ["../bindle.db", "bindle.db", "notes.txt"] {
        assert!(matches!(
            restore(&mut conn, &copies, name, at(10)),
            Err(StoreError::NotFound)
        ));
    }
    std::fs::write(copies.join("bindle-20261001-090000.db"), "not sqlite").expect("junk");
    assert!(restore(&mut conn, &copies, "bindle-20261001-090000.db", at(10)).is_err());
}
