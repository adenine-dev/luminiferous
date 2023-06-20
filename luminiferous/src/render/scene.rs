use crate::{
    aggregates::{Aggregate, AggregateT},
    cameras::Camera,
    lights::{Light, Visibility},
    maths::Ray,
    primitive::SurfaceInteraction,
    stats::STATS,
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
        STATS.shadow_intersection_tests.inc();

        if let Some(intersection) = self.aggregate.intersect(visibility.ray) {
            if intersection.t < visibility.end.distance(visibility.ray.o) {
                return false;
            }
        }
        true
    }

    pub fn intersect(&self, ray: Ray) -> Option<SurfaceInteraction> {
        // TODO: lights
        STATS.regular_intersection_tests.inc();

        self.aggregate.intersect(ray)
    }
}
