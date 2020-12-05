use chrono::{Datelike, FixedOffset, NaiveDate, TimeZone, Utc};

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

pub fn download_input(
    _session_cookie: &str,
    opt_year: Option<PuzzleYear>,
    opt_day: Option<PuzzleDay>,
    filename: &str,
) -> Result<(), String> {
    let (year, day) = puzzle_day_year(opt_year, opt_day)?;

    eprintln!(
        "Downloading input for day {}, {} and saving it to '{}'...",
        day, year, filename
    );
    Ok(())
}

pub fn submit_answer(
    _session_cookie: &str,
    opt_year: Option<PuzzleYear>,
    opt_day: Option<PuzzleDay>,
    part: &str,
    answer: &str,
) -> Result<(), String> {
    let (year, day) = puzzle_day_year(opt_year, opt_day)?;

    eprintln!(
        "Submitting answer '{}' for part {}, day {}, {}...",
        answer, part, day, year
    );
    Ok(())
}
