// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Finds the person a promise is with (handbook P7).
//!
//! People already in Bindle are matched anywhere, in any case. A new name is
//! only taken when the words around it make it clear: "to Rohan", "@rohan",
//! "Rohan ko", "Rohan will". Guessing wrong is worse than asking in Sort.

use crate::tokens::{Span, Token};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PersonHit {
    pub span: Span,
    /// The name as it should be shown, for example "Rohan".
    pub name: String,
    /// The name came before "will", "owes" or a Hinglish "-ega" verb, so the
    /// person is the one making the promise.
    pub is_subject: bool,
}

/// Small words that point at the next word being a person.
const POINTERS: &[&str] = &[
    "to", "for", "with", "from", "call", "ask", "tell", "pay", "text", "message", "email", "ping",
    "remind", "meet", "thank",
];

/// Hinglish words that come after a name: "Rohan ko", "Priya se", "Aman ka".
const AFTER_NAME: &[&str] = &["ko", "se", "ka", "ki", "ke", "ne"];

/// Words that look like names at the start of a sentence but never are.
const NOT_NAMES: &[&str] = &[
    "i", "me", "my", "we", "you", "they", "he", "she", "it", "the", "a", "an",
];

/// Finds the person, skipping tokens already `taken`. `known` holds the names
/// of people already in Bindle.
pub(crate) fn find(tokens: &[Token], taken: &[Span], known: &[String]) -> Option<PersonHit> {
    let free = |i: usize| !taken.iter().any(|s| s.contains(i));
    known_person(tokens, &free, known).or_else(|| new_person(tokens, &free))
}

fn known_person(
    tokens: &[Token],
    free: &dyn Fn(usize) -> bool,
    known: &[String],
) -> Option<PersonHit> {
    for name in known {
        let words: Vec<String> = name.split_whitespace().map(str::to_lowercase).collect();
        if words.is_empty() {
            continue;
        }
        let found = (0..tokens.len()).find(|&i| {
            (0..words.len()).all(|k| {
                free(i + k)
                    && tokens
                        .get(i + k)
                        .map(|t| t.word.as_str().trim_start_matches('@'))
                        == Some(&words[k])
            })
        });
        if let Some(i) = found {
            let end = i + words.len();
            return Some(PersonHit {
                span: Span::new(i, end),
                name: name.clone(),
                is_subject: is_subject(tokens, end),
            });
        }
    }
    None
}

fn new_person(tokens: &[Token], free: &dyn Fn(usize) -> bool) -> Option<PersonHit> {
    (0..tokens.len()).filter(|&i| free(i)).find_map(|i| {
        let token = &tokens[i];
        let hit = |name: &str| {
            Some(PersonHit {
                span: Span::new(i, i + 1),
                name: title_case(name),
                is_subject: is_subject(tokens, i + 1),
            })
        };
        if let Some(handle) = token.raw.strip_prefix('@') {
            return looks_like_name(handle, false)
                .then(|| hit(handle))
                .flatten();
        }
        if !looks_like_name(token.raw, true) {
            return None;
        }
        let before = i.checked_sub(1).map(|k| tokens[k].word.as_str());
        let after = tokens.get(i + 1).map(|t| t.word.as_str());
        let pointed = before.is_some_and(|w| POINTERS.contains(&w));
        let hinglish = after.is_some_and(|w| AFTER_NAME.contains(&w));
        (pointed || hinglish || is_subject(tokens, i + 1))
            .then(|| hit(token.raw))
            .flatten()
    })
}

/// True when the words from `next` on say this person will do something.
fn is_subject(tokens: &[Token], next: usize) -> bool {
    let word = |k: usize| tokens.get(next + k).map(|t| t.word.as_str()).unwrap_or("");
    matches!(word(0), "will" | "owes" | "promised" | "said" | "should")
        || (word(0), word(1)) == ("is", "sending")
        || crate::direction::is_their_verb(word(0))
        || crate::direction::is_their_verb(word(1))
}

/// A name is letters only, starts with a capital (unless written after @),
/// and is not a pronoun or article.
fn looks_like_name(word: &str, needs_capital: bool) -> bool {
    let first = word.chars().next();
    !word.is_empty()
        && word.chars().all(char::is_alphabetic)
        && (!needs_capital || first.is_some_and(char::is_uppercase))
        && !NOT_NAMES.contains(&word.to_lowercase().as_str())
}

fn title_case(word: &str) -> String {
    let mut chars = word.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}
