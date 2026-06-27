use crate::AutoJsJson;

use super::types::{WebsocketQueryId, WsQuery, WsSubscribeQuery};

/// Outgoing message to the server (enum format: one JSON key = one variant).
///
/// - **Subscribe:** `{"Subscribe":{"query_id","auth","query"}}`
/// - **Unsubscribe:** `{"Unsubscribe":{"query_id"}}`
#[derive(Debug, Clone, AutoJsJson, PartialEq)]
pub enum WsClientMessageTo {
    Subscribe {
        query_id: WebsocketQueryId,
        auth: String,
        query: WsSubscribeQuery,
    },
    Unsubscribe {
        query_id: WebsocketQueryId,
    },
}

impl WsClientMessageTo {
    pub fn subscribe(query_id: WebsocketQueryId, auth: String, query: WsQuery) -> Self {
        Self::Subscribe {
            query_id,
            auth,
            query: query.to_ws_subscribe_query(),
        }
    }

    pub fn unsubscribe(query_id: WebsocketQueryId) -> Self {
        Self::Unsubscribe { query_id }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{JsJson, JsJsonContext, JsJsonDeserialize, JsJsonNumber, to_json};

    use super::{
        super::types::{
            WebsocketQueryId, WsSubscribeQuery, WsWhereClause, WsWhereLogic, WsWhereOp,
        },
        *,
    };

    fn ctx() -> JsJsonContext {
        JsJsonContext::new("")
    }

    #[test]
    fn ws_client_subscribe_serializes_as_subscribe_variant_object() {
        let msg = WsClientMessageTo::Subscribe {
            query_id: WebsocketQueryId("ddssaa333333".into()),
            auth: "token-abc".into(),
            query: WsSubscribeQuery {
                table: "items".into(),
                logic: WsWhereLogic::And,
                filters: vec![WsWhereClause {
                    op: WsWhereOp::Eq,
                    column: "category".into(),
                    value: JsJson::String("alpha".into()),
                }],
                limit: None,
            },
        };

        let expected = JsJson::Object(BTreeMap::from([(
            "Subscribe".into(),
            JsJson::Object(BTreeMap::from([
                ("auth".into(), JsJson::String("token-abc".into())),
                (
                    "query".into(),
                    JsJson::Object(BTreeMap::from([
                        ("logic".into(), JsJson::String("And".into())),
                        ("table".into(), JsJson::String("items".into())),
                        (
                            "where".into(),
                            JsJson::List(vec![JsJson::Object(BTreeMap::from([
                                ("column".into(), JsJson::String("category".into())),
                                ("op".into(), JsJson::String("Eq".into())),
                                ("value".into(), JsJson::String("alpha".into())),
                            ]))]),
                        ),
                    ])),
                ),
                ("query_id".into(), JsJson::String("ddssaa333333".into())),
            ])),
        )]));

        assert_eq!(to_json(msg), expected);
    }

    #[test]
    fn ws_subscribe_query_uses_where_json_key_not_filters() {
        let msg = WsClientMessageTo::Subscribe {
            query_id: WebsocketQueryId("1".into()),
            auth: String::new(),
            query: WsSubscribeQuery {
                table: "items".into(),
                logic: WsWhereLogic::And,
                filters: vec![],
                limit: None,
            },
        };

        let expected = JsJson::Object(BTreeMap::from([(
            "Subscribe".into(),
            JsJson::Object(BTreeMap::from([
                ("auth".into(), JsJson::String(String::new())),
                (
                    "query".into(),
                    JsJson::Object(BTreeMap::from([
                        ("logic".into(), JsJson::String("And".into())),
                        ("table".into(), JsJson::String("items".into())),
                        ("where".into(), JsJson::List(vec![])),
                    ])),
                ),
                ("query_id".into(), JsJson::String("1".into())),
            ])),
        )]));

        let json = to_json(msg);
        assert_eq!(json, expected);
        let map = json.get_hashmap(&ctx()).expect("root object");
        let subscribe = map.get("Subscribe").expect("Subscribe").clone();
        let subscribe_map = subscribe.get_hashmap(&ctx()).expect("payload");
        let query = subscribe_map.get("query").expect("query").clone();
        let qmap = query.get_hashmap(&ctx()).expect("query object");
        assert!(
            !qmap.contains_key("filters"),
            "Rust field `filters` must not appear as `filters` in JSON"
        );
    }

    #[test]
    fn ws_client_subscribe_round_trips_through_js_json() {
        let msg = WsClientMessageTo::Subscribe {
            query_id: WebsocketQueryId("q9".into()),
            auth: "jwt".into(),
            query: WsSubscribeQuery {
                table: "items".into(),
                logic: WsWhereLogic::And,
                filters: vec![WsWhereClause {
                    op: WsWhereOp::Eq,
                    column: "c".into(),
                    value: JsJson::False,
                }],
                limit: None,
            },
        };

        let json = to_json(msg.clone());
        let back = WsClientMessageTo::from_json(ctx(), json).expect("deserialize");
        assert_eq!(back, msg);
    }

    #[test]
    fn ws_client_unsubscribe_serializes_and_round_trips() {
        let msg = WsClientMessageTo::Unsubscribe {
            query_id: WebsocketQueryId("query_7".into()),
        };

        let expected = JsJson::Object(BTreeMap::from([(
            "Unsubscribe".into(),
            JsJson::Object(BTreeMap::from([(
                "query_id".into(),
                JsJson::String("query_7".into()),
            )])),
        )]));

        let wire = to_json(msg.clone());
        assert_eq!(wire, expected.clone());
        let back = WsClientMessageTo::from_json(ctx(), wire).expect("deserialize");
        assert_eq!(back, msg);
    }

    #[test]
    fn ws_subscribe_serializes_like_and_logic_and_gt() {
        let msg = WsClientMessageTo::Subscribe {
            query_id: WebsocketQueryId("text-search-1".into()),
            auth: "<JWT>".into(),
            query: WsSubscribeQuery {
                table: "items".into(),
                logic: WsWhereLogic::And,
                filters: vec![WsWhereClause {
                    op: WsWhereOp::Like,
                    column: "label".into(),
                    value: JsJson::String("Ars".into()),
                }],
                limit: None,
            },
        };
        let json = to_json(msg);
        let root = json.get_hashmap(&ctx()).unwrap();
        let sub = root
            .get("Subscribe")
            .unwrap()
            .clone()
            .get_hashmap(&ctx())
            .unwrap();
        let query = sub
            .get("query")
            .unwrap()
            .clone()
            .get_hashmap(&ctx())
            .unwrap();
        assert_eq!(query.get("logic"), Some(&JsJson::String("And".into())));
        let where_list = query.get("where").unwrap().clone();
        let JsJson::List(rows) = where_list else {
            panic!("where list");
        };
        assert_eq!(rows.len(), 1);

        let msg2 = WsClientMessageTo::Subscribe {
            query_id: WebsocketQueryId("query-1".into()),
            auth: "<JWT>".into(),
            query: WsSubscribeQuery {
                table: "items".into(),
                logic: WsWhereLogic::And,
                filters: vec![
                    WsWhereClause {
                        op: WsWhereOp::Eq,
                        column: "owner_id".into(),
                        value: JsJson::Number(JsJsonNumber(42.0)),
                    },
                    WsWhereClause {
                        op: WsWhereOp::Gt,
                        column: "amount".into(),
                        value: JsJson::Number(JsJsonNumber(100.0)),
                    },
                ],
                limit: None,
            },
        };
        let json2 = to_json(msg2.clone());
        let root2 = json2.clone().get_hashmap(&ctx()).unwrap();
        let sub2 = root2
            .get("Subscribe")
            .unwrap()
            .clone()
            .get_hashmap(&ctx())
            .unwrap();
        let query2 = sub2
            .get("query")
            .unwrap()
            .clone()
            .get_hashmap(&ctx())
            .unwrap();
        assert_eq!(query2.get("logic"), Some(&JsJson::String("And".into())));
        let back2 = WsClientMessageTo::from_json(ctx(), json2).expect("deserialize");
        assert_eq!(back2, msg2);
    }
}
