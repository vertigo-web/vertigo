use std::collections::HashMap;
use std::sync::Mutex;

use actix_web::{HttpResponse, Responder, web};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: u32,
    pub name: String,
}

#[derive(Deserialize)]
pub struct NewItem {
    pub name: String,
}

pub struct ItemsState {
    pub items: HashMap<u32, Item>,
    pub next_id: u32,
}

pub type ItemsData = web::Data<Mutex<ItemsState>>;

pub fn new_state() -> ItemsData {
    let mut items = HashMap::new();
    for (id, name) in [(1u32, "Apples"), (2, "Bread"), (3, "Coffee")] {
        items.insert(
            id,
            Item {
                id,
                name: name.to_string(),
            },
        );
    }
    web::Data::new(Mutex::new(ItemsState { items, next_id: 4 }))
}

fn sorted_items(state: &ItemsState) -> Vec<Item> {
    let mut list: Vec<Item> = state.items.values().cloned().collect();
    list.sort_by_key(|i| i.id);
    list
}

pub async fn list(data: ItemsData) -> impl Responder {
    let state = data.lock().unwrap();
    HttpResponse::Ok().json(sorted_items(&state))
}

pub async fn get_one(id: web::Path<u32>, data: ItemsData) -> impl Responder {
    let state = data.lock().unwrap();
    match state.items.get(&id.into_inner()) {
        Some(item) => HttpResponse::Ok().json(item),
        None => HttpResponse::NotFound().finish(),
    }
}

pub async fn create(body: web::Json<NewItem>, data: ItemsData) -> impl Responder {
    let mut state = data.lock().unwrap();
    let id = state.next_id;
    state.next_id += 1;
    let item = Item {
        id,
        name: body.into_inner().name,
    };
    state.items.insert(id, item.clone());
    HttpResponse::Created().json(item)
}

pub async fn update(
    id: web::Path<u32>,
    body: web::Json<NewItem>,
    data: ItemsData,
) -> impl Responder {
    let id = id.into_inner();
    let mut state = data.lock().unwrap();
    match state.items.get_mut(&id) {
        Some(item) => {
            item.name = body.into_inner().name;
            HttpResponse::Ok().json(item.clone())
        }
        None => HttpResponse::NotFound().finish(),
    }
}

pub async fn delete(id: web::Path<u32>, data: ItemsData) -> impl Responder {
    let mut state = data.lock().unwrap();
    match state.items.remove(&id.into_inner()) {
        Some(_) => HttpResponse::NoContent().finish(),
        None => HttpResponse::NotFound().finish(),
    }
}
