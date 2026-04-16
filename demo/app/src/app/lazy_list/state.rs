use vertigo::{
    AutoJsJson, CollectionKey, FetchMethod, LazyListCache, RequestBuilder, Value, get_driver,
    store, transaction,
};

#[derive(Debug, AutoJsJson, PartialEq, Eq, Clone)]
pub struct Item {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, AutoJsJson, PartialEq, Eq, Clone)]
pub struct NewItemDto {
    pub name: String,
}

#[derive(PartialEq)]
pub struct ItemKey;

impl CollectionKey for ItemKey {
    type Key = u32;
    type Value = Item;
    fn get_key(val: &Item) -> u32 {
        val.id
    }
}

#[store]
pub fn state_items() -> LazyListCache<ItemKey> {
    RequestBuilder::get("/api/items")
        .lazy_list_cache::<ItemKey>(|status, body| {
            if status == 200 {
                Some(body.into::<Vec<Item>>())
            } else {
                None
            }
        })
        .with_item_fetch(
            |id: &u32| RequestBuilder::get(format!("/api/items/{id}")),
            |status, body| {
                if status == 200 {
                    Some(body.into::<Item>())
                } else {
                    None
                }
            },
        )
}

#[store]
pub fn state_new_name() -> Value<String> {
    Value::default()
}

#[store]
pub fn state_editing() -> Value<Option<u32>> {
    Value::new(None)
}

#[store]
pub fn state_edit_buffer() -> Value<String> {
    Value::default()
}

#[store]
pub fn state_status() -> Value<Option<String>> {
    Value::new(None)
}

const PLACEHOLDER_ID: u32 = 0;

pub fn create_item(name: String) {
    if name.is_empty() {
        return;
    }
    let cache = state_items();
    cache.optimistically_set_item(Item {
        id: PLACEHOLDER_ID,
        name: name.clone(),
    });
    state_new_name().set(String::new());
    state_status().set(None);

    get_driver().spawn(async move {
        let response = RequestBuilder::post("/api/items")
            .body_json(NewItemDto { name })
            .call()
            .await;

        match response.into(|status, body| {
            if status == 201 || status == 200 {
                Some(body.into::<Item>())
            } else {
                None
            }
        }) {
            Ok(item) => {
                cache.update_item_with_old_key(&PLACEHOLDER_ID, item);
            }
            Err(err) => {
                cache.rollback(&PLACEHOLDER_ID);
                state_status().set(Some(format!("Create failed: {err}")));
            }
        }
    });
}

pub fn save_edit(id: u32, name: String) {
    let cache = state_items();
    cache.optimistically_set_item(Item {
        id,
        name: name.clone(),
    });
    transaction(|_| {
        state_editing().set(None);
        state_edit_buffer().set(String::new());
        state_status().set(None);
    });

    get_driver().spawn(async move {
        let response = RequestBuilder::new(FetchMethod::PUT, format!("/api/items/{id}"))
            .body_json(NewItemDto { name })
            .call()
            .await;

        match response.into(|status, body| {
            if status == 200 {
                Some(body.into::<Item>())
            } else {
                None
            }
        }) {
            Ok(item) => {
                cache.update_item(item);
            }
            Err(err) => {
                cache.rollback(&id);
                state_status().set(Some(format!("Update failed: {err}")));
            }
        }
    });
}

pub fn cancel_edit(id: u32) {
    let cache = state_items();
    cache.rollback(&id);
    transaction(|_| {
        state_editing().set(None);
        state_edit_buffer().set(String::new());
    });
}

pub fn begin_edit(item: &Item) {
    transaction(|_| {
        state_edit_buffer().set(item.name.clone());
        state_editing().set(Some(item.id));
    });
}

pub fn delete_item(id: u32) {
    let cache = state_items();
    cache.optimistically_remove_item(&id);
    state_status().set(None);

    get_driver().spawn(async move {
        let response = RequestBuilder::new(FetchMethod::DELETE, format!("/api/items/{id}"))
            .call()
            .await;

        let status = response.status();
        if status == Some(204) || status == Some(200) {
            cache.remove_item(&id);
        } else {
            cache.rollback(&id);
            state_status().set(Some(format!(
                "Delete failed: status {}",
                status
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "network error".into())
            )));
        }
    });
}

pub fn refresh() {
    state_status().set(None);
    state_items().force_update(true);
}
