# Reactive graph — invariants

One `Graph` owns a set of `Value`, `Computed`, and subscription nodes.
Nodes from different graphs do not see each other.

A **wave** is one run of `propagate`. A **transaction** batches writes.
**Cutoff** means we skip dependents when a value did not change.
**Connect** / **disconnect** start and stop external work (`when_connect`).

## How a write runs

1. `set` or `transaction` writes values and marks them dirty.
2. The outermost transaction starts a wave.
3. The wave refreshes ready nodes. Unchanged nodes do not dirty their children.
4. After the wave: connect and disconnect.
5. Then `on_after_transaction` hooks.

A `set` from `when_connect` starts this again.
A `set` from compute or subscribe is logged and ignored. It never writes.

## Invariants

### 1. Compute and subscribe must not write

Do not call `Value::set` (or `change`) from:

- a compute closure
- a subscribe callback
- a wave that is already running

Compute only reads. Subscribe only talks to the outside world (DOM, logs).
Neither may write back into the graph.

If they do, the write is ignored and the console gets:

```text
vertigo: Value::set is not allowed from a computed, a subscribe callback, or during propagation
```

This covers anything that runs *inside* those closures, not only the code you wrote there.
Dropping something from a subscribe callback — a component going away while the view is
rebuilt — runs its `Drop` inside the callback, so a `set` from there is ignored too.
To clear a value on unmount, do it from a `when_connect` resource: those are dropped after
the wave.

You may write from click/input handlers, timers, fetch, sockets,
`on_after_transaction`, and `when_connect` / `Value::with_connect`.

### 2. Connect and disconnect wait until the wave is done

`when_connect` does not run the moment a node gets a child.
Disconnect does not run the moment it loses the last child.
Both wait until the wave ends.
If the graph is idle, they run at once.

If a node is watched and then unwatched in the same wave, nothing happens.
If it is unwatched and then watched, it connects once.

`when_connect` runs after the wave, so `Value::with_connect` may call `set`.
That `set` is a new transaction and a new wave.

That wave can change who is watched, including the node that is connecting right now.
The connect state is matched to the graph again once the closure returns.
A node unwatched by its own connect is disconnected. It never stays connected.

Connect and disconnect must not undo each other.
A connect that unwatches its own node, whose disconnect watches it again, never ends.
After 100 connects of one node in one flush, the loop is cut:
an error is logged and that node is left disconnected.
A chain — one connect watching the next node — is not a loop and is never cut.

### 3. Only the outermost transaction starts a wave

`Value::set` is a transaction.
A `transaction` inside another `transaction` only writes and marks dirty.
The wave starts when the outer call returns.

### 4. Unchanged values stop the update

After a refresh, children run only if the new value is different (`PartialEq`).
That is why `Computed<T>` needs `T: PartialEq`.

A subscription has no children. It does not pass the change on.

### 5. Dependencies come from `get`, not from a declaration

Calling `get` records that node as a parent of the one that is computing.
The next run replaces the whole parent list.
If a computed stops reading a node, it no longer depends on it.

A child keeps its parents alive (strong refs).
The graph does not (weak refs).
Drop the last handle of a node, and the node is removed.

### 6. A wave refreshes each node at most once

A dirty node is ready when none of its parents are still dirty.
Children are marked dirty only when a parent’s value changed.
If a compute reads a parent that is still stale, that parent is refreshed first.
If dirty nodes remain and none are ready, or a node is refreshed while it is already
refreshing, there is a cycle, and the program panics.

A node does not refresh a second time in the same wave.

### 7. After a wave, every value is correct

When the wave ends, every `Value` and `Computed` matches the current sources.
A subscriber sees one value for the wave: the one that matches the sources.

The wave runs until nothing is dirty.
A cycle panics. Nodes are not dropped to "save" the wave.

### 8. Graphs do not mix

`Value::new` and `Computed::from` use one graph per thread.
`Graph::new()` makes a separate graph.
A write on graph A never sees nodes of graph B.
