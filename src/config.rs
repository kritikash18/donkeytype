//! Module reading and parsing config
//!
//! Default options of configuration are:
//!
//! | name              | default value          | type in JSON | description                                                          |
//! | ----------------- | ---------------------- | ------------ | -------------------------------------------------------------------- |
//! | `duration`        | `30`                   | number       | duration of the test in seconds                                      |
//! | `numbers`         | `false`                | boolean      | flag indicating if numbers should be inserted in expected input      |
//! | `numbers_ratio`   | `0.05` if numbers=TRUE | number       | ratio for putting numbers in the test                                |
//! | `dictionary_path` | `"src/dict/words.txt"` | string       | dictionary words to sample from while creating test's expected input |
//!
//! `NOTE: If provided numbers_ratio is not between 0 to 1.0, Default numbers_ratio = 0.05 will be used.`
//!
//!
//! Configuration will grow when more features are added (_different modes_, _different languages_, _configuring colors_).
//!
//! You can provide this config as options when running the program like so:
//!
//! ```shell
//! cargo run -- --duration 60 --dictionary-path "/usr/share/dict/words" --numbers true --numbers-ratio 0.1
//! ```
//!
//! or put them in a config file in `~/.config/donkeytype/donkeytype-config.json`:
//!
//! ```json
//! {
//!     "duration": 60,
//!     "dictionary_path": "/usr/share/dict/words",
//!     "numbers": true,
//!     "numbers_ratio": 0.1
//! }
//! ```

use anyhow::{Context, Result};
use mockall::*;
use serde::{Deserialize, Serialize};
use std::{fs, io::Read, path::PathBuf, time::Duration};

use crate::color_scheme::ColorScheme;

use crate::Args;

/// Main program configuration
#[derive(Debug)]
pub struct Config {
    pub duration: Duration,
    pub numbers: bool,
    pub numbers_ratio: f64,
    pub dictionary_path: PathBuf,
    pub uppercase: bool,
    pub uppercase_ratio: f64,
    pub color_config: ColorScheme,
}

/// Used by `serde` crate to parse config file into a rust struct
#[derive(Deserialize, Serialize, Debug)]
struct ConfigFile {
    pub duration: Option<u64>,
    pub numbers: Option<bool>,
    pub numbers_ratio: Option<f64>,
    pub dictionary_path: Option<String>,
    pub uppercase: Option<bool>,
    pub uppercase_ratio: Option<f64>,
    pub color_config: Option<ConfigFileColorScheme>,
}

#[derive(Deserialize, Serialize, Debug)]
struct ConfigFileColorScheme {
    pub correct_match_fg: Option<String>,
    pub correct_match_bg: Option<String>,
    pub incorrect_match_fg: Option<String>,
    pub incorrect_match_bg: Option<String>,
}

#[automock]
impl Config {
    /// Provide default values for configuration options
    pub fn default() -> Self {
        Self {
            duration: Duration::from_secs(30),
            numbers: false,
            numbers_ratio: 0.05,
            dictionary_path: PathBuf::from("src/dict/words.txt"),
            uppercase: false,
            uppercase_ratio: 0.45,
            color_config: ColorScheme::default(),
        }
    }

    /// Setup configuration
    ///
    /// Create config with default values
    /// then overwrite them with any values provided in config file
    /// then overwrite it again with any values provide as arguments to the program
    pub fn new(args: Args, config_file_path: PathBuf) -> Result<Self> {
        let config = {
            let mut config = Self::default();

            let config_file = open_config_file_if_exists(config_file_path.clone())
                .context("Unable to open config file")?;
            if let Some(config_file) = config_file {
                augment_config_with_config_file(&mut config, config_file)
                    .context("Unable to augment config with config file")?;
            }
            augment_config_with_args(&mut config, args);

            config
        };

        Ok(config)
    }
}

/// Overwrite provided config with options parsed from configuration file
fn augment_config_with_config_file(config: &mut Config, mut config_file: fs::File) -> Result<()> {
    if config_file.metadata().is_ok() {
        let mut config_file_content = String::new();
        config_file
            .read_to_string(&mut config_file_content)
            .context("Unable to read file")?;

        let config_from_file: ConfigFile =
            serde_json::from_str(&config_file_content).context("Unable to parse config file")?;

        if let Some(duration) = config_from_file.duration {
            config.duration = Duration::from_secs(duration);
        }

        if let Some(numbers) = config_from_file.numbers {
            config.numbers = numbers;
        }

        if let Some(numbers_ratio) = config_from_file.numbers_ratio {
            if numbers_ratio >= 0.0 && numbers_ratio <= 1.0 {
                config.numbers_ratio = numbers_ratio
            }
        }

        if let Some(dictionary_path) = config_from_file.dictionary_path {
            config.dictionary_path = PathBuf::from(dictionary_path);
        }

        if let Some(uppercase) = config_from_file.uppercase {
            config.uppercase = uppercase;
        }

        if let Some(uppercase_ratio) = config_from_file.uppercase_ratio {
            if uppercase_ratio >= 0.0 && uppercase_ratio <= 1.0 {
                config.uppercase_ratio = uppercase_ratio
            }
        }

        if let Some(color_config) = config_from_file.color_config {
            if let Some(correct_match_fg) = color_config.correct_match_fg {
              config.color_config.correct_match_fg = correct_match_fg.parse().unwrap();
            }

            if let Some(correct_match_bg) = color_config.correct_match_bg {
              config.color_config.correct_match_bg = correct_match_bg.parse().unwrap();
            }

            if let Some(incorrect_match_fg) = color_config.incorrect_match_fg {
              config.color_config.incorrect_match_fg = incorrect_match_fg.parse().unwrap();
            }

            if let Some(incorrect_match_bg) = color_config.incorrect_match_bg {
              config.color_config.incorrect_match_bg = incorrect_match_bg.parse().unwrap();
            }
        }
    }

    Ok(())
}

fn open_config_file_if_exists(config_file: PathBuf) -> Result<Option<fs::File>> {
    if config_file.exists() {
        let config_file = fs::File::open(config_file).context("Unable to open config file")?;
        return Ok(Some(config_file));
    }

    return Ok(None);
}

/// Overwrite provided config with values from args object
fn augment_config_with_args(config: &mut Config, args: Args) {
    if let Some(numbers_flag) = args.numbers {
        config.numbers = numbers_flag;
    }
    if let Some(numbers_ratio) = args.numbers_ratio {
        if numbers_ratio >= 0.0 && numbers_ratio <= 1.0 {
            config.numbers_ratio = numbers_ratio
        }
    }
    if let Some(duration) = args.duration {
        config.duration = Duration::from_secs(duration);
    }
    if let Some(dictionary_path) = args.dictionary_path {
        config.dictionary_path = PathBuf::from(dictionary_path);
    }

    if let Some(uppercase_flag) = args.uppercase {
        config.uppercase = uppercase_flag
    }

    if let Some(uppercase_ratio) = args.uppercase_ratio {
        if uppercase_ratio >= 0.0 && uppercase_ratio <= 1.0 {
            config.uppercase_ratio = uppercase_ratio
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;

    #[test]
    fn should_create_default_values() {
        let config = Config::default();

        assert_eq!(config.duration, Duration::from_secs(30));
        assert_eq!(config.numbers, false);
        assert_eq!(config.numbers_ratio, 0.05);
    }

    #[test]
    fn should_create_new_with_default_values() {
        let args = Args {
            duration: None,
            numbers: None,
            numbers_ratio: None,
            dictionary_path: None,
            uppercase: None,
            uppercase_ratio: None
        };
        let config = Config::new(args, PathBuf::new()).expect("Unable to create config");

        assert_eq!(config.duration, Duration::from_secs(30));
        assert_eq!(config.numbers, false);
        assert_eq!(config.numbers_ratio, 0.05);
    }

    #[test]
    fn should_create_new_config_with_config_file_values() {
        let mut config_file = tempfile::NamedTempFile::new().expect("Unable to create temp file");
        config_file
            .write_all(r#"{"duration": 10, "numbers": true }"#.as_bytes())
            .expect("Unable to write to temp file");

        let args = Args {
            duration: None,
            numbers: None,
            numbers_ratio: None,
            dictionary_path: None,
            uppercase: None,
            uppercase_ratio: None
        };
        let config =
            Config::new(args, config_file.path().to_path_buf()).expect("Unable to create config");

        assert_eq!(config.duration, Duration::from_secs(10));
        assert_eq!(config.numbers, true);
        assert_eq!(config.numbers_ratio, 0.05);
    }

    #[test]
    fn should_create_new_with_argument_values() {
        let args = Args {
            duration: Some(10),
            numbers: Some(true),
            numbers_ratio: None,
            dictionary_path: None,
            uppercase: None,
            uppercase_ratio: None
        };
        let config = Config::new(args, PathBuf::new()).expect("Unable to create config");

        assert_eq!(config.duration, Duration::from_secs(10));
        assert_eq!(config.numbers, true);
        assert_eq!(config.numbers_ratio, 0.05);
    }

    #[test]
    fn args_should_take_precedence_over_config_file() {
        let mut config_file = tempfile::NamedTempFile::new().expect("Unable to create temp file");
        config_file
            .write_all(r#"{"duration": 10, "numbers": true }"#.as_bytes())
            .expect("Unable to write to temp file");

        let args = Args {
            duration: Some(20),
            numbers: Some(false),
            numbers_ratio: None,
            dictionary_path: Some(String::from("/etc/dict/words")),
            uppercase: None,
            uppercase_ratio: None
        };
        let config =
            Config::new(args, config_file.path().to_path_buf()).expect("Unable to create config");

        assert_eq!(config.duration, Duration::from_secs(20));
        assert_eq!(config.numbers, false);
        assert_eq!(config.numbers_ratio, 0.05);
        assert_eq!(config.dictionary_path, PathBuf::from("/etc/dict/words"));
    }
}
