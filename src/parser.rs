#[derive(Debug, PartialEq)]

pub enum LogLevel {
    Info,
    Warn,
    Error,
}

pub enum ParseResult {
    Valid(LogLevel),
    Malformed(&'static str),
}

#[derive(Debug, Clone, Copy)]
pub struct LogFormat {
    delimiter: char,
    level_idx: usize,
    min_fields: usize,
}

impl LogFormat {
    pub const PIPE: LogFormat = LogFormat {
        delimiter: '|',
        level_idx: 1,
        min_fields: 4,
    };

    pub const CSV: LogFormat = LogFormat {
        delimiter: ',',
        level_idx: 1,
        min_fields: 4,
    };

    pub const TSV: LogFormat = LogFormat {
        delimiter: '\t',
        level_idx: 1,
        min_fields: 4,
    };

    pub fn from_name(name: &str) -> Option<LogFormat> {
        match name.to_ascii_lowercase().as_str() {
            "pipe" => Some(LogFormat::PIPE),
            "csv" => Some(LogFormat::CSV),
            "tsv" => Some(LogFormat::TSV),
            _ => None,
        }
    }
}

pub fn parse_line(line: &str, format: LogFormat) -> ParseResult {
    let line = line.trim();

    if line.is_empty() {
        return ParseResult::Malformed("empty line");
    }

    let parts: Vec<&str> = line.split(format.delimiter).collect();

    if parts.len() < format.min_fields {
        return ParseResult::Malformed("too few fields");
    }

    let level_str = parts[format.level_idx].trim();

    match level_str {
        "INFO" => ParseResult::Valid(LogLevel::Info),
        "WARN" => ParseResult::Valid(LogLevel::Warn),
        "ERROR" => ParseResult::Valid(LogLevel::Error),
        _ => ParseResult::Malformed("unknown level"),
    }
}
