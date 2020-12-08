use super::*;
use glsl::{ArgString, RawString};

#[derive(Debug, Clone, Copy, Default)]
pub struct GeometryMarker;

impl ITypeMarker for GeometryMarker {}

#[derive(Debug)]
pub struct NamedGeometry {
    pub name: String,
    pub args: Vec<String>,
}

impl IGeometry for NamedGeometry {}

impl MakeExpr for NamedGeometry {
    fn make_expr(&self, ctx: &Context, _: &mut glsl::Function) -> glsl::Expr {
        let mut func = glsl::FunctionCall::new(&self.name);
        for arg in &self.args {
            func.push_arg(ArgString::new(arg, &ctx.arg));
        }

        func.push_arg(RawString::new(&ctx.arg));

        func.into()
    }
}

#[derive(Debug)]
pub struct RawGeometry {
    pub expr: String,
}

impl IGeometry for RawGeometry {}

impl MakeExpr for RawGeometry {
    fn make_expr(&self, ctx: &Context, _: &mut glsl::Function) -> glsl::Expr {
        ArgString::new(&self.expr, &ctx.arg).into()
    }
}
