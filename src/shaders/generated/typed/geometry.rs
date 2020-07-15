use super::*;


#[derive(Debug, Clone, Copy, Default)]
pub struct GeometryMarker;


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
            func.push_arg(glsl::Expr::from(arg.as_str()));
        }

        func.push_arg(ctx.p.as_str().into());

        func.into()
    }
}