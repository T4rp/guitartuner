use core::f32;
use cpal::{
    OutputCallbackInfo,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use guitartuning::fft::{Complex, ditfft2, hann_window, hann_window_cpx};
use std::{fs, thread, time::Duration};

const FFT_SIZE: usize = 4096;

const NOTE_FREQ: &[(&str, f32)] = &[
    ("E2", 82.41),
    ("A2", 110.0),
    ("D3", 146.83),
    ("G3", 196.0),
    ("E4", 329.63),
];

const FREQ_THRESH_HOLD: f32 = 50.0;

fn closest_note(freq: f32) -> Option<&'static str> {
    let mut closest_dist = f32::MAX;
    let mut detected_note: Option<&'static str> = None;

    for (note, f) in NOTE_FREQ.iter() {
        let diff = (*f - freq).abs();

        if diff >= FREQ_THRESH_HOLD {
            continue;
        }

        if diff < closest_dist {
            detected_note = Some(note);
            closest_dist = diff;
        }
    }

    detected_note
}

fn plot_sound(note: &'static str, input: &[Complex], output: &[Complex]) {
    use plotters::prelude::*;

    let root = BitMapBackend::new("soundpre.png", (1024, 768)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    root.draw_text(
        format!("note: {}", note).as_str(),
        &("sans-serif", 24).into_text_style(&root),
        (0, 0),
    )
    .unwrap();

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .right_y_label_area_size(40)
        .margin(5)
        .caption("Sound chart", ("sans-serif", 50.0).into_font())
        .build_cartesian_2d(0f32..(input.len() as f32), (-50.0f32..50.0f32))
        .unwrap();

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .y_label_formatter(&|x| format!("{:e}", x))
        .draw()
        .unwrap();

    chart
        .draw_series(LineSeries::new(
            input
                .iter()
                .enumerate()
                .map(|(i, x)| return (i as f32, x.re * 50.0)),
            &BLUE,
        ))
        .unwrap()
        .label("Original");

    chart
        .draw_series(LineSeries::new(
            output
                .iter()
                .enumerate()
                .map(|(i, x)| return (i as f32, x.re)),
            &RED,
        ))
        .unwrap()
        .label("Fourier transformed");

    root.present().unwrap();
    fs::copy("soundpre.png", "sound.png").unwrap();
}

fn main() {
    let host = cpal::default_host();
    let default_input = host.default_input_device().unwrap();

    let mut support_input_configs = default_input.supported_input_configs().unwrap();
    let config = support_input_configs
        .next()
        .unwrap()
        .with_max_sample_rate()
        .config();

    let sample_rate = config.sample_rate.0;

    thread::sleep(Duration::from_secs(2));

    let mut input: Vec<Complex> = vec![Complex::new(0.0, 0.0); FFT_SIZE];

    let stream = default_input
        .build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                for (i, m) in data.iter().take(FFT_SIZE).enumerate() {
                    input[i] = Complex::new(*m, 0.0);
                }

                hann_window_cpx(&mut input);

                let output = ditfft2(&input, input.len(), 1);

                let mut last_largest = 0.0;
                let mut max_index_at = 0;

                for (i, x) in output.iter().enumerate() {
                    if x.re > last_largest {
                        last_largest = x.re;
                        max_index_at = i
                    }
                }

                let magnitude = output[max_index_at].re;

                let max_freq = max_index_at as f32 * sample_rate as f32 / input.len() as f32;

                let freq = if max_index_at == 0 || max_index_at == output.len() {
                    max_freq
                } else {
                    let alpha = output[max_index_at - 1].re;
                    let beta = output[max_index_at].re;
                    let gamma = output[max_index_at + 1].re;

                    // parabolic interpolation
                    max_freq + 0.5 * (alpha - gamma) / (alpha - 2.0 * beta + gamma)
                };

                // if let Some(note) = closest_note(freq) {
                //     if magnitude > 25.0 {
                println!("note: {}, {} hz, mag: {}", "", freq, magnitude);
                plot_sound("", &input, &output);
                // }
                // }
            },
            move |err| {
                eprintln!("{}", err);
            },
            None,
        )
        .unwrap();

    stream.play().unwrap();
    thread::sleep(Duration::from_secs(100));
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
