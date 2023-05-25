use crate::Color;

use super::SpectrumT;

#[derive(Debug)]
pub struct RGBSpectrum {
    pub color: Color,
}

impl RGBSpectrum {
    pub fn new(color: Color) -> Self {
        Self { color }
    }
}

impl SpectrumT for RGBSpectrum {}
