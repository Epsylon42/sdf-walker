use luminance::backend::shader::Shader;
use luminance::context::GraphicsContext;
use luminance::shader::{Program, UniformInterface};

mod generated;

pub use generated::*;

#[derive(Debug, thiserror::Error)]
#[error("{}", .0)]
pub struct GetProgramError(#[from] luminance::shader::ProgramError);

pub trait ShaderProvider {
    fn get_sources(&self) -> [String; 2];

    fn get_program<C, S, Out, Uni>(&self, ctx: &mut C) -> Result<Program<C::Backend, S, Out, Uni>, GetProgramError>
    where
        C: GraphicsContext,
        C::Backend: Shader,
        S: luminance::vertex::Semantics,
        Uni: UniformInterface<C::Backend>,
    {
        let [vertex, fragment] = self.get_sources();

        let program = ctx.new_shader_program()
            .from_strings(&vertex, None, None, &fragment)?
            .ignore_warnings();

        Ok(program)
    }
}
