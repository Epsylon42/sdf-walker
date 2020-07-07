use super::*;



#[derive(Debug)]
pub struct Fold<F, T, M> {
    pub func: F,
    pub items: Vec<T>,
    pub marker: M,
}

impl<F: IFunc, T: IGeometry> IGeometry for Fold<F, T, GeometryMarker> {}
impl<F: IFunc, T: IOpaqueShape> IOpaqueShape for Fold<F, T, OpaqueMarker> {}

impl<F: IFunc, T: MakeExpr, M: Into<TypeMarker> + Clone + Debug + 'static> MakeExpr for Fold<F, T, M> 
{
    fn make_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr {
        match self.items.len() {
            0 => self.func.id(self.marker.clone().into()).into(),
            1 => self.items[0].make_expr(ctx, func),
            _ => {
                let func_name = self.func.name(self.marker.clone().into());

                let mut expr = self.items[0].make_expr(ctx, func);
                for item in self.items.iter().skip(1) {
                    let mut next_expr = glsl::FunctionCall::new(func_name);
                    next_expr.push_arg(expr);
                    next_expr.push_arg(item.make_expr(ctx, func));
                    expr = next_expr.into()
                }

                expr.into()
            }
        }
    }
}
