// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! What the AI reads: short, plain lines in Bindle's words (handbook P3).
//! Late promises are "running late", never anything harsher.

use bindle_core::{Direction, Money, Promise, State, Strength, When};
use bindle_store::home::HomeView;
use bindle_store::people::{PersonDetail, PersonSummary};
use chrono::NaiveDate;
use std::fmt::Write;

/// Home as text, section by section. Empty sections are left out.
pub fn home(view: &HomeView, today: NaiveDate, name: Option<&str>) -> String {
    let mut out = format!("Bindle Home for {}", today.format("%A %-d %B %Y"));
    if let Some(name) = name {
        let _ = write!(out, ", for {name}");
    }
    out.push_str(".\n");
    let _ = writeln!(
        out,
        "{} of {} things planned today. {}",
        view.today_count,
        view.day_capacity,
        if view.wrapped_up {
            "The day is wrapped up."
        } else {
            ""
        }
    );
    section(&mut out, "Top 3 today", &view.top3, today);
    section(&mut out, "Could go in the top 3", &view.suggestions, today);
    section(
        &mut out,
        "Running late (be kind about these)",
        &view.running_late,
        today,
    );
    let waiting: Vec<Promise> = view
        .waiting_on_others
        .iter()
        .map(|w| w.promise.clone())
        .collect();
    section(&mut out, "Waiting on others", &waiting, today);
    section(&mut out, "Coming up", &view.coming_up, today);
    out.push_str("\nThis week:");
    for day in &view.week {
        let _ = write!(
            out,
            " {} {}{};",
            day.day.format("%a"),
            day.count,
            if day.full { " (full)" } else { "" }
        );
    }
    let _ = write!(
        out,
        "\nKept this week: {}, moved: {}, let go: {}.",
        view.this_week.kept, view.this_week.moved, view.this_week.let_go
    );
    out
}

fn section(out: &mut String, heading: &str, list: &[Promise], today: NaiveDate) {
    if list.is_empty() {
        return;
    }
    let _ = write!(out, "\n{heading}:\n");
    for promise in list {
        let _ = writeln!(out, "{}", line(promise, today));
    }
}

/// A heading and one line per promise, or a note that there are none.
pub fn list(heading: &str, list: &[Promise], today: NaiveDate) -> String {
    if list.is_empty() {
        return format!("{heading}: none.");
    }
    let mut out = format!("{heading} ({}):\n", list.len());
    for promise in list {
        let _ = writeln!(out, "{}", line(promise, today));
    }
    out
}

/// One promise: title, who, when, amount and state, when not open.
pub fn line(p: &Promise, today: NaiveDate) -> String {
    let mut parts: Vec<String> = Vec::new();
    match (p.direction, p.person.as_deref()) {
        (Some(Direction::Mine), Some(who)) => parts.push(format!("you promised {who}")),
        (Some(Direction::Mine), None) => parts.push("you promised someone".into()),
        (Some(Direction::Theirs), Some(who)) => parts.push(format!("{who} owes you")),
        (Some(Direction::Theirs), None) => parts.push("someone owes you".into()),
        (Some(Direction::SelfPromise), _) => parts.push("for yourself".into()),
        (None, Some(who)) => parts.push(format!("with {who}")),
        (None, None) => {}
    }
    if let Some(when) = when_phrase(p, today) {
        parts.push(when);
    }
    if let Some(money) = &p.amount {
        parts.push(money_words(money));
    }
    if p.strength == Strength::Soft {
        parts.push("soft".into());
    }
    if let Some(step) = &p.first_step {
        parts.push(format!("first step: {step}"));
    }
    if p.state != State::Open {
        parts.push(state_word(p.state).to_lowercase());
    }
    if parts.is_empty() {
        format!("- {}", p.title)
    } else {
        format!("- {} ({})", p.title, parts.join("; "))
    }
}

/// "due Fri 17 Oct, 6:00 PM", "today", "was planned for Mon 13 Oct", ...
pub fn when_phrase(p: &Promise, today: NaiveDate) -> Option<String> {
    let (when, deadline) = match (p.due, p.do_on) {
        (Some(due), _) => (due, true),
        (None, Some(on)) => (on, false),
        (None, None) => return None,
    };
    let day = day_words(when, today);
    Some(match (deadline, when.day < today) {
        (true, true) => format!("was due {day}"),
        (true, false) => format!("due {day}"),
        (false, true) => format!("was planned for {day}"),
        (false, false) => format!("planned for {day}"),
    })
}

fn day_words(when: When, today: NaiveDate) -> String {
    let day = match (when.day - today).num_days() {
        0 => "today".to_string(),
        1 => "tomorrow".to_string(),
        -1 => "yesterday".to_string(),
        _ => when.day.format("%a %-d %b").to_string(),
    };
    match when.time {
        Some(t) => format!("{day}, {}", t.format("%-I:%M %p")),
        None => day,
    }
}

fn money_words(m: &Money) -> String {
    let symbol = match m.currency.as_str() {
        "INR" => "₹",
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        other => return format!("{} {other}", units(m.minor)),
    };
    format!("{symbol}{}", units(m.minor))
}

fn units(minor: i64) -> String {
    if minor % 100 == 0 {
        (minor / 100).to_string()
    } else {
        format!("{}.{:02}", minor / 100, minor.abs() % 100)
    }
}

/// The state in Bindle's words.
pub fn state_word(state: State) -> &'static str {
    match state {
        State::Unsorted => "Waiting in Sort",
        State::Open => "Open",
        State::Later => "Later",
        State::Done => "Done",
        State::LetGo => "Let go",
    }
}

/// Everyone, with open counts both ways.
pub fn people(list: &[PersonSummary]) -> String {
    if list.is_empty() {
        return "No people in Bindle yet.".into();
    }
    let mut out = format!("People ({}):\n", list.len());
    for p in list {
        let _ = writeln!(
            out,
            "- {}: you owe {} open, they owe you {} open",
            p.name, p.you_owe, p.they_owe
        );
    }
    out
}

/// Both sides with one person.
pub fn person(d: &PersonDetail, today: NaiveDate) -> String {
    let mut out = d.name.clone();
    if let Some(pronouns) = &d.pronouns {
        let _ = write!(out, " ({pronouns})");
    }
    out.push('\n');
    section(&mut out, &format!("You owe {}", d.name), &d.you_owe, today);
    section(
        &mut out,
        &format!("{} owes you", d.name),
        &d.they_owe,
        today,
    );
    section(&mut out, "Kept in the last 30 days", &d.kept_lately, today);
    if d.you_owe.is_empty() && d.they_owe.is_empty() && d.kept_lately.is_empty() {
        out.push_str("Nothing open either way.");
    }
    out
}
