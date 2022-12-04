# aoc-cli

[![Build and test](https://github.com/scarvalhojr/aoc-cli/actions/workflows/build-and-test.yml/badge.svg)](https://github.com/scarvalhojr/aoc-cli/actions/workflows/build-and-test.yml)
[![Clippy and format](https://github.com/scarvalhojr/aoc-cli/actions/workflows/clippy-and-fmt.yml/badge.svg)](https://github.com/scarvalhojr/aoc-cli/actions/workflows/clippy-and-fmt.yml)
[![Release](https://github.com/scarvalhojr/aoc-cli/actions/workflows/release.yml/badge.svg)](https://github.com/scarvalhojr/aoc-cli/actions/workflows/release.yml)
[![crates.io](https://img.shields.io/crates/v/aoc-cli.svg)](https://crates.io/crates/aoc-cli)

[Advent of Code](https://adventofcode.com) command-line tool.

Read Advent of Code puzzle descriptions, download puzzle input, submit answers
and check if they are correct, all from the comfort of your terminal.

## Features

- Validate arguments (year, day, part) and check if puzzle is unlocked.
- Infer puzzle day when possible (last unlocked puzzle during Advent of Code
  month).
- If year is not provided, it is assumed to be the current or the last Advent of
  Code event.
- Load Advent of Code session cookie from a file.
- Save puzzle description to a file in Markdown format.

## Installation options

### Compile from source

- Install a recent version of Rust using [rustup](https://rustup.rs/) or update
it with:
```
# rustup update
```

`aoc-cli` relies on [openssl-sys](https://crates.io/crates/openssl-sys), which
requires OpenSSL libraries and headers for compiling it. On Linux, you need
to install a package with OpenSSL development headers such as `libssl-dev` or
`openssl-devel`.

- Install `aoc-cli` with cargo:
```
# cargo install aoc-cli
```
### Homebrew

On macOS and Linux, use [Homebrew](https://brew.sh):

```
# brew install scarvalhojr/tap/aoc-cli
```

### Windows Package Manager

On Windows 10 and 11, use the
[Windows Package Manager](https://learn.microsoft.com/en-us/windows/package-manager/winget/)
command line tool:

```
# winget install aoc-cli
```

### Download release artifacts

Executables for selected platforms are available in
[GitHub releases](https://github.com/scarvalhojr/aoc-cli/releases). Simply
download and extract the file. An installer is also available for Windows.

The Linux package is statically-linked with
OpenSSL and [musl C library](https://www.musl-libc.org/), and it should just
work on most Linux distributions.

The macOS and Windows packages should
automatically detect installed OpenSSL libraries. The MSVC Windows packages
require the redistributable Visual C++ runtime library, whereas the MinGW
packages require the [Minimalist GNU for Windows](https://osdn.net/projects/mingw/)
runtime libraries.

* x86 64-bit Linux - `aoc-cli-<version>-x86_64-unknown-linux-musl.tar.gz`
* x86 64-bit macOS (10.7 or newer) - `aoc-cli-<version>-x86_64-apple-darwin.tar.gz`
* x86 64-bit Windows installer - `aoc-cli-<version>-x86_64.msi`
* x86 64-bit Windows MSVC (Windows 7 or newer) - `aoc-cli-<version>-x86_64-pc-windows-msvc.zip`
* x86 64-bit Windows MinGW (Windows 7 or newer) - `aoc-cli-<version>-x86_64-pc-windows-gnu.zip`
* i686 32-bit Windows MSVC (Windows 7 or newer) - `aoc-cli-<version>-i686-pc-windows-msvc.zip`
* i686 32-bit Windows MinGW (Windows 7 or newer) - `aoc-cli-<version>-i686-pc-windows-gnu.zip`

## Session cookie

Different Advent of Code users get different puzzle input. To download your
input and submit your answer, you need an adventofcode.com session cookie. To
obtain your session cookie, login to the
[Advent of Code](https://adventofcode.com) website and inspect the `session`
value of the cookie that gets stored in your browser. Put the session number (a
long hex string) either in a file called `.adventofcode.session` in your home
directory or `adventofcode.session` in your users config directory (`~/.config` on Linux, `~/Library/Application Support` on macOS or `~\AppData\Roaming` on Windows). This file should only contain your session number, in a single line.

## Usage

```
# aoc help

Advent of Code command-line tool

Usage: aoc [OPTIONS] [COMMAND]

Commands:
  read      Read puzzle statement (the default command) [aliases: r]
  download  Save puzzle description and input to files [aliases: d]
  submit    Submit puzzle answer [aliases: s]
  help      Print this message or the help of the given subcommand(s)

Options:
  -d, --day <DAY>            Puzzle day [default: last unlocked day (during Advent of Code month)]
  -y, --year <YEAR>          Puzzle year [default: year of current or last Advent of Code event]
  -s, --session-file <PATH>  Path to session cookie file [default: ~/.adventofcode.session]
  -w, --width <WIDTH>        Width at which to wrap output [default: terminal width]
  -o, --overwrite            Overwrite files if they already exist
  -I, --input-only           Download puzzle input only
  -D, --description-only     Download puzzle description only
  -i, --input-file <PATH>    Path where to save puzzle input [default: input]
  -p, --puzzle-file <PATH>   Path where to save puzzle description [default: puzzle.md]
  -h, --help                 Print help information
  -V, --version              Print version information
```

```
# aoc help submit

Submit puzzle answer

Usage: aoc submit [OPTIONS] <PART> <ANSWER>

Arguments:
  <PART>    Puzzle part [possible values: 1, 2]
  <ANSWER>  Puzzle answer

Options:
  -d, --day <DAY>            Puzzle day [default: last unlocked day (during Advent of Code month)]
  -y, --year <YEAR>          Puzzle year [default: year of current or last Advent of Code event]
  -s, --session-file <PATH>  Path to session cookie file [default: ~/.adventofcode.session]
  -w, --width <WIDTH>        Width at which to wrap output [default: terminal width]
  -o, --overwrite            Overwrite files if they already exist
  -I, --input-only           Download puzzle input only
  -D, --description-only     Download puzzle description only
  -i, --input-file <PATH>    Path where to save puzzle input [default: input]
  -p, --puzzle-file <PATH>   Path where to save puzzle description [default: puzzle.md]
  -h, --help                 Print help information
```

### Read puzzle description

Read today's puzzle (if today is an Advent of Code day) in plain text from the
comfort of your terminal.

```
# aoc read

Loaded session cookie from "/home/user/.adventofcode.session".

## --- Day 2: Rock Paper Scissors ---

The Elves begin to set up camp on the beach. To decide whose tent gets to be
closest to the snack storage, a giant [Rock Paper Scissors][1] tournament is
already in progress.
...
```

### Download puzzle input

Download description and input for today's puzzle and save them to files. By
default the description is saved to "puzzle.md" and the input is saved to
"input" in the current directory:

```
# aoc download

Loaded session cookie from "/home/user/.adventofcode.session".
Fetching puzzle for day 2, 2022...
Saving puzzle description to "puzzle.md"...
Downloading input for day 2, 2022...
Saving puzzle input to "input"...
Done!
```

### Submit puzzle answers

Submit the answer to part 1 of today's puzzle (in this example, the answer is
999):

```
# aoc submit 1 999

Loaded session cookie from "/home/user/.adventofcode.session".
Submitting answer for part 1, day 2, 2022...

That's the right answer! You are one gold star closer to saving your vacation. [[Continue to Part Two]][1]

[1] /2022/day/2#part2
```

### Command abbreviations

Any prefix of a command can be used instead of the full command name. For instance:
- Instead of `aoc read`, type `aoc r`, `aoc re` or `aoc rea`.
- Instead of `aoc download`, type `aoc d`, `aoc do`, `aoc dow`, `aoc down`, etc.
- Instead of `aoc submit`, type `aoc s`, `aoc su`, `aoc sub`, etc.

### More examples

Download puzzle from a previous day (assumes the current year):

```
# aoc download --day 1
```

Download puzzle input from a previous year and save it to a file with a given
name:

```
# aoc download --year 2015 --day 1 --input-only --input-file /home/user/aoc/2015/1/input
```

An attempt to download a puzzle that is still locked fails
(puzzles unlock every day between 1st and 25th of December at midnight
EST/UTC-5):

```
# aoc download --year 2030 --day 25

Loaded session cookie from "/home/user/.adventofcode.session".
Error: Puzzle 25 of 2030 is still locked.
```

Submit the answer to a previous year:
```
# aoc s 1 999 -y 2015 -d 1
```

Specify path to session cookie file:
```
# aoc download --session-file /tmp/.aoc.session
```

## Contribute

Feedback and pull requests are welcome.

## Support Advent of Code

Advent of Code is a free online Advent calendar of small programming puzzles
created by [Eric Wastl](http://was.tl/) and maintained by volunteers. Please
consider [supporting their work](https://adventofcode.com/support).
