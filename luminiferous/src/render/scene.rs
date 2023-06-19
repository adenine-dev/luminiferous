use crate::{
    aggregates::{Aggregate, AggregateT},
    cameras::Camera,
    lights::{Light, Visibility},
};

pub struct Scene {
    pub lights: Vec<Light>,
    pub aggregate: Aggregate,
    pub camera: Camera,
}

impl Scene {
    pub fn new(lights: Vec<Light>, aggregate: Aggregate, camera: Camera) -> Self {
        Self {
            lights,
            aggregate,
            camera,
        }
    }

    pub fn test_visibility(&self, visibility: Visibility) -> bool {
        // TODO: lights
        if let Some(intersection) = self.aggregate.intersect(visibility.ray) {
            if intersection.t < visibility.end.distance(visibility.ray.o) {
                return false;
            }
        }
        true
    }
}
