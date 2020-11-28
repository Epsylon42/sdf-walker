use super::*;
use std::time::Instant;

impl CtxDetails for GlutinSurface {
    type FbCol = ();

    fn swap_buffers(&mut self) {
        self.swap_buffers();
    }

    fn update_backbuffer(&mut self) -> Framebuffer<Self::Backend, Dim2, Self::FbCol, ()> {
        self.back_buffer().unwrap()
    }
}

pub fn new_app(size: [u32; 2], mut scene_loader: SceneDescLoader) -> (App<GlutinSurface, ()>, EventLoop<()>) {
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

    pub fn draw(&mut self, time: f32) {
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
        let mut start = Instant::now();
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
                                virtual_keycode: Some(VirtualKeyCode::R),
                                state: ElementState::Released,
                                ..
                            },
                        ..
                    } => {
                        start = now;
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
