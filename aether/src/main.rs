use std::path::Path;

use luminiferous::{
    film::Film,
    maths::{Color, Extent2, UVector2, Vector2, Vector3},
    primitive::Primitive,
    rfilters::*,
    sensors::{ProjectiveCamera, Sensor, SensorT},
    shapes::{Shape, ShapeT, Sphere},
};

fn main() {
    println!("initializing");
    // let width = 3840;
    // let height = 2160;
    // let width = 512;
    // let height = 384;
    let width = 400;
    let height = 225;

    let mut film = Film::new(
        UVector2::new(width, height),
        TentFilter::new(Vector2::splat(1.0)),
    );

    // let camera = Sensor::ProjectiveCamera(ProjectiveCamera::new_orthographic(Extent2::new(
    //     width as f32,
    //     height as f32,
    // )));
    let camera = Sensor::ProjectiveCamera(ProjectiveCamera::new_perspective(
        Extent2::new(width as f32, height as f32),
        core::f32::consts::FRAC_PI_2,
    ));

    let sphere = Primitive::new(Shape::Sphere(Sphere::new(
        Vector3::new(0.0, 0.0, -1.0),
        0.5,
    )));

    println!("rendering...");
    for y in 0..height {
        for x in 0..width {
            for _ in 0..50 {
                let x = x as f32 + (rand::random::<f32>() - 0.5);
                let y = y as f32 + (rand::random::<f32>() - 0.5);
                let p = Vector2::new(x, y);
                let ray = camera.sample_ray(p);

                // let l = 0.5 * (1.0 + ((x.powi(2) + y.powi(2)) / 100.0).sin());
                // let mut l_x = ray.d.x;
                // let mut l_y = ray.d.y;
                // let mut l_z = ray.d.z;

                let mut l_x = 1.0;
                let mut l_y = 1.0;
                let mut l_z = 1.0;

                if let Some(intersection) = sphere.intersect(ray) {
                    let interaction = intersection.get_surface_interaction(ray);

                    let n = (interaction.n + Vector3::splat(1.0)) * 0.5;
                    l_x = n.x;
                    l_y = n.y;
                    l_z = n.z;
                }

                film.apply_sample(p, Color::new(l_x, l_y, l_z));
            }
        }
    }

    println!("writing output...");
    film.develop(Path::new("output"));

    println!("successfully wrote output :>");
}
