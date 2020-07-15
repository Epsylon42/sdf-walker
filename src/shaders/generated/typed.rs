use std::fmt::Debug;

use super::codegen as glsl;

pub mod traits;
pub mod geometry;
pub mod opaque;
pub mod transform;
pub mod fold;

pub use traits::*;
pub use geometry::*;
pub use opaque::*;
pub use transform::*;
pub use fold::*;


pub struct Context {
    p: String,
}

impl Context {
    pub fn new() -> Self {
        Context {
            p: String::from("p"),
        }
    }

    fn with_p(p: String) -> Self {
        Context { p }
    }
}


#[derive(Debug, Clone, Copy)]
pub enum TypeMarker {
    Geometry(GeometryMarker),
    Opaque(OpaqueMarker)
}

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
