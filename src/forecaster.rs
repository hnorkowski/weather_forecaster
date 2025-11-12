use rand::{Rng, rngs::ThreadRng, seq::SliceRandom};
use std::{collections::HashMap, fmt::Debug};
use strum::IntoEnumIterator;

use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::config::Config;

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
        match self {
            WeatherOptions::Clear => 2.4 / 14.0,
            WeatherOptions::LightCloud => 2.0 / 14.0,
            WeatherOptions::MediumCloud => 2.0 / 14.0,
            WeatherOptions::HeavyCloud => 1.0 / 14.0,
            WeatherOptions::Overcast => 1.0 / 14.0,
            WeatherOptions::LightRain => 0.4 / 14.0,
            WeatherOptions::Rain => 0.2 / 14.0,
            WeatherOptions::Storm => 0.3 / 14.0,
            WeatherOptions::Thunderstorm => 0.3 / 14.0,
            WeatherOptions::Foggy => 1.0 / 14.0,
            WeatherOptions::FogWithRain => 0.2 / 14.0,
            WeatherOptions::HeavyFog => 1.0 / 14.0,
            WeatherOptions::HeavyFogWithRain => 0.2 / 14.0,
            WeatherOptions::Hazy => 2.0 / 14.0,
            WeatherOptions::Random => 0.0,
        }
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
            [LightRain],
            [Rain, FogWithRain, HeavyFogWithRain],
            [Storm, Thunderstorm],
            [Foggy, HeavyFog, Hazy],
            [Random]
        )
    }

    #[must_use]
    pub fn rain_intensity(&self) -> usize {
        match self {
            WeatherOptions::LightRain => 1,
            WeatherOptions::Rain => 2,
            WeatherOptions::Storm => 3,
            WeatherOptions::Thunderstorm => 3,
            WeatherOptions::FogWithRain => 2,
            WeatherOptions::HeavyFogWithRain => 2,

            _ => 0,
        }
    }

    pub fn get_default_probablities() -> HashMap<WeatherOptions, f64> {
        let mut map = HashMap::new();

        for option in WeatherOptions::iter() {
            map.insert(option, option.get_default_probabiliy());
        }

        map
    }
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    clap::ValueEnum,
    EnumIter,
)]
pub enum Sessions {
    Practice = 1,
    Qualifying = 2,
    Race = 3,
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
    probabilities: HashMap<WeatherOptions, f64>,
    weather_slots: HashMap<Sessions, usize>,
    rng: ThreadRng,
}

impl Default for WeatherForecaster {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl WeatherForecaster {
    pub fn new(mut config: Config) -> Self {
        // sanatize proabilities
        let accumulated_probability: f64 = config.probabilities.values().sum();
        if accumulated_probability > 1.0 {
            eprintln!(
                "WARN: Your specified probabilites accumulate to {}%",
                (accumulated_probability * 100.0).round_to_decimal_place(2)
            );
            eprintln!("        -> Automatically fixing probabilities by normalizing them");
            eprintln!("        -> This might result in unexpected probabilities!");
        }

        let missing_entries = WeatherOptions::iter().len() - config.probabilities.len();
        let remaining_probability = (1.0 - accumulated_probability).clamp(0.0, 1.0);
        let remaining_options_probability = if missing_entries != 0 {
            remaining_probability / missing_entries as f64
        } else {
            0.0
        };

        let mut initial_probabilities = WeatherOptions::get_default_probablities();
        for entry in WeatherOptions::iter() {
            let probability = config
                .probabilities
                .get(&entry)
                .unwrap_or(&remaining_options_probability);
            initial_probabilities.insert(entry, *probability);
        }

        // sanitize weather slots
        let default_config = Config::default();
        for (session, slots) in default_config.weather_slots.into_iter() {
            let entry = config.weather_slots.entry(session).or_insert(slots);
            *entry = (*entry).clamp(1, 4);
        }

        let mut forecaster = Self {
            probabilities: initial_probabilities,
            weather_slots: config.weather_slots,
            rng: rand::rng(),
        };
        forecaster.normalize_probabilities();
        forecaster.print_probabilities();
        forecaster
    }

    pub fn print_probabilities(&self) {
        let max_length_option = WeatherOptions::iter()
            .map(|option| format!("{option:?}").len())
            .max()
            .unwrap()
            .max("Weather".len());

        println!("Using the following probabilities to generate a random weather forecast:");
        println!();
        println!("{:<len$} : Probability", "Weather", len = max_length_option);
        println!("{:-<len$} : -----------", "", len = max_length_option);
        for option in WeatherOptions::iter() {
            let probability =
                (*self.probabilities.get(&option).unwrap() * 100.0).round_to_decimal_place(2);
            println!(
                "{:<len$} : {probability}%",
                format!("{option:?}"),
                len = max_length_option
            );
        }
        println!();
    }

    fn generate_weather_option(&mut self, might_rain: bool) -> WeatherOptions {
        loop {
            let next_option: f64 = self.rng.random();
            let mut current_value = 0.0;
            let mut selected = WeatherOptions::Clear;
            for option in WeatherOptions::iter() {
                current_value += self.probabilities.get(&option).unwrap();
                if current_value > next_option {
                    selected = option;
                    break;
                }
            }
            if might_rain || selected.rain_intensity() == 0 {
                return selected;
            }
        }
    }

    pub fn generate_weather_option_in_group(
        &mut self,
        weather_option: WeatherOptions,
    ) -> WeatherOptions {
        let group = weather_option.get_group();
        let mut option;
        loop {
            option = self.generate_weather_option(true);
            if group.contains(&option) {
                break;
            }
        }
        option
    }

    pub fn generate_forecast(&mut self, sessions: &[Sessions]) -> WeatherForecast {
        let mut forecast = WeatherForecast::default();

        // race
        if sessions.contains(&Sessions::Race) {
            forecast.forecast.insert(
                Sessions::Race,
                self.generate_single_session_forecast(
                    *self.weather_slots.get(&Sessions::Race).unwrap(),
                    true,
                ),
            );
        }
        let race_rain = forecast
            .forecast
            .get(&Sessions::Race)
            .map(|options| {
                options
                    .iter()
                    .max_by_key(|option| option.rain_intensity())
                    .unwrap()
            })
            .filter(|option| option.rain_intensity() > 0)
            .copied();

        // quali
        if sessions.contains(&Sessions::Qualifying) {
            let quali = self.generate_single_session_forecast(
                *self.weather_slots.get(&Sessions::Qualifying).unwrap(),
                race_rain.is_some(),
            );

            forecast.forecast.insert(Sessions::Qualifying, quali);
        }

        // practice
        let practice_rain = race_rain.map(|option| self.generate_weather_option_in_group(option));
        if sessions.contains(&Sessions::Practice) {
            let mut practice = self.generate_single_session_forecast(
                *self.weather_slots.get(&Sessions::Practice).unwrap(),
                race_rain.is_some(),
            );
            if let Some(practice_rain) = practice_rain {
                *practice.last_mut().unwrap() = practice_rain;
                practice.shuffle(&mut self.rng);
            }
            forecast.forecast.insert(Sessions::Practice, practice);
        }

        forecast
    }

    fn generate_single_session_forecast(
        &mut self,
        weather_slots: usize,
        might_rain: bool,
    ) -> Vec<WeatherOptions> {
        if self.get_available_weather_options(might_rain) >= weather_slots {
            let mut options = Vec::new();
            while options.len() < weather_slots {
                let option = self.generate_weather_option(might_rain);
                if !options.contains(&option) {
                    options.push(option);
                }
            }
            options
        } else {
            (0..weather_slots)
                .map(|_| self.generate_weather_option(might_rain))
                .collect()
        }
    }

    fn get_available_weather_options(&self, with_rain: bool) -> usize {
        self.probabilities
            .iter()
            .filter(|(option, probability)| {
                (option.rain_intensity() == 0 || with_rain) && **probability > 0.0
            })
            .count()
    }

    fn normalize_probabilities(&mut self) {
        let sum: f64 = self.probabilities.values().sum();
        assert!(sum <= 1.0);
        let factor = 1.0 / sum;
        for probability in self.probabilities.values_mut() {
            *probability *= factor;
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct WeatherForecast {
    forecast: HashMap<Sessions, Vec<WeatherOptions>>,
}

impl std::fmt::Display for WeatherForecast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for session in Sessions::iter() {
            if let Some(forecast) = self.forecast.get(&session) {
                writeln!(f, r#""{session}WeatherSlots": {},"#, forecast.len())?;
                for (index, option) in forecast.iter().enumerate() {
                    writeln!(f, r#""{session}WeatherSlot{index}": "{option:?}","#)?;
                }
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

trait Round {
    fn round_to_decimal_place(&self, decimal_places: i32) -> Self;
}

impl Round for f64 {
    fn round_to_decimal_place(&self, decimal_places: i32) -> Self {
        let factor = 10.0_f64.powi(decimal_places);
        (self * factor).round() / factor
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use float_cmp::assert_approx_eq;
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn sane_probabilities() {
        let mut sum = 0.0;
        for option in WeatherOptions::iter() {
            sum += option.get_default_probabiliy();
        }
        assert_approx_eq!(f64, sum, 1.0, epsilon = 0.0001);
    }

    #[test]
    fn options_picked_correctly() {
        const NUMBER_OF_PICKS: usize = 100_000_000;
        let mut picked_times: HashMap<WeatherOptions, usize> =
            WeatherOptions::iter().map(|option| (option, 0)).collect();

        let mut forecaster = WeatherForecaster::default();

        for _ in 0..NUMBER_OF_PICKS {
            let option = forecaster.generate_weather_option(true);
            *picked_times.get_mut(&option).unwrap() += 1;
        }

        for (option, picks) in picked_times {
            let real_probability = picks as f64 / NUMBER_OF_PICKS as f64;
            let actual_probability = option.get_default_probabiliy();
            println!("{option:?} should be {real_probability} and is {actual_probability}");
            assert_approx_eq!(f64, real_probability, actual_probability, epsilon = 0.0005);
        }
    }
}
