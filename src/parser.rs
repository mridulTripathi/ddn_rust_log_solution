#[derive(Debug, PartialEq)]

pub enum LogLevel {
    Info,
    Warn,
    Error,
}

pub enum ParseResult {
    Valid(LogLevel),
    Malformed,
}

pub fn parse_line(line: &str) -> ParseResult {
    let line = line.trim();

    if line.is_empty() {
        return ParseResult::Malformed;
    }

    let mut parts = line.splitn(4, '|');

    let _timestamp = match parts.next() {
        Some(t) => t,
        None => return ParseResult::Malformed,
    };

    let level_str = match parts.next() {
        Some(l) => l,
        None => return ParseResult::Malformed,
    };

    if parts.next().is_none() {
        return ParseResult::Malformed;
    }
    if parts.next().is_none() {
        return ParseResult::Malformed;
    }

    match level_str {
        "INFO" => ParseResult::Valid(LogLevel::Info),
        "WARN" => ParseResult::Valid(LogLevel::Warn),
        "ERROR" => ParseResult::Valid(LogLevel::Error),
        _ => ParseResult::Malformed,
    }
}
