use crate::{
    aggregates::{Aggregate, AggregateT, Bvh},
    cameras::Camera,
    lights::{Light, Visibility},
    materials::Material,
    maths::{Ray, Transform3},
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
            if intersection.shape_intersection.t < visibility.end.distance(visibility.ray.o) {
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
}

pub struct SceneBuilder {
    lights: Vec<Light>,
    primitives: Vec<Primitive>,
    camera: Option<Camera>,
    materials: Vec<Material>,
}

impl SceneBuilder {
    #[allow(clippy::new_without_default)] // something will probably eventually happen here idk
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
        self.camera = Some(camera);

        self
    }

    pub fn primitive(
        &mut self,
        shape: Shape,
        material: Material,
        world_to_object: Option<Transform3>,
    ) -> &mut Self {
        // TODO: material reuse/real material ids
        self.materials.push(material);
        self.primitives.push(Primitive::new(
            shape,
            self.materials.len() - 1,
            world_to_object,
        ));

        self
    }

    pub fn primitives(
        &mut self,
        shapes: Vec<Shape>,
        material: Material,
        world_to_object: Option<Transform3>,
    ) -> &mut Self {
        self.materials.push(material);
        self.primitives.extend(
            shapes
                .into_iter()
                .map(|s| Primitive::new(s, self.materials.len() - 1, world_to_object)),
        );

        self
    }

    //FIXME: turn this into a result
    pub fn build(self) -> Option<Scene> {
        let aggregate = Aggregate::Bvh(Bvh::new(self.primitives));
        Some(Scene::new(
            self.lights,
            aggregate,
            self.camera?,
            self.materials,
        ))
    }
}
