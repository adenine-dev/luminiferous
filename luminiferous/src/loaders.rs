use std::{error::Error, path::Path};

mod assimp_loader;
pub use assimp_loader::*;

mod pbrt_loader;
pub use pbrt_loader::*;

use crate::{
    maths::UExtent2,
    scene::{Scene, SceneBuilder},
};

pub type SceneResult = Result<Scene, Box<dyn Error>>;

pub struct SceneCreationParams {
    pub extent: UExtent2,
}

pub trait Loader {
    fn load_from_file(sb: &mut SceneBuilder, path: &Path, params: SceneCreationParams);
}
