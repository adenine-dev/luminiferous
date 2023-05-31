use crate::{
    maths::Ray,
    primitive::{Primitive, SurfaceInteraction},
};

use super::AggregateT;

pub struct Vector {
    pub primitives: Vec<Primitive>,
}

impl Vector {
    pub fn new(primitives: Vec<Primitive>) -> Self {
        Vector { primitives }
    }
}

impl AggregateT for Vector {
    fn intersect(&self, ray: Ray) -> Option<SurfaceInteraction> {
        let mut t_max = f32::INFINITY;
        let mut intersection = None;

        for p in self.primitives.iter() {
            if let Some(i) = p.intersect(ray) {
                if i.shape_intersection.t < t_max {
                    t_max = i.shape_intersection.t;
                    intersection = Some(i);
                }
            }
        }

        intersection.map(|i| i.get_surface_interaction(ray))
    }
}
