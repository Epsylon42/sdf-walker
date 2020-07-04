#![feature(or_patterns)]

extern crate nalgebra_glm as glm;

use luminance::{
    context::GraphicsContext,
    framebuffer::Framebuffer,
    pipeline::PipelineState,
    render_state::RenderState,
    shader::program::{Program, Uniform},
    tess::{Mode, Tess, TessBuilder, TessSliceIndex},
    texture::Dim2,
    linear::M44,
};
use luminance_derive::{Semantics, Vertex, UniformInterface};
use luminance_glfw::{Action, GlfwSurface, Key, Surface as _, WindowDim, WindowEvent, WindowOpt, MouseButton};

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let mut app = App::new(
        (800, 600),
        &ShaderSrc {
            vertex: "vertex.glsl".into(),
            fragment: "fragment.glsl".into(),
        },
    );
    app.run();
}

#[derive(Debug, Clone, Copy, Semantics)]
pub enum VertexSemantics {
    #[sem(name = "idx", repr = "i32", wrapper = "VertexIndex")]
    Index,
}

#[derive(Vertex)]
#[vertex(sem = "VertexSemantics")]
pub struct Vertex {
    pub idx: VertexIndex,
}

#[derive(UniformInterface)]
pub struct Uniforms {
    aspect: Uniform<f32>,
    fov: Uniform<f32>,
    cam: Uniform<M44>,
    cam_pos: Uniform<[f32; 3]>,
    light: Uniform<[f32; 3]>,
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

struct ShaderSrc {
    vertex: PathBuf,
    fragment: PathBuf,
}

impl ShaderSrc {
    pub fn load(&self) -> Program<VertexSemantics, (), Uniforms> {
        let vertex = fs::read_to_string(&self.vertex).unwrap();
        let fragment = fs::read_to_string(&self.fragment).unwrap();

        Program::from_strings(None, &vertex, None, &fragment)
            .map_err(|e| {
                println!("{}", e);
            })
            .unwrap()
            .ignore_warnings()
    }
}

struct App {
    surface: GlfwSurface,
    bb: Framebuffer<Dim2, (), ()>,
    triangle: Tess,
    program: Program<VertexSemantics, (), Uniforms>,

    size: glm::Vec2,
    prev_cursor: Option<glm::Vec2>,

    pos: glm::Vec3,

    rot: glm::Vec2,
    camera_up: glm::Vec3,
    camera_fw: glm::Vec3,

    holding_lmb: bool,
}

impl App {
    pub fn new((w, h): (u32, u32), shaders: &ShaderSrc) -> Self {
        let opt = WindowOpt::default()
            .set_num_samples(4);

        let mut surface =
            GlfwSurface::new(WindowDim::Windowed(w, h), "sdf-walker", opt).unwrap();

        let bb = surface.back_buffer().unwrap();

        let triangle = TessBuilder::new(&mut surface)
            .add_vertices(SCREEN)
            .set_mode(Mode::Triangle)
            .build()
            .unwrap();

        App {
            surface,
            bb,
            triangle,
            program: shaders.load(),

            size: glm::vec2(w as f32, h as f32),
            prev_cursor: None,

            pos: glm::Vec3::zeros(),

            rot: glm::vec2(0.0, 0.0),
            camera_up: glm::Vec3::y(),
            camera_fw: glm::Vec3::z(),

            holding_lmb: false,
        }
    }

    pub fn run(&mut self) {
        let start = Instant::now();
        let mut prev = Instant::now();

        'outer: loop {
            let now = Instant::now();
            let t = (now - start).as_secs_f32();
            let delta = (now - prev).as_secs_f32();
            prev = now;

            let camera = self.camera_rotation();

            for event in self.surface.poll_events() {
                match event {
                    WindowEvent::MouseButton(MouseButton::Button1, action, _) => {
                        match action {
                            Action::Press => self.holding_lmb = true,
                            Action::Release => self.holding_lmb = false,
                            _ => {}
                        };
                    }

                    WindowEvent::CursorPos(x, y) => {
                        let cursor = glm::vec2(x as f32, y as f32) * -2.0;
                        let diff = (cursor - self.prev_cursor.unwrap_or(cursor)).zip_map(&self.size, |a, b| a / b);
                        self.prev_cursor = Some(cursor);

                        if self.holding_lmb {
                            self.rot += diff;
                        }
                    }

                    WindowEvent::Key(key, _, Action::Press | Action::Repeat, _) => {
                        let dir = match key {
                            Key::W => glm::vec3(0.0, 0.0, 1.0),
                            Key::S => glm::vec3(0.0, 0.0, -1.0),
                            Key::A => glm::vec3(-1.0, 0.0, 0.0),
                            Key::D => glm::vec3(1.0, 0.0, 0.0),
                            Key::E => glm::vec3(0.0, 1.0, 0.0),
                            Key::Q => glm::vec3(0.0, -1.0, 0.0),
                            _ => glm::Vec3::zeros(),
                        };

                        self.pos += glm::quat_rotate_vec3(&glm::quat_inverse(&camera), &dir) * delta * 10.0;
                    }

                    WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
                        break 'outer
                    }
                    _ => {}
                }
            }

            self.draw(t);
        }
    }

    fn camera_rotation(&self) -> glm::Quat {
        let side_axis = glm::cross(&self.camera_up, &self.camera_fw);
        let rot = glm::quat_rotate(&glm::quat_identity(), self.rot.y, &side_axis);
        glm::quat_rotate(&rot, self.rot.x, &self.camera_up)
    }

    fn draw(&mut self, _: f32) {
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

        surface.pipeline_builder().pipeline(
            bb,
            &PipelineState::default().set_clear_color([0.0, 0.0, 0.0, 1.0]),
            |_, mut shader_gate| {
                shader_gate.shade(program, |iface, mut render_gate| {
                    iface.aspect.update(size.x / size.y);
                    iface.fov.update(glm::pi::<f32>() / 2.0);
                    iface.cam.update(glm::quat_to_mat4(&camera).into());
                    iface.cam_pos.update([pos.x, pos.y, pos.z]);
                    iface.light.update([1.0, -1.0, 1.0]);

                    render_gate.render(&RenderState::default(), |mut tess_gate| {
                        tess_gate.render(triangle.slice(..));
                    })
                })
            },
        );

        self.surface.swap_buffers();
    }
}
