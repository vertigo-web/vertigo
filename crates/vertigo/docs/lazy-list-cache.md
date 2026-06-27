# `LazyListCache` — optimistic, per-item reactive list cache

`LazyListCache<T>` is the list-oriented sibling of [`LazyCache`](crate::LazyCache).
Where `LazyCache<Vec<Item>>` treats the whole list as one reactive unit,
`LazyListCache<T>` stores each item separately — keyed by a stable id — so that:

- each item is **independently reactive** (a change to one row doesn't re-render
  the others),
- you can layer **optimistic edits / inserts / removals** on top of the
  server-confirmed data, then **commit** or **rollback** them per item,
- newly created rows can be **re-keyed in place** once the server assigns a real id.

It is the recommended structure for CRUD-style lists. For a single large object
where you don't need per-item granularity, prefer
[`LazyCache::optimistically_change`](crate::LazyCache::optimistically_change) instead.

> A runnable end-to-end example lives in the demo at
> [`demo/app/src/app/lazy_list/`](https://github.com/vertigo-web/vertigo/tree/master/demo/app/src/app/lazy_list) — `state.rs`
> wires the cache and CRUD actions, `component.rs` renders the rows.

---

## The mental model

`T` is a **marker type** implementing [`CollectionKey`](crate::CollectionKey).
It declares the item type (`T::Value`) and how to derive a stable key from it
(`T::Key`, via `T::get_key`). See the
[companion guide](crate::guides::value_synchronize_and_collections) for the
full rationale.

Internally the cache holds, per visible item, a `ListItem<V>` with **two**
reactive cells:

```text
ListItem<V> {
    original:     Value<V>,              // last value confirmed by the server
    override_val: Value<Option<Option<V>>>,
                  //   None        -> no optimistic override active
                  //   Some(Some)  -> optimistic edit/insert in effect
                  //   Some(None)  -> optimistically removed
}
```

plus three pieces of cross-item bookkeeping:

- `keys: Value<Vec<T::Key>>` — the **insertion-ordered, reactive** list of
  currently-visible keys. This is the cell that consumers ultimately subscribe
  to for "the list changed shape" notifications.
- `items: HashMap<Key, ListItem>` — O(1) per-key storage.
- `optimistic_placeholders: HashSet<Key>` — keys that exist *only* because of an
  optimistic insert and have never been confirmed by the server. This set is
  what makes [`rollback`] able to distinguish "undo an edit on a real
  row" from "remove a failed draft entirely".

The split between `original` and `override_val` is the heart of the design:
**all reads are override-aware** (override wins, else the server `original`), and
the commit/rollback methods manipulate the two cells in well-defined ways.

```text
            ┌──────────────────────── reads ────────────────────────┐
            │                                                       │
   get() / granular() / get_by_key()
   prefer `override_val`, fall back to `original`
   (optimistic *edits* show, *inserts* appear,
    *removals* drop the row / yield None)
            │                                                        │
            └────────────────────► ListItem ◄────────────────────────┘
                             original / override_val
            ┌────────────────────── writes ──────────────────────────┐
            │                                                        │
  apply_response (server)   optimistically_set_item     update_item / commit     rollback
  -> sets `original`,       -> sets `override_val`      -> fold override into    -> clear override,
     clears placeholders       (+ placeholder if new)     `original`, clear it      or drop the draft
```

---

## Construction

Usually you don't call `new` directly; you build from a `RequestBuilder`:

```rust,ignore
use vertigo::{AutoJsJson, CollectionKey, RequestBuilder, LazyListCache};

#[derive(Debug, AutoJsJson, PartialEq, Eq, Clone)]
pub struct Item { pub id: u32, pub name: String }

#[derive(PartialEq)]
pub struct ItemKey;
impl CollectionKey for ItemKey {
    type Key = u32;
    type Value = Item;
    fn get_key(val: &Item) -> u32 { val.id }
}

let cache: LazyListCache<ItemKey> = RequestBuilder::get("/api/items")
    .lazy_list_cache::<ItemKey>(|status, body| {
        if status == 200 { Some(body.into::<Vec<Item>>()) } else { None }
    })
    // optional: enable per-item fetching (GET /api/items/{id})
    .with_item_fetch(
        |id: &u32| RequestBuilder::get(format!("/api/items/{id}")),
        |status, body| if status == 200 { Some(body.into::<Item>()) } else { None },
    );
```

- [`lazy_list_cache`](crate::RequestBuilder::lazy_list_cache) on
  [`RequestBuilder`](crate::RequestBuilder) builds the list cache from the list endpoint.
- [`with_item_fetch`] adds an *optional* single-item endpoint used by
  [`fetch_item`]. Without it, `fetch_item` is a no-op.
- [`new_resource`](crate::LazyListCache::new_resource) is a
  shortcut when `T::Value: JsJsonDeserialize` and a plain `GET` returning `200` is enough.

The cache is lazy: it fetches on first read ([`get`] / [`get_by_key`] / [`granular`])
when in the `Uninitialized` state.

---

## Reading the list

There are three read paths. **All are override-aware** — they show the list *as
the user currently sees it*, with optimistic edits/inserts/removals applied —
and differ only in shape (whole list vs. per-item reactive vs. single row).

### `get(context) -> Resource<Rc<Vec<T::Value>>>`

The whole list as one reactive unit, in key order, **as the user currently sees
it**. Optimistic edits replace the value, optimistic inserts appear, and
optimistically-removed rows are **omitted** entirely (the deletion shows
immediately, before [`remove_item`] confirms it). Items without an active override
fall back to their server-confirmed `original` — i.e. this is [`get_by_key`]
applied across the whole list.

Triggers a fetch when uninitialized. Returns `Resource::Loading` / `Error`
before data is ready.

### `granular(context, filter) -> Resource<Rc<Vec<Computed<Option<T::Value>>>>>`

The same override-aware view, but as **individually reactive** items: every
element is its own `Computed<Option<T::Value>>`. A subscriber re-evaluates only
for the specific items that changed — ideal for long lists with sparse updates.
An active edit/insert yields the override value; an optimistically-removed row
yields `None` so it drops out of the rendered list.

The optional `filter` predicate makes each item's `Computed` yield `None` when
the item doesn't pass, so the UI can hide it without recomputing siblings.
Override and `filter` compose: a removed row is `None` regardless of the
predicate; a surviving row is then subject to it.

### `get_by_key(context, key) -> Resource<Rc<T::Value>>`

A single item:

- if an override edit is active (`Some(Some(v))`) → returns `v`,
- if optimistically removed (`Some(None)`) → returns `Resource::Loading`,
- otherwise → the `original` (or `Error` if the key is unknown once loaded).

This is the method a per-row component subscribes to. (The demo's `render_row`
does exactly this.)

> **Rule of thumb:** all reads honor optimistic state, so just pick by shape —
> [`get`] / [`to_computed`] for the whole list as one unit, [`granular`] for per-item
> reactivity (per-row re-render isolation on large lists), [`get_by_key`] for a
> single row.

---

## Mutations: the optimistic workflow

All mutations operate on `override_val` and/or the key list; none of them issue
network requests — you drive the request yourself and call the commit/rollback
method when it resolves.

| Method | Effect |
| --- | --- |
| [`optimistically_set_item`]`(item)` | Set an override edit. If the key is new, also create a placeholder (`original` seeded with the draft) and append the key, so the row is immediately visible. |
| [`optimistically_remove_item`]`(&key)` | Mark as removed (`Some(None)`); [`get_by_key`] reports `Loading` until confirmed or rolled back. No-op for unknown keys. |
| [`update_item`]`(item)` | Commit: set `original` to `item`, clear the override, ensure the key is present, drop any placeholder marker. Inserts the key if unknown. |
| [`update_item_with_old_key`]`(&old_key, item)` | Like [`update_item`], but re-keys a placeholder **in place** when the server assigned a different id (preserves list position). Delegates to [`update_item`] when the keys are equal. |
| [`remove_item`]`(&key)` | Confirm a deletion: drop the item from `items`, `keys`, and placeholder set. |
| [`commit`]`(&key)` | Fold the current override into `original` (or delete the row if it was `Some(None)`) **without** the server — the local optimistic state becomes the truth. |
| [`rollback`]`(&key)` | Undo: clear the override and restore `original`; **but** if the key was a pure optimistic placeholder (never confirmed), remove the row entirely. |

### Update flow (editing a confirmed row)

```text
orig=v0, override=None
        │ optimistically_set_item(v1)   → UI shows v1 via get_by_key
        ▼
orig=v0, override=Some(v1)
   ├─ server OK  → update_item(v1)  → orig=v1, override=None
   └─ server err → rollback(key)    → orig=v0, override=None
```

### Create flow (temp id → server-assigned id)

```text
keys: [1,2,3]
        │ optimistically_set_item(draft, id=0)
        ▼
keys: [1,2,3,0]   placeholder(0), orig=draft, override=Some(draft)
   ├─ server OK (assigns id=42) → update_item_with_old_key(&0, item{id:42})
   │                              → keys: [1,2,3,42] (same position), committed
   └─ server err                → rollback(&0)  → keys: [1,2,3] (draft removed)
```

The key subtlety: a failed *create* must not leave its draft visible, while a
failed *edit* of a confirmed row must keep the row. [`rollback`] distinguishes the
two using `optimistic_placeholders` — which is why [`update_item`] /
[`update_item_with_old_key`] / [`commit`] all clear the placeholder marker once a row
is confirmed.

---

## Refresh semantics

- [`force_update`]`(with_loading)` re-fetches the list now.
- [`forget`] resets to `Uninitialized` so the next read refetches.
- On a successful response, `apply_response` (internal) updates `original`s,
  inserts new keys, drops keys no longer returned, and **clears all placeholder
  markers** — the server response is treated as the source of truth. Active
  `override_val` edits on still-present keys are *preserved* (so an in-flight
  optimistic edit survives a background refresh and still shows in every read;
  rolling it back then reveals the refreshed server value underneath).

---

## Per-item fetching

If [`with_item_fetch`] was configured, [`fetch_item`]`(key)` fetches a single item
and updates its `original` (inserting it and appending the key if it wasn't
present). In-flight requests for the same key are deduplicated. Without
[`with_item_fetch`] it is a no-op.

---

## Rendering

`LazyListCache` is itself [`ToComputed`](crate::ToComputed) ([`to_computed`] yields
`Computed<Resource<Rc<Vec<T::Value>>>>` via [`get`], override-aware), and offers a
convenience [`render`](crate::LazyListCache::render) that shows `Loading…` / `error = …` / your render closure.

The simplest render drives the whole list from [`to_computed`] — edits, inserts
and removals are reflected automatically. For finer control (per-row re-render
isolation), render the **shape** from [`granular`] and each **row** from its own
[`Computed`](crate::Computed) over [`get_by_key`], so only changed rows re-render. The demo's
`component.rs` uses the per-row pattern:

```rust,ignore
// shape
let list = state_items().to_computed().render_value(|res| match res {
    Resource::Loading      => dom! { <div>"Loading…"</div> },
    Resource::Error(err)   => dom! { <div>"Error: " { err }</div> },
    Resource::Ready(items) => render_rows(items), // iterates ids
});

// per row — override-aware, independently reactive
fn render_row(id: u32) -> DomNode {
    let cache = state_items();
    Computed::from(move |ctx| cache.get_by_key(ctx, &id))
        .render_value(/* ... */)
}
```

> Note: the keyed memoized renderers
> [`render_resource_list_memo`](crate::render::render_resource_list_memo)
> operate on a plain `LazyCache<Vec<Item>>`, not on `LazyListCache`.
> `LazyListCache` already provides per-item reactivity directly through
> [`granular`] / [`get_by_key`], so render rows from those instead.

---

## Quick reference

| Concern | API |
| --- | --- |
| Build | [`RequestBuilder::lazy_list_cache`](crate::RequestBuilder::lazy_list_cache), [`new`](crate::LazyListCache::new), [`new_resource`](crate::LazyListCache::new_resource), [`with_item_fetch`] |
| Read whole list (override-aware) | [`get`], [`to_computed`] |
| Read per-item reactive (override-aware) | [`granular`] |
| Read one row (override-aware) | [`get_by_key`] |
| Optimistic write | [`optimistically_set_item`], [`optimistically_remove_item`] |
| Commit | [`update_item`], [`update_item_with_old_key`], [`remove_item`], [`commit`] |
| Undo | [`rollback`] |
| Refresh | [`force_update`], [`forget`], [`fetch_item`] |
| Render | [`render`](crate::LazyListCache::render), [`to_computed`], or per-row [`Computed`](crate::Computed) over [`get_by_key`] |

[`get`]: crate::LazyListCache::get
[`granular`]: crate::LazyListCache::granular
[`get_by_key`]: crate::LazyListCache::get_by_key
[`to_computed`]: crate::LazyListCache::to_computed
[`new_resource`]: crate::LazyListCache::new_resource
[`with_item_fetch`]: crate::LazyListCache::with_item_fetch
[`fetch_item`]: crate::LazyListCache::fetch_item
[`optimistically_set_item`]: crate::LazyListCache::optimistically_set_item
[`optimistically_remove_item`]: crate::LazyListCache::optimistically_remove_item
[`update_item`]: crate::LazyListCache::update_item
[`update_item_with_old_key`]: crate::LazyListCache::update_item_with_old_key
[`remove_item`]: crate::LazyListCache::remove_item
[`commit`]: crate::LazyListCache::commit
[`rollback`]: crate::LazyListCache::rollback
[`force_update`]: crate::LazyListCache::force_update
[`forget`]: crate::LazyListCache::forget
