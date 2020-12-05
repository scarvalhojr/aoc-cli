use chrono::{Datelike, FixedOffset, NaiveDate, TimeZone, Utc};

pub type PuzzleYear = i32;
pub type PuzzleDay = u32;
pub type PuzzlePart = u32;

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

pub fn latest_puzzle_year() -> PuzzleYear {
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

pub fn download_input(
    year: PuzzleYear,
    day: PuzzleDay,
    filename: &str,
) -> Result<(), String> {
    eprintln!(
        "Downloading input for day {}, {} and saving it to '{}'...",
        day, year, filename
    );
    Ok(())
}

pub fn submit_answer(
    year: PuzzleYear,
    day: PuzzleDay,
    part: PuzzlePart,
    answer: &str,
) -> Result<(), String> {
    eprintln!(
        "Submitting answer '{}' for part {}, day {}, {}...",
        answer, part, day, year
    );
    Ok(())
}
