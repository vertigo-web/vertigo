use std::collections::BTreeSet;

use crate::{computed::graph_id::GraphId, struct_mut::ValueMut};

use super::hook::Hooks;

pub struct SetValueManager {
    ids: BTreeSet<GraphId>,
    set_func: Vec<Box<dyn FnOnce() -> Option<GraphId> + 'static>>,
}

impl SetValueManager {
    pub fn new() -> Self {
        Self {
            set_func: Vec::new(),
            ids: BTreeSet::new(),
        }
    }

    pub fn add(&mut self, id: GraphId, set_func: impl FnOnce() -> bool + 'static) {
        self.set_func.push(Box::new(move || {
            match set_func() {
                true => Some(id),
                false => None,
            }
        }));
        self.ids.insert(id);
    }

    pub fn exec_set(self) -> BTreeSet<GraphId> {
        let mut result = BTreeSet::new();

        for set_func in self.set_func.into_iter() {
            if let Some(id) = set_func() {
                result.insert(id);
            }
        }

        result
    }

    pub fn is_in(&self, id: GraphId) -> bool {
        self.ids.contains(&id)
    }
}

enum State {
    Idle,
    Modification {
        //Modifying the first layer
        level: u16,               //current transacion level
        manager: SetValueManager,
    },
    Refreshing,
}

impl Default for State {
    fn default() -> Self {
        Self::Idle
    }
}

pub struct TransactionState {
    state: ValueMut<State>,
    pub(crate) hooks: Hooks,
}

impl TransactionState {
    pub fn new() -> TransactionState {
        TransactionState {
            state: ValueMut::new(State::Idle),
            hooks: Hooks::new(),
        }
    }

    pub fn up(&self) {
        let TransactionState { state, hooks: _ } = self;

        state.move_to_void(move |state| {
            match state {
                State::Idle => {
                    State::Modification {
                        level: 1,
                        manager: SetValueManager::new(),
                    }
                }
                State::Modification { mut level, manager } => {
                    level += 1;
                    State::Modification { level, manager }
                }
                State::Refreshing => {
                    panic!("You cannot change the source value while the dependency graph is being refreshed");
                }
            }
        })
    }

    pub fn down(&self) -> Option<SetValueManager> {
        self.state.move_to(|state| -> (State, Option<SetValueManager>) {
            match state {
                State::Idle => {
                    log::error!("You cannot call 'down' for a state 'TransactionState::Idle'");

                    (State::Idle, None)
                }
                State::Modification { mut level, manager } => {
                    level -= 1;

                    if level == 0 {
                        return (State::Refreshing, Some(manager));
                    }

                    (State::Modification { level, manager }, None)
                }
                State::Refreshing => {
                    log::error!("You cannot change the source value while the dependency graph is being refreshed");

                    (State::Refreshing, None)
                }
            }
        })
    }

    pub fn move_to_idle(&self) {
        let TransactionState { state, hooks } = self;

        state.move_to_void(move |state| {
            match state {
                State::Idle => {
                    log::error!("you cannot go from 'TransactionState::Idle' to 'TransactionState::Idle'");
                    State::Idle
                }
                State::Modification { level, manager } => {
                    log::error!("you cannot go from 'TransactionState::Modification' to 'TransactionState::Idle'");
                    State::Modification { level, manager }
                }
                State::Refreshing => {
                    hooks.fire_end();
                    State::Idle
                }
            }
        });
    }

    pub fn add_edge_to_refresh(&self, id: GraphId, set_func: impl FnOnce() -> bool + 'static) {
        self.state.change(move |mut state| {
            match &mut state {
                State::Modification { manager, .. } => {
                    manager.add(id, set_func);
                }
                _ => {
                    log::error!("You can only call the trigger if you are in a transaction block");
                }
            }
        })
    }

    pub fn is_in_queue_to_refresh(&self, id: GraphId) -> bool {
        self.state.map(move |state| {
            match state {
                State::Modification { manager, .. } => manager.is_in(id),
                _ => false,
            }
        })
    }
}
