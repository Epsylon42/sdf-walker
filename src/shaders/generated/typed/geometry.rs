use super::*;

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
            func.push_arg(arg);
        }

        func.push_arg(&ctx.arg);

        func.into()
    }
}
