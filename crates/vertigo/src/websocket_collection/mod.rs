//! Server-pushed reactive collections over a single WebSocket.
//!
//! A caller subscribes with a [`WsQuery`] (a table + `where` clauses); the server
//! replies with an `Init` snapshot, then streams incremental `Set` / `Delete` frames
//! (optionally wrapped in `Batch`) as the underlying rows change. Each subscription is
//! reconciled into a key→row map and surfaced as a single reactive
//! [`WsCollection::items_sorted`] (`Computed<Option<Vec<T>>>`) the UI binds to —
//! `None` while the first snapshot is in flight, `Some(rows)` once loaded.
//!
//! This is the *server-driven* sibling of [`LazyListCache`](crate::LazyListCache):
//! where `LazyListCache` is built around client-initiated optimistic CRUD, here the
//! server owns the data and pushes every change down the socket. One [`WsSocket`] (one
//! connection) multiplexes many subscriptions, routing each frame to the right
//! collection by `query_id`. Subscriptions are reference-counted by [`DropResource`]:
//! the keepalive lives inside the `Computed`, so a collection unsubscribes
//! automatically when it is dropped, and all live subscriptions are re-sent on
//! reconnect.
//!
//! See the [guide](crate::guides::websocket_collection) for a worked example.

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use vertigo_macro::bind;

use crate::{
    Computed, DropResource, JsJsonDeserialize, Value, WebsocketConnection, WebsocketMessage,
    dev::ValueMut, get_driver, spawn,
};

mod types;
mod ws_message_from;
mod ws_message_to;

pub use types::{CollectionWhereValue, IntoCollectionWhereValue, WsQuery};
pub use ws_message_from::{WsMessageKind, WsServerMessageFrom};

use self::{types::WebsocketQueryId, ws_message_to::WsClientMessageTo};

/// Callback invoked for each server frame routed to a subscription.
pub type Callback = Rc<dyn Fn(WsServerMessageFrom)>;

/// Supplies the auth token sent with every `Subscribe`.
///
/// Invoked synchronously at subscribe time (and again for each live subscription on
/// reconnect), so it should read the *current* token from wherever the app keeps it.
/// Return `None` to defer the subscription — it is logged and skipped until a token is
/// available. Use `Rc::new(|| None)` if the server requires no authentication.
pub type AuthTokenProvider = Rc<dyn Fn() -> Option<String>>;

#[derive(Clone)]
struct Subscription {
    query: WsQuery,
    callback: Callback,
}

/// One WebSocket connection shared across every subscription.
///
/// Construct once with [`WsSocket::new`] (commonly behind a [`store`](crate::store)
/// singleton so the whole app shares a single connection) and pass it to each
/// [`WsCollection`]. It owns the connection, keeps the set of live subscriptions keyed
/// by `query_id`, and on (re)connect re-sends a `Subscribe` for each one so collections
/// survive a dropped socket transparently.
pub struct WsSocket {
    _drop: DropResource,
    connection_box: Rc<ValueMut<Option<WebsocketConnection>>>,
    collections: Rc<ValueMut<HashMap<WebsocketQueryId, Subscription>>>,
    auth: AuthTokenProvider,
}

impl WsSocket {
    fn run_unsubscribe(
        collections: &Rc<ValueMut<HashMap<WebsocketQueryId, Subscription>>>,
        connection_box: &Rc<ValueMut<Option<WebsocketConnection>>>,
        query_id: WebsocketQueryId,
    ) {
        collections.change(|inner| {
            inner.remove(&query_id);
        });

        let message = WsClientMessageTo::unsubscribe(query_id);

        connection_box.change(|opt| {
            if let Some(conn) = opt {
                conn.send(message);
            } else {
                log::warn!("ws-collection - no connection, cannot unsubscribe");
            }
        });
    }

    /// Opens a connection to `url` and returns a handle ready to [`subscribe`](Self::subscribe).
    ///
    /// `auth` is consulted for the token sent with every `Subscribe` (see
    /// [`AuthTokenProvider`]). The connection is reconnected by the driver; every live
    /// subscription is re-sent automatically when it comes back up.
    pub fn new(url: impl Into<String>, auth: AuthTokenProvider) -> Self {
        let url = url.into();
        let collections = Rc::new(ValueMut::new(
            HashMap::<WebsocketQueryId, Subscription>::new(),
        ));
        let connection_box = Rc::new(ValueMut::new(None));

        let drop = get_driver().websocket(
            url.as_str(),
            bind!(connection_box, collections, auth, |message| {
                match message {
                    WebsocketMessage::Connection(connection) => {
                        log::info!("ws-collection - connection ...");
                        collections.change(|inner| {
                            for (query_id, subscription) in inner.iter() {
                                send_subscribe(
                                    &connection,
                                    query_id.clone(),
                                    subscription.query.clone(),
                                    &auth,
                                );
                            }
                        });

                        connection_box.set(Some(connection));
                    }
                    WebsocketMessage::Message(message) => {
                        // Tolerant: a single undecodable frame (e.g. server/client protocol
                        // drift) is logged and skipped — connection and other subscriptions live on.
                        let parsed = match WsServerMessageFrom::from_js_json(message) {
                            Ok(p) => p,
                            Err(err) => {
                                log::error!("ws-collection - skipping undecodable frame: {err}");
                                return;
                            }
                        };
                        dispatch(&collections, parsed);
                    }
                    WebsocketMessage::Close => {
                        log::info!("ws-collection - close ...");
                    }
                }
            }),
        );

        Self {
            _drop: drop,
            connection_box,
            collections,
            auth,
        }
    }

    /// Registers a subscription for `query` and starts receiving frames.
    ///
    /// `callback` is invoked for every `Init` / `Set` / `Delete` frame routed to this
    /// subscription's `query_id`. Returns a [`DropResource`] that owns the
    /// subscription: drop it to send an `Unsubscribe` and stop receiving updates.
    /// Higher-level callers normally use [`WsCollection`] rather than this directly.
    pub fn subscribe(&self, query: WsQuery, callback: Callback) -> DropResource {
        let query_id = WebsocketQueryId::next();

        self.collections.change(|inner| {
            inner.insert(
                query_id.clone(),
                Subscription {
                    query: query.clone(),
                    callback,
                },
            );
        });

        let query_id_for_drop = query_id.clone();
        self.send_subscribe(query_id, query);

        let collections = self.collections.clone();
        let connection_box = self.connection_box.clone();
        DropResource::new(move || {
            Self::run_unsubscribe(&collections, &connection_box, query_id_for_drop);
        })
    }

    fn send_subscribe(&self, query_id: WebsocketQueryId, query: WsQuery) {
        let auth = self.auth.clone();
        self.connection_box.change(|opt| {
            if let Some(conn) = opt {
                send_subscribe(conn, query_id, query, &auth);
            } else {
                log::info!("ws-collection - no connection, cannot subscribe");
            }
        });
    }
}

/// Routes a parsed server message. `Batch` is unwrapped recursively (each inner message
/// re-enters here); `Message` is logged at the socket level (not delivered to subscriptions);
/// `Init`/`Set`/`Delete` are dispatched to the matching subscription's callback by `query_id`.
fn dispatch(
    collections: &Rc<ValueMut<HashMap<WebsocketQueryId, Subscription>>>,
    parsed: WsServerMessageFrom,
) {
    match parsed {
        WsServerMessageFrom::Batch(items) => {
            for item in items {
                dispatch(collections, item);
            }
        }
        WsServerMessageFrom::Message(data) => match data.kind {
            WsMessageKind::Error => log::error!(
                "ws-collection - server reported error (query_id={:?}): {}",
                data.query_id,
                data.message,
            ),
            WsMessageKind::Log => log::info!(
                "ws-collection - server log (query_id={:?}): {}",
                data.query_id,
                data.message,
            ),
        },
        other => {
            let Some(query_id) = other.query_id() else {
                return;
            };
            collections.change(move |inner| {
                let Some(sub) = inner.get(&query_id) else {
                    log::warn!("ws-collection - no subscription for query_id={query_id:?}");
                    return;
                };
                (sub.callback)(other);
            });
        }
    }
}

fn send_subscribe(
    connection: &WebsocketConnection,
    query_id: WebsocketQueryId,
    query: WsQuery,
    auth: &AuthTokenProvider,
) {
    let connection = connection.clone();
    let auth = auth.clone();
    spawn(async move {
        let Some(token) = auth() else {
            log::error!("ws-collection - no auth token, deferring subscribe");
            return;
        };

        let message = WsClientMessageTo::subscribe(query_id, token, query);
        connection.send(message);
    });
}

fn map_to_sorted_vec<T: Clone>(opt: Option<HashMap<String, T>>) -> Option<Vec<T>> {
    opt.map(|map| {
        let mut entries: Vec<(String, T)> = map.into_iter().collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        entries.into_iter().map(|(_, v)| v).collect()
    })
}

/// Reconciles an incoming `Set` model (`new`) against the previously stored one
/// (`prev`), returning the model to store. The default merge simply returns
/// `new` (full replacement); callers can opt into a custom merge via
/// [`WsCollection::new_with_merge`] to carry forward fields that live updates
/// omit (e.g. an image URL only present in the initial snapshot).
pub type MergeFn<T> = Rc<dyn Fn(T, &T) -> T>;

fn default_merge<T>() -> MergeFn<T> {
    Rc::new(|new, _prev| new)
}

/// A single server-pushed subscription reconciled into a reactive list.
///
/// Wraps one [`WsSocket::subscribe`] and folds its `Init`/`Set`/`Delete` frames into a
/// key→row map, exposed as [`items_sorted`](Self::items_sorted). The socket
/// subscription is kept alive by a [`DropResource`] captured inside the
/// `items_sorted` `Computed`, so callers typically destructure the struct and keep
/// only `items_sorted` — the subscription lives exactly as long as that `Computed`.
///
/// Pick a constructor by how the query is supplied:
/// - [`new`](Self::new) / [`new_with_merge`](Self::new_with_merge) — a fixed,
///   long-lived query known up front.
/// - [`empty`](Self::empty) + [`set_query`](Self::set_query) — the query changes over
///   time (e.g. a filter); each swap clears the list back to loading (`None`).
/// - [`extend_query`](Self::extend_query) — swap the query *without* clearing, so the
///   current rows stay visible until the new snapshot lands (e.g. growing a page).
pub struct WsCollection<T: Clone + PartialEq + 'static> {
    /// Snapshot of the current rows as a vector sorted by key — stable ordering
    /// across re-issues so callers comparing `PartialEq` see consistent results.
    /// `None` until the first `Init` snapshot arrives (i.e. the loading state).
    pub items_sorted: Computed<Option<Vec<T>>>,
    socket: Rc<WsSocket>,
    state: Value<Option<HashMap<String, T>>>,
    active_drop: Rc<RefCell<Option<DropResource>>>,
    merge: MergeFn<T>,
}

impl<T: Clone + PartialEq + JsJsonDeserialize + 'static> WsCollection<T> {
    /// Static, long-lived subscription. Callers commonly destructure the
    /// struct and keep only `items_sorted`; the `DropResource` is captured by
    /// the `map` closure so it stays alive for as long as the Computed (and
    /// thus the underlying socket subscribe) does.
    pub fn new(socket: Rc<WsSocket>, query: WsQuery) -> Self {
        Self::new_with_merge(socket, query, default_merge())
    }

    /// Like [`Self::new`], but reconciles each `Set` against the previously
    /// stored row via `merge` instead of replacing it outright. Used where live
    /// updates omit fields present at init (e.g. an image only sent in the snapshot).
    pub fn new_with_merge(socket: Rc<WsSocket>, query: WsQuery, merge: MergeFn<T>) -> Self {
        let state: Value<Option<HashMap<String, T>>> = Value::new(None);
        let keepalive = socket.subscribe(
            query,
            Self::make_message_callback(state.clone(), merge.clone()),
        );
        let items_sorted = state.clone().map(move |opt| {
            let _ = &keepalive;
            map_to_sorted_vec(opt)
        });
        Self {
            items_sorted,
            socket,
            state,
            active_drop: Rc::new(RefCell::new(None)),
            merge,
        }
    }

    /// Construct with no active subscription. Caller must hold the struct
    /// alive (e.g. inside an `Rc`) and invoke [`Self::set_query`] to start
    /// receiving rows; the socket subscribe lives as long as the struct.
    pub fn empty(socket: Rc<WsSocket>) -> Self {
        let state: Value<Option<HashMap<String, T>>> = Value::new(None);
        let items_sorted = state.clone().map(map_to_sorted_vec);
        Self {
            items_sorted,
            socket,
            state,
            active_drop: Rc::new(RefCell::new(None)),
            merge: default_merge(),
        }
    }

    /// Replace the active subscription with one driven by `query`. Only valid
    /// for collections built via [`Self::empty`]; the row map is cleared back
    /// to `None` (loading) when actually re-issuing, and live `Set`/`Delete`
    /// updates resume once the new `Init` arrives.
    pub fn set_query(&self, query: WsQuery) {
        let had_previous = self.active_drop.replace(None).is_some();
        if had_previous {
            self.state.set(None);
        }
        let drop = self.socket.subscribe(
            query,
            Self::make_message_callback(self.state.clone(), self.merge.clone()),
        );
        self.active_drop.replace(Some(drop));
    }

    /// Replace the active subscription with `query` without resetting `state`
    /// to `None`. The current row map stays visible until the new `Init` lands
    /// and replaces it — used by pagination so growing the window doesn't
    /// flash the list to a loading state.
    pub fn extend_query(&self, query: WsQuery) {
        self.active_drop.replace(None);
        let drop = self.socket.subscribe(
            query,
            Self::make_message_callback(self.state.clone(), self.merge.clone()),
        );
        self.active_drop.replace(Some(drop));
    }

    fn make_message_callback(
        state: Value<Option<HashMap<String, T>>>,
        merge: MergeFn<T>,
    ) -> Callback {
        Rc::new(move |message| match message {
            WsServerMessageFrom::Init(data) => {
                let mut map = HashMap::new();
                for item in data.list {
                    match crate::from_json::<T>(item.model) {
                        Ok(model) => {
                            map.insert(item.id, model);
                        }
                        Err(err) => {
                            log::error!("ws-collection - init: skip row id={:?}: {err}", item.id);
                        }
                    }
                }
                state.set(Some(map));
            }
            WsServerMessageFrom::Set(data) => match crate::from_json::<T>(data.model) {
                Ok(model) => {
                    let merge = merge.clone();
                    state.change(move |opt| {
                        let map = opt.get_or_insert_with(HashMap::new);
                        let merged = match map.get(&data.model_id) {
                            Some(prev) => (merge)(model, prev),
                            None => model,
                        };
                        map.insert(data.model_id, merged);
                    });
                }
                Err(err) => {
                    log::error!("ws-collection - set: could not decode model: {err}");
                }
            },
            WsServerMessageFrom::Delete(data) => {
                state.change(|opt| {
                    if let Some(map) = opt {
                        map.remove(&data.model_id);
                    }
                });
            }
            WsServerMessageFrom::Batch(_) | WsServerMessageFrom::Message(_) => {}
        })
    }
}
