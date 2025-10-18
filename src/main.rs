use clap::Parser;

/// Parse timestamps in various formats
#[derive(Parser, Debug)]
#[command(name = "when")]
#[command(about = "Parse and convert timestamps in unknown formats", long_about = None)]
struct Args {
    /// The timestamp string to parse
    timestamp: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("Parsing input: '{}'", args.timestamp);
    Ok(())
}
