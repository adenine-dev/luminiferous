use std::path::Path;

use luminiferous::{film::Film, rfilters::*, Color, UVector2, Vector2};

fn main() {
    println!("initializing");
    // let width = 3840;
    // let height = 2160;
    let width = 512;
    let height = 384;
    let mut film = Film::new(
        UVector2::new(width, height),
        BoxFilter::new(Vector2::splat(0.5)),
    );

    println!("rendering...");
    for y in 0..height {
        for x in 0..width {
            for _ in 0..100 {
                let x = x as f32 + (rand::random::<f32>() - 0.5);
                let y = y as f32 + (rand::random::<f32>() - 0.5);

                let l = 0.5 * (1.0 + ((x.powi(2) + y.powi(2)) / 100.0).sin());
                film.apply_sample(Vector2::new(x as f32, y as f32), Color::new(l, l, l));
            }
        }
    }

    println!("writing output...");
    film.develop(Path::new("output"));

    println!("successfully wrote output :>");
}
