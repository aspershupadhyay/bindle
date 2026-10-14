// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Turns what people type into structured parts.
//!
//! This is the first slice of handbook P7: relative day words in English and
//! Hinglish. People, amounts and full date phrases come next, each with tests.

use chrono::{Duration, NaiveDate};

/// A day word found in the text, and the date it means.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayMatch {
    /// The word as the person typed it, e.g. "kal".
    pub word: String,
    /// The date it points to.
    pub date: NaiveDate,
    /// True when the word can mean more than one day ("kal" is yesterday or
    /// tomorrow in Hindi). The app shows these with a question mark chip.
    pub ambiguous: bool,
}

/// Finds the first relative day word in `text`, measured from `today`.
///
/// "kal" means tomorrow here, because people jot down promises about the
/// future far more often than the past. It is marked ambiguous so the chip asks.
pub fn find_day(text: &str, today: NaiveDate) -> Option<DayMatch> {
    for raw in text.split_whitespace() {
        let word = raw
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase();
        let found = match word.as_str() {
            "today" | "aaj" | "tonight" => Some((0, false)),
            "tomorrow" | "tmrw" => Some((1, false)),
            "kal" => Some((1, true)),
            "parso" | "parson" => Some((2, true)),
            _ => None,
        };
        if let Some((days, ambiguous)) = found {
            return Some(DayMatch {
                word,
                date: today + Duration::days(days),
                ambiguous,
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 14).expect("valid test date")
    }

    #[test]
    fn kal_means_tomorrow_and_asks_to_confirm() {
        let found = find_day("send reel edit to Rohan kal", today()).expect("a day word");
        assert_eq!(found.date, NaiveDate::from_ymd_opt(2026, 10, 15).unwrap());
        assert!(found.ambiguous);
    }

    #[test]
    fn aaj_means_today() {
        let found = find_day("Aaj bill bharna hai", today()).expect("a day word");
        assert_eq!(found.date, today());
        assert!(!found.ambiguous);
    }

    #[test]
    fn punctuation_around_the_word_is_ignored() {
        let found = find_day("call mom (tomorrow)", today()).expect("a day word");
        assert_eq!(found.word, "tomorrow");
    }

    #[test]
    fn text_without_a_day_word_has_no_match() {
        assert_eq!(find_day("buy a new kalam", today()), None);
    }
}
