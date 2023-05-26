use crate::maths::{Extent2, Matrix4, Point2, Ray, Transform, Vector2, Vector3};

use super::SensorT;

#[derive(Debug)]
pub enum Projection {
    Orthographic,
    Perspective,
}

#[derive(Debug)]
pub struct ProjectiveCamera {
    pub raster_to_camera: Transform,
    pub extent: Extent2,
    pub projection: Projection,
}

impl ProjectiveCamera {
    pub fn new_orthographic(extent: Extent2) -> Self {
        let near = 0.001;
        let far = 1.0;
        let screen_window_min = Point2::new(-1.0, -1.0);
        let screen_window_max = Point2::new(1.0, 1.0);

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
            extent,
            raster_to_camera: Transform::new(
                camera_to_screen.inverse() * screen_to_raster.inverse(),
            ),
            projection: Projection::Orthographic,
        }
    }

    pub fn new_perspective(extent: Extent2, fov_radians: f32) -> Self {
        let near = 0.001;
        let far = 1.0;

        let screen_window_min = Point2::new(-1.0, -1.0);
        let screen_window_max = Point2::new(1.0, 1.0);

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
            extent,
            raster_to_camera: Transform::new(
                camera_to_screen.inverse() * screen_to_raster.inverse(),
            ),
            projection: Projection::Perspective,
        }
    }
}

impl SensorT for ProjectiveCamera {
    fn sample_ray(&self, p: Vector2) -> Ray {
        let p_camera = self.raster_to_camera.transform_point(p.extend(0.0));

        match self.projection {
            Projection::Orthographic => Ray::new(p_camera, Vector3::NEG_Z),
            Projection::Perspective => Ray::new(Vector3::splat(0.0), p_camera.normalize()),
        }
    }
}
