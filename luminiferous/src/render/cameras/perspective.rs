use crate::prelude::*;
use crate::{film::Film, media::Medium};

use super::{CameraSample, CameraT};

#[derive(Debug)]
pub struct PerspectiveCamera {
    pub raster_to_camera: Transform3,
    pub lens_radius: f32,
    pub focal_dist: f32,
    film: Film,
    to_world: Transform3,
    medium: Option<Medium>,
}

impl PerspectiveCamera {
    pub fn new_perspective(
        film: Film,
        to_world: Transform3,
        fov_radians: f32,
        lens_radius: f32,
        focal_dist: f32,
        medium: Option<Medium>,
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
            to_world,
            raster_to_camera: Transform3::new(
                camera_to_screen.inverse() * screen_to_raster.inverse(),
            ),
            lens_radius,
            focal_dist,
            medium,
        }
    }
}

impl CameraT for PerspectiveCamera {
    fn sample_ray(&self, sample: CameraSample) -> Ray {
        let p_camera = self
            .raster_to_camera
            .transform_point(sample.p_film.extend(0.0));

        let mut ray = Ray::new(Vector3::splat(0.0), p_camera.normalize());

        if self.lens_radius > 0.0 {
            let p_lens = self.lens_radius * warp::square_to_uniform_disk_concentric(sample.p_lens);

            let ft = self.focal_dist / ray.d.z;
            let focus = ray.at(ft);

            ray.o = Point3::new(p_lens.x, p_lens.y, 0.0);
            ray.d = (ray.o - focus).normalize();
        }

        self.to_world.transform_ray(ray)
    }

    fn get_film(&self) -> &Film {
        &self.film
    }

    fn medium(&self) -> Option<Medium> {
        self.medium.clone()
    }
}
