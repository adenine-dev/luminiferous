use crate::{
    film::Film,
    maths::{warp, Extent2, Matrix4, Point2, Point3, Ray, Transform3, Vector3},
};

use super::{CameraSample, CameraT};

#[derive(Debug)]
pub enum Projection {
    Orthographic,
    Perspective,
}

#[derive(Debug)]
pub struct ProjectiveCamera {
    pub raster_to_camera: Transform3,
    pub projection: Projection,
    pub lens_radius: f32,
    pub focal_dist: f32,
    film: Film,
}

impl ProjectiveCamera {
    pub fn new_orthographic(film: Film, lens_radius: f32, focal_dist: f32) -> Self {
        let near = 0.001;
        let far = 1.0;
        let screen_window_min = Point2::new(-1.0, -1.0);
        let screen_window_max = Point2::new(1.0, 1.0);

        let extent = film.get_extent().as_vec2();

        let aspect_ratio = extent.x / extent.y;

        let camera_to_screen = Matrix4::orthographic_rh(
            aspect_ratio * -1.0,
            aspect_ratio * 1.0,
            -1.0,
            1.0,
            near,
            far,
        );

        let screen_to_raster = Matrix4::from_scale(Vector3::new(extent.x, extent.y, 1.0))
            * Matrix4::from_scale(Vector3::new(
                1.0 / (screen_window_max.x - screen_window_min.x),
                1.0 / (screen_window_min.y - screen_window_max.y),
                1.0,
            ))
            * Matrix4::from_translation(Vector3::new(
                -screen_window_min.x,
                -screen_window_max.y,
                0.0,
            ));

        Self {
            film,
            raster_to_camera: Transform3::new(
                camera_to_screen.inverse() * screen_to_raster.inverse(),
            ),
            projection: Projection::Orthographic,
            lens_radius,
            focal_dist,
        }
    }

    pub fn new_perspective(
        film: Film,
        fov_radians: f32,
        lens_radius: f32,
        focal_dist: f32,
    ) -> Self {
        let near = 0.001;
        let far = 1.0;

        let screen_window_min = Point2::new(-1.0, -1.0);
        let screen_window_max = Point2::new(1.0, 1.0);

        let extent = film.get_extent().as_vec2();

        let aspect_ratio = extent.x / extent.y;

        let camera_to_screen = Matrix4::perspective_rh(fov_radians, aspect_ratio, near, far);

        let screen_to_raster = Matrix4::from_scale(Vector3::new(extent.x, extent.y, 1.0))
            * Matrix4::from_scale(Vector3::new(
                1.0 / (screen_window_max.x - screen_window_min.x),
                1.0 / (screen_window_min.y - screen_window_max.y),
                1.0,
            ))
            * Matrix4::from_translation(Vector3::new(
                -screen_window_min.x,
                -screen_window_max.y,
                0.0,
            ));

        Self {
            film,
            raster_to_camera: Transform3::new(
                camera_to_screen.inverse() * screen_to_raster.inverse(),
            ),
            projection: Projection::Perspective,
            lens_radius,
            focal_dist,
        }
    }
}

impl CameraT for ProjectiveCamera {
    fn sample_ray(&self, sample: CameraSample) -> Ray {
        let p_camera = self
            .raster_to_camera
            .transform_point(sample.p_film.extend(0.0));

        let mut ray = match self.projection {
            Projection::Orthographic => Ray::new(p_camera, Vector3::NEG_Z),
            Projection::Perspective => Ray::new(Vector3::splat(0.0), p_camera.normalize()),
        };

        if self.lens_radius > 0.0 {
            let p_lens = self.lens_radius * warp::square_to_uniform_disk_concentric(sample.p_lens);

            let ft = self.focal_dist / ray.d.z;
            let focus = ray.at(ft);

            ray.o = Point3::new(p_lens.x, p_lens.y, 0.0);
            ray.d = (ray.o - focus).normalize();
        }

        ray
    }

    fn get_film(&self) -> &Film {
        &self.film
    }
}
