use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::forecaster::{Sessions, WeatherOptions};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub probabilities: HashMap<WeatherOptions, f64>,
    pub weather_slots: HashMap<Sessions, usize>,
    pub set_clipboard: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            probabilities: WeatherOptions::get_default_probablities(),
            weather_slots: [
                (Sessions::Practice, 4),
                (Sessions::Qualifying, 2),
                (Sessions::Race, 4),
            ]
            .into_iter()
            .collect(),
            set_clipboard: false,
        }
    }
}

impl Config {
    pub fn generate_default_config(path: &Path) -> Result<(), std::io::Error> {
        let yaml = serde_yaml::to_string(&Config::default()).unwrap();
        std::fs::write(path, yaml)?;
        Ok(())
    }
}
