mod aoc;

use aoc::*;
use clap::{
    crate_description, crate_version, value_t_or_exit, App, Arg, ArgMatches,
};
use home::home_dir;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::exit;

const SESSION_COOKIE_FILE: &str = ".adventofcode.session";

fn main() -> Result<(), String> {
    let args = parse_args();
    let year = if args.is_present("year") {
        Some(value_t_or_exit!(args, "year", PuzzleYear))
    } else {
        None
    };
    let day = if args.is_present("day") {
        Some(value_t_or_exit!(args, "day", PuzzleDay))
    } else {
        None
    };

    let session_cookie = read_session_cookie(args.value_of("session"));

    match args.value_of("command").unwrap() {
        cmd if cmd == "download" || cmd == "d" => {
            let filename = args.value_of("file").unwrap();
            download_input(&session_cookie, year, day, filename)
        }
        cmd if cmd == "submit" || cmd == "s" => {
            let part = args.value_of("part").unwrap();
            let answer = args.value_of("answer").unwrap();
            submit_answer(&session_cookie, year, day, part, answer)
        }
        cmd if cmd == "read" || cmd == "r" => {
            read_puzzle(&session_cookie, year, day)
        }
        _ => unreachable!(),
    }
}

fn parse_args() -> ArgMatches<'static> {
    fn year_validator(value: String) -> Result<(), String> {
        match value.parse() {
            Ok(year) if is_valid_year(year) => Ok(()),
            Ok(_) => Err(String::from("Not an Advent of Code year")),
            _ => Err(String::from("Invalid number")),
        }
    }

    fn day_validator(value: String) -> Result<(), String> {
        match value.parse() {
            Ok(day) if is_valid_day(day) => Ok(()),
            Ok(_) => Err(String::from("Not an Advent of Code day")),
            _ => Err(String::from("Invalid number")),
        }
    }

    App::new("aoc")
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            Arg::with_name("command")
                .value_name("COMMAND")
                .possible_values(&["download", "d", "read", "r", "submit", "s"])
                .required(true)
                .help("Command to execute")
                .long_help(
                    "Commands are 'read' (or 'r') to read puzzle description, \
                    'download' (or 'd') to download puzzle input, and 'submit' \
                    (or 's') to submit a puzzle answer.",
                ),
        )
        .arg(
            Arg::with_name("part")
                .value_name("PART")
                .possible_values(&["1", "2"])
                .required_ifs(&[("command", "submit"), ("command", "s")])
                .help("Puzzle part (required when submitting answers)"),
        )
        .arg(
            Arg::with_name("answer")
                .value_name("ANSWER")
                .required_ifs(&[("command", "submit"), ("command", "s")])
                .help("Puzzle answer (required when submitting answers)"),
        )
        .arg(
            Arg::with_name("year")
                .long("year")
                .short("y")
                .value_name("YEAR")
                .takes_value(true)
                .validator(year_validator)
                .help(
                    "Puzzle year [default: year of current or last Advent of \
                    Code]",
                ),
        )
        .arg(
            Arg::with_name("day")
                .short("d")
                .long("day")
                .value_name("DAY")
                .takes_value(true)
                .validator(day_validator)
                .help(
                    "Puzzle day [default: last unlocked day (during Advent of \
                    Code)]",
                ),
        )
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("PATH")
                .takes_value(true)
                .help("Path to file where to save puzzle input")
                .default_value("input"),
        )
        .arg(
            Arg::with_name("session")
                .short("s")
                .long("session")
                .value_name("PATH")
                .takes_value(true)
                .help(
                    "Path to session cookie file [default \
                    ~/.adventofcode.session]",
                ),
        )
        .get_matches()
}

fn read_session_cookie(session_file: Option<&str>) -> String {
    let path = if let Some(file) = session_file {
        PathBuf::from(file)
    } else if let Some(dir) = home_dir() {
        dir.join(SESSION_COOKIE_FILE)
    } else {
        eprintln!("error: Failed to find home directory.");
        exit(2);
    };

    match read_to_string(&path) {
        Ok(cookie) => {
            eprintln!("Loaded session cookie from \"{}\".", path.display());
            cookie
        }
        Err(err) => {
            eprintln!(
                "error: Failed to read session cookie from \"{}\": {}",
                path.display(),
                err
            );
            exit(2);
        }
    }
}
