mod fft;

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
