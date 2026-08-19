use std::{cell::Cell, rc::Rc};

use super::{Computed, DropResource, Graph, Value};

#[test]
fn basic_sum() {
    let g = Graph::new();
    let a = g.value(1);
    let b = g.value(2);
    let sum = g.computed({
        let a = a.clone();
        let b = b.clone();
        move |ctx| a.get(ctx) + b.get(ctx)
    });

    g.transaction(|ctx| {
        assert_eq!(sum.get(ctx), 3);
    });

    a.set(4);
    g.transaction(|ctx| {
        assert_eq!(sum.get(ctx), 6);
    });
}

#[test]
fn two_sets_in_one_transaction_compute_once() {
    let g = Graph::new();
    let a = g.value(0);
    let b = g.value(0);
    let runs = Rc::new(Cell::new(0));
    let sum = g.computed({
        let a = a.clone();
        let b = b.clone();
        let runs = runs.clone();
        move |ctx| {
            runs.set(runs.get() + 1);
            a.get(ctx) + b.get(ctx)
        }
    });

    g.transaction(|ctx| {
        assert_eq!(sum.get(ctx), 0);
    });
    runs.set(0);

    g.transaction(|_| {
        a.set(10);
        b.set(20);
    });

    g.transaction(|ctx| {
        assert_eq!(sum.get(ctx), 30);
    });
    assert_eq!(runs.get(), 1);
}

#[test]
fn cutoff_leaves_fanout_untouched() {
    let g = Graph::new();
    let a = g.value(1);
    let even = g.computed({
        let a = a.clone();
        move |ctx| a.get(ctx) % 2 == 0
    });

    let child_runs = Rc::new(Cell::new(0));
    let children: Vec<_> = (0..10_000)
        .map(|i| {
            let even = even.clone();
            let child_runs = child_runs.clone();
            g.computed(move |ctx| {
                child_runs.set(child_runs.get() + 1);
                (even.get(ctx), i)
            })
        })
        .collect();

    g.transaction(|ctx| {
        assert!(!even.get(ctx));
        for child in &children {
            let _ = child.get(ctx);
        }
    });
    assert_eq!(child_runs.get(), 10_000);

    child_runs.set(0);
    a.set(3);
    g.transaction(|ctx| {
        assert!(!even.get(ctx));
    });
    assert_eq!(
        child_runs.get(),
        0,
        "odd→odd: even cut off, 10_000 must not run"
    );

    a.set(4);
    g.transaction(|ctx| {
        assert!(even.get(ctx));
        assert_eq!(children[0].get(ctx), (true, 0));
    });
    assert_eq!(
        child_runs.get(),
        10_000,
        "odd→even: fan-out must run once each"
    );
}

#[test]
fn diamond_waits_for_both_parents() {
    let g = Graph::new();
    let a = g.value(1);
    let left = g.computed({
        let a = a.clone();
        move |ctx| a.get(ctx) + 1
    });
    let right = g.computed({
        let a = a.clone();
        move |ctx| a.get(ctx) * 10
    });
    let d_runs = Rc::new(Cell::new(0));
    let d = g.computed({
        let left = left.clone();
        let right = right.clone();
        let d_runs = d_runs.clone();
        move |ctx| {
            d_runs.set(d_runs.get() + 1);
            left.get(ctx) + right.get(ctx)
        }
    });

    g.transaction(|ctx| {
        assert_eq!(d.get(ctx), 12);
    });
    d_runs.set(0);

    a.set(2);
    g.transaction(|ctx| {
        assert_eq!(left.get(ctx), 3);
        assert_eq!(right.get(ctx), 20);
        assert_eq!(d.get(ctx), 23);
    });
    assert_eq!(d_runs.get(), 1);
}

#[test]
fn subscribe_skips_when_parent_cuts_off() {
    let g = Graph::new();
    let a = g.value(1);
    let even = g.computed({
        let a = a.clone();
        move |ctx| a.get(ctx) % 2 == 0
    });

    let fires = Rc::new(Cell::new(0));
    let last = Rc::new(Cell::new(false));
    let _sub: DropResource = even.subscribe({
        let fires = fires.clone();
        let last = last.clone();
        move |v| {
            fires.set(fires.get() + 1);
            last.set(v);
        }
    });

    assert_eq!(fires.get(), 1);
    assert!(!last.get());

    a.set(3);
    assert_eq!(fires.get(), 1);

    a.set(4);
    assert_eq!(fires.get(), 2);
    assert!(last.get());
}

#[test]
fn equal_set_does_not_enqueue() {
    let g = Graph::new();
    let a = g.value(5);
    let runs = Rc::new(Cell::new(0));
    let c = g.computed({
        let a = a.clone();
        let runs = runs.clone();
        move |ctx| {
            runs.set(runs.get() + 1);
            a.get(ctx)
        }
    });

    g.transaction(|ctx| {
        assert_eq!(c.get(ctx), 5);
    });
    runs.set(0);

    a.set(5);
    assert_eq!(runs.get(), 0);
}

#[test]
fn default_graph_value_new() {
    let a = Value::new(1);
    let double = Computed::from({
        let a = a.clone();
        move |ctx| a.get(ctx) * 2
    });

    super::transaction(|ctx| {
        assert_eq!(double.get(ctx), 2);
    });

    a.set(5);
    super::transaction(|ctx| {
        assert_eq!(double.get(ctx), 10);
    });
}

#[test]
fn when_connect_tracks_watchers() {
    let connect_count = Rc::new(Cell::new(0));
    let disconnect_count = Rc::new(Cell::new(0));

    let value = Value::new(1);
    let comp = value.to_computed().when_connect({
        let connect_count = connect_count.clone();
        let disconnect_count = disconnect_count.clone();
        move || {
            connect_count.set(connect_count.get() + 1);
            DropResource::new({
                let disconnect_count = disconnect_count.clone();
                move || {
                    disconnect_count.set(disconnect_count.get() + 1);
                }
            })
        }
    });

    assert_eq!(connect_count.get(), 0);
    assert_eq!(disconnect_count.get(), 0);

    let drop_resource = comp.subscribe(|_| {});

    assert_eq!(connect_count.get(), 1);
    assert_eq!(disconnect_count.get(), 0);

    drop(drop_resource);

    assert_eq!(connect_count.get(), 1);
    assert_eq!(disconnect_count.get(), 1);
}

#[test]
fn when_connect_multiple_subscribers_share_one_connection() {
    let connect_count = Rc::new(Cell::new(0));
    let disconnect_count = Rc::new(Cell::new(0));

    let value = Value::new(1);
    let comp = value.to_computed().when_connect({
        let connect_count = connect_count.clone();
        let disconnect_count = disconnect_count.clone();
        move || {
            connect_count.set(connect_count.get() + 1);
            DropResource::new({
                let disconnect_count = disconnect_count.clone();
                move || {
                    disconnect_count.set(disconnect_count.get() + 1);
                }
            })
        }
    });

    let drop1 = comp.clone().subscribe(|_| {});
    assert_eq!(connect_count.get(), 1);

    let drop2 = comp.subscribe(|_| {});
    assert_eq!(connect_count.get(), 1);

    drop(drop1);
    assert_eq!(disconnect_count.get(), 0);

    drop(drop2);
    assert_eq!(disconnect_count.get(), 1);
}

#[test]
fn nested_computed_subscription() {
    let token_value = Value::new("token1".to_string());
    let token_computed = token_value.to_computed();

    let bearer_auth = Computed::from({
        let token_computed = token_computed.clone();
        move |_ctx| Some(token_computed.clone())
    });

    let counter = Rc::new(Cell::new(0));

    let revalidate_trigger = Computed::from({
        let bearer_auth = bearer_auth.clone();
        move |ctx| bearer_auth.get(ctx).map(|c| c.get(ctx))
    });

    let _drop = revalidate_trigger.subscribe({
        let counter = counter.clone();
        move |_| {
            counter.set(counter.get() + 1);
        }
    });

    assert_eq!(counter.get(), 1);

    token_value.set("token2".to_string());
    assert_eq!(counter.get(), 2);
}

#[test]
fn nested_computed_subscription_no_flattening() {
    let token_value = Value::new("token1".to_string());
    let token_computed = token_value.to_computed();

    let bearer_auth = Computed::from({
        let token_computed = token_computed.clone();
        move |_ctx| Some(token_computed.clone())
    });

    let counter = Rc::new(Cell::new(0));

    let _drop = bearer_auth.subscribe({
        let counter = counter.clone();
        move |_| {
            counter.set(counter.get() + 1);
        }
    });

    assert_eq!(counter.get(), 1);

    token_value.set("token2".to_string());
    assert_eq!(counter.get(), 1);
}

#[test]
fn on_after_transaction_fires_after_set() {
    let fires = Rc::new(Cell::new(0));
    let _hook = super::on_after_transaction({
        let fires = fires.clone();
        move || {
            fires.set(fires.get() + 1);
        }
    });

    let a = Value::new(1);
    a.set(2);
    assert_eq!(fires.get(), 1);
}
