use crate::{is_valid_day, is_valid_year, PuzzleDay, PuzzleYear};
use clap::{Parser, Subcommand};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(version, about, infer_subcommands = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Puzzle day [default: last unlocked day (during Advent of Code month)]
    #[arg(short, long, global = true, value_parser = valid_day)]
    pub day: Option<PuzzleDay>,

    /// Puzzle year [default: year of current or last Advent of Code event]
    #[arg(short, long, global = true, value_parser = valid_year)]
    pub year: Option<PuzzleYear>,

    /// Path to session cookie file [default: ~/.adventofcode.session]
    #[arg(short, long, alias = "session", global = true, value_name = "PATH")]
    pub session_file: Option<String>,

    /// Width at which to wrap output [default: terminal width]
    #[arg(short, long, global = true, value_parser = valid_width)]
    pub width: Option<usize>,

    /// Overwrite files if they already exist
    #[arg(short, long, global = true)]
    pub overwrite: bool,

    /// Download puzzle input only
    #[arg(short = 'I', long, global = true)]
    pub input_only: bool,

    /// Download puzzle description only
    #[arg(
        short = 'P',
        short_alias = 'D',
        long,
        alias = "description-only",
        global = true,
        conflicts_with = "input_only"
    )]
    pub puzzle_only: bool,

    /// Path where to save puzzle input
    #[arg(
        short,
        long,
        alias = "input",
        global = true,
        value_name = "PATH",
        default_value = "input"
    )]
    pub input_file: String,

    /// Path where to save puzzle description
    #[arg(
        short,
        long,
        alias = "puzzle",
        global = true,
        value_name = "PATH",
        default_value = "puzzle.md"
    )]
    pub puzzle_file: String,

    /// Restrict log messages to errors only
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Enable debug logging
    #[arg(long, global = true, conflicts_with = "quiet")]
    pub debug: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Read puzzle statement (the default command)
    #[command(visible_alias = "r")]
    Read,

    /// Save puzzle description and input to files
    #[command(visible_alias = "d")]
    Download,

    /// Submit puzzle answer
    #[command(visible_alias = "s")]
    Submit {
        /// Puzzle part
        #[arg(value_parser = ["1", "2"])]
        part: String,

        /// Puzzle answer
        answer: String,
    },

    /// Get private leaderboard results
    #[command(visible_alias = "pr")]
    PrivateLeaderboard { leaderboard: String },
}

fn convert_number<T: FromStr>(s: &str) -> Result<T, String>
where
    <T as FromStr>::Err: Display,
{
    s.parse::<T>().map_err(|err| format!("{}", err))
}

fn valid_day(s: &str) -> Result<PuzzleDay, String> {
    convert_number(s).and_then(|day| {
        if is_valid_day(day) {
            Ok(day)
        } else {
            Err("invalid Advent of Code day".to_string())
        }
    })
}

fn valid_year(s: &str) -> Result<PuzzleYear, String> {
    convert_number(s).and_then(|day| {
        if is_valid_year(day) {
            Ok(day)
        } else {
            Err("invalid Advent of Code year".to_string())
        }
    })
}

fn valid_width(s: &str) -> Result<usize, String> {
    convert_number(s).and_then(|width| {
        if width > 0 {
            Ok(width)
        } else {
            Err("invalid output width".to_string())
        }
    })
}
