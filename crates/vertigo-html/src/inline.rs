// use vertigo::node_attr::{NodeAttr, text};
use vertigo::VDomNode;
use std::rc::Rc;

pub trait Inline {
    fn embed(self) -> VDomNode;
}

impl Inline for VDomNode {
    fn embed(self) -> VDomNode {
        self
    }
}

impl Inline for &str {
    fn embed(self) -> VDomNode {
        VDomNode::text(self)
    }
}

impl Inline for String {
    fn embed(self) -> VDomNode {
        VDomNode::text(&self)
    }
}

impl Inline for Rc<String> {
    fn embed(self) -> VDomNode {
        VDomNode::text(&*self)
    }
}

macro_rules! impl_to_string {
    ($ty: ty) => {
        impl Inline for $ty {
            fn embed(self) -> VDomNode {
                VDomNode::text(self.to_string())
            }
        }
    };
}

impl_to_string!(i8);
impl_to_string!(i16);
impl_to_string!(i32);
impl_to_string!(i64);
impl_to_string!(i128);
impl_to_string!(isize);

impl_to_string!(u8);
impl_to_string!(u16);
impl_to_string!(u32);
impl_to_string!(u64);
impl_to_string!(u128);
impl_to_string!(usize);

impl_to_string!(f32);
impl_to_string!(f64);

impl_to_string!(Rc<i8>);
impl_to_string!(Rc<i16>);
impl_to_string!(Rc<i32>);
impl_to_string!(Rc<i64>);
impl_to_string!(Rc<i128>);
impl_to_string!(Rc<isize>);

impl_to_string!(Rc<u8>);
impl_to_string!(Rc<u16>);
impl_to_string!(Rc<u32>);
impl_to_string!(Rc<u64>);
impl_to_string!(Rc<u128>);
impl_to_string!(Rc<usize>);

impl_to_string!(Rc<f32>);
impl_to_string!(Rc<f64>);
