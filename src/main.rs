mod aoc;
mod args;

use aoc::*;
use args::*;
use clap::Parser;
use std::env;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::exit;

const SESSION_COOKIE_FILE: &str = "adventofcode.session";
const HIDDEN_SESSION_COOKIE_FILE: &str = ".adventofcode.session";
const SESSION_COOKIE_ENV: &str = "ADVENT_OF_CODE_SESSION";
const DEFAULT_COL_WIDTH: usize = 80;

fn main() -> Result<(), String> {
    let args = Args::parse();

    let session = read_session_cookie(&args.session_file);

    let width = args
        .width
        .or_else(|| term_size::dimensions().map(|(w, _)| w))
        .unwrap_or(DEFAULT_COL_WIDTH);

    match &args.command {
        Some(Command::Download) => download(&args, &session),
        Some(Command::Submit { part, answer }) => {
            submit(&args, &session, width, part, answer)
        }
        _ => read(&args, &session, width),
    }
}

fn read_session_cookie(session_file: &Option<String>) -> String {
    let path = if let Some(file) = session_file {
        PathBuf::from(file)
    } else if let Ok(cookie) = env::var(SESSION_COOKIE_ENV) {
        return cookie;
    } else if let Some(file) = dirs::home_dir()
        .map(|dir| dir.join(HIDDEN_SESSION_COOKIE_FILE))
        .filter(|file| file.exists())
    {
        file
    } else if let Some(dir) = dirs::config_dir() {
        dir.join(SESSION_COOKIE_FILE)
    } else {
        eprintln!("error: Failed to find config directory.");
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
