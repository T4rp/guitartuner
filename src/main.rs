use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{thread, time::Duration};

fn main() {
    let host = cpal::default_host();
    let default_input = host.default_input_device().unwrap();

    let mut support_input_configs = default_input.supported_input_configs().unwrap();
    let config = support_input_configs
        .next()
        .unwrap()
        .with_max_sample_rate()
        .config();

    let stream = default_input
        .build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                println!("{}", data[0]);
                println!("{}", data.last().unwrap());
                println!("{}", data.len());
            },
            move |err| {
                eprintln!("{}", err);
            },
            None,
        )
        .unwrap();

    stream.play().unwrap();
    thread::sleep(Duration::from_secs(10));
}

#[cfg(test)]
mod tests {
    use plotters::prelude::*;

    #[test]
    fn plot() {
        let root = BitMapBackend::new("test.png", (1024, 768)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(35)
            .y_label_area_size(40)
            .right_y_label_area_size(40)
            .margin(5)
            .caption("Dual Y-Axis Example", ("sans-serif", 50.0).into_font())
            .build_cartesian_2d(0f32..10f32, (0.1f32..1e10f32).log_scale())
            .unwrap()
            .set_secondary_coord(0f32..10f32, -1.0f32..1.0f32);

        chart
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            .y_desc("Log Scale")
            .y_label_formatter(&|x| format!("{:e}", x))
            .draw()
            .unwrap();

        chart
            .configure_secondary_axes()
            .y_desc("Linear Scale")
            .draw()
            .unwrap();

        chart
            .draw_series(LineSeries::new(
                (0..=100).map(|x| (x as f32 / 10.0, (1.02f32).powf(x as f32 * x as f32 / 10.0))),
                &BLUE,
            ))
            .unwrap()
            .label("y = 1.02^x^2")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

        chart
            .draw_secondary_series(LineSeries::new(
                (0..=100).map(|x| (x as f32 / 10.0, (x as f32 / 5.0).sin())),
                &RED,
            ))
            .unwrap()
            .label("y = sin(2x)")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

        chart
            .configure_series_labels()
            .background_style(RGBColor(128, 128, 128))
            .draw()
            .unwrap();
    }
}
