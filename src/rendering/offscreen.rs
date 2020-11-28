use super::*;

impl CtxDetails for GlutinOffscreen {
    type FbCol = pixel::NormRGBA8UI;

    fn swap_buffers(&mut self) {}
    fn update_backbuffer(&mut self) -> Framebuffer<Self::Backend, Dim2, Self::FbCol, ()> {
        unimplemented!()
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
    pub fn to_image(&mut self) -> image::RgbImage {
        let color = self.bb.color_slot().get_raw_texels().unwrap();
        let image = image::RgbaImage::from_raw(self.size[0], self.size[1], color).unwrap();
        image::DynamicImage::ImageRgba8(image).flipv().to_rgb()
    }
}

pub fn new_app_offscreen(
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
