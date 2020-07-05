use super::ShaderProvider;

use std::path::PathBuf;

mod codegen;
mod desc;

pub struct GeneratedScene {
    pub source: PathBuf,
}

impl ShaderProvider for GeneratedScene {
    fn get_sources(&self) -> [String; 2] {
        let vertex = include_str!("../glsl/vertex.glsl").to_string();

        let header = include_str!("../glsl/header.glsl");
        let library = include_str!("../glsl/library.glsl");
        let footer = include_str!("../glsl/footer.glsl");

        let scene_source = std::fs::read_to_string(&self.source).unwrap();
        let scene: desc::SceneDesc = ron::de::from_str(&scene_source).unwrap();

        let fragment = format!("{}{}{}{}", header, library, scene.to_string(), footer);

        println!("{}", fragment);

        [vertex, fragment]
    }
}
