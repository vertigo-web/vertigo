//! Hydration from the `mount` side: snapshot injected by mock, one batch on output.

use std::{cell::RefCell, rc::Rc};

use crate::{self as vertigo, dom};
use crate::{
    DomNode,
    dev::command::DriverDomCommand,
    driver_module::{
        api::api_dom_snapshot,
        driver::get_driver,
        get_driver_dom,
        hydration::{DomSnapshot, SnapshotAttr, SnapshotNode},
    },
};

fn snap_element(name: &str, attrs: Vec<(&str, &str)>, children: Vec<u32>) -> SnapshotNode {
    SnapshotNode::Element {
        name: name.to_string(),
        attrs: attrs
            .into_iter()
            .map(|(name, value)| SnapshotAttr {
                name: name.to_string(),
                value: value.to_string(),
            })
            .collect(),
        children,
    }
}

/// `<html><head><title>a title</title></head><body><div>hello</div></body></html>`, as the
/// server would render it.
fn server_rendered() -> DomSnapshot {
    DomSnapshot {
        nodes: vec![
            snap_element("html", vec![], vec![1, 3]),
            snap_element("head", vec![], vec![2]),
            snap_element("title", vec![], vec![5]),
            snap_element("body", vec![], vec![4]),
            snap_element("div", vec![], vec![6]),
            SnapshotNode::Text {
                value: "a title".to_string(),
            },
            SnapshotNode::Text {
                value: "hello".to_string(),
            },
        ],
        head: Some(1),
        body: Some(3),
    }
}

fn app() -> DomNode {
    dom! {
        <html>
            <head>
                <title>"a title"</title>
            </head>
            <body>
                <div>"hello"</div>
            </body>
        </html>
    }
}

/// Mounts and returns commands as the browser would see them.
///
/// `inspect_batch`, not `inspect_command`: the latter fires when the command is queued, and
/// hydration replaces the queued stream with the reconciled one, so adoptions would never pass
/// through that path.
///
/// Does not call `init_env()` to avoid installing a global logger that would interfere with
/// other tests' log capture (particularly `keyed_computed_list::a_row_read_after_the_list_moves_on_is_reported`).
fn mount_capturing(init_app: impl FnOnce() -> DomNode) -> Vec<DriverDomCommand> {
    let seen: Rc<RefCell<Vec<DriverDomCommand>>> = Rc::new(RefCell::new(Vec::new()));

    let _tee = get_driver_dom().inspect_batch({
        let seen = seen.clone();
        move |batch| seen.borrow_mut().extend(batch)
    });

    // Inline mount logic without calling init_env() to avoid global logger installation
    let driver = get_driver();
    get_driver_dom().arm_hydration();
    driver.transaction(|_| {
        let root_view = init_app();
        driver.set_root(root_view);
    });
    get_driver_dom().flush_hydration();

    // See `Driver::take_root` - dropping the tree when closing the thread reaches into
    // already freed store and aborts the process.
    drop(get_driver().take_root());

    seen.borrow().clone()
}

/// The entire document rendered by the server is adopted, not recreated.
#[test]
fn a_server_rendered_document_is_adopted_whole() {
    api_dom_snapshot().set_mock(server_rendered());

    let commands = mount_capturing(app);

    let adopted = commands
        .iter()
        .filter(|command| matches!(command, DriverDomCommand::NodeAdopt { .. }))
        .count();

    let created = commands
        .iter()
        .filter(|command| {
            matches!(
                command,
                DriverDomCommand::CreateNode { .. }
                    | DriverDomCommand::CreateText { .. }
                    | DriverDomCommand::CreateComment { .. }
            )
        })
        .count();

    assert_eq!(
        adopted, 4,
        "title, its text, the div and its text: {commands:?}"
    );
    assert_eq!(
        created, 0,
        "nothing should be rebuilt - the roots are resolved by id: {commands:?}"
    );
    assert!(
        !commands
            .iter()
            .any(|command| matches!(command, DriverDomCommand::SnapshotRemove { .. })),
        "there are no leftovers: {commands:?}"
    );
}

/// Without a snapshot - server rendering, tests on host - the stream goes out unchanged.
#[test]
fn without_a_snapshot_the_buffer_goes_out_verbatim() {
    let commands = mount_capturing(app);

    assert!(
        !commands
            .iter()
            .any(|command| matches!(command, DriverDomCommand::NodeAdopt { .. })),
        "nothing to adopt without a snapshot: {commands:?}"
    );

    let created = commands
        .iter()
        .filter(|command| matches!(command, DriverDomCommand::CreateNode { .. }))
        .count();

    assert_eq!(created, 5, "html, head, title, body, div: {commands:?}");
}

/// Leftover server markup that the client tree doesn't want goes to removal.
#[test]
fn leftover_server_markup_is_removed() {
    let mut snapshot = server_rendered();

    // Extra `<footer>` in body that the application doesn't draw.
    snapshot.nodes.push(snap_element("footer", vec![], vec![]));
    let footer = snapshot.nodes.len() as u32 - 1;

    if let Some(SnapshotNode::Element { children, .. }) = snapshot.nodes.get_mut(3) {
        children.push(footer);
    }

    api_dom_snapshot().set_mock(snapshot);

    let commands = mount_capturing(app);

    let removed: Vec<u32> = commands
        .iter()
        .filter_map(|command| match command {
            DriverDomCommand::SnapshotRemove { snapshot } => Some(*snapshot),
            _ => None,
        })
        .collect();

    assert_eq!(removed, vec![footer]);
}
