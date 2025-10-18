mod config;
mod input_parser;
mod output_printer;

use anyhow::{Context, Result};
use clap::Parser;
use config::Settings;
use input_parser::InputParser;
use output_printer::OutputPrinter;

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

    let datetime = InputParser::parse(args.timestamp)?;
    OutputPrinter::print(&mut std::io::stdout(), &datetime, &settings);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_print_output() {
        // Create a known datetime
        let dt = Utc.with_ymd_and_hms(2006, 1, 2, 15, 4, 5).unwrap();

        // Use default settings
        let settings = Settings::default();

        // Capture the output
        let mut output = Vec::new();
        OutputPrinter::print(&mut output, &dt, &settings);
        let output_str = String::from_utf8(output).unwrap();

        // Verify the output contains expected strings (without color codes for testing)
        assert!(output_str.contains("2006-01-02"));
        assert!(output_str.contains("Local"));
        assert!(output_str.contains("UTC"));
        assert!(output_str.contains("Unix"));
        assert!(output_str.contains("1136214245"));
    }
}
