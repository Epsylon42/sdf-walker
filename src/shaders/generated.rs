use std::path::PathBuf;

mod codegen;
mod desc;
mod parser;
mod typed;

pub use desc::{SceneDesc, loader::SceneDescLoader};

pub struct GeneratedScene {
    pub source: PathBuf,
}

impl GeneratedScene {
    pub fn get_vertex() -> String {
        include_str!("../glsl/vertex.glsl").to_string()
    }

    pub fn compile_fragment(main: &str) -> String {
        let header = include_str!("../glsl/header.glsl");
        let library = include_str!("../glsl/library.glsl");
        let footer = include_str!("../glsl/footer.glsl");

        format!("{}{}{}{}", header, library, main, footer)
    }
}
