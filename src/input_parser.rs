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
        if let Ok(dt) = dateparser::parse(input) {
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
}
