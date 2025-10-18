use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    /// List of output formats to display
    #[serde(default = "default_outputs")]
    pub outputs: Vec<Output>,
}

/// Output configuration
#[derive(Debug, Deserialize, Clone)]
pub struct Output {
    /// Label to display before the output
    pub label: String,
    /// Format specification
    #[serde(flatten)]
    pub format: OutputFormat,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum OutputFormat {
    Rfc3339 {
        #[serde(default)]
        timezone: Option<String>,
    },
    Iso8601 {
        #[serde(default)]
        timezone: Option<String>,
    },
    Rfc2822 {
        #[serde(default)]
        timezone: Option<String>,
    },
    Unix,
    UnixMillis,
    /// Custom format string (using chrono format specifiers)
    Custom {
        format: String,
        #[serde(default)]
        timezone: Option<String>,
    },
}

fn default_outputs() -> Vec<Output> {
    vec![
        Output {
            label: "Local".to_string(),
            format: OutputFormat::Rfc3339 { timezone: None },
        },
        Output {
            label: "UTC".to_string(),
            format: OutputFormat::Rfc3339 {
                timezone: Some("UTC".to_string()),
            },
        },
        Output {
            label: "Unix".to_string(),
            format: OutputFormat::Unix,
        },
    ]
}

impl Settings {
    pub fn new() -> Result<Self> {
        let mut builder = config::Config::builder();

        if let Some(config_dir) = dirs::config_dir() {
            builder =
                builder.add_source(config::File::from(config_dir.join("when")).required(false));
        }

        let config = builder.build().context("Failed to build configuration")?;

        // If no config file exists, use defaults
        match config.try_deserialize() {
            Ok(settings) => Ok(settings),
            Err(_) => Ok(Self::default()),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            outputs: default_outputs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default() {
        let settings = Settings::default();
        assert_eq!(settings.outputs.len(), 3);
        assert_eq!(settings.outputs[0].label, "Local");
        assert_eq!(settings.outputs[1].label, "UTC");
        assert_eq!(settings.outputs[2].label, "Unix");
    }

    #[test]
    fn test_settings_new_uses_defaults() {
        // This should work even without a config file
        let result = Settings::new();
        assert!(result.is_ok());
        let settings = result.unwrap();
        // Should have default values when no config file exists
        assert!(!settings.outputs.is_empty());
    }
}
