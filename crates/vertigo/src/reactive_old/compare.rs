//! Wall-clock and work-count comparison: previous graph (`reactive_old`) vs current (`reactive`).
//!
//! Run with: `cargo test -p vertigo --lib reactive_old::compare -- --nocapture`

use std::{
    cell::Cell,
    rc::Rc,
    time::{Duration, Instant},
};

use crate::reactive::{self as new, Graph};
use crate::reactive_old as old;

const FANOUT: usize = 10_000;
const CHAIN: usize = 200;
const UPDATES: usize = 200;

fn elapsed(f: impl FnOnce()) -> Duration {
    let start = Instant::now();
    f();
    start.elapsed()
}

fn report(name: &str, old_d: Duration, new_d: Duration) {
    let old_ms = old_d.as_secs_f64() * 1000.0;
    let new_ms = new_d.as_secs_f64() * 1000.0;
    let speedup = if new_ms <= 0.0 {
        f64::INFINITY
    } else {
        old_ms / new_ms
    };
    eprintln!("compare {name:>28}: old={old_ms:8.3}ms  new={new_ms:8.3}ms  speedup={speedup:6.2}x");
}

struct FanoutOld {
    source: old::Value<i32>,
    _children: Vec<old::Computed<(bool, usize)>>,
    _subs: Vec<crate::DropResource>,
    runs: Rc<Cell<usize>>,
}

fn setup_fanout_old() -> FanoutOld {
    let source = old::Value::new(1);
    let even = old::Computed::from({
        let source = source.clone();
        move |ctx| source.get(ctx) % 2 == 0
    });
    let runs = Rc::new(Cell::new(0));
    let children: Vec<_> = (0..FANOUT)
        .map(|i| {
            let even = even.clone();
            let runs = runs.clone();
            old::Computed::from(move |ctx| {
                runs.set(runs.get() + 1);
                (even.get(ctx), i)
            })
        })
        .collect();
    let subs = children
        .iter()
        .cloned()
        .map(|child| child.subscribe(|_| {}))
        .collect();
    FanoutOld {
        source,
        _children: children,
        _subs: subs,
        runs,
    }
}

struct FanoutNew {
    source: new::Value<i32>,
    _children: Vec<new::Computed<(bool, usize)>>,
    _subs: Vec<crate::DropResource>,
    runs: Rc<Cell<usize>>,
}

fn setup_fanout_new() -> FanoutNew {
    let g = Graph::new();
    let source = g.value(1);
    let even = g.computed({
        let source = source.clone();
        move |ctx| source.get(ctx) % 2 == 0
    });
    let runs = Rc::new(Cell::new(0));
    let children: Vec<_> = (0..FANOUT)
        .map(|i| {
            let even = even.clone();
            let runs = runs.clone();
            g.computed(move |ctx| {
                runs.set(runs.get() + 1);
                (even.get(ctx), i)
            })
        })
        .collect();
    let subs = children
        .iter()
        .cloned()
        .map(|child| child.subscribe(|_| {}))
        .collect();
    FanoutNew {
        source,
        _children: children,
        _subs: subs,
        runs,
    }
}

#[test]
fn cutoff_fanout_skips_unchanged_children() {
    let old_g = setup_fanout_old();
    let new_g = setup_fanout_new();

    old_g.runs.set(0);
    new_g.runs.set(0);

    let old_d = elapsed(|| old_g.source.set(3));
    let new_d = elapsed(|| new_g.source.set(3));

    report("cutoff 10k (odd→odd)", old_d, new_d);

    assert_eq!(
        new_g.runs.get(),
        0,
        "new graph must cut off when even/odd is unchanged"
    );
    assert_eq!(
        old_g.runs.get(),
        FANOUT,
        "old graph invalidates the whole fan-out"
    );
}

#[test]
fn fanout_runs_when_parity_changes() {
    let old_g = setup_fanout_old();
    let new_g = setup_fanout_new();

    old_g.runs.set(0);
    new_g.runs.set(0);

    let old_d = elapsed(|| old_g.source.set(2));
    let new_d = elapsed(|| new_g.source.set(2));

    report("fanout 10k (odd→even)", old_d, new_d);

    assert_eq!(old_g.runs.get(), FANOUT);
    assert_eq!(new_g.runs.get(), FANOUT);
}

fn setup_chain_old() -> (old::Value<i32>, Vec<crate::DropResource>) {
    let source = old::Value::new(0);
    let mut prev: old::Computed<i32> = source.to_computed();
    let mut subs = Vec::new();
    for _ in 0..CHAIN {
        let next = old::Computed::from({
            let prev = prev.clone();
            move |ctx| prev.get(ctx) + 1
        });
        subs.push(next.clone().subscribe(|_| {}));
        prev = next;
    }
    (source, subs)
}

fn setup_chain_new() -> (new::Value<i32>, Vec<crate::DropResource>) {
    let g = Graph::new();
    let source = g.value(0);
    let mut prev: new::Computed<i32> = source.to_computed();
    let mut subs = Vec::new();
    for _ in 0..CHAIN {
        let next = g.computed({
            let prev = prev.clone();
            move |ctx| prev.get(ctx) + 1
        });
        subs.push(next.clone().subscribe(|_| {}));
        prev = next;
    }
    (source, subs)
}

#[test]
fn deep_chain_updates() {
    let (old_src, _old_subs) = setup_chain_old();
    let (new_src, _new_subs) = setup_chain_new();

    // Warm the caches.
    old_src.set(1);
    new_src.set(1);

    let old_d = elapsed(|| {
        for i in 2..(2 + UPDATES as i32) {
            old_src.set(i);
        }
    });
    let new_d = elapsed(|| {
        for i in 2..(2 + UPDATES as i32) {
            new_src.set(i);
        }
    });

    report(&format!("chain {CHAIN} x {UPDATES} sets"), old_d, new_d);
}

fn setup_diamond_old() -> (old::Value<i32>, crate::DropResource) {
    let a = old::Value::new(1);
    let left = old::Computed::from({
        let a = a.clone();
        move |ctx| a.get(ctx) + 1
    });
    let right = old::Computed::from({
        let a = a.clone();
        move |ctx| a.get(ctx) * 10
    });
    let d = old::Computed::from({
        let left = left.clone();
        let right = right.clone();
        move |ctx| left.get(ctx) + right.get(ctx)
    });
    let sub = d.subscribe(|_| {});
    (a, sub)
}

fn setup_diamond_new() -> (new::Value<i32>, crate::DropResource) {
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
    let d = g.computed({
        let left = left.clone();
        let right = right.clone();
        move |ctx| left.get(ctx) + right.get(ctx)
    });
    let sub = d.subscribe(|_| {});
    (a, sub)
}

#[test]
fn diamond_repeated_updates() {
    let (old_a, _old_sub) = setup_diamond_old();
    let (new_a, _new_sub) = setup_diamond_new();

    old_a.set(2);
    new_a.set(2);

    let old_d = elapsed(|| {
        for i in 3..(3 + UPDATES as i32) {
            old_a.set(i);
        }
    });
    let new_d = elapsed(|| {
        for i in 3..(3 + UPDATES as i32) {
            new_a.set(i);
        }
    });

    report(&format!("diamond x {UPDATES} sets"), old_d, new_d);
}
