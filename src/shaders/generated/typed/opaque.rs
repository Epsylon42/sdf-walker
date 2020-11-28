use super::*;
use glsl::{ArgString, RawString};

#[derive(Debug, Clone, Copy, Default)]
pub struct OpaqueMarker;

impl ITypeMarker for OpaqueMarker {}

#[derive(Debug)]
pub struct OpaqueShape<G: IGeometry> {
    pub color: Vec<String>,
    pub geometry: G,
}

impl<G: IGeometry> IOpaqueShape for OpaqueShape<G> {}

impl<G: IGeometry> MakeExpr for OpaqueShape<G> {
    fn make_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr {
        let mut vec4 = glsl::FunctionCall::new("vec4");
        for c in &self.color {
            vec4.push_arg(ArgString::new(c, &ctx.arg));
        }

        vec4.push_arg(self.geometry.make_expr(ctx, func));

        vec4.into()
    }
}

#[derive(Debug, Clone)]
pub struct NamedOpaqueShape {
    pub name: String,
    pub args: Vec<String>,
}

impl IOpaqueShape for NamedOpaqueShape {}

impl MakeExpr for NamedOpaqueShape {
    fn make_expr(&self, ctx: &Context, _: &mut glsl::Function) -> glsl::Expr {
        let mut func = glsl::FunctionCall::new(&self.name);
        for arg in &self.args {
            func.push_arg(ArgString::new(arg, &ctx.arg));
        }

        func.push_arg(RawString::new(&ctx.arg));

        func.into()
    }
}
