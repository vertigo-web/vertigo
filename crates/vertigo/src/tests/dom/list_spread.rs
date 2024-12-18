use crate::dom;
use crate::{self as vertigo, DomNode};

#[test]
fn children_from_iter() {
    let list = (0..10).map(|i| dom! { <li>{i}</li> });

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

#[test]
fn iter_option() {
    let some_label = Some("Label".to_string());
    let none_label = Option::<String>::None;

    let node = dom! {
        <div>
            {..some_label}
            {..none_label}
        </div>
    };

    let DomNode::Node { node } = node else {
        panic!("Expected DomNode::Node")
    };

    assert_eq!(node.get_children().len(), 1);
}
