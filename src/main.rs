#![feature(or_patterns)]
#![feature(box_syntax)]

extern crate nalgebra_glm as glm;

use std::time::Instant;

use luminance::{
    context::GraphicsContext,
    framebuffer::Framebuffer,
    pipeline::{PipelineGate, PipelineState, PipelineError},
    render_state::RenderState,
    shader::{Program, Uniform},
    tess::{Mode, Tess, TessBuilder, View},
    texture::Dim2,
    pixel,
};
use luminance_derive::{Semantics, UniformInterface, Vertex};
use luminance_glutin::GlutinOffscreen;
use glutin::event_loop::EventLoop;

mod shaders;

use shaders::*;

fn main() {
    #[cfg(feature = "file")]
    let provider = FileLoader {
        vertex: "vertex.glsl".into(),
        fragment: "fragment.glsl".into(),
    };

    #[cfg(feature = "embedded")]
    let provider = EmbeddedLoader;

    #[cfg(feature = "generated")]
    let provider = GeneratedScene {
        source: std::env::args().nth(1)
            .unwrap_or_else(|| String::from("test.scene"))
            .into(),
    };

    let mut app = App::new([800, 600], provider);

    let fps = 24;

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    for i in 0..(fps*10) {
        use std::io::Write;

        app.draw(i as f32 / fps as f32);
        stdout.write(&app.to_image().into_raw()).unwrap();
    }
}

#[derive(Debug, Clone, Copy, Semantics)]
pub enum VertexSemantics {
    #[sem(name = "idx", repr = "i32", wrapper = "VertexIndex")]
    Index,
}

#[derive(Clone, Copy, Vertex)]
#[vertex(sem = "VertexSemantics")]
pub struct Vertex {
    pub idx: VertexIndex,
}

#[derive(UniformInterface)]
pub struct Uniforms {
    #[uniform(unbound)]
    aspect: Uniform<f32>,
    #[uniform(unbound)]
    fov: Uniform<f32>,
    #[uniform(unbound)]
    cam: Uniform<[[f32; 4]; 4]>,
    #[uniform(unbound)]
    cam_pos: Uniform<[f32; 3]>,
    #[uniform(unbound)]
    light: Uniform<[f32; 3]>,
    #[uniform(unbound)]
    time: Uniform<f32>,
}

const SCREEN: [Vertex; 6] = [
    Vertex {
        idx: VertexIndex::new(0),
    },
    Vertex {
        idx: VertexIndex::new(1),
    },
    Vertex {
        idx: VertexIndex::new(2),
    },
    Vertex {
        idx: VertexIndex::new(2),
    },
    Vertex {
        idx: VertexIndex::new(3),
    },
    Vertex {
        idx: VertexIndex::new(0),
    },
];

type Backend = <GlutinOffscreen as GraphicsContext>::Backend;

struct App {
    el: EventLoop<()>,

    surface: GlutinOffscreen,
    bb: Framebuffer<Backend, Dim2, pixel::NormRGBA8UI, ()>,

    triangle: Tess<Backend, Vertex>,
    program: Program<Backend, VertexSemantics, (), Uniforms>,

    size: [u32; 2],

    pos: glm::Vec3,
    rot: glm::Vec2,
    camera_up: glm::Vec3,
    camera_fw: glm::Vec3,
}

impl App {
    pub fn new(size: [u32; 2], shaders: impl ShaderProvider) -> Self {
        let el = EventLoop::new();
        let ctx_builder = glutin::ContextBuilder::new();

        let mut surface = GlutinOffscreen::new_gl33_from_builder(&el, ctx_builder).unwrap();

        let bb = surface.new_framebuffer(size, 1, <_>::default()).unwrap();

        let triangle = TessBuilder::new(&mut surface)
            .set_vertices(SCREEN)
            .set_mode(Mode::Triangle)
            .build()
            .unwrap();

        App {
            el,
            program: shaders.get(&mut surface),

            surface,
            bb,
            triangle,

            size,

            pos: glm::Vec3::zeros(),
            rot: glm::vec2(0.0, 0.0),
            camera_up: glm::Vec3::y(),
            camera_fw: glm::Vec3::z(),
        }
    }

    fn camera_rotation(&self) -> glm::Quat {
        let side_axis = glm::cross(&self.camera_up, &self.camera_fw);
        let rot = glm::quat_rotate(&glm::quat_identity(), self.rot.y, &side_axis);
        glm::quat_rotate(&rot, self.rot.x, &self.camera_up)
    }

    fn draw(&mut self, time: f32) {
        let camera = self.camera_rotation();

        let Self {
            surface,
            program,
            bb,
            triangle,
            size,
            pos,
            ..
        } = self;

        surface.new_pipeline_gate().pipeline::<PipelineError, _, _, _, _>(
            bb,
            &PipelineState::default().set_clear_color([0.0, 0.0, 0.0, 1.0]),
            |_, mut shader_gate| {
                shader_gate.shade(program, |mut iface, uni, mut render_gate| {
                    let fov = glm::pi::<f32>() / 2.0;

                    iface.set(&uni.aspect, size[0] as f32 / size[1] as f32);
                    iface.set(&uni.fov, fov);
                    iface.set(&uni.cam, glm::quat_to_mat4(&camera).into());
                    iface.set(&uni.cam_pos, [pos.x, pos.y, pos.z]);
                    iface.set(&uni.light, [1.0, -1.0, 1.0]);
                    iface.set(&uni.time, time);

                    render_gate.render(&RenderState::default(), |mut tess_gate| {
                        tess_gate.render(triangle.view(..).unwrap())
                    })
                })
            },
        );
    }

    fn to_image(&mut self) -> image::RgbImage {
        let color = self.bb.color_slot().get_raw_texels().unwrap();
        let image = image::RgbaImage::from_raw(self.size[0], self.size[1], color).unwrap();
        image::DynamicImage::ImageRgba8(image).to_rgb()
    }
}
