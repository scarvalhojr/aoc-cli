use crate::args::Args;
use chrono::{Datelike, FixedOffset, NaiveDate, TimeZone, Utc};
use dirs::{config_dir, home_dir};
use html2md::parse_html;
use html2text::from_read;
use log::{debug, info};
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::header::{
    HeaderMap, HeaderValue, InvalidHeaderValue, CONTENT_TYPE, COOKIE,
    USER_AGENT,
};
use reqwest::redirect::Policy;
use serde::Deserialize;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::env;
use std::fs::{read_to_string, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use thiserror::Error;

pub type PuzzleYear = i32;
pub type PuzzleDay = u32;

const FIRST_EVENT_YEAR: PuzzleYear = 2015;
const DECEMBER: u32 = 12;
const FIRST_PUZZLE_DAY: PuzzleDay = 1;
const LAST_PUZZLE_DAY: PuzzleDay = 25;
const RELEASE_TIMEZONE_OFFSET: i32 = -5 * 3600;

const SESSION_COOKIE_FILE: &str = "adventofcode.session";
const HIDDEN_SESSION_COOKIE_FILE: &str = ".adventofcode.session";
const SESSION_COOKIE_ENV_VAR: &str = "ADVENT_OF_CODE_SESSION";

const PKG_REPO: &str = env!("CARGO_PKG_REPOSITORY");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type AocResult<T> = Result<T, AocError>;

#[derive(Error, Debug)]
pub enum AocError {
    #[error("Invalid puzzle date: day {0}, year {1}")]
    InvalidPuzzleDate(PuzzleDay, PuzzleYear),

    #[error("Could not infer puzzle day for year {0}")]
    NonInferablePuzzleDate(PuzzleYear),

    #[error("Puzzle {0} of {1} is still locked")]
    LockedPuzzle(PuzzleDay, PuzzleYear),

    #[error("Failed to find user config directory")]
    MissingConfigDir,

    #[error("Failed to read session cookie from '{filename}': {source}")]
    SessionFileReadError {
        filename: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid session cookie: {source}")]
    InvalidSessionCookie {
        #[from]
        source: InvalidHeaderValue,
    },

    #[error("HTTP request error: {source}")]
    HttpRequestError {
        #[from]
        source: reqwest::Error,
    },

    #[error("Failed to parse Advent of Code response")]
    AocResponseError,

    #[error("Failed to write to file '{filename}': {source}")]
    FileWriteError {
        filename: String,
        #[source]
        source: std::io::Error,
    },
}

pub fn is_valid_year(year: PuzzleYear) -> bool {
    year >= FIRST_EVENT_YEAR
}

pub fn is_valid_day(day: PuzzleDay) -> bool {
    (FIRST_PUZZLE_DAY..=LAST_PUZZLE_DAY).contains(&day)
}

fn latest_event_year() -> PuzzleYear {
    let now = FixedOffset::east_opt(RELEASE_TIMEZONE_OFFSET)
        .unwrap()
        .from_utc_datetime(&Utc::now().naive_utc());

    if now.month() < DECEMBER {
        now.year() - 1
    } else {
        now.year()
    }
}

fn current_event_day(year: PuzzleYear) -> Option<PuzzleDay> {
    let now = FixedOffset::east_opt(RELEASE_TIMEZONE_OFFSET)?
        .from_utc_datetime(&Utc::now().naive_utc());

    if now.month() == DECEMBER && now.year() == year {
        Some(now.day())
    } else {
        None
    }
}

fn puzzle_unlocked(year: PuzzleYear, day: PuzzleDay) -> AocResult<bool> {
    let timezone = FixedOffset::east_opt(RELEASE_TIMEZONE_OFFSET).unwrap();
    let now = timezone.from_utc_datetime(&Utc::now().naive_utc());
    let puzzle_date = NaiveDate::from_ymd_opt(year, DECEMBER, day)
        .ok_or(AocError::InvalidPuzzleDate(day, year))?
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let unlock_time = timezone.from_local_datetime(&puzzle_date).single();

    if let Some(time) = unlock_time {
        Ok(now.signed_duration_since(time).num_milliseconds() >= 0)
    } else {
        Ok(false)
    }
}

fn puzzle_year_day(
    opt_year: Option<PuzzleYear>,
    opt_day: Option<PuzzleDay>,
) -> AocResult<(PuzzleYear, PuzzleDay)> {
    let year = opt_year.unwrap_or_else(latest_event_year);
    let day = opt_day
        .or_else(|| current_event_day(year))
        .ok_or(AocError::NonInferablePuzzleDate(year))?;

    if !puzzle_unlocked(year, day)? {
        return Err(AocError::LockedPuzzle(day, year));
    }

    Ok((year, day))
}

pub fn load_session_cookie(session_file: &Option<String>) -> AocResult<String> {
    if session_file.is_none() {
        if let Ok(cookie) = env::var(SESSION_COOKIE_ENV_VAR) {
            debug!(
                "🍪 Loaded session cookie from '{SESSION_COOKIE_ENV_VAR}' \
                 environment variable"
            );
            return Ok(cookie);
        }
    }

    let path = if let Some(file) = session_file {
        PathBuf::from(file)
    } else if let Some(file) = home_dir()
        .map(|dir| dir.join(HIDDEN_SESSION_COOKIE_FILE))
        .filter(|file| file.exists())
    {
        file
    } else if let Some(dir) = config_dir() {
        dir.join(SESSION_COOKIE_FILE)
    } else {
        return Err(AocError::MissingConfigDir);
    };

    let cookie =
        read_to_string(&path).map_err(|err| AocError::SessionFileReadError {
            filename: path.display().to_string(),
            source: err,
        });

    if cookie.is_ok() {
        debug!("🍪 Loaded session cookie from '{}'", path.display());
    }

    cookie
}

fn build_client(session_cookie: &str, content_type: &str) -> AocResult<Client> {
    let cookie_header =
        HeaderValue::from_str(&format!("session={}", session_cookie.trim()))?;
    let content_type_header = HeaderValue::from_str(content_type).unwrap();
    let user_agent = format!("{PKG_REPO} {PKG_VERSION}");
    let user_agent_header = HeaderValue::from_str(&user_agent).unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, cookie_header);
    headers.insert(CONTENT_TYPE, content_type_header);
    headers.insert(USER_AGENT, user_agent_header);

    Client::builder()
        .default_headers(headers)
        .redirect(Policy::none())
        .build()
        .map_err(AocError::from)
}

fn get_description(
    session_cookie: &str,
    year: PuzzleYear,
    day: PuzzleDay,
) -> AocResult<String> {
    debug!("🦌 Fetching puzzle for day {}, {}", day, year);

    let url = format!("https://adventofcode.com/{}/day/{}", year, day);
    let response = build_client(session_cookie, "text/html")?
        .get(&url)
        .send()
        .and_then(|response| response.error_for_status())
        .and_then(|response| response.text())?;

    let desc = Regex::new(r"(?i)(?s)<main>(?P<main>.*)</main>")
        .unwrap()
        .captures(&response)
        .ok_or(AocError::AocResponseError)?
        .name("main")
        .unwrap()
        .as_str()
        .to_string();

    Ok(desc)
}

fn get_input(
    session_cookie: &str,
    year: PuzzleYear,
    day: PuzzleDay,
) -> AocResult<String> {
    debug!("🦌 Downloading input for day {}, {}", day, year);

    let url = format!("https://adventofcode.com/{}/day/{}/input", year, day);
    build_client(session_cookie, "text/plain")?
        .get(&url)
        .send()
        .and_then(|response| response.error_for_status())
        .and_then(|response| response.text())
        .map_err(AocError::from)
}

fn save_file(filename: &str, overwrite: bool, contents: &str) -> AocResult<()> {
    let mut file = OpenOptions::new();
    if overwrite {
        file.create(true);
    } else {
        file.create_new(true);
    };

    file.write(true)
        .truncate(true)
        .open(filename)
        .and_then(|mut file| file.write_all(contents.as_bytes()))
        .map_err(|err| AocError::FileWriteError {
            filename: filename.to_string(),
            source: err,
        })
}

pub fn download(args: &Args, session_cookie: &str) -> AocResult<()> {
    let (year, day) = puzzle_year_day(args.year, args.day)?;

    if !args.input_only {
        let desc = get_description(session_cookie, year, day)?;
        save_file(&args.puzzle_file, args.overwrite, &parse_html(&desc))?;
        info!("🎅 Saved puzzle description to '{}'", args.puzzle_file);
    }

    if !args.puzzle_only {
        let input = get_input(session_cookie, year, day)?;
        save_file(&args.input_file, args.overwrite, &input)?;
        info!("🎅 Saved puzzle input to '{}'", args.input_file);
    }

    Ok(())
}

pub fn submit(
    args: &Args,
    session_cookie: &str,
    col_width: usize,
    part: &str,
    answer: &str,
) -> AocResult<()> {
    let (year, day) = puzzle_year_day(args.year, args.day)?;

    debug!("🦌 Submitting answer for part {part}, day {day}, {year}");
    let url = format!("https://adventofcode.com/{}/day/{}/answer", year, day);
    let content_type = "application/x-www-form-urlencoded";
    let response = build_client(session_cookie, content_type)?
        .post(&url)
        .body(format!("level={}&answer={}", part, answer))
        .send()
        .and_then(|response| response.error_for_status())
        .and_then(|response| response.text())?;

    let result = Regex::new(r"(?i)(?s)<main>(?P<main>.*)</main>")
        .unwrap()
        .captures(&response)
        .ok_or(AocError::AocResponseError)?
        .name("main")
        .unwrap()
        .as_str();

    println!("\n{}", from_read(result.as_bytes(), col_width));
    Ok(())
}

pub fn read(
    args: &Args,
    session_cookie: &str,
    col_width: usize,
) -> AocResult<()> {
    let (year, day) = puzzle_year_day(args.year, args.day)?;
    let desc = get_description(session_cookie, year, day)?;
    println!("\n{}", from_read(desc.as_bytes(), col_width));
    Ok(())
}

fn get_private_leaderboard_results(
    args: &Args,
    session: &str,
    leaderboard: &str,
    year: PuzzleYear,
) -> AocResult<PrivateLeaderboard> {
    debug!("🦌 Fetching private leaderboard {}", leaderboard);

    let url = format!(
        "https://adventofcode.com/{}/leaderboard/private/view/{}.json",
        year, leaderboard
    );

    let leaderboard: PrivateLeaderboard =
        build_client(session, "application/json")?
            .get(&url)
            .send()
            .and_then(|response| response.error_for_status())
            .and_then(|response| response.json())
            .map_err(AocError::from)?;
    Ok(leaderboard)
}

pub fn show_private_leaderboard_results(
    args: &Args,
    session: &str,
    leaderboard: &str,
) -> AocResult<()> {
    let (year, day) = puzzle_year_day(args.year, args.day)?;
    let leaderboard =
        get_private_leaderboard_results(args, session, leaderboard, year)?;

    let mut members: Vec<_> = leaderboard.members.values().collect();
    members.sort_by_key(|m| Reverse(m.local_score));
    members.iter().enumerate().for_each(|(idx, m)| {
        let display_name = m
            .name
            .clone()
            .unwrap_or(format!("anonymous user #{}", m.id));

        let stars: String = (1..=25)
            .map(|d| {
                if d > day {
                    ' '
                } else {
                    let stars = m.stars_per_day(d);
                    match stars {
                        2 => '★',
                        1 => '☆',
                        _ => '.',
                    }
                }
            })
            .collect();

        let order = idx + 1;
        println!("{}\t{}\t{}\t{}", order, m.local_score, stars, display_name);
    });

    Ok(())
}

#[derive(Deserialize)]
struct PrivateLeaderboard {
    owner_id: usize,
    event: String,
    members: HashMap<String, Member>,
}

#[derive(Deserialize)]
struct Member {
    name: Option<String>,
    id: u64,
    global_score: u64,
    local_score: u64,
    stars: u8,
    completion_day_level: HashMap<u32, DayLevel>,
}

impl Member {
    fn stars_per_day(&self, day: u32) -> u8 {
        self.completion_day_level
            .get(&day)
            .map(|d| d.stars.len() as u8)
            .unwrap_or(0)
    }
}

#[derive(Deserialize)]
struct DayLevel {
    #[serde(flatten)]
    stars: HashMap<String, Star>,
}

#[derive(Deserialize)]
struct Star {
    get_star_ts: u64,
    star_index: u64,
}
