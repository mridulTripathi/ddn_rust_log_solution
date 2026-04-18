use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::process;

mod parser;
mod stats;

use parser::{LogLevel, ParseResult, parse_line};
use stats::Stats;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        process::exit(1);
    }

    let file_path = &args[1];

    let file = File::open(file_path).unwrap_or_else(|err| {
        eprintln!("Error while opening the file '{}': {}", file_path, err);
        process::exit(1)
    });

    match analyze(file) {
        Ok(stats) => {
            stats.print_summary();
        }
        Err(err) => {
            eprintln!("Error while analyzing the file '{}': {}", file_path, err);
            process::exit(1);
        }
    }
}

fn analyze<R: io::Read>(reader: R) -> io::Result<Stats> {
    let buf_reader = BufReader::new(reader);

    let mut stats = Stats::new();

    for line_result in buf_reader.lines() {
        let line = line_result?;

        match parse_line(&line) {
            ParseResult::Valid(LogLevel::Info) => stats.info += 1,
            ParseResult::Valid(LogLevel::Warn) => stats.warn += 1,
            ParseResult::Valid(LogLevel::Error) => stats.error += 1,
            ParseResult::Malformed => stats.malformed += 1
        }
    }

    Ok(stats)
}