# `WsCollection` — server-pushed reactive collections over a WebSocket

`WsCollection<T>` subscribes to a **server-side query** over a single WebSocket and
keeps a reactive list in sync with it. You describe *what* you want with a
[`WsQuery`](crate::WsQuery) (a table plus `where` clauses); the server replies with an
initial snapshot, then **pushes** every subsequent change — row inserted, row updated,
row removed — down the same connection. The collection folds those frames into a list
you bind to.

It is the **server-driven** counterpart to the fetch-based caches:

| Type | Who drives changes | Shape |
| --- | --- | --- |
| [`LazyCache`](crate::LazyCache) | client refetches on a TTL | one whole resource |
| [`LazyListCache`](crate::LazyListCache) | client, via optimistic CRUD | a keyed list |
| `WsCollection` | **server, via pushed frames** | a keyed list |

Reach for `WsCollection` when the source of truth lives on the server and changes out
from under the client (live dashboards, queues, feeds) — not when the client owns the
edits (use `LazyListCache` for that).

---

## The mental model

```text
  WsSocket (one connection)
  ├── subscription query_0  ─┐
  ├── subscription query_1   │  server frames are routed by query_id
  └── subscription query_2  ─┘
            │
            ▼  Init / Set / Delete / Batch
  WsCollection<T>
     state: Value<Option<HashMap<key, T>>>   // None = loading, Some(map) = loaded
            │  map_to_sorted_vec (stable order by key)
            ▼
     items_sorted: Computed<Option<Vec<T>>>  // what the UI binds to
```

- One [`WsSocket`](crate::WsSocket) owns **one** connection and multiplexes any number
  of subscriptions, each identified by a `query_id`.
- Each `WsCollection<T>` is **one** subscription. Incoming rows are reconciled into a
  `key → row` map and exposed, sorted by key for stable ordering, as
  [`items_sorted`](crate::WsCollection::items_sorted): a
  `Computed<Option<Vec<T>>>` that is `None` until the first snapshot lands and
  `Some(rows)` thereafter.
- The subscription is held alive by a [`DropResource`](crate::DropResource) **captured
  inside** the `items_sorted` `Computed`. Drop the collection (or the `Computed`) and the
  subscription is torn down automatically — there is no separate `unsubscribe` to call.

`T` is your row type; it only needs `Clone + PartialEq + JsJsonDeserialize`. The row
payload travels as opaque JSON, so the collection is agnostic to your domain — the
server decides the columns.

---

## Construction

First build the socket once, then build a collection per subscription.

```rust,ignore
use std::rc::Rc;
use vertigo::{AutoJsJson, WsSocket, WsQuery, WsCollection};

// The connection. Usually wrapped in a `#[store]` so the whole app shares one socket.
let socket = Rc::new(WsSocket::new(
    "wss://example.com/ws",
    Rc::new(|| Some(current_auth_token())), // see "Authentication" below
));

#[derive(Clone, PartialEq, AutoJsJson)]
struct Item { id: u32, name: String }

// A long-lived subscription: every row of `items` where amount > 100.
let collection: WsCollection<Item> = WsCollection::new(
    socket.clone(),
    WsQuery::and("items").gt("amount", 100.0),
);

// Bind `items_sorted` in your view. Often you keep only this field:
let rows = collection.items_sorted; // Computed<Option<Vec<Item>>>
```

Sharing one socket app-wide is the common case — define a singleton with
[`store`](crate::store) and supply your app's URL and token source there:

```rust,ignore
use std::rc::Rc;
use vertigo::{store, WsSocket};

#[store]
fn ws_socket() -> Rc<WsSocket> {
    Rc::new(WsSocket::new(
        socket_url(),
        Rc::new(|| current_auth_token()),
    ))
}
```

Then every `WsCollection::new(ws_socket(), query)` reuses the same connection.

---

## The query builder

[`WsQuery`](crate::WsQuery) is a small builder for the subscription request:

```rust,ignore
WsQuery::and("items")            // table + top-level logic (AND)
    .eq("category", "alpha")     // category = "alpha"
    .gt("amount", 100.0)         // AND amount > 100
    .like("label", "Ar")         // AND label LIKE "Ar"
    .eq_null("closed_at")        // AND closed_at IS NULL
    .limit(50);                  // cap the row count (server clamps to its max)

WsQuery::or("items")             // top-level logic OR
    .eq("status", "new")
    .eq("status", "queued");     // status = "new" OR status = "queued"
```

- [`and`](crate::WsQuery::and) / [`or`](crate::WsQuery::or) choose how the clauses
  combine and name the table.
- [`eq`](crate::WsQuery::eq) / [`gt`](crate::WsQuery::gt) / [`lt`](crate::WsQuery::lt)
  accept any value implementing [`IntoCollectionWhereValue`](crate::IntoCollectionWhereValue)
  — strings, the integer and float primitives, and `bool`.
- [`like`](crate::WsQuery::like) takes a string pattern;
  [`eq_null`](crate::WsQuery::eq_null) matches a missing value.

The wire form is `{ "table", "logic", "where": [ { "op", "column", "value" }, … ], "limit" }`.

---

## The update lifecycle

Each frame the server pushes is one of:

| Frame | Effect on the collection |
| --- | --- |
| `Init` | Full snapshot — **replaces** the whole row map. This is what flips `items_sorted` from `None` to `Some`. |
| `Set` | Upsert one row by id. By default it replaces the row; with a custom merge it is reconciled against the previous row (see below). |
| `Delete` | Drop one row by id. |
| `Batch` | A list of the above, unwrapped recursively and applied in order. |
| `Message` | A server-level log or error. Logged at the socket; **not** delivered to any collection. |

```text
server ──frame──▶ WsSocket.dispatch(query_id) ──▶ WsCollection.state ──▶ items_sorted
                    │
                    ├─ Batch → re-dispatch each inner frame
                    └─ Message → log only
```

### Custom merge

`Set` frames sometimes carry only the fields that changed, omitting values that were
present in the `Init` snapshot (e.g. an image URL sent once up front). Use
[`new_with_merge`](crate::WsCollection::new_with_merge) to fold each update onto the
previous row instead of replacing it:

```rust,ignore
let collection = WsCollection::new_with_merge(
    socket.clone(),
    WsQuery::and("items").eq("id", id),
    Rc::new(|incoming: Item, prev: &Item| incoming.preserving_image_from(prev)),
);
```

The closure receives `(incoming, &previous)` and returns the row to store. The default
(used by [`new`](crate::WsCollection::new)) simply keeps `incoming`.

---

## Managing the subscription

When the query itself changes over the collection's life, build it
[`empty`](crate::WsCollection::empty) and re-point it:

```rust,ignore
// Held in an Rc so the struct (and its subscription) outlives the closures below.
let collection = Rc::new(WsCollection::<Item>::empty(socket.clone()));

// Filter changed → reissue and show a loading state while the new snapshot loads.
collection.set_query(WsQuery::and("items").eq("category", chosen_category));

// Scrolled to the bottom → grow the window WITHOUT flashing to loading.
collection.extend_query(WsQuery::and("items").limit(next_page_size));
```

- [`set_query`](crate::WsCollection::set_query) clears the rows to `None` (loading)
  before the new `Init` arrives — right when the result set is conceptually different.
- [`extend_query`](crate::WsCollection::extend_query) keeps the current rows visible
  until the new `Init` replaces them — right for pagination, where the new query is a
  superset and a flash to loading would be jarring.

On reconnect the socket re-sends a `Subscribe` for **every** live subscription, so a
dropped connection recovers transparently.

---

## Rendering

`items_sorted` is a plain `Computed<Option<Vec<T>>>`: `None` is loading, `Some(vec)` is
loaded. Bind it directly, or project it first:

```rust,ignore
// Loading vs. loaded
collection.items_sorted.render_value(|opt| match opt {
    None        => dom! { <div>"Loading…"</div> },
    Some(items) => render_table(items),
});

// Or flatten away the loading state when an empty list is an acceptable placeholder
let rows: Computed<Vec<Item>> = collection.items_sorted.map(|o| o.unwrap_or_default());
```

For long lists where only a few rows change at a time, pair the inner `Vec` with
[`render_list_memo`](crate::render::render_list_memo) so unchanged rows are not
re-rendered — see the
[value-synchronize & collections guide](crate::guides::value_synchronize_and_collections).

---

## Authentication

The [`AuthTokenProvider`](crate::AuthTokenProvider) closure you pass to
[`WsSocket::new`](crate::WsSocket::new) is consulted for the token sent with **every**
`Subscribe` — both at subscribe time and again for each live subscription on reconnect.
Have it read the *current* token from wherever your app keeps it (a reactive store, for
instance) so a token refreshed mid-session is picked up. Returning `None` defers that
subscribe — it is logged and skipped until a token exists. If the server needs no auth,
pass `Rc::new(|| Some(String::new()))` (or `|| None` and have the server ignore it).

---

## Quick reference

| Concern | API |
| --- | --- |
| Connection | [`WsSocket::new`](crate::WsSocket::new), [`WsSocket::subscribe`](crate::WsSocket::subscribe) |
| Build (fixed query) | [`WsCollection::new`](crate::WsCollection::new), [`new_with_merge`](crate::WsCollection::new_with_merge) |
| Build (changing query) | [`WsCollection::empty`](crate::WsCollection::empty) |
| Read | [`items_sorted`](crate::WsCollection::items_sorted) (`Computed<Option<Vec<T>>>`) |
| Reissue query | [`set_query`](crate::WsCollection::set_query) (clears to loading), [`extend_query`](crate::WsCollection::extend_query) (keeps rows) |
| Query builder | [`WsQuery::and`](crate::WsQuery::and) / [`or`](crate::WsQuery::or) + `eq` / `like` / `gt` / `lt` / `eq_null` / `limit` |
| Auth | [`AuthTokenProvider`](crate::AuthTokenProvider) passed to [`WsSocket::new`](crate::WsSocket::new) |
