use std::{
    f32::consts::PI,
    ops::{Add, AddAssign, Mul, MulAssign},
};

#[derive(Clone, Copy, Debug)]
pub struct Complex {
    pub re: f32,
    pub im: f32,
}

impl Complex {
    pub fn new(re: f32, im: f32) -> Self {
        Self { re, im }
    }
}

impl Add for Complex {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Complex {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl AddAssign for Complex {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Mul for Complex {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Complex {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

impl MulAssign for Complex {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

pub fn cnum(re: f32, im: f32) -> Complex {
    Complex::new(re, im)
}

pub fn exp_im(im: f32) -> Complex {
    Complex::new(im.cos(), im.sin())
}

pub fn ditfft2(x: &[Complex], n: usize, s: usize) -> Vec<Complex> {
    if n == 1 {
        return vec![x[0]];
    }

    let even = ditfft2(x, n / 2, s * 2);
    let odd = ditfft2(&x[s..], n / 2, s * 2);

    let mut output = vec![Complex::new(0.0, 0.0); n];
    for k in 0..(n / 2) {
        let q = exp_im(-2.0 * PI * k as f32 * n as f32) * odd[k];
        output[k] = even[k] + q;
        output[k + n / 2] = even[k] + (q * Complex::new(-1.0, 0.0))
    }

    output
}

pub fn hann_window(input: &mut [f32]) {
    let len = input.len();
    for (i, n) in input.iter_mut().enumerate() {
        let multiplier = 0.5 * (1.0 - (2.0 * PI * i as f32 / len as f32).cos());
        *n = multiplier * *n
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotters::prelude::*;

    #[test]
    fn runs_normally() {
        let input = vec![0.0, 1.0, 3.0, 4.0];
        let cinput: Vec<Complex> = input.iter().map(|n| Complex::new(*n, 0.0)).collect();
        std::hint::black_box(ditfft2(&cinput, cinput.len(), 1));
    }

    #[test]
    fn run_and_plot() {
        let mut input: Vec<f32> = (0..100)
            .into_iter()
            .map(|x| (x as f32 / 10.0 * 10.0).sin())
            .collect();

        let mut windowed_input = input.clone();

        hann_window(&mut windowed_input);

        let cinput: Vec<Complex> = windowed_input
            .iter()
            .map(|n| Complex::new(*n, 0.0))
            .collect();
        let out = ditfft2(&cinput, cinput.len(), 1);

        let mut last_largest = 0.0;
        let mut max_index_at = 0;

        for (i, x) in out.iter().enumerate() {
            if x.re > last_largest {
                last_largest = x.re;
                max_index_at = i
            }
        }

        let root = BitMapBackend::new("coolio.png", (1024, 768)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        root.draw_text(
            format!("Largest: {}", max_index_at as f32 / 10.0).as_str(),
            &("sans-serif", 24).into_text_style(&root),
            (0, 0),
        );

        let mut chart = ChartBuilder::on(&root)
            .x_label_area_size(35)
            .y_label_area_size(40)
            .right_y_label_area_size(40)
            .margin(5)
            .caption("FFT plot", ("sans-serif", 50.0).into_font())
            .build_cartesian_2d(0f32..100.0, (-50.0f32..50.0f32))
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
                input.iter().enumerate().map(|(i, x)| return (i as f32, *x)),
                &BLUE,
            ))
            .unwrap()
            .label("Original");

        chart
            .draw_series(LineSeries::new(
                out.iter().enumerate().map(|(i, x)| return (i as f32, x.re)),
                &RED,
            ))
            .unwrap()
            .label("Fourier transformed");

        root.present().unwrap();
    }
}
