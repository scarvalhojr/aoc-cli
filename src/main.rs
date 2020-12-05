mod aoc;

use aoc::*;
use clap::{
    crate_description, crate_version, value_t_or_exit, App, AppSettings, Arg,
    ArgMatches, Error, ErrorKind, SubCommand,
};

fn main() {
    let args = parse_args();
    let (year, day) = get_year_day(&args);

    let result = match args.subcommand() {
        ("download", Some(download_args)) => {
            let filename = download_args.value_of("filename").unwrap();
            download_input(year, day, filename)
        }
        ("submit", Some(submit_args)) => {
            let part = value_t_or_exit!(submit_args, "part", PuzzlePart);
            let answer = submit_args.value_of("answer").unwrap();
            submit_answer(year, day, part, &answer)
        }
        _ => unreachable!(),
    };

    match result {
        Ok(()) => eprintln!("Done!"),
        Err(err) => eprintln!("ERROR: {}", err),
    };
}

fn parse_args() -> ArgMatches<'static> {
    fn year_validator(value: String) -> Result<(), String> {
        if let Ok(year) = value.parse() {
            if is_valid_year(year) {
                return Ok(());
            }
        }
        Err(String::from("Invalid year"))
    }

    fn day_validator(value: String) -> Result<(), String> {
        if let Ok(day) = value.parse() {
            if is_valid_day(day) {
                return Ok(());
            }
        }
        Err(String::from("Invalid day"))
    }

    App::new("aoc")
        .version(crate_version!())
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::InferSubcommands)
        .setting(AppSettings::DeriveDisplayOrder)
        .subcommand(
            SubCommand::with_name("download")
                .about("Download puzzle input")
                .arg(
                    Arg::with_name("filename")
                        .help("Name of file where to save puzzle input")
                        .default_value("input"),
                ),
        )
        .subcommand(
            SubCommand::with_name("submit")
                .about("Submit puzzle answer")
                .arg(
                    Arg::with_name("part")
                        .possible_values(&["1", "2"])
                        .help("Puzzle part")
                        .required(true),
                )
                .arg(
                    Arg::with_name("answer")
                        .help("Puzzle answer")
                        .required(true),
                ),
        )
        .arg(
            Arg::with_name("year")
                .long("year")
                .short("y")
                .takes_value(true)
                .validator(year_validator)
                .help("Puzzle year"),
        )
        .arg(
            Arg::with_name("day")
                .short("d")
                .long("day")
                .takes_value(true)
                .validator(day_validator)
                .help("Puzzle day"),
        )
        .get_matches()
}

fn get_year_day(args: &ArgMatches) -> (PuzzleYear, PuzzleDay) {
    let year = if args.is_present("year") {
        value_t_or_exit!(args, "year", PuzzleYear)
    } else {
        latest_puzzle_year()
    };

    let day = if args.is_present("day") {
        value_t_or_exit!(args, "day", PuzzleDay)
    } else if let Some(curr_day) = current_event_day(year) {
        curr_day
    } else {
        Error::with_description(
            format!("Day could not be inferred for {}.", year).as_str(),
            ErrorKind::MissingRequiredArgument,
        )
        .exit();
    };

    if !puzzle_unlocked(year, day) {
        Error::with_description(
            format!("Puzzle {} of {} is still locked.", day, year).as_str(),
            ErrorKind::ValueValidation,
        )
        .exit();
    }

    (year, day)
}
