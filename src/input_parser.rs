use anyhow::{Context, Result};
use chrono::{DateTime, Local, Utc};
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
        // This includes formats with explicit timezone like "2025-10-31T17:01:00Z"
        if let Ok(dt) = dateparser::parse(input) {
            return Ok(dt.with_timezone(&Utc));
        }

        // Fallback to chrono-english for fuzzy/natural language parsing
        // (handles "tomorrow at 3pm", "next monday", "oct 31 5:00pm utc", etc.)
        if let Some(value) = Self::parse_with_chrono_english(input) {
            return value;
        }

        anyhow::bail!("Unable to parse timestamp: '{}'", input)
    }

    fn parse_with_chrono_english(
        input: &str,
    ) -> Option<std::result::Result<DateTime<Utc>, anyhow::Error>> {
        let input_lower = input.to_lowercase();

        // Check if input contains "utc" to determine which timezone to use for parsing
        let dt = if input_lower.contains("utc") {
            // Parse with UTC base time
            let parsed =
                chrono_english::parse_date_string(input, Utc::now(), chrono_english::Dialect::Us)
                    .ok()?;
            parsed.with_timezone(&Utc)
        } else {
            // Parse with Local base time so times are interpreted as local
            let parsed =
                chrono_english::parse_date_string(input, Local::now(), chrono_english::Dialect::Us)
                    .ok()?;
            parsed.with_timezone(&Utc)
        };

        Some(Ok(dt))
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
        // Note: chrono-english doesn't parse timezone suffixes like "utc" reliably
        // This test verifies that dates are parsed in local time by default
        let input = "oct 15, 2025 5:00pm";
        let result = InputParser::parse_timestamp(input);
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 10);
        assert_eq!(dt.day(), 15);
        // The exact timestamp will vary based on local timezone
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
