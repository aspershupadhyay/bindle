// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Sparsh Upadhyay and the Bindle contributors.

//! The rest of the Code screen, read only: your most active repositories with
//! their open pull requests, issues and checks; a check that keeps failing;
//! sixteen weeks of contributions; and your pull requests merged each week.
//! Each part is a bonus: if one cannot be read, the others still show.

use crate::github::{Client, GithubError};
use bindle_store::github_stats::{Checks, FailingCheck, GithubStats, Repo};
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use serde::Deserialize;

/// How many repositories the Code screen lists.
const REPOS: usize = 4;
/// How many recent check runs decide passing, failing or flaky.
const RUNS: usize = 12;

#[derive(Deserialize)]
struct RepoJson {
    full_name: String,
    language: Option<String>,
    #[serde(default)]
    stargazers_count: u32,
    #[serde(default)]
    private: bool,
    #[serde(default)]
    open_issues_count: u32,
    default_branch: Option<String>,
    html_url: String,
    #[serde(default)]
    archived: bool,
}

fn parse<T: for<'a> Deserialize<'a>>(body: &str) -> Result<T, GithubError> {
    serde_json::from_str(body).map_err(|e| GithubError::Unexpected(e.to_string()))
}

#[derive(Deserialize)]
struct Runs {
    #[serde(default)]
    workflow_runs: Vec<Run>,
}

#[derive(Deserialize, Clone)]
struct Run {
    name: Option<String>,
    conclusion: Option<String>,
    html_url: String,
}

/// Passing, failing or flaky, from runs newest first. Runs still going are
/// skipped; with no finished runs there is nothing to say.
pub fn checks_of(conclusions: &[&str]) -> Option<Checks> {
    let done: Vec<bool> = conclusions
        .iter()
        .filter(|c| matches!(**c, "success" | "failure" | "timed_out"))
        .map(|c| *c == "success")
        .collect();
    let (latest, rest) = done.split_first()?;
    let failures = done.iter().filter(|ok| !**ok).count();
    Some(match (latest, rest.first()) {
        (false, Some(false)) => Checks::Failing,
        _ if failures >= 2 => Checks::Flaky,
        (false, _) => Checks::Failing,
        _ => Checks::Passing,
    })
}

/// The workflow that failed most often in `runs`, if it failed at least twice.
fn failing_check(repo: &str, runs: &[Run]) -> Option<FailingCheck> {
    let mut by_name: Vec<(String, u32, u32, String)> = Vec::new();
    for run in runs.iter().take(RUNS) {
        let Some(conclusion) = run.conclusion.as_deref() else {
            continue;
        };
        let name = run.name.clone().unwrap_or_else(|| "A check".into());
        let entry = match by_name.iter_mut().find(|(n, ..)| *n == name) {
            Some(entry) => entry,
            None => {
                by_name.push((name, 0, 0, String::new()));
                by_name.last_mut().expect("just pushed")
            }
        };
        entry.2 += 1;
        if conclusion != "success" && conclusion != "skipped" {
            entry.1 += 1;
            if entry.3.is_empty() {
                entry.3 = run.html_url.clone();
            }
        }
    }
    by_name
        .into_iter()
        .filter(|(_, failed, ..)| *failed >= 2)
        .max_by_key(|(_, failed, ..)| *failed)
        .map(|(workflow, failed, of, url)| FailingCheck {
            repo: repo.to_string(),
            workflow,
            failed,
            of,
            url,
        })
}

#[derive(Deserialize)]
struct Graph {
    data: Option<GraphData>,
}
#[derive(Deserialize)]
struct GraphData {
    viewer: Viewer,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Viewer {
    contributions_collection: Collection,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Collection {
    contribution_calendar: Calendar,
}
#[derive(Deserialize)]
struct Calendar {
    weeks: Vec<Week>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Week {
    contribution_days: Vec<Day>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Day {
    date: String,
    contribution_count: u32,
}

/// Contributions per day, oldest first, from GitHub's GraphQL answer.
pub fn parse_calendar(body: &str) -> Result<Vec<(String, u32)>, GithubError> {
    let graph: Graph = parse(body)?;
    let data = graph
        .data
        .ok_or_else(|| GithubError::Unexpected("no contribution calendar".into()))?;
    Ok(data
        .viewer
        .contributions_collection
        .contribution_calendar
        .weeks
        .into_iter()
        .flat_map(|w| w.contribution_days)
        .map(|d| (d.date, d.contribution_count))
        .collect())
}

#[derive(Deserialize)]
struct Search {
    items: Vec<Closed>,
}
#[derive(Deserialize)]
struct Closed {
    closed_at: Option<String>,
}

/// Merged pull requests counted into eight weeks (Monday to Sunday), oldest
/// first, the last being this week.
pub fn merged_by_week(body: &str, today: NaiveDate) -> Result<Vec<u32>, GithubError> {
    let search: Search = parse(body)?;
    let this_monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    let mut weeks = vec![0u32; 8];
    for at in search.items.iter().filter_map(|i| i.closed_at.as_deref()) {
        let Ok(at) = DateTime::parse_from_rfc3339(at) else {
            continue;
        };
        let monday = at.date_naive() - Duration::days(at.weekday().num_days_from_monday() as i64);
        let back = (this_monday - monday).num_days() / 7;
        if (0..8).contains(&back) {
            weeks[7 - back as usize] += 1;
        }
    }
    Ok(weeks)
}

const CALENDAR: &str = "query($from: DateTime!, $to: DateTime!) { viewer { contributionsCollection(from: $from, to: $to) { contributionCalendar { weeks { contributionDays { date contributionCount } } } } } }";

impl Client {
    /// Reads everything the Code screen shows beyond the work lists. Parts
    /// that fail are left empty; only a refused token fails the whole look.
    pub async fn stats(&self, now: DateTime<Utc>) -> Result<GithubStats, GithubError> {
        let mut stats = GithubStats::default();
        let repos: Vec<RepoJson> = parse(
            &self
                .get(
                    "/user/repos",
                    &[
                        ("sort", "pushed"),
                        ("per_page", "10"),
                        ("affiliation", "owner,collaborator"),
                    ],
                )
                .await?,
        )?;
        for repo in repos.into_iter().filter(|r| !r.archived).take(REPOS) {
            let pulls = self
                .get(
                    &format!("/repos/{}/pulls", repo.full_name),
                    &[("state", "open"), ("per_page", "100")],
                )
                .await
                .ok()
                .and_then(|body| parse::<Vec<serde_json::Value>>(&body).ok())
                .map_or(0, |p| p.len() as u32);
            let branch = repo.default_branch.clone().unwrap_or_else(|| "main".into());
            let runs = self
                .get(
                    &format!("/repos/{}/actions/runs", repo.full_name),
                    &[("branch", branch.as_str()), ("per_page", "12")],
                )
                .await
                .ok()
                .and_then(|body| parse::<Runs>(&body).ok())
                .map(|r| r.workflow_runs)
                .unwrap_or_default();
            let conclusions: Vec<&str> = runs
                .iter()
                .filter_map(|r| r.conclusion.as_deref())
                .collect();
            if stats.failing.is_none() {
                stats.failing = failing_check(&repo.full_name, &runs);
            }
            stats.repos.push(Repo {
                checks: checks_of(&conclusions),
                open_prs: pulls,
                open_issues: repo.open_issues_count.saturating_sub(pulls),
                full_name: repo.full_name,
                language: repo.language,
                stars: repo.stargazers_count,
                private: repo.private,
                url: repo.html_url,
            });
        }
        let from = (now - Duration::weeks(16)).to_rfc3339();
        let body = serde_json::json!({ "query": CALENDAR, "variables": { "from": from, "to": now.to_rfc3339() } });
        if let Ok(days) = self
            .post("/graphql", &body)
            .await
            .and_then(|b| parse_calendar(&b))
        {
            stats.days = days;
        }
        let since = (now - Duration::weeks(8)).format("%Y-%m-%d").to_string();
        let query = format!("is:pr author:@me is:merged closed:>={since}");
        if let Ok(weeks) = self
            .get(
                "/search/issues",
                &[("q", query.as_str()), ("per_page", "100")],
            )
            .await
            .and_then(|b| merged_by_week(&b, now.date_naive()))
        {
            stats.merged_weeks = weeks;
        }
        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks_read_passing_failing_and_flaky() {
        assert_eq!(checks_of(&["success", "success"]), Some(Checks::Passing));
        assert_eq!(
            checks_of(&["failure", "failure", "success"]),
            Some(Checks::Failing)
        );
        assert_eq!(
            checks_of(&["success", "failure", "success", "failure"]),
            Some(Checks::Flaky)
        );
        assert_eq!(checks_of(&["failure", "success"]), Some(Checks::Failing));
        assert_eq!(checks_of(&["cancelled"]), None);
        assert_eq!(checks_of(&[]), None);
    }

    #[test]
    fn the_check_that_fails_most_is_named_with_its_latest_logs() {
        let run = |name: &str, conclusion: &str, n: u32| Run {
            name: Some(name.into()),
            conclusion: Some(conclusion.into()),
            html_url: format!("https://github.com/o/r/actions/runs/{n}"),
        };
        let runs = [
            run("CI", "failure", 9),
            run("Lint", "success", 8),
            run("CI", "success", 7),
            run("CI", "failure", 6),
            run("Lint", "failure", 5),
            run("CI", "failure", 4),
        ];
        let failing = failing_check("o/r", &runs).unwrap();
        assert_eq!(
            (failing.workflow.as_str(), failing.failed, failing.of),
            ("CI", 3, 4)
        );
        assert!(failing.url.ends_with("/9"));
        assert!(failing_check("o/r", &runs[1..3]).is_none());
    }

    #[test]
    fn the_calendar_reads_day_by_day() {
        let body = r#"{"data":{"viewer":{"contributionsCollection":{"contributionCalendar":{"weeks":[
            {"contributionDays":[{"date":"2026-10-11","contributionCount":0},{"date":"2026-10-12","contributionCount":4}]},
            {"contributionDays":[{"date":"2026-10-13","contributionCount":2}]}]}}}}}"#;
        let days = parse_calendar(body).unwrap();
        assert_eq!(
            days,
            [
                ("2026-10-11".into(), 0),
                ("2026-10-12".into(), 4),
                ("2026-10-13".into(), 2)
            ]
        );
        assert!(parse_calendar(r#"{"errors":[{"message":"no"}]}"#).is_err());
    }

    #[test]
    fn merged_pull_requests_land_in_their_week() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 15).unwrap(); // a Thursday
        let body = r#"{"items":[
            {"closed_at":"2026-10-12T09:00:00Z"},
            {"closed_at":"2026-10-14T09:00:00Z"},
            {"closed_at":"2026-10-11T23:00:00Z"},
            {"closed_at":"2026-08-01T09:00:00Z"},
            {"closed_at":null}]}"#;
        let weeks = merged_by_week(body, today).unwrap();
        assert_eq!(weeks.len(), 8);
        assert_eq!(weeks[7], 2, "Monday and Wednesday of this week");
        assert_eq!(weeks[6], 1, "the Sunday before");
        assert_eq!(
            weeks.iter().sum::<u32>(),
            3,
            "older than eight weeks is left out"
        );
    }
}
