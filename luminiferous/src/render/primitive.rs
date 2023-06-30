use crate::{
    maths::{Bounds3, Normal3, Point2, Point3, Ray},
    shapes::{Shape, ShapeIntersection, ShapeT},
};

#[derive(Clone)]
pub struct Primitive {
    pub shape: Shape,
    pub material_index: usize,
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

impl<'a> Primitive {
    pub fn new(shape: Shape, material_index: usize) -> Self {
        Self {
            shape,
            material_index,
        }
    }

    pub fn intersect(&'a self, ray: Ray) -> Option<Intersection<'a>> {
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
        self.shape.make_bounds()
    }
}

impl<'a> Intersection<'a> {
    pub fn get_surface_interaction(&self, ray: Ray) -> SurfaceInteraction<'a> {
        let shape_interaction = self
            .primitive
            .shape
            .get_surface_interaction(ray, self.shape_intersection);

        SurfaceInteraction {
            primitive: self.primitive,
            t: self.shape_intersection.t,
            p: shape_interaction.p,
            n: shape_interaction.n,
            uv: shape_interaction.uv,
        }
    }
}
