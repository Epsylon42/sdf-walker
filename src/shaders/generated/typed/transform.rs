use super::*;


#[derive(Debug)]
pub struct Transform<F, T, M> {
    pub tf: F,
    pub item: T,
    pub marker: M,
}

impl<F: ITransform, T: IGeometry> IGeometry for Transform<F, T, GeometryMarker> {}
impl<F: ITransform, T: IOpaqueShape> IOpaqueShape for Transform<F, T, OpaqueMarker> {}

impl<F: ITransform, T: MakeExpr, M: Debug + 'static> MakeExpr for Transform<F, T, M> {
    fn make_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr {
        self.tf.wrap(ctx, func, &self.item)
    }
}

#[derive(Debug)]
pub struct At {
    pub args: Vec<String>,
}

impl ITransform for At {
    fn wrap(&self, ctx: &Context, func: &mut glsl::Function, inside: &impl MakeExpr) -> glsl::Expr {
        let mut at = glsl::FunctionCall::new("at");
        for arg in &self.args {
            at.push_arg(arg.as_str().into());
        }
        at.push_arg(ctx.p.as_str().into());

        let ident = func.gen_definition("vec3", glsl::Expr::from(at));
        inside.make_expr(&Context::with_p(ident), func)
    }
}

#[derive(Debug)]
pub struct Repeat {
    pub args: Vec<String>,
}

impl ITransform for Repeat {
    fn wrap(&self, ctx: &Context, func: &mut glsl::Function, inside: &impl MakeExpr) -> glsl::Expr {
        let mut at = glsl::FunctionCall::new("repeat");
        for arg in &self.args {
            at.push_arg(arg.as_str().into());
        }
        at.push_arg(ctx.p.as_str().into());

        let ident = func.gen_definition("vec3", glsl::Expr::from(at));
        inside.make_expr(&Context::with_p(ident), func)
    }
}

#[derive(Debug)]
pub struct Onionize {
    pub args: Vec<String>
}

impl ITransform for Onionize {
    fn wrap(&self, ctx: &Context, func: &mut glsl::Function, inside: &impl MakeExpr) -> glsl::Expr {
        let expr = inside.make_expr(&ctx, func);
        let expr_ident = func.gen_definition("float", expr);

        let mut onionize = glsl::FunctionCall::new("sd_onionize");
        assert_eq!(self.args.len(), 1);
        onionize.push_arg(glsl::Expr::from(self.args[0].clone()));
        onionize.push_arg(glsl::Expr::from(expr_ident));

        onionize.into()
    }
}
