mod homogeneous;
pub use homogeneous::*;

use crate::prelude::*;
use crate::{phase_functions::PhaseFunction, primitive::Interaction, spectra::Spectrum};

#[derive(Debug, Clone)]
pub struct MediumInteraction<'a> {
    pub p: Point3,
    pub wi: Vector3,
    pub medium: Option<&'a Medium>,
    pub phase_function: Option<PhaseFunction>,
}

impl MediumInteraction<'_> {
    pub fn valid(&self) -> bool {
        self.phase_function.is_some() && self.medium.is_some()
    }

    pub fn as_interaction(&self) -> Interaction {
        Interaction {
            p: self.p,
            n: self.wi,
        }
    }
}

// #[enum_dispatch]
pub trait MediumT {
    fn transmittance(&self, ray: Ray, t_max: f32) -> Spectrum;
    fn sample(&self, ray: Ray, t_max: f32, u1: f32) -> Option<(MediumInteraction, Spectrum)>;
}

// #[enum_dispatch(MediumT)]
#[derive(Debug, Clone, PartialEq)]
pub enum Medium {
    Homogeneous(HomogeneousMedium),
}

impl MediumT for Medium {
    fn transmittance(&self, ray: Ray, t_max: f32) -> Spectrum {
        match self {
            Medium::Homogeneous(m) => m.transmittance(ray, t_max),
        }
    }

    fn sample(&self, ray: Ray, t_max: f32, u1: f32) -> Option<(MediumInteraction, Spectrum)> {
        let res = match self {
            Medium::Homogeneous(m) => m.sample(ray, t_max, u1),
        };

        res.map(|(mut mi, s)| {
            mi.medium = Some(self);
            (mi, s)
        })
    }
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
