use crate::args::Args;
use chrono::{Datelike, FixedOffset, NaiveDate, TimeZone, Utc};
use colored::{Color, Colorize};
use dirs::{config_dir, home_dir};
use html2md::parse_html;
use html2text::from_read;
use http::StatusCode;
use log::{debug, info, warn};
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::header::{
    HeaderMap, HeaderValue, InvalidHeaderValue, CONTENT_TYPE, COOKIE,
    USER_AGENT,
};
use reqwest::redirect::Policy;
use serde::Deserialize;
use std::cmp::{Ordering, Reverse};
use std::collections::HashMap;
use std::env;
use std::fs::{read_to_string, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use thiserror::Error;

pub type PuzzleYear = i32;
pub type PuzzleDay = u32;
pub type MemberId = u64;
pub type Score = u64;

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

const GOLD: Color = Color::Yellow;
const SILVER: Color = Color::TrueColor {
    r: 160,
    g: 160,
    b: 160,
};
const DARK_GRAY: Color = Color::TrueColor {
    r: 96,
    g: 96,
    b: 96,
};

pub type AocResult<T> = Result<T, AocError>;

#[derive(Error, Debug)]
pub enum AocError {
    #[error("Invalid puzzle date: day {0}, year {1}")]
    InvalidPuzzleDate(PuzzleDay, PuzzleYear),

    #[error("{0} is not a valid Advent of Code year")]
    InvalidEventYear(PuzzleYear),

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

    #[error("The private leaderboard does not exist or you are not a member")]
    PrivateLeaderboardNotAvailable,

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
        if now.day() > LAST_PUZZLE_DAY {
            Some(LAST_PUZZLE_DAY)
        } else {
            Some(now.day())
        }
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

fn last_unlocked_day(year: PuzzleYear) -> AocResult<PuzzleDay> {
    if let Some(day) = current_event_day(year) {
        return Ok(day);
    }

    let now = FixedOffset::east_opt(RELEASE_TIMEZONE_OFFSET)
        .unwrap()
        .from_utc_datetime(&Utc::now().naive_utc());

    if year >= FIRST_EVENT_YEAR && year < now.year() {
        Ok(LAST_PUZZLE_DAY)
    } else {
        Err(AocError::InvalidEventYear(year))
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

fn get_private_leaderboard(
    session: &str,
    leaderboard_id: &str,
    year: PuzzleYear,
) -> AocResult<PrivateLeaderboard> {
    debug!("🦌 Fetching private leaderboard {leaderboard_id}");

    let url = format!(
        "https://adventofcode.com/{year}/leaderboard/private/view\
        /{leaderboard_id}.json",
    );

    let response = build_client(session, "application/json")?
        .get(&url)
        .send()
        .and_then(|response| response.error_for_status())?;

    if response.status() == StatusCode::FOUND {
        // A 302 reponse is a redirect and it means
        // the leaderboard doesn't exist or we can't access it
        return Err(AocError::PrivateLeaderboardNotAvailable);
    }

    response.json().map_err(AocError::from)
}

pub fn private_leaderboard(
    args: &Args,
    session: &str,
    leaderboard_id: &str,
) -> AocResult<()> {
    let year = args.year.unwrap_or_else(latest_event_year);
    let last_unlocked_day = last_unlocked_day(year)?;
    let leaderboard = get_private_leaderboard(session, leaderboard_id, year)?;
    let owner_name = leaderboard
        .get_owner_name()
        .ok_or(AocError::AocResponseError)?;

    println!(
        "Private leaderboard of {} for Advent of Code {}.\n\n\
        {} indicates the user got both stars for that day,\n\
        {} means just the first star, and a {} means none.\n",
        owner_name.bold(),
        year.to_string().bold(),
        "Gold *".color(GOLD),
        "silver *".color(SILVER),
        "gray dot (.)".color(DARK_GRAY),
    );

    let mut members: Vec<_> = leaderboard.members.values().collect();
    members.sort_by_key(|member| Reverse(*member));

    let highest_score = members.first().map(|m| m.local_score).unwrap_or(0);
    let score_width = highest_score.to_string().len();
    let highest_rank = 1 + leaderboard.members.len();
    let rank_width = highest_rank.to_string().len();
    let header_pad: String =
        vec![' '; rank_width + score_width].into_iter().collect();

    for header in ["         1111111111222222", "1234567890123456789012345"] {
        let (on, off) = header.split_at(last_unlocked_day as usize);
        println!("{header_pad}   {}{}", on, off.color(DARK_GRAY));
    }

    for (member, rank) in members.iter().zip(1..) {
        let stars: String = (FIRST_PUZZLE_DAY..=LAST_PUZZLE_DAY)
            .map(|day| {
                if day > last_unlocked_day {
                    " ".normal()
                } else {
                    match member.count_stars(day) {
                        2 => "*".color(GOLD),
                        1 => "*".color(SILVER),
                        _ => ".".color(DARK_GRAY),
                    }
                }
                .to_string()
            })
            .collect();

        println!(
            "{rank:rank_width$}) {:score_width$} {stars}  {}",
            member.local_score,
            member.get_name(),
        );
    }

    Ok(())
}

#[derive(Deserialize)]
struct PrivateLeaderboard {
    owner_id: MemberId,
    members: HashMap<MemberId, Member>,
}

impl PrivateLeaderboard {
    fn get_owner_name(&self) -> Option<String> {
        self.members.get(&self.owner_id).map(|m| m.get_name())
    }
}

#[derive(Eq, Deserialize)]
struct Member {
    id: MemberId,
    name: Option<String>,
    local_score: Score,
    completion_day_level: HashMap<PuzzleDay, DayLevel>,
}

type DayLevel = HashMap<String, CollectedStar>;

#[derive(Eq, Deserialize, PartialEq)]
struct CollectedStar {}

impl Member {
    fn get_name(&self) -> String {
        self.name
            .as_ref()
            .cloned()
            .unwrap_or(format!("(anonymous user #{})", self.id))
    }

    fn count_stars(&self, day: PuzzleDay) -> usize {
        self.completion_day_level
            .get(&day)
            .map(|stars| stars.len())
            .unwrap_or(0)
    }
}

impl Ord for Member {
    fn cmp(&self, other: &Self) -> Ordering {
        // Members are sorted by increasing local score and then decreasing ID
        self.local_score
            .cmp(&other.local_score)
            .then(self.id.cmp(&other.id).reverse())
    }
}

impl PartialOrd for Member {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Member {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

fn get_calendar(session_cookie: &str, year: PuzzleYear) -> AocResult<String> {
    debug!("🦌 Fetching {year} calendar");

    let url = format!("https://adventofcode.com/{year}");
    let response = build_client(session_cookie, "text/html")?
        .get(&url)
        .send()?;

    if response.status() == StatusCode::NOT_FOUND {
        // A 402 reponse means the calendar for
        // the requested year is not yet available
        return Err(AocError::InvalidEventYear(year));
    }

    let contents = response.error_for_status()?.text()?;

    if Regex::new(r#"href="/[0-9]{4}/auth/login""#)
        .unwrap()
        .is_match(&contents)
    {
        warn!("🍪 It looks like you are not logged in, try logging in again");
    }

    let main = Regex::new(r"(?i)(?s)<main>(?P<main>.*)</main>")
        .unwrap()
        .captures(&contents)
        .ok_or(AocError::AocResponseError)?
        .name("main")
        .unwrap()
        .as_str()
        .to_string();

    // Remove elements that won't render well in the terminal
    let cleaned_up = Regex::new(concat!(
        // Remove all hyperlinks
        r#"(href="[^"]*")"#,
        // Remove 2015 "calendar-bkg"
        r#"|(<div class="calendar-bkg">[[:space:]]*"#,
        r#"(<div>[^<]*</div>[[:space:]]*)*</div>)"#,
        // Remove 2017 "naughty/nice" animation
        r#"|(<div class="calendar-printer">(?s:.)*"#,
        r#"\|O\|</span></div>[[:space:]]*)"#,
        // Remove 2018 "space mug"
        r#"|(<pre id="spacemug"[^>]*>[^<]*</pre>)"#,
        // Remove 2019 shadows
        r#"|(<span style="color[^>]*position:absolute"#,
        r#"[^>]*>\.</span>)"#,
        // Remove 2019 "sunbeam"
        r#"|(<span class="sunbeam"[^>]*>"#,
        r#"<span style="animation-delay[^>]*>\*</span></span>)"#,
    ))
    .unwrap()
    .replace_all(&main, "")
    .to_string();

    let class_regex =
        Regex::new(r#"<a [^>]*class="(?P<class>[^"]*)""#).unwrap();
    let star_regex = Regex::new(concat!(
        r#"(?P<stars><span class="calendar-mark-complete">\*</span>"#,
        r#"<span class="calendar-mark-verycomplete">\*</span>)"#,
    ))
    .unwrap();

    // Remove stars that have not been collected
    let calendar = cleaned_up
        .lines()
        .map(|line| {
            let class = class_regex
                .captures(line)
                .and_then(|c| c.name("class"))
                .map(|c| c.as_str())
                .unwrap_or("");

            let stars = if class.contains("calendar-verycomplete") {
                "**"
            } else if class.contains("calendar-complete") {
                "*"
            } else {
                ""
            };

            star_regex.replace(line, stars)
        })
        .collect::<Vec<_>>()
        .join("\n");

    Ok(calendar)
}

pub fn calendar(
    args: &Args,
    session_cookie: &str,
    col_width: usize,
) -> AocResult<()> {
    let year = args.year.unwrap_or_else(latest_event_year);
    let desc = get_calendar(session_cookie, year)?;
    println!("\n{}", from_read(desc.as_bytes(), col_width));
    Ok(())
}
