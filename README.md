# aoc-cli

[![Build and test](https://github.com/scarvalhojr/aoc-cli/actions/workflows/build-and-test.yml/badge.svg)](https://github.com/scarvalhojr/aoc-cli/actions/workflows/build-and-test.yml)
[![Clippy](https://github.com/scarvalhojr/aoc-cli/actions/workflows/rust-clippy.yml/badge.svg)](https://github.com/scarvalhojr/aoc-cli/actions/workflows/rust-clippy.yml)

[Advent of Code](https://adventofcode.com) command-line helper tool.

## Features

- Read puzzle description, download input and save it to a file.
- Submit answers and check if they are correct.
- Load Advent of Code session cookie from a file.
- Validate arguments (year, day, part) and check if puzzle is unlocked.
- If year is not provided, it is assumed to be the current or the last Advent of
  Code event.
- Infer puzzle day when possible (last unlocked puzzle during Advent of Code).

## Install

- Install a recent version of Rust using [rustup](https://rustup.rs/) or update
it with:
```
# rustup update
```

- Install `aoc-cli` with cargo:
```
# cargo install aoc-cli
```

## Session cookie

Different Advent of Code users get different puzzle input. To download your
input and submit your answer, you need an adventofcode.com session cookie. To
obtain your session cookie, login to the
[Advent of Code](https://adventofcode.com) website and inspect the `session`
value of the cookie that gets stored in your browser. Put the session number (a
long hex string) in a file called `.adventofcode.session` in your home
directory. This file should only contain your session number, in a single line.

## Usage

```
# aoc help

Advent of Code command-line helper tool

Usage: aoc [OPTIONS] [COMMAND]

Commands:
  read      Read puzzle statement (the default command) [aliases: r]
  download  Download puzzle input [aliases: d]
  submit    Submit puzzle answer [aliases: s]
  help      Print this message or the help of the given subcommand(s)

Options:
  -d, --day <DAY>       Puzzle day [default: last unlocked day (during Advent of Code month)]
  -y, --year <YEAR>     Puzzle year [default: year of current or last Advent of Code event]
  -s, --session <PATH>  Path to session cookie file [default: ~/.adventofcode.session]
  -f, --file <PATH>     Path where to save puzzle input [default: input]
  -w, --width <WIDTH>   Width at which to wrap output [default: terminal width]
  -h, --help            Print help information
  -V, --version         Print version information
```

```
# aoc help submit

Submit puzzle answer

Usage: aoc submit <PART> <ANSWER>

Arguments:
  <PART>    Puzzle part [possible values: 1, 2]
  <ANSWER>  Puzzle answer

Options:
  -h, --help  Print help information
```

### Read puzzle description

Read today's puzzle (if today is an Advent of Code day) in plain text from the
comfort of your terminal:

```
# aoc read

## --- Day 5: Binary Boarding ---

You board your plane only to discover a new problem: you dropped your boarding
pass! You aren't sure which seat is yours, and all of the flight attendants are
busy with the flood of people that suddenly made it through passport control.
...
```

### Download puzzle input

Download input for today's puzzle and save it to a file named "input" on the
current directory:

```
# aoc download

Loaded session cookie from "/home/user/.adventofcode.session".
Downloading input for day 5, 2020...
Saving puzzle input to "input"...
Done!
```

### Submit puzzle answers

Submit the answer to part 1 of today's puzzle (in this example, the answer is
999):

```
# aoc submit 1 999

Loaded session cookie from "/home/user/.adventofcode.session".
Submitting answer for part 1, day 5, 2020...

That's the right answer! You are one gold star closer to saving your vacation. [[Continue to Part Two]][1]

[1] /2020/day/5#part2
```

### Command abbreviations

- Type `aoc r` instead of `aoc read`
- Type `aoc d` instead of `aoc download`
- Type `aoc s` instead of `aoc submit`

### More examples

Download the input from a previous day (assumes the current year):

```
# aoc download --day 1

Loaded session cookie from "/home/user/.adventofcode.session".
Downloading input for day 1, 2020...
Saving puzzle input to "input"...
Done!
```

Download the input from a previous year and save it to a file with a given
name:

```
# aoc download --year 2015 --day 1 --file /home/user/aoc/2015/1/input
```

An attempt to download the input of a puzzle that is still locked fails
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
# aoc download --session /tmp/.aoc.session
```

## Contribute

Feedback and pull requests are welcome.

## Support Advent of Code

Advent of Code is a free online Advent calendar of small programming puzzles
created by [Eric Wastl](http://was.tl/) and maintained by volunteers. Please
consider [supporting their work](https://adventofcode.com/support).
