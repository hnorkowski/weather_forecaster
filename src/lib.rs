use rand::Rng;
use std::{collections::HashMap, process::exit};
use strum::IntoEnumIterator;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Serialize, Deserialize)]
pub enum WeatherOptions {
    Clear,
    LightCloud,
    MediumCloud,
    HeavyCloud,
    Overcast,
    LightRain,
    Rain,
    Storm,
    Thunderstorm,
    Foggy,
    FogWithRain,
    HeavyFog,
    HeavyFogWithRain,
    Hazy,
    Random,
}

impl WeatherOptions {
    pub fn get_default_probabiliy(&self) -> f64 {
        let default_probabiliy = 1.0 / (Self::iter().len() - 1) as f64;
        match self {
            WeatherOptions::Clear => default_probabiliy,
            WeatherOptions::LightCloud => default_probabiliy,
            WeatherOptions::MediumCloud => default_probabiliy,
            WeatherOptions::HeavyCloud => default_probabiliy,
            WeatherOptions::Overcast => default_probabiliy,
            WeatherOptions::LightRain => default_probabiliy,
            WeatherOptions::Rain => default_probabiliy,
            WeatherOptions::Storm => default_probabiliy,
            WeatherOptions::Thunderstorm => default_probabiliy,
            WeatherOptions::Foggy => default_probabiliy,
            WeatherOptions::FogWithRain => default_probabiliy,
            WeatherOptions::HeavyFog => default_probabiliy,
            WeatherOptions::HeavyFogWithRain => default_probabiliy,
            WeatherOptions::Hazy => default_probabiliy,
            WeatherOptions::Random => 0.0,
        }
    }

    pub fn get_default_reintroduction_probabiliy(&self) -> f64 {
        1.0 + self.get_default_probabiliy() * 0.5
    }

    pub fn get_group(&self) -> &[WeatherOptions] {
        macro_rules! weather_groups {
            ( $([$( $option:ident ),+]),+ ) => {
                match self {
                    $(
                        $(WeatherOptions::$option )|+ => {
                            &[$(WeatherOptions::$option),+]
                        }
                    ),+
                }
            };
        }

        weather_groups!(
            [Clear, LightCloud],
            [MediumCloud, HeavyCloud, Overcast],
            [
                LightRain,
                Rain,
                Storm,
                Thunderstorm,
                FogWithRain,
                HeavyFogWithRain
            ],
            [Foggy, HeavyFog, Hazy],
            [Random]
        )
    }

    pub fn get_default_probablities() -> HashMap<WeatherOptions, f64> {
        let mut map = HashMap::new();

        for option in WeatherOptions::iter() {
            map.insert(option, option.get_default_probabiliy());
        }

        map
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, clap::ValueEnum)]
pub enum Sessions {
    Practice,
    Qualifying,
    Race,
}

impl std::fmt::Display for Sessions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Qualifying => write!(f, "Qualify"),
            other => write!(f, "{other:?}"),
        }
    }
}

#[derive(Debug)]
pub struct WeatherForecaster {
    initial_probabilities: HashMap<WeatherOptions, f64>,
    current_probabilities: HashMap<WeatherOptions, f64>,
}

impl WeatherForecaster {
    pub fn new(starting_probabilities: HashMap<WeatherOptions, f64>) -> Self {
        let accumulated_probability: f64 = starting_probabilities.values().sum();
        if accumulated_probability > 1.0 {
            eprintln!("The specified probabilites accumulate to more than 100%");
            exit(1);
        }

        let missing_entries = WeatherOptions::iter().len() - starting_probabilities.len();
        if accumulated_probability - 1.0 > 0.001 && missing_entries == 0 {
            eprintln!(
                "You specified all probabilites but they do not add up to 100%: {accumulated_probability}"
            );
            exit(1);
        }

        let remaining_probability = 1.0 - accumulated_probability;
        let default_probability = if missing_entries != 0 {
            remaining_probability / missing_entries as f64
        } else {
            0.0
        };

        let mut initial_probabilities = WeatherOptions::get_default_probablities();
        for entry in WeatherOptions::iter() {
            let probability = starting_probabilities
                .get(&entry)
                .unwrap_or(&default_probability);
            initial_probabilities.insert(entry, *probability);
        }
        let current_probabilities = initial_probabilities.clone();

        Self { initial_probabilities, current_probabilities }
    }

    pub fn generate_next_forecast(
        &mut self,
        sessions: &[Sessions],
        weather_slots: usize,
    ) -> WeatherForecast {
        let mut rng = rand::rng();
        let mut forecast = WeatherForecast::default();
        for session in sessions {
            let mut weather = Vec::new();
            for _ in 0..weather_slots {
                // TODO: reintroduce options
                let next_option: f64 = rng.random();
                let mut current_value = 0.0;
                for option in WeatherOptions::iter() {
                    current_value += self.current_probabilities.get(&option).unwrap();
                    if current_value > next_option {
                        self.disable_group(&option);
                        self.normalize_probabilities();
                        dbg!(&self.current_probabilities);
                        self.reintroduce_probabilities();
                        weather.push(option);
                        break;
                    }
                }
            }
            assert_eq!(weather.len(), weather_slots);
            forecast.forecast.insert(*session, weather);
        }

        forecast
    }

    fn disable_group(&mut self, weather_option: &WeatherOptions) {
        for group_option in weather_option.get_group() {
            *self.current_probabilities.get_mut(group_option).unwrap() = 0.0;
        }
    }

    fn normalize_probabilities(&mut self) {
        let sum: f64 = self.current_probabilities.values().sum();
        let factor = 1.0 / sum;
        for probability in self.current_probabilities.values_mut() {
            *probability *= factor;
        }
    }

    fn reintroduce_probabilities(&mut self) {
        for (option, probability) in self.current_probabilities.iter_mut() {
            let default_probabiliy = *self.initial_probabilities.get(option).unwrap();
            let factor = option.get_default_reintroduction_probabiliy();

            if *probability - 0.001 <= 0.0 {
                *probability = factor - 1.0;
            } else if *probability + 0.001 < default_probabiliy {
                *probability = (*probability * factor).clamp(0.0, default_probabiliy);
            }
        }
        self.normalize_probabilities();
    }
}

#[derive(Debug, Clone, Default)]
pub struct WeatherForecast {
    forecast: IndexMap<Sessions, Vec<WeatherOptions>>,
}

impl std::fmt::Display for WeatherForecast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (session, forecast) in &self.forecast {
            writeln!(f, r#""{session}WeatherSlots": {},"#, forecast.len())?;
            for (index, option) in forecast.iter().enumerate() {
                writeln!(f, r#""{session}WeatherSlot{index}": "{option:?}","#)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
