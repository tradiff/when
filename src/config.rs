use anyhow::{Context, Result};
use serde::Deserialize;

/// Application configuration
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    /// Placeholder configuration value
    pub foo: String,
    /// Another placeholder configuration value
    pub bar: i32,
}

impl Settings {
    pub fn new() -> Result<Self> {
        let defaults = Self::default();

        let mut builder = config::Config::builder()
            .set_default("foo", defaults.foo)?
            .set_default("bar", defaults.bar)?;

        if let Some(config_dir) = dirs::config_dir() {
            builder =
                builder.add_source(config::File::from(config_dir.join("when")).required(false));
        }

        let config = builder.build().context("Failed to build configuration")?;

        config
            .try_deserialize()
            .context("Failed to deserialize configuration")
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            foo: "default_foo".to_string(),
            bar: 42,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default() {
        let settings = Settings::default();
        assert_eq!(settings.foo, "default_foo");
        assert_eq!(settings.bar, 42);
    }

    #[test]
    fn test_settings_new_uses_defaults() {
        // This should work even without a config file
        let result = Settings::new();
        assert!(result.is_ok());
        let settings = result.unwrap();
        // Should have default values when no config file exists
        assert!(!settings.foo.is_empty());
    }
}
