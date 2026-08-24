use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use super::{Computed, DropResource, Graph, Value};

#[test]
fn basic_sum() {
    let g = Graph::new();
    let logs = g.logger().listen();
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
    logs.assert_eq(&[]);
}

#[test]
fn two_sets_in_one_transaction_compute_once() {
    let g = Graph::new();
    let logs = g.logger().listen();
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
    logs.assert_eq(&[]);
}

#[test]
fn cutoff_leaves_fanout_untouched() {
    let g = Graph::new();
    let logs = g.logger().listen();
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
    logs.assert_eq(&[]);
}

#[test]
fn diamond_waits_for_both_parents() {
    let g = Graph::new();
    let logs = g.logger().listen();
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
    logs.assert_eq(&[]);
}

#[test]
fn subscribe_skips_when_parent_cuts_off() {
    let g = Graph::new();
    let logs = g.logger().listen();
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
    logs.assert_eq(&[]);
}

#[test]
fn equal_set_does_not_enqueue() {
    let g = Graph::new();
    let logs = g.logger().listen();
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
    logs.assert_eq(&[]);
}

#[test]
fn default_graph_value_new() {
    let logs = super::default_graph().logger().listen();
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
    logs.assert_eq(&[]);
}

#[test]
fn when_connect_tracks_watchers() {
    let logs = super::default_graph().logger().listen();
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
    logs.assert_eq(&[]);
}

#[test]
fn when_connect_multiple_subscribers_share_one_connection() {
    let logs = super::default_graph().logger().listen();
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
    logs.assert_eq(&[]);
}

#[test]
fn when_connect_disconnects_while_computed_lives() {
    let logs = super::default_graph().logger().listen();
    let connect = Rc::new(Cell::new(0));
    let disconnect = Rc::new(Cell::new(0));
    let value = Value::new(1);
    let comp = value.to_computed().when_connect({
        let connect = connect.clone();
        let disconnect = disconnect.clone();
        move || {
            connect.set(1);
            DropResource::new({
                let disconnect = disconnect.clone();
                move || disconnect.set(1)
            })
        }
    });
    let keep = comp.clone();
    let sub = comp.subscribe(|_| {});
    assert_eq!(connect.get(), 1);
    drop(sub);
    assert_eq!(disconnect.get(), 1);
    drop(keep);
    logs.assert_eq(&[]);
}

#[test]
fn when_connect_runs_after_on_after_transaction() {
    let g = Graph::new();
    let logs = g.logger().listen();
    let order = Rc::new(RefCell::new(Vec::new()));
    let _hook = g.on_after_transaction({
        let order = order.clone();
        move || order.borrow_mut().push("hook")
    });
    let value = g.value(1);
    let comp = value.to_computed().when_connect({
        let order = order.clone();
        move || {
            order.borrow_mut().push("connect");
            DropResource::new(|| {})
        }
    });
    let _sub = comp.subscribe({
        let order = order.clone();
        move |_| order.borrow_mut().push("subscribe")
    });
    assert_eq!(*order.borrow(), ["subscribe", "hook", "connect"]);
    logs.assert_eq(&[]);
}

/// A `when_connect` closure may write, and that write runs a whole wave before the
/// closure returns. If the wave takes the last child away from the node that is
/// connecting, the resource it returns must be dropped - the node is no longer watched,
/// so nothing is left to keep the external work alive.
#[test]
fn connect_that_unwatches_itself_disconnects() {
    let g = Graph::new();
    let logs = g.logger().listen();
    let connects = Rc::new(Cell::new(0));
    let disconnects = Rc::new(Cell::new(0));

    let flag = g.value(true);
    let source = g.value(1);

    let connected = source.to_computed().when_connect({
        let connects = connects.clone();
        let disconnects = disconnects.clone();
        let flag = flag.clone();
        move || {
            connects.set(connects.get() + 1);
            // From `create` after the graph has settled: costs `connected` its only child.
            flag.set(false);
            DropResource::new({
                let disconnects = disconnects.clone();
                move || disconnects.set(disconnects.get() + 1)
            })
        }
    });

    let reader = g.computed({
        let flag = flag.clone();
        let connected = connected.clone();
        move |ctx| {
            if flag.get(ctx) { connected.get(ctx) } else { 0 }
        }
    });
    let _sub = reader.subscribe(|_| {});

    assert_eq!(connects.get(), 1);
    assert_eq!(
        disconnects.get(),
        1,
        "unwatched, so it must not stay connected"
    );

    // And it stays disconnected: no later wave revives it.
    source.set(2);
    assert_eq!(connects.get(), 1);
    assert_eq!(disconnects.get(), 1);
    logs.assert_eq(&[]);
}

#[test]
fn watch_and_unwatch_in_one_transaction_does_not_connect() {
    let g = Graph::new();
    let logs = g.logger().listen();
    let connects = Rc::new(Cell::new(0));
    let disconnects = Rc::new(Cell::new(0));
    let value = g.value(1);
    let comp = value.to_computed().when_connect({
        let connects = connects.clone();
        let disconnects = disconnects.clone();
        move || {
            connects.set(connects.get() + 1);
            DropResource::new({
                let disconnects = disconnects.clone();
                move || disconnects.set(disconnects.get() + 1)
            })
        }
    });
    g.transaction(|_| {
        let sub = comp.subscribe(|_| {});
        drop(sub);
    });
    assert_eq!(connects.get(), 0);
    assert_eq!(disconnects.get(), 0);
    logs.assert_eq(&[]);
}

#[test]
fn nested_computed_subscription() {
    let logs = super::default_graph().logger().listen();
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
    logs.assert_eq(&[]);
}

#[test]
fn nested_computed_subscription_no_flattening() {
    let logs = super::default_graph().logger().listen();
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
    logs.assert_eq(&[]);
}

#[test]
fn on_after_transaction_fires_after_set() {
    let logs = super::default_graph().logger().listen();
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
    logs.assert_eq(&[]);
}

#[test]
fn set_from_subscribe_is_ignored() {
    let g = Graph::new();
    let logs = g.logger().listen();
    let a = g.value(0);
    let b = g.value(0);
    let _sub = a.to_computed().subscribe({
        let b = b.clone();
        move |_| b.set(1)
    });
    logs.assert_eq(&[super::graph::BLOCKED_WRITE]);

    g.transaction(|ctx| {
        assert_eq!(b.get(ctx), 0);
    });
    logs.assert_eq(&[]);

    a.set(2);
    logs.assert_eq(&[super::graph::BLOCKED_WRITE]);

    g.transaction(|ctx| {
        assert_eq!(b.get(ctx), 0);
    });
    logs.assert_eq(&[]);
}

#[test]
fn set_from_compute_is_ignored() {
    let g = Graph::new();
    let logs = g.logger().listen();
    let a = g.value(0);
    let b = g.value(0);
    let c = g.computed({
        let a = a.clone();
        let b = b.clone();
        move |ctx| {
            b.set(a.get(ctx));
            a.get(ctx)
        }
    });
    logs.assert_eq(&[]);

    g.transaction(|ctx| {
        assert_eq!(c.get(ctx), 0);
        assert_eq!(b.get(ctx), 0);
    });
    logs.assert_eq(&[super::graph::BLOCKED_WRITE]);
    logs.assert_eq(&[]);
}

/// `when_connect` runs after the wave, so a write from `create` is a new transaction
/// and must reach the subscriber too, not just land in the value.
#[test]
fn write_from_when_connect_reaches_the_subscriber() {
    let g = Graph::new();
    let logs = g.logger().listen();

    let source = g.value(0);
    let observed = g
        .computed({
            let source = source.clone();
            move |ctx| source.get(ctx)
        })
        .when_connect({
            let source = source.clone();
            move || {
                source.set(7);
                DropResource::new(|| {})
            }
        });

    let seen = Rc::new(RefCell::new(Vec::new()));
    let _sub = observed.subscribe({
        let seen = seen.clone();
        move |value| seen.borrow_mut().push(value)
    });

    assert_eq!(*seen.borrow(), vec![0, 7]);
    logs.assert_eq(&[]);
}

/// Disconnect must not write. Connect takes the node's last child away; dropping the
/// resource tries to give it back and is ignored, so the node stays disconnected.
#[test]
fn disconnect_must_not_write() {
    let g = Graph::new();
    let logs = g.logger().listen();
    let connects = Rc::new(Cell::new(0));
    let flag = g.value(true);
    let source = g.value(1);

    let connected = source.to_computed().when_connect({
        let connects = connects.clone();
        let flag = flag.clone();
        move || {
            connects.set(connects.get() + 1);
            flag.set(false);
            DropResource::new({
                let flag = flag.clone();
                move || flag.set(true)
            })
        }
    });

    let reader = g.computed({
        let flag = flag.clone();
        let connected = connected.clone();
        move |ctx| {
            if flag.get(ctx) { connected.get(ctx) } else { 0 }
        }
    });
    let _sub = reader.subscribe(|_| {});

    assert_eq!(connects.get(), 1);
    logs.assert_eq(&[super::graph::BLOCKED_WRITE]);
    g.transaction(|ctx| assert!(!flag.get(ctx)));
}

/// Dropping something from inside a subscribe callback - a component going away while the
/// view is rebuilt - runs that `Drop` inside the callback, so its writes are refused like
/// any other write from there.
#[test]
fn a_write_from_a_drop_inside_a_callback_is_ignored() {
    let g = Graph::new();
    let logs = g.logger().listen();
    let cleared = g.value(false);
    let source = g.value(0);

    let resource = Rc::new(RefCell::new(Some(DropResource::new({
        let cleared = cleared.clone();
        move || cleared.set(true)
    }))));

    let _sub = source.to_computed().subscribe({
        let resource = resource.clone();
        move |value| {
            if value == 1 {
                resource.borrow_mut().take();
            }
        }
    });
    logs.assert_eq(&[]);

    source.set(1);

    logs.assert_eq(&[super::graph::BLOCKED_WRITE]);
    assert!(!g.transaction(|ctx| cleared.get(ctx)));
}

#[test]
fn set_from_drop_resource_is_ignored() {
    let g = Graph::new();
    let logs = g.logger().listen();
    let a = g.value(0);
    let resource = DropResource::new({
        let a = a.clone();
        move || a.set(1)
    });
    drop(resource);
    logs.assert_eq(&[super::graph::BLOCKED_WRITE]);
    g.transaction(|ctx| assert_eq!(a.get(ctx), 0));
}

/// A compute closure that reads its own value closes a cycle. It is caught by the read,
/// while the path is still on the refresh stack, so the panic can name it.
#[test]
#[should_panic(expected = "NodeId(2) -> NodeId(2)")]
fn a_computed_reading_itself_panics() {
    let g = Graph::new();
    let source = g.value(1i32);
    let holder: Rc<RefCell<Option<Computed<i32>>>> = Rc::new(RefCell::new(None));

    let c = g.computed({
        let source = source.clone();
        let holder = holder.clone();
        move |ctx| {
            let base = source.get(ctx);
            // The first run must not close the cycle - a `Computed` that reads itself
            // before it ever had a value just recurses in `ensure`.
            if base < 5 {
                return base;
            }
            match holder.borrow().clone() {
                Some(myself) => base + myself.get(ctx),
                None => base,
            }
        }
    });
    *holder.borrow_mut() = Some(c.clone());
    let _sub = c.subscribe(|_| {});

    source.set(5);
}

/// Two computeds that start reading each other mid-wave. The path names both.
#[test]
#[should_panic(expected = "cycle in the reactive graph")]
fn two_computeds_reading_each_other_panic() {
    let g = Graph::new();
    let flag = g.value(false);
    let hold_a: Rc<RefCell<Option<Computed<i32>>>> = Rc::new(RefCell::new(None));
    let hold_b: Rc<RefCell<Option<Computed<i32>>>> = Rc::new(RefCell::new(None));

    let a = g.computed({
        let flag = flag.clone();
        let hold_b = hold_b.clone();
        move |ctx| match (flag.get(ctx), hold_b.borrow().clone()) {
            (true, Some(other)) => other.get(ctx) + 1,
            _ => 1,
        }
    });
    let b = g.computed({
        let flag = flag.clone();
        let hold_a = hold_a.clone();
        move |ctx| match (flag.get(ctx), hold_a.borrow().clone()) {
            (true, Some(other)) => other.get(ctx) + 1,
            _ => 2,
        }
    });
    *hold_a.borrow_mut() = Some(a.clone());
    *hold_b.borrow_mut() = Some(b.clone());
    let _sa = a.subscribe(|_| {});
    let _sb = b.subscribe(|_| {});

    flag.set(true);
}

/// Two `when_connect` closures that write can hand the connection back and forth: the
/// first takes its own reader away and gives it to the second, and the second does the
/// reverse. Neither write is illegal - `create` is a legal place to write - and there is
/// no state this settles in, so the flush has to end it.
///
/// One connect per node per flush is what ends it. `second` connects, its write hands the
/// reader back to `first`, and `first` has already had its turn in this flush, so it is
/// left disconnected and the graph says which node that was. Which of the two loses
/// depends on the order a round is processed in, and that order is by node id, so the
/// outcome is the same on every run.
///
/// The counter is a safety valve, not the thing under test: past it the closures stop
/// writing, so a graph that cannot end the loop fails an assert instead of hanging.
#[test]
fn two_connects_that_hand_the_connection_back_and_forth_are_cut() {
    const VALVE: u32 = 1000;

    let g = Graph::new();
    let logs = g.logger().listen();
    let connects = Rc::new(Cell::new(0));
    let flag = g.value(true);
    let source = g.value(1);

    let connect = |wants: bool| {
        let connects = connects.clone();
        let flag = flag.clone();
        move || {
            connects.set(connects.get() + 1);
            if connects.get() < VALVE {
                flag.set(wants);
            }
            DropResource::new(|| {})
        }
    };

    let first = source.to_computed().when_connect(connect(false));
    let second = source.to_computed().when_connect(connect(true));

    let reader = g.computed({
        let flag = flag.clone();
        let first = first.clone();
        let second = second.clone();
        move |ctx| {
            if flag.get(ctx) {
                first.get(ctx)
            } else {
                second.get(ctx)
            }
        }
    });
    let _sub = reader.subscribe(|_| {});

    assert!(
        connects.get() < VALVE,
        "connected {} times - the flush only stopped because the test refused to keep writing",
        connects.get()
    );
    assert_eq!(
        connects.get(),
        2,
        "two nodes, one connect each - a flush connects a node once"
    );

    let messages = logs.take();
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert!(
        messages[0].starts_with(super::graph::CONNECT_LOOP),
        "{messages:?}"
    );
}
