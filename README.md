# aoc-cli

[Advent of Code](https://adventofcode.com) command-line helper tool.

## Features

- Download puzzle input and save it to a file.
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
very long hex string) in a file called `.adventofcode.session` in your home
directory. This file should only contain your session number, in a single line.

## Usage

```
# aoc --help

aoc 0.1.0
Advent of Code command-line helper tool

USAGE:
    aoc [OPTIONS] <command> [ARGS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --day <day>      Puzzle day [default: last unlocked day (during Advent of Code)]
    -f, --file <file>    Save puzzle input to file [default: input]
    -y, --year <year>    Puzzle year [default: year of current or last Advent of Code]

ARGS:
    <command>    Command to execute [possible values: download, d, submit, s]
    <part>       Puzzle part (required for submit) [possible values: 1, 2]
    <answer>     Puzzle answer (required for submit)
```

- Download input for today's puzzle (if today is an Advent of Code day) and save
  it to a file named "input" on the current directory:
```
# aoc download

Loaded session cookie from "/home/user/.adventofcode.session".
Downloading input for day 5, 2020...
Saving puzzle input to "input"...
Done!
```

Or simply:
```
# aoc d
```

- Submit the answer to part 1 of today's puzzle (in this example, the answer is
  999):
```
# aoc submit 1 999

Loaded session cookie from "/home/user/.adventofcode.session".
Submitting answer for part 1, day 5, 2020...
Done!
```

- Download the input from a previous day (assumes the current year):
```
# aoc download --day 1

Loaded session cookie from "/home/user/.adventofcode.session".
Downloading input for day 1, 2020...
Saving puzzle input to "input"...
Done!
```

- Download the input from a previous year and save it to a file with a given
  name:
```
# aoc download --year 2015 --day 1 --file /home/user/aoc/2015/1/input
```

- An attempt to download the input of a puzzle that is still locked fails
  (puzzles unlock every day between 1st and 25th of December at midnight
  EST/UTC-5):
```
# aoc download --year 2030 --day 25

Loaded session cookie from "/home/user/.adventofcode.session".
Error: Puzzle 25 of 2030 is still locked.
```

- Submit the answer to a previous year:
```
# aoc s 1 999 -y 2015 -d 1
```

## Contribute

Feedback and pull requests are welcome.

## Support Advent of Code

Advent of Code is a free online Advent calendar of small programming puzzles
created by [Eric Wastl](http://was.tl/) and maintained by volunteers. Please
consider [supporting their work](https://adventofcode.com/support).
