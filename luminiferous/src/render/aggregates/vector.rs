use std::mem::size_of;

use crate::{
    maths::Ray,
    primitive::{Intersection, Primitive},
    stats::STATS,
};

use super::AggregateT;

pub struct Vector {
    pub primitives: Vec<Primitive>,
}

impl Vector {
    pub fn new(primitives: Vec<Primitive>) -> Self {
        STATS
            .primitive_memory
            .add((primitives.len() * size_of::<Primitive>() + size_of::<Self>()) as u64);
        Vector { primitives }
    }
}

impl AggregateT for Vector {
    fn intersect_p(&self, ray: Ray) -> (Option<Intersection>, usize) {
        let mut t_max = f32::INFINITY;
        let mut intersection = None;
        let mut n = 0;

        for p in self.primitives.iter() {
            n += 1;
            if let Some(i) = p.intersect(ray) {
                if i.shape_intersection.t < t_max {
                    t_max = i.shape_intersection.t;
                    intersection = Some(i);
                }
            }
        }

        (intersection, n)
    }
}
