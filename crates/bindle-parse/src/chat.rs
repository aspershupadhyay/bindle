// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Promises hiding in a conversation (handbook P7): a pasted WhatsApp export,
//! a Slack or SMS thread, or an email. Finds what each person said they would
//! do, in English and Hinglish, and who it is for.
//!
//! Three kinds are caught:
//! - you say you will do something: "I'll send the deck by Friday";
//! - someone else says they will: "Rohan: I'll pay you back kal";
//! - someone asks and the other agrees: "can you send the photos?" then "sure".
//!
//! Days are read from when each message was sent, so "kal" said on Monday is
//! Tuesday, whenever the chat is pasted. Nothing here is saved: the caller
//! shows what was found and the person picks what goes to Sort.

use crate::{read, Reading};
use bindle_core::Direction;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

/// One message: who sent it, when, and what it said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub speaker: Option<String>,
    pub at: Option<NaiveDateTime>,
    pub text: String,
}

/// A promise found in the conversation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Caught {
    /// Who said it, as the chat names them.
    pub speaker: Option<String>,
    /// When it was said, if the chat says.
    pub said_at: Option<NaiveDateTime>,
    /// The words that made it a promise, as written.
    pub said: String,
    /// What Bindle understood, with who owes it set from who said it.
    pub reading: Reading,
}

/// Everything found in one paste.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Catch {
    /// Everyone who speaks, most messages first.
    pub speakers: Vec<String>,
    /// Who Bindle took to be you.
    pub me: Option<String>,
    pub found: Vec<Caught>,
}

/// Reads `text` and finds its promises. `me` is the name the chat uses for
/// the person (for example their WhatsApp name, or "You"); when `None`,
/// "You" or "Me" is used if a speaker is called that. Lines with no speaker
/// count as the person's own words, like a pasted email they wrote.
pub fn catch(text: &str, me: Option<&str>, now: NaiveDateTime, known_people: &[String]) -> Catch {
    let messages = messages(text);
    let speakers = speakers(&messages);
    let me = me
        .map(str::to_string)
        .or_else(|| {
            speakers
                .iter()
                .find(|s| matches!(s.to_lowercase().as_str(), "you" | "me"))
                .cloned()
        })
        .filter(|m| !m.trim().is_empty());
    let is_me = |speaker: &Option<String>| match (speaker, &me) {
        (None, _) => true,
        (Some(s), Some(m)) => s.eq_ignore_ascii_case(m),
        (Some(_), None) => false,
    };
    // In a chat between two, "you" is the other one.
    let other = |speaker: &Option<String>| -> Option<String> {
        let others: Vec<&String> = speakers
            .iter()
            .filter(|s| !is_me(&Some((*s).clone())) && Some(*s) != speaker.as_ref())
            .collect();
        match (is_me(speaker), others.as_slice()) {
            (true, [one]) => Some((*one).clone()),
            _ => None,
        }
    };

    let mut found: Vec<Caught> = Vec::new();
    let mut last: Option<&Message> = None;
    for message in &messages {
        let when = message.at.unwrap_or(now);
        let mut said_here = false;
        for sentence in sentences(&message.text) {
            if let Some(promise) = commitment(&sentence) {
                let person = if is_me(&message.speaker) {
                    other(&message.speaker)
                } else {
                    message.speaker.clone()
                };
                let mine = is_me(&message.speaker);
                found.push(caught(
                    message,
                    &sentence,
                    &promise.what,
                    promise.english,
                    mine,
                    person,
                    when,
                    known_people,
                ));
                said_here = true;
            }
        }
        // "Can you send the photos?" … "Sure." The one who agreed owes it.
        if !said_here && agrees(&message.text) {
            if let Some(asked) = last.filter(|l| l.speaker != message.speaker) {
                if let Some(request) = asks(&asked.text) {
                    let mine = is_me(&message.speaker);
                    let person = if mine {
                        asked.speaker.clone()
                    } else {
                        message.speaker.clone()
                    };
                    let said = format!("{} — {}", asked.text.trim(), message.text.trim());
                    let english = !request
                        .split_whitespace()
                        .any(|w| w.ends_with("na") || w == "do");
                    found.push(caught(
                        message,
                        &said,
                        &request,
                        english,
                        mine,
                        person,
                        when,
                        known_people,
                    ));
                }
            }
        }
        last = Some(message);
    }
    // The same promise said twice is kept once, at its latest wording.
    let mut seen = std::collections::HashSet::new();
    found.reverse();
    found.retain(|c| seen.insert(c.reading.title.to_lowercase()));
    found.reverse();
    Catch {
        speakers,
        me,
        found,
    }
}

#[allow(clippy::too_many_arguments)]
fn caught(
    message: &Message,
    said: &str,
    rest: &str,
    english: bool,
    mine: bool,
    person: Option<String>,
    when: NaiveDateTime,
    known_people: &[String],
) -> Caught {
    let mut known = known_people.to_vec();
    known.extend(person.clone());
    let words = match (mine, english) {
        (true, _) => you_to(rest, person.as_deref()),
        (false, true) => format!(
            "{} will {}",
            person.as_deref().unwrap_or("They"),
            you_to_me(rest)
        ),
        // Hinglish keeps its own words: "Meera: sabzi le aaungi".
        (false, false) => format!("{}: {rest}", person.as_deref().unwrap_or("They")),
    };
    let mut reading = read(&words, when, &known);
    reading.direction = Some(if mine {
        Direction::Mine
    } else {
        Direction::Theirs
    });
    // The chat names them exactly; that beats a guess from the words.
    if person.is_some() {
        reading.person = person;
    }
    Caught {
        speaker: message.speaker.clone(),
        said_at: message.at,
        said: said.trim().to_string(),
        reading,
    }
}

/// Splits pasted text into messages. Knows WhatsApp's two export formats,
/// "Name: text" lines, and plain text; a line that starts no message
/// continues the one before.
pub fn messages(text: &str) -> Vec<Message> {
    let mut out: Vec<Message> = Vec::new();
    for raw in text.lines() {
        let line = raw
            .trim_matches(|c: char| c.is_whitespace() || c == '\u{200e}' || c == '\u{200f}')
            .replace('\u{202f}', " ");
        if line.is_empty() {
            continue;
        }
        if let Some(message) = whatsapp(&line).or_else(|| named(&line)) {
            out.push(message);
        } else if let Some(previous) = out.last_mut() {
            previous.text.push('\n');
            previous.text.push_str(&line);
        } else {
            out.push(Message {
                speaker: None,
                at: None,
                text: line,
            });
        }
    }
    // Media and system lines say nothing anyone promised.
    out.retain(|m| {
        let t = m.text.to_lowercase();
        !(t.contains("<media omitted>")
            || t.contains("image omitted")
            || t.contains("messages and calls are end-to-end encrypted")
            || t == "this message was deleted")
    });
    out
}

/// Everyone who speaks, most messages first.
fn speakers(messages: &[Message]) -> Vec<String> {
    let mut counts: Vec<(String, usize)> = Vec::new();
    for name in messages.iter().filter_map(|m| m.speaker.clone()) {
        match counts.iter_mut().find(|(n, _)| *n == name) {
            Some((_, c)) => *c += 1,
            None => counts.push((name, 1)),
        }
    }
    counts.sort_by_key(|c| std::cmp::Reverse(c.1));
    counts.into_iter().map(|(n, _)| n).collect()
}

/// "[14/10/26, 6:30:12 PM] Rohan: text" (iPhone) or
/// "14/10/2026, 18:30 - Rohan: text" (Android).
fn whatsapp(line: &str) -> Option<Message> {
    let (stamp, rest) = if let Some(inner) = line.strip_prefix('[') {
        let (stamp, rest) = inner.split_once(']')?;
        (stamp.trim(), rest.trim_start())
    } else {
        let (stamp, rest) = line.split_once(" - ")?;
        (stamp.trim(), rest)
    };
    let (date, time) = stamp.split_once(',')?;
    let at = date_of(date.trim())?.and_time(time_of(time.trim())?);
    let (speaker, text) = rest.split_once(": ")?;
    Some(Message {
        speaker: Some(speaker.trim().to_string()),
        at: Some(at),
        text: text.trim().to_string(),
    })
}

/// "Rohan: text", when the name looks like a name (a few words, no digits).
fn named(line: &str) -> Option<Message> {
    let (speaker, text) = line.split_once(": ")?;
    let speaker = speaker.trim();
    let looks_like_a_name = !speaker.is_empty()
        && speaker.len() <= 32
        && speaker.split_whitespace().count() <= 4
        && !speaker.chars().any(|c| c.is_ascii_digit())
        && !speaker.to_lowercase().starts_with("re")
        && speaker
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '@');
    looks_like_a_name.then(|| Message {
        speaker: Some(speaker.to_string()),
        at: None,
        text: text.trim().to_string(),
    })
}

/// "14/10/26", "14/10/2026" or "14.10.26": day first, as written in India.
fn date_of(text: &str) -> Option<NaiveDate> {
    let parts: Vec<u32> = text
        .split(['/', '.', '-'])
        .map(|p| p.trim().parse().ok())
        .collect::<Option<_>>()?;
    let [day, month, year] = parts[..] else {
        return None;
    };
    let year = if year < 100 { 2000 + year } else { year };
    NaiveDate::from_ymd_opt(year as i32, month, day)
}

/// "6:30 PM", "6:30:12 pm" or "18:30".
fn time_of(text: &str) -> Option<NaiveTime> {
    let lower = text.to_lowercase();
    let (clock, pm, am) = if let Some(c) = lower.strip_suffix("pm") {
        (c.trim(), true, false)
    } else if let Some(c) = lower.strip_suffix("am") {
        (c.trim(), false, true)
    } else {
        (lower.trim(), false, false)
    };
    let mut parts = clock.split(':').map(|p| p.trim().parse::<u32>().ok());
    let mut hour = parts.next()??;
    let minute = parts.next()??;
    let second = parts.next().flatten().unwrap_or(0);
    if pm && hour < 12 {
        hour += 12;
    }
    if am && hour == 12 {
        hour = 0;
    }
    NaiveTime::from_hms_opt(hour, minute, second)
}

fn sentences(text: &str) -> Vec<String> {
    text.split(['.', '!', '\n', ';'])
        .flat_map(|s| s.split('?'))
        .flat_map(split_promises)
        .map(|s| s.trim().to_string())
        .filter(|s| s.split_whitespace().count() >= 2)
        .collect()
}

/// "I'll send it, and I can share the rest tomorrow": two promises.
fn split_promises(sentence: &str) -> Vec<String> {
    let lower = sentence.to_lowercase();
    let mut cuts = vec![0];
    for joint in [
        ", and i",
        " and i'll",
        " and i’ll",
        " and i will",
        " and i can",
        ", i'll",
        ", i will",
    ] {
        let mut from = 0;
        while let Some(at) = lower[from..].find(joint) {
            let at = from + at;
            // Cut before "i", keeping the opener with the second promise.
            let i = at + joint.find('i').unwrap_or(0);
            cuts.push(i);
            from = at + joint.len();
        }
    }
    cuts.sort_unstable();
    cuts.dedup();
    cuts.push(sentence.len());
    cuts.windows(2)
        .map(|w| {
            let part = sentence[w[0]..w[1]].trim().trim_end_matches([',', ' ']);
            part.strip_suffix(" and")
                .unwrap_or(part)
                .trim_end_matches([',', ' '])
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

/// Words that start a promise, with what follows them.
const OPENERS: &[&str] = &[
    "i'll",
    "i’ll",
    "i will",
    "i'm going to",
    "i’m going to",
    "i am going to",
    "i promise to",
    "i promise",
    "let me",
    "will do",
    "i can",
    "i'll get",
];

/// Fillers before an opener: "ok, I'll…", "sure so I'll…".
const FILLERS: &[&str] = &[
    "ok", "okay", "sure", "yes", "yeah", "yep", "haan", "ha", "alright", "and", "also", "so",
    "cool", "fine", "great", "done", "then", "and then", "but",
];

/// What a sentence commits its speaker to.
struct Commitment {
    what: String,
    /// Said with an English opener ("I'll …"), so "they will …" reads well.
    english: bool,
}

/// When a sentence commits its speaker, the thing they will do.
fn commitment(sentence: &str) -> Option<Commitment> {
    let mut rest = sentence.trim();
    loop {
        let lower = rest.to_lowercase();
        let Some(filler) = FILLERS
            .iter()
            .find(|f| lower.starts_with(*f) && lower[f.len()..].starts_with([',', ' ', '!']))
        else {
            break;
        };
        rest = rest[filler.len()..].trim_start_matches([',', ' ', '!']);
    }
    let lower = rest.to_lowercase();
    // Not a promise: "I'll be late", "I can't", "I'll try".
    if [
        "i'll be ",
        "i’ll be ",
        "i can't",
        "i can’t",
        "i cannot",
        "i'll try",
        "i’ll try",
        "i will not",
        "i won't",
        "i won’t",
    ]
    .iter()
    .any(|no| lower.starts_with(no))
    {
        return None;
    }
    for opener in OPENERS {
        if lower.starts_with(opener) && lower[opener.len()..].starts_with(' ') {
            let what = rest[opener.len()..].trim();
            return (what.split_whitespace().count() >= 1).then(|| Commitment {
                what: what.to_string(),
                english: true,
            });
        }
    }
    // Hinglish: "kal bhej dunga", "main kar deta hoon", "de dungi".
    let words: Vec<&str> = lower.split_whitespace().collect();
    let my_verb = words.iter().any(|w| {
        let w = w.trim_matches(|c: char| !c.is_alphanumeric());
        w.len() > 4 && (w.ends_with("unga") || w.ends_with("ungi") || w.ends_with("oonga"))
    });
    let doing_now = words.windows(2).any(|pair| {
        matches!(
            pair[0],
            "deta" | "deti" | "karta" | "karti" | "bhejta" | "bhejti" | "dekhta" | "dekhti"
        ) && matches!(
            pair[1].trim_matches(|c: char| !c.is_alphanumeric()),
            "hoon" | "hu" | "hun" | "hu."
        )
    });
    (my_verb || doing_now).then(|| {
        // "main" and "mai" are "I": the title starts with what is done.
        let what = ["main ", "mai ", "mein "]
            .iter()
            .find_map(|i| lower.starts_with(i).then(|| rest[i.len()..].to_string()))
            .unwrap_or_else(|| rest.to_string());
        Commitment {
            what,
            english: false,
        }
    })
}

/// "Sure", "ok will do", "haan pakka", "done": agreeing to the last ask.
fn agrees(text: &str) -> bool {
    let lower = text.trim().to_lowercase();
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|w| !w.is_empty())
        .collect();
    !words.is_empty()
        && words.len() <= 5
        && words.iter().all(|w| {
            matches!(
                *w,
                "sure"
                    | "ok"
                    | "okay"
                    | "k"
                    | "yes"
                    | "yeah"
                    | "yep"
                    | "haan"
                    | "ha"
                    | "han"
                    | "done"
                    | "will"
                    | "do"
                    | "pakka"
                    | "theek"
                    | "thik"
                    | "hai"
                    | "bilkul"
                    | "on"
                    | "it"
                    | "sounds"
                    | "good"
                    | "no"
                    | "problem"
                    | "of"
                    | "course"
                    | "absolutely"
                    | "np"
                    | "ji"
                    | "zaroor"
                    | "consider"
                    | "it's"
                    | "its"
            )
        })
        && !words.iter().all(|w| matches!(*w, "no" | "problem"))
}

/// When a message asks someone to do something, what it asks.
fn asks(text: &str) -> Option<String> {
    let lower = text.trim().to_lowercase();
    for opener in [
        "can you ",
        "could you ",
        "will you ",
        "would you ",
        "can u ",
        "could u ",
        "please ",
        "pls ",
        "plz ",
    ] {
        if let Some(at) = lower.find(opener) {
            let start = at + opener.len();
            let what = text.trim()[start..]
                .trim_end_matches(['?', '.', '!', ' '])
                .trim_end_matches(" please")
                .trim_end_matches(" pls");
            return (what.split_whitespace().count() >= 2).then(|| what.to_string());
        }
    }
    // Hinglish: "photos bhej dena", "kal call kar dena please".
    let trimmed = lower
        .trim_end_matches(['?', '.', '!', ' '])
        .trim_end_matches(" please")
        .trim_end_matches(" pls")
        .trim_end_matches(" plz");
    ["dena", "dijiye", "karna", "kar do", "bhej do", "de do"]
        .iter()
        .any(|end| trimmed.ends_with(end))
        .then(|| text.trim()[..trimmed.len()].to_string())
}

/// For your own promise: "send you the deck" becomes "send Rohan the deck".
fn you_to(words: &str, person: Option<&str>) -> String {
    let Some(person) = person else {
        return words.to_string();
    };
    words
        .split(' ')
        .map(|w| match w.to_lowercase().as_str() {
            "you" | "u" => person.to_string(),
            "your" => format!("{person}'s"),
            _ => w.to_string(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// For their promise: "pay you back" becomes "pay me back".
fn you_to_me(words: &str) -> String {
    words
        .split(' ')
        .map(|w| match w.to_lowercase().as_str() {
            "you" | "u" => "me".to_string(),
            "your" => "my".to_string(),
            "yourself" => "myself".to_string(),
            _ => w.to_string(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}
