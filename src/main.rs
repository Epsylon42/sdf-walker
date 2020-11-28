#![feature(or_patterns)]
#![feature(box_syntax)]

extern crate nalgebra_glm as glm;
#[macro_use] extern crate lazy_static;

mod shaders;
mod rendering;

use shaders::*;
use rendering::{onscreen::new_app, offscreen::new_app_offscreen};

fn main() {
    let source_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("test.scene"));

    let mut loader = SceneDescLoader::new(source_path);

    for arg in std::env::args().skip(2).chain(std::iter::once("".into())) {
        match arg.as_str() {
            "nocamera" => {
                loader.switch_camera(false);
            }

            "interactive" => {
                let (app, el) = new_app([800, 600], loader);
                app.run(el);
            }

            _ => {
                let scene = loader.load().unwrap();

                let duration = scene
                    .camera
                    .as_ref()
                    .map(|cam| cam.duration().ceil())
                    .unwrap_or(10.0) as usize;

                let (mut app, _) = new_app_offscreen([800, 600], scene);

                let fps = 24;

                let stdout = std::io::stdout();
                let mut stdout = stdout.lock();

                for i in 0..(fps * duration) {
                    use std::io::Write;

                    app.draw(i as f32 / fps as f32);
                    stdout.write_all(&app.to_image().into_raw()).unwrap();
                }
                return;
            }
        }
    }
}


