use std::rc::Rc;

use vertigo::{AutoJsJson, Value, WsCollection, WsQuery, WsSocket, transaction};

/// Mirrors the JSON the demo server sends in each row's `model`
/// (see `demo/server/src/ws_collection.rs`). Field names line up with the
/// server's serde output.
#[derive(Clone, Debug, PartialEq, AutoJsJson)]
pub struct Ukulele {
    pub id: u32,
    pub kind: String,
    pub name: String,
    pub scale_inches: f64,
    pub tuning: String,
    pub description: String,
    pub stock: u32,
}

/// The ukulele types the filter dropdown offers. `""` means "all kinds".
pub const KINDS: &[&str] = &[
    "Soprano",
    "Concert",
    "Tenor",
    "Baritone",
    "Pineapple",
    "Banjo",
    "Resonator",
    "Bass",
    "Sopranino",
];

const TABLE: &str = "ukuleles";

#[derive(Clone)]
pub struct WsCollectionState {
    /// One shared connection; held alive for the lifetime of the demo state.
    _socket: Rc<WsSocket>,
    /// The reactive, server-driven collection. Built `empty` so the filter
    /// controls can re-point it via `set_query`.
    pub collection: Rc<WsCollection<Ukulele>>,
    /// Selected kind filter (`""` = all).
    pub kind: Value<String>,
    /// Free-text `LIKE` search on the ukulele name.
    pub search: Value<String>,
}

impl WsCollectionState {
    pub fn new(ws_url: String) -> Self {
        // The demo server ignores auth; send an empty token so subscribes are not deferred.
        let socket = Rc::new(WsSocket::new(ws_url, Rc::new(|| Some(String::new()))));
        let collection = Rc::new(WsCollection::<Ukulele>::empty(socket.clone()));

        let kind = Value::new(String::new());
        let search = Value::new(String::new());

        let state = Self {
            _socket: socket,
            collection,
            kind,
            search,
        };

        // Issue the first subscription (no filters → all rows).
        state.reissue("", "");
        state
    }

    /// Rebuild the query from the current controls and re-point the collection.
    pub fn reissue(&self, kind: &str, search: &str) {
        self.collection.set_query(build_query(kind, search));
    }

    pub fn set_kind(&self, kind: String) {
        let search = transaction(|ctx| self.search.get(ctx));
        self.kind.set(kind.clone());
        self.reissue(&kind, &search);
    }

    pub fn set_search(&self, search: String) {
        let kind = transaction(|ctx| self.kind.get(ctx));
        self.search.set(search.clone());
        self.reissue(&kind, &search);
    }
}

fn build_query(kind: &str, search: &str) -> WsQuery {
    let mut query = WsQuery::and(TABLE);
    if !kind.is_empty() {
        query = query.eq("kind", kind);
    }
    if !search.is_empty() {
        query = query.like("name", search);
    }
    query
}
