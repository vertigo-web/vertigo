//! WebSocket endpoint that speaks the `vertigo::WsCollection` protocol.
//!
//! A connected client sends `Subscribe` / `Unsubscribe` frames; this server replies with
//! `init` snapshots and then streams `set` / `delete` frames as the (simulated) catalogue
//! changes. The data is a catalogue of ukuleles; a background timer mutates `stock` and
//! occasionally removes/re-adds a row so the client list visibly updates on its own.
//!
//! The wire shapes are hand-written serde structs that mirror
//! `crates/vertigo/src/websocket_collection/{ws_message_from,ws_message_to,types}.rs`
//! exactly — this crate cannot depend on vertigo's WASM-side protocol types.

use std::collections::HashMap;
use std::time::Duration;

use actix_web::{Error, HttpRequest, HttpResponse, rt, web};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::connection::{Connection, ConnectionStream, SocketError};

const TICK: Duration = Duration::from_millis(2500);

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

#[derive(Clone, Serialize)]
pub struct Ukulele {
    pub id: u32,
    pub kind: String,
    pub name: String,
    pub scale_inches: f64,
    pub tuning: String,
    pub description: String,
    pub stock: u32,
}

impl Ukulele {
    fn to_model(&self) -> Value {
        json!({
            "id": self.id,
            "kind": self.kind,
            "name": self.name,
            "scale_inches": self.scale_inches,
            "tuning": self.tuning,
            "description": self.description,
            "stock": self.stock,
        })
    }
}

/// One row's static facts per ukulele type, faithful to
/// <https://www.ukuleleworld.com/types-of-ukuleles/>.
struct KindSpec {
    kind: &'static str,
    scale_inches: f64,
    tuning: &'static str,
    description: &'static str,
}

const KINDS: &[KindSpec] = &[
    KindSpec {
        kind: "Soprano",
        scale_inches: 21.0,
        tuning: "G-C-E-A",
        description: "The most common ukulele: bright, classic tone at lower volume.",
    },
    KindSpec {
        kind: "Concert",
        scale_inches: 23.0,
        tuning: "G-C-E-A",
        description: "More bass and volume than a soprano while staying portable.",
    },
    KindSpec {
        kind: "Tenor",
        scale_inches: 26.0,
        tuning: "G-C-E-A",
        description: "Louder and fuller, with extended range; a performer favourite.",
    },
    KindSpec {
        kind: "Baritone",
        scale_inches: 30.0,
        tuning: "D-G-B-E",
        description: "Noticeably deeper sound, tuned like the top four guitar strings.",
    },
    KindSpec {
        kind: "Pineapple",
        scale_inches: 21.0,
        tuning: "G-C-E-A",
        description: "Oval body shape for a louder, stronger sound.",
    },
    KindSpec {
        kind: "Banjo",
        scale_inches: 23.0,
        tuning: "G-C-E-A",
        description: "Circular banjo body: loud, bright tone with minimal sustain.",
    },
    KindSpec {
        kind: "Resonator",
        scale_inches: 26.0,
        tuning: "G-C-E-A",
        description: "Metal resonator plates give a notably louder, distinctive tone.",
    },
    KindSpec {
        kind: "Bass",
        scale_inches: 30.0,
        tuning: "E-A-D-G",
        description: "Deep bass voice; needs amplification for adequate volume.",
    },
    KindSpec {
        kind: "Sopranino",
        scale_inches: 12.0,
        tuning: "G-C-E-A",
        description: "Tiny piccolo-scale ukulele with an especially bright voice.",
    },
];

/// A few maker/wood adjectives to vary instance names without a rand dependency.
const FLAVOURS: &[&str] = &["Mahogany", "Koa", "Spruce", "Acacia", "Maple", "Walnut"];

fn seed() -> Vec<Ukulele> {
    let mut out = Vec::new();
    let mut id = 1u32;
    // ~24 instances: a few per kind, names/stock varied deterministically by index.
    for (k_idx, spec) in KINDS.iter().enumerate() {
        let count = 2 + (k_idx % 2); // 2 or 3 instances per kind
        for n in 0..count {
            let flavour = FLAVOURS[(k_idx + n) % FLAVOURS.len()];
            out.push(Ukulele {
                id,
                kind: spec.kind.to_string(),
                name: format!("{flavour} {} #{}", spec.kind, n + 1),
                scale_inches: spec.scale_inches,
                tuning: spec.tuning.to_string(),
                description: spec.description.to_string(),
                stock: 3 + ((id * 7) % 20),
            });
            id += 1;
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Wire protocol — outgoing (server -> client)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct Row {
    id: String,
    model: Value,
}

/// Mirrors `WsServerMessageFrom`: externally-tagged with lowercase variant names and
/// snake_case payload fields (Init/Set/Delete carry no `rename_all`).
#[derive(Serialize)]
enum ServerFrame {
    #[serde(rename = "init")]
    Init { query_id: String, list: Vec<Row> },
    #[serde(rename = "set")]
    Set {
        query_id: String,
        model_id: String,
        model: Value,
    },
    #[serde(rename = "delete")]
    Delete { query_id: String, model_id: String },
}

// ---------------------------------------------------------------------------
// Wire protocol — incoming (client -> server)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
enum ClientFrame {
    Subscribe {
        query_id: String,
        #[allow(dead_code)]
        auth: String,
        query: SubscribeQuery,
    },
    Unsubscribe {
        query_id: String,
    },
}

#[derive(Deserialize, Clone)]
struct SubscribeQuery {
    #[allow(dead_code)]
    table: String,
    logic: Logic,
    #[serde(rename = "where")]
    where_list: Vec<WhereClause>,
    limit: Option<usize>,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
enum Logic {
    And,
    Or,
}

#[derive(Deserialize, Clone)]
struct WhereClause {
    op: WhereOp,
    column: String,
    value: Value,
}

#[derive(Deserialize, Clone, Copy)]
enum WhereOp {
    Eq,
    Like,
    Gt,
    Lt,
}

impl WhereClause {
    fn matches(&self, item: &Ukulele) -> bool {
        let field = field_value(item, &self.column);
        match self.op {
            WhereOp::Eq => values_eq(&field, &self.value),
            WhereOp::Like => {
                let needle = self.value.as_str().unwrap_or_default().to_lowercase();
                field
                    .as_str()
                    .map(|s| s.to_lowercase().contains(&needle))
                    .unwrap_or(false)
            }
            WhereOp::Gt => field_num(&field) > field_num(&self.value),
            WhereOp::Lt => field_num(&field) < field_num(&self.value),
        }
    }
}

fn field_value(item: &Ukulele, column: &str) -> Value {
    match column {
        "id" => json!(item.id),
        "kind" => json!(item.kind),
        "name" => json!(item.name),
        "scale_inches" => json!(item.scale_inches),
        "tuning" => json!(item.tuning),
        "stock" => json!(item.stock),
        _ => Value::Null,
    }
}

fn values_eq(a: &Value, b: &Value) -> bool {
    if let (Some(x), Some(y)) = (a.as_str(), b.as_str()) {
        return x == y;
    }
    match (field_num(a), field_num(b)) {
        (Some(x), Some(y)) => x == y,
        _ => a == b,
    }
}

fn field_num(v: &Value) -> Option<f64> {
    v.as_f64()
}

impl SubscribeQuery {
    fn matches(&self, item: &Ukulele) -> bool {
        if self.where_list.is_empty() {
            return true;
        }
        match self.logic {
            Logic::And => self.where_list.iter().all(|c| c.matches(item)),
            Logic::Or => self.where_list.iter().any(|c| c.matches(item)),
        }
    }

    /// Rows of `catalogue` matching this query, sorted by id, capped by `limit`.
    fn select<'a>(&self, catalogue: &'a HashMap<u32, Ukulele>) -> Vec<&'a Ukulele> {
        let mut rows: Vec<&Ukulele> = catalogue.values().filter(|it| self.matches(it)).collect();
        rows.sort_by_key(|it| it.id);
        if let Some(limit) = self.limit {
            rows.truncate(limit);
        }
        rows
    }
}

// ---------------------------------------------------------------------------
// Per-connection session
// ---------------------------------------------------------------------------

struct Session {
    catalogue: HashMap<u32, Ukulele>,
    /// Insertion order of seeded rows — used to revive a deleted row deterministically.
    order: Vec<u32>,
    subscriptions: HashMap<String, SubscribeQuery>,
    tick: u64,
    /// (id, original-row) parked by a simulated delete, revived a few ticks later.
    parked: Option<(u64, Ukulele)>,
}

impl Session {
    fn new() -> Self {
        let seeded = seed();
        let order: Vec<u32> = seeded.iter().map(|u| u.id).collect();
        let catalogue = seeded.into_iter().map(|u| (u.id, u)).collect();
        Session {
            catalogue,
            order,
            subscriptions: HashMap::new(),
            tick: 0,
            parked: None,
        }
    }

    async fn on_subscribe(
        &mut self,
        conn: &Connection,
        query_id: String,
        query: SubscribeQuery,
    ) -> Result<(), SocketError> {
        let list = query
            .select(&self.catalogue)
            .into_iter()
            .map(|item| Row {
                id: item.id.to_string(),
                model: item.to_model(),
            })
            .collect();
        let frame = ServerFrame::Init {
            query_id: query_id.clone(),
            list,
        };
        conn.send(serde_json::to_string(&frame)?).await?;
        self.subscriptions.insert(query_id, query);
        Ok(())
    }

    fn on_unsubscribe(&mut self, query_id: &str) {
        self.subscriptions.remove(query_id);
    }

    /// Emit a `set` for `item` to every subscription whose filter it matches, and a
    /// `delete` to subscriptions that previously matched but no longer do.
    async fn broadcast_upsert(&self, conn: &Connection, item: &Ukulele) -> Result<(), SocketError> {
        for (query_id, query) in self.subscriptions.iter() {
            let frame = if query.matches(item) {
                ServerFrame::Set {
                    query_id: query_id.clone(),
                    model_id: item.id.to_string(),
                    model: item.to_model(),
                }
            } else {
                ServerFrame::Delete {
                    query_id: query_id.clone(),
                    model_id: item.id.to_string(),
                }
            };
            conn.send(serde_json::to_string(&frame)?).await?;
        }
        Ok(())
    }

    async fn broadcast_delete(&self, conn: &Connection, id: u32) -> Result<(), SocketError> {
        for query_id in self.subscriptions.keys() {
            let frame = ServerFrame::Delete {
                query_id: query_id.clone(),
                model_id: id.to_string(),
            };
            conn.send(serde_json::to_string(&frame)?).await?;
        }
        Ok(())
    }

    /// One simulation step: nudge a row's stock (→ `set`), and every 4th tick park a row
    /// (→ `delete`) then revive the previously-parked one (→ `set`).
    async fn simulate(&mut self, conn: &Connection) -> Result<(), SocketError> {
        self.tick += 1;

        // Revive a row parked two ticks ago.
        if let Some((parked_tick, row)) = self.parked.clone()
            && self.tick >= parked_tick + 2
        {
            self.catalogue.insert(row.id, row.clone());
            self.parked = None;
            self.broadcast_upsert(conn, &row).await?;
        }

        if self.order.is_empty() {
            return Ok(());
        }

        // Every 4th tick, park (delete) a present row if nothing is currently parked.
        if self.tick.is_multiple_of(4) && self.parked.is_none() {
            let idx = (self.tick as usize / 4) % self.order.len();
            let id = self.order[idx];
            if let Some(row) = self.catalogue.remove(&id) {
                self.parked = Some((self.tick, row));
                self.broadcast_delete(conn, id).await?;
                return Ok(());
            }
        }

        // Otherwise mutate one present row's stock and push a `set`.
        let idx = (self.tick as usize) % self.order.len();
        let id = self.order[idx];
        if let Some(item) = self.catalogue.get_mut(&id) {
            item.stock = (item.stock + 1 + (id % 3)) % 40;
            let snapshot = item.clone();
            self.broadcast_upsert(conn, &snapshot).await?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Actix wiring
// ---------------------------------------------------------------------------

pub async fn handler(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (response, session, msg_stream) = actix_ws::handle(&req, stream)?;
    rt::spawn(run(session, msg_stream));
    Ok(response)
}

async fn run(session: actix_ws::Session, msg_stream: actix_ws::MessageStream) {
    let (conn, mut receiver) = ConnectionStream::new(session, msg_stream);
    if let Err(err) = process(&conn, &mut receiver).await {
        println!("WsCollection client disconnected -> {err}");
    }
}

async fn process(conn: &Connection, receiver: &mut ConnectionStream) -> Result<(), SocketError> {
    println!("New ws-collection connection: {}", conn.get_id());
    let mut session = Session::new();
    let mut ticker = tokio::time::interval(TICK);

    loop {
        tokio::select! {
            incoming = receiver.expect_get_text_message() => {
                let text = incoming?;
                match serde_json::from_str::<ClientFrame>(&text) {
                    Ok(ClientFrame::Subscribe { query_id, query, .. }) => {
                        session.on_subscribe(conn, query_id, query).await?;
                    }
                    Ok(ClientFrame::Unsubscribe { query_id }) => {
                        session.on_unsubscribe(&query_id);
                    }
                    Err(err) => {
                        println!("ws-collection: undecodable client frame: {err}; raw={text}");
                    }
                }
            }
            _ = ticker.tick() => {
                session.simulate(conn).await?;
            }
        }
    }
}
