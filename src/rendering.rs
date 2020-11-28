use std::collections::HashSet;

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

use crate::shaders::*;

pub mod onscreen;
pub mod offscreen;


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

pub trait CtxDetails: GraphicsContext
where
    Self::Backend: backend::framebuffer::Framebuffer<Dim2>,
{
    type FbCol: backend::color_slot::ColorSlot<Self::Backend, Dim2>;

    fn swap_buffers(&mut self);
    fn update_backbuffer(&mut self) -> Framebuffer<Self::Backend, Dim2, Self::FbCol, ()>
    where
        Self::Backend: backend::framebuffer::Framebuffer<Dim2>;
}

pub struct App<Ctx, Col>
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
