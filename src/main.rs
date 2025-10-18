use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Parser;

/// Parse timestamps in various formats
#[derive(Parser, Debug)]
#[command(name = "when")]
#[command(about = "Parse and convert timestamps in unknown formats", long_about = None)]
struct Args {
    /// The timestamp string to parse (defaults to current date/time)
    timestamp: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let datetime = parse_input(args.timestamp)?;
    print_output(&mut std::io::stdout(), &datetime);

    Ok(())
}

fn parse_input(timestamp: Option<String>) -> Result<DateTime<Utc>> {
    if let Some(input) = timestamp {
        println!("Parsing input: '{}'", input);
        parse_timestamp(&input).context("Failed to parse timestamp")
    } else {
        println!("Using current date/time");
        Ok(Utc::now())
    }
}

fn parse_timestamp(input: &str) -> Result<DateTime<Utc>> {
    if let Ok(dt) = dateparser::parse(input) {
        return Ok(dt.with_timezone(&Utc));
    }

    anyhow::bail!("Unable to parse timestamp: '{}'", input)
}

fn print_output<W: std::io::Write>(writer: &mut W, datetime: &DateTime<Utc>) {
    writeln!(
        writer,
        "  UTC:       {}",
        datetime.with_timezone(&chrono::Utc)
    )
    .ok();
    writeln!(
        writer,
        "  Local:     {}",
        datetime.with_timezone(&chrono::Local)
    )
    .ok();
    writeln!(writer, "  Unix:      {}", datetime.timestamp()).ok();
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_parse_timestamp_unix_timestamp() {
        let input = "1705323000"; // 2024-01-15 12:30:00 UTC
        let result = parse_timestamp(input);
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.timestamp(), 1705323000);
    }

    #[test]
    fn test_parse_timestamp_human_readable() {
        let input = "January 15, 2024 12:30 PM";
        let result = parse_timestamp(input);
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
    }

    #[test]
    fn test_parse_timestamp_invalid() {
        let input = "not a valid timestamp";
        let result = parse_timestamp(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_input_none() {
        let result = parse_input(None);
        assert!(result.is_ok());
        // Should return current time, just verify it's close to now
        let dt = result.unwrap();
        let now = Utc::now();
        let diff = (now.timestamp() - dt.timestamp()).abs();
        assert!(diff < 2); // Within 2 seconds
    }

    #[test]
    fn test_parse_timestamp_rfc2822() {
        let input = "Mon, 15 Jan 2024 12:30:00 +0000";
        let result = parse_timestamp(input);
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
    }

    #[test]
    fn test_print_output() {
        // Create a known datetime: 2024-01-15 12:30:00 UTC
        use chrono::TimeZone;
        let dt = Utc.with_ymd_and_hms(2024, 1, 15, 12, 30, 0).unwrap();

        // Capture the output
        let mut output = Vec::new();
        print_output(&mut output, &dt);
        let output_str = String::from_utf8(output).unwrap();

        // Verify the output contains expected strings
        assert!(output_str.contains("UTC:       2024-01-15 12:30:00 UTC"));
        assert!(output_str.contains("Unix:      1705321800"));
        // Local time will vary by timezone, so just check it exists
        assert!(output_str.contains("Local:"));

        // Verify the output has all three lines
        let lines: Vec<&str> = output_str.lines().collect();
        assert_eq!(lines.len(), 3);
    }
}
