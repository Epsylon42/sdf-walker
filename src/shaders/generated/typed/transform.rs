use super::*;
use glsl::{ArgString, RawString};

#[derive(Debug)]
pub struct Transform<F, T, M> {
    pub tf: F,
    pub item: T,
    pub marker: M,
}

impl<F: ITransform, T: IGeometry> IGeometry for Transform<F, T, GeometryMarker> {}
impl<F: ITransform, T: IOpaqueShape> IOpaqueShape for Transform<F, T, OpaqueMarker> {}

impl<F: ITransform, T: MakeExpr, M: ITypeMarker> MakeExpr for Transform<F, T, M> {
    fn make_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr {
        self.tf.wrap(ctx, func, &self.item, self.marker.into())
    }
}

#[derive(Debug)]
pub struct FunctionTf {
    pub func: String,
    pub args: Vec<String>,
}

impl ITransform for FunctionTf {
    fn wrap(&self, ctx: &Context, func: &mut glsl::Function, inside: &impl MakeExpr, _typ: TypeMarker) -> glsl::Expr {
        let mut tf = glsl::FunctionCall::new(&self.func);
        for arg in &self.args {
            tf.push_arg(ArgString::new(arg, &ctx.arg));
        }
        tf.push_arg(RawString::new(&ctx.arg));

        let ident = func.gen_definition("Arg", tf);
        inside.make_expr(&Context::with_arg(ident), func)
    }
}

#[derive(Debug)]
pub struct Onionize {
    pub args: Vec<String>,
}

impl ITransform for Onionize {
    fn wrap(
        &self,
        ctx: &Context,
        func: &mut glsl::Function,
        inside: &impl MakeExpr,
        _: TypeMarker,
    ) -> glsl::Expr {
        let expr = inside.make_expr(ctx, func);
        let expr_ident = func.gen_definition("float", expr);

        let mut onionize = glsl::FunctionCall::new("sd_onionize");
        assert_eq!(self.args.len(), 1);
        onionize.push_arg(ArgString::new(&self.args[0], &ctx.arg));
        onionize.push_arg(RawString::new(expr_ident));

        onionize.into()
    }
}

#[derive(Debug)]
pub struct Scale {
    pub args: Vec<String>,
}

impl ITransform for Scale {
    fn wrap(
        &self,
        ctx: &Context,
        func: &mut glsl::Function,
        inside: &impl MakeExpr,
        typ: TypeMarker,
    ) -> glsl::Expr {
        let mut scale = glsl::FunctionCall::new("uscale");
        scale.push_arg(ArgString::new(&self.args[0], &ctx.arg));
        scale.push_arg(RawString::new(&ctx.arg));
        let ident = func.gen_definition("Arg", scale);

        let expr = inside.make_expr(&Context::with_arg(ident), func);

        let s = match typ {
            TypeMarker::Geometry(_) => format!("(({}) * ({}))", expr.to_string(), self.args[0]),
            TypeMarker::Opaque(_) => {
                let expr = func.gen_definition("vec4", expr);
                format!(
                    "vec4({expr}.xyz, {expr}.w * ({scale}))",
                    expr = expr,
                    scale = self.args[0]
                )
            }
        };

        ArgString::new(s, &ctx.arg).into()
    }
}

#[derive(Debug)]
pub struct AdvancedRepeat {
    pub args: Vec<String>,
}

impl ITransform for AdvancedRepeat {
    fn wrap(&self, ctx: &Context, func: &mut glsl::Function, inside: &impl MakeExpr, typ: TypeMarker) -> glsl::Expr {
        let mut fold = Fold {
            func: Union,
            items: Vec::new(),
            marker: typ,
        };

        for x in -1..=1 {
            if x != 0 && self.args[0] == "0" {
                continue;
            }

            for y in -1..=1 {
                if y != 0 && self.args[1] == "0" {
                    continue;
                }

                for z in -1..=1 {
                    if z != 0 && self.args[2] == "0" {
                        continue;
                    }

                    let offset = self.args.iter().zip(&[x, y, z])
                        .map(|(repeat_offset, mult)| format!("(({}) * ({}))", repeat_offset, mult))
                        .collect::<Vec<_>>();

                    let item = Transform {
                        tf: FunctionTf { func: String::from("at"), args: offset },
                        item: inside,
                        marker: typ,
                    };

                    fold.items.push(item);
                }
            }
        }

        let repeat = Transform {
            tf: FunctionTf { func: String::from("repeat"), args: self.args.clone() },
            item: fold,
            marker: typ,
        };

        repeat.make_expr(ctx, func)
    }
}
