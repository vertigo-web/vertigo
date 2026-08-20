# The reactive graph

How `Value`, `Computed` and `subscribe` fit together, and what happens between a write and
the resulting recomputation.

## The model

Three kinds of node:

| node | created by | holds |
| ---- | ---------- | ----- |
| value | [`Value::new`](crate::Value::new) | a value you write |
| computed | [`Computed::from`](crate::Computed::from), `map`, `to_computed` | a cached value derived from other nodes |
| subscription | [`Computed::subscribe`](crate::Computed::subscribe) | a callback, no value |

Edges are never declared. They are recorded while a node computes: reading a node through
[`Value::get`](crate::Value::get) / [`Computed::get`](crate::Computed::get) registers the
node that was read as a parent of the node doing the reading. Each run replaces the whole
parent set, so a computed that stops reading something stops depending on it.

A child holds a strong reference to its parents, so a parent cannot be dropped while
something still reads it. The graph itself only holds weak references, so dropping the last
`Value` / `Computed` handle removes the node.

## Transactions

A write is applied immediately, but nothing downstream runs until the outermost transaction
closes:

```rust,ignore
transaction(|_| {
    first_name.set("Ada".to_string());
    last_name.set("Lovelace".to_string());
});
// one propagation pass here, not two
```

Transactions nest; only the outermost one propagates. `Value::set` outside a transaction is
its own one-write transaction, which is why a lone `set` still works.

After propagation finishes, the callbacks registered with
[`on_after_transaction`](crate::reactive::on_after_transaction) run. The DOM driver uses
one of those to flush its batched commands.

## One propagation pass

The writes leave a set of dirty nodes. The pass repeatedly takes a node whose parents are
all up to date, recomputes it, and then decides whether to carry on:

```text
value written ──> node recomputed ──> value changed? ──yes──> dependents queued
                                            │
                                            no
                                            │
                                            └──> stop
```

That last step is the **equality cutoff**. A node's dependents are queued only when its new
value differs (`PartialEq`) from the old one. A change that computes back to the same value
therefore stops where it is instead of travelling through the graph. This is why `Computed`
requires `T: PartialEq`.

Queue order alone would not be enough: two paths of different lengths reach a join at
different points of the pass. So a `get` that finds a stale parent refreshes it there and
then, before returning. A node sees only fresh parents, computes at most once per pass, and
a subscriber never observes a value assembled from a half-updated graph.

A node that ends up reading its own value - directly, or around a cycle - cannot be made
fresh, so the pass panics and names the path it went round.

The cutoff is what makes wide fan-out cheap:

```rust,ignore
let is_even = number.map(|n| n % 2 == 0);
// thousands of nodes reading `is_even`

number.set(3);  // was 1: `is_even` recomputes, stays false, nothing downstream runs
number.set(4);  // now it flips, and the fan-out runs
```

A subscription never has dependents, so its callback is a leaf of the pass.

## Laziness

A `Computed` does not compute when it is created. It computes on the first read, and after
that it is kept up to date by propagation for as long as something references it. A compute
closure therefore runs on a schedule you do not control - it must be a pure function of the
nodes it reads.

## Writing from a callback

`Value::set` from a compute closure or a `subscribe` callback is forbidden. The write is
ignored and logged. Those closures run during a wave (or the first refresh of a
subscription); feeding values back into the graph would re-enter the wave. Domain rules:
[`invariants`](crate::reactive::invariants).

```rust,ignore
selected_id.to_computed().subscribe(move |id| {
    // ignored: subscribe must not write
    form_dirty.set(false);
});
```

The rule follows the call stack, not the source: anything dropped while a callback runs -
a component going away as the view is rebuilt - has its `Drop` run inside that callback,
so a `set` from there is ignored as well. Clearing a value on unmount belongs in a
`when_connect` resource, which is dropped after the wave.

Legal places to write: DOM/event handlers, timers, fetch/socket callbacks,
`on_after_transaction`, and `when_connect` / `Value::with_connect`.

## Connecting to the outside world

[`when_connect`](crate::Computed::when_connect) runs a closure **after the wave** in which
a node gains its first dependent, and drops the returned
[`DropResource`](crate::DropResource) after the wave in which it loses its last one.
Watched-then-unwatched in the same wave is a no-op. This is how a node backed by a fetch,
a timer or a socket only does work while something is actually looking at it.
[`Value::with_connect`](crate::Value::with_connect) packages the common shape of it;
because `create` runs after the wave, it may write the `Value`.

That write can change who is watched, which is what lets one connect pull in the next. It
also means two of them can undo each other - a connect that takes away its own node's last
child, whose disconnect gives it back. That never settles, so after 100 connects of one
node in a single flush the loop is cut: the error is logged and the node is left
disconnected. A chain, however long, connects each node once and is never cut.

## Isolated graphs

`Value::new` and `Computed::from` use one graph per thread. `Graph::new()` creates a
separate one; nodes belonging to different graphs never see each other. Tests use this to
avoid sharing state.

## Coming from 0.12

* The old `Dependencies` type is gone; use [`transaction`](crate::transaction),
  [`Driver::transaction`](crate::Driver::transaction) and
  [`Driver::on_after_transaction`](crate::Driver::on_after_transaction).
* `Computed<T>` now requires `T: PartialEq` everywhere, not only for `subscribe`.
* `subscribe_all` is gone. It reported every recomputation including the ones that produced
  the same value, and those no longer notify anybody.
* Writing a `Value` from a compute or subscribe callback is ignored and logged (same idea as
  0.12's *"You cannot change the source value while the dependency graph is being refreshed"*).
  `when_connect` / `Value::with_connect` run after the wave and may write.
* Reading through `transaction(|ctx| ...)` serves the cached value instead of recomputing
  the chain behind it.
