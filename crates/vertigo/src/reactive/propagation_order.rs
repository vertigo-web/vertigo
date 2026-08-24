//! Ordering inside one propagation pass.
//!
//! A node refreshes at most once per wave. Children are queued only when a parent
//! changed (equality cutoff). `get` pulls a stale ancestor so a join still sees every
//! branch, and a subscriber sees one value: the one that matches the sources.

use super::Graph;
use crate::struct_mut::ValueMut;
use std::rc::Rc;

/// Static dependency set, no conditionals: `c` always reads both branches. The two paths
/// from the two roots have different lengths.
#[test]
fn static_unequal_path_lengths_compute_once() {
    let g = Graph::new();
    let logs = g.logger().listen();
    let d1 = g.value(1i32);
    let d2 = g.value(1i32);

    // short path: d1 -> b1
    let b1 = g.computed({
        let d1 = d1.clone();
        move |ctx| d1.get(ctx)
    });

    // long path: d2 -> e2 -> f2 -> b2
    let e2 = g.computed({
        let d2 = d2.clone();
        move |ctx| d2.get(ctx)
    });
    let f2 = g.computed({
        let e2 = e2.clone();
        move |ctx| e2.get(ctx)
    });
    let b2 = g.computed({
        let f2 = f2.clone();
        move |ctx| f2.get(ctx)
    });

    let runs = Rc::new(ValueMut::new(0));
    let c = g.computed({
        let b1 = b1.clone();
        let b2 = b2.clone();
        let runs = runs.clone();
        move |ctx| {
            runs.change(|n| *n += 1);
            b1.get(ctx) * 1000 + b2.get(ctx)
        }
    });

    let seen = Rc::new(ValueMut::new(Vec::new()));
    let _sub = c.subscribe({
        let seen = seen.clone();
        move |value| seen.change(|seen| seen.push(value))
    });

    runs.set(0);
    seen.set(Vec::new());

    g.transaction(|_| {
        d1.set(7);
        d2.set(9);
    });

    assert_eq!(runs.get(), 1);
    assert_eq!(seen.get(), vec![7009]);
    logs.assert_eq(&[]);
}

/// A conditional read discovers parents that were not in the previous parent set.
/// `get` still pulls them, so `c` runs once with both branches already fresh.
#[test]
fn dynamic_branch_switch_computes_once() {
    let g = Graph::new();
    let logs = g.logger().listen();
    let flag = g.value(true);
    let d1 = g.value(1i32);
    let d2 = g.value(1i32);

    // d1 -> e1 -> b1
    let e1 = g.computed({
        let d1 = d1.clone();
        move |ctx| d1.get(ctx)
    });
    let b1 = g.computed({
        let e1 = e1.clone();
        move |ctx| e1.get(ctx)
    });

    // d2 -> e2 -> f2 -> b2, one hop longer so it settles later
    let e2 = g.computed({
        let d2 = d2.clone();
        move |ctx| d2.get(ctx)
    });
    let f2 = g.computed({
        let e2 = e2.clone();
        move |ctx| e2.get(ctx)
    });
    let b2 = g.computed({
        let f2 = f2.clone();
        move |ctx| f2.get(ctx)
    });

    // Both branches observed, so each holds a cached value and has to refresh in the pass.
    let _keep_b1 = b1.clone().subscribe(|_| {});
    let _keep_b2 = b2.clone().subscribe(|_| {});

    let runs = Rc::new(ValueMut::new(0));
    let c = g.computed({
        let flag = flag.clone();
        let b1 = b1.clone();
        let b2 = b2.clone();
        let runs = runs.clone();
        move |ctx| {
            runs.change(|n| *n += 1);
            if flag.get(ctx) {
                0
            } else {
                let first = b1.get(ctx);
                if first >= 100 {
                    first + b2.get(ctx)
                } else {
                    first
                }
            }
        }
    });

    let seen = Rc::new(ValueMut::new(Vec::new()));
    let _sub = c.subscribe({
        let seen = seen.clone();
        move |value| seen.change(|seen| seen.push(value))
    });

    let pass = |apply: &dyn Fn()| -> (usize, Vec<i32>) {
        runs.set(0);
        seen.set(Vec::new());
        g.transaction(|_| apply());
        (runs.get(), seen.get())
    };

    let (runs_1, seen_1) = pass(&|| {
        d1.set(500);
        d2.set(7);
        flag.set(false);
    });
    assert_eq!(runs_1, 1);
    assert_eq!(seen_1, vec![507]);

    let (runs_2, seen_2) = pass(&|| {
        d1.set(900);
        d2.set(8);
    });
    assert_eq!(runs_2, 1);
    assert_eq!(seen_2, vec![908]);

    let (runs_3, seen_3) = pass(&|| flag.set(true));
    assert_eq!(runs_3, 1);
    assert_eq!(seen_3, vec![0]);

    let (runs_4, seen_4) = pass(&|| {
        d1.set(1500);
        d2.set(9);
        flag.set(false);
    });
    assert_eq!(runs_4, 1);
    assert_eq!(seen_4, vec![1509]);
    logs.assert_eq(&[]);
}

/// Build a fan-in: `paths` chains of *different* lengths, all rooted in values written in
/// one transaction, all read by one node `c`. Chain `k` is `k + 1` hops long.
///
/// No node here depends on itself, nothing writes from a callback: the graph is a plain
/// acyclic fan-in.
///
/// Returns how many times `c` computed, and the last value its subscriber saw.
fn fan_in(paths: usize) -> (u32, Option<i32>) {
    let g = Graph::new();
    let logs = g.logger().listen();
    let roots = (0..paths).map(|_| g.value(0i32)).collect::<Vec<_>>();

    let mut tails = Vec::new();
    for (k, root) in roots.iter().enumerate() {
        let mut node = g.computed({
            let root = root.clone();
            move |ctx| root.get(ctx)
        });
        for _ in 0..k {
            node = g.computed({
                let prev = node.clone();
                move |ctx| prev.get(ctx)
            });
        }
        tails.push(node);
    }

    let runs = Rc::new(ValueMut::new(0u32));
    let c = g.computed({
        let tails = tails.clone();
        let runs = runs.clone();
        move |ctx| {
            runs.change(|n| *n += 1);
            tails.iter().map(|tail| tail.get(ctx)).sum::<i32>()
        }
    });

    let last = Rc::new(ValueMut::new(None));
    let _sub = c.subscribe({
        let last = last.clone();
        move |value| last.set(Some(value))
    });

    runs.set(0);
    g.transaction(|_| {
        for (i, root) in roots.iter().enumerate() {
            root.set(i as i32 + 1);
        }
    });

    logs.assert_eq(&[]);
    (runs.get(), last.get())
}

/// However many paths of differing length reach `c`, it computes once and the value it
/// settles on accounts for all of them. 101 paths is past any per-node budget the graph
/// has ever had, so a cut-off would show up here as a missing contribution.
#[test]
fn one_run_regardless_of_incoming_paths() {
    for paths in [2usize, 3, 5, 10, 101] {
        let expected_sum = (paths * (paths + 1) / 2) as i32;
        let (runs, last) = fan_in(paths);

        assert_eq!(runs, 1, "{paths} paths");
        assert_eq!(last, Some(expected_sum), "{paths} paths");
    }
}

/// Build a lattice `depth` levels tall: every level reads both nodes of the level below,
/// so the number of *paths* from the base to the top doubles per level while the number
/// of *nodes* only grows by two.
///
/// Nothing in the lattice changes; a separate trigger starts the wave, and the probe that
/// reads the top has to confirm the whole lattice is fresh. Walking it once per path
/// instead of once per node is the difference between linear and exponential.
fn lattice_walks(depth: usize) -> u64 {
    let g = Graph::new();
    let base = g.value(1i32);
    let mut left = g.computed({
        let base = base.clone();
        move |ctx| base.get(ctx)
    });
    let mut right = g.computed({
        let base = base.clone();
        move |ctx| base.get(ctx) + 1
    });
    for _ in 0..depth {
        let next_left = g.computed({
            let (left, right) = (left.clone(), right.clone());
            move |ctx| left.get(ctx) + right.get(ctx)
        });
        let next_right = g.computed({
            let (left, right) = (left.clone(), right.clone());
            move |ctx| left.get(ctx) * 2 + right.get(ctx)
        });
        left = next_left;
        right = next_right;
    }
    let top = g.computed({
        let (left, right) = (left.clone(), right.clone());
        move |ctx| left.get(ctx) + right.get(ctx)
    });

    let trigger = g.value(0i32);
    let probe = g.computed({
        let trigger = trigger.clone();
        let top = top.clone();
        move |ctx| trigger.get(ctx) + top.get(ctx)
    });
    let _sub = probe.subscribe(|_| {});

    g.take_parent_walks();
    trigger.set(1);
    g.take_parent_walks()
}

/// One walk per node of the lattice, not per path through it.
#[test]
fn a_clean_lattice_is_walked_once_per_node() {
    for depth in [2usize, 4, 8, 16] {
        // `top`, two nodes per level, and the two at level zero. The base value is not
        // walked: it has no parents, so `ensure_fresh` settles it from the edge map.
        let nodes = 1 + 2 * depth + 2;

        assert_eq!(lattice_walks(depth) as usize, nodes, "depth {depth}");
    }
}

/// Outside a wave every node is already settled, so a read walks nothing at all.
#[test]
fn a_read_outside_a_wave_walks_nothing() {
    let g = Graph::new();
    let source = g.value(1i32);
    let doubled = g.computed({
        let source = source.clone();
        move |ctx| source.get(ctx) * 2
    });
    let _sub = doubled.clone().subscribe(|_| {});

    source.set(2);
    g.take_parent_walks();

    g.transaction(|ctx| {
        assert_eq!(doubled.get(ctx), 4);
        assert_eq!(source.get(ctx), 2);
    });

    assert_eq!(g.take_parent_walks(), 0);
}
