use serde::Deserialize;

use super::codegen::{Function, Glsl};

#[derive(Deserialize)]
pub struct SceneDesc {
    #[serde(default)]
    pub definitions: Vec<Definition>,
    pub shape: Shape,
}

impl SceneDesc {
    pub fn to_glsl(&self, glsl: &mut Glsl) {
        for def in &self.definitions {
            def.make(glsl);
        }

        let mut map = glsl.add_function("vec4", "map", &[("vec3", "p")]);
        let body = self.shape.to_glsl("p", &mut map);

        map.ret(body);
    }
}

impl ToString for SceneDesc {
    fn to_string(&self) -> String {
        let mut glsl = Glsl::new();
        self.to_glsl(&mut glsl);
        glsl.to_string()
    }
}

#[derive(Deserialize)]
pub struct Definition {
    name: String,
    #[serde(default)]
    args: Vec<(String, String)>,
    definition: DefinitionType,
}

#[derive(Deserialize)]
pub enum DefinitionType {
    Shape(Shape),
    Geometry(Geometry),
}

impl Definition {
    fn make(&self, glsl: &mut Glsl) {
        let Definition {
            name,
            args,
            definition,
        } = self;

        let ret = match definition {
            DefinitionType::Shape(_) => "vec4",
            DefinitionType::Geometry(_) => "float",
        };

        let mut args = args.clone();
        args.push((String::from("vec3"), String::from("p")));

        let mut func = glsl.add_function(ret, format!("definition_{}", name), &args);

        let body = match definition {
            DefinitionType::Shape(shape) => shape.to_glsl("p", &mut func),
            DefinitionType::Geometry(geom) => geom.to_glsl("p", &mut func),
        };

        func.ret(body);
    }
}

#[derive(Deserialize)]
pub enum Shape {
    Geometry {
        color: [Value; 3],
        geom: Geometry,
    },

    Definition {
        color: [Value; 3],
        name: String,
        #[serde(default)]
        args: Vec<Value>,
    },

    Union(Vec<Shape>),

    Tf {
        pos: Value,
        shape: Box<Shape>,
    },
}

impl Shape {
    fn to_glsl(&self, pos: &str, func: &mut Function) -> String {
        match self {
            Shape::Geometry {
                color: [r, g, b],
                geom,
            } => format!("vec4({},{},{}, {})", r.to_string(), g.to_string(), b.to_string(), geom.to_glsl(pos, func)),
            Shape::Definition { color: [r,g,b], name, args } => {
                let args = args.iter()
                    .map(|v| v.to_string() + ", ")
                    .collect::<Vec<_>>()
                    .join("");

                format!("vec4({},{},{}, definition_{}({}{}))", r.to_string(), g.to_string(), b.to_string(), name, args, pos)
            }

            Shape::Union(shapes) => funcfold(
                "vec4(0,0,0, (1.0/0.0))",
                "csd_union",
                |sh| sh.to_glsl(pos, func),
                shapes,
            ),

            Shape::Tf { pos: at, shape } => {
                let pos = func.gen_definition("vec3", format!("vat({}, {})", at.to_string(), pos));
                shape.to_glsl(&pos, func)
            }
        }
    }
}

#[derive(Deserialize)]
pub enum Geometry {
    Simple {
        name: String,
        #[serde(default)]
        args: Vec<Value>,
    },

    Definition {
        name: String,
        #[serde(default)]
        args: Vec<Value>,
    },

    Union(Vec<Geometry>),
    Isect(Vec<Geometry>),
    Diff([Box<Geometry>; 2]),
    Onion {
        thick: String,
        geom: Box<Geometry>,
    },

    Tf {
        pos: Value,
        geom: Box<Geometry>,
    },
}

impl Geometry {
    fn to_glsl(&self, pos: &str, func: &mut Function) -> String {
        match self {
            Geometry::Simple { name, args } => {
                let args = args.iter()
                    .map(|v| v.to_string() + ", ")
                    .collect::<Vec<_>>()
                    .join("");

                format!("sd_{}({}{})", name, args, pos)
            },

            Geometry::Definition { name, args } => {
                let args = args.iter()
                    .map(|v| v.to_string() + ", ")
                    .collect::<Vec<_>>()
                    .join("");

                format!("definition_{}({}{})", name, args, pos)
            }

            Geometry::Union(parts) => funcfold(
                "(1.0/0.0)",
                "sd_union",
                |geom| geom.to_glsl(pos, func),
                parts,
            ),
            Geometry::Isect(parts) => {
                funcfold("0.0", "sd_isect", |geom| geom.to_glsl(pos, func), parts)
            }
            Geometry::Diff([fst, snd]) => format!(
                "sd_diff({}, {})",
                fst.to_glsl(pos, func),
                snd.to_glsl(pos, func)
            ),
            Geometry::Onion { geom, thick } => {
                format!("sd_onionize({}, {})", thick, geom.to_glsl(pos, func))
            }

            Geometry::Tf { pos: at, geom } => {
                let pos = func.gen_definition("vec3", format!("vat({}, {})", at.to_string(), pos));
                geom.to_glsl(&pos, func)
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Value {
    Int(i32),
    Float(f32),
    String(String),
    Vec3([Box<Value>; 3]),
    Vec2([Box<Value>; 2]),
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::Int(x) => x.to_string(),
            Value::Float(x) => x.to_string(),
            Value::String(x) => x.clone(),
            Value::Vec3([x, y, z]) => format!("vec3({},{},{})", x.to_string(), y.to_string(), z.to_string()),
            Value::Vec2([x, y]) => format!("vec2({},{})", x.to_string(), y.to_string()),
        }
    }
}

fn funcfold<T, F>(zero: &str, func: &str, mut stringify: F, items: &[T]) -> String
where
    F: FnMut(&T) -> String,
{
    if items.len() == 0 {
        zero.to_string()
    } else {
        items
            .into_iter()
            .skip(1)
            .fold(stringify(&items[0]), |acc, x| {
                format!("{}({}, {})", func, acc, stringify(x))
            })
    }
}
