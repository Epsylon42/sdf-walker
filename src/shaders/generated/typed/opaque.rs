use super::*;


#[derive(Debug, Clone, Copy, Default)]
pub struct OpaqueMarker;


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
            vec4.push_arg(c.as_str().into());
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
            func.push_arg(glsl::Expr::from(arg.as_str()));
        }

        func.push_arg(ctx.arg.as_str().into());

        func.into()
    }
}
