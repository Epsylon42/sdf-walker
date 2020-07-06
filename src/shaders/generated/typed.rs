use std::fmt::Debug;

use super::codegen as glsl;


pub struct Context {
    p: String,
}

impl Context {
    pub fn new() -> Self {
        Context { p: String::from("p") }
    }

    fn with_p(p: String) -> Self {
        Context { p }
    }
}


pub trait IGeometry: Debug {
    fn to_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr;
}

impl IGeometry for Box<dyn IGeometry> {
    fn to_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr {
        IGeometry::to_expr(&**self, ctx, func)
    }
}

#[derive(Debug)]
pub struct Geometry {
    pub name: String,
    pub args: Vec<String>
}

impl IGeometry for Geometry {
    fn to_expr(&self, ctx: &Context, _: &mut glsl::Function) -> glsl::Expr {
        let mut func = glsl::FunctionCall::new(&self.name);
        for arg in &self.args {
            func.push_arg(glsl::Expr::from(arg.as_str()));
        }

        func.push_arg(ctx.p.as_str().into());

        func.into()
    }
}


pub trait IOpaqueShape: Debug {
    fn to_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr;
}

impl IOpaqueShape for Box<dyn IOpaqueShape> {
    fn to_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr {
        IOpaqueShape::to_expr(&**self, ctx, func)
    }
}

#[derive(Debug)]
pub struct OpaqueShape<G: IGeometry> {
    pub color: [String; 3],
    pub geometry: G,
}

impl<G: IGeometry> IOpaqueShape for OpaqueShape<G> {
    fn to_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr {
        let mut vec4 = glsl::FunctionCall::new("vec4");
        for c in &self.color {
            vec4.push_arg(c.as_str().into());
        }

        vec4.push_arg(self.geometry.to_expr(ctx, func));

        vec4.into()
    }
}

#[derive(Debug)]
pub struct NamedOpaqueShape {
    pub name: String,
    pub args: Vec<String>
}

impl IOpaqueShape for NamedOpaqueShape {
    fn to_expr(&self, ctx: &Context, _: &mut glsl::Function) -> glsl::Expr {
        let mut func = glsl::FunctionCall::new(&self.name);
        for arg in &self.args {
            func.push_arg(glsl::Expr::from(arg.as_str()));
        }

        func.push_arg(ctx.p.as_str().into());

        func.into()
    }
}


#[derive(Debug)]
pub struct Fold<F, T> {
    pub func: F,
    pub items: Vec<T>
}

impl<T: IGeometry, F: IFunc> IGeometry for Fold<F, T> {
    fn to_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr {
        match self.items.len() {
            0 => F::ID.into(),
            1 => self.items[0].to_expr(ctx, func),
            _ => {
                let mut expr = self.items[0].to_expr(ctx, func);
                for item in self.items.iter().skip(1) {
                    let mut next_expr = glsl::FunctionCall::new(F::GEOM);
                    next_expr.push_arg(expr);
                    next_expr.push_arg(item.to_expr(ctx, func));
                    expr = next_expr.into()
                }

                expr.into()
            }
        }
    }
}

impl<T: IOpaqueShape, F: IFunc> IOpaqueShape for Fold<F, T> {
    fn to_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr {
        match self.items.len() {
            0 => F::ID.into(),
            1 => self.items[0].to_expr(ctx, func),
            _ => {
                let mut expr = self.items[0].to_expr(ctx, func);
                for item in self.items.iter().skip(1) {
                    let mut next_expr = glsl::FunctionCall::new(F::OPAQ);
                    next_expr.push_arg(expr);
                    next_expr.push_arg(item.to_expr(ctx, func));
                    expr = next_expr.into()
                }

                expr.into()
            }
        }
    }
}


#[derive(Debug)]
pub struct Transform<F, T> {
    pub tf: F,
    pub item: T,
}

impl<F: ITransform, T: IGeometry> IGeometry for Transform<F, T> {
    fn to_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr {
        let ctx = self.tf.create(ctx, func);
        self.item.to_expr(&ctx, func)
    }
}

impl<F: ITransform, T: IOpaqueShape> IOpaqueShape for Transform<F, T> {
    fn to_expr(&self, ctx: &Context, func: &mut glsl::Function) -> glsl::Expr {
        let ctx = self.tf.create(ctx, func);
        self.item.to_expr(&ctx, func)
    }
}



#[derive(Debug)]
pub struct Union;
#[derive(Debug)]
pub struct Isect;
#[derive(Debug)]
pub struct Diff;

pub trait IFunc: Debug {
    const GEOM: &'static str;
    const OPAQ: &'static str;
    const ID: &'static str;
}

impl IFunc for Union {
    const GEOM: &'static str = "sd_union";
    const OPAQ: &'static str = "csd_union";
    const ID: &'static str = "(1.0/0.0)";
}
impl IFunc for Isect {
    const GEOM: &'static str = "sd_isect";
    const OPAQ: &'static str = "csd_isect";
    const ID: &'static str = "0.0";
}
impl IFunc for Diff {
    const GEOM: &'static str = "sd_diff";
    const OPAQ: &'static str = "csd_diff";
    const ID: &'static str = "(1.0/0.0)";
}

#[derive(Debug)]
pub struct At {
    pub args: Vec<String>
}

pub trait ITransform: Debug {
    fn create(&self, ctx: &Context, func: &mut glsl::Function) -> Context;
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
