pub mod RealDom;
pub mod RealDomId;
pub mod RealDomChild;
pub mod RealDomNode;
pub mod RealDomText;
pub mod RealDomComment;
pub mod RealDomComponent;
pub mod VDom;
pub mod VDomNode;
pub mod VDomText;
pub mod VDomComponent;
pub mod VDomComponentId;

pub fn node<T: Into<String>>(name: T) -> VDom::VDom {
    VDom::VDom::node(name)
}

pub fn text<T: Into<String>>(name: T) -> VDom::VDom {
    VDom::VDom::text(name)
}

