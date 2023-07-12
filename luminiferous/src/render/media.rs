use enum_dispatch::enum_dispatch;

mod homogeneous;
pub use homogeneous::*;

use crate::prelude::*;
use crate::{phase_functions::PhaseFunction, spectra::Spectrum};

pub struct MediumInteraction {
    pub p: Point3,
    pub wi: Vector3,
    pub phase_function: Option<PhaseFunction>,
}

impl MediumInteraction {
    pub fn valid(&self) -> bool {
        self.phase_function.is_some()
    }
}

#[enum_dispatch]
pub trait MediumT {
    fn sample(&self, ray: Ray, t_max: f32, u1: f32) -> Option<(MediumInteraction, Spectrum)>;
}

#[enum_dispatch(MediumT)]
#[derive(Debug, Clone, PartialEq)]
pub enum Medium {
    Homogeneous(HomogeneousMedium),
}

#[derive(Debug, Clone)]
pub struct MediumInterface {
    pub inside: Option<Medium>,
    pub outside: Option<Medium>,
}

impl MediumInterface {
    pub fn none() -> Self {
        Self {
            inside: None,
            outside: None,
        }
    }

    pub fn new(inside: Option<Medium>, outside: Option<Medium>) -> Self {
        Self { inside, outside }
    }

    pub fn is_transition(&self) -> bool {
        self.inside != self.outside
    }
}
