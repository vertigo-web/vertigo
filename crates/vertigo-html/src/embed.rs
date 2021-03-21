// use vertigo::node_attr::{NodeAttr, text};
use vertigo::{VDomComponent, VDomElement, VDomNode};
use std::rc::Rc;

pub trait Embed {
    fn embed(self) -> VDomNode;
}

impl Embed for VDomNode {
    fn embed(self) -> VDomNode {
        self
    }
}

impl Embed for VDomElement {
    fn embed(self) -> VDomNode {
        VDomNode::Element {
            node: self
        }
    }
}

impl Embed for VDomComponent {
    fn embed(self) -> VDomNode {
        VDomNode::Component {
            node: self
        }
    }
}

impl Embed for &str {
    fn embed(self) -> VDomNode {
        VDomNode::text(self)
    }
}

impl Embed for String {
    fn embed(self) -> VDomNode {
        VDomNode::text(self)
    }
}

impl Embed for &String {
    fn embed(self) -> VDomNode {
        VDomNode::text(self)
    }
}

impl Embed for Rc<String> {
    fn embed(self) -> VDomNode {
        VDomNode::text(&*self)
    }
}

macro_rules! impl_to_string {
    ($ty: ty) => {
        impl Embed for $ty {
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
