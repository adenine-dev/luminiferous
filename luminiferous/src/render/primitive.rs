use crate::{
    materials::Material,
    maths::{Point3, Ray, Vector3},
    shapes::{Shape, ShapeIntersection, ShapeT},
};

pub struct Primitive {
    pub shape: Shape,
    pub material: Material,
}

pub struct Intersection<'a> {
    pub primitive: &'a Primitive,
    pub shape_intersection: ShapeIntersection,
}

pub struct SurfaceInteraction<'a> {
    pub primitive: &'a Primitive,
    pub t: f32,
    pub p: Point3,
    pub n: Vector3,
}

impl<'a> Primitive {
    pub fn new(shape: Shape, material: Material) -> Self {
        Self { shape, material }
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
        }
    }
}
