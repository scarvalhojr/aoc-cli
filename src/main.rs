mod aoc;
mod args;

use aoc::*;
use args::*;
use clap::{crate_description, crate_name, Parser};
use env_logger::{Builder, Env};
use exit_code::*;
use log::{error, info, warn, LevelFilter};
use std::process::exit;

const DEFAULT_COL_WIDTH: usize = 80;

fn main() {
    let args = Args::parse();

    setup_log(&args);

    info!("🎄 {} - {}", crate_name!(), crate_description!());

    match run(&args) {
        Ok(_) => exit(SUCCESS),
        Err(err) => {
            error!("🔔 {err}");
            let exit_code = match err {
                AocError::InvalidPuzzleDate(..) => USAGE_ERROR,
                AocError::InvalidEventYear(..) => USAGE_ERROR,
                AocError::NonInferablePuzzleDate(..) => USAGE_ERROR,
                AocError::LockedPuzzle(..) => USAGE_ERROR,
                AocError::MissingConfigDir => NO_INPUT,
                AocError::SessionFileReadError { .. } => IO_ERROR,
                AocError::InvalidSessionCookie { .. } => DATA_ERROR,
                AocError::HttpRequestError { .. } => FAILURE,
                AocError::AocResponseError => FAILURE,
                AocError::PrivateLeaderboardNotAvailable => FAILURE,
                AocError::FileWriteError { .. } => CANNOT_CREATE,
            };

            if exit_code == FAILURE {
                // Unexpected responses from adventofcode.com including
                // HTTP 302/400/500 may be due to invalid or expired cookies
                warn!(
                    "🍪 Your session cookie may be invalid or expired, try \
                    logging in again"
                );
            }

            exit(exit_code);
        }
    };
}

fn setup_log(args: &Args) {
    let mut log_builder =
        Builder::from_env(Env::default().default_filter_or("info"));

    if args.quiet {
        log_builder.filter_module("aoc", LevelFilter::Error);
    } else if args.debug {
        log_builder.filter_module("aoc", LevelFilter::Debug);
    }

    log_builder.format_timestamp(None).init();
}

fn run(args: &Args) -> AocResult<()> {
    let session = load_session_cookie(&args.session_file)?;

    let width = args
        .width
        .or_else(|| term_size::dimensions().map(|(w, _)| w))
        .unwrap_or(DEFAULT_COL_WIDTH);

    match &args.command {
        Some(Command::Calendar) => calendar(args, &session, width),
        Some(Command::Download) => download(args, &session),
        Some(Command::Submit { part, answer }) => {
            submit(args, &session, width, part, answer)
        }
        Some(Command::PrivateLeaderboard { leaderboard_id }) => {
            private_leaderboard(args, &session, leaderboard_id)
        }
        _ => read(args, &session, width),
    }
}
