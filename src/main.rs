mod aoc;

use aoc::*;
use clap::{
    crate_description, crate_version, value_t_or_exit, App, Arg, ArgMatches,
};

fn main() {
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

    let result = match args.value_of("command").unwrap() {
        cmd if cmd == "download" || cmd == "d" => {
            let filename = args.value_of("filename").unwrap();
            download_input(year, day, filename)
        }
        cmd if cmd == "submit" || cmd == "s" => {
            let part = args.value_of("part").unwrap();
            let answer = args.value_of("answer").unwrap();
            submit_answer(year, day, part, answer)
        }
        _ => unreachable!(),
    };

    match result {
        Ok(()) => eprintln!("Done!"),
        Err(err) => eprintln!("Error: {}", err),
    };
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
                .possible_values(&["download", "d", "submit", "s"])
                .required(true)
                .help("Command to execute")
        )
        .arg(
            Arg::with_name("part")
                .possible_values(&["1", "2"])
                .required_ifs(&[("command", "submit"), ("command", "s")])
                .help("Puzzle part (required for submit)")
        )
        .arg(
            Arg::with_name("answer")
                .required_ifs(&[("command", "submit"), ("command", "s")])
                .help("Puzzle answer (required for submit")
        )
        .arg(
            Arg::with_name("year")
                .long("year")
                .short("y")
                .takes_value(true)
                .validator(year_validator)
                .help("Puzzle year [default: year of current or last Advent of Code]"),
        )
        .arg(
            Arg::with_name("day")
                .short("d")
                .long("day")
                .takes_value(true)
                .validator(day_validator)
                .help("Puzzle day [default: last unlocked day (during Advent of Code)]"),
        )
        .arg(
            Arg::with_name("filename")
                .short("f")
                .long("filename")
                .takes_value(true)
                .help("File name where to save puzzle input")
                .default_value("input"),
        )
        .get_matches()
}
