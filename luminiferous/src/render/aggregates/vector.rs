use std::mem::size_of;

use crate::prelude::*;
use crate::primitive::{Intersection, Primitive};

use super::AggregateT;

pub struct Vector {
    pub primitives: Vec<Primitive>,
    bounds: Bounds3,
}

impl Vector {
    pub fn new(primitives: Vec<Primitive>) -> Self {
        STATS
            .primitive_memory
            .add((primitives.len() * size_of::<Primitive>() + size_of::<Self>()) as u64);
        STATS.aggregate_memory.add(size_of::<Self>() as u64);

        let bounds = primitives
            .iter()
            .fold(primitives[0].make_bounds(), |b, p| b.union(p.make_bounds()));

        Vector { primitives, bounds }
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

    fn bounds(&self) -> Bounds3 {
        self.bounds
    }
}
