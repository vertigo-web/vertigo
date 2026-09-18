# Hydration

Mount collects the whole tree, then sends once. After that, every transaction
flushes as usual.

```mermaid
sequenceDiagram
    participant JS
    participant Mount
    participant Buffer
    participant Rust

    JS->>Mount: vertigo_entry_function
    Note over Mount,Buffer: flush hook is not installed yet

    Mount->>Rust: transaction { init_app(); set_root() }
    Rust->>Buffer: CreateNode / InsertBefore / ...
    Note over Rust: transaction ends: hooks, then flush_watch
    Note over Buffer: commands stay in the buffer

    Mount->>JS: DomSnapshotGet
    JS-->>Mount: snapshot (or null)
    Mount->>Mount: reconcile / discard / verbatim
    Mount->>JS: one DomBulkUpdate
    Note over JS: NodeAdopt / SnapshotRemove / the rest

    Mount->>Mount: enable_dom_flush()

    Note over Rust,JS: ordinary life from here
    Rust->>JS: transaction → flush_dom_changes → DomBulkUpdate
```

[`start_app`](crate::start_app) is the Rust side of that sequence. It does not
install the post-transaction flush hook until the first send has gone out.

## Why the first send waits

Matching needs the complete tree. `when_connect` / `Value::with_connect` can
still append DOM commands after `on_after_transaction` (`flush_watch` runs
later), so a flush at the end of the mount transaction would send a half-built
document. Commands therefore stay in the buffer until `transaction` returns,
and only then does `flush_mount` run.

## What `flush_mount` does

This is the one special moment. Rust asks the browser for a snapshot
(`DomSnapshotGet`) and then:

- snapshot present → reconcile the buffer against it (or discard the server
  markup when `--disable-hydration` is set)
- no snapshot (SSR, host tests) → the buffer goes out as queued

After that send, `enable_dom_flush` hangs `on_after_transaction` →
`flush_dom_changes`. A click, a `set`, a timer: transaction, send, no
hydration.
