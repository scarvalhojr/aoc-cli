mod aoc;
mod args;

use aoc::*;
use args::*;
use clap::{crate_description, crate_name, Parser};
use env_logger::{Builder, Env};
use exit_code::*;
use log::{error, info, warn, LevelFilter};
use std::process::exit;

fn main() {
    let args = Args::parse();

    setup_log(&args);

    info!("🎄 {} - {}", crate_name!(), crate_description!());

    match build_client(&args).and_then(|client| run(&args, client)) {
        Ok(_) => exit(SUCCESS),
        Err(err) => {
            error!("🔔 {err}");
            let exit_code = match err {
                AocError::InvalidPuzzleDate(..) => USAGE_ERROR,
                AocError::InvalidEventYear(..) => USAGE_ERROR,
                AocError::InvalidEventDay(..) => USAGE_ERROR,
                AocError::NonInferablePuzzleDate(..) => USAGE_ERROR,
                AocError::LockedPuzzle(..) => USAGE_ERROR,
                AocError::SessionFileNotFound => NO_INPUT,
                AocError::SessionFileReadError { .. } => IO_ERROR,
                AocError::InvalidSessionCookie { .. } => DATA_ERROR,
                AocError::HttpRequestError { .. } => FAILURE,
                AocError::AocResponseError => FAILURE,
                AocError::PrivateLeaderboardNotAvailable => FAILURE,
                AocError::FileWriteError { .. } => CANNOT_CREATE,
                AocError::ClientFieldMissing(..) => USAGE_ERROR,
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

fn build_client(args: &Args) -> AocResult<AocClient> {
    let mut builder = AocClient::builder();

    if let Some(file) = &args.session_file {
        builder.session_cookie_from_file(file)?;
    } else {
        builder.session_cookie_from_default_locations()?;
    }

    match (&args.year, &args.day) {
        (Some(y), Some(d)) => builder.year(*y)?.day(*d)?,
        (Some(y), None) => builder.year(*y)?.latest_puzzle_day()?,
        (None, Some(d)) => builder.latest_event_year()?.day(*d)?,
        (None, None) => builder.latest_puzzle_day()?,
    };

    if let Some(width) = &args.width {
        builder.output_width(*width);
    }

    builder.overwrite_file(args.overwrite).build()
}

fn run(args: &Args, client: AocClient) -> AocResult<()> {
    match &args.command {
        Some(Command::Calendar) => client.show_calendar(),
        Some(Command::Download) => {
            if !args.input_only {
                client.save_puzzle_markdown()?;
            }
            if !args.puzzle_only {
                client.save_input()?;
            }
            Ok(())
        }
        Some(Command::Submit { part, answer }) => {
            client.submit_answer(part, answer)
        }
        Some(Command::PrivateLeaderboard { leaderboard_id }) => {
            client.show_private_leaderboard(leaderboard_id)
        }
        _ => client.show_puzzle_text(),
    }
}
