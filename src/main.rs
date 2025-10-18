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
    print_output(&datetime);

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

fn print_output(datetime: &DateTime<Utc>) {
    println!("  UTC:       {}", datetime.with_timezone(&chrono::Utc));
    println!("  Local:     {}", datetime.with_timezone(&chrono::Local));
    println!("  Unix:      {}", datetime.timestamp());
}
