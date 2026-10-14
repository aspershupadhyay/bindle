// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! Triage checks on real-shaped emails, for Wednesday 14 October 2026.

use super::*;
use chrono::NaiveDate;

const ME: &str = "sparsh@gmail.com";

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 14, 12, 0, 0).unwrap()
}

fn ist() -> FixedOffset {
    FixedOffset::east_opt(5 * 3600 + 1800).unwrap()
}

/// A raw email. `extra` holds more header lines.
fn raw(
    from: &str,
    to: &str,
    subject: &str,
    date: &str,
    id: &str,
    extra: &str,
    body: &str,
) -> Vec<u8> {
    format!(
        "From: {from}\r\nTo: {to}\r\nSubject: {subject}\r\nDate: {date}\r\nMessage-ID: <{id}>\r\n{extra}Content-Type: text/plain; charset=utf-8\r\n\r\n{body}"
    )
    .into_bytes()
}

fn mail(bytes: Vec<u8>) -> Mail {
    read_mail(&bytes).expect("a readable email")
}

fn priya_brief() -> Mail {
    mail(raw(
        "Priya Sharma <priya@brand.example>",
        "Sparsh <sparsh@gmail.com>",
        "Sponsor brief: Q4 carousel series",
        "Mon, 12 Oct 2026 10:12:00 +0530",
        "brief-1@brand.example",
        "",
        "Hi Sparsh,\r\n\r\nWe would love 4 carousels between 20 October and 10 November.\r\nCould you share your rate card?\r\n\r\nThanks, Priya\r\n",
    ))
}

#[test]
fn an_email_is_read_into_what_matters() {
    let m = priya_brief();
    assert_eq!(m.from_name, "Priya Sharma");
    assert_eq!(m.from_address, "priya@brand.example");
    assert_eq!(m.to, ["sparsh@gmail.com"]);
    assert_eq!(m.subject, "Sponsor brief: Q4 carousel series");
    assert_eq!(m.thread, "brief-1@brand.example");
    assert_eq!(m.at, Utc.with_ymd_and_hms(2026, 10, 12, 4, 42, 0).unwrap());
    assert!(!m.automated);
    assert!(m.text.starts_with("Hi Sparsh,"));
}

#[test]
fn quoted_replies_and_signatures_are_not_new_words() {
    let body = "Sounds great.\nI will send the first concept by Friday.\n\n-- \nSparsh\n";
    assert_eq!(
        new_words(body),
        "Sounds great.\nI will send the first concept by Friday."
    );
    let body = "Done, thanks!\n\nOn Mon, 12 Oct 2026 at 10:12, Priya <priya@brand.example> wrote:\n> We would love";
    assert_eq!(new_words(body), "Done, thanks!");
    assert_eq!(new_words("Yes\n> quoted\nmore"), "Yes");
}

#[test]
fn snippets_are_short_and_cut_at_a_word() {
    let long = "word ".repeat(100);
    let s = snippet(&long);
    assert!(s.ends_with('…'));
    assert!(s.chars().count() <= SNIPPET_CHARS + 1);
    assert_eq!(
        snippet("Hi Sparsh,\n\nWe would love"),
        "Hi Sparsh, We would love"
    );
}

#[test]
fn automated_email_never_needs_a_reply() {
    let newsletter = mail(raw(
        "Substack <news@substack.example>",
        ME,
        "This week in AI",
        "Tue, 13 Oct 2026 08:00:00 +0000",
        "n1@substack.example",
        "List-Unsubscribe: <https://substack.example/u>\r\n",
        "Read more",
    ));
    let receipt = mail(raw(
        "Razorpay <no-reply@razorpay.example>",
        ME,
        "Invoice 0042",
        "Tue, 13 Oct 2026 08:00:00 +0000",
        "r1@razorpay.example",
        "",
        "Payment received",
    ));
    assert!(newsletter.automated && receipt.automated);
    assert!(triage(&[newsletter, receipt], &[], ME, now(), ist(), gmail_link).is_empty());
}

#[test]
fn a_thread_needs_a_reply_until_you_answer() {
    let found = triage(&[priya_brief()], &[], ME, now(), ist(), gmail_link);
    assert_eq!(found.len(), 1);
    let item = &found[0];
    assert_eq!(item.kind, MailKind::Reply);
    assert_eq!(item.person, "Priya Sharma");
    assert_eq!(item.subject, "Sponsor brief: Q4 carousel series");
    assert_eq!(item.id, "reply:brief-1@brand.example");
    assert_eq!(
        item.url,
        "https://mail.google.com/mail/u/0/#search/rfc822msgid%3Abrief-1%40brand.example"
    );
    assert!(item.snippet.starts_with("Hi Sparsh, We would love"));

    // You answer in the same thread: it no longer needs you.
    let answer = mail(raw(
        "Sparsh <sparsh@gmail.com>",
        "Priya Sharma <priya@brand.example>",
        "Re: Sponsor brief: Q4 carousel series",
        "Mon, 12 Oct 2026 14:00:00 +0530",
        "answer-1@gmail.com",
        "In-Reply-To: <brief-1@brand.example>\r\nReferences: <brief-1@brand.example>\r\n",
        "Sounds great. I will send the first concept by Friday.\n\nOn Mon, Priya wrote:\n> We would love",
    ));
    let found = triage(&[priya_brief()], &[answer], ME, now(), ist(), gmail_link);
    assert!(found.iter().all(|f| f.kind != MailKind::Reply));

    // And the promise in your answer is noticed, with Friday read from Monday.
    let promise = found
        .iter()
        .find(|f| f.kind == MailKind::Promise)
        .expect("a noticed promise");
    let noticed = promise.promise.as_ref().unwrap();
    assert_eq!(noticed.title, "Send the first concept");
    assert_eq!(
        noticed.when.map(|w| w.day),
        NaiveDate::from_ymd_opt(2026, 10, 16)
    );
    assert!(noticed.is_deadline);
    assert_eq!(promise.person, "Priya Sharma");
}

#[test]
fn email_you_were_only_copied_on_still_counts_but_mail_to_others_does_not() {
    let to_team = mail(raw(
        "Rohan <rohan@studio.example>",
        "team@studio.example",
        "Office clones video",
        "Tue, 13 Oct 2026 09:00:00 +0000",
        "team-1@studio.example",
        "",
        "Thoughts?",
    ));
    assert!(triage(&[to_team], &[], ME, now(), ist(), gmail_link).is_empty());

    let copied = mail(raw(
        "Rohan <rohan@studio.example>",
        "neha@studio.example",
        "Office clones video",
        "Tue, 13 Oct 2026 09:00:00 +0000",
        "cc-1@studio.example",
        "Cc: Sparsh <SPARSH@gmail.com>\r\n",
        "Thoughts?",
    ));
    assert_eq!(
        triage(&[copied], &[], ME, now(), ist(), gmail_link).len(),
        1
    );
}

#[test]
fn a_question_you_asked_waits_after_a_day() {
    let asked = |date: &str| {
        mail(raw(
            "Sparsh <sparsh@gmail.com>",
            "Neha Kapoor <neha@gymbros.example>",
            "GymBros animation timings",
            date,
            "ask-1@gmail.com",
            "",
            "Could you send the new curl timings?",
        ))
    };
    let found = triage(
        &[],
        &[asked("Mon, 12 Oct 2026 18:00:00 +0530")],
        ME,
        now(),
        ist(),
        gmail_link,
    );
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].kind, MailKind::Waiting);
    assert_eq!(found[0].person, "Neha Kapoor");

    // Asked an hour ago: too soon to call it waiting.
    let recent = triage(
        &[],
        &[asked("Wed, 14 Oct 2026 11:00:00 +0000")],
        ME,
        now(),
        ist(),
        gmail_link,
    );
    assert!(recent.is_empty());
}

#[test]
fn old_threads_are_left_alone() {
    let old = mail(raw(
        "Priya Sharma <priya@brand.example>",
        ME,
        "Last month",
        "Mon, 14 Sep 2026 10:00:00 +0000",
        "old-1@brand.example",
        "",
        "Hello?",
    ));
    assert!(triage(&[old], &[], ME, now(), ist(), gmail_link).is_empty());
}

#[test]
fn promises_need_a_day_and_your_own_words() {
    let at = Utc.with_ymd_and_hms(2026, 10, 12, 8, 0, 0).unwrap();
    assert!(notice_promise("Thanks, see you soon.", at, ist()).is_none());
    assert!(notice_promise("Can you send the invoice by Friday?", at, ist()).is_none());
    assert!(notice_promise("I'll think about it.", at, ist()).is_none());
    let p = notice_promise("Hi! I'll share the deck tomorrow. Cheers", at, ist()).unwrap();
    assert_eq!(p.title, "Share the deck");
    assert_eq!(p.when.map(|w| w.day), NaiveDate::from_ymd_opt(2026, 10, 13));
    let p = notice_promise("Let me check and revert by tomorrow", at, ist()).unwrap();
    assert_eq!(p.title, "Check and revert");
}

#[test]
fn reply_waits_measure_from_their_first_unanswered_email() {
    let follow_up = mail(raw(
        "Priya Sharma <priya@brand.example>",
        "Sparsh <sparsh@gmail.com>",
        "Re: Sponsor brief",
        "Mon, 12 Oct 2026 12:12:00 +0530",
        "brief-2@brand.example",
        "In-Reply-To: <brief-1@brand.example>\r\nReferences: <brief-1@brand.example>\r\n",
        "Any news?",
    ));
    let answer = mail(raw(
        "Sparsh <sparsh@gmail.com>",
        "Priya Sharma <priya@brand.example>",
        "Re: Sponsor brief",
        "Mon, 12 Oct 2026 14:12:00 +0530",
        "answer-1@gmail.com",
        "In-Reply-To: <brief-2@brand.example>\r\nReferences: <brief-1@brand.example> <brief-2@brand.example>\r\n",
        "Yes, sending today.",
    ));
    let waits = reply_waits(&[priya_brief(), follow_up], &[answer], ME);
    assert_eq!(waits.len(), 1, "two emails, one answer: one wait");
    let wait = &waits[0];
    assert_eq!(wait.id, "email:brief-1@brand.example");
    assert_eq!(wait.person, "Priya Sharma");
    assert_eq!((wait.answered_at - wait.asked_at).num_hours(), 4);

    // An email nobody answered yet is not a wait (it is in Inbox instead).
    assert!(reply_waits(&[priya_brief()], &[], ME).is_empty());
}
