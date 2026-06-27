use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{AutoJsJson, JsJson, JsJsonContext, JsJsonDeserialize, JsJsonNumber, JsJsonSerialize};

static WEBSOCKET_QUERY_ID_SEQ: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub enum CollectionWhereValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

/// Converts a value into [`CollectionWhereValue`] for [`WsQuery::eq`] / [`WsQuery::gt`] / [`WsQuery::lt`].
pub trait IntoCollectionWhereValue {
    fn into_collection_where_value(self) -> CollectionWhereValue;
}

impl IntoCollectionWhereValue for CollectionWhereValue {
    fn into_collection_where_value(self) -> CollectionWhereValue {
        self
    }
}

impl IntoCollectionWhereValue for String {
    fn into_collection_where_value(self) -> CollectionWhereValue {
        CollectionWhereValue::String(self)
    }
}

impl IntoCollectionWhereValue for &str {
    fn into_collection_where_value(self) -> CollectionWhereValue {
        CollectionWhereValue::String(self.to_string())
    }
}

impl IntoCollectionWhereValue for f64 {
    fn into_collection_where_value(self) -> CollectionWhereValue {
        CollectionWhereValue::Number(self)
    }
}

impl IntoCollectionWhereValue for f32 {
    fn into_collection_where_value(self) -> CollectionWhereValue {
        CollectionWhereValue::Number(f64::from(self))
    }
}

impl IntoCollectionWhereValue for i64 {
    fn into_collection_where_value(self) -> CollectionWhereValue {
        CollectionWhereValue::Number(self as f64)
    }
}

impl IntoCollectionWhereValue for i32 {
    fn into_collection_where_value(self) -> CollectionWhereValue {
        CollectionWhereValue::Number(f64::from(self))
    }
}

impl IntoCollectionWhereValue for u64 {
    fn into_collection_where_value(self) -> CollectionWhereValue {
        CollectionWhereValue::Number(self as f64)
    }
}

impl IntoCollectionWhereValue for u32 {
    fn into_collection_where_value(self) -> CollectionWhereValue {
        CollectionWhereValue::Number(f64::from(self))
    }
}

impl IntoCollectionWhereValue for bool {
    fn into_collection_where_value(self) -> CollectionWhereValue {
        CollectionWhereValue::Boolean(self)
    }
}

#[derive(Clone, Debug)]
pub enum CollectionWhere {
    Eq {
        column: String,
        value: CollectionWhereValue,
    },
    /// `LIKE` pattern — wire `value` is always a JSON string.
    Like { column: String, value: String },
    Gt {
        column: String,
        value: CollectionWhereValue,
    },
    Lt {
        column: String,
        value: CollectionWhereValue,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, AutoJsJson)]
pub enum WsWhereLogic {
    And,
    Or,
}

/// Builder for a server subscription query.
///
/// Start with [`and`](Self::and) or [`or`](Self::or) — naming the table and the
/// top-level boolean logic combining the `where` clauses — then chain
/// [`eq`](Self::eq) / [`like`](Self::like) / [`gt`](Self::gt) / [`lt`](Self::lt) /
/// [`eq_null`](Self::eq_null) to append filters and [`limit`](Self::limit) to cap the
/// row count. Converted to the wire form (`{ table, logic, where: [...], limit }`) by
/// [`to_ws_subscribe_query`](Self::to_ws_subscribe_query) when the subscription is sent.
#[derive(Clone, Debug)]
pub struct WsQuery {
    table: String,
    logic: WsWhereLogic,
    where_list: Vec<CollectionWhere>,
    limit: Option<usize>,
}

impl WsQuery {
    /// Subscribe query with `"logic": "And"` — combine `where` clauses with AND.
    pub fn and(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            logic: WsWhereLogic::And,
            where_list: Vec::new(),
            limit: None,
        }
    }

    /// Subscribe query with `"logic": "Or"` — combine `where` clauses with OR.
    pub fn or(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            logic: WsWhereLogic::Or,
            where_list: Vec::new(),
            limit: None,
        }
    }

    /// Override the server-default row limit. The server clamps to its own MAX.
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn eq<V: IntoCollectionWhereValue>(mut self, column: impl Into<String>, value: V) -> Self {
        self.where_list.push(CollectionWhere::Eq {
            column: column.into(),
            value: value.into_collection_where_value(),
        });
        self
    }

    pub fn like(mut self, column: impl Into<String>, pattern: impl Into<String>) -> Self {
        self.where_list.push(CollectionWhere::Like {
            column: column.into(),
            value: pattern.into(),
        });
        self
    }

    pub fn gt<V: IntoCollectionWhereValue>(mut self, column: impl Into<String>, value: V) -> Self {
        self.where_list.push(CollectionWhere::Gt {
            column: column.into(),
            value: value.into_collection_where_value(),
        });
        self
    }

    pub fn lt<V: IntoCollectionWhereValue>(mut self, column: impl Into<String>, value: V) -> Self {
        self.where_list.push(CollectionWhere::Lt {
            column: column.into(),
            value: value.into_collection_where_value(),
        });
        self
    }

    /// Adds `WHERE column IS NULL` — filters to rows where the column has no value.
    pub fn eq_null(mut self, column: impl Into<String>) -> Self {
        self.where_list.push(CollectionWhere::Eq {
            column: column.into(),
            value: CollectionWhereValue::Null,
        });
        self
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, AutoJsJson)]
pub struct WebsocketQueryId(pub(crate) String);

impl WebsocketQueryId {
    pub fn next() -> Self {
        let n = WEBSOCKET_QUERY_ID_SEQ.fetch_add(1, Ordering::Relaxed);
        Self(format!("query_{n}"))
    }
}

/// Operator in the `where` clause (the `"op"` field).
#[derive(Debug, Clone, AutoJsJson, PartialEq, Eq)]
pub enum WsWhereOp {
    Eq,
    Like,
    Gt,
    Lt,
}

/// One entry from the `"where"` array in the `query` object.
#[derive(Debug, Clone, AutoJsJson, PartialEq)]
pub struct WsWhereClause {
    pub op: WsWhereOp,
    pub column: String,
    pub value: JsJson,
}

/// The `"query"` object in the Subscribe message.
///
/// `logic` is always serialized (AND/OR). Serialized manually so the `where` field uses the JSON key `"where"`.
#[derive(Debug, Clone, PartialEq)]
pub struct WsSubscribeQuery {
    pub table: String,
    pub logic: WsWhereLogic,
    pub filters: Vec<WsWhereClause>,
    pub limit: Option<usize>,
}

impl JsJsonSerialize for WsSubscribeQuery {
    fn to_json(self) -> JsJson {
        let mut map = BTreeMap::new();
        map.insert("table".to_string(), self.table.to_json());
        map.insert("logic".to_string(), self.logic.to_json());
        map.insert(
            "where".to_string(),
            JsJson::List(self.filters.into_iter().map(|c| c.to_json()).collect()),
        );
        if let Some(limit) = self.limit {
            map.insert(
                "limit".to_string(),
                JsJson::Number(JsJsonNumber(limit as f64)),
            );
        }
        JsJson::Object(map)
    }
}

impl JsJsonDeserialize for WsSubscribeQuery {
    fn from_json(context: JsJsonContext, json: JsJson) -> Result<Self, JsJsonContext> {
        let map = json.get_hashmap(&context)?;
        let table_json = map
            .get("table")
            .cloned()
            .ok_or_else(|| context.add("missing field 'table'"))?;
        let table = String::from_json(context.add("field: 'table'"), table_json)?;

        let logic_json = map
            .get("logic")
            .cloned()
            .ok_or_else(|| context.add("missing field 'logic'"))?;
        let logic = WsWhereLogic::from_json(context.add("field: 'logic'"), logic_json)?;

        let where_json = map
            .get("where")
            .cloned()
            .ok_or_else(|| context.add("missing field 'where'"))?;
        let filters = match where_json {
            JsJson::List(items) => {
                let mut out = Vec::with_capacity(items.len());
                for (i, item) in items.into_iter().enumerate() {
                    let ctx = context.add(format!("where[{i}]"));
                    out.push(WsWhereClause::from_json(ctx, item)?);
                }
                out
            }
            other => {
                return Err(context.add(format!("where must be a list, got {}", other.typename())));
            }
        };

        let limit = match map.get("limit") {
            None | Some(JsJson::Null) => None,
            Some(JsJson::Number(JsJsonNumber(n))) => Some(*n as usize),
            Some(other) => {
                return Err(context.add(format!(
                    "limit must be a number or absent, got {}",
                    other.typename()
                )));
            }
        };

        Ok(Self {
            table,
            logic,
            filters,
            limit,
        })
    }
}

impl WsQuery {
    pub fn to_ws_subscribe_query(&self) -> WsSubscribeQuery {
        WsSubscribeQuery {
            table: self.table.clone(),
            logic: self.logic,
            filters: self.where_list.iter().map(where_clause_to_wire).collect(),
            limit: self.limit,
        }
    }
}

fn where_clause_to_wire(w: &CollectionWhere) -> WsWhereClause {
    match w {
        CollectionWhere::Eq { column, value } => WsWhereClause {
            op: WsWhereOp::Eq,
            column: column.clone(),
            value: collection_where_value_to_js_json(value),
        },
        CollectionWhere::Like { column, value } => WsWhereClause {
            op: WsWhereOp::Like,
            column: column.clone(),
            value: JsJson::String(value.clone()),
        },
        CollectionWhere::Gt { column, value } => WsWhereClause {
            op: WsWhereOp::Gt,
            column: column.clone(),
            value: collection_where_value_to_js_json(value),
        },
        CollectionWhere::Lt { column, value } => WsWhereClause {
            op: WsWhereOp::Lt,
            column: column.clone(),
            value: collection_where_value_to_js_json(value),
        },
    }
}

fn collection_where_value_to_js_json(value: &CollectionWhereValue) -> JsJson {
    match value {
        CollectionWhereValue::String(s) => JsJson::String(s.clone()),
        CollectionWhereValue::Number(n) => JsJson::Number(JsJsonNumber(*n)),
        CollectionWhereValue::Boolean(b) => {
            if *b {
                JsJson::True
            } else {
                JsJson::False
            }
        }
        CollectionWhereValue::Null => JsJson::Null,
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{JsJson, JsJsonContext, JsJsonNumber, to_json};

    use super::*;

    fn ctx() -> JsJsonContext {
        JsJsonContext::new("")
    }

    #[test]
    fn ws_where_clause_serializes_op_column_value() -> Result<(), Box<dyn Error>> {
        let clause = WsWhereClause {
            op: WsWhereOp::Eq,
            column: "category".into(),
            value: JsJson::String("alpha".into()),
        };

        let json = to_json(clause);
        let map = json.get_hashmap(&ctx())?;

        assert_eq!(map.get("op"), Some(&JsJson::String("Eq".into())));
        assert_eq!(map.get("column"), Some(&JsJson::String("category".into())));
        assert_eq!(map.get("value"), Some(&JsJson::String("alpha".into())));
        Ok(())
    }

    #[test]
    fn query_to_ws_subscribe_query_preserves_value_kinds() -> Result<(), Box<dyn Error>> {
        let query = WsQuery {
            table: "items".into(),
            logic: WsWhereLogic::And,
            limit: None,
            where_list: vec![
                CollectionWhere::Eq {
                    column: "category".into(),
                    value: CollectionWhereValue::String("alpha".into()),
                },
                CollectionWhere::Eq {
                    column: "score".into(),
                    value: CollectionWhereValue::Number(3.5),
                },
                CollectionWhere::Eq {
                    column: "active".into(),
                    value: CollectionWhereValue::Boolean(true),
                },
            ],
        };

        let wire = query.to_ws_subscribe_query();
        let json = to_json(wire);
        let qmap = json.get_hashmap(&ctx())?;
        let JsJson::List(rows) = qmap.get("where").ok_or("where array")? else {
            return Err("where must be a list".into());
        };
        assert_eq!(rows.len(), 3);

        let row0 = rows[0].clone().get_hashmap(&ctx())?;
        assert_eq!(row0.get("value"), Some(&JsJson::String("alpha".into())));

        let row1 = rows[1].clone().get_hashmap(&ctx())?;
        assert_eq!(row1.get("value"), Some(&JsJson::Number(JsJsonNumber(3.5))));

        let row2 = rows[2].clone().get_hashmap(&ctx())?;
        assert_eq!(row2.get("value"), Some(&JsJson::True));
        Ok(())
    }

    #[test]
    fn websocket_query_id_serializes_as_plain_string() {
        let id = WebsocketQueryId("query_0".into());
        assert_eq!(to_json(id), JsJson::String("query_0".into()));
    }

    #[test]
    fn subscribe_query_like_text_column_serializes_logic_and_where() -> Result<(), Box<dyn Error>> {
        let query = WsQuery::and("items").like("label", "Ars");
        let wire = query.to_ws_subscribe_query();
        let json = to_json(wire);
        let qmap = json.get_hashmap(&ctx())?;
        assert_eq!(qmap.get("logic"), Some(&JsJson::String("And".into())));
        let JsJson::List(rows) = qmap.get("where").ok_or("where array")? else {
            return Err("where must be a list".into());
        };
        assert_eq!(rows.len(), 1);
        let row = rows[0].clone().get_hashmap(&ctx())?;
        assert_eq!(row.get("op"), Some(&JsJson::String("Like".into())));
        assert_eq!(row.get("column"), Some(&JsJson::String("label".into())));
        assert_eq!(row.get("value"), Some(&JsJson::String("Ars".into())));
        Ok(())
    }

    #[test]
    fn subscribe_query_and_eq_gt_serializes_logic_and_where() -> Result<(), Box<dyn Error>> {
        let query = WsQuery::and("items")
            .eq("owner_id", 42i64)
            .gt("amount", 100.0);
        let wire = query.to_ws_subscribe_query();
        let json = to_json(wire);
        let qmap = json.get_hashmap(&ctx())?;
        assert_eq!(qmap.get("logic"), Some(&JsJson::String("And".into())));
        let JsJson::List(rows) = qmap.get("where").ok_or("where array")? else {
            return Err("where must be a list".into());
        };
        assert_eq!(rows.len(), 2);
        let row0 = rows[0].clone().get_hashmap(&ctx())?;
        assert_eq!(row0.get("op"), Some(&JsJson::String("Eq".into())));
        assert_eq!(row0.get("value"), Some(&JsJson::Number(JsJsonNumber(42.0))));
        let row1 = rows[1].clone().get_hashmap(&ctx())?;
        assert_eq!(row1.get("op"), Some(&JsJson::String("Gt".into())));
        assert_eq!(
            row1.get("value"),
            Some(&JsJson::Number(JsJsonNumber(100.0)))
        );
        Ok(())
    }

    #[test]
    fn subscribe_query_lt_serializes_op_and_value() -> Result<(), Box<dyn Error>> {
        let query = WsQuery::and("items").lt("amount", 50.0);
        let wire = query.to_ws_subscribe_query();
        let json = to_json(wire);
        let qmap = json.get_hashmap(&ctx())?;
        assert_eq!(qmap.get("logic"), Some(&JsJson::String("And".into())));
        let JsJson::List(rows) = qmap.get("where").ok_or("where array")? else {
            return Err("where must be a list".into());
        };
        assert_eq!(rows.len(), 1);
        let row = rows[0].clone().get_hashmap(&ctx())?;
        assert_eq!(row.get("op"), Some(&JsJson::String("Lt".into())));
        assert_eq!(row.get("column"), Some(&JsJson::String("amount".into())));
        assert_eq!(row.get("value"), Some(&JsJson::Number(JsJsonNumber(50.0))));
        Ok(())
    }

    #[test]
    fn subscribe_query_new_or_serializes_logic_or() -> Result<(), Box<dyn Error>> {
        let query = WsQuery::or("items").eq("category", "alpha");
        let wire = query.to_ws_subscribe_query();
        let json = to_json(wire);
        let qmap = json.get_hashmap(&ctx())?;
        assert_eq!(qmap.get("logic"), Some(&JsJson::String("Or".into())));
        Ok(())
    }
}
