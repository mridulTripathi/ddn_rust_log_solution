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

#[cfg(test)]
mod tests {
    use super::{LogFormat, LogLevel, ParseResult, parse_line};

    #[test]
    fn parses_pipe_info_line() {
        let line = "2025-01-01T00:00:00Z|INFO|auth|login ok";
        let parsed = parse_line(line, LogFormat::PIPE);
        assert!(matches!(parsed, ParseResult::Valid(LogLevel::Info)));
    }

    #[test]
    fn parses_csv_warn_line() {
        let line = "2025-01-01T00:00:00Z,WARN,api,slow request";
        let parsed = parse_line(line, LogFormat::CSV);
        assert!(matches!(parsed, ParseResult::Valid(LogLevel::Warn)));
    }

    #[test]
    fn parses_tsv_error_line_with_spaces() {
        let line = "2025-01-01T00:00:00Z\t ERROR \tworker\ttask failed";
        let parsed = parse_line(line, LogFormat::TSV);
        assert!(matches!(parsed, ParseResult::Valid(LogLevel::Error)));
    }

    #[test]
    fn returns_empty_line_reason() {
        let parsed = parse_line("   ", LogFormat::PIPE);
        assert!(matches!(parsed, ParseResult::Malformed("empty line")));
    }

    #[test]
    fn returns_too_few_fields_reason() {
        let parsed = parse_line("2025-01-01|INFO|auth", LogFormat::PIPE);
        assert!(matches!(parsed, ParseResult::Malformed("too few fields")));
    }

    #[test]
    fn returns_unknown_level_reason() {
        let parsed = parse_line("2025-01-01|DEBUG|auth|message", LogFormat::PIPE);
        assert!(matches!(parsed, ParseResult::Malformed("unknown level")));
    }

    #[test]
    fn format_lookup_is_case_insensitive() {
        assert!(matches!(LogFormat::from_name("PiPe"), Some(_)));
        assert!(matches!(LogFormat::from_name("CSV"), Some(_)));
        assert!(matches!(LogFormat::from_name("tSv"), Some(_)));
        assert!(LogFormat::from_name("json").is_none());
    }
}
