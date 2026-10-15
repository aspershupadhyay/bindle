// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! The sample Bindle: a made-up person's week, shown the first time Bindle
//! opens so nobody meets an empty screen. It belongs to Aarav, who shares a
//! home with Meera, works on a few open-source projects, writes email, and
//! posts on YouTube.
//!
//! Everything is written through the same functions the app uses, into a
//! database of its own (in memory), so every button works on it and the
//! person's real data is never touched. Dates are counted from `today`, so
//! the sample always looks like this week.

use crate::creator::{self, Channel, CreatorAccount, FoundComment, FoundPost};
use crate::insights::{self, WaitKind, WaitSample};
use crate::lifecycle::{self, Sorting};
use crate::mail::{self, FoundMail, MailAccount, MailKind, NoticedPromise};
use crate::promises::{self, Jot};
use crate::settings::{self, Settings};
use crate::work::{self, FoundItem, GithubAccount, WorkKind};
use crate::{cloud, home, people, StoreError};
use bindle_core::{Direction, Mode, Money, State, Strength, When};
use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, SecondsFormat, Utc, Weekday};
use rusqlite::{params, Connection};

/// The sample person's own account in the shared space.
pub const ME: &str = "acct-aarav";
const PARTNER: &str = "acct-meera";
const SPACE: &str = "sample-home";

fn stamp(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// One promise to keep: what it says, who, when.
struct Keep<'a> {
    title: &'a str,
    direction: Direction,
    person: Option<&'a str>,
    when: Option<NaiveDate>,
    time: Option<(u32, u32)>,
    deadline: bool,
    soft: bool,
    rupees: Option<i64>,
    first_step: Option<&'a str>,
    mode: Mode,
}

impl<'a> Keep<'a> {
    fn new(title: &'a str, direction: Direction) -> Self {
        Self {
            title,
            direction,
            person: None,
            when: None,
            time: None,
            deadline: false,
            soft: false,
            rupees: None,
            first_step: None,
            mode: Mode::Life,
        }
    }
    fn with(mut self, person: &'a str) -> Self {
        self.person = Some(person);
        self
    }
    fn on(mut self, day: NaiveDate) -> Self {
        self.when = Some(day);
        self
    }
    fn by(mut self, day: NaiveDate) -> Self {
        self.when = Some(day);
        self.deadline = true;
        self
    }
    fn at(mut self, hour: u32, minute: u32) -> Self {
        self.time = Some((hour, minute));
        self
    }
    fn soft(mut self) -> Self {
        self.soft = true;
        self
    }
    fn rupees(mut self, rupees: i64) -> Self {
        self.rupees = Some(rupees);
        self
    }
    fn first(mut self, step: &'a str) -> Self {
        self.first_step = Some(step);
        self
    }
    fn work(mut self) -> Self {
        self.mode = Mode::Work;
        self
    }

    /// Jots it down at `at` and keeps it in Sort, as a person would.
    fn save(&self, conn: &mut Connection, at: DateTime<Utc>) -> Result<String, StoreError> {
        let mut jot = Jot::plain(self.title);
        jot.amount = self.rupees.map(|r| Money {
            minor: r * 100,
            currency: "INR".into(),
        });
        let id = promises::jot_down_in(conn, &jot, self.mode, at)?.id;
        let sorting = Sorting {
            title: self.title.into(),
            direction: Some(self.direction),
            strength: if self.soft {
                Strength::Soft
            } else {
                Strength::Firm
            },
            person: self.person.map(Into::into),
            first_step: self.first_step.map(Into::into),
            when: self.when.map(|day| When {
                day,
                time: self
                    .time
                    .and_then(|(h, m)| NaiveTime::from_hms_opt(h, m, 0)),
            }),
            is_deadline: self.deadline,
        };
        lifecycle::sort(conn, &id, &sorting, State::Open, at)?;
        Ok(id)
    }
}

/// The next `weekday` after `today` (a week on when today is that day).
fn next(today: NaiveDate, weekday: Weekday) -> NaiveDate {
    let ahead = (7 + weekday.num_days_from_monday() as i64
        - today.weekday().num_days_from_monday() as i64)
        % 7;
    today + Duration::days(if ahead == 0 { 7 } else { ahead })
}

/// Fills an empty database with the sample Bindle. `today` is the person's
/// local day and `now` the moment it is made.
pub fn seed(conn: &mut Connection, today: NaiveDate, now: DateTime<Utc>) -> Result<(), StoreError> {
    let hours = |h: i64| now - Duration::hours(h);
    let days = |d: i64| now - Duration::days(d);
    let tomorrow = today + Duration::days(1);
    let friday = next(today, Weekday::Fri);
    let saturday = next(today, Weekday::Sat);
    let sunday = next(today, Weekday::Sun);

    settings::save(
        conn,
        &Settings {
            name: Some("Aarav".into()),
            onboarded: true,
            avatar: Some("short".into()),
            ..Settings::default()
        },
    )?;

    // Life: what Aarav owes, what others owe him, and his own.
    let deck = Keep::new("Send the deck to Priya", Direction::Mine)
        .with("Priya")
        .by(friday)
        .first("Export the last three slides")
        .save(conn, days(2))?;
    let kabir = Keep::new("Return Kabir's drill", Direction::Mine)
        .with("Kabir")
        .on(tomorrow)
        .save(conn, days(3))?;
    Keep::new("Call Mummy", Direction::Mine)
        .with("Mummy")
        .on(sunday)
        .at(19, 0)
        .save(conn, days(1))?;
    Keep::new("Renew passport photos", Direction::SelfPromise)
        .by(today - Duration::days(2))
        .first("Find the studio's timings")
        .save(conn, days(9))?;
    Keep::new("Catch up with Ishaan over chai", Direction::Mine)
        .with("Ishaan")
        .soft()
        .save(conn, days(6))?;
    let rohan = Keep::new("Rohan pays back for the dinner", Direction::Theirs)
        .with("Rohan")
        .by(today - Duration::days(1))
        .rupees(500)
        .save(conn, days(5))?;
    home::nudge(conn, &rohan, hours(20))?;
    Keep::new("Neha sends the curl timings", Direction::Theirs)
        .with("Neha")
        .on(tomorrow)
        .save(conn, days(1))?;
    Keep::new("Gym before work", Direction::SelfPromise)
        .on(today)
        .at(7, 0)
        .save(conn, days(1))?;
    for (title, by, done) in [
        ("Send rent to the landlord", "Landlord", days(2)),
        ("Share the trip photos with Kabir", "Kabir", days(1)),
    ] {
        let id = Keep::new(title, Direction::Mine)
            .with(by)
            .on(done.date_naive())
            .save(conn, done - Duration::days(1))?;
        lifecycle::mark_done(conn, &id, done)?;
    }
    home::add_to_top3(conn, today, &deck, now)?;
    home::add_to_top3(conn, today, &kabir, now)?;

    // Waiting in Sort, as they were typed: Sort reads them back.
    let mut dentist = Jot::plain("dentist on the 3rd at 5pm");
    dentist.title = "Dentist".into();
    promises::jot_down(conn, &dentist, hours(1))?;
    let mut photos = Jot::plain("Kabir will send the photos kal");
    photos.title = "Kabir will send the photos".into();
    photos.person = Some("Kabir".into());
    photos.direction = Some(Direction::Theirs);
    photos.when = Some(When {
        day: tomorrow,
        time: None,
    });
    photos.unsure_words = Some("kal".into());
    promises::jot_down(conn, &photos, hours(2))?;

    for (name, avatar, pronouns) in [
        ("Priya", "long", Some("she/her")),
        ("Rohan", "curly", None),
        ("Kabir", "beard", None),
        ("Mummy", "grey", None),
        ("Ishaan", "cap", None),
        ("Neha", "bun", Some("she/her")),
        ("Landlord", "turban", None),
    ] {
        let id = people::find_or_add(conn, name, &stamp(now))?;
        people::set_look(conn, &id, Some(avatar), pronouns, &stamp(now))?;
    }

    ours(conn, today, now, friday, saturday)?;
    github(conn, now)?;
    email(conn, now, friday)?;
    youtube(conn, today, now)?;
    linked_work(conn, today, now)?;
    Ok(())
}

/// Work promises made from what the sources found, spread over the week.
fn linked_work(
    conn: &mut Connection,
    today: NaiveDate,
    now: DateTime<Utc>,
) -> Result<(), StoreError> {
    let monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    let keep =
        |conn: &mut Connection, id: &str, day: NaiveDate, done: bool| -> Result<(), StoreError> {
            let promise = promises::get(conn, id)?;
            let sorting = Sorting {
                title: promise.title.clone(),
                direction: Some(Direction::Mine),
                strength: Strength::Firm,
                person: promise.person.clone(),
                first_step: None,
                when: Some(When { day, time: None }),
                is_deadline: false,
            };
            lifecycle::sort(conn, id, &sorting, State::Open, now - Duration::days(3))?;
            if done {
                lifecycle::mark_done(conn, id, now - Duration::hours(2))?;
            }
            Ok(())
        };
    let review = work::make_promise(conn, "github:r42", now - Duration::days(3))?.id;
    keep(
        conn,
        &review,
        monday + Duration::days(1),
        today > monday + Duration::days(1),
    )?;
    let issue = work::make_promise(conn, "github:i39", now - Duration::days(3))?.id;
    keep(conn, &issue, monday + Duration::days(3), false)?;
    let reply = mail::make_promise(conn, "gmail:priya", now - Duration::days(3))?.id;
    keep(conn, &reply, monday, today > monday)?;
    let comment = creator::make_promise(conn, "c-1", now - Duration::days(3))?.id;
    keep(conn, &comment, monday + Duration::days(4), false)?;
    Ok(())
}

/// A shared home with Meera: what each added, and what each kept.
fn ours(
    conn: &mut Connection,
    today: NaiveDate,
    now: DateTime<Utc>,
    friday: NaiveDate,
    saturday: NaiveDate,
) -> Result<(), StoreError> {
    let days = |d: i64| now - Duration::days(d);
    cloud::save_cloud_account(
        conn,
        &cloud::CloudAccount {
            relay: Some("https://cloud.bindle.example".into()),
            name: Some("Aarav".into()),
            account: Some(ME.into()),
            device: Some("sample-laptop".into()),
            kit_saved_at: Some(stamp(days(3))),
        },
    )?;
    cloud::add_space(
        conn,
        SPACE,
        "Home",
        "https://cloud.bindle.example",
        days(30),
    )?;
    for (account, name, role, joined) in [
        (ME, "Aarav", "owner", days(30)),
        (PARTNER, "Meera", "member", days(29)),
    ] {
        conn.execute(
            "INSERT INTO member (space_id, account_id, display_name, avatar, role, joined_at)
             VALUES (?1, ?2, ?3, NULL, ?4, ?5)",
            params![SPACE, account, name, role, stamp(joined)],
        )?;
    }
    conn.execute(
        "UPDATE sync_cursor SET synced_at = ?2 WHERE space_id = ?1",
        params![SPACE, stamp(now - Duration::minutes(4))],
    )?;

    let shared = |conn: &mut Connection,
                  keep: Keep,
                  by: &str,
                  at: DateTime<Utc>|
     -> Result<String, StoreError> {
        let id = keep.save(conn, at)?;
        cloud::share(conn, &id, Some(SPACE), by)?;
        Ok(id)
    };
    shared(
        conn,
        Keep::new("Pay the electricity bill", Direction::Mine)
            .by(friday)
            .rupees(2340),
        PARTNER,
        days(1),
    )?;
    shared(
        conn,
        Keep::new("Book Diwali train tickets to Lucknow", Direction::Mine)
            .on(saturday)
            .first("Check the Tatkal timing"),
        PARTNER,
        days(2),
    )?;
    shared(
        conn,
        Keep::new("Call the landlord about the leaking tap", Direction::Mine).on(friday),
        ME,
        days(1),
    )?;
    shared(
        conn,
        Keep::new("Mummy's birthday gift", Direction::Mine)
            .on(next(today, Weekday::Tue))
            .rupees(3000),
        ME,
        days(4),
    )?;
    for (title, by, kept_by, ago) in [
        ("Refill the gas cylinder", PARTNER, PARTNER, 0),
        ("Renew the car insurance", ME, ME, 2),
        ("Grocery run for the week", PARTNER, PARTNER, 4),
    ] {
        let id = shared(
            conn,
            Keep::new(title, Direction::Mine).on(today - Duration::days(ago)),
            by,
            days(ago + 1),
        )?;
        lifecycle::mark_done(conn, &id, days(ago) - Duration::hours(1))?;
        conn.execute(
            "UPDATE promise SET done_by = ?2 WHERE id = ?1",
            params![id, kept_by],
        )?;
    }
    Ok(())
}

/// GitHub: two reviews owed, two pull requests waiting, three issues.
fn github(conn: &mut Connection, now: DateTime<Utc>) -> Result<(), StoreError> {
    let at = |h: i64| stamp(now - Duration::hours(h));
    let item = |id: &str,
                kind: WorkKind,
                repo: &str,
                number: u32,
                title: &str,
                author: &str,
                draft: bool,
                labels: &[&str],
                age: i64| FoundItem {
        id: format!("github:{id}"),
        kind,
        repo: repo.into(),
        number,
        title: title.into(),
        author: author.into(),
        url: format!("https://github.com/{repo}/pull/{number}"),
        draft,
        labels: labels.iter().map(|l| l.to_string()).collect(),
        created_at: at(age),
        updated_at: at(age / 2),
    };
    use WorkKind::{Issue, Mine, Review};
    let items = [
        item(
            "r42",
            Review,
            "aarav/latch",
            42,
            "Permission prompts for the share sheet",
            "kunal-m",
            false,
            &[],
            50,
        ),
        item(
            "r88",
            Review,
            "aarav/gymbros",
            88,
            "Exercise animation loader",
            "neha-k",
            false,
            &[],
            6,
        ),
        item(
            "m21",
            Mine,
            "aarav/codex-claude-bridge",
            21,
            "Per-user login cache for Codex sessions",
            "aarav-dev",
            false,
            &[],
            26,
        ),
        item(
            "m3",
            Mine,
            "aarav/bindle",
            3,
            "Hybrid logical clock for sync",
            "aarav-dev",
            true,
            &[],
            3,
        ),
        item(
            "i17",
            Issue,
            "aarav/codex-claude-bridge",
            17,
            "Token refresh fails on Windows",
            "ritika-s",
            false,
            &["bug"],
            9,
        ),
        item(
            "i39",
            Issue,
            "aarav/latch",
            39,
            "Share sheet drops the caption on Android 15",
            "kunal-m",
            false,
            &["bug"],
            49,
        ),
        item(
            "i12",
            Issue,
            "aarav/unimitra",
            12,
            "MCA Sem 1 notes page returns 404",
            "a-student",
            false,
            &[],
            98,
        ),
    ];
    work::sync(conn, "github", &items, now)?;
    work::save_github_account(
        conn,
        &GithubAccount {
            login: Some("aarav-dev".into()),
            synced_at: Some(stamp(now - Duration::minutes(2))),
            error: None,
        },
    )?;

    // Work promises for the Board: one being made, some shipped this week.
    let making = Keep::new("Fix the token refresh on Windows", Direction::Mine)
        .with("ritika-s")
        .by(now.date_naive() + Duration::days(2))
        .work()
        .save(conn, now - Duration::hours(8))?;
    crate::board::set_making(conn, &making, true)?;
    Keep::new("Write the sync design note", Direction::Mine)
        .on(now.date_naive() + Duration::days(3))
        .work()
        .save(conn, now - Duration::hours(30))?;
    for (title, ago) in [
        ("Ship the latch 0.9 release notes", 20),
        ("Review gymbros #81", 44),
        ("Answer Priya's rate-card question", 70),
    ] {
        let id = Keep::new(title, Direction::Mine)
            .on((now - Duration::hours(ago)).date_naive())
            .work()
            .save(conn, now - Duration::hours(ago + 5))?;
        lifecycle::mark_done(conn, &id, now - Duration::hours(ago))?;
    }

    // Eight weeks of how long people waited on Aarav.
    let mut samples = Vec::new();
    for week in 0..8i64 {
        for (n, (kind, person, base)) in [
            (WaitKind::EmailReply, "Priya Sharma", 5.5),
            (WaitKind::EmailReply, "Rohan Mehta", 3.0),
            (WaitKind::Review, "kunal-m", 11.0),
            (WaitKind::Review, "neha-k", 14.0),
        ]
        .into_iter()
        .enumerate()
        {
            // Email gets quicker week by week; reviews drift slower.
            let hours = match kind {
                WaitKind::EmailReply => base - 0.3 * (7 - week) as f64,
                WaitKind::Review => base + 0.6 * (7 - week) as f64,
            }
            .max(1.0);
            let asked = now - Duration::days(week * 7 + 2 + n as i64);
            samples.push(WaitSample {
                id: format!("sample-{week}-{n}"),
                kind,
                person: person.into(),
                asked_at: asked,
                answered_at: asked + Duration::minutes((hours * 60.0) as i64),
            });
        }
    }
    insights::record_waits(conn, &samples)?;

    // The Code screen's cards: repositories, a flaky check, contributions.
    use crate::github_stats::{self, Checks, FailingCheck, GithubStats, Repo};
    let repo = |name: &str,
                language: &str,
                stars: u32,
                private: bool,
                prs: u32,
                issues: u32,
                checks: Checks| Repo {
        full_name: format!("aarav/{name}"),
        language: Some(language.into()),
        stars,
        private,
        open_prs: prs,
        open_issues: issues,
        checks: Some(checks),
        url: format!("https://github.com/aarav/{name}"),
    };
    let today = now.date_naive();
    let days = (0..112i64)
        .rev()
        .map(|back| {
            let day = today - Duration::days(back);
            // Busier lately, quieter on Sundays, a nine-day run to today.
            let base = ((back * 7 + 3) % 5) as u32 + if back < 40 { 2 } else { 0 };
            let count = if back < 9 {
                base.max(1)
            } else if day.weekday() == Weekday::Sun || (back * 13) % 7 == 0 {
                0
            } else {
                base
            };
            (day.format("%Y-%m-%d").to_string(), count)
        })
        .collect();
    github_stats::save(
        conn,
        &GithubStats {
            repos: vec![
                repo("latch", "TypeScript", 184, false, 3, 9, Checks::Passing),
                repo(
                    "codex-claude-bridge",
                    "TypeScript",
                    96,
                    false,
                    2,
                    6,
                    Checks::Flaky,
                ),
                repo("bindle", "Rust", 0, true, 1, 4, Checks::Passing),
                repo("gymbros", "TypeScript", 21, false, 1, 3, Checks::Passing),
            ],
            failing: Some(FailingCheck {
                repo: "aarav/codex-claude-bridge".into(),
                workflow: "CI (windows-latest)".into(),
                failed: 3,
                of: 12,
                url: "https://github.com/aarav/codex-claude-bridge/actions".into(),
            }),
            days,
            merged_weeks: vec![3, 5, 2, 6, 4, 5, 3, 7],
        },
    )?;
    Ok(())
}

/// Email: two replies owed, one wait, and a promise Aarav wrote.
fn email(conn: &mut Connection, now: DateTime<Utc>, friday: NaiveDate) -> Result<(), StoreError> {
    let at = |h: i64| stamp(now - Duration::hours(h));
    let mail = |id: &str,
                kind: MailKind,
                subject: &str,
                person: &str,
                address: &str,
                age: i64,
                snippet: &str,
                promise: Option<NoticedPromise>| FoundMail {
        id: format!("gmail:{id}"),
        kind,
        thread: format!("thread-{id}"),
        subject: subject.into(),
        person: person.into(),
        address: address.into(),
        at: at(age),
        snippet: snippet.into(),
        url: "https://mail.google.com/".into(),
        promise,
    };
    let items = [
        mail(
            "priya",
            MailKind::Reply,
            "Sponsor brief: Q4 carousel series",
            "Priya Sharma",
            "priya@studio.example",
            30,
            "We would love 4 carousels between 20 October and 10 November, one a week. Could you share your rate card and a first concept so we can lock dates?",
            None,
        ),
        mail(
            "rohan",
            MailKind::Reply,
            "Collab on the Office clones video?",
            "Rohan Mehta",
            "rohan@example.com",
            20,
            "Saw Case 018 do well. Want to do a split screen? I can shoot on Saturday.",
            None,
        ),
        mail(
            "neha",
            MailKind::Waiting,
            "Re: GymBros animation timings",
            "Neha Kapoor",
            "neha@example.com",
            40,
            "I will send the new curl timings tomorrow.",
            None,
        ),
        mail(
            "promise",
            MailKind::Promise,
            "Re: Sponsor brief: Q4 carousel series",
            "Priya Sharma",
            "priya@studio.example",
            70,
            "Sounds great. I will send the first concept by Friday.",
            Some(NoticedPromise {
                title: "Send Priya the first concept".into(),
                when: Some(When {
                    day: friday,
                    time: None,
                }),
                is_deadline: true,
            }),
        ),
    ];
    mail::sync(conn, "gmail", &items, now)?;
    mail::save_gmail_account(
        conn,
        &MailAccount {
            address: Some("aarav@gmail.com".into()),
            synced_at: Some(stamp(now - Duration::minutes(2))),
            error: None,
            provider: Some("gmail".into()),
            host: None,
            port: None,
            today: Some(mail::MailDay {
                total: 41,
                fyi: 14,
                receipts: 6,
                newsletters: 12,
            }),
        },
    )?;
    Ok(())
}

/// YouTube: twelve weeks of growth, recent posts, and comments to answer.
fn youtube(conn: &mut Connection, today: NaiveDate, now: DateTime<Utc>) -> Result<(), StoreError> {
    let channel = Channel {
        id: "UC-sample".into(),
        title: "Aarav Makes".into(),
        handle: Some("@aaravmakes".into()),
        description: "Small apps, explained slowly.".into(),
        followers: Some(18_420),
        views: 1_204_000,
        posts: 96,
    };
    let post = |n: u32, title: &str, short: bool, age: i64, views: u64| FoundPost {
        id: format!("yt-{n}"),
        title: title.into(),
        url: format!("https://youtube.com/watch?v=sample{n}"),
        short,
        duration_seconds: if short { 42 } else { 640 + n * 30 },
        published_at: stamp(now - Duration::days(age)),
        views,
        likes: views / 25,
        comments: views / 300,
    };
    let posts = [
        post(
            1,
            "Case 018: I rebuilt Office in a weekend",
            false,
            3,
            48_200,
        ),
        post(
            2,
            "Why my to-do app failed (and what I built instead)",
            false,
            10,
            31_900,
        ),
        post(
            3,
            "One keyboard shortcut that saves an hour",
            true,
            5,
            22_400,
        ),
        post(
            4,
            "Hinglish dates are hard: kal, parso, agle hafte",
            true,
            12,
            15_100,
        ),
        post(
            5,
            "Local-first apps, explained with chai",
            false,
            19,
            12_800,
        ),
        post(6, "Pixel art for apps in 10 minutes", true, 24, 9_300),
    ];
    let comment = |n: u32, post: &FoundPost, author: &str, text: &str, age: i64| FoundComment {
        id: format!("c-{n}"),
        post_id: post.id.clone(),
        post_title: post.title.clone(),
        author: author.into(),
        text: text.into(),
        url: post.url.clone(),
        likes: 3 + n as u64,
        published_at: stamp(now - Duration::hours(age)),
    };
    let comments = [
        comment(
            1,
            &posts[0],
            "@catchtheerror",
            "Does WordCraft open old .doc files too?",
            5,
        ),
        comment(
            2,
            &posts[1],
            "@byeliteminds",
            "Will you open source the parser? Would love Tamil dates.",
            20,
        ),
        comment(
            3,
            &posts[2],
            "@meeracodes",
            "What's the shortcut on a Mac?",
            30,
        ),
    ];
    creator::sync(conn, "youtube", &channel, &posts, &comments, now)?;
    // Twelve weeks of daily numbers, growing a little faster lately.
    for back in 1..=84i64 {
        let day = today - Duration::days(back);
        let lost = (back as u64) * 140 + (back as u64).pow(2) / 2;
        conn.execute(
            "INSERT OR IGNORE INTO creator_snapshot (source, day, followers, views, posts)
             VALUES ('youtube', ?1, ?2, ?3, ?4)",
            params![
                day.format("%Y-%m-%d").to_string(),
                18_420u64.saturating_sub(lost) as i64,
                1_204_000u64.saturating_sub(lost * 70) as i64,
                96 - back / 7,
            ],
        )?;
    }
    creator::save_youtube_account(
        conn,
        &CreatorAccount {
            channel: Some("@aaravmakes".into()),
            found: Some(channel),
            synced_at: Some(stamp(now - Duration::minutes(14))),
            error: None,
        },
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open_in_memory;
    use chrono::TimeZone;

    fn made() -> (Connection, NaiveDate, DateTime<Utc>) {
        let now = Utc.with_ymd_and_hms(2026, 10, 14, 12, 0, 0).unwrap();
        let today = now.date_naive();
        let mut conn = open_in_memory().unwrap();
        seed(&mut conn, today, now).expect("the sample is made");
        (conn, today, now)
    }

    #[test]
    fn every_screen_has_something_to_show() {
        let (conn, today, now) = made();
        let home = home::home(&conn, today, now).unwrap();
        assert_eq!(home.top3.len(), 2);
        assert!(!home.running_late.is_empty());
        assert!(!home.waiting_on_others.is_empty());
        assert_eq!(promises::sort_queue(&conn).unwrap().len(), 2);
        assert!(people::summaries(&conn).unwrap().len() >= 6);
        let cards = crate::github_stats::load(&conn).unwrap();
        assert_eq!(cards.repos.len(), 4);
        assert!(cards.best_week());
        assert!(cards.streak() >= 9);
        let desk = crate::insights::desk_view(
            &conn,
            now,
            today,
            now - Duration::days(today.weekday().num_days_from_monday() as i64),
        )
        .unwrap();
        let by = |f: fn(&crate::insights::WorkDay) -> u32| desk.week.iter().map(f).sum::<u32>();
        assert!(by(|d| d.code) >= 2 && by(|d| d.email) >= 1 && by(|d| d.content) >= 1);
        let code = work::code_view(&conn, now).unwrap();
        assert_eq!(code.reviews.len(), 2);
        let inbox = mail::inbox_view(&conn).unwrap();
        assert_eq!(inbox.reply.len(), 2);
        assert_eq!(inbox.noticed.len(), 1);
        let creator = creator::creator_view(&conn, now).unwrap();
        assert_eq!(creator.comments.len(), 3);
        let spaces = cloud::spaces(&conn).unwrap();
        assert_eq!(spaces.len(), 1);
        assert_eq!(cloud::members(&conn, SPACE).unwrap().len(), 2);
        assert_eq!(cloud::shared_promises(&conn, SPACE).unwrap().len(), 7);
        assert!(settings::load(&conn).unwrap().onboarded);
        assert!(
            home.today_count <= 5,
            "today is full but not overflowing: {}",
            home.today_count
        );
    }

    #[test]
    fn the_sample_is_dated_from_today() {
        for day in 0..7 {
            let now = Utc.with_ymd_and_hms(2026, 10, 12 + day, 9, 0, 0).unwrap();
            let mut conn = open_in_memory().unwrap();
            seed(&mut conn, now.date_naive(), now).expect("any day of the week works");
        }
    }
}
