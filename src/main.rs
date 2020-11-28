#![feature(or_patterns)]
#![feature(box_syntax)]

extern crate nalgebra_glm as glm;

use std::collections::HashSet;
use std::time::Instant;

use glutin::event::{
    DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent,
};
use glutin::event_loop::{ControlFlow, EventLoop};
use luminance::{
    backend::{self, color_slot::ColorSlot, shader::Uniformable},
    context::GraphicsContext,
    framebuffer::Framebuffer,
    pipeline::{PipelineError, PipelineState},
    pixel,
    render_state::RenderState,
    shader::{Program, Uniform},
    tess::{self, Mode, Tess, TessBuilder, View},
    texture::Dim2,
};
use luminance_derive::{Semantics, UniformInterface, Vertex};
use luminance_glutin::{GlutinOffscreen, GlutinSurface};

mod shaders;

use shaders::*;

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
                //let duration = scene
                    //.camera
                    //.as_ref()
                    //.map(|cam| cam.duration().ceil())
                    //.unwrap_or(10.0) as usize;

                //let (mut app, _) = new_app_offscreen([800, 600], scene);

                //let fps = 24;

                //let stdout = std::io::stdout();
                //let mut stdout = stdout.lock();

                //for i in 0..(fps * duration) {
                    //use std::io::Write;

                    //app.draw(i as f32 / fps as f32);
                    //stdout.write_all(&app.to_image().into_raw()).unwrap();
                //}
                //return;
            }
        }
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

trait CtxDetails: GraphicsContext
where
    Self::Backend: backend::framebuffer::Framebuffer<Dim2>,
{
    type FbCol: backend::color_slot::ColorSlot<Self::Backend, Dim2>;

    fn swap_buffers(&mut self);
    fn update_backbuffer(&mut self) -> Framebuffer<Self::Backend, Dim2, Self::FbCol, ()>
    where
        Self::Backend: backend::framebuffer::Framebuffer<Dim2>;
}

impl CtxDetails for GlutinSurface {
    type FbCol = ();

    fn swap_buffers(&mut self) {
        self.swap_buffers();
    }

    fn update_backbuffer(&mut self) -> Framebuffer<Self::Backend, Dim2, Self::FbCol, ()> {
        self.back_buffer().unwrap()
    }
}

impl CtxDetails for GlutinOffscreen {
    type FbCol = pixel::NormRGBA8UI;

    fn swap_buffers(&mut self) {}
    fn update_backbuffer(&mut self) -> Framebuffer<Self::Backend, Dim2, Self::FbCol, ()> {
        unimplemented!()
    }
}

struct App<Ctx, Col>
where
    Ctx: GraphicsContext,
    Ctx::Backend: backend::framebuffer::Framebuffer<Dim2>,
    Ctx::Backend: backend::tess::Tess<Vertex, (), (), tess::Interleaved>,
    Ctx::Backend: backend::shader::Shader,
    Col: ColorSlot<Ctx::Backend, Dim2>,
{
    scene_loader: Option<SceneDescLoader>,
    scene: SceneDesc,

    surface: Ctx,
    bb: Framebuffer<Ctx::Backend, Dim2, Col, ()>,

    triangle: Tess<Ctx::Backend, Vertex>,
    program: Program<Ctx::Backend, VertexSemantics, (), Uniforms>,

    size: [u32; 2],
    prev_cursor: Option<glm::Vec2>,
    holding_lmb: bool,
    pressed_keys: HashSet<VirtualKeyCode>,

    pos: glm::Vec3,
    rot: glm::Vec2,
    camera_up: glm::Vec3,
    camera_fw: glm::Vec3,
}

impl<Ctx, Col> App<Ctx, Col>
where
    Ctx: GraphicsContext + CtxDetails<FbCol = Col>,
    Ctx::Backend: backend::framebuffer::Framebuffer<Dim2>,
    Ctx::Backend: backend::tess::Tess<Vertex, (), (), tess::Interleaved>,
    Ctx::Backend: backend::shader::Shader,
    Ctx::Backend: backend::pipeline::Pipeline<Dim2>,
    Ctx::Backend: backend::render_gate::RenderGate,
    Ctx::Backend: backend::tess_gate::TessGate<Vertex, (), (), tess::Interleaved>,
    f32: Uniformable<Ctx::Backend>,
    [[f32; 4]; 4]: Uniformable<Ctx::Backend>,
    [f32; 3]: Uniformable<Ctx::Backend>,
    Col: ColorSlot<Ctx::Backend, Dim2>,
{
    fn camera_rotation(&self) -> glm::Quat {
        let side_axis = glm::cross(&self.camera_up, &self.camera_fw);
        let rot = glm::quat_rotate(&glm::quat_identity(), self.rot.y, &side_axis);
        glm::quat_rotate(&rot, self.rot.x, &self.camera_up)
    }

    fn draw(&mut self, time: f32) {
        let camera = self.camera_rotation();

        let Self {
            scene,
            surface,
            program,
            bb,
            triangle,
            size,
            pos,
            ..
        } = self;

        surface
            .new_pipeline_gate()
            .pipeline::<PipelineError, _, _, _, _>(
                bb,
                &PipelineState::default().set_clear_color([0.0, 0.0, 0.0, 1.0]),
                |_, mut shader_gate| {
                    shader_gate.shade(program, |mut iface, uni, mut render_gate| {
                        let fov = glm::pi::<f32>() / 2.0;

                        let (cam_pos, cam_rot) = if let Some(camera) = scene.camera.as_ref() {
                            camera.get_transform_at(time)
                        } else {
                            (*pos, camera)
                        };

                        iface.set(&uni.aspect, size[0] as f32 / size[1] as f32);
                        iface.set(&uni.fov, fov);
                        iface.set(&uni.cam, glm::quat_to_mat4(&cam_rot).into());
                        iface.set(&uni.cam_pos, [cam_pos.x, cam_pos.y, cam_pos.z]);
                        iface.set(&uni.light, [1.0, -1.0, 1.0]);
                        iface.set(&uni.time, time);

                        render_gate.render(&RenderState::default(), |mut tess_gate| {
                            tess_gate.render(triangle.view(..).unwrap())
                        })
                    })
                },
            );

        surface.swap_buffers();
    }

    pub fn update_scene_if_necessary(&mut self) {
        let new_scene = self.scene_loader
            .as_mut()
            .and_then(|loader| loader.load_if_updated());

        match new_scene {
            Some(Ok(new_scene)) => {
                match new_scene.get_program(&mut self.surface) {
                    Ok(new_program) => {
                        self.scene = new_scene;
                        self.program = new_program;
                    }

                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            }

            Some(Err(e)) => {
                eprintln!("{}", e);
            }

            None => {}
        }
    }

    pub fn run(mut self, el: EventLoop<()>) -> !
    where
        Ctx: 'static,
        Col: 'static,
    {
        let start = Instant::now();
        let mut prev = Instant::now();
        let mut now = Instant::now();
        let mut delta = 0.0;

        el.run(move |event, _, ctl| {
            match event {
                Event::MainEventsCleared => {
                    let camera = self.camera_rotation();

                    for key in &self.pressed_keys {
                        let dir = match key {
                            VirtualKeyCode::W => glm::vec3(0.0, 0.0, 1.0),
                            VirtualKeyCode::S => glm::vec3(0.0, 0.0, -1.0),
                            VirtualKeyCode::A => glm::vec3(-1.0, 0.0, 0.0),
                            VirtualKeyCode::D => glm::vec3(1.0, 0.0, 0.0),
                            _ => glm::Vec3::zeros(),
                        };

                        self.pos +=
                            glm::quat_rotate_vec3(&glm::quat_inverse(&camera), &dir) * delta * 10.0;

                        let abs_dir = match key {
                            VirtualKeyCode::E => glm::vec3(0.0, 1.0, 0.0),
                            VirtualKeyCode::Q => glm::vec3(0.0, -1.0, 0.0),
                            _ => glm::Vec3::zeros(),
                        };
                        self.pos += abs_dir * delta * 10.0;
                    }
                }

                Event::RedrawRequested(_) | Event::NewEvents(_) => {
                    now = Instant::now();
                    delta = (now - prev).as_secs_f32();
                    let t = (now - start).as_secs_f32();

                    self.update_scene_if_necessary();
                    self.draw(t);
                    prev = now;
                }

                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta: (x, y) },
                    ..
                } => {
                    let prev_cursor = self.prev_cursor.unwrap_or(glm::vec2(0.0, 0.0));
                    let cursor = prev_cursor + glm::vec2(x as f32, y as f32) * -2.0;

                    let diff = (cursor - prev_cursor).zip_map(
                        &glm::vec2(self.size[0] as f32, self.size[1] as f32),
                        |a, b| a / b,
                    );
                    self.prev_cursor = Some(cursor);

                    self.rot += diff;
                }

                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::MouseInput {
                        button: MouseButton::Left,
                        state,
                        ..
                    } => {
                        self.holding_lmb = state == ElementState::Pressed;
                    }

                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                state: ElementState::Released,
                                ..
                            },
                        ..
                    } => {
                        *ctl = ControlFlow::Exit;
                        return;
                    }

                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(key),
                                state,
                                ..
                            },
                        ..
                    } => {
                        match state {
                            ElementState::Pressed => self.pressed_keys.insert(key),
                            ElementState::Released => self.pressed_keys.remove(&key),
                        };
                    }

                    WindowEvent::Resized(size) => {
                        self.size = [size.width, size.height];
                        self.bb = self.surface.update_backbuffer();
                    }

                    _ => {}
                },

                _ => {}
            }

            *ctl = ControlFlow::WaitUntil(now + std::time::Duration::from_millis(33));
        });
    }
}

impl<Ctx> App<Ctx, pixel::NormRGBA8UI>
where
    Ctx: GraphicsContext,
    Ctx::Backend: backend::framebuffer::Framebuffer<Dim2>,
    Ctx::Backend: backend::tess::Tess<Vertex, (), (), tess::Interleaved>,
    Ctx::Backend: backend::shader::Shader,
    Ctx::Backend: backend::texture::Texture<Dim2, pixel::NormRGBA8UI>,
{
    fn to_image(&mut self) -> image::RgbImage {
        let color = self.bb.color_slot().get_raw_texels().unwrap();
        let image = image::RgbaImage::from_raw(self.size[0], self.size[1], color).unwrap();
        image::DynamicImage::ImageRgba8(image).flipv().to_rgb()
    }
}

fn new_app_offscreen(
    size: [u32; 2],
    scene: SceneDesc,
) -> (App<GlutinOffscreen, pixel::NormRGBA8UI>, EventLoop<()>) {
    let el = EventLoop::new();
    let ctx_builder = glutin::ContextBuilder::new();

    let mut surface = GlutinOffscreen::new_gl33_from_builder(&el, ctx_builder).unwrap();

    let bb = surface.new_framebuffer(size, 1, <_>::default()).unwrap();

    let triangle = TessBuilder::new(&mut surface)
        .set_vertices(SCREEN)
        .set_mode(Mode::Triangle)
        .build()
        .unwrap();

    let app = App {
        scene_loader: None,
        program: scene.get_program(&mut surface).unwrap(),
        scene,

        surface,
        bb,
        triangle,

        size,
        prev_cursor: None,
        holding_lmb: false,
        pressed_keys: HashSet::new(),

        pos: glm::Vec3::zeros(),
        rot: glm::vec2(0.0, 0.0),
        camera_up: glm::Vec3::y(),
        camera_fw: glm::Vec3::z(),
    };

    (app, el)
}

fn new_app(size: [u32; 2], mut scene_loader: SceneDescLoader) -> (App<GlutinSurface, ()>, EventLoop<()>) {
    let (mut surface, el) = GlutinSurface::new_gl33_from_builders(
        |_, wb| wb.with_inner_size(glutin::dpi::Size::Physical(size.into())),
        |_, cb| cb,
    )
    .unwrap();

    surface.ctx.window().set_cursor_visible(false);
    let _ = surface.ctx.window().set_cursor_grab(true);

    let bb = surface.back_buffer().unwrap();

    let triangle = TessBuilder::new(&mut surface)
        .set_vertices(SCREEN)
        .set_mode(Mode::Triangle)
        .build()
        .unwrap();

    let scene = scene_loader.load().unwrap();

    let app = App {
        scene_loader: Some(scene_loader),
        program: scene.get_program(&mut surface).unwrap(),
        scene,

        surface,
        bb,
        triangle,

        size,
        prev_cursor: None,
        holding_lmb: false,
        pressed_keys: HashSet::new(),

        pos: glm::Vec3::zeros(),
        rot: glm::vec2(0.0, 0.0),
        camera_up: glm::Vec3::y(),
        camera_fw: glm::Vec3::z(),
    };

    (app, el)
}
