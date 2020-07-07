use super::codegen::Glsl;
use super::typed::*;

#[derive(Debug, Clone, PartialEq)]
pub struct SceneDesc {
    pub statements: Vec<Statement>,
}

impl ToString for SceneDesc {
    fn to_string(&self) -> String {
        let mut glsl = Glsl::new();

        let fold = Statement {
            name: String::from("union"),
            args: Vec::new(),
            body: self.statements.clone(),
        };

        let shape = fold.visit(&OpaqueVisitor).unwrap();

        let mut map = glsl.add_function("vec4", "map", &[("vec3", "p")]);
        let expr = shape.make_expr(&Context::new(), &mut map);
        map.ret(expr);

        glsl.to_string()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub name: String,
    pub args: Vec<String>,
    pub body: Vec<Statement>,
}

impl ToString for Statement {
    fn to_string(&self) -> String {
        let body = self.body.iter().map(|s| s.to_string()).collect::<Vec<_>>();

        format!(
            "{}({}){{{}}}",
            self.name,
            self.args.join(", "),
            body.join("; ")
        )
    }
}

#[derive(Debug, Clone)]
pub struct StatementError(String);

impl Statement {
    pub fn visit<V: StatementVisitor + ?Sized>(
        &self,
        vis: &V,
    ) -> Result<V::Output, StatementError> {
        let x = match self.name.as_str() {
            "union" => {
                assert!(self.args.is_empty());
                vis.construct_fold(Union, vis.visit_body(self)?)
            }

            "intersection" => {
                assert!(self.args.is_empty());
                vis.construct_fold(Isect, vis.visit_body(self)?)
            }

            "difference" => {
                assert!(self.args.is_empty());
                vis.construct_fold(Diff, vis.visit_body(self)?)
            }

            "opaque" => {
                let geom = GeometryVisitor;

                vis.construct_opaque(
                    self.args.clone(),
                    geom.construct_fold(Union, geom.visit_body(self)?),
                )?
            }

            "at" => {
                assert!(self.args.len() == 1 || self.args.len() == 3);
                vis.construct_transform(
                    At {
                        args: self.args.clone(),
                    },
                    vis.construct_fold(Union, vis.visit_body(self)?),
                )
            }

            _ => {
                assert!(self.body.is_empty());
                vis.construct_named(self.name.clone(), self.args.clone())
            }
        };

        Ok(x)
    }
}

pub trait StatementVisitor {
    type Output;

    fn construct_named(&self, name: String, args: Vec<String>) -> Self::Output;
    fn construct_fold(&self, func: impl IFunc, items: Vec<Self::Output>) -> Self::Output;
    fn construct_transform(&self, tf: impl ITransform, item: Self::Output) -> Self::Output;

    fn construct_opaque(
        &self,
        _color: Vec<String>,
        _geometry: impl IGeometry,
    ) -> Result<Self::Output, StatementError> {
        Err(StatementError(String::from(
            "cannot construct opaque shape",
        )))
    }

    fn visit_body(&self, stmt: &Statement) -> Result<Vec<Self::Output>, StatementError> {
        stmt.body.iter().map(|stmt| stmt.visit(self)).collect()
    }
}

pub struct GeometryVisitor;
impl StatementVisitor for GeometryVisitor {
    type Output = Box<dyn IGeometry>;

    fn construct_named(&self, name: String, args: Vec<String>) -> Self::Output {
        box NamedGeometry { name, args }
    }

    fn construct_fold(&self, func: impl IFunc, items: Vec<Self::Output>) -> Self::Output {
        box Fold { func, items, marker: GeometryMarker }
    }

    fn construct_transform(&self, tf: impl ITransform, item: Self::Output) -> Self::Output {
        box Transform { tf, item }
    }
}

pub struct OpaqueVisitor;
impl StatementVisitor for OpaqueVisitor {
    type Output = Box<dyn IOpaqueShape>;

    fn construct_named(&self, name: String, args: Vec<String>) -> Self::Output {
        box NamedOpaqueShape { name, args }
    }

    fn construct_fold(&self, func: impl IFunc, items: Vec<Self::Output>) -> Self::Output {
        box Fold { func, items, marker: OpaqueMarker }
    }

    fn construct_transform(&self, tf: impl ITransform, item: Self::Output) -> Self::Output {
        box Transform { tf, item }
    }

    fn construct_opaque(
        &self,
        color: Vec<String>,
        geometry: impl IGeometry,
    ) -> Result<Self::Output, StatementError> {
        Ok(box OpaqueShape { color, geometry })
    }
}
