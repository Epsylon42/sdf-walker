use super::*;
use glsl::{ArgString, RawString};

#[derive(Debug, Clone, Copy, Default)]
pub struct TransparentMarker;

impl ITypeMarker for TransparentMarker {}

#[derive(Debug)]
pub struct TransparentShape<G: IGeometry> {
    pub color: Vec<String>,
    pub geometry: G
}

impl<G: IGeometry> ITransparentShape for TransparentShape<G> {}

impl<G: IGeometry> MakeExpr for TransparentShape<G> {
    fn make_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr {
        let mut transparent = glsl::FunctionCall::new("MapTransparent");

        let mut color = glsl::FunctionCall::new("vec4");
        for c in &self.color {
            color.push_arg(ArgString::new(c, &ctx.arg));
        }

        transparent.push_arg(color);
        transparent.push_arg(self.geometry.make_expr(ctx, func));

        transparent.into()
    }
}

#[derive(Debug, Clone)]
pub struct NamedTransparentShape {
    pub name: String,
    pub args: Vec<String>,
}

impl ITransparentShape for NamedTransparentShape {}

impl MakeExpr for NamedTransparentShape {
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
pub struct RawTransparent {
    pub expr: String,
}

impl ITransparentShape for RawTransparent {}

impl MakeExpr for RawTransparent {
    fn make_expr(&self, ctx: &Context, _: &mut glsl::Function) -> glsl::Expr {
        ArgString::new(&self.expr, &ctx.arg).into()
    }
}
