use std::fmt::Debug;

use super::codegen as glsl;

pub mod fold;
pub mod geometry;
pub mod opaque;
pub mod traits;
pub mod transform;

pub use fold::*;
pub use geometry::*;
pub use opaque::*;
pub use traits::*;
pub use transform::*;

pub struct Context {
    arg: String,
}

impl Context {
    pub fn new() -> Self {
        Context {
            arg: String::from("arg"),
        }
    }

    fn with_arg(arg: String) -> Self {
        Context { arg }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TypeMarker {
    Geometry(GeometryMarker),
    Opaque(OpaqueMarker),
}

impl ITypeMarker for TypeMarker {}

impl TypeMarker {
    fn typ(&self) -> &'static str {
        match self {
            TypeMarker::Geometry(_) => "float",
            TypeMarker::Opaque(_) => "vec4",
        }
    }
}

impl From<GeometryMarker> for TypeMarker {
    fn from(m: GeometryMarker) -> Self {
        TypeMarker::Geometry(m)
    }
}

impl From<OpaqueMarker> for TypeMarker {
    fn from(m: OpaqueMarker) -> Self {
        TypeMarker::Opaque(m)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Union;
#[derive(Debug, Clone, Copy)]
pub struct Isect;
#[derive(Debug, Clone, Copy)]
pub struct Diff;
#[derive(Debug, Clone)]
pub struct SmoothUnion {
    pub args: Vec<String>
}

impl IFunc for Union {
    fn name(&self, typ: TypeMarker) -> &'static str {
        match typ {
            TypeMarker::Geometry(_) => "sd_union",
            TypeMarker::Opaque(_) => "csd_union",
        }
    }
    fn id(&self, typ: TypeMarker) -> &'static str {
        match typ {
            TypeMarker::Geometry(_) => "1.0/0.0",
            TypeMarker::Opaque(_) => "vec3(0,0,0, 1.0/0.0)",
        }
    }
}

impl IFunc for Isect {
    fn name(&self, typ: TypeMarker) -> &'static str {
        match typ {
            TypeMarker::Geometry(_) => "sd_isect",
            TypeMarker::Opaque(_) => "csd_isect",
        }
    }
    fn id(&self, typ: TypeMarker) -> &'static str {
        match typ {
            TypeMarker::Geometry(_) => "0.0",
            TypeMarker::Opaque(_) => "vec3(0,0,0, 0.0)",
        }
    }
}

impl IFunc for Diff {
    fn name(&self, typ: TypeMarker) -> &'static str {
        match typ {
            TypeMarker::Geometry(_) => "sd_diff",
            TypeMarker::Opaque(_) => "csd_diff",
        }
    }
    fn id(&self, typ: TypeMarker) -> &'static str {
        match typ {
            TypeMarker::Geometry(_) => "1.0/0.0",
            TypeMarker::Opaque(_) => "vec3(0,0,0, 1.0/0.0)",
        }
    }
}

impl IFunc for SmoothUnion {
    fn name(&self, typ: TypeMarker) -> &'static str {
        match typ {
            TypeMarker::Geometry(_) => "sd_smooth_union",
            TypeMarker::Opaque(_) => "csd_smooth_union",
        }
    }
    fn id(&self, typ: TypeMarker) -> &'static str {
        match typ {
            TypeMarker::Geometry(_) => "1.0/0.0",
            TypeMarker::Opaque(_) => "vec3(0,0,0, 1.0/0.0)",
        }
    }
    fn extra_args(&self) -> &[String] {
        &self.args
    }
}
