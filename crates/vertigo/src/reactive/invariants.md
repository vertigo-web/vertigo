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
4. Then `on_after_transaction` hooks.
5. After that: connect and disconnect. Installing and dropping external handlers is
   not part of the wave.

A `set` from `when_connect` (`create`) starts this again.
A `set` from compute, subscribe, or a `DropResource` destructor is logged and ignored.
It never writes.

## Invariants

### 1. Compute, subscribe and Drop must not write

Do not call `Value::set` (or `change`) from:

- a compute closure
- a subscribe callback
- a `DropResource` destructor
- a wave that is already running

Compute only reads. Subscribe only talks to the outside world (DOM, logs).
Drop only tears down an external subscription (timer, socket, `popstate`).
None of them may write back into the graph.

If they do, the write is ignored and the console gets:

```text
vertigo: Value::set is not allowed from a computed, a subscribe callback, a DropResource, or during propagation
```

This covers anything that runs *inside* those closures, not only the code you wrote there.
Dropping something from a subscribe callback — a component going away while the view is
rebuilt — runs its `Drop` inside the callback, so a `set` from there is ignored too.

You may write from click/input handlers, timers, fetch, sockets,
`on_after_transaction`, and `when_connect` / `Value::with_connect` (`create` only).

### 2. Connect and disconnect wait until the wave is done

`when_connect` does not run the moment a node gets a child.
Disconnect does not run the moment it loses the last child.
Both wait until the wave ends, and until `on_after_transaction` has run.
If the graph is idle, they run at once.

If a node is watched and then unwatched in the same wave, nothing happens.
If it is unwatched and then watched, it connects once.

`create` runs after the graph has settled, so `Value::with_connect` may call `set`
(for example to clear the value when attaching). That `set` is a new transaction
and a new wave.

That wave can change who is watched, including the node that is connecting right now.
The connect state is matched to the graph again once the closure returns.
A node unwatched by its own connect is disconnected. It never stays connected.

Disconnect must not write. It cannot watch the node again, so connect and disconnect
cannot bounce. A chain — one connect watching the next node — is not a loop.

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
