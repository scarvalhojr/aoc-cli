mod aoc;
mod args;

use aoc::*;
use args::*;
use clap::Parser;
use home::home_dir;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::exit;

const SESSION_COOKIE_FILE: &str = ".adventofcode.session";
const DEFAULT_COL_WIDTH: usize = 80;

fn main() -> Result<(), String> {
    let args = Args::parse();

    let session = read_session_cookie(args.session);

    let width = args
        .width
        .or_else(|| term_size::dimensions().map(|(w, _)| w))
        .unwrap_or(DEFAULT_COL_WIDTH);

    match &args.command {
        Some(Command::Download) => {
            download_input(&session, args.year, args.day, &args.file)
        }
        Some(Command::Submit { part, answer }) => {
            submit_answer(&session, args.year, args.day, part, answer, width)
        }
        _ => read_puzzle(&session, args.year, args.day, width),
    }
}

fn read_session_cookie(session_file: Option<String>) -> String {
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
