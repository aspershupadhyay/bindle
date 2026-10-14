// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Reading email for what needs you, from any mailbox (handbook P14).
//!
//! Given recent emails from the inbox and from sent mail, [`triage`] finds:
//! threads where someone wrote to you last and is waiting on your reply,
//! threads where you asked something and nobody has answered, and promises
//! you wrote ("I'll send it by Friday"), read the way Jot down reads a line.
//! Automated email (newsletters, receipts, no-reply senders) never counts.

use bindle_store::insights::{WaitKind, WaitSample};
use bindle_store::mail::{FoundMail, MailKind, NoticedPromise};
use chrono::{DateTime, Duration, FixedOffset, TimeZone, Utc};
use mail_parser::{HeaderValue, MessageParser};
use std::collections::HashMap;

/// How far back a thread can still need you.
pub const LOOK_BACK_DAYS: i64 = 14;
/// How far back a promise in sent mail is still worth noticing.
pub const PROMISE_DAYS: i64 = 7;
/// A question counts as waiting once a day has passed without an answer.
const WAITING_AFTER_HOURS: i64 = 24;
const SNIPPET_CHARS: usize = 280;

/// One email, as triage needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mail {
    pub message_id: String,
    /// The first message of its thread, from `References` or `In-Reply-To`.
    pub thread: String,
    pub from_name: String,
    pub from_address: String,
    /// Everyone it was sent to, To and Cc, lower case.
    pub to: Vec<String>,
    /// Display name of the first person it was sent to.
    pub to_name: String,
    pub to_address: String,
    pub subject: String,
    pub at: DateTime<Utc>,
    /// A newsletter, receipt, alert or other email no person wrote.
    pub automated: bool,
    /// The new words in it, without quoted earlier emails or a signature.
    pub text: String,
}

/// Reads a raw email. `None` when it has no sender or date.
pub fn read_mail(raw: &[u8]) -> Option<Mail> {
    let message = MessageParser::default().parse(raw)?;
    let from = message.from()?.first()?;
    let from_address = from.address()?.to_lowercase();
    let at = Utc
        .timestamp_opt(message.date()?.to_timestamp(), 0)
        .single()?;
    let message_id = message
        .message_id()
        .map(str::to_string)
        .unwrap_or_else(|| format!("{}-{}", from_address, at.timestamp()));
    let references = text_list(message.references());
    let in_reply_to = text_list(message.in_reply_to());
    let thread = references
        .first()
        .or(in_reply_to.first())
        .cloned()
        .unwrap_or_else(|| message_id.clone());
    let mut to = Vec::new();
    let mut first_to = None;
    for list in [message.to(), message.cc()].into_iter().flatten() {
        for addr in list.iter() {
            if let Some(address) = addr.address() {
                if first_to.is_none() {
                    first_to = Some((
                        addr.name().unwrap_or(address).to_string(),
                        address.to_lowercase(),
                    ));
                }
                to.push(address.to_lowercase());
            }
        }
    }
    let (to_name, to_address) = first_to.unwrap_or_default();
    let header = |name: &str| message.header_raw(name).is_some();
    let automated = header("List-Unsubscribe")
        || header("List-Id")
        || message
            .header_raw("Auto-Submitted")
            .is_some_and(|v| !v.trim().eq_ignore_ascii_case("no"))
        || message
            .header_raw("Precedence")
            .is_some_and(|v| matches!(v.trim().to_lowercase().as_str(), "bulk" | "list" | "junk"))
        || is_robot_address(&from_address);
    let body = message
        .body_text(0)
        .map(|t| t.into_owned())
        .unwrap_or_default();
    Some(Mail {
        message_id,
        thread,
        from_name: from.name().unwrap_or(&from_address).trim().to_string(),
        from_address,
        to,
        to_name: to_name.trim().to_string(),
        to_address,
        subject: message
            .subject()
            .unwrap_or("(no subject)")
            .trim()
            .to_string(),
        at,
        automated,
        text: new_words(&body),
    })
}

fn text_list(value: &HeaderValue) -> Vec<String> {
    match value {
        HeaderValue::Text(t) => vec![t.to_string()],
        HeaderValue::TextList(list) => list.iter().map(|t| t.to_string()).collect(),
        _ => Vec::new(),
    }
}

/// "noreply@", "notifications@", "mailer-daemon@" and the like.
fn is_robot_address(address: &str) -> bool {
    let local = address.split('@').next().unwrap_or("");
    let local = local.replace(['-', '_', '.'], "");
    [
        "noreply",
        "donotreply",
        "notification",
        "notify",
        "mailerdaemon",
        "postmaster",
        "bounce",
        "alerts",
        "newsletter",
    ]
    .iter()
    .any(|robot| local.contains(robot))
}

/// The words someone just wrote: stops at the quoted earlier email ("On
/// Mon, Priya wrote:", lines starting with ">", "Original Message") or a
/// signature ("-- "), and joins what is left into one tidy text.
pub fn new_words(body: &str) -> String {
    let mut kept = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();
        let quote_intro = lower.starts_with("on ") && lower.ends_with("wrote:");
        if trimmed.starts_with('>')
            || quote_intro
            || lower.contains("original message")
            || lower.starts_with("from:") && !kept.is_empty()
            || line == "-- "
            || trimmed == "--"
        {
            break;
        }
        kept.push(trimmed);
    }
    let text = kept.join("\n");
    // Collapse runs of blank lines.
    let mut tidy = String::new();
    for paragraph in text.split("\n\n").map(str::trim).filter(|p| !p.is_empty()) {
        if !tidy.is_empty() {
            tidy.push_str("\n\n");
        }
        tidy.push_str(paragraph);
    }
    tidy
}

/// The first few lines, on one line, cut at a word.
pub fn snippet(text: &str) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= SNIPPET_CHARS {
        return flat;
    }
    let cut: String = flat.chars().take(SNIPPET_CHARS).collect();
    match cut.rfind(' ') {
        Some(space) => format!("{}…", &cut[..space]),
        None => format!("{cut}…"),
    }
}

/// Opens one email in Gmail's web page, by its Message-ID.
pub fn gmail_link(message_id: &str) -> String {
    let id = message_id.trim_matches(['<', '>']);
    let mut encoded = String::new();
    for byte in id.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'-' | b'_' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    format!("https://mail.google.com/mail/u/0/#search/rfc822msgid%3A{encoded}")
}

/// Finds what needs you in `inbox` and `sent`, as of `now`. `me` is your
/// address; `link` turns a Message-ID into a link that opens it; `zone` is
/// the person's time zone, for reading days such as "Friday".
pub fn triage(
    inbox: &[Mail],
    sent: &[Mail],
    me: &str,
    now: DateTime<Utc>,
    zone: FixedOffset,
    link: fn(&str) -> String,
) -> Vec<FoundMail> {
    let me = me.to_lowercase();
    let since = now - Duration::days(LOOK_BACK_DAYS);
    let mut threads: HashMap<&str, Vec<&Mail>> = HashMap::new();
    for mail in inbox.iter().chain(sent).filter(|m| m.at >= since) {
        threads.entry(mail.thread.as_str()).or_default().push(mail);
    }
    let mut found = Vec::new();
    for (thread, mut mails) in threads {
        mails.sort_by_key(|m| m.at);
        mails.dedup_by(|a, b| a.message_id == b.message_id);
        let Some(latest) = mails.last() else { continue };
        let mine = latest.from_address == me;
        if !mine && !latest.automated && latest.to.contains(&me) {
            found.push(FoundMail {
                id: format!("reply:{thread}"),
                kind: MailKind::Reply,
                thread: thread.to_string(),
                subject: latest.subject.clone(),
                person: latest.from_name.clone(),
                address: latest.from_address.clone(),
                at: latest.at.to_rfc3339(),
                snippet: snippet(&latest.text),
                url: link(&latest.message_id),
                promise: None,
            });
        } else if mine
            && latest.text.contains('?')
            && !latest.to_address.is_empty()
            && latest.to_address != me
            && now - latest.at >= Duration::hours(WAITING_AFTER_HOURS)
        {
            found.push(FoundMail {
                id: format!("waiting:{thread}"),
                kind: MailKind::Waiting,
                thread: thread.to_string(),
                subject: latest.subject.clone(),
                person: latest.to_name.clone(),
                address: latest.to_address.clone(),
                at: latest.at.to_rfc3339(),
                snippet: snippet(&latest.text),
                url: link(&latest.message_id),
                promise: None,
            });
        }
    }
    let promise_since = now - Duration::days(PROMISE_DAYS);
    for mail in sent
        .iter()
        .filter(|m| m.at >= promise_since && m.from_address == me)
    {
        if let Some(promise) = notice_promise(&mail.text, mail.at, zone) {
            found.push(FoundMail {
                id: format!("promise:{}", mail.message_id),
                kind: MailKind::Promise,
                thread: mail.thread.clone(),
                subject: mail.subject.clone(),
                person: mail.to_name.clone(),
                address: mail.to_address.clone(),
                at: mail.at.to_rfc3339(),
                snippet: snippet(&mail.text),
                url: link(&mail.message_id),
                promise: Some(promise),
            });
        }
    }
    found.sort_by(|a, b| a.at.cmp(&b.at).then_with(|| a.id.cmp(&b.id)));
    found
}

/// How long people waited for your replies: in each thread, from the first
/// email someone wrote you after your last word, to your next email.
/// Automated email and email not sent to you are not questions.
pub fn reply_waits(inbox: &[Mail], sent: &[Mail], me: &str) -> Vec<WaitSample> {
    let me = me.to_lowercase();
    let mut threads: HashMap<&str, Vec<&Mail>> = HashMap::new();
    for mail in inbox.iter().chain(sent) {
        threads.entry(mail.thread.as_str()).or_default().push(mail);
    }
    let mut samples = Vec::new();
    for mut mails in threads.into_values() {
        mails.sort_by_key(|m| m.at);
        mails.dedup_by(|a, b| a.message_id == b.message_id);
        let mut asked: Option<&Mail> = None;
        for mail in mails {
            if mail.from_address == me {
                if let Some(question) = asked.take() {
                    samples.push(WaitSample {
                        id: format!("email:{}", question.message_id),
                        kind: WaitKind::EmailReply,
                        person: question.from_name.clone(),
                        asked_at: question.at,
                        answered_at: mail.at,
                    });
                }
            } else if asked.is_none() && !mail.automated && mail.to.contains(&me) {
                asked = Some(mail);
            }
        }
    }
    samples.sort_by_key(|s| s.answered_at);
    samples
}

/// The first promise in something you wrote at `at`: a sentence that says
/// you will do something, with a day ("I'll send the concept by Friday").
/// Days are read from when it was written, in `zone`, not from today.
pub fn notice_promise(text: &str, at: DateTime<Utc>, zone: FixedOffset) -> Option<NoticedPromise> {
    let written = at.with_timezone(&zone).naive_local();
    for sentence in sentences(text) {
        if sentence.ends_with('?') {
            continue;
        }
        let reading = bindle_parse::read(&sentence, written, &[]);
        let lower = sentence.to_lowercase();
        let promising = reading.direction == Some(bindle_core::Direction::Mine)
            || lower.starts_with("will ")
            || lower.starts_with("let me ");
        if promising && reading.when.is_some() {
            return Some(NoticedPromise {
                title: plain_title(&reading.title),
                when: reading.when,
                is_deadline: reading.is_deadline,
            });
        }
    }
    None
}

/// Splits text into sentences at ". ", "! ", "? " and line ends.
fn sentences(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\n' {
            push_sentence(&mut out, &mut current);
            continue;
        }
        current.push(c);
        if matches!(c, '.' | '!' | '?') && chars.peek().is_none_or(|n| n.is_whitespace()) {
            push_sentence(&mut out, &mut current);
        }
    }
    push_sentence(&mut out, &mut current);
    out
}

fn push_sentence(out: &mut Vec<String>, current: &mut String) {
    let sentence = current
        .trim()
        .trim_end_matches(['.', '!'])
        .trim()
        .to_string();
    if !sentence.is_empty() {
        out.push(sentence);
    }
    current.clear();
}

/// "I will send the first concept" becomes "Send the first concept".
fn plain_title(title: &str) -> String {
    let trimmed = title.trim().trim_end_matches(['.', ',', ' ']);
    let lower = trimmed.to_lowercase();
    let rest = [
        "i will ",
        "i'll ",
        "i’ll ",
        "i am going to ",
        "i'm going to ",
        "will ",
        "let me ",
        "i shall ",
    ]
    .iter()
    .find_map(|start| lower.starts_with(start).then(|| &trimmed[start.len()..]))
    .unwrap_or(trimmed)
    .trim();
    let mut chars = rest.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => trimmed.to_string(),
    }
}

#[cfg(test)]
#[path = "mail_tests.rs"]
mod tests;
