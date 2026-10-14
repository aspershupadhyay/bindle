// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Finds the day a promise is about, in English and Hinglish (handbook P7).

use crate::tokens::{Span, Token};
use chrono::{Datelike, Duration, NaiveDate, Weekday};

/// The day found in the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DayHit {
    pub span: Span,
    pub date: NaiveDate,
    /// The words could mean another day too ("kal", "next Friday").
    pub ambiguous: bool,
    /// Said with "by", "before" or "tak": a deadline, not a plan.
    pub deadline: bool,
    /// "tonight" also says when in the day.
    pub tonight: bool,
}

/// Finds the first day phrase in `tokens`, measured from `today`.
pub(crate) fn find(tokens: &[Token], today: NaiveDate) -> Option<DayHit> {
    (0..tokens.len()).find_map(|i| at(tokens, i, today).map(|hit| widen(tokens, hit)))
}

/// Tries every day pattern starting at token `i`, longest first.
fn at(tokens: &[Token], i: usize, today: NaiveDate) -> Option<DayHit> {
    let w = |k: usize| tokens.get(i + k).map(|t| t.word.as_str()).unwrap_or("");
    let hit = |len: usize, date: NaiveDate, ambiguous: bool| DayHit {
        span: Span::new(i, i + len),
        date,
        ambiguous,
        deadline: false,
        tonight: false,
    };

    if (w(0), w(1), w(2)) == ("day", "after", "tomorrow") {
        return Some(hit(3, today + Duration::days(2), false));
    }
    if let Some(days) = count_of_days(w(0), w(1), w(2)) {
        return Some(hit(3, today + Duration::days(days), false));
    }
    if matches!(w(0), "next" | "agle" | "agla") {
        if matches!(w(1), "week" | "hafte" | "hafta") {
            return Some(hit(2, next_week(today, Weekday::Mon), false));
        }
        if let Some(day) = weekday(w(1)) {
            return Some(hit(2, next_week(today, day), true));
        }
    }
    if matches!(w(0), "this" | "is") {
        if let Some(day) = weekday(w(1)) {
            return Some(hit(2, upcoming(today, day), false));
        }
        if w(1) == "weekend" {
            return Some(hit(2, upcoming(today, Weekday::Sat), false));
        }
    }
    if let Some((len, date)) = written_date(tokens, i, today) {
        return Some(hit(len, date, false));
    }
    let one = |date, ambiguous| Some(hit(1, date, ambiguous));
    match w(0) {
        "today" | "aaj" => one(today, false),
        "tonight" => Some(DayHit {
            tonight: true,
            ..hit(1, today, false)
        }),
        "tomorrow" | "tmrw" | "tmr" => one(today + Duration::days(1), false),
        "kal" => one(today + Duration::days(1), true),
        "parso" | "parson" => one(today + Duration::days(2), true),
        "weekend" => one(upcoming(today, Weekday::Sat), false),
        word => weekday(word).and_then(|day| one(upcoming(today, day), false)),
    }
}

/// Pulls in the small words around a day: "on Friday", "by Friday",
/// "Friday tak", "shukravar ko".
fn widen(tokens: &[Token], mut hit: DayHit) -> DayHit {
    let before = hit
        .span
        .start
        .checked_sub(1)
        .map(|i| tokens[i].word.as_str());
    match before {
        Some("on") => hit.span.start -= 1,
        Some("by" | "before" | "until" | "till") => {
            hit.span.start -= 1;
            hit.deadline = true;
        }
        _ => {}
    }
    let after = |k: usize| tokens.get(hit.span.end + k).map(|t| t.word.as_str());
    if after(0) == Some("ko") && hit.span.start == hit.span.end - 1 {
        hit.span.end += 1;
    } else if after(0) == Some("tak") {
        hit.span.end += 1;
        hit.deadline = true;
    } else if (after(0), after(1)) == (Some("se"), Some("pehle")) {
        hit.span.end += 2;
        hit.deadline = true;
    }
    hit
}

/// "in 3 days", "in 2 weeks", "3 din mein", "2 hafte mein".
fn count_of_days(a: &str, b: &str, c: &str) -> Option<i64> {
    let unit = |word: &str| match word {
        "day" | "days" | "din" => Some(1),
        "week" | "weeks" | "hafte" | "hafta" => Some(7),
        _ => None,
    };
    let number = |word: &str| match word {
        "a" | "one" | "ek" => Some(1),
        "two" | "do" => Some(2),
        "three" | "teen" => Some(3),
        _ => word.parse::<i64>().ok().filter(|n| (1..=365).contains(n)),
    };
    if a == "in" {
        return Some(number(b)? * unit(c)?);
    }
    if matches!(c, "mein" | "me" | "main" | "mai") {
        return Some(number(a)? * unit(b)?);
    }
    None
}

/// English and Hindi weekday names. Hindi names need the "-var" ending, so
/// common words like "shani" or "guru" are not taken for days, and "sat" and
/// "sun" are left out because they are everyday English words.
fn weekday(word: &str) -> Option<Weekday> {
    use Weekday::*;
    Some(match word {
        "monday" | "mon" | "somvar" | "somvaar" => Mon,
        "tuesday" | "tue" | "tues" | "mangalvar" | "mangalvaar" => Tue,
        "wednesday" | "wed" | "budhvar" | "budhvaar" => Wed,
        "thursday" | "thu" | "thurs" | "guruvar" | "guruvaar" => Thu,
        "friday" | "fri" | "shukravar" | "shukravaar" => Fri,
        "saturday" | "shanivar" | "shanivaar" => Sat,
        "sunday" | "ravivar" | "ravivaar" | "itvaar" | "itwar" => Sun,
        _ => return None,
    })
}

/// The next `day`, counting today: said on a Friday, "Friday" means today.
fn upcoming(today: NaiveDate, day: Weekday) -> NaiveDate {
    let ahead = (7 + day.num_days_from_monday() - today.weekday().num_days_from_monday()) % 7;
    today + Duration::days(ahead as i64)
}

/// `day` in the week after this one (weeks start on Monday).
fn next_week(today: NaiveDate, day: Weekday) -> NaiveDate {
    let monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    monday + Duration::days(7 + day.num_days_from_monday() as i64)
}

/// "14 Oct", "14th October", "Oct 14", "14/10", "14-10-2026", "2026-10-14".
/// Numbers are day first, as written in India. A date without a year that has
/// already passed this year means next year.
fn written_date(tokens: &[Token], i: usize, today: NaiveDate) -> Option<(usize, NaiveDate)> {
    let w = |k: usize| tokens.get(i + k).map(|t| t.word.as_str()).unwrap_or("");
    if let Some(date) = numeric_date(w(0), today) {
        return Some((1, date));
    }
    let (len, day, month) = if let (Some(d), Some(m)) = (day_number(w(0)), month(w(1))) {
        (2, d, m)
    } else if let (Some(m), Some(d)) = (month(w(0)), day_number(w(1))) {
        (2, d, m)
    } else {
        return None;
    };
    if let Some(year) = year_number(w(len)) {
        return NaiveDate::from_ymd_opt(year, month, day).map(|date| (len + 1, date));
    }
    this_or_next_year(today, month, day).map(|date| (len, date))
}

fn numeric_date(word: &str, today: NaiveDate) -> Option<NaiveDate> {
    if let Ok(date) = NaiveDate::parse_from_str(word, "%Y-%m-%d") {
        return Some(date);
    }
    let parts: Vec<&str> = word.split(['/', '-']).collect();
    let number = |k: usize| parts.get(k)?.parse::<u32>().ok();
    match parts.len() {
        2 => this_or_next_year(today, number(1)?, number(0)?),
        3 => {
            let year = number(2)? as i32;
            let year = if year < 100 { 2000 + year } else { year };
            NaiveDate::from_ymd_opt(year, number(1)?, number(0)?)
        }
        _ => None,
    }
}

fn this_or_next_year(today: NaiveDate, month: u32, day: u32) -> Option<NaiveDate> {
    let date = NaiveDate::from_ymd_opt(today.year(), month, day)?;
    if date >= today {
        Some(date)
    } else {
        NaiveDate::from_ymd_opt(today.year() + 1, month, day)
    }
}

fn day_number(word: &str) -> Option<u32> {
    let digits = word.trim_end_matches(|c: char| c.is_ascii_alphabetic());
    let suffix = &word[digits.len()..];
    if !matches!(suffix, "" | "st" | "nd" | "rd" | "th") {
        return None;
    }
    digits.parse().ok().filter(|d| (1..=31).contains(d))
}

fn year_number(word: &str) -> Option<i32> {
    word.parse().ok().filter(|y| (2000..=2100).contains(y))
}

/// A month by name or by its first three or more letters: "oct", "sept".
fn month(word: &str) -> Option<u32> {
    const NAMES: [&str; 12] = [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ];
    if word.len() < 3 {
        return None;
    }
    NAMES
        .iter()
        .position(|name| name.starts_with(word))
        .map(|i| i as u32 + 1)
}
