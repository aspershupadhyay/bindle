// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Who owes the promise, and how firm it is (handbook P7).
//!
//! When the words do not say, the direction stays empty and Sort asks.

use crate::tokens::{Span, Token};
use bindle_core::Direction;

/// Hinglish future verbs for he, she or they: "bhejega", "degi", "karenge".
pub(crate) fn is_their_verb(word: &str) -> bool {
    word.len() > 4 && (word.ends_with("ega") || word.ends_with("egi") || word.ends_with("enge"))
}

/// Hinglish future verbs for I: "bhejunga", "karungi".
fn is_my_verb(word: &str) -> bool {
    word.len() > 5 && (word.ends_with("unga") || word.ends_with("ungi") || word.ends_with("oonga"))
}

/// Reads the direction from the words. `subject_found` is true when a person
/// was found right before "will", "owes" or a Hinglish "-ega" verb.
pub(crate) fn find(tokens: &[Token], subject_found: bool) -> Option<Direction> {
    let words: Vec<&str> = tokens.iter().map(|t| t.word.as_str()).collect();
    let has = |phrase: &[&str]| words.windows(phrase.len()).any(|w| w == phrase);

    if has(&["just", "for", "me"])
        || has(&["for", "myself"])
        || has(&["apne", "liye"])
        || has(&["khud", "ke", "liye"])
    {
        return Some(Direction::SelfPromise);
    }
    if subject_found
        || has(&["waiting", "on"])
        || has(&["waiting", "for"])
        || words
            .iter()
            .any(|w| matches!(*w, "they'll" | "he'll" | "she'll" | "owes"))
    {
        return Some(Direction::Theirs);
    }
    if words
        .iter()
        .any(|w| matches!(*w, "i'll" | "mujhe" | "i'm") || is_my_verb(w))
        || has(&["i", "will"])
        || has(&["i", "need", "to"])
        || has(&["i", "owe"])
        || has(&["remind", "me"])
    {
        return Some(Direction::Mine);
    }
    None
}

/// Finds words that make a promise soft, such as "sometime" or "kabhi".
/// Returns their span so they can be left out of the title.
pub(crate) fn soft_words(tokens: &[Token]) -> Option<Span> {
    let words: Vec<&str> = tokens.iter().map(|t| t.word.as_str()).collect();
    for (i, w) in words.iter().enumerate() {
        if matches!(*w, "sometime" | "someday" | "kabhi" | "whenever") {
            return Some(Span::new(i, i + 1));
        }
        if words[i..].starts_with(&["one", "of", "these", "days"]) {
            return Some(Span::new(i, i + 4));
        }
    }
    None
}
