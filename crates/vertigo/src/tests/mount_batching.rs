//! The mount has to reach the browser as one DOM batch.
//!
//! Hydration compares the tree the application built against the document the server
//! rendered. The tree is complete only after the mount transaction closes and
//! `flush_watch` runs - `<body>` is created late, after the whole `<head>` subtree
//! and after anything the app built before reaching the `dom!` block - so nothing may
//! reach the browser until that moment. The post-transaction flush hook is installed
//! only after that first send. From that point forward the driver flushes after each
//! transaction as usual.
//!
//! With `init_app` running outside any transaction, the first reactive binding closed an
//! *outermost* transaction and fired the hook. Now `mount` wraps construction in one
//! transaction, which makes those nested.

use std::{cell::RefCell, rc::Rc};

use crate::{self as vertigo, dom};
use crate::{
    DomNode, Value,
    dev::command::DriverDomCommand,
    driver_module::{driver::get_driver, get_driver_dom},
    exports::mount,
    reactive::on_after_transaction,
};

/// The shape the demo has: a subtree carrying a reactive binding, built as a statement
/// *before* the `dom!` that creates `<html>`/`<head>`/`<body>`.
fn app_with_a_binding_built_before_the_root() -> DomNode {
    let title = Value::new("a title".to_string());
    let label = Value::new("a label".to_string());

    // Subscribes on construction - this is what used to flush.
    let header = dom! { <div>{label}</div> };

    dom! {
        <html>
            <head>
                <title>{title}</title>
            </head>
            <body>
                <div>{header}</div>
            </body>
        </html>
    }
}

/// Mount, and return the commands split the way the browser receives them.
///
/// `flush_dom_changes` drains the command buffer, so one batch is "everything emitted since
/// the last time the post-transaction hooks fired". That is reconstructed here by teeing the
/// command stream (`inspect_command`) and cutting it at each hook fire - the same signal the
/// driver's own flush hook runs on, so the cuts land in the same places.
fn mount_capturing_batches(init_app: impl FnOnce() -> DomNode) -> Vec<Vec<DriverDomCommand>> {
    let pending: Rc<RefCell<Vec<DriverDomCommand>>> = Rc::new(RefCell::new(Vec::new()));
    let batches: Rc<RefCell<Vec<Vec<DriverDomCommand>>>> = Rc::new(RefCell::new(Vec::new()));

    let _tee = get_driver_dom().inspect_command({
        let pending = pending.clone();
        move |command| pending.borrow_mut().push(command)
    });

    let _cut = on_after_transaction({
        let pending = pending.clone();
        let batches = batches.clone();
        move || {
            let batch = pending.borrow_mut().drain(..).collect::<Vec<_>>();
            if !batch.is_empty() {
                batches.borrow_mut().push(batch);
            }
        }
    });

    mount(init_app);

    // Whatever `mount`'s trailing send picked up, for completeness.
    let tail = pending.borrow_mut().drain(..).collect::<Vec<_>>();
    if !tail.is_empty() {
        batches.borrow_mut().push(tail);
    }

    // Release the tree while the stores it drops into are still alive - see
    // `Driver::take_root`. Dropping it at thread teardown aborts the process.
    drop(get_driver().take_root());

    batches.borrow().clone()
}

/// One batch for the whole mount, however the tree was assembled.
#[test]
fn mount_emits_a_single_batch() {
    let batches = mount_capturing_batches(app_with_a_binding_built_before_the_root);

    assert_eq!(
        batches.len(),
        1,
        "the mount should reach the browser as one batch - hydration only ever sees the first \
         one, so a second means the rest of the tree is never matched"
    );
}

/// ...and that batch carries the roots hydration starts its walk from.
///
/// `<html>`, `<head>` and `<body>` are the only ids assigned by name (`DomId::from_name`).
/// `<body>` is the last of the three to be created - `dom!` evaluates a parent before its
/// children, so it goes `<html>`, then the whole `<head>` subtree, then `<body>` - which makes
/// its presence what says the batch was not cut short. Hydration bails without id 3.
#[test]
fn first_batch_contains_the_document_roots() {
    let batches = mount_capturing_batches(app_with_a_binding_built_before_the_root);

    let Some(first) = batches.first() else {
        panic!("the mount should emit some commands");
    };

    let created: Vec<u64> = first
        .iter()
        .filter_map(|command| match command {
            DriverDomCommand::CreateNode { id, .. } => Some(id.to_u64()),
            _ => None,
        })
        .collect();

    for (id, name) in [(1, "html"), (2, "head"), (3, "body")] {
        assert!(
            created.contains(&id),
            "<{name}> (id {id}) should be in the first batch, got {created:?}"
        );
    }
}

/// After the mount send, the post-transaction hook is live - a later write reaches the browser.
#[test]
fn after_the_mount_send_a_write_flushes() {
    let sends: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));

    let _tee = get_driver_dom().inspect_batch({
        let sends = sends.clone();
        move |_| {
            *sends.borrow_mut() += 1;
        }
    });

    let label = Value::new("one".to_string());
    let label_for_app = label.clone();

    mount(move || {
        dom! {
            <html>
                <head></head>
                <body>{label_for_app}</body>
            </html>
        }
    });

    assert_eq!(*sends.borrow(), 1, "mount is one send");

    label.set("two".to_string());

    assert_eq!(
        *sends.borrow(),
        2,
        "a write after mount should flush on its own"
    );

    drop(get_driver().take_root());
}
