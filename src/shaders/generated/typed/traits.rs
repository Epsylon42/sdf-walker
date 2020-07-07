use super::*;


pub trait MakeExpr: Debug + 'static {
    fn make_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr;
}

impl MakeExpr for Box<dyn MakeExpr> {
    fn make_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr {
        MakeExpr::make_expr(&**self, ctx, func)
    }
}


pub trait IGeometry: MakeExpr {}
impl IGeometry for Box<dyn IGeometry> {}
impl MakeExpr for Box<dyn IGeometry> {
    fn make_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr {
        MakeExpr::make_expr(&**self, ctx, func)
    }
}

pub trait IOpaqueShape: MakeExpr {}
impl IOpaqueShape for Box<dyn IOpaqueShape> {}
impl MakeExpr for Box<dyn IOpaqueShape> {
    fn make_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr {
        MakeExpr::make_expr(&**self, ctx, func)
    }
}

pub trait ITransform: Debug + 'static {
    fn create(&self, ctx: &Context, func: &mut glsl::Function) -> Context;
}


pub trait IFunc: Debug + 'static {
    fn name(&self, typ: TypeMarker) -> &'static str;
    fn id(&self, typ: TypeMarker) -> &'static str;
}
