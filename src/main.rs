#![feature(or_patterns)]
#![feature(box_syntax)]

extern crate nalgebra_glm as glm;
#[macro_use] extern crate lazy_static;

use std::path::PathBuf;

use structopt::StructOpt;

mod shaders;
mod rendering;

use shaders::*;
use rendering::{onscreen::new_app, offscreen::new_app_offscreen};

#[derive(StructOpt)]
struct Opt { 
    source: PathBuf,

    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt)]
enum Command {
    Render {
        #[structopt(long)]
        width: u32,
        #[structopt(long)]
        height: u32,
        #[structopt(long, default_value = "30")]
        fps: f32,
    },

    Interactive {
        #[structopt(long, short)]
        camera: bool,
    }
}

fn main() {
    let opt = Opt::from_args();

    let mut loader = SceneDescLoader::new(opt.source);

    match opt.command {
        Command::Render { width, height, fps } => render(loader, [width, height], fps),
        Command::Interactive { camera } => {
            loader.switch_camera(camera);
            let (app, el) = new_app([800, 600], loader);
            app.run(el);
        }
    }
}

fn render(mut loader: SceneDescLoader, size: [u32; 2], fps: f32) {
    let scene = loader.load().unwrap();

    let duration = scene
        .camera
        .as_ref()
        .map(|cam| cam.duration().ceil())
        .unwrap_or(10.0) as usize;

    let (mut app, _) = new_app_offscreen(size, scene);

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    for i in 0..(fps * duration as f32) as u32 {
        use std::io::Write;

        app.draw(i as f32 / fps);
        stdout.write_all(&app.to_image().into_raw()).unwrap();
    }
}
