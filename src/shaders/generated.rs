use super::ShaderProvider;

use std::path::PathBuf;

mod codegen;
mod desc;
mod parser;
mod typed;

pub struct GeneratedScene {
    pub source: PathBuf,
}

impl ShaderProvider for GeneratedScene {
    fn get_sources(&self) -> [String; 2] {
        let vertex = include_str!("../glsl/vertex.glsl").to_string();

        let header = include_str!("../glsl/header.glsl");
        let library = include_str!("../glsl/library.glsl");
        let footer = include_str!("../glsl/footer.glsl");

        let scene_source = std::fs::read(&self.source).unwrap();
        let desc = parser::scene(&scene_source).unwrap().1;

        let fragment = format!("{}{}{}{}", header, library, dbg!(desc).to_string(), footer);

        println!("{}", fragment);

        [vertex, fragment]
    }
}
