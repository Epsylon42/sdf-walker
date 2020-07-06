use super::codegen::Glsl;
use super::typed::*;

#[derive(Debug, Clone, PartialEq)]
pub struct SceneDesc {
    pub statements: Vec<Statement>
}

impl ToString for SceneDesc {
    fn to_string(&self) -> String {
        let mut glsl = Glsl::new();

        let fold = Statement {
            name: String::from("union"),
            args: Vec::new(),
            body: self.statements.clone()
        };

        let shape = fold.to_opaque();

        let mut map = glsl.add_function("vec4", "map", &[("vec3", "p")]);
        let expr = shape.to_expr(&Context::new(), &mut map);
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
        let body = self.body
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        format!("{}({}){{{}}}", self.name, self.args.join(", "), body.join("; "))
    }
}

impl Statement {
    fn map_body<T>(&self, func: impl Fn(&Statement) -> T) -> Vec<T> {
        self.body
            .iter()
            .map(func)
            .collect()
    }

    pub fn to_geometry(&self) -> Box<dyn IGeometry> {
        match self.name.as_str() {
            "union" => {
                assert!(self.args.is_empty());

                box Fold {
                    func: Union,
                    items: self.map_body(Self::to_geometry),
                }
            }

            "intersection" => {
                assert!(self.args.is_empty());

                box Fold {
                    func: Isect,
                    items: self.map_body(Self::to_geometry),
                }
            }

            "difference" => {
                assert!(self.args.is_empty());

                box Fold {
                    func: Diff,
                    items: self.map_body(Self::to_geometry),
                }
            }

            "at" => {
                assert!(self.args.len() == 1 || self.args.len() == 3);

                box Transform {
                    tf: At {
                        args: self.args.clone()
                    },
                    item: Fold {
                        func: Union,
                        items: self.map_body(Self::to_geometry),
                    }
                }
            }

            _ => {
                assert!(self.body.is_empty());

                box Geometry {
                    name: self.name.clone(),
                    args: self.args.clone()
                }
            }
        }
    }

    pub fn to_opaque(&self) -> Box<dyn IOpaqueShape> {
        match self.name.as_str() {
            "union" => {
                assert!(self.args.is_empty());

                box Fold {
                    func: Union,
                    items: self.map_body(Self::to_opaque),
                }
            }

            "intersection" => {
                assert!(self.args.is_empty());

                box Fold {
                    func: Isect,
                    items: self.map_body(Self::to_opaque),
                }
            }

            "difference" => {
                assert!(self.args.is_empty());

                box Fold {
                    func: Diff,
                    items: self.map_body(Self::to_opaque),
                }
            }

            "at" => {
                assert!(self.args.len() == 1 || self.args.len() == 3);

                box Transform {
                    tf: At {
                        args: self.args.clone()
                    },
                    item: Fold {
                        func: Union,
                        items: self.map_body(Self::to_opaque),
                    }
                }
            }

            "opaque" => {
                assert!(self.args.len() == 3);

                let color = [self.args[0].clone(), self.args[1].clone(), self.args[2].clone()];

                box OpaqueShape {
                    color,
                    geometry: Fold {
                        func: Union,
                        items: self.map_body(Self::to_geometry)
                    }
                }
            }

            _ => {
                assert!(self.body.is_empty());

                box NamedOpaqueShape {
                    name: self.name.clone(),
                    args: self.args.clone()
                }
            }
        }
    }
}
