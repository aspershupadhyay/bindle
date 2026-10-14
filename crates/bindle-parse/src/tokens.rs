// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Splits text into words, keeping where each one sits so it can be cut out
//! of the title later.

/// One word of the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Token<'a> {
    /// The word as typed, without surrounding punctuation: "Rohan", "₹2,340".
    pub raw: &'a str,
    /// The same word in lower case, for matching.
    pub word: String,
    /// Byte offsets of `raw` in the original text.
    pub start: usize,
    pub end: usize,
}

/// Characters that may start a word even though they are not letters.
fn keeps_at_start(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '₹' | '$' | '€' | '£' | '@')
}

/// Splits on whitespace and trims punctuation such as brackets and commas.
pub(crate) fn tokenize(text: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut offset = 0;
    for piece in text.split_whitespace() {
        let at = offset + text[offset..].find(piece).unwrap_or(0);
        offset = at + piece.len();
        let trimmed_start = piece.trim_start_matches(|c: char| !keeps_at_start(c));
        let trimmed = trimmed_start.trim_end_matches(|c: char| !c.is_alphanumeric());
        if trimmed.is_empty() {
            continue;
        }
        let start = at + (piece.len() - trimmed_start.len());
        tokens.push(Token {
            raw: trimmed,
            // Curly apostrophes from phone keyboards match like straight ones.
            word: trimmed.to_lowercase().replace('’', "'"),
            start,
            end: start + trimmed.len(),
        });
    }
    tokens
}

/// A run of tokens, `start..end`, that a reader recognised.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn contains(&self, index: usize) -> bool {
        (self.start..self.end).contains(&index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brackets_and_commas_are_trimmed_but_offsets_point_into_the_text() {
        let text = "call mom (tomorrow), ok";
        let tokens = tokenize(text);
        assert_eq!(tokens[2].word, "tomorrow");
        assert_eq!(&text[tokens[2].start..tokens[2].end], "tomorrow");
    }

    #[test]
    fn currency_symbols_stay_on_the_amount() {
        let tokens = tokenize("pay ₹2,340.");
        assert_eq!(tokens[1].raw, "₹2,340");
    }
}
