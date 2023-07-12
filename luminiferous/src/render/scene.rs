use std::{collections::HashMap, path::Path};

use crate::{
    aggregates::{Aggregate, AggregateT, Bvh},
    bsdfs::BsdfFlags,
    cameras::Camera,
    lights::{Light, LightT, Visibility},
    loaders::{Loader, SceneCreationParams},
    materials::{Material, MaterialT},
    maths::{Bounds3, Ray, Transform3},
    media::MediumInterface,
    primitive::{Primitive, SurfaceInteraction},
    shapes::Shape,
    stats::STATS,
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

    pub fn test_visibility(&self, visibility: Visibility) -> bool {
        STATS.shadow_intersection_tests.inc();

        if let Some(intersection) = self.aggregate.intersect_p(visibility.ray).0 {
            if intersection.shape_intersection.t < visibility.end.distance(visibility.ray.o)
            // && !self.materials[intersection.primitive.material_index]
            //     .bsdf_flags()
            //     .contains(BsdfFlags::Null)
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
    named_materials: HashMap<String, usize>,
}

impl SceneBuilder {
    pub fn new() -> Self {
        Self {
            lights: vec![],
            primitives: vec![],
            camera: None,
            materials: vec![],
            named_materials: HashMap::new(),
        }
    }

    pub fn light(&mut self, light: Light) -> &mut Self {
        self.lights.push(light);

        self
    }

    pub fn camera(&mut self, camera: Camera) -> &mut Self {
        if self.camera.is_some() {
            println!("[WARN]: replacing scene camera.");
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
        self.materials.push(material);
        self.primitives.extend(shapes.into_iter().map(|s| {
            Primitive::new(
                s,
                self.materials.len() - 1,
                world_to_object,
                medium_interface.clone(),
            )
        }));

        self
    }

    pub fn material(&mut self, key: String, material: Material) -> &mut Self {
        //TODO: material reuse
        self.materials.push(material);
        self.named_materials.insert(key, self.materials.len() - 1);

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
            println!("[WARN]: attempting to build scene without camera.");
        }
        Some(Scene::new(
            self.lights,
            aggregate,
            self.camera?,
            self.materials,
        ))
    }
}
