mod rgb_spectrum;
pub use rgb_spectrum::*;

pub trait SpectrumT {}

pub enum Spectrum {
    RGB(RGBSpectrum),
}
