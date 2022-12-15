# aoc-cli

[![Build and test](https://github.com/scarvalhojr/aoc-cli/actions/workflows/build-and-test.yml/badge.svg)](https://github.com/scarvalhojr/aoc-cli/actions/workflows/build-and-test.yml)
[![Clippy and format](https://github.com/scarvalhojr/aoc-cli/actions/workflows/clippy-and-fmt.yml/badge.svg)](https://github.com/scarvalhojr/aoc-cli/actions/workflows/clippy-and-fmt.yml)
[![Release](https://github.com/scarvalhojr/aoc-cli/actions/workflows/release.yml/badge.svg)](https://github.com/scarvalhojr/aoc-cli/actions/workflows/release.yml)
[![crates.io](https://img.shields.io/crates/v/aoc-cli.svg)](https://crates.io/crates/aoc-cli)

## Advent of Code command-line tool 🎄

Read [Advent of Code](https://adventofcode.com) puzzle descriptions, download
puzzle input, submit answers and check if they are correct, all from the
comfort of your terminal.

## Features ⭐️

- Validate arguments (year, day, part) and check if puzzle is unlocked.
- Infer puzzle day when possible (last unlocked puzzle during Advent of Code
  month).
- If year is not provided, it is assumed to be the current or the last Advent of
  Code event.
- Load Advent of Code session cookie from a file.
- Save puzzle description to a file in Markdown format.
- Show the state of private leaderboards.

## Installation options 🎅

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

## Session cookie 🍪

Different Advent of Code users get different puzzle input. To download your
input and submit your answer, you need an adventofcode.com session cookie. To
obtain your session cookie, login to the
[Advent of Code](https://adventofcode.com) website and inspect the `session`
value of the cookie that gets stored in your browser - see instructions
[here](https://www.cookieyes.com/blog/how-to-check-cookies-on-your-website-manually).

The session cookie (a long hex string) must be provided in a single line (no
line breaks) in one of the following ways (listed in order of precedence):
1. In a file specified via the `--session-file` command line option.
2. In an `ADVENT_OF_CODE_SESSION` environment variable.
3. In a file called `.adventofcode.session` (note the dot) in your home
   directory (`/home/alice` on Linux, `C:\Users\Alice` on Windows,
   `/Users/Alice` on macOS).
4. In a file called `adventofcode.session` (no dot) in your user's config
   directory (`/home/alice/.config` on Linux, `C:\Users\Alice\AppData\Roaming`
   on Windows, `/Users/Alice/Library/Application Support` on macOS).

## Usage ⛄️

```
# aoc help

Advent of Code command-line tool

Usage: aoc [OPTIONS] [COMMAND]

Commands:
  read                 Read puzzle statement (the default command) [aliases: r]
  download             Save puzzle description and input to files [aliases: d]
  submit               Submit puzzle answer [aliases: s]
  private-leaderboard  Show the state of a private leaderboard [aliases: p]
  help                 Print this message or the help of the given subcommand(s)

Options:
  -d, --day <DAY>            Puzzle day [default: last unlocked day (during Advent of Code month)]
  -y, --year <YEAR>          Puzzle year [default: year of current or last Advent of Code event]
  -s, --session-file <PATH>  Path to session cookie file [default: ~/.adventofcode.session]
  -w, --width <WIDTH>        Width at which to wrap output [default: terminal width]
  -o, --overwrite            Overwrite files if they already exist
  -I, --input-only           Download puzzle input only
  -P, --puzzle-only          Download puzzle description only
  -i, --input-file <PATH>    Path where to save puzzle input [default: input]
  -p, --puzzle-file <PATH>   Path where to save puzzle description [default: puzzle.md]
  -q, --quiet                Restrict log messages to errors only
      --debug                Enable debug logging
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
  -P, --puzzle-only          Download puzzle description only
  -i, --input-file <PATH>    Path where to save puzzle input [default: input]
  -p, --puzzle-file <PATH>   Path where to save puzzle description [default: puzzle.md]
  -q, --quiet                Restrict log messages to errors only
      --debug                Enable debug logging
  -h, --help                 Print help information
```

### Read puzzle description

Read today's puzzle (if today is an Advent of Code day) in plain text from the
comfort of your terminal.

```
# aoc read

[INFO  aoc] 🎄 aoc-cli - Advent of Code command-line tool

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

[INFO  aoc] 🎄 aoc-cli - Advent of Code command-line tool
[INFO  aoc::aoc] 🎅 Saved puzzle description to 'puzzle.md'
[INFO  aoc::aoc] 🎅 Saved puzzle input to 'input'
```

### Submit puzzle answers

Submit the answer to part 1 of today's puzzle (in this example, the answer is
999):

```
# aoc submit 1 999

[INFO  aoc] 🎄 aoc-cli - Advent of Code command-line tool

That's the right answer! You are one gold star closer to saving your vacation. [[Continue to Part Two]][1]

[1] /2022/day/2#part2
```

### Show private leaderboard

If you are a member of a [private leaderboard](https://adventofcode.com/leaderboard/private),
you can see how you are faring against your friends by passing the leaderboad
number:

```
# aoc private-leaderboard 1234

[INFO  aoc] 🎄 aoc-cli - Advent of Code command-line tool
Private leaderboard of me & my friends for Advent of Code 2022.

                 1111111111222222
        1234567890123456789012345
 1) 254 ★★★★★★★★★★★★...            Whitney Effertz
 2) 252 ★★★★★★★★★★.★★★.            Emery Zboncak
 3) 134 ★★★★★★★........            Ezra Parisian
 4)  72 ★★★★...........            Asha Gerlach
 5)  54 ★★★★...........            Frederik Robel
 6)  20 ★..............            Graciela Herzog
 7)   0 ...............            Thad Prohaska
```

### Command abbreviations

Any non-ambiguous prefix of a command can be used instead of the full command
name. For instance:
- Instead of `aoc read`, type `aoc r`, `aoc re` or `aoc rea`.
- Instead of `aoc download`, type `aoc d`, `aoc do`, `aoc dow`, `aoc down`, etc.
- Instead of `aoc submit`, type `aoc s`, `aoc su`, `aoc sub`, etc.
- Instead of `aoc private-leaderboard`, type `aoc p`, `aoc pr`, `aoc pri` etc.

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

[INFO  aoc] 🎄 aoc-cli - Advent of Code command-line tool
[ERROR aoc] 🔔 Puzzle 25 of 2030 is still locked
```

Submit the answer to a previous year:
```
# aoc s 1 999 -y 2015 -d 1
```

Specify path to session cookie file:
```
# aoc download --session-file /tmp/.aoc.session
```

## Contribute 🦌

Feedback and pull requests are welcome. Please see [CONTRIBUTING](CONTRIBUTING.md)
for guidelines and ideas.

## Support Advent of Code 🎁

Advent of Code is a free online Advent calendar of small programming puzzles
created by [Eric Wastl](http://was.tl/) and maintained by volunteers. Please
consider [supporting their work](https://adventofcode.com/support).
