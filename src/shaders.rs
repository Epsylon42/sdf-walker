use luminance::backend::shader::Shader;
use luminance::context::GraphicsContext;
use luminance::shader::{Program, UniformInterface};

mod generated;

pub use generated::{GeneratedScene, SceneDesc};

pub trait ShaderProvider {
    fn get_sources(&self) -> [String; 2];

    fn get_program<C, S, Out, Uni>(&self, ctx: &mut C) -> Program<C::Backend, S, Out, Uni>
    where
        C: GraphicsContext,
        C::Backend: Shader,
        S: luminance::vertex::Semantics,
        Uni: UniformInterface<C::Backend>,
    {
        let [vertex, fragment] = self.get_sources();

        ctx.new_shader_program()
            .from_strings(&vertex, None, None, &fragment)
            .unwrap()
            .ignore_warnings()
    }
}
