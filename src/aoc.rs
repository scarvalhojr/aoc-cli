use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, TimeZone, Utc};
use colored::{Color, Colorize};
use dirs::{config_dir, home_dir};
use html2md::parse_html;
use html2text::from_read;
use http::StatusCode;
use log::{debug, info, warn};
use regex::Regex;
use reqwest::blocking::Client as HttpClient;
use reqwest::header::{
    HeaderMap, HeaderValue, CONTENT_TYPE, COOKIE, USER_AGENT,
};
use reqwest::redirect::Policy;
use serde::Deserialize;
use std::cmp::{Ordering, Reverse};
use std::collections::HashMap;
use std::env;
use std::fmt::{Display, Formatter};
use std::fs::{read_to_string, OpenOptions};
use std::io::Write;
use std::path::Path;
use thiserror::Error;

pub type PuzzleYear = i32;
pub type PuzzleDay = u32;
pub type LeaderboardId = u32;
pub type MemberId = u64;
pub type Score = u64;

pub enum PuzzlePart {
    PartOne,
    PartTwo,
}

const FIRST_EVENT_YEAR: PuzzleYear = 2015;
const DECEMBER: u32 = 12;
const FIRST_PUZZLE_DAY: PuzzleDay = 1;
const LAST_PUZZLE_DAY: PuzzleDay = 25;
const RELEASE_TIMEZONE_OFFSET: i32 = -5 * 3600;

const SESSION_COOKIE_FILE: &str = "adventofcode.session";
const HIDDEN_SESSION_COOKIE_FILE: &str = ".adventofcode.session";
const SESSION_COOKIE_ENV_VAR: &str = "ADVENT_OF_CODE_SESSION";

const DEFAULT_COL_WIDTH: usize = 80;

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

    #[error("{0} is not a valid Advent of Code day")]
    InvalidPuzzleDay(PuzzleDay),

    #[error("Puzzle {0} of {1} is still locked")]
    LockedPuzzle(PuzzleDay, PuzzleYear),

    #[error("Session cookie file not found in home or config directory")]
    SessionFileNotFound,

    #[error("Failed to read session cookie from '{filename}': {source}")]
    SessionFileReadError {
        filename: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid session cookie")]
    InvalidSessionCookie,

    #[error("HTTP request error: {0}")]
    HttpRequestError(#[from] reqwest::Error),

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

    #[error("Failed to create client due to missing field: {0}")]
    ClientFieldMissing(String),

    #[error("Invalid puzzle part number")]
    InvalidPuzzlePart,

    #[error("Output width must be greater than zero")]
    InvalidOutputWidth,
}

pub struct AocClient {
    session_cookie: String,
    unlock_datetime: DateTime<FixedOffset>,
    year: PuzzleYear,
    day: PuzzleDay,
    output_width: usize,
    overwrite_file: bool,
    input_file: String,
    puzzle_file: String,
}

#[must_use]
pub struct AocClientBuilder {
    session_cookie: Option<String>,
    year: Option<PuzzleYear>,
    day: Option<PuzzleDay>,
    output_width: usize,
    overwrite_file: bool,
    input_file: String,
    puzzle_file: String,
}

impl AocClient {
    pub fn builder() -> AocClientBuilder {
        AocClientBuilder::default()
    }

    pub fn day_unlocked(&self) -> bool {
        let timezone = FixedOffset::east_opt(RELEASE_TIMEZONE_OFFSET).unwrap();
        let now = timezone.from_utc_datetime(&Utc::now().naive_utc());
        now.signed_duration_since(self.unlock_datetime)
            .num_milliseconds()
            >= 0
    }

    fn ensure_day_unlocked(&self) -> AocResult<()> {
        if self.day_unlocked() {
            Ok(())
        } else {
            Err(AocError::LockedPuzzle(self.day, self.year))
        }
    }

    pub fn get_puzzle_html(&self) -> AocResult<String> {
        self.ensure_day_unlocked()?;

        debug!("🦌 Fetching puzzle for day {}, {}", self.day, self.year);

        let url =
            format!("https://adventofcode.com/{}/day/{}", self.year, self.day);
        let response = http_client(&self.session_cookie, "text/html")?
            .get(url)
            .send()
            .and_then(|response| response.error_for_status())
            .and_then(|response| response.text())?;
        let puzzle_html = Regex::new(r"(?i)(?s)<main>(?P<main>.*)</main>")
            .unwrap()
            .captures(&response)
            .ok_or(AocError::AocResponseError)?
            .name("main")
            .unwrap()
            .as_str()
            .to_string();

        Ok(puzzle_html)
    }

    pub fn get_input(&self) -> AocResult<String> {
        self.ensure_day_unlocked()?;

        debug!("🦌 Fetching input for day {}, {}", self.day, self.year);

        let url = format!(
            "https://adventofcode.com/{}/day/{}/input",
            self.year, self.day
        );
        http_client(&self.session_cookie, "text/plain")?
            .get(url)
            .send()
            .and_then(|response| response.error_for_status())
            .and_then(|response| response.text())
            .map_err(AocError::from)
    }

    pub fn submit_answer<P, D>(
        &self,
        puzzle_part: P,
        answer: D,
    ) -> AocResult<String>
    where
        P: TryInto<PuzzlePart>,
        AocError: From<P::Error>,
        D: Display,
    {
        self.ensure_day_unlocked()?;
        let part: PuzzlePart = puzzle_part.try_into()?;

        debug!(
            "🦌 Submitting answer for part {part}, day {}, {}",
            self.day, self.year
        );

        let url = format!(
            "https://adventofcode.com/{}/day/{}/answer",
            self.year, self.day
        );
        let content_type = "application/x-www-form-urlencoded";
        http_client(&self.session_cookie, content_type)?
            .post(url)
            .body(format!("level={part}&answer={answer}"))
            .send()
            .and_then(|response| response.error_for_status())
            .and_then(|response| response.text())
            .map_err(AocError::HttpRequestError)
    }

    pub fn submit_answer_and_show_result<P, D>(
        &self,
        part: P,
        answer: D,
    ) -> AocResult<()>
    where
        P: TryInto<PuzzlePart>,
        AocError: From<P::Error>,
        D: Display,
    {
        let response = self.submit_answer(part, answer)?;
        let result = Regex::new(r"(?i)(?s)<main>(?P<main>.*)</main>")
            .unwrap()
            .captures(&response)
            .ok_or(AocError::AocResponseError)?
            .name("main")
            .unwrap()
            .as_str();

        println!("\n{}", from_read(result.as_bytes(), self.output_width));
        Ok(())
    }

    pub fn show_puzzle_text(&self) -> AocResult<()> {
        let puzzle_html = self.get_puzzle_html()?;
        let puzzle_text = from_read(puzzle_html.as_bytes(), self.output_width);
        println!("\n{puzzle_text}");
        Ok(())
    }

    pub fn save_puzzle_markdown(&self) -> AocResult<()> {
        let puzzle_html = self.get_puzzle_html()?;
        let puzzle_markdow = parse_html(&puzzle_html);
        save_file(&self.puzzle_file, self.overwrite_file, &puzzle_markdow)?;
        info!("🎅 Saved puzzle to '{}'", &self.puzzle_file);
        Ok(())
    }

    pub fn save_input(&self) -> AocResult<()> {
        let input = self.get_input()?;
        save_file(&self.input_file, self.overwrite_file, &input)?;
        info!("🎅 Saved input to '{}'", &self.input_file);
        Ok(())
    }

    pub fn get_calendar_html(&self) -> AocResult<String> {
        debug!("🦌 Fetching {} calendar", self.year);

        let url = format!("https://adventofcode.com/{}", self.year);
        let response = http_client(&self.session_cookie, "text/html")?
            .get(url)
            .send()?;

        if response.status() == StatusCode::NOT_FOUND {
            // A 402 reponse means the calendar for
            // the requested year is not yet available
            return Err(AocError::InvalidEventYear(self.year));
        }

        let contents = response.error_for_status()?.text()?;

        if Regex::new(r#"href="/[0-9]{4}/auth/login""#)
            .unwrap()
            .is_match(&contents)
        {
            warn!(
                "🍪 It looks like you are not logged in, try logging in again"
            );
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

    pub fn show_calendar(&self) -> AocResult<()> {
        let calendar = self.get_calendar_html()?;
        println!("\n{}", from_read(calendar.as_bytes(), self.output_width));
        Ok(())
    }

    fn get_private_leaderboard(
        &self,
        leaderboard_id: LeaderboardId,
    ) -> AocResult<PrivateLeaderboard> {
        debug!("🦌 Fetching private leaderboard {leaderboard_id}");

        let url = format!(
            "https://adventofcode.com/{}/leaderboard/private/view\
            /{leaderboard_id}.json",
            self.year,
        );
        let response = http_client(&self.session_cookie, "application/json")?
            .get(url)
            .send()
            .and_then(|response| response.error_for_status())?;

        if response.status() == StatusCode::FOUND {
            // A 302 reponse is a redirect and it means
            // the leaderboard doesn't exist or we can't access it
            return Err(AocError::PrivateLeaderboardNotAvailable);
        }

        response.json().map_err(AocError::from)
    }

    pub fn show_private_leaderboard(
        &self,
        leaderboard_id: LeaderboardId,
    ) -> AocResult<()> {
        let last_unlocked_day = last_unlocked_day(self.year)
            .ok_or(AocError::InvalidEventYear(self.year))?;
        let leaderboard = self.get_private_leaderboard(leaderboard_id)?;
        let owner_name = leaderboard
            .get_owner_name()
            .ok_or(AocError::AocResponseError)?;

        println!(
            "Private leaderboard of {} for Advent of Code {}.\n\n\
            {} indicates the user got both stars for that day,\n\
            {} means just the first star, and a {} means none.\n",
            owner_name.bold(),
            self.year.to_string().bold(),
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

        for header in ["         1111111111222222", "1234567890123456789012345"]
        {
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
}

impl Default for AocClientBuilder {
    fn default() -> Self {
        let session_cookie = None;
        let year = None;
        let day = None;
        let output_width = term_size::dimensions()
            .map(|(w, _)| w)
            .unwrap_or(DEFAULT_COL_WIDTH);
        let overwrite_file = false;
        let input_file = "input".to_string();
        let puzzle_file = "puzzle.md".to_string();

        Self {
            session_cookie,
            year,
            day,
            output_width,
            overwrite_file,
            input_file,
            puzzle_file,
        }
    }
}

impl AocClientBuilder {
    pub fn build(&self) -> AocResult<AocClient> {
        for (missing, field) in [
            (self.session_cookie.is_none(), "session cookie"),
            (self.year.is_none(), "year"),
            (self.day.is_none(), "day"),
        ] {
            if missing {
                return Err(AocError::ClientFieldMissing(field.to_string()));
            }
        }

        let day = self.day.unwrap();
        let year = self.year.unwrap();
        let timezone = FixedOffset::east_opt(RELEASE_TIMEZONE_OFFSET).unwrap();
        let local_datetime = NaiveDate::from_ymd_opt(year, DECEMBER, day)
            .ok_or(AocError::InvalidPuzzleDate(day, year))?
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let unlock_datetime = timezone
            .from_local_datetime(&local_datetime)
            .single()
            .ok_or(AocError::InvalidPuzzleDate(day, year))?;

        Ok(AocClient {
            session_cookie: self.session_cookie.clone().unwrap(),
            unlock_datetime,
            year: self.year.unwrap(),
            day: self.day.unwrap(),
            output_width: self.output_width,
            overwrite_file: self.overwrite_file,
            input_file: self.input_file.clone(),
            puzzle_file: self.puzzle_file.clone(),
        })
    }

    pub fn session_cookie(
        &mut self,
        session_cookie: impl AsRef<str>,
    ) -> AocResult<&mut Self> {
        let cookie = session_cookie.as_ref().trim();
        if cookie.is_empty() || !cookie.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(AocError::InvalidSessionCookie);
        }
        self.session_cookie = Some(cookie.to_string());
        Ok(self)
    }

    pub fn session_cookie_from_default_locations(
        &mut self,
    ) -> AocResult<&mut Self> {
        if let Ok(cookie) = env::var(SESSION_COOKIE_ENV_VAR) {
            debug!(
                "🍪 Loaded session cookie from '{SESSION_COOKIE_ENV_VAR}' \
                environment variable"
            );

            self.session_cookie(&cookie)
        } else {
            let path = if let Some(home_path) = home_dir()
                .map(|dir| dir.join(HIDDEN_SESSION_COOKIE_FILE))
                .filter(|file| file.exists())
            {
                home_path
            } else if let Some(config_path) = config_dir()
                .map(|dir| dir.join(SESSION_COOKIE_FILE))
                .filter(|file| file.exists())
            {
                config_path
            } else {
                return Err(AocError::SessionFileNotFound);
            };

            self.session_cookie_from_file(path)
        }
    }

    pub fn session_cookie_from_file<P: AsRef<Path>>(
        &mut self,
        file: P,
    ) -> AocResult<&mut Self> {
        let cookie = read_to_string(&file).map_err(|err| {
            AocError::SessionFileReadError {
                filename: file.as_ref().display().to_string(),
                source: err,
            }
        })?;

        debug!(
            "🍪 Loaded session cookie from '{}'",
            file.as_ref().display()
        );
        self.session_cookie(&cookie)
    }

    pub fn year(&mut self, year: PuzzleYear) -> AocResult<&mut Self> {
        if year >= FIRST_EVENT_YEAR {
            self.year = Some(year);
            Ok(self)
        } else {
            Err(AocError::InvalidEventYear(year))
        }
    }

    pub fn latest_event_year(&mut self) -> AocResult<&mut Self> {
        let now = FixedOffset::east_opt(RELEASE_TIMEZONE_OFFSET)
            .unwrap()
            .from_utc_datetime(&Utc::now().naive_utc());

        let year = if now.month() < DECEMBER {
            now.year() - 1
        } else {
            now.year()
        };

        self.year(year)
    }

    pub fn day(&mut self, day: PuzzleDay) -> AocResult<&mut Self> {
        if (FIRST_PUZZLE_DAY..=LAST_PUZZLE_DAY).contains(&day) {
            self.day = Some(day);
            Ok(self)
        } else {
            Err(AocError::InvalidPuzzleDay(day))
        }
    }

    pub fn latest_puzzle_day(&mut self) -> AocResult<&mut Self> {
        if self.year.is_none() {
            self.latest_event_year()?;
        }

        let event_year = self.year.unwrap();
        let now = FixedOffset::east_opt(RELEASE_TIMEZONE_OFFSET)
            .unwrap()
            .from_utc_datetime(&Utc::now().naive_utc());

        if event_year == now.year() && now.month() == DECEMBER {
            if now.day() <= LAST_PUZZLE_DAY {
                return self.day(now.day());
            } else {
                return self.day(LAST_PUZZLE_DAY);
            }
        } else if event_year < now.year() {
            // For past events, return the last puzzle day
            return self.day(LAST_PUZZLE_DAY);
        } else {
            // For future events, return the first puzzle day
            return self.day(FIRST_PUZZLE_DAY);
        }
    }

    pub fn output_width(&mut self, width: usize) -> AocResult<&mut Self> {
        if width > 0 {
            self.output_width = width;
            Ok(self)
        } else {
            Err(AocError::InvalidOutputWidth)
        }
    }

    pub fn overwrite_file(&mut self, overwrite: bool) -> &mut Self {
        self.overwrite_file = overwrite;
        self
    }
}

pub fn last_unlocked_day(year: PuzzleYear) -> Option<PuzzleDay> {
    let now = FixedOffset::east_opt(RELEASE_TIMEZONE_OFFSET)
        .unwrap()
        .from_utc_datetime(&Utc::now().naive_utc());

    if year == now.year() && now.month() == DECEMBER {
        if now.day() > LAST_PUZZLE_DAY {
            Some(LAST_PUZZLE_DAY)
        } else {
            Some(now.day())
        }
    } else if year >= FIRST_EVENT_YEAR && year < now.year() {
        Some(LAST_PUZZLE_DAY)
    } else {
        None
    }
}

fn http_client(
    session_cookie: &str,
    content_type: &str,
) -> AocResult<HttpClient> {
    let cookie_header =
        HeaderValue::from_str(&format!("session={}", session_cookie.trim()))
            .map_err(|_| AocError::InvalidSessionCookie)?;
    let content_type_header = HeaderValue::from_str(content_type).unwrap();
    let user_agent = format!("{PKG_REPO} {PKG_VERSION}");
    let user_agent_header = HeaderValue::from_str(&user_agent).unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, cookie_header);
    headers.insert(CONTENT_TYPE, content_type_header);
    headers.insert(USER_AGENT, user_agent_header);

    HttpClient::builder()
        .default_headers(headers)
        .redirect(Policy::none())
        .build()
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

impl Display for PuzzlePart {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PartOne => write!(f, "1"),
            Self::PartTwo => write!(f, "2"),
        }
    }
}

impl TryFrom<&String> for PuzzlePart {
    type Error = AocError;

    fn try_from(s: &String) -> Result<Self, Self::Error> {
        s.as_str().try_into()
    }
}

impl TryFrom<&str> for PuzzlePart {
    type Error = AocError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "1" => Ok(Self::PartOne),
            "2" => Ok(Self::PartTwo),
            _ => Err(AocError::InvalidPuzzlePart),
        }
    }
}
