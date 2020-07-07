use super::*;


#[derive(Debug)]
pub struct Transform<F, T> {
    pub tf: F,
    pub item: T,
}

impl<F: ITransform, T: IGeometry> IGeometry for Transform<F, T> {}
impl<F: ITransform, T: IOpaqueShape> IOpaqueShape for Transform<F, T> {}

impl<F: ITransform, T: MakeExpr> MakeExpr for Transform<F, T> {
    fn make_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr {
        let ctx = self.tf.create(ctx, func);
        self.item.make_expr(&ctx, func)
    }
}

#[derive(Debug)]
pub struct At {
    pub args: Vec<String>,
}

impl ITransform for At {
    fn create(&self, ctx: &Context, func: &mut glsl::Function) -> Context {
        let mut at = glsl::FunctionCall::new("at");
        for arg in &self.args {
            at.push_arg(arg.as_str().into());
        }
        at.push_arg(ctx.p.as_str().into());

        let ident = func.gen_definition("vec3", glsl::Expr::from(at));
        Context::with_p(ident)
    }
}
