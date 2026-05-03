use crate::dev::inspect::{DomDebugFragment, log_start};
use crate::dom;
use crate::{self as vertigo};

#[test]
fn children_from_iter() {
    let list = (0..10).map(|i| dom! { <li>{i}</li> });

    log_start();
    let _el = dom! {
        <ul>
            "Children: "
            {..list}
        </ul>
    };
    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(
        html,
        "<ul>Children: <li>0</li><li>1</li><li>2</li><li>3</li><li>4</li><li>5</li><li>6</li><li>7</li><li>8</li><li>9</li></ul>"
    );
}

#[test]
fn children_from_iter_inline() {
    log_start();
    let _el = dom! {
        <ul>
            "Children: "
            {..(0..10).map(|i| dom! { <li>{i}</li> })}
        </ul>
    };
    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(
        html,
        "<ul>Children: <li>0</li><li>1</li><li>2</li><li>3</li><li>4</li><li>5</li><li>6</li><li>7</li><li>8</li><li>9</li></ul>"
    );
}

#[test]
fn children_from_block_with_iter_inline() {
    log_start();
    let _el = dom! {
        <ul>
            "Children: "
            {
                let iter = (0..10).map(|i| dom! { <li>{i}</li> });
                ..iter.skip(2)
            }
        </ul>
    };
    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(
        html,
        "<ul>Children: <li>2</li><li>3</li><li>4</li><li>5</li><li>6</li><li>7</li><li>8</li><li>9</li></ul>"
    );
}

#[test]
fn child_from_block() {
    log_start();
    let _el = dom! {
        <ul>
            "Children: "
            {
                let mut iter = (0..10).map(|i| dom! { <li>{i}</li> });
                ..iter.next()
            }
        </ul>
    };
    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(html, "<ul>Children: <li>0</li></ul>");
}

#[test]
fn iter_option() {
    let some_label = Some("Label".to_string());
    let none_label = Option::<String>::None;

    log_start();
    let _el = dom! {
        <div>
            {..some_label}
            {..none_label}
        </div>
    };
    let html = DomDebugFragment::from_log().to_pseudo_html();
    assert_eq!(html, "<div>Label</div>");
}
