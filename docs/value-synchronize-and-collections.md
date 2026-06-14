# `Value::synchronize`, `ValueSynchronize`, `CollectionKey`, and the memoized list renderers

This document explains a small cluster of related building blocks in Vertigo and
how they fit together:

- [`Value::synchronize`](#valuesynchronize) — the generic "mirror this reactive
  source into a derived structure" mechanism.
- [`ValueSynchronize`](#valuesynchronize-trait) — the trait that a derived
  structure implements to become a valid synchronization *target*.
- [`CollectionKey`](#collectionkey) — a marker trait describing how to identify
  items in a list (the key) and what the item type is.
- [`render_list_memo`](#render_list_memo) and
  [`render_resource_list_memo`](#render_resource_list_memo) — high-level helpers
  that render reactive lists while **memoizing each item**, so only items that
  actually changed are re-rendered.

The thread connecting all of them is `Collection<T>`: an internal structure that
turns a flat `Vec<Item>` into a list of *stable, per-item* [`Computed`](crate::Computed)s keyed by
[`CollectionKey`](crate::CollectionKey). It is created and kept up to date through `synchronize`, and it
is the data source that the memoized list renderers consume.

```text
  Source of truth                synchronize()            Per-item reactive view          Renderer
 ┌──────────────────┐         ┌────────────────────┐    ┌───────────────────────────┐   ┌──────────────────────────┐
 │ Value<Rc<Vec<T>>>│         │ ValueSynchronize   │    │ Collection<T>             │   │ render_list_memo         │
 │   or             │ ──────▶ │   (Collection<T>   │──▶ │  Vec<CollectionModel<T>>  │──▶│   /                      │
 │ LazyCache<Vec<T>>│  event  │    is the target)  │    │  each item = Computed<V>  │   │ render_resource_list_memo│
 └──────────────────┘         └────────────────────┘    └───────────────────────────┘   └──────────────────────────┘
```

---

## `Value::synchronize`

[`Value<T>`](crate::Value) is the basic reactive cell. [`synchronize`](crate::Value::synchronize) lets you create a derived,
self-updating object `R` that mirrors the value and keeps following every change:

```rust,ignore
pub fn synchronize<R: ValueSynchronize<T> + Clone + 'static>(&self)
    -> (R, DropResource)
```

What it does:

1. Reads the current value of the `Value`.
2. Constructs the target `R` from it via `R::new(init_value)`.
3. Subscribes (`add_event`) so every subsequent `set` on the `Value` is pushed
   into the target via `R::set(...)`.
4. Returns the target plus a [`DropResource`](crate::DropResource). **The subscription lives only as
   long as that `DropResource` is held** — drop it and the mirroring stops.

The same pattern exists on the resource-fetching types:

- [`LazyCache<T>::synchronize`](crate::LazyCache::synchronize) mirrors the cache's current `Resource<Rc<T>>` into
  a target of type [`ValueSynchronize`](crate::ValueSynchronize)`<Rc<T>>`. Loading / Error / Uninitialized
  states are normalized to `T::default()` (hence the extra `T: Default + Clone`
  bound), so the target always sees a concrete `Rc<T>`.
- Internally this is implemented by `CacheValue<T>::synchronize`, which performs
  the same normalize-and-subscribe dance.

So `synchronize` is *one* mechanism with several entry points ([`Value`](crate::Value),
[`LazyCache`](crate::LazyCache), `CacheValue`), all parameterized over the target type `R`.

---

## `ValueSynchronize` trait

```rust,ignore
pub trait ValueSynchronize<T: PartialEq + Clone + 'static>: Sized {
    fn new(value: T) -> Self;
    fn set(&self, value: T);
}
```

This is the contract a type must satisfy to be a valid [`synchronize`](crate::Value::synchronize) *target*:

- `new(value)` — build the target from the source's initial value.
- `set(value)` — accept each subsequent update.

Note `set` takes `&self` (not `&mut self`): synchronization targets are expected
to use interior reactivity ([`Value`](crate::Value), `Rc`, etc.) internally, so a shared handle
can absorb updates. That is exactly why targets must also be `Clone` — the
closure registered with `add_event` captures a clone of the target.

The only target implemented in-tree today is `Collection<T>`, but the trait is
public so you can write your own (e.g. a target that maintains an index, a
running total, or a sorted view).

---

## `CollectionKey`

```rust,ignore
pub trait CollectionKey {
    type Key: Eq + Hash + Clone + std::fmt::Debug + 'static;
    type Value: Clone + PartialEq + 'static;
    fn get_key(val: &Self::Value) -> Self::Key;
}
```

[`CollectionKey`](crate::CollectionKey) is a **marker / descriptor trait**. You implement it on a
zero-sized marker type (not on the item itself), and it declares three things:

- `Value` — the item type stored in the list.
- `Key` — a stable identity for an item (e.g. a database id).
- `get_key` — how to extract the key from an item.

Typical implementation:

```rust,ignore
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Item { pub id: u32, pub name: String }

pub struct ItemKey; // marker type

impl CollectionKey for ItemKey {
    type Key = u32;
    type Value = Item;
    fn get_key(val: &Item) -> u32 { val.id }
}
```

The marker type (`ItemKey`) is the generic parameter `T` threaded through
`Collection<T>`, `CollectionModel<T>`, [`LazyListCache<T>`](crate::LazyListCache), and the memoized list
renderers. Keying matters for two reasons:

1. **Memoization** — items keep the *same* per-item `Value`/`Computed` across
   updates as long as their key is stable. Only items whose content actually
   changed cause a re-render.
2. **Deduplication** — `Collection` logs an error and skips items with a
   duplicate key within a single list.

---

## How `Collection<T>` ties it together

`Collection<T>` is the internal structure produced by `synchronize`. It is *not*
exported at the crate root, but understanding it explains the whole flow:

- It holds an `ItemDataCollection<T>` — a `HashMap<Key, (Value<V>, Computed<V>)>`
  — plus an `order: Value<Vec<CollectionModel<T>>>` describing the current list
  order.
- On `set(new_list)` it:
  - extracts the key of each item,
  - reuses the existing per-item `Value` if the key is already known (calling
    `value.set(item)`, which only triggers downstream work when the item is
    actually `!=` the previous one),
  - creates a fresh per-item `Value`/`Computed` for new keys,
  - updates `order`,
  - retains only the keys still present (dropping per-item state for removed
    items).
- It implements `ValueSynchronize<Rc<Vec<T::Value>>>`, which is what makes it a
  legal `synchronize` target.

`Collection::get()` returns `Computed<Vec<CollectionModel<T>>>`, where each
`CollectionModel<T>` is `{ key: T::Key, model: Computed<T::Value> }`. The
*outer* computed changes when the list's order/membership changes; each *inner*
`model` computed changes only when that specific item changes. This two-level
reactivity is the source of the memoization.

---

## `render_list_memo`

```rust,ignore
pub fn render_list_memo<T: CollectionKey + 'static>(
    value: &Value<Rc<Vec<T::Value>>>,
    render: impl Fn(&Computed<T::Value>) -> DomNode + 'static,
) -> DomNode
```

Renders a reactive list from a `Value<Rc<Vec<Item>>>`, memoizing each item.

Internally it:

1. calls `value.synchronize::<Collection<T>>()` to obtain the keyed collection
   and its `DropResource`,
2. feeds `collection.get()` into the lower-level `render_list`, using each
   item's `key` for identity and rendering each item from its **own per-item
   `Computed`**,
3. attaches the synchronization `DropResource` (and a handle on the source
   `value`) to the resulting node so everything is cleaned up when the node is
   dropped.

Because the render closure receives a `&`[`Computed`](crate::Computed)`<T::Value>` (not a bare value),
the rendered subtree for an item re-runs only when that item changes — not when
sibling items or the list order change.

Use this when **your list already lives in a [`Value`](crate::Value)** that you own.

---

## `render_resource_list_memo`

```rust,ignore
pub fn render_resource_list_memo<T: CollectionKey + 'static>(
    value: &LazyCache<Vec<T::Value>>,
    render: impl Fn(&Computed<T::Value>) -> DomNode + 'static,
) -> DomNode
```

Identical in shape to [`render_list_memo`](crate::render::render_list_memo), but the source is a
[`LazyCache`](crate::LazyCache)`<Vec<Item>>` — a lazily-loaded, possibly remote resource that
auto-refreshes on a TTL. It calls `value.synchronize::<Collection<T>>()` on the
`LazyCache` (which normalizes Loading/Error to an empty/`default` list) and then
renders exactly like `render_list_memo`.

Use this when **your list comes from a fetched resource** and you still want
per-item memoization.

> Related: [`LazyListCache<T>`](crate::LazyListCache) is a higher-level wrapper (also keyed by
> [`CollectionKey`](crate::CollectionKey)) that adds optimistic create/update/delete and per-item
> fetching on top of a list resource. `render_resource_list_memo` is the
> lower-level renderer for a plain `LazyCache<Vec<Item>>`. See the
> [`LazyListCache` guide](crate::guides::lazy_list_cache).

---

## End-to-end example

```rust,ignore
use std::rc::Rc;
use vertigo::{dom, CollectionKey, Computed, DomNode, Value};
use vertigo::render::render_list_memo; // path-dependent; see "Public surface"

#[derive(Clone, PartialEq, Eq, Debug)]
struct Item { id: u32, name: String }

struct ItemKey;
impl CollectionKey for ItemKey {
    type Key = u32;
    type Value = Item;
    fn get_key(v: &Item) -> u32 { v.id }
}

fn view(items: &Value<Rc<Vec<Item>>>) -> DomNode {
    render_list_memo::<ItemKey>(items, |item: &Computed<Item>| {
        let item = item.clone();
        item.render_value(|it| dom! { <div>{it.name}</div> })
    })
}
```

When `items` is updated:

- items with unchanged content are **not** re-rendered (their inner `Computed`
  did not change),
- only added / removed / reordered / mutated items cause DOM work.

---

## Public surface (what you can use directly)

| Item                          | Exported as                              |
| ----------------------------- | ---------------------------------------- |
| [`Value::synchronize`](crate::Value::synchronize) | method on [`vertigo::Value`](crate::Value) |
| [`LazyCache::synchronize`](crate::LazyCache::synchronize) | method on [`vertigo::LazyCache`](crate::LazyCache) |
| [`ValueSynchronize`](crate::ValueSynchronize) | `vertigo::ValueSynchronize`              |
| [`CollectionKey`](crate::CollectionKey) | `vertigo::CollectionKey`                 |
| [`render_list_memo`](crate::render::render_list_memo) | `vertigo::render::render_list_memo`      |
| [`render_resource_list_memo`](crate::render::render_resource_list_memo) | `vertigo::render::render_resource_list_memo` |
| `Collection`, `CollectionModel` | `vertigo::render::collection::*` (lower-level) |

[`render_list_memo`](crate::render::render_list_memo) / [`render_resource_list_memo`](crate::render::render_resource_list_memo) are reachable through the public
`render` module (`vertigo::render::…`); they are not re-exported at the crate
root the way [`CollectionKey`](crate::CollectionKey) is.
