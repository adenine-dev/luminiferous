use core::ops::*;
use enum_dispatch::enum_dispatch;

mod rgb_spectrum;
pub use rgb_spectrum::*;

#[enum_dispatch]
pub trait SpectrumT:
    Sized
    // + Default
    + PartialEq 
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self, Output = Self>
    + Div<Self, Output = Self>
    + AddAssign<Self>
    + SubAssign<Self>
    + MulAssign<Self>
    + DivAssign<Self>
    + Mul<f32, Output = Self>
    + Div<f32, Output = Self>
    + MulAssign<f32>
    + DivAssign<f32>
    + Neg
{
    fn from_rgb(r: f32, g: f32, b: f32) -> Self;
    
    fn splat(x: f32) -> Self;

    fn zero() -> Self;
    
    fn is_black(&self) -> bool;

    fn has_nan(&self) -> bool;

    fn to_rgb(&self) -> [f32; 3];

    fn to_xyz(&self) -> [f32; 3];

    fn y(&self) -> f32;

    fn exp(&self) -> Self;

    fn sqrt(&self) -> Self;
}

pub type Spectrum = RgbSpectrum;

// #[derive(Debug, Clone)]
// #[enum_dispatch(SpectrumT)]
// pub enum Spectrum {
//     Rgb(RgbSpectrum),
// }


