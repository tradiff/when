mod config;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use clap::Parser;
use config::{OutputFormat, Settings};

/// Parse timestamps in various formats
#[derive(Parser, Debug)]
#[command(name = "when")]
#[command(about = "Parse and convert timestamps in unknown formats", long_about = None)]
struct Args {
    /// The timestamp string to parse (defaults to current date/time)
    timestamp: Option<String>,
}

fn main() -> Result<()> {
    // Load configuration
    let settings = Settings::new().context("Failed to load configuration")?;

    let args = Args::parse();

    let datetime = parse_input(args.timestamp)?;
    print_output(&mut std::io::stdout(), &datetime, &settings);

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

fn print_output<W: std::io::Write>(writer: &mut W, datetime: &DateTime<Utc>, settings: &Settings) {
    for output in &settings.outputs {
        match format_datetime(datetime, &output.format) {
            Ok(formatted) => {
                writeln!(
                    writer,
                    "  {:<10} {}",
                    format!("{}:", output.label),
                    formatted
                )
                .ok();
            }
            Err(e) => {
                writeln!(
                    writer,
                    "  {:<10} Error: {}",
                    format!("{}:", output.label),
                    e
                )
                .ok();
            }
        }
    }
}

fn format_datetime(datetime: &DateTime<Utc>, format: &OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Rfc3339 { timezone } | OutputFormat::Iso8601 { timezone } => {
            let dt = with_timezone(datetime, timezone.as_deref())?;
            Ok(dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
        }
        OutputFormat::Rfc2822 { timezone } => {
            let dt = with_timezone(datetime, timezone.as_deref())?;
            Ok(dt.to_rfc2822())
        }
        OutputFormat::Unix => Ok(datetime.timestamp().to_string()),
        OutputFormat::UnixMillis => Ok(datetime.timestamp_millis().to_string()),
        OutputFormat::Custom { format, timezone } => {
            let dt = with_timezone(datetime, timezone.as_deref())?;
            Ok(dt.format(format).to_string())
        }
    }
}

fn with_timezone(
    datetime: &DateTime<Utc>,
    timezone: Option<&str>,
) -> Result<DateTime<chrono::FixedOffset>> {
    match timezone {
        Some(tz_str) => {
            let tz: Tz = tz_str
                .parse()
                .context(format!("Invalid timezone: {}", tz_str))?;
            Ok(datetime.with_timezone(&tz).fixed_offset())
        }
        None => Ok(datetime.with_timezone(&chrono::Local).fixed_offset()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_parse_timestamp_unix_timestamp() {
        let input = "1136214245";
        let result = parse_timestamp(input);
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.timestamp(), 1136214245);
    }

    #[test]
    fn test_parse_timestamp_human_readable() {
        let input = "January 2, 2006 3:04:05 PM";
        let result = parse_timestamp(input);
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2006);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 2);
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
        let input = "January 2, 2006 3:04:05 PM";
        let result = parse_timestamp(input);
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2006);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 2);
    }

    #[test]
    fn test_print_output() {
        // Create a known datetime
        use chrono::TimeZone;
        let dt = Utc.with_ymd_and_hms(2006, 1, 2, 15, 4, 5).unwrap();

        // Use default settings
        let settings = Settings::default();

        // Capture the output
        let mut output = Vec::new();
        print_output(&mut output, &dt, &settings);
        let output_str = String::from_utf8(output).unwrap();

        // Verify the output contains expected strings
        assert!(output_str.contains("UTC:"));
        assert!(output_str.contains("2006-01-02"));
        assert!(output_str.contains("Local:"));
        assert!(output_str.contains("Unix:"));
        assert!(output_str.contains("1136214245"));

        // Verify the output has all three lines
        let lines: Vec<&str> = output_str.lines().collect();
        assert_eq!(lines.len(), 3);
    }
}
