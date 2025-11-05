use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::exit,
};

use clap::Parser;
use cli_clipboard::{ClipboardContext, ClipboardProvider};
use serde::{Deserialize, Serialize};

use weather_forecaster::{Sessions, WeatherForecaster, WeatherOptions, plot};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Config {
    starting_probabilities: HashMap<WeatherOptions, f64>,
    plot_probabilities: bool,
    set_clipboard: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            starting_probabilities: WeatherOptions::get_default_probablities(),
            plot_probabilities: false,
            set_clipboard: false,
        }
    }
}

impl Config {
    fn generate_default_config(path: &Path) {
        let yaml = serde_yaml::to_string(&Config::default()).unwrap_or_print();
        std::fs::write(path, yaml).unwrap();
    }
}

#[derive(Debug, Parser)]
struct Args {
    /// Config file
    #[arg(short, long, default_value = "./config.yml")]
    config_file: PathBuf,

    /// Number of session to generate weather for
    #[arg(
        short,
        long,
        value_delimiter = ' ',
        num_args = 1..,
        default_value = "practice qualifying race"
    )]
    sessions: Vec<Sessions>,

    /// Number of weather slots
    #[arg(short, long, default_value = "4")]
    weather_slots: usize,
}

fn main() {
    let args = Args::parse();

    if !std::fs::exists(&args.config_file).unwrap_or_print() {
        Config::generate_default_config(&args.config_file);
    }

    let config: Config =
        serde_yaml::from_str(&std::fs::read_to_string(&args.config_file).unwrap_or_print())
            .unwrap_or_print();

    let mut forecaster = WeatherForecaster::new(config.starting_probabilities);

    let forecast = forecaster.generate_next_forecast(&args.sessions, args.weather_slots);

    println!("Forecast for your next Raceday:");
    println!("// {}\n", "=".repeat(80));
    print!("{forecast}");
    println!("// {}", "=".repeat(80));

    if config.plot_probabilities
        && let Err(error) = plot::plot_history(&forecaster.history)
    {
        eprintln!("Plotting failed: {error}");
    }

    if let Ok(mut clipboard) = ClipboardContext::new()
        && config.set_clipboard
    {
        clipboard.set_contents(forecast.to_string()).unwrap();
    }
}

trait UnwrapOrPrint<T> {
    fn unwrap_or_print(self) -> T;
}

impl<T, E: std::error::Error> UnwrapOrPrint<T> for Result<T, E> {
    fn unwrap_or_print(self) -> T {
        match self {
            Ok(value) => value,
            Err(error) => {
                eprintln!("{error}");
                exit(1)
            }
        }
    }
}
