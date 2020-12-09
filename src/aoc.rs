use chrono::{Datelike, FixedOffset, NaiveDate, TimeZone, Utc};
use html2text::from_read;
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, COOKIE};
use reqwest::redirect::Policy;
use std::fs::OpenOptions;
use std::io::Write;

pub type PuzzleYear = i32;
pub type PuzzleDay = u32;

const FIRST_EVENT_YEAR: PuzzleYear = 2015;
const DECEMBER: u32 = 12;
const FIRST_PUZZLE_DAY: PuzzleDay = 1;
const LAST_PUZZLE_DAY: PuzzleDay = 25;
const RELEASE_TIMEZONE_OFFSET: i32 = -5 * 3600;

pub fn is_valid_year(year: PuzzleYear) -> bool {
    year >= FIRST_EVENT_YEAR
}

pub fn is_valid_day(day: PuzzleDay) -> bool {
    day >= FIRST_PUZZLE_DAY && day <= LAST_PUZZLE_DAY
}

pub fn latest_event_year() -> PuzzleYear {
    let now = FixedOffset::east(RELEASE_TIMEZONE_OFFSET)
        .from_utc_datetime(&Utc::now().naive_utc());

    if now.month() < DECEMBER {
        now.year() - 1
    } else {
        now.year()
    }
}

pub fn current_event_day(year: PuzzleYear) -> Option<PuzzleDay> {
    let now = FixedOffset::east(RELEASE_TIMEZONE_OFFSET)
        .from_utc_datetime(&Utc::now().naive_utc());

    if now.month() == DECEMBER && now.year() == year {
        Some(now.day())
    } else {
        None
    }
}

pub fn puzzle_unlocked(year: PuzzleYear, day: PuzzleDay) -> bool {
    let timezone = FixedOffset::east(RELEASE_TIMEZONE_OFFSET);
    let now = timezone.from_utc_datetime(&Utc::now().naive_utc());
    let unlock_time = timezone
        .from_local_datetime(
            &NaiveDate::from_ymd(year, DECEMBER, day).and_hms(0, 0, 0),
        )
        .single();

    if let Some(time) = unlock_time {
        now.signed_duration_since(time).num_milliseconds() >= 0
    } else {
        false
    }
}

fn puzzle_day_year(
    opt_year: Option<PuzzleYear>,
    opt_day: Option<PuzzleDay>,
) -> Result<(PuzzleYear, PuzzleDay), String> {
    let year = opt_year.unwrap_or_else(latest_event_year);
    let day = opt_day
        .or_else(|| current_event_day(year))
        .ok_or_else(|| format!("Could not infer puzzle day for {}.", year))?;

    if !puzzle_unlocked(year, day) {
        return Err(format!("Puzzle {} of {} is still locked.", day, year));
    }

    Ok((year, day))
}

fn build_client(
    session_cookie: &str,
    content_type: &str,
) -> Result<Client, String> {
    let cookie_header =
        HeaderValue::from_str(&format!("session={}", session_cookie.trim()))
            .map_err(|err| format!("Invalid session cookie: {}", err))?;
    let content_type_header = HeaderValue::from_str(content_type).unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, cookie_header);
    headers.insert(CONTENT_TYPE, content_type_header);

    Client::builder()
        .default_headers(headers)
        .redirect(Policy::none())
        .build()
        .map_err(|err| err.to_string())
}

pub fn download_input(
    session_cookie: &str,
    opt_year: Option<PuzzleYear>,
    opt_day: Option<PuzzleDay>,
    filename: &str,
) -> Result<(), String> {
    let (year, day) = puzzle_day_year(opt_year, opt_day)?;

    eprintln!("Downloading input for day {}, {}...", day, year);
    let url = format!("https://adventofcode.com/{}/day/{}/input", year, day);
    let content_type = "text/plain";
    let puzzle_input = build_client(session_cookie, content_type)?
        .get(&url)
        .send()
        .and_then(|response| response.error_for_status())
        .and_then(|response| response.text())
        .map_err(|err| err.to_string())?;

    eprintln!("Saving puzzle input to \"{}\"...", filename);
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(filename)
        .map_err(|err| format!("Failed to create file: {}", err))?
        .write(puzzle_input.as_bytes())
        .map_err(|err| format!("Failed to write to file: {}", err))?;

    eprintln!("Done!");
    Ok(())
}

pub fn submit_answer(
    session_cookie: &str,
    opt_year: Option<PuzzleYear>,
    opt_day: Option<PuzzleDay>,
    part: &str,
    answer: &str,
) -> Result<(), String> {
    let (year, day) = puzzle_day_year(opt_year, opt_day)?;

    eprintln!(
        "Submitting answer for part {}, day {}, {}...",
        part, day, year
    );
    let url = format!("https://adventofcode.com/{}/day/{}/answer", year, day);
    let content_type = "application/x-www-form-urlencoded";
    let response = build_client(session_cookie, content_type)?
        .post(&url)
        .body(format!("level={}&answer={}", part, answer))
        .send()
        .and_then(|response| response.error_for_status())
        .and_then(|response| response.text())
        .map_err(|err| err.to_string())?;

    let result = Regex::new(r"(?i)(?s)<main>(?P<main>.*)</main>")
        .unwrap()
        .captures(&response)
        .ok_or_else(|| "Failed to parse response")?
        .name("main")
        .unwrap()
        .as_str();

    let term_width = term_size::dimensions().map(|(w,_)| w).unwrap_or(80);
    println!("\n{}", from_read(result.as_bytes(), term_width));
    Ok(())
}

pub fn read_puzzle(
    session_cookie: &str,
    opt_year: Option<PuzzleYear>,
    opt_day: Option<PuzzleDay>,
) -> Result<(), String> {
    let (year, day) = puzzle_day_year(opt_year, opt_day)?;

    let url = format!("https://adventofcode.com/{}/day/{}", year, day);
    let content_type = "text/html";
    let response = build_client(session_cookie, content_type)?
        .get(&url)
        .send()
        .and_then(|response| response.error_for_status())
        .and_then(|response| response.text())
        .map_err(|err| err.to_string())?;

    let description = Regex::new(r"(?i)(?s)<main>(?P<main>.*)</main>")
        .unwrap()
        .captures(&response)
        .ok_or_else(|| "Failed to parse puzzle description page")?
        .name("main")
        .unwrap()
        .as_str();

    let term_width = term_size::dimensions().map(|(w,_)| w).unwrap_or(80);
    println!("\n{}", from_read(description.as_bytes(), term_width));
    Ok(())

}
