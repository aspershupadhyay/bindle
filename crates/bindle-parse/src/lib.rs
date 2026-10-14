// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Understands what people type in Jot down (handbook P7).
//!
//! [`read`] takes a line such as "send reel edit to Rohan kal 8pm" and finds
//! the day, the time, an amount, the person, who owes it and how firm it is.
//! Everything it finds is a suggestion: the promise still waits in Sort, where
//! the person confirms it. Words that might mean two things are flagged with
//! `needs_check`, so the app can show them with a question mark.
//!
//! This crate is pure: no clock, no database. Callers pass "now" and the names
//! of people already in Bindle.

mod days;
mod direction;
mod money;
mod people;
mod times;
mod tokens;

use bindle_core::{Direction, Money, Strength, When};
use chrono::{Duration, NaiveDateTime};
use serde::Serialize;
use tokens::Span;

/// What Bindle understood from one line of text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Reading {
    /// The line with the day, time, amount and soft words taken out.
    pub title: String,
    /// When it is planned, or due if `is_deadline`.
    pub when: Option<When>,
    /// Said with "by", "before" or "tak": the day is a deadline.
    pub is_deadline: bool,
    /// Something could mean two things ("kal", "at 8"); Sort should ask.
    pub needs_check: bool,
    /// The words that could mean two things, exactly as typed.
    pub unsure_words: Option<String>,
    pub amount: Option<Money>,
    pub person: Option<String>,
    pub direction: Option<Direction>,
    pub strength: Strength,
}

/// Reads one line of text, measured from `now` (local time).
/// `known_people` are the names of people already in Bindle.
pub fn read(text: &str, now: NaiveDateTime, known_people: &[String]) -> Reading {
    let tokens = tokens::tokenize(text);
    let today = now.date();
    let mut taken: Vec<Span> = Vec::new();

    let day = days::find(&tokens, today);
    taken.extend(day.as_ref().map(|d| d.span));
    let money = money::find(&tokens).filter(|m| !overlaps(&taken, m.span));
    taken.extend(money.as_ref().map(|m| m.span));
    let time = times::find(&tokens, &taken, day.as_ref().is_some_and(|d| d.tonight));
    taken.extend(time.iter().flat_map(|t| t.spans.iter().copied()));
    let soft = direction::soft_words(&tokens).filter(|s| !overlaps(&taken, *s));
    taken.extend(soft);
    let person = people::find(&tokens, &taken, known_people);

    let unsure = match (&day, &time) {
        (Some(d), _) if d.ambiguous => Some(d.span),
        (_, Some(t)) if t.ambiguous => t.spans.first().copied(),
        _ => None,
    };
    let when = match (&day, &time) {
        (Some(d), t) => Some(When {
            day: d.date,
            time: t.as_ref().map(|t| t.time),
        }),
        // A time on its own means the next time the clock shows it.
        (None, Some(t)) => {
            let day = if t.time > now.time() {
                today
            } else {
                today + Duration::days(1)
            };
            Some(When {
                day,
                time: Some(t.time),
            })
        }
        (None, None) => None,
    };

    Reading {
        title: title(text, &tokens, &taken),
        when,
        is_deadline: day.as_ref().is_some_and(|d| d.deadline),
        needs_check: unsure.is_some(),
        unsure_words: unsure.map(|span| words(text, &tokens, span)),
        amount: money.map(|m| m.money),
        direction: direction::find(&tokens, person.as_ref().is_some_and(|p| p.is_subject)),
        person: person.map(|p| p.name),
        strength: if soft.is_some() {
            Strength::Soft
        } else {
            Strength::Firm
        },
    }
}

fn overlaps(taken: &[Span], span: Span) -> bool {
    taken
        .iter()
        .any(|t| span.start < t.end && t.start < span.end)
}

/// The original text of a span of tokens.
fn words(text: &str, tokens: &[tokens::Token], span: Span) -> String {
    text[tokens[span.start].start..tokens[span.end - 1].end].to_string()
}

/// The text without the taken tokens, tidied and starting with a capital.
/// If nothing would be left, the text is kept as typed.
fn title(text: &str, tokens: &[tokens::Token], taken: &[Span]) -> String {
    let mut kept = String::with_capacity(text.len());
    let mut from = 0;
    for (i, token) in tokens.iter().enumerate() {
        if taken.iter().any(|s| s.contains(i)) {
            kept.push_str(&text[from..token.start]);
            kept.push(' ');
            from = token.end;
        }
    }
    kept.push_str(&text[from..]);
    let tidy = kept.split_whitespace().collect::<Vec<_>>().join(" ");
    let tidy = ["( )", "()", "[ ]", "[]"]
        .iter()
        .fold(tidy.replace(" ,", ","), |t, empty| t.replace(empty, ""));
    let tidy = tidy.split_whitespace().collect::<Vec<_>>().join(" ");
    let tidy = tidy.trim().trim_end_matches([',', ';', ':', '-']).trim();
    let mut chars = tidy.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => text.trim().to_string(),
    }
}
