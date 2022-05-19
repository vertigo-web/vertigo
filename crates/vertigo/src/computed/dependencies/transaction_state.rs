use std::collections::BTreeSet;

use crate::{computed::graph_id::GraphId, struct_mut::ValueMut};

use super::hook::Hooks;

#[derive(PartialEq)]
enum State {
    Idle,
    Modification {
        //Modifying the first layer
        level: u16,               //current transacion level
        edges: BTreeSet<GraphId>, //edges to refresh
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
    hooks: Hooks,
}

impl TransactionState {
    pub fn new() -> TransactionState {
        TransactionState {
            state: ValueMut::new(State::Idle),
            hooks: Hooks::new(),
        }
    }

    pub fn up(&self) -> bool {
        let TransactionState { state, hooks } = self;

        state.move_to(move |state| {
            match state {
                State::Idle => {
                    hooks.fire_start();

                    (
                        State::Modification {
                            level: 1,
                            edges: BTreeSet::new(),
                        },
                        true
                    )
                }
                State::Modification { mut level, edges } => {
                    level += 1;

                    (
                        State::Modification { level, edges },
                        true
                    )
                }
                State::Refreshing => {
                    log::error!("You cannot change the source value while the dependency graph is being refreshed");

                    (
                        State::Refreshing,
                        false
                    )
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
                State::Modification { mut level, mut edges } => {
                    level -= 1;

                    if level == 0 {
                        let edges_copy = std::mem::take(&mut edges);

                        return (State::Refreshing, Some(edges_copy));
                    }

                    (State::Modification { level, edges }, None)
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
                State::Modification { level, edges } => {
                    log::error!("you cannot go from 'TransactionState::Modification' to 'TransactionState::Idle'");
                    State::Modification { level, edges }
                }
                State::Refreshing => {
                    hooks.fire_end();
                    State::Idle
                }
            }
        });
    }

    pub fn add_edge_to_refresh(&self, new_edge: GraphId) {
        self.state.change(move |mut state| {
            match &mut state {
                State::Modification { edges, .. } => {
                    edges.insert(new_edge);
                }
                _ => {
                    log::error!("You can only call the trigger if you are in a transaction block");
                }
            }
        })
    }

    pub fn set_hook(&self, before_start: Box<dyn Fn()>, after_end: Box<dyn Fn()>) {
        self.hooks.add(before_start, after_end);
    }

    pub fn is_idle(&self) -> bool {
        self.state.map(|state| *state == State::Idle)
    }
}
