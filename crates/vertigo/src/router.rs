use std::rc::Rc;

use crate::{Driver, computed::{Client, Value}, utils::{BoxRefCell, DropResource}};

#[derive(PartialEq, Clone, Copy)]
enum Direction {
    Loading,
    Pushing,
    Popping,
}

#[derive(PartialEq)]
pub struct HashRouter {
    sender: Client,
    receiver: DropResource,
}

impl HashRouter {
    /// Create new HashRouter which sets route value upon hash change in browser bar.
    /// If callback is provided then it is fired instead.
    pub fn new<T>(driver: &Driver, route: Value<T>, callback: Box<dyn Fn(&String)>) -> Self
    where
        T: PartialEq + ToString
    {
        let direction = Rc::new(BoxRefCell::new(Direction::Loading, "hash router"));

        let sender = route.to_computed().subscribe({
            let driver = driver.clone();
            let direction = direction.clone();
            move |route| {
                let dir = direction.get(|state| *state);
                match dir {
                    // First change is upon page loading, ignore it but accept further pushes
                    Direction::Loading =>
                        direction.change((), |state, _| *state = Direction::Pushing),
                    Direction::Pushing =>
                        driver.push_hash_location(route.to_string()),
                    _ => ()
                }
            }
        });

        let receiver = driver.on_hash_route_change({
            Box::new(move |url: &String| {
                direction.change((), |state, _| *state = Direction::Popping);
                callback(url);
                direction.change((), |state, _| *state = Direction::Pushing);
            })
        });

        Self {
            sender,
            receiver,
        }
    }
}
