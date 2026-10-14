// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! The parsing test table (handbook P7). Every row is something a person
//! might type into Jot down, and what Bindle should understand from it.
//! "Now" is Wednesday 14 October 2026, 6:30 PM.

use bindle_core::{Direction, Money, Strength};
use bindle_parse::{read, Reading};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

fn now() -> NaiveDateTime {
    NaiveDate::from_ymd_opt(2026, 10, 14)
        .unwrap()
        .and_hms_opt(18, 30, 0)
        .unwrap()
}

fn reads(text: &str) -> Reading {
    read(text, now(), &["Priya Sharma".to_string()])
}

/// A day in October 2026 (or the given month).
fn oct(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, day).unwrap()
}

fn at(h: u32, m: u32) -> Option<NaiveTime> {
    NaiveTime::from_hms_opt(h, m, 0)
}

fn inr(rupees: i64) -> Option<Money> {
    Some(Money {
        minor: rupees * 100,
        currency: "INR".into(),
    })
}

/// Checks the title, the day and the time of one row.
#[track_caller]
fn row(text: &str, title: &str, day: Option<NaiveDate>, time: Option<NaiveTime>) -> Reading {
    let reading = reads(text);
    assert_eq!(reading.title, title, "title of {text:?}");
    assert_eq!(reading.when.map(|w| w.day), day, "day of {text:?}");
    assert_eq!(reading.when.and_then(|w| w.time), time, "time of {text:?}");
    reading
}

// Days

#[test]
fn english_day_words() {
    row("call mom today", "Call mom", Some(oct(14)), None);
    row("call mom tomorrow", "Call mom", Some(oct(15)), None);
    row("call mom (tmrw)", "Call mom", Some(oct(15)), None);
    row(
        "renew passport day after tomorrow",
        "Renew passport",
        Some(oct(16)),
        None,
    );
}

#[test]
fn hinglish_day_words() {
    row(
        "Aaj bill bharna hai",
        "Bill bharna hai",
        Some(oct(14)),
        None,
    );
    row("parso dentist", "Dentist", Some(oct(16)), None);
}

#[test]
fn kal_means_tomorrow_and_asks_to_confirm() {
    let r = row(
        "send reel edit to Rohan kal",
        "Send reel edit to Rohan",
        Some(oct(15)),
        None,
    );
    assert!(r.needs_check);
    assert_eq!(r.unsure_words.as_deref(), Some("kal"));
}

#[test]
fn weekdays_mean_the_next_one_and_today_counts() {
    row("gym on friday", "Gym", Some(oct(16)), None);
    row("standup wednesday", "Standup", Some(oct(14)), None);
    row("call bank Mon", "Call bank", Some(oct(19)), None);
    row("shukravar ko party", "Party", Some(oct(16)), None);
    row("this sunday lunch", "Lunch", Some(oct(18)), None);
}

#[test]
fn next_weekday_is_in_next_week_and_asks_to_confirm() {
    let r = row(
        "review deck next friday",
        "Review deck",
        Some(oct(23)),
        None,
    );
    assert!(r.needs_check);
    row("agle somvar rent", "Rent", Some(oct(19)), None);
    row("plan trip next week", "Plan trip", Some(oct(19)), None);
}

#[test]
fn counts_of_days_and_weeks() {
    row("follow up in 3 days", "Follow up", Some(oct(17)), None);
    row("follow up in a week", "Follow up", Some(oct(21)), None);
    row("2 din mein reply", "Reply", Some(oct(16)), None);
}

#[test]
fn written_dates_are_day_first() {
    row("tax filing 31 oct", "Tax filing", Some(oct(31)), None);
    row("tax filing October 31st", "Tax filing", Some(oct(31)), None);
    row("tax filing 31/10", "Tax filing", Some(oct(31)), None);
    row(
        "visa 5-1-2027",
        "Visa",
        NaiveDate::from_ymd_opt(2027, 1, 5),
        None,
    );
    row(
        "visa 2027-01-05",
        "Visa",
        NaiveDate::from_ymd_opt(2027, 1, 5),
        None,
    );
}

#[test]
fn a_date_that_has_passed_this_year_means_next_year() {
    row(
        "anniversary 2 feb",
        "Anniversary",
        NaiveDate::from_ymd_opt(2027, 2, 2),
        None,
    );
}

#[test]
fn by_and_tak_make_a_deadline() {
    let r = row(
        "submit report by friday",
        "Submit report",
        Some(oct(16)),
        None,
    );
    assert!(r.is_deadline);
    let r = row("report friday tak", "Report", Some(oct(16)), None);
    assert!(r.is_deadline);
    assert!(!reads("gym friday").is_deadline);
}

// Times

#[test]
fn clock_times() {
    row(
        "call mom tomorrow 8pm",
        "Call mom",
        Some(oct(15)),
        at(20, 0),
    );
    row(
        "call mom tomorrow 8:30 pm",
        "Call mom",
        Some(oct(15)),
        at(20, 30),
    );
    row(
        "standup tomorrow 9.15am",
        "Standup",
        Some(oct(15)),
        at(9, 15),
    );
    row("deploy tomorrow 20:30", "Deploy", Some(oct(15)), at(20, 30));
}

#[test]
fn a_time_alone_means_the_next_time_the_clock_shows_it() {
    row("call Aman at 8pm", "Call Aman", Some(oct(14)), at(20, 0));
    row("call Aman at 9am", "Call Aman", Some(oct(15)), at(9, 0));
}

#[test]
fn a_bare_hour_is_a_guess_that_asks() {
    let r = row(
        "call Aman tomorrow at 4",
        "Call Aman",
        Some(oct(15)),
        at(16, 0),
    );
    assert!(r.needs_check);
    let r = row("gym tomorrow at 7", "Gym", Some(oct(15)), at(7, 0));
    assert!(r.needs_check);
}

#[test]
fn parts_of_the_day_set_morning_or_evening() {
    row(
        "kal shaam 7 baje call Papa",
        "Call Papa",
        Some(oct(15)),
        at(19, 0),
    );
    row("kal subah walk", "Walk", Some(oct(15)), at(9, 0));
    row(
        "drinks tomorrow in the evening",
        "Drinks",
        Some(oct(15)),
        at(18, 0),
    );
    row("call Priya tonight", "Call Priya", Some(oct(14)), at(21, 0));
    row("raat ko 10 baje movie", "Movie", Some(oct(14)), at(22, 0));
}

#[test]
fn numbers_that_are_not_times_stay_in_the_title() {
    row("buy 3 notebooks", "Buy 3 notebooks", None, None);
    row("read chapter 8", "Read chapter 8", None, None);
}

// Money

#[test]
fn rupee_amounts() {
    assert_eq!(reads("Pay electricity bill ₹2,340 today").amount, inr(2340));
    assert_eq!(reads("pay maid Rs 500").amount, inr(500));
    assert_eq!(reads("pay maid Rs.500").amount, inr(500));
    assert_eq!(reads("pay maid 500 rupees").amount, inr(500));
    assert_eq!(reads("Rohan owes ₹18k").amount, inr(18_000));
    assert_eq!(reads("car loan 1.5 lakh").amount, inr(150_000));
}

#[test]
fn money_leaves_the_title() {
    row(
        "Pay electricity bill ₹2,340 today",
        "Pay electricity bill",
        Some(oct(14)),
        None,
    );
}

#[test]
fn paise_and_other_currencies() {
    assert_eq!(
        reads("refund ₹99.50").amount,
        Some(Money {
            minor: 9950,
            currency: "INR".into()
        })
    );
    assert_eq!(
        reads("domain renewal $12").amount,
        Some(Money {
            minor: 1200,
            currency: "USD".into()
        })
    );
}

#[test]
fn numbers_without_a_currency_are_not_money() {
    assert_eq!(reads("hit 2k subscribers").amount, None);
    assert_eq!(reads("rsvp to the wedding").amount, None);
}

// People and direction

#[test]
fn a_name_after_to_or_for_is_a_person() {
    assert_eq!(
        reads("send reel edit to Rohan kal").person.as_deref(),
        Some("Rohan")
    );
    assert_eq!(reads("call Mom").person.as_deref(), Some("Mom"));
    assert_eq!(
        reads("ping @aman about thumbnails").person.as_deref(),
        Some("Aman")
    );
    assert_eq!(
        reads("Rohan ko payment bhejna").person.as_deref(),
        Some("Rohan")
    );
}

#[test]
fn people_already_in_bindle_are_found_in_any_case() {
    assert_eq!(
        reads("reply priya sharma about the brief")
            .person
            .as_deref(),
        Some("Priya Sharma")
    );
}

#[test]
fn a_capital_at_the_start_is_not_taken_for_a_name() {
    assert_eq!(reads("Pay electricity bill").person, None);
    assert_eq!(reads("Book Diwali train tickets").person, None);
}

#[test]
fn someone_who_will_do_it_owes_it() {
    let r = reads("Rohan will pay the invoice by friday");
    assert_eq!(r.person.as_deref(), Some("Rohan"));
    assert_eq!(r.direction, Some(Direction::Theirs));
    assert_eq!(
        reads("Aman thumbnails bhejega").direction,
        Some(Direction::Theirs)
    );
    assert_eq!(
        reads("waiting on Priya for the contract").direction,
        Some(Direction::Theirs)
    );
}

#[test]
fn i_will_is_mine_and_just_for_me_is_self() {
    assert_eq!(reads("I'll send the deck").direction, Some(Direction::Mine));
    assert_eq!(reads("I’ll send the deck").direction, Some(Direction::Mine));
    assert_eq!(
        reads("main kal deck bhejunga").direction,
        Some(Direction::Mine)
    );
    assert_eq!(
        reads("run 5k just for me").direction,
        Some(Direction::SelfPromise)
    );
}

#[test]
fn without_clear_words_the_direction_waits_for_sort() {
    assert_eq!(reads("send reel edit to Rohan kal").direction, None);
}

// Strength and tidy titles

#[test]
fn sometime_and_kabhi_make_a_soft_promise() {
    let r = row(
        "catch up with Aman sometime",
        "Catch up with Aman",
        None,
        None,
    );
    assert_eq!(r.strength, Strength::Soft);
    assert_eq!(reads("kabhi coffee with Priya").strength, Strength::Soft);
    assert_eq!(reads("coffee with Priya").strength, Strength::Firm);
}

#[test]
fn text_that_is_only_a_day_is_kept_as_typed() {
    row("tomorrow", "tomorrow", Some(oct(15)), None);
}

#[test]
fn text_without_anything_to_find_is_left_alone() {
    let r = row("buy a new kalam", "Buy a new kalam", None, None);
    assert_eq!(r.person, None);
    assert_eq!(r.amount, None);
    assert!(!r.needs_check);
}

#[test]
fn odd_input_never_panics() {
    let odd = [
        "",
        "   ",
        "₹",
        "rs",
        "at",
        "at 99",
        "12:",
        ":30",
        "25:00pm",
        "@",
        "@@",
        "next",
        "in",
        "in 0 days",
        "31/02",
        "99/99/99",
        "0/0",
        "2026-13-40",
        "₹99999999999999999999",
        "₹1.999",
        "1.5.5 lakh",
        "मंगलवार को बैठक",
        "३१ अक्टूबर",
        "😀 kal 😀",
        "(((",
        "tak",
        "by",
        "Feb 30",
        "kal ko",
        "ko",
    ];
    for text in odd {
        let _ = reads(text);
    }
}
