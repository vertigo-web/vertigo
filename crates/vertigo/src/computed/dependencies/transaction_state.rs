use std::collections::BTreeSet;

use crate::{computed::graph_id::GraphId, struct_mut::ValueMut};

#[derive(PartialEq)]
enum State {
    Idle,
    Modification {
        //Modifying the first layer
        level: u16,               //current transacion level
        client_ids: BTreeSet<GraphId>,
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
}

impl TransactionState {
    pub fn new() -> TransactionState {
        TransactionState {
            state: ValueMut::new(State::Idle),
        }
    }

    pub fn up(&self) {
        let TransactionState { state } = self;

        state.move_to_void(move |state| {
            match state {
                State::Idle => {
                    State::Modification {
                        level: 1,
                        client_ids: BTreeSet::new(),
                    }
                }
                State::Modification { mut level, client_ids } => {
                    level += 1;
                    State::Modification { level, client_ids }
                }
                State::Refreshing => {
                    panic!("You cannot change the source value while the dependency graph is being refreshed");
                }
            }
        })
    }

    pub fn down(&self) -> Option<BTreeSet<GraphId>> {
        self.state.move_to(|state| -> (State, Option<BTreeSet<GraphId>>) {
            match state {
                State::Idle => {
                    log::error!("You cannot call 'down' for a state 'TransactionState::Idle'");

                    (State::Idle, None)
                }
                State::Modification { mut level, client_ids } => {
                    level -= 1;

                    if level == 0 {
                        return (State::Refreshing, Some(client_ids));
                    }

                    (State::Modification { level, client_ids }, None)
                }
                State::Refreshing => {
                    log::error!("You cannot change the source value while the dependency graph is being refreshed");

                    (State::Refreshing, None)
                }
            }
        })
    }

    pub fn move_to_idle(&self) {
        let TransactionState { state } = self;

        state.move_to_void(move |state| {
            match state {
                State::Idle => {
                    log::error!("you cannot go from 'TransactionState::Idle' to 'TransactionState::Idle'");
                    State::Idle
                }
                State::Modification { level, client_ids } => {
                    log::error!("you cannot go from 'TransactionState::Modification' to 'TransactionState::Idle'");
                    State::Modification { level, client_ids }
                }
                State::Refreshing => {
                    State::Idle
                }
            }
        });
    }

    pub fn add_clients_to_refresh(&self, client: BTreeSet<GraphId>) {
        self.state.change(move |mut state| {
            match &mut state {
                State::Modification { client_ids, .. } => {
                    client_ids.extend(client.into_iter());
                }
                _ => {
                    log::error!("You can only call the trigger if you are in a transaction block");
                }
            }
        })
    }
}
