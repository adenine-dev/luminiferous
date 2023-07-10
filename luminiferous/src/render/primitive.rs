use crate::{
    maths::{face_forward, Bounds3, Normal3, Point2, Point3, Ray, Transform3, Vector3},
    shapes::{Shape, ShapeIntersection, ShapeT},
};

#[derive(Debug, Clone)]
pub struct Primitive {
    pub shape: Shape,
    pub material_index: usize,
    pub world_to_object: Option<Transform3>,
}

pub struct Intersection<'a> {
    pub primitive: &'a Primitive,
    pub shape_intersection: ShapeIntersection,
}

pub struct SurfaceInteraction<'a> {
    pub primitive: &'a Primitive,
    pub t: f32,
    pub p: Point3,
    pub n: Normal3,
    pub uv: Point2,
}

impl<'a> SurfaceInteraction<'a> {
    pub fn spawn_ray(&self, d: Vector3) -> Ray {
        Ray::new(self.p + face_forward(self.n, d) * 1e-6, d)
    }
}

impl<'a> Primitive {
    pub fn new(
        mut shape: Shape,
        material_index: usize,
        mut world_to_object: Option<Transform3>,
    ) -> Self {
        if let Some(t) = world_to_object {
            if shape.transform(&t) {
                world_to_object = None;
            }
        }

        Self {
            shape,
            material_index,
            world_to_object,
        }
    }

    pub fn intersect(&'a self, mut ray: Ray) -> Option<Intersection<'a>> {
        if let Some(world_to_object) = self.world_to_object {
            ray = world_to_object.transform_ray_inv(ray);
        }

        let shape_intersection = self.shape.intersect(ray);
        if shape_intersection.t > 0.0 {
            Some(Intersection {
                primitive: self,
                shape_intersection,
            })
        } else {
            None
        }
    }

    pub fn make_bounds(&self) -> Bounds3 {
        let mut bounds = self.shape.make_bounds();
        if let Some(transform) = self.world_to_object {
            bounds = transform.transform_bounds(bounds);
        }
        bounds
    }
}

impl<'a> Intersection<'a> {
    pub fn get_surface_interaction(&self, ray: Ray) -> SurfaceInteraction<'a> {
        let r = if let Some(world_to_object) = self.primitive.world_to_object {
            world_to_object.transform_ray_inv(ray)
        } else {
            ray
        };

        let shape_interaction = self
            .primitive
            .shape
            .get_surface_interaction(r, self.shape_intersection);

        let mut si = SurfaceInteraction {
            primitive: self.primitive,
            t: self.shape_intersection.t,
            p: shape_interaction.p,
            n: shape_interaction.n,
            uv: shape_interaction.uv,
        };

        if let Some(transform) = self.primitive.world_to_object {
            si = transform.transform_surface_interaction(ray, si);
        }

        si
    }
}
