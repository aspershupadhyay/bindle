// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Finds an amount of money: "₹2,340", "Rs 500", "500 rupees", "₹2k",
//! "1.5 lakh", "$20" (handbook P7). Sums are kept in minor units, never floats.

use crate::tokens::{Span, Token};
use bindle_core::Money;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MoneyHit {
    pub span: Span,
    pub money: Money,
}

/// Finds the first amount of money.
pub(crate) fn find(tokens: &[Token]) -> Option<MoneyHit> {
    (0..tokens.len()).find_map(|i| at(tokens, i))
}

fn at(tokens: &[Token], i: usize) -> Option<MoneyHit> {
    let word = |k: usize| tokens.get(k).map(|t| t.word.as_str()).unwrap_or("");
    let (mut currency, rest) = currency_prefix(word(i));
    // "₹ 500" or "Rs 500": the number is the next word.
    let (number, mut end) = if rest.is_empty() && currency.is_some() {
        (word(i + 1), i + 2)
    } else {
        (rest, i + 1)
    };
    let (number, has_k) = match number.strip_suffix('k') {
        Some(n) => (n, true),
        None => (number, false),
    };
    let mut minor = to_minor(number)?;
    if has_k {
        minor = minor.checked_mul(1_000)?;
    }
    if let Some(times) = multiplier(word(end)) {
        minor = minor.checked_mul(times)?;
        currency = currency.or(Some("INR"));
        end += 1;
    }
    if currency.is_none() && is_rupee_word(word(end)) {
        currency = Some("INR");
        end += 1;
    }
    // "pay 500", "owes me 2k": after a money word, a bare number is rupees.
    if currency.is_none() && after_money_word(tokens, i) && minor >= 1_000 && !is_unit(word(end)) {
        currency = Some("INR");
    }
    // A bare "2k" could be views or steps; only a currency makes it money.
    let currency = currency?;
    Some(MoneyHit {
        span: Span::new(i, end),
        money: Money {
            minor,
            currency: currency.to_string(),
        },
    })
}

/// Whether one of the four words before `i` is about paying.
fn after_money_word(tokens: &[Token], i: usize) -> bool {
    tokens[i.saturating_sub(4)..i].iter().any(|t| {
        matches!(
            t.word.as_str(),
            "pay"
                | "pays"
                | "paying"
                | "repay"
                | "owe"
                | "owes"
                | "lend"
                | "lent"
                | "refund"
                | "transfer"
                | "udhaar"
                | "udhar"
                | "paise"
                | "borrowed"
        )
    })
}

/// Words that make a number something other than money: "pay in 3 days".
fn is_unit(word: &str) -> bool {
    matches!(
        word,
        "day"
            | "days"
            | "hour"
            | "hours"
            | "hrs"
            | "min"
            | "mins"
            | "minutes"
            | "weeks"
            | "week"
            | "months"
            | "pm"
            | "am"
            | "baje"
            | "times"
            | "people"
            | "din"
    )
}

/// Splits a currency marker off the front: "₹500" gives INR and "500".
fn currency_prefix(word: &str) -> (Option<&'static str>, &str) {
    for (marker, code) in [
        ("₹", "INR"),
        ("$", "USD"),
        ("€", "EUR"),
        ("£", "GBP"),
        ("rs.", "INR"),
        ("rs", "INR"),
        ("inr", "INR"),
    ] {
        if let Some(rest) = word.strip_prefix(marker) {
            let is_word_marker = marker.chars().all(|c| c.is_ascii_alphabetic() || c == '.');
            // "rsvp" is not rupees: a word marker must stand alone or touch digits.
            if !is_word_marker || rest.is_empty() || rest.starts_with(|c: char| c.is_ascii_digit())
            {
                return (Some(code), rest);
            }
        }
    }
    (None, word)
}

fn is_rupee_word(word: &str) -> bool {
    matches!(
        word,
        "rs" | "rs." | "inr" | "rupees" | "rupee" | "rupaye" | "rupay" | "rupiya"
    )
}

fn multiplier(word: &str) -> Option<i64> {
    match word {
        "k" | "thousand" | "hazaar" | "hazar" => Some(1_000),
        "lakh" | "lakhs" | "lac" => Some(100_000),
        "crore" | "crores" | "cr" => Some(10_000_000),
        _ => None,
    }
}

/// Reads "2,340" or "2340.5" as minor units: 234000 or 234050.
fn to_minor(text: &str) -> Option<i64> {
    let plain: String = text.chars().filter(|&c| c != ',').collect();
    let (whole, fraction) = plain.split_once('.').unwrap_or((&plain, ""));
    if whole.is_empty() || fraction.len() > 2 || !whole.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let whole: i64 = whole.parse().ok()?;
    let fraction: i64 = match fraction.len() {
        0 => 0,
        1 => fraction.parse::<i64>().ok()? * 10,
        _ => fraction.parse().ok()?,
    };
    whole.checked_mul(100)?.checked_add(fraction)
}
