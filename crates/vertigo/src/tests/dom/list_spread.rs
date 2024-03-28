use crate::dom;
use crate::{self as vertigo, DomNode};

#[test]
fn children_from_iter() {
    let list = (0..10)
        .map(|i| dom! { <li>{i}</li> });

    let node = dom! {
        <ul>
            "Children: "
            {..list}
        </ul>
    };

    let DomNode::Node { node } = node else {
        panic!("Expected DomNode::Node")
    };

    assert_eq!(node.get_children().len(), 11);
}

#[test]
fn children_from_iter_inline() {
    let node = dom! {
        <ul>
            "Children: "
            {..(0..10).map(|i| dom! { <li>{i}</li> })}
        </ul>
    };

    let DomNode::Node { node } = node else {
        panic!("Expected DomNode::Node")
    };

    assert_eq!(node.get_children().len(), 11);
}
