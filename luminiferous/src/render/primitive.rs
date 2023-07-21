use crate::prelude::*;

use crate::shapes::ShapeSample;
use crate::{
    media::{Medium, MediumInterface},
    shapes::{Shape, ShapeIntersection, ShapeT},
};

#[derive(Debug, Clone)]
pub struct Primitive {
    pub shape: Shape,
    pub material_index: usize,
    pub area_light_index: Option<usize>,
    pub world_to_object: Option<Transform3>,
    pub medium_interface: MediumInterface,
}

pub struct Intersection<'a> {
    pub primitive: &'a Primitive,
    pub shape_intersection: ShapeIntersection,
}

#[derive(Clone, Copy, Debug)]
pub struct Interaction {
    pub p: Point3,
    pub n: Normal3,
}

impl Interaction {
    #[inline]
    pub fn spawn_ray(&self, d: Vector3) -> Ray {
        Ray::new(self.p + self.n * 1e-4, d)
    }

    #[inline]
    pub fn spawn_ray_to(&self, p: Point3) -> Ray {
        self.spawn_ray((self.p - p).normalize())
    }
}

#[derive(Debug)]
pub struct SurfaceInteraction<'a> {
    pub primitive: &'a Primitive,
    pub t: f32,
    pub p: Point3,
    pub n: Normal3,
    pub uv: Point2,
    pub dp_du: Vector3,
    pub dp_dv: Vector3,
    // pub shading_frame: Frame3,
}

impl<'a> SurfaceInteraction<'a> {
    pub fn new(
        primitive: &'a Primitive,
        t: f32,
        p: Point3,
        n: Normal3,
        uv: Point2,
        dp_du: Vector3,
        dp_dv: Vector3,
    ) -> Self {
        // let shading_frame = Frame3::new(n);

        SurfaceInteraction {
            primitive,
            t,
            p,
            n,
            uv,
            dp_du,
            dp_dv,
            // shading_frame,
        }
    }

    pub fn target_medium(&self, d: Vector3) -> Option<Medium> {
        if self.n.dot(d) > 0.0 {
            self.primitive.medium_interface.outside.clone()
        } else {
            self.primitive.medium_interface.inside.clone()
        }
    }

    #[inline]
    pub fn as_interaction(&self) -> Interaction {
        Interaction {
            p: self.p,
            n: self.n,
        }
    }
}

impl<'a> Primitive {
    pub fn new(
        mut shape: Shape,
        material_index: usize,
        area_light_index: Option<usize>,
        mut world_to_object: Option<Transform3>,
        medium_interface: MediumInterface,
    ) -> Self {
        if let Some(t) = world_to_object {
            if shape.transform(&t) {
                world_to_object = None;
            }
        }

        Self {
            shape,
            material_index,
            area_light_index,
            world_to_object,
            medium_interface,
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

    pub fn area(&self) -> f32 {
        //TODO: transforms?
        self.shape.area()
    }

    pub fn sample(&self, u: Point2) -> ShapeSample {
        let mut s = self.shape.sample(u);
        if let Some(world_to_object) = self.world_to_object {
            s.p = world_to_object.transform_point(s.p);
            s.n = world_to_object.transform_normal(s.n);
        }

        s
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
            dp_du: shape_interaction.dp_du,
            dp_dv: shape_interaction.dp_dv,
        };

        if let Some(transform) = self.primitive.world_to_object {
            si = transform.transform_surface_interaction(ray, si);
        }

        si
    }
}
