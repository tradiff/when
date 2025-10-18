use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use comfy_table::{Cell, Color, ContentArrangement, Table, presets};

use crate::config::{OutputFormat, Settings};

const RESET_STYLE: &str = "\x1b[0m";
const LABEL_STYLE: &str = "\x1b[0m\x1b[38;5;178m"; // bold amber
const DATE_STYLE: &str = "\x1b[0m\x1b[38;5;37m"; // soft teal
const TIME_STYLE: &str = "\x1b[0m\x1b[1m\x1b[38;5;75m"; // sky blue
const PUNCTUATION_STYLE: &str = "\x1b[0m\x1b[2m"; // dimmed
const ZONE_STYLE: &str = "\x1b[0m\x1b[2m"; // dimmed

pub struct OutputPrinter;

impl OutputPrinter {
    pub fn print<W: std::io::Write>(writer: &mut W, datetime: &DateTime<Utc>, settings: &Settings) {
        writeln!(writer).ok();

        let mut table = Table::new();
        table
            .load_preset(presets::NOTHING)
            .set_content_arrangement(ContentArrangement::Dynamic);

        for output in &settings.outputs {
            match Self::format_datetime(datetime, &output.format) {
                Ok(formatted) => {
                    let styled_label = format!("{}{}{}", LABEL_STYLE, output.label, RESET_STYLE);
                    table.add_row(vec![
                        Cell::new(styled_label),
                        Cell::new(formatted).fg(Color::White),
                    ]);
                }
                Err(e) => {
                    let styled_label = format!("{}{}{}", LABEL_STYLE, output.label, RESET_STYLE);
                    table.add_row(vec![
                        Cell::new(styled_label),
                        Cell::new(format!("Error: {}", e)).fg(Color::Red),
                    ]);
                }
            }
        }

        let table_string = table.to_string();
        for line in table_string.lines() {
            writeln!(writer, "{}", line).ok();
        }
        writeln!(writer).ok();
    }

    fn format_datetime(datetime: &DateTime<Utc>, format: &OutputFormat) -> Result<String> {
        match format {
            OutputFormat::Rfc3339 { timezone } | OutputFormat::Iso8601 { timezone } => {
                let dt = Self::with_timezone(datetime, timezone.as_deref())?;
                let result = dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
                let result = Self::colorize_iso8601(&result);
                Ok(result)
            }
            OutputFormat::Rfc2822 { timezone } => {
                let dt = Self::with_timezone(datetime, timezone.as_deref())?;
                let result = dt.to_rfc2822();
                let result = Self::colorize_rfc2822(&result);
                Ok(result)
            }
            OutputFormat::Unix => {
                let timestamp = datetime.timestamp().to_string();
                let result = Self::colorize_unix_timestamp(&timestamp);
                Ok(result)
            }
            OutputFormat::UnixMillis => {
                let timestamp = datetime.timestamp_millis().to_string();
                let result = Self::colorize_unix_timestamp(&timestamp);
                Ok(result)
            }
            OutputFormat::Custom { format, timezone } => {
                let dt = Self::with_timezone(datetime, timezone.as_deref())?;
                let styled_format = Self::colorize_custom_format(format);
                let result = dt.format(&styled_format).to_string();
                Ok(result)
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

    fn colorize_iso8601(s: &str) -> String {
        // ISO 8601 format is: YYYY-MM-DDTHH:MM:SS+TZ or YYYY-MM-DDTHH:MM:SSZ
        // Date: chars 0-9, Time: chars 11-18, Timezone: chars 19+
        if s.len() >= 20 && s.chars().nth(10) == Some('T') {
            let mut result = s.to_string();
            // Insert in reverse order to keep positions valid
            result.insert_str(19, PUNCTUATION_STYLE); // After time
            result.insert_str(11, TIME_STYLE); // Before time
            result.insert_str(10, PUNCTUATION_STYLE); // After date
            result.insert_str(0, DATE_STYLE); // Before date
            result
        } else {
            s.to_string()
        }
    }

    /// Colorizes the time portion of an RFC2822 formatted string.
    /// Format: Mon, 02 Jan 2006 15:04:05 +0000
    fn colorize_rfc2822(s: &str) -> String {
        let parts: Vec<&str> = s.split_whitespace().collect();

        if parts.len() >= 6 && parts[4].contains(':') {
            // Find positions of time part (parts[4]) in original string
            // We need to find where it starts and ends
            let date_prefix = format!("{} {} {} {}", parts[0], parts[1], parts[2], parts[3]);
            let time_start = date_prefix.len() + 1; // +1 for space before time
            let time_end = time_start + parts[4].len();

            let mut result = s.to_string();
            // Insert in reverse order to keep positions valid
            result.insert_str(time_end, PUNCTUATION_STYLE); // After time
            result.insert_str(time_start, TIME_STYLE); // Before time
            result.insert_str(date_prefix.len(), RESET_STYLE); // After date
            result.insert_str(0, DATE_STYLE); // Before date
            result
        } else {
            s.to_string()
        }
    }

    fn colorize_custom_format(format: &str) -> String {
        // Map each style to its associated format specifiers
        // https://docs.rs/chrono/latest/chrono/format/strftime/index.html
        let style_specifiers = [
            (
                DATE_STYLE,
                vec![
                    "%Y", "%C", "%y", "%q", "%m", "%b", "%B", "%h", "%d", "%e", "%a,", "%a", "%A,",
                    "%A", "%w", "%u", "%U", "%W", "%G", "%g", "%V", "%j", "%D", "%x", "%F", "%v",
                ],
            ),
            (
                TIME_STYLE,
                vec![
                    "%H:", "%H", "%k:", "%k", "%-I:", "%_I:", "%0I:", "%I:", "%-I", "%_I", "%0I",
                    "%I", "%l:", "%l", "%P", "%p", "%M:", "%M", "%S:", "%S", "%f", "%.f", "%.3f",
                    "%.6f", "%9f", "%3f", "%6f", "%9f", "%R", "%T", "%X", "%r",
                ],
            ),
            (ZONE_STYLE, vec!["%Z", "%z", "%:z", "%::z", "%:::z"]),
        ];

        let mut result = format.to_string();

        for (style, specifiers) in style_specifiers {
            for timestamp_specifier in specifiers {
                // Only colorize if the specifier is not already wrapped in color codes
                if result.contains(timestamp_specifier)
                    && !result.contains(&format!("{}{}", DATE_STYLE, timestamp_specifier))
                    && !result.contains(&format!("{}{}", TIME_STYLE, timestamp_specifier))
                    && !result.contains(&format!("{}{}", ZONE_STYLE, timestamp_specifier))
                {
                    result = result.replace(
                        timestamp_specifier,
                        &format!("{}{}{}", style, timestamp_specifier, RESET_STYLE),
                    );
                }
            }
        }

        result
    }

    fn colorize_unix_timestamp(timestamp: &str) -> String {
        // For unix timestamps, just use the time styling
        let mut result = timestamp.to_string();
        result.insert_str(0, TIME_STYLE);
        result.insert_str(result.len(), RESET_STYLE);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colorize_iso8601() {
        let input = "2006-01-02T15:04:05+00:00";
        let result = OutputPrinter::colorize_iso8601(input);

        // Should have styled date and time
        assert!(result.contains(DATE_STYLE));
        assert!(result.contains(TIME_STYLE));
        assert!(result.contains("2006-01-02"));
        assert!(result.contains("15:04:05"));
    }

    #[test]
    fn test_colorize_rfc2822() {
        let input = "Mon, 02 Jan 2006 15:04:05 +0000";
        let result = OutputPrinter::colorize_rfc2822(input);

        // Should have styled date parts and time parts
        assert!(result.contains(DATE_STYLE));
        assert!(result.contains(TIME_STYLE));
        assert!(result.contains("15:04:05"));
    }

    #[test]
    fn test_colorize_custom_format() {
        let input = "%Y-%m-%d %H:%M:%S";
        let result = OutputPrinter::colorize_custom_format(input);

        assert!(result.contains(DATE_STYLE));
        assert!(result.contains(TIME_STYLE));

        assert!(result.contains("%Y"));
        assert!(result.contains("%m"));
        assert!(result.contains("%d"));
        assert!(result.contains("%H"));
        assert!(result.contains("%M"));
        assert!(result.contains("%S"));
    }
}
