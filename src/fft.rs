use std::ops::{Add, AddAssign};

#[derive(Clone, Copy, Debug)]
pub struct Complex {
    pub re: f32,
    pub im: f32,
}

impl Complex {
    fn new(re: f32, im: f32) -> Self {
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

pub fn cnum(re: f32, im: f32) -> Complex {
    Complex::new(re, im)
}

pub fn exp_im(im: f32) -> Complex {
    Complex::new(im.cos(), im.sin())
}
