use std::fmt::Debug;


pub trait IGeometry: Debug {

}

impl IGeometry for Box<dyn IGeometry> {

}

#[derive(Debug)]
pub struct Geometry {
    pub name: String,
    pub args: Vec<String>
}

impl IGeometry for Geometry {

}


pub trait IOpaqueShape: Debug {

}

impl IOpaqueShape for Box<dyn IOpaqueShape> {

}

#[derive(Debug)]
pub struct OpaqueShape<G: IGeometry> {
    pub color: [String; 3],
    pub geometry: G,
}

#[derive(Debug)]
pub struct NamedOpaqueShape {
    pub name: String,
    pub args: Vec<String>
}

impl<G: IGeometry> IOpaqueShape for OpaqueShape<G> {

}

impl IOpaqueShape for NamedOpaqueShape {

}


#[derive(Debug)]
pub struct Fold<F, T> {
    pub func: F,
    pub items: Vec<T>
}

impl<T: IGeometry, F: IFunc> IGeometry for Fold<F, T> {

}

impl<T: IOpaqueShape, F: IFunc> IOpaqueShape for Fold<F, T> {

}


#[derive(Debug)]
pub struct Transform<F, T> {
    pub tf: F,
    pub item: T,
}

impl<F: ITransform, T: IGeometry> IGeometry for Transform<F, T> {

}

impl<F: ITransform, T: IOpaqueShape> IOpaqueShape for Transform<F, T> {

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
}

impl IFunc for Union {
    const GEOM: &'static str = "sd_union";
    const OPAQ: &'static str = "csd_union";
}
impl IFunc for Isect {
    const GEOM: &'static str = "sd_isect";
    const OPAQ: &'static str = "csd_isect";
}
impl IFunc for Diff {
    const GEOM: &'static str = "sd_diff";
    const OPAQ: &'static str = "csd_diff";
}

#[derive(Debug)]
pub struct At {
    pub args: Vec<String>
}

pub trait ITransform: Debug {

}

impl ITransform for At {

}
