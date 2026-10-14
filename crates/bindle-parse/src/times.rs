// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Finds the time of day: "8pm", "8:30 pm", "20:30", "at 8", "shaam 7 baje",
//! "in the evening" (handbook P7).

use crate::tokens::{Span, Token};
use chrono::NaiveTime;

/// The time found in the text. A clock time and a part of the day can sit
/// apart ("7 baje shaam ko"), so there can be two spans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TimeHit {
    pub spans: Vec<Span>,
    pub time: NaiveTime,
    /// "at 8" without morning or evening: a guess the app asks about.
    pub ambiguous: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Half {
    Am,
    Pm,
}

struct Clock {
    span: Span,
    hour: u32,
    minute: u32,
    half: Option<Half>,
}

struct Period {
    span: Span,
    half: Half,
    default_hour: u32,
}

/// Finds the time, skipping tokens already `taken` by other readers.
/// `tonight` is set when the day reader already saw "tonight".
pub(crate) fn find(tokens: &[Token], taken: &[Span], tonight: bool) -> Option<TimeHit> {
    let free = |i: usize| !taken.iter().any(|s| s.contains(i));
    let clock = (0..tokens.len())
        .filter(|&i| free(i))
        .find_map(|i| clock_at(tokens, i));
    let period = (0..tokens.len())
        .filter(|&i| free(i))
        .find_map(|i| period_at(tokens, i))
        .or_else(|| {
            tonight.then_some(Period {
                span: Span::new(0, 0),
                half: Half::Pm,
                default_hour: 21,
            })
        });

    let (spans, time, ambiguous) = match (clock, period) {
        (Some(c), p) => {
            let half = c.half.or(p.as_ref().map(|p| p.half));
            let (hour, ambiguous) = to_24_hour(c.hour, half);
            let mut spans = vec![c.span];
            spans.extend(p.map(|p| p.span));
            (
                spans,
                NaiveTime::from_hms_opt(hour, c.minute, 0)?,
                ambiguous,
            )
        }
        (None, Some(p)) => (
            vec![p.span],
            NaiveTime::from_hms_opt(p.default_hour, 0, 0)?,
            false,
        ),
        (None, None) => return None,
    };
    let spans = spans.into_iter().filter(|s| s.end > s.start).collect();
    Some(TimeHit {
        spans,
        time,
        ambiguous,
    })
}

/// Turns a 12-hour reading into 24 hours. Without am or pm, 1 to 6 is taken
/// as afternoon and 7 to 11 as morning, and the result is marked as a guess.
fn to_24_hour(hour: u32, half: Option<Half>) -> (u32, bool) {
    match half {
        Some(Half::Am) => (hour % 12, false),
        Some(Half::Pm) => (hour % 12 + 12, false),
        None if hour > 12 || hour == 0 || hour == 12 => (hour, false),
        None if hour <= 6 => (hour + 12, true),
        None => (hour, true),
    }
}

fn clock_at(tokens: &[Token], i: usize) -> Option<Clock> {
    let word = tokens[i].word.as_str();
    let next = tokens.get(i + 1).map(|t| t.word.as_str());
    let before = i.checked_sub(1).map(|k| tokens[k].word.as_str());

    let (digits, mut half) = split_half(word);
    let (hour, minute, with_colon) = read_digits(digits)?;
    let mut end = i + 1;
    if half.is_none() {
        if let Some(h) = next.and_then(half_word) {
            half = Some(h);
            end += 1;
        }
    }
    let said_baje = matches!(next, Some("baje" | "bje" | "o'clock"));
    if said_baje {
        end += 1;
    }
    let said_at = before == Some("at");
    // A bare number is only a time when something says so.
    if half.is_none() && !with_colon && !said_baje && !said_at {
        return None;
    }
    if half.is_some() && hour > 12 {
        return None;
    }
    let start = if said_at { i - 1 } else { i };
    Some(Clock {
        span: Span::new(start, end),
        hour,
        minute,
        half,
    })
}

/// Splits "8:30pm" into "8:30" and pm.
fn split_half(word: &str) -> (&str, Option<Half>) {
    for (suffix, half) in [
        ("am", Half::Am),
        ("a.m", Half::Am),
        ("pm", Half::Pm),
        ("p.m", Half::Pm),
    ] {
        if let Some(digits) = word.strip_suffix(suffix) {
            return (digits, Some(half));
        }
    }
    (word, None)
}

fn half_word(word: &str) -> Option<Half> {
    match word {
        "am" | "a.m" => Some(Half::Am),
        "pm" | "p.m" => Some(Half::Pm),
        _ => None,
    }
}

/// Reads "8", "8:30" or "8.30" into hour and minute.
fn read_digits(text: &str) -> Option<(u32, u32, bool)> {
    let (hour, minute, with_colon) = match text.split_once([':', '.']) {
        Some((h, m)) if m.len() == 2 => (h.parse().ok()?, m.parse().ok()?, true),
        Some(_) => return None,
        None => (text.parse().ok()?, 0, false),
    };
    (hour <= 23 && minute <= 59 && !text.is_empty()).then_some((hour, minute, with_colon))
}

fn period_at(tokens: &[Token], i: usize) -> Option<Period> {
    let (half, default_hour) = match tokens[i].word.as_str() {
        "morning" | "subah" | "subeh" => (Half::Am, 9),
        "noon" => (Half::Pm, 12),
        "afternoon" | "dopahar" | "dopehar" => (Half::Pm, 14),
        "evening" | "shaam" | "sham" => (Half::Pm, 18),
        "night" | "raat" | "tonight" => (Half::Pm, 21),
        _ => return None,
    };
    let word = |k: usize| tokens.get(k).map(|t| t.word.as_str());
    let mut start = i;
    if i >= 2 && (word(i - 2), word(i - 1)) == (Some("in"), Some("the")) {
        start = i - 2;
    } else if i >= 1 && matches!(word(i - 1), Some("at" | "this")) {
        start = i - 1;
    }
    let end = if word(i + 1) == Some("ko") {
        i + 2
    } else {
        i + 1
    };
    Some(Period {
        span: Span::new(start, end),
        half,
        default_hour,
    })
}
