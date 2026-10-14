// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! The people promises are with (handbook P5).

use crate::promises::{read_row, COLUMNS, FROM};
use crate::StoreError;
use bindle_core::Promise;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use uuid::Uuid;

/// The pictures a person can have. Each names what is drawn, not who someone
/// is: anyone can pick any of them, and change it whenever they like.
pub const AVATARS: &[&str] = &[
    "short", "long", "curly", "bun", "beard", "turban", "scarf", "cap", "grey",
];

/// Whether `word` is one of [`AVATARS`].
pub fn is_avatar(word: &str) -> bool {
    AVATARS.contains(&word)
}

/// One person on the People screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PersonSummary {
    pub id: String,
    pub name: String,
    /// Their picture, or `None` for their first letter.
    pub avatar: Option<String>,
    /// Open promises you made to them.
    pub you_owe: u32,
    /// Open promises they made to you.
    pub they_owe: u32,
}

/// Everything between you and one person.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PersonDetail {
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,
    /// How they like to be referred to, in their words ("she/her").
    pub pronouns: Option<String>,
    /// Kept from AI: neither they nor their promises are shown to one.
    pub hidden_from_ai: bool,
    pub you_owe: Vec<Promise>,
    pub they_owe: Vec<Promise>,
    /// Promises with them kept since `since`, newest first, at most ten.
    pub kept_lately: Vec<Promise>,
}

/// Everyone, with open promises first (most first), then by name.
pub fn summaries(conn: &Connection) -> Result<Vec<PersonSummary>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT person.id, person.display_name, person.avatar,
                count(CASE WHEN p.state = 'open' AND p.direction IN ('mine', 'self') THEN 1 END),
                count(CASE WHEN p.state = 'open' AND p.direction = 'theirs' THEN 1 END)
         FROM person LEFT JOIN promise p ON p.person_id = person.id
         WHERE person.archived_at IS NULL
         GROUP BY person.id
         ORDER BY count(CASE WHEN p.state = 'open' THEN 1 END) DESC, person.display_name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(PersonSummary {
            id: row.get(0)?,
            name: row.get(1)?,
            avatar: row.get(2)?,
            you_owe: row.get(3)?,
            they_owe: row.get(4)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// What you owe one person, what they owe you, and what was kept since `since`
/// (an RFC 3339 time).
pub fn detail(conn: &Connection, id: &str, since: &str) -> Result<PersonDetail, StoreError> {
    let (name, avatar, pronouns, hidden_from_ai) = conn
        .query_row(
            "SELECT display_name, avatar, pronouns, hidden_from_ai FROM person WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?
        .ok_or(StoreError::NotFound)?;
    let list =
        |condition: &str, order: &str, since: Option<&str>| -> Result<Vec<Promise>, StoreError> {
            let sql = format!(
                "SELECT {COLUMNS} {FROM} WHERE p.person_id = ?1 AND {condition} ORDER BY {order}"
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows = match since {
                Some(since) => stmt.query_map(params![id, since], read_row)?,
                None => stmt.query_map(params![id], read_row)?,
            };
            Ok(rows.collect::<Result<_, _>>()?)
        };
    let soonest = "coalesce(p.due, p.do_on) IS NULL, coalesce(p.due, p.do_on), p.created_at";
    Ok(PersonDetail {
        id: id.to_string(),
        name,
        avatar,
        pronouns,
        hidden_from_ai,
        you_owe: list(
            "p.state = 'open' AND p.direction IN ('mine', 'self')",
            soonest,
            None,
        )?,
        they_owe: list("p.state = 'open' AND p.direction = 'theirs'", soonest, None)?,
        kept_lately: list(
            "p.state = 'done' AND p.closed_at >= ?2",
            "p.closed_at DESC LIMIT 10",
            Some(since),
        )?,
    })
}

/// Renames a person, for example "Wifey" to "Ananya". Search follows along.
pub fn rename(conn: &Connection, id: &str, name: &str, at: &str) -> Result<(), StoreError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(StoreError::EmptyName);
    }
    let taken: Option<String> = conn
        .query_row(
            "SELECT id FROM person WHERE display_name = ?1 COLLATE NOCASE AND id != ?2 AND archived_at IS NULL",
            params![name, id],
            |r| r.get(0),
        )
        .optional()?;
    if taken.is_some() {
        return Err(StoreError::NameTaken(name.to_string()));
    }
    let changed = conn.execute(
        "UPDATE person SET display_name = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, name, at],
    )?;
    if changed == 0 {
        return Err(StoreError::NotFound);
    }
    Ok(())
}

/// Gives a person a picture from [`AVATARS`], or their first letter for
/// `None`, and their pronouns, if they gave any.
pub fn set_look(
    conn: &Connection,
    id: &str,
    avatar: Option<&str>,
    pronouns: Option<&str>,
    at: &str,
) -> Result<(), StoreError> {
    if avatar.is_some_and(|a| !is_avatar(a)) {
        return Err(StoreError::UnknownAvatar);
    }
    let pronouns = pronouns.map(str::trim).filter(|p| !p.is_empty());
    let changed = conn.execute(
        "UPDATE person SET avatar = ?2, pronouns = ?3, updated_at = ?4 WHERE id = ?1",
        params![id, avatar, pronouns, at],
    )?;
    if changed == 0 {
        return Err(StoreError::NotFound);
    }
    Ok(())
}

/// Archives a person with nothing open between you: they leave People and
/// Jot down's suggestions, and their kept promises stay as they were.
/// Naming them in a new promise brings them back.
pub fn archive(conn: &Connection, id: &str, at: &str) -> Result<(), StoreError> {
    let (name, open): (String, u32) = conn
        .query_row(
            "SELECT person.display_name,
                    (SELECT count(*) FROM promise p WHERE p.person_id = person.id AND p.state = 'open')
             FROM person WHERE person.id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .ok_or(StoreError::NotFound)?;
    if open > 0 {
        return Err(StoreError::StillOpen(name));
    }
    conn.execute(
        "UPDATE person SET archived_at = ?2, updated_at = ?2 WHERE id = ?1",
        params![id, at],
    )?;
    Ok(())
}

/// Brings an archived person back to People. If someone else has taken
/// their name since, the two are told apart by adding "(2)".
pub fn unarchive(conn: &Connection, id: &str, at: &str) -> Result<(), StoreError> {
    let name: String = conn
        .query_row("SELECT display_name FROM person WHERE id = ?1", [id], |r| {
            r.get(0)
        })
        .optional()?
        .ok_or(StoreError::NotFound)?;
    let mut free = name.clone();
    let mut n = 2;
    while conn
        .query_row(
            "SELECT 1 FROM person WHERE display_name = ?1 COLLATE NOCASE AND id != ?2 AND archived_at IS NULL",
            params![free, id],
            |_| Ok(()),
        )
        .optional()?
        .is_some()
    {
        free = format!("{name} ({n})");
        n += 1;
    }
    conn.execute(
        "UPDATE person SET archived_at = NULL, display_name = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, free, at],
    )?;
    Ok(())
}

/// Everyone archived, by name.
pub fn archived(conn: &Connection) -> Result<Vec<PersonSummary>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT id, display_name, avatar FROM person
         WHERE archived_at IS NOT NULL ORDER BY display_name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(PersonSummary {
            id: row.get(0)?,
            name: row.get(1)?,
            avatar: row.get(2)?,
            you_owe: 0,
            they_owe: 0,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// The names of everyone in Bindle, for the parser to recognise.
/// People hidden from AI are still listed: this never leaves the device.
pub fn names(conn: &Connection) -> Result<Vec<String>, StoreError> {
    let mut stmt = conn.prepare(
        "SELECT display_name FROM person WHERE archived_at IS NULL ORDER BY display_name",
    )?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// Finds a person by name, ignoring case, or adds them. Returns their id.
pub fn find_or_add(conn: &Connection, name: &str, at: &str) -> Result<String, StoreError> {
    let existing = conn
        .query_row(
            "SELECT id FROM person WHERE display_name = ?1 COLLATE NOCASE AND archived_at IS NULL",
            [name],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(id) = existing {
        return Ok(id);
    }
    // Someone archived comes back with their history rather than as a stranger.
    let archived: Option<String> = conn
        .query_row(
            "SELECT id FROM person WHERE display_name = ?1 COLLATE NOCASE
             ORDER BY archived_at DESC LIMIT 1",
            [name],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(id) = archived {
        conn.execute(
            "UPDATE person SET archived_at = NULL, updated_at = ?2 WHERE id = ?1",
            params![id, at],
        )?;
        return Ok(id);
    }
    let id = Uuid::now_v7().to_string();
    conn.execute(
        "INSERT INTO person (id, display_name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
        params![id, name, at],
    )?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifecycle::{self, Sorting};
    use crate::open_in_memory;
    use crate::promises::{jot_down, Jot};
    use bindle_core::{Direction, State, Strength};
    use chrono::{TimeZone, Utc};

    fn kept_with(conn: &mut Connection, title: &str, person: &str, direction: Direction) -> String {
        let now = Utc.with_ymd_and_hms(2026, 10, 14, 12, 0, 0).unwrap();
        let id = jot_down(conn, &Jot::plain(title), now).expect("jot").id;
        let sorting = Sorting {
            title: title.into(),
            direction: Some(direction),
            strength: Strength::Soft,
            person: Some(person.into()),
            first_step: None,
            when: None,
            is_deadline: false,
        };
        lifecycle::sort(conn, &id, &sorting, State::Open, now).expect("keep");
        id
    }

    #[test]
    fn people_with_open_promises_come_first_with_both_counts() {
        let mut conn = open_in_memory().expect("open");
        find_or_add(&conn, "Aarti", "x").expect("add");
        kept_with(&mut conn, "Send lesson plan", "Rohan", Direction::Mine);
        kept_with(&mut conn, "Pay invoice", "Rohan", Direction::Theirs);
        kept_with(&mut conn, "Thumbnails", "Aman", Direction::Theirs);
        let list = summaries(&conn).expect("people");
        let rows: Vec<_> = list
            .iter()
            .map(|p| (p.name.as_str(), p.you_owe, p.they_owe))
            .collect();
        assert_eq!(rows, [("Rohan", 1, 1), ("Aman", 0, 1), ("Aarti", 0, 0)]);
    }

    #[test]
    fn a_person_shows_both_sides_and_what_was_kept_lately() {
        let mut conn = open_in_memory().expect("open");
        kept_with(&mut conn, "Send lesson plan", "Rohan", Direction::Mine);
        kept_with(&mut conn, "Pay invoice", "Rohan", Direction::Theirs);
        let done = kept_with(&mut conn, "Share notes", "Rohan", Direction::Mine);
        let now = Utc.with_ymd_and_hms(2026, 10, 14, 13, 0, 0).unwrap();
        lifecycle::mark_done(&mut conn, &done, now).expect("done");
        let id = summaries(&conn).expect("people")[0].id.clone();
        let rohan = detail(&conn, &id, "2026-10-01T00:00:00Z").expect("detail");
        assert_eq!(rohan.you_owe.len(), 1);
        assert_eq!(rohan.they_owe[0].title, "Pay invoice");
        assert_eq!(rohan.kept_lately[0].title, "Share notes");
        let later = detail(&conn, &id, "2026-10-15T00:00:00Z").expect("detail");
        assert!(later.kept_lately.is_empty());
    }

    #[test]
    fn a_rename_cannot_take_someone_elses_name() {
        let conn = open_in_memory().expect("open");
        let wifey = find_or_add(&conn, "Wifey", "x").expect("add");
        find_or_add(&conn, "Rohan", "x").expect("add");
        assert!(matches!(
            rename(&conn, &wifey, "rohan", "y"),
            Err(StoreError::NameTaken(_))
        ));
        assert!(matches!(
            rename(&conn, &wifey, "  ", "y"),
            Err(StoreError::EmptyName)
        ));
        rename(&conn, &wifey, "Ananya", "y").expect("rename");
        assert_eq!(names(&conn).expect("names"), ["Ananya", "Rohan"]);
    }

    #[test]
    fn the_same_name_in_another_case_is_the_same_person() {
        let conn = open_in_memory().expect("open");
        let first = find_or_add(&conn, "Rohan", "2026-10-14").expect("add");
        let again = find_or_add(&conn, "rohan", "2026-10-14").expect("find");
        assert_eq!(first, again);
        assert_eq!(names(&conn).expect("names"), ["Rohan"]);
    }

    #[test]
    fn only_someone_with_nothing_open_can_be_archived() {
        let mut conn = open_in_memory().expect("open");
        let open = kept_with(&mut conn, "Send lesson plan", "Rohan", Direction::Mine);
        let rohan = find_or_add(&conn, "Rohan", "x").expect("find");
        assert!(matches!(
            archive(&conn, &rohan, "y"),
            Err(StoreError::StillOpen(name)) if name == "Rohan"
        ));
        let now = Utc.with_ymd_and_hms(2026, 10, 14, 13, 0, 0).unwrap();
        lifecycle::mark_done(&mut conn, &open, now).expect("done");
        archive(&conn, &rohan, "y").expect("archive");
        assert!(summaries(&conn).expect("people").is_empty());
        assert!(names(&conn).expect("names").is_empty());
        assert_eq!(archived(&conn).expect("archived")[0].name, "Rohan");
        let kept = detail(&conn, &rohan, "2026-10-01T00:00:00Z").expect("detail");
        assert_eq!(kept.kept_lately.len(), 1, "their history stays");
        assert!(matches!(
            archive(&conn, "nobody", "y"),
            Err(StoreError::NotFound)
        ));
    }

    #[test]
    fn naming_an_archived_person_brings_them_back() {
        let conn = open_in_memory().expect("open");
        let rohan = find_or_add(&conn, "Rohan", "x").expect("add");
        archive(&conn, &rohan, "y").expect("archive");
        assert_eq!(find_or_add(&conn, "rohan", "z").expect("find"), rohan);
        assert!(archived(&conn).expect("archived").is_empty());
        assert_eq!(names(&conn).expect("names"), ["Rohan"]);
    }

    #[test]
    fn bringing_back_never_makes_two_people_with_one_name() {
        let conn = open_in_memory().expect("open");
        let first = find_or_add(&conn, "Rohan", "x").expect("add");
        archive(&conn, &first, "y").expect("archive");
        let wifey = find_or_add(&conn, "Wifey", "x").expect("add");
        rename(&conn, &wifey, "Rohan", "y").expect("the name is free while archived");
        unarchive(&conn, &first, "z").expect("bring back");
        assert_eq!(names(&conn).expect("names"), ["Rohan", "Rohan (2)"]);
    }

    #[test]
    fn anyone_can_have_any_picture_and_their_own_pronouns() {
        let conn = open_in_memory().expect("open");
        let id = find_or_add(&conn, "Ananya", "x").expect("add");
        set_look(&conn, &id, Some("beard"), Some(" she/her "), "y").expect("look");
        let ananya = detail(&conn, &id, "2026-10-01T00:00:00Z").expect("detail");
        assert_eq!(ananya.avatar.as_deref(), Some("beard"));
        assert_eq!(ananya.pronouns.as_deref(), Some("she/her"));
        assert!(matches!(
            set_look(&conn, &id, Some("dragon"), None, "y"),
            Err(StoreError::UnknownAvatar)
        ));
        set_look(&conn, &id, None, Some(""), "z").expect("back to a letter");
        let ananya = detail(&conn, &id, "2026-10-01T00:00:00Z").expect("detail");
        assert_eq!((ananya.avatar, ananya.pronouns), (None, None));
    }
}
