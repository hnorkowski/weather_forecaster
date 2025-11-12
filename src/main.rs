use std::{path::PathBuf, process::exit};

use clap::Parser;
use cli_clipboard::{ClipboardContext, ClipboardProvider};

use weather_forecaster::{
    config::Config,
    forecaster::{Sessions, WeatherForecaster},
};

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
}

fn main() {
    let args = Args::parse();

    if !std::fs::exists(&args.config_file).unwrap_or_print() {
        Config::generate_default_config(&args.config_file).unwrap_or_print();
    }

    let config: Config =
        serde_yaml::from_str(&std::fs::read_to_string(&args.config_file).unwrap_or_print())
            .unwrap_or_print();

    let mut forecaster = WeatherForecaster::new(config.clone());

    let forecast = forecaster.generate_forecast(&args.sessions);

    println!("Forecast for your next Raceday:");
    println!("// {}\n", "=".repeat(80));
    print!("{forecast}");
    println!("// {}", "=".repeat(80));

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
