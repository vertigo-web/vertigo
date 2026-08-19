# `CollectionKey` and the memoized list renderers

This document explains a small cluster of related building blocks in Vertigo and
how they fit together:

- [`CollectionKey`](#collectionkey) — a marker trait describing how to identify
  items in a list (the key) and what the item type is.
- [`render_list_memo`](#render_list_memo) and
  [`render_resource_list_memo`](#render_resource_list_memo) — high-level helpers
  that render reactive lists while **memoizing each item**, so only items that
  actually changed are re-rendered.

Per-item [`Computed`](crate::Computed)s come from
[`keyed_computed_list`](crate::keyed_computed_list).

```text
  Source of truth             keyed_computed_list           Per-item reactive view          Renderer
 ┌──────────────────┐         ┌────────────────────┐    ┌───────────────────────────┐   ┌──────────────────────────┐
 │ Value<Rc<Vec<T>>>│         │ keyed_computed_list│    │ Vec<KeyedListItem>        │   │ render_list_memo         │
 │   or             │ ──────▶ │                    │──▶ │  each item = Computed<V>  │──▶│   /                      │
 │ LazyCache<Vec<T>>│  graph  │                    │    │                           │   │ render_resource_list_memo│
 └──────────────────┘         └────────────────────┘    └───────────────────────────┘   └──────────────────────────┘
```

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
[`LazyListCache<T>`](crate::LazyListCache) and the memoized list renderers. Keying matters for two reasons:

1. **Memoization** — items keep the *same* per-item `Computed` across
   updates as long as their key is stable. Only items whose content actually
   changed cause a re-render.
2. **Deduplication** — [`keyed_computed_list`](crate::keyed_computed_list) logs an error and skips
   items with a duplicate key within a single list.

---

## `render_list_memo`

```rust,ignore
pub fn render_list_memo<T: CollectionKey + 'static>(
    value: &Value<Rc<Vec<T::Value>>>,
    render: impl Fn(&Computed<T::Value>) -> DomNode + 'static,
) -> DomNode
```

Renders a reactive list from a `Value<Rc<Vec<Item>>>`, memoizing each item.

Internally it maps the source to `Computed<Vec<Item>>` and calls
[`render_list`](crate::render::render_list), which runs
[`keyed_computed_list`](crate::keyed_computed_list) so each key keeps a stable
per-item [`Computed`](crate::Computed).

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
auto-refreshes on a TTL. Loading / Error states are normalized to an empty list,
then the cache is fed through [`keyed_computed_list`](crate::keyed_computed_list)
exactly like `render_list_memo`.

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
| [`CollectionKey`](crate::CollectionKey) | `vertigo::CollectionKey`                 |
| [`render_list_memo`](crate::render::render_list_memo) | `vertigo::render::render_list_memo`      |
| [`render_resource_list_memo`](crate::render::render_resource_list_memo) | `vertigo::render::render_resource_list_memo` |

[`render_list_memo`](crate::render::render_list_memo) / [`render_resource_list_memo`](crate::render::render_resource_list_memo) are reachable through the public
`render` module (`vertigo::render::…`); they are not re-exported at the crate
root the way [`CollectionKey`](crate::CollectionKey) is.
