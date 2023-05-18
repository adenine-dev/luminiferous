use std::path::Path;

use luminiferous::{film::Film, UVector2, Vector2};

fn main() {
    println!("initializing");
    let width = 3840 / 10;
    let height = 2160 / 10;
    let mut film = Film::new(UVector2::new(width, height));

    println!("rendering...");
    for y in 0..height {
        for x in 0..width {
            film.apply_sample(
                Vector2::new(x as f32, y as f32),
                (
                    x as f32 / (width - 1) as f32,
                    y as f32 / (height - 1) as f32,
                    1.0,
                ),
            );
        }
    }

    println!("writing output...");
    film.develop(Path::new("output"));

    println!("successfully wrote output :>");
}
