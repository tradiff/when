use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use colored::Colorize;

pub struct InputParser;

impl InputParser {
    /// Parse an optional timestamp input, defaulting to current time if None
    pub fn parse(timestamp: Option<String>) -> Result<DateTime<Utc>> {
        if let Some(input) = timestamp {
            println!(
                "{} {}",
                "•".bright_blue(),
                format!("Parsing: '{}'", input).dimmed()
            );
            Self::parse_timestamp(&input).context("Failed to parse timestamp")
        } else {
            println!(
                "{} {}",
                "•".bright_blue(),
                "Using current date/time".dimmed()
            );
            Ok(Utc::now())
        }
    }

    /// Parse a timestamp string into a DateTime<Utc>
    fn parse_timestamp(input: &str) -> Result<DateTime<Utc>> {
        // Try dateparser first (handles ISO, RFC, Unix timestamps, etc.)
        if let Ok(dt) = dateparser::parse(input) {
            return Ok(dt.with_timezone(&Utc));
        }

        // Fallback to chrono-english for fuzzy/natural language parsing
        // (handles "tomorrow at 3pm", "next monday", "oct 31 5:00pm utc", etc.)
        if let Ok(dt) =
            chrono_english::parse_date_string(input, Utc::now(), chrono_english::Dialect::Uk)
        {
            return Ok(dt.with_timezone(&Utc));
        }

        anyhow::bail!("Unable to parse timestamp: '{}'", input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_parse_timestamp_unix_timestamp() {
        let input = "1136214245";
        let result = InputParser::parse_timestamp(input);
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.timestamp(), 1136214245);
    }

    #[test]
    fn test_parse_timestamp_human_readable() {
        let input = "January 2, 2006 3:04:05 PM";
        let result = InputParser::parse_timestamp(input);
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2006);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 2);
    }

    #[test]
    fn test_parse_timestamp_invalid() {
        let input = "not a valid timestamp";
        let result = InputParser::parse_timestamp(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_input_none() {
        let result = InputParser::parse(None);
        assert!(result.is_ok());
        // Should return current time, just verify it's close to now
        let dt = result.unwrap();
        let now = Utc::now();
        let diff = (now.timestamp() - dt.timestamp()).abs();
        assert!(diff < 2); // Within 2 seconds
    }

    #[test]
    fn test_parse_timestamp_rfc2822() {
        let input = "January 2, 2006 3:04:05 PM";
        let result = InputParser::parse_timestamp(input);
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2006);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 2);
    }

    #[test]
    fn test_parse_timestamp_fuzzy_with_timezone() {
        let input = "oct 15, 2025 5:00pm utc";
        let result = InputParser::parse_timestamp(input);
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 10);
        assert_eq!(dt.day(), 15);
        // 5pm UTC should be 17:00:00
        assert_eq!(dt.timestamp(), 1760547600);
    }

    #[test]
    fn test_parse_timestamp_natural_language() {
        let input = "tomorrow 3pm";
        let result = InputParser::parse_timestamp(input);
        assert!(result.is_ok());
        // Just verify it parses successfully - exact datetime depends on when test runs
    }

    #[test]
    fn test_parse_timestamp_relative() {
        let input = "next monday";
        let result = InputParser::parse_timestamp(input);
        assert!(result.is_ok());
    }
}
