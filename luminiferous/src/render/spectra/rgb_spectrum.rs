use core::ops::*;

use super::SpectrumT;

#[doc(hidden)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RgbSpectrum {
    c: [f32; 3],
}

impl SpectrumT for RgbSpectrum {
    fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Self { c: [r, g, b] }
    }

    fn splat(x: f32) -> Self {
        Self { c: [x, x, x] }
    }

    fn zero() -> Self {
        Self { c: [0.0, 0.0, 0.0] }
    }

    fn is_black(&self) -> bool {
        for s in self.c {
            if s != 0.0 {
                return false;
            }
        }

        true
    }

    fn has_nan(&self) -> bool {
        for s in self.c {
            if s.is_nan() {
                return true;
            }
        }

        false
    }

    fn to_rgb(&self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }

    fn to_xyz(&self) -> [f32; 3] {
        [
            0.412453 * self[0] + 0.357580 * self[1] + 0.180423 * self[2],
            0.212671 * self[0] + 0.715160 * self[1] + 0.072169 * self[2],
            0.019334 * self[0] + 0.119193 * self[1] + 0.950227 * self[2],
        ]
    }

    fn y(&self) -> f32 {
        0.212671 * self[0] + 0.715160 * self[1] + 0.072169 * self[2]
    }

    fn exp(&self) -> Self {
        Self {
            c: self.c.map(|x| x.exp()),
        }
    }
}

impl Deref for RgbSpectrum {
    type Target = Rgb;

    fn deref(&self) -> &Self::Target {
        // SAFETY: self is guaranteed to be a valid pointer
        unsafe { &*(self as *const Self).cast() }
    }
}

impl DerefMut for RgbSpectrum {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: see above
        unsafe { &mut *(self as *mut Self).cast() }
    }
}

impl Index<usize> for RgbSpectrum {
    type Output = f32;
    #[inline]
    fn index(&self, i: usize) -> &Self::Output {
        &self.c[i]
    }
}

impl IndexMut<usize> for RgbSpectrum {
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.c[i]
    }
}

impl Neg for RgbSpectrum {
    type Output = RgbSpectrum;
    fn neg(self) -> Self::Output {
        RgbSpectrum {
            c: self.c.map(|x| -x),
        }
    }
}

impl Add for RgbSpectrum {
    type Output = RgbSpectrum;

    fn add(self, rhs: RgbSpectrum) -> RgbSpectrum {
        RgbSpectrum {
            c: [self[0] + rhs[0], self[1] + rhs[1], self[2] + rhs[2]],
        }
    }
}

impl Sub for RgbSpectrum {
    type Output = RgbSpectrum;

    fn sub(self, rhs: RgbSpectrum) -> RgbSpectrum {
        RgbSpectrum {
            c: [self[0] - rhs[0], self[1] - rhs[1], self[2] - rhs[2]],
        }
    }
}

impl Mul for RgbSpectrum {
    type Output = RgbSpectrum;

    fn mul(self, rhs: RgbSpectrum) -> RgbSpectrum {
        RgbSpectrum {
            c: [self[0] * rhs[0], self[1] * rhs[1], self[2] * rhs[2]],
        }
    }
}

impl Div for RgbSpectrum {
    type Output = RgbSpectrum;

    fn div(self, rhs: RgbSpectrum) -> RgbSpectrum {
        RgbSpectrum {
            c: [self[0] / rhs[0], self[1] / rhs[1], self[2] / rhs[2]],
        }
    }
}

impl AddAssign for RgbSpectrum {
    fn add_assign(&mut self, rhs: Self) {
        self[0] += rhs[0];
        self[1] += rhs[1];
        self[2] += rhs[2];
    }
}

impl SubAssign for RgbSpectrum {
    fn sub_assign(&mut self, rhs: Self) {
        self[0] -= rhs[0];
        self[1] -= rhs[1];
        self[2] -= rhs[2];
    }
}

impl MulAssign for RgbSpectrum {
    fn mul_assign(&mut self, rhs: Self) {
        self[0] *= rhs[0];
        self[1] *= rhs[1];
        self[2] *= rhs[2];
    }
}

impl DivAssign for RgbSpectrum {
    fn div_assign(&mut self, rhs: Self) {
        self[0] /= rhs[0];
        self[1] /= rhs[1];
        self[2] /= rhs[2];
    }
}

impl MulAssign<f32> for RgbSpectrum {
    fn mul_assign(&mut self, rhs: f32) {
        self[0] *= rhs;
        self[1] *= rhs;
        self[2] *= rhs;
    }
}

impl DivAssign<f32> for RgbSpectrum {
    fn div_assign(&mut self, rhs: f32) {
        self[0] /= rhs;
        self[1] /= rhs;
        self[2] /= rhs;
    }
}

impl Mul<f32> for RgbSpectrum {
    type Output = RgbSpectrum;

    fn mul(self, rhs: f32) -> RgbSpectrum {
        RgbSpectrum {
            c: [self[0] * rhs, self[1] * rhs, self[2] * rhs],
        }
    }
}

impl Div<f32> for RgbSpectrum {
    type Output = RgbSpectrum;

    fn div(self, rhs: f32) -> RgbSpectrum {
        RgbSpectrum {
            c: [self[0] / rhs, self[1] / rhs, self[2] / rhs],
        }
    }
}

impl Mul<RgbSpectrum> for f32 {
    type Output = RgbSpectrum;

    fn mul(self, rhs: RgbSpectrum) -> RgbSpectrum {
        RgbSpectrum {
            c: [self * rhs[0], self * rhs[1], self * rhs[2]],
        }
    }
}

impl Div<RgbSpectrum> for f32 {
    type Output = RgbSpectrum;

    fn div(self, rhs: RgbSpectrum) -> RgbSpectrum {
        RgbSpectrum {
            c: [self / rhs[0], self / rhs[1], self / rhs[2]],
        }
    }
}
