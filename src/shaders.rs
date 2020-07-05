use std::fs;
use std::path::{Path, PathBuf};

use luminance::shader::program::{Program, UniformInterface};

#[cfg(feature = "generated")]
mod generated;

#[cfg(feature = "generated")]
pub use generated::GeneratedScene;

pub trait ShaderProvider {
    fn get_sources(&self) -> [String; 2];

    fn get<S, Out, Uni>(&self) -> Program<S, Out, Uni>
    where
        S: luminance::vertex::Semantics,
        Uni: UniformInterface,
    {
        let [vertex, fragment] = self.get_sources();

        Program::from_strings(None, &vertex, None, &fragment)
            .map_err(|e| {
                println!("{}", e);
            })
            .unwrap()
            .ignore_warnings()
    }
}

#[derive(Debug, Clone)]
pub struct FileLoader {
    pub vertex: PathBuf,
    pub fragment: PathBuf,
}

impl ShaderProvider for FileLoader {
    fn get_sources(&self) -> [String; 2] {
        let vertex = fs::read_to_string(Path::new("src/glsl").join(&self.vertex)).unwrap();
        let fragment = fs::read_to_string(Path::new("src/glsl").join(&self.fragment)).unwrap();

        [vertex, include_library(&fragment)]
    }
}

#[cfg(feature = "embedded")]
pub struct EmbeddedLoader;

#[cfg(feature = "embedded")]
impl ShaderProvider for EmbeddedLoader {
    fn get_sources(&self) -> [String; 2] {
        let vertex = include_str!("glsl/vertex.glsl").to_string();
        let fragment = include_str!("glsl/fragment.glsl");

        [vertex, include_library(fragment)]
    }
}

fn include_library(code: &str) -> String {
    let library = include_str!("glsl/library.glsl");

    code.replace("{{library}}", library)
}
