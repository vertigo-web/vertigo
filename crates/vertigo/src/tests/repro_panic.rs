use std::rc::Rc;
use vertigo::render::collection::{Collection, CollectionKey};
use vertigo::{Value, transaction};

#[derive(Clone, PartialEq, Debug, Default)]
struct Item {
    id: i32,
    value: i32,
}

struct ItemKey;
impl CollectionKey for ItemKey {
    type Key = i32;
    type Value = Item;
    fn get_key(val: &Item) -> i32 {
        val.id
    }
}

#[test]
fn test_collection_new_in_refresh() {
    let val = Value::new(1);

    // Create a computed that creates a Collection inside its map
    let comp = val.to_computed().map(|v| {
        println!("Computing map for {}", v);
        let list = Rc::new(vec![Item { id: v, value: 10 }]);

        // This should pass with the fix
        let _col: Collection<ItemKey> = Collection::new(list);
        v
    });

    let _sub = comp.subscribe(|_| {});

    println!("Triggering update...");
    transaction(|_| {
        val.set(2);
    });
    println!("Update triggered.");
}

#[test]
fn test_duplicate_keys_in_refresh() {
    let val = Value::new(1);

    let comp = val.to_computed().map(|v| {
        println!("Computing map for {}", v);
        // List with duplicate keys!
        let list = Rc::new(vec![
            Item { id: 100, value: 1 },
            Item { id: 100, value: 2 }, // Duplicate key '100'
        ]);

        let _col: Collection<ItemKey> = Collection::new(list);
        v
    });

    let _sub = comp.subscribe(|_| {});

    println!("Triggering update...");
    transaction(|_| {
        val.set(2);
    });
    println!("Update triggered.");
}
