use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use colored::Colorize;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};

use crate::config::{OutputFormat, Settings};

const MARGIN: &str = "  ";

pub struct OutputPrinter;

impl OutputPrinter {
    pub fn print<W: std::io::Write>(writer: &mut W, datetime: &DateTime<Utc>, settings: &Settings) {
        writeln!(writer).ok();
        writeln!(
            writer,
            "{}{}",
            MARGIN,
            "Timestamp Conversions".bold().bright_cyan()
        )
        .ok();

        let mut table = Table::new();
        table
            .load_preset(UTF8_BORDERS_ONLY)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic);

        for output in &settings.outputs {
            match Self::format_datetime(datetime, &output.format) {
                Ok(formatted) => {
                    table.add_row(vec![
                        Cell::new(&output.label).fg(Color::Yellow),
                        Cell::new(formatted).fg(Color::White),
                    ]);
                }
                Err(e) => {
                    table.add_row(vec![
                        Cell::new(&output.label)
                            .fg(Color::Yellow)
                            .add_attribute(Attribute::Bold),
                        Cell::new(format!("Error: {}", e)).fg(Color::Red),
                    ]);
                }
            }
        }

        let table_string = table.to_string();
        for line in table_string.lines() {
            let line = Self::colorize_borders(line);
            let line = MARGIN.to_string() + &line;
            writeln!(writer, "{}", line).ok();
        }
        writeln!(writer).ok();
    }

    /// comfy-table doesn't support styling borders directly. This function is a hack to apply styling to the UTF-8 border characters after the table is rendered to a string.
    fn colorize_borders(line: &str) -> String {
        line.chars()
            .map(|c| match c {
                '╭' | '╮' | '╰' | '╯' | '─' | '│' | '┆' | '╞' | '╡' | '═' => {
                    format!("{}", c.to_string().dimmed())
                }
                _ => c.to_string(),
            })
            .collect()
    }

    fn format_datetime(datetime: &DateTime<Utc>, format: &OutputFormat) -> Result<String> {
        match format {
            OutputFormat::Rfc3339 { timezone } | OutputFormat::Iso8601 { timezone } => {
                let dt = Self::with_timezone(datetime, timezone.as_deref())?;
                Ok(dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
            }
            OutputFormat::Rfc2822 { timezone } => {
                let dt = Self::with_timezone(datetime, timezone.as_deref())?;
                Ok(dt.to_rfc2822())
            }
            OutputFormat::Unix => Ok(datetime.timestamp().to_string()),
            OutputFormat::UnixMillis => Ok(datetime.timestamp_millis().to_string()),
            OutputFormat::Custom { format, timezone } => {
                let dt = Self::with_timezone(datetime, timezone.as_deref())?;
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
}
