use std::path::Path;

use crate::lights::AreaLight;
use crate::prelude::*;
use crate::{
    aggregates::{Aggregate, AggregateT, Bvh},
    cameras::Camera,
    lights::{Light, Visibility},
    loaders::{Loader, SceneCreationParams},
    materials::Material,
    media::MediumInterface,
    primitive::{Primitive, SurfaceInteraction},
    shapes::Shape,
};

pub struct Scene {
    pub lights: Vec<Light>,
    pub aggregate: Aggregate,
    pub camera: Camera,
    pub materials: Vec<Material>,
}

impl Scene {
    pub fn new(
        lights: Vec<Light>,
        aggregate: Aggregate,
        camera: Camera,
        materials: Vec<Material>,
    ) -> Self {
        Self {
            lights,
            aggregate,
            camera,
            materials,
        }
    }

    pub fn unoccluded(&self, visibility: Visibility) -> bool {
        STATS.shadow_intersection_tests.inc();

        if let Some(intersection) = self.aggregate.intersect_p(visibility.ray).0 {
            if intersection.shape_intersection.t < visibility.end.distance(visibility.ray.o)
                && intersection.shape_intersection.t > 0.0
            {
                return false;
            }
        }
        true
    }

    pub fn intersect(&self, ray: Ray) -> (Option<SurfaceInteraction>, usize) {
        // TODO: lights
        STATS.regular_intersection_tests.inc();

        self.aggregate.intersect(ray)
    }

    pub fn bounds(&self) -> Bounds3 {
        self.aggregate.bounds()
    }
}

#[derive(Default)]
pub struct SceneBuilder {
    lights: Vec<Light>,
    primitives: Vec<Primitive>,
    camera: Option<Camera>,
    materials: Vec<Material>,
}

impl SceneBuilder {
    pub fn new() -> Self {
        Self {
            lights: vec![],
            primitives: vec![],
            camera: None,
            materials: vec![],
        }
    }

    pub fn light(&mut self, light: Light) -> &mut Self {
        self.lights.push(light);

        self
    }

    pub fn camera(&mut self, camera: Camera) -> &mut Self {
        if self.camera.is_some() {
            warnln!("replacing scene camera.");
        }

        self.camera = Some(camera);

        self
    }

    pub fn primitive(
        &mut self,
        shape: Shape,
        material: Material,
        world_to_object: Option<Transform3>,
        medium_interface: MediumInterface,
    ) -> &mut Self {
        // TODO: material reuse/real material ids
        self.materials.push(material);
        self.primitives.push(Primitive::new(
            shape,
            self.materials.len() - 1,
            None,
            world_to_object,
            medium_interface,
        ));

        self
    }

    pub fn primitives(
        &mut self,
        shapes: Vec<Shape>,
        material: Material,
        world_to_object: Option<Transform3>,
        medium_interface: MediumInterface,
    ) -> &mut Self {
        if shapes.is_empty() {
            return self;
        }

        self.materials.push(material);
        self.primitives.extend(shapes.into_iter().map(|s| {
            Primitive::new(
                s,
                self.materials.len() - 1,
                None,
                world_to_object,
                medium_interface.clone(),
            )
        }));

        self
    }

    pub fn area_light(&mut self, mut light: AreaLight) -> &mut Self {
        light.primitive.area_light_index = Some(self.lights.len());
        self.primitives.push(light.primitive.clone());

        self.lights.push(Light::Area(light));
        self
    }

    pub fn area_lights(&mut self, lights: Vec<AreaLight>) -> &mut Self {
        let old_len = self.lights.len();
        self.lights
            .extend(lights.into_iter().enumerate().map(|(i, mut light)| {
                light.primitive.area_light_index = Some(old_len + i);
                self.primitives.push(light.primitive.clone());
                Light::Area(light)
            }));
        self
    }

    pub fn load_with<L: Loader>(&mut self, path: &Path, params: SceneCreationParams) -> &mut Self {
        L::load_from_file(self, path, params);
        self
    }

    //FIXME: turn this into a result
    pub fn build(self) -> Option<Scene> {
        let aggregate = Aggregate::Bvh(Bvh::new(self.primitives));
        if self.camera.is_none() {
            warnln!("attempting to build scene without camera.");
        }
        Some(Scene::new(
            self.lights,
            aggregate,
            self.camera?,
            self.materials,
        ))
    }
}
