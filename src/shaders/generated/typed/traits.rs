use super::*;

pub trait MakeExpr: Debug {
    fn make_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr;
}

impl MakeExpr for Box<dyn MakeExpr> {
    fn make_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr {
        MakeExpr::make_expr(&**self, ctx, func)
    }
}
impl<'a, T: MakeExpr> MakeExpr for &'a T {
    fn make_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr {
        MakeExpr::make_expr(&**self, ctx, func)
    }
}

pub trait IGeometry: MakeExpr + 'static {}
impl IGeometry for Box<dyn IGeometry> {}
impl MakeExpr for Box<dyn IGeometry> {
    fn make_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr {
        MakeExpr::make_expr(&**self, ctx, func)
    }
}

pub trait IOpaqueShape: MakeExpr + 'static {}
impl IOpaqueShape for Box<dyn IOpaqueShape> {}
impl MakeExpr for Box<dyn IOpaqueShape> {
    fn make_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr {
        MakeExpr::make_expr(&**self, ctx, func)
    }
}

pub trait ITransform: Debug + 'static {
    fn wrap(
        &self,
        ctx: &Context,
        func: &mut glsl::Function,
        inside: &impl MakeExpr,
        typ: TypeMarker,
    ) -> glsl::Expr;
}

pub trait IFunc: Debug + 'static {
    fn name(&self, typ: TypeMarker) -> &'static str;
    fn id(&self, typ: TypeMarker) -> &'static str;
    fn extra_args(&self) -> &[String] {
        &[]
    }
}

pub trait ITypeMarker: Into<TypeMarker> + Debug + Copy + 'static {}
