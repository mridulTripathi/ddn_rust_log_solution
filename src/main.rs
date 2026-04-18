use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::process;

use rayon::prelude::*;

mod parser;
mod stats;

use parser::{LogFormat, LogLevel, ParseResult, parse_line};
use stats::Stats;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <log_file> [--format pipe|csv|tsv]", args[0]);
        process::exit(1);
    }

    let file_path = &args[1];
    let format = parse_format_arg(&args[2..]).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });

    let file = File::open(file_path).unwrap_or_else(|err| {
        eprintln!("Error while opening the file '{}': {}", file_path, err);
        process::exit(1)
    });

    match analyze(file, format) {
        Ok(stats) => {
            stats.print_summary();
        }
        Err(err) => {
            eprintln!("Error while analyzing the file '{}': {}", file_path, err);
            process::exit(1);
        }
    }
}

fn parse_format_arg(args: &[String]) -> Result<LogFormat, String> {
    let mut idx = 0;
    let mut selected = LogFormat::PIPE;
    while idx < args.len() {
        match args[idx].as_str() {
            "--format" => {
                let Some(value) = args.get(idx + 1) else {
                    return Err("--format requires one value: pipe|csv|tsv".to_string());
                };
                selected = LogFormat::from_name(value)
                    .ok_or_else(|| format!("Unsupported format '{}'. Use pipe|csv|tsv", value))?;
                idx += 2;
            }
            unknown => {
                return Err(format!(
                    "Unknown argument '{}'. Usage: <log_file> [--format pipe|csv|tsv]",
                    unknown
                ));
            }
        }
    }

    Ok(selected)
}

fn analyze<R: io::Read>(reader: R, format: LogFormat) -> io::Result<Stats> {
    let buf_reader = BufReader::new(reader);

    let mut stats = Stats::new();
    let mut chunk: Vec<String> = Vec::with_capacity(2048);
    const CHUNK_SIZE: usize = 2048;

    for line_result in buf_reader.lines() {
        let line = line_result?;
        chunk.push(line);

        if chunk.len() >= CHUNK_SIZE {
            stats.merge(analyze_chunk(&chunk, format));
            chunk.clear();
        }
    }

    if !chunk.is_empty() {
        stats.merge(analyze_chunk(&chunk, format));
    }

    Ok(stats)
}

fn analyze_chunk(lines: &[String], format: LogFormat) -> Stats {
    lines
        .par_iter()
        .map(|line| {
            let mut stats = Stats::new();
            match parse_line(line, format) {
                ParseResult::Valid(LogLevel::Info) => stats.info += 1,
                ParseResult::Valid(LogLevel::Warn) => stats.warn += 1,
                ParseResult::Valid(LogLevel::Error) => stats.error += 1,
                ParseResult::Malformed(reason) => stats.record_malformed(reason),
            }
            stats
        })
        .reduce(Stats::new, |mut a, b| {
            a.merge(b);
            a
        })
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::{analyze, analyze_chunk, parse_format_arg};
    use crate::parser::LogFormat;

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| (*part).to_string()).collect()
    }

    #[test]
    fn parse_format_arg_defaults_to_pipe() {
        let parsed = parse_format_arg(&[]).expect("default format should parse");
        let result = analyze_chunk(&["a|INFO|b|c".to_string()], parsed);
        assert_eq!(result.info, 1);
    }

    #[test]
    fn parse_format_arg_accepts_known_values() {
        let parsed = parse_format_arg(&args(&["--format", "csv"])).expect("csv should parse");
        let stats = analyze_chunk(&["a,WARN,b,c".to_string()], parsed);
        assert_eq!(stats.warn, 1);
    }

    #[test]
    fn parse_format_arg_rejects_missing_format_value() {
        let err =
            parse_format_arg(&args(&["--format"])).expect_err("should fail for missing value");
        assert!(err.contains("--format requires one value"));
    }

    #[test]
    fn parse_format_arg_rejects_unknown_value() {
        let err = parse_format_arg(&args(&["--format", "json"]))
            .expect_err("should fail for unknown format");
        assert!(err.contains("Unsupported format"));
    }

    #[test]
    fn parse_format_arg_rejects_unknown_argument() {
        let err = parse_format_arg(&args(&["--unknown", "value"]))
            .expect_err("should fail for unknown arg");
        assert!(err.contains("Unknown argument"));
    }

    #[test]
    fn analyze_counts_valid_and_malformed_lines() {
        let input = "\
2025-01-01T00:00:00Z|INFO|auth|ok
2025-01-01T00:00:01Z|WARN|auth|slow
2025-01-01T00:00:02Z|ERROR|auth|failed
2025-01-01T00:00:03Z|DEBUG|auth|unknown
bad line

";
        let stats = analyze(Cursor::new(input.as_bytes()), LogFormat::PIPE)
            .expect("analysis should succeed");

        assert_eq!(stats.info, 1);
        assert_eq!(stats.warn, 1);
        assert_eq!(stats.error, 1);
        assert_eq!(stats.malformed, 3);
        assert_eq!(stats.malformed_by_reason.get("unknown level"), Some(&1));
        assert_eq!(stats.malformed_by_reason.get("too few fields"), Some(&1));
        assert_eq!(stats.malformed_by_reason.get("empty line"), Some(&1));
    }

    #[test]
    fn analyze_supports_csv_input() {
        let input = "\
2025-01-01T00:00:00Z,INFO,auth,ok
2025-01-01T00:00:01Z,WARN,auth,slow
";
        let stats = analyze(Cursor::new(input.as_bytes()), LogFormat::CSV)
            .expect("csv analysis should succeed");

        assert_eq!(stats.info, 1);
        assert_eq!(stats.warn, 1);
        assert_eq!(stats.error, 0);
        assert_eq!(stats.malformed, 0);
    }

    #[test]
    fn analyze_chunk_handles_empty_input() {
        let lines: Vec<String> = Vec::new();
        let stats = analyze_chunk(&lines, LogFormat::PIPE);
        assert_eq!(stats.total(), 0);
    }
}
