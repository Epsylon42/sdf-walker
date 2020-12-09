use crate::shaders::{GeneratedScene, ShaderProvider};

use std::collections::HashSet;

use super::codegen::Glsl;
use super::parser;
use super::typed::*;

pub mod camera;
pub mod loader;

use camera::{CameraDesc, CameraDescError};

#[derive(Debug, thiserror::Error)]
pub enum SceneDescError {
    #[error("Parse error")]
    ParseError,
    #[error("{}", .0)]
    StatementError(#[from] StatementError),
    #[error("Duplicate camera")]
    DuplicateCamera,
    #[error("{}", .0)]
    CameraError(#[from] CameraDescError),
}

#[derive(Debug, Clone)]
pub struct SceneDesc {
    pub vertex: String,
    pub fragment: String,
    pub camera: Option<CameraDesc>,
}

impl SceneDesc {
    pub fn parse(source: &[u8]) -> Result<Self, SceneDescError> {
        let statements = parser::scene(source)
            .map_err(|_| SceneDescError::ParseError)?.1;
        Self::from_statements(statements)
    }

    pub fn from_statements(statements: Vec<Statement>) -> Result<Self, SceneDescError> {
        let mut glsl = Glsl::new();

        let mut fold_opaque = Statement {
            name: String::from("union"),
            args: Vec::new(),
            body: Vec::new(),
        };

        let mut fold_transparent = Statement {
            name: String::from("union"),
            args: Vec::new(),
            body: Vec::new()
        };

        let mut camera = None;

        for stmt in statements {
            match stmt.name.as_str() {
                "define_geometry" => define_object(&mut glsl, stmt, GeometryVisitor)?,
                "define_opaque" => define_object(&mut glsl, stmt, OpaqueVisitor)?,
                "define_transparent" => define_object(&mut glsl, stmt, TransparentVisitor)?,
                "camera" => {
                    if camera.is_some() {
                        return Err(SceneDescError::DuplicateCamera)
                    } else {
                        camera = Some(CameraDesc::new(stmt)?)
                    }
                }
                _ => {
                    if stmt.apply(&TransparentVisitor).is_ok() {
                        fold_transparent.body.push(stmt);
                    } else {
                        fold_opaque.body.push(stmt);
                    }
                }
            }
        }

        let opaque = fold_opaque.apply(&OpaqueVisitor)?;
        let transparent = fold_transparent.apply(&TransparentVisitor)?;

        let mut map = glsl.add_function("vec4", "map_impl", &[("Arg", "arg")]);
        let expr = opaque.make_expr(&Context::new(), &mut map);
        map.ret(&mut glsl, expr);

        let mut map_transparent = glsl.add_function("MapTransparent", "map_transparent_impl", &[("Arg", "arg")]);
        let expr = transparent.make_expr(&Context::new(), &mut map_transparent);
        map_transparent.ret(&mut glsl, expr);

        Ok(
            SceneDesc {
                vertex: GeneratedScene::get_vertex(),
                fragment: GeneratedScene::compile_fragment(&glsl.to_string()),
                camera,
            }
        )
    }
}

impl ShaderProvider for SceneDesc {
    fn get_sources(&self) -> [String; 2] {
        [self.vertex.clone(), self.fragment.clone()]
    }
}

fn define_object(
    glsl: &mut Glsl,
    stmt: Statement,
    visitor: impl StatementVisitor,
) -> Result<(), StatementError> {
    if stmt.args.is_empty() {
        return Err(StatementError(format!("{} requires at least one argument", stmt.name)));
    }

    let fold = Statement {
        name: String::from("union"),
        args: Vec::new(),
        body: stmt.body,
    };

    let object = fold.apply(&visitor)?;

    let name = &stmt.args[0];
    let args = stmt.args
        .iter()
        .skip(1)
        .map(|arg| {
            let parts = arg.split_whitespace().collect::<Vec<_>>();

            (parts[0], parts[1])
        })
        .chain(std::iter::once(("Arg", "arg")))
        .collect::<Vec<_>>();

    let type_name = visitor.get_type_marker().typ();
    let mut func = glsl.add_function(type_name, name, &args);
    let expr = object.make_expr(&Context::new(), &mut func);
    func.ret(glsl, expr);

    Ok(())
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

#[derive(Debug, Clone, thiserror::Error)]
#[error("Statement error: {}", .0)]
pub struct StatementError(String);

impl Statement {
    pub fn apply<V: StatementVisitor + ?Sized>(
        &self,
        vis: &V,
    ) -> Result<V::Output, StatementError> {
        lazy_static! {
            static ref SIMPLE_FUNCTIONS: HashSet<&'static str> = {
                let simple_functions = [
                    "at",
                    "vat",
                    "rotate",
                    "repeat",
                    "at_t",
                    "start_at_t",
                    "end_at_t",
                    "repeat_t",
                    "map_t",
                ];

                simple_functions.iter()
                    .cloned()
                    .collect()
            };
        }

        let x = match self.name.as_str() {
            "raw" => {
                assert_eq!(self.args.len(), 1);
                vis.construct_raw(self.args[0].clone())
            }

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

            "smooth_union" => {
                assert_eq!(self.args.len(), 1);
                vis.construct_fold(SmoothUnion{ args: self.args.clone() }, vis.visit_body(self)?)
            }

            "advanced_repeat" => {
                assert_eq!(self.args.len(), 3);
                vis.construct_transform(
                    AdvancedRepeat {
                        args: self.args.clone()
                    },
                    vis.construct_fold(Union, vis.visit_body(self)?)
                )
            }

            "opaque" => {
                let geom = GeometryVisitor;

                vis.construct_opaque(
                    self.args.clone(),
                    geom.construct_fold(Union, geom.visit_body(self)?),
                )?
            }

            "transparent" => {
                let geom = GeometryVisitor;

                vis.construct_transparent(
                    self.args.clone(),
                    geom.construct_fold(Union, geom.visit_body(self)?),
                )?
            }

            "onionize" => {
                assert!(self.args.len() == 1);
                vis.construct_transform(
                    Onionize {
                        args: self.args.clone(),
                    },
                    vis.construct_fold(Union, vis.visit_body(self)?),
                )
            }

            "scale" => {
                assert!(self.args.len() == 1);
                vis.construct_transform(
                    Scale {
                        args: self.args.clone(),
                    },
                    vis.construct_fold(Union, vis.visit_body(self)?),
                )
            }

            "cond" => {
                assert!(self.args.len() == 1);
                vis.construct_transform(
                    Cond {
                        args: self.args.clone(),
                    },
                    vis.construct_fold(Union, vis.visit_body(self)?),
                )
            }

            "let" => {
                assert_eq!(self.args.len(), 3);
                vis.construct_transform(
                    Let {
                        args: self.args.clone(),
                    },
                    vis.construct_fold(Union, vis.visit_body(self)?),
                )
            }

            x if SIMPLE_FUNCTIONS.contains(x) => {
                vis.construct_transform(
                    FunctionTf {
                        func: String::from(x),
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
    type Output: MakeExpr;

    fn get_type_marker(&self) -> TypeMarker;

    fn construct_named(&self, name: String, args: Vec<String>) -> Self::Output;
    fn construct_raw(&self, expr: String) -> Self::Output;
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

    fn construct_transparent(
        &self,
        _color: Vec<String>,
        _geometry: impl IGeometry,
    ) -> Result<Self::Output, StatementError> {
        Err(StatementError(String::from(
            "cannot construct transparent shape",
        )))
    }

    fn visit_body(&self, stmt: &Statement) -> Result<Vec<Self::Output>, StatementError> {
        stmt.body.iter().map(|stmt| stmt.apply(self)).collect()
    }
}

pub struct GeometryVisitor;
impl StatementVisitor for GeometryVisitor {
    type Output = Box<dyn IGeometry>;

    fn get_type_marker(&self) -> TypeMarker {
        TypeMarker::Geometry(GeometryMarker)
    }

    fn construct_named(&self, name: String, args: Vec<String>) -> Self::Output {
        box NamedGeometry { name, args }
    }

    fn construct_raw(&self, expr: String) -> Self::Output {
        box RawGeometry { expr }
    }

    fn construct_fold(&self, func: impl IFunc, items: Vec<Self::Output>) -> Self::Output {
        box Fold {
            func,
            items,
            marker: GeometryMarker,
        }
    }

    fn construct_transform(&self, tf: impl ITransform, item: Self::Output) -> Self::Output {
        box Transform {
            tf,
            item,
            marker: GeometryMarker,
        }
    }
}

pub struct OpaqueVisitor;
impl StatementVisitor for OpaqueVisitor {
    type Output = Box<dyn IOpaqueShape>;

    fn get_type_marker(&self) -> TypeMarker {
        TypeMarker::Opaque(OpaqueMarker)
    }

    fn construct_named(&self, name: String, args: Vec<String>) -> Self::Output {
        box NamedOpaqueShape { name, args }
    }

    fn construct_raw(&self, expr: String) -> Self::Output {
        box RawOpaque { expr }
    }

    fn construct_fold(&self, func: impl IFunc, items: Vec<Self::Output>) -> Self::Output {
        box Fold {
            func,
            items,
            marker: OpaqueMarker,
        }
    }

    fn construct_transform(&self, tf: impl ITransform, item: Self::Output) -> Self::Output {
        box Transform {
            tf,
            item,
            marker: OpaqueMarker,
        }
    }

    fn construct_opaque(
        &self,
        color: Vec<String>,
        geometry: impl IGeometry,
    ) -> Result<Self::Output, StatementError> {
        Ok(box OpaqueShape { color, geometry })
    }
}

pub struct TransparentVisitor;
impl StatementVisitor for TransparentVisitor {
    type Output = Box<dyn ITransparentShape>;

    fn get_type_marker(&self) -> TypeMarker {
        TypeMarker::Transparent(TransparentMarker)
    }

    fn construct_named(&self, name: String, args: Vec<String>) -> Self::Output {
        box NamedTransparentShape { name, args }
    }

    fn construct_raw(&self, expr: String) -> Self::Output {
        box RawTransparent { expr }
    }

    fn construct_fold(&self, func: impl IFunc, items: Vec<Self::Output>) -> Self::Output {
        box Fold {
            func,
            items,
            marker: TransparentMarker
        }
    }

    fn construct_transform(&self, tf: impl ITransform, item: Self::Output) -> Self::Output {
        box Transform {
            tf,
            item,
            marker: TransparentMarker,
        }
    }

    fn construct_transparent(&self, color: Vec<String>, geometry: impl IGeometry) -> Result<Self::Output, StatementError> {
        Ok(box TransparentShape { color, geometry })
    }
}
