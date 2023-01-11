use aoc_client::{LeaderboardId, PuzzleDay, PuzzleYear};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, infer_subcommands = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Puzzle day [default: last unlocked day (during Advent of Code month)]
    #[arg(short, long, global = true)]
    pub day: Option<PuzzleDay>,

    /// Puzzle year [default: year of current or last Advent of Code event]
    #[arg(short, long, global = true)]
    pub year: Option<PuzzleYear>,

    /// Path to session cookie file [default: ~/.adventofcode.session]
    #[arg(short, long, alias = "session", global = true, value_name = "PATH")]
    pub session_file: Option<String>,

    /// Width at which to wrap output [default: terminal width]
    #[arg(short, long, global = true)]
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
    /// Show Advent of Code calendar and stars collected
    #[command(visible_alias = "c")]
    Calendar,

    /// Save puzzle description and input to files
    #[command(visible_alias = "d")]
    Download,

    /// Read puzzle statement (the default command)
    #[command(visible_alias = "r")]
    Read,

    /// Submit puzzle answer
    #[command(visible_alias = "s")]
    Submit {
        /// Puzzle part
        #[arg(value_parser = ["1", "2"])]
        part: String,

        /// Puzzle answer
        answer: String,
    },

    /// Show the state of a private leaderboard
    #[command(visible_alias = "p")]
    PrivateLeaderboard {
        /// Private leaderboard ID
        leaderboard_id: LeaderboardId,
    },
}
