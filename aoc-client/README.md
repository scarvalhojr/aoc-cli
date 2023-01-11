# aoc-client

[![Build and test](https://github.com/scarvalhojr/aoc-cli/actions/workflows/build-and-test.yml/badge.svg)](https://github.com/scarvalhojr/aoc-cli/actions/workflows/build-and-test.yml)
[![Clippy and format](https://github.com/scarvalhojr/aoc-cli/actions/workflows/clippy-and-fmt.yml/badge.svg)](https://github.com/scarvalhojr/aoc-cli/actions/workflows/clippy-and-fmt.yml)
[![crates.io](https://img.shields.io/crates/v/aoc-client.svg)](https://crates.io/crates/aoc-client)

## Advent of Code library 🎄

`aoc-client` is a Rust library for [Advent of Code](https://adventofcode.com).
It is used to build the [aoc-cli](https://crates.io/crates/aoc-cli) command-line
tool but can also be integrated into other projects.

## Usage ⛄️

Add the following dependency to your Rust project (in `Cargo.toml`):

```toml
[dependencies]
aoc-client = "0.1"
```

Create a `AocClient` instance and call its methods:

```rust
use aoc_client::{AocClient, AocResult};

fn main() -> AocResult<()> {
    let client = AocClient::builder()
        .session_cookie_from_default_locations()?
        .year(2022)?
        .day(1)?
        .build()?;

    let _input: String = client.get_input()?;

    // Solve part 1 using your input and then submit your answer
    let answer_part1 = 1234;
    client.submit_answer(1, answer_part1)?;

    // Solve part 2 using your input and then submit your answer
    let answer_part2 = 5678;
    client.submit_answer(2, answer_part2)?;

    Ok(())
}
```

## Contribute 🦌

Feedback and pull requests are welcome. Please see [CONTRIBUTING](../CONTRIBUTING.md)
for guidelines and ideas.

## Support Advent of Code 🎁

Advent of Code is a free online Advent calendar of small programming puzzles
created by [Eric Wastl](http://was.tl/) and maintained by volunteers. Please
consider [supporting their work](https://adventofcode.com/support).
