use std::collections::HashMap;

use plotters::prelude::*;
use strum::IntoEnumIterator;

use crate::WeatherOptions;

pub fn plot_history(
    history: &HashMap<WeatherOptions, Vec<f64>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("probability_evolution.png", (1920, 1080)).into_drawing_area();
    root.fill(&RGBColor(36, 36, 36))?;

    let max_value = *history
        .values()
        .map(|data| {
            data.iter()
                .reduce(|max, next| if next > max { next } else { max })
                .unwrap()
        })
        .reduce(|max, next| if next > max { next } else { max })
        .unwrap()
        * 100.0;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Weather Option Probability Evolution",
            ("DejaVu Sans", 32).into_font().color(&WHITE),
        )
        .x_label_area_size(40)
        .y_label_area_size(60)
        .margin(20)
        .build_cartesian_2d(
            0..history.get(&WeatherOptions::Clear).unwrap().len() + 2,
            0.0..(max_value * 1.2),
        )?;

    chart
        .configure_mesh()
        .x_desc("updates")
        .y_desc("probability")
        .axis_style(WHITE)
        .label_style(("DejaVu Sans", 24).into_font().color(&WHITE))
        .bold_line_style(WHITE.mix(0.3))
        .light_line_style(WHITE.mix(0.1))
        .draw()?;

    for (index, option) in WeatherOptions::iter().enumerate() {
        let data: Vec<f64> = history
            .get(&option)
            .unwrap()
            .iter()
            .map(|probability| probability * 100.0)
            .collect();

        chart
            .draw_series(LineSeries::new(
                data.into_iter().enumerate(),
                Palette99::pick(index).stroke_width(3),
            ))?
            .label(format!("{option:?}"))
            .legend(move |(x, y)| {
                // Add color patch for legend
                Rectangle::new(
                    [(x, y - 4), (x + 20, y + 4)],
                    Palette99::pick(index).filled(),
                )
            });
    }

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .background_style(BLACK.mix(0.8))
        .border_style(WHITE)
        .label_font(("DejaVu Sans", 28).into_font().color(&WHITE)) // Make legend text visible
        .draw()?;

    Ok(())
}
