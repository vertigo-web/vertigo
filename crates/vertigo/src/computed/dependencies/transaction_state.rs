use std::collections::BTreeSet;

use crate::computed::graph_id::GraphId;
use super::hook::Hooks;

enum State {
    Idle,
    Modification {                          //Modifying the first layer
        level: u16,                         //current transacion level
        edges: BTreeSet<GraphId>,            //edges to refresh
    },
    Refreshing
}

pub struct TransactionState {
    state: State,
    hooks: Hooks,
}

impl TransactionState {
    pub fn new() -> TransactionState {
        TransactionState {
            state: State::Idle,
            hooks: Hooks::new(),
        }
    }

    fn up_state(state: &mut State, hooks: &mut Hooks) -> bool {
        match state {
            State::Idle => {
                hooks.fire_start();

                *state = State::Modification {
                    level: 1,
                    edges: BTreeSet::new()
                };

                true
            },
            State::Modification { level, .. } => {
                *level += 1;
                true
            },
            State::Refreshing => {
                log::error!("You cannot change the source value while the dependency graph is being refreshed");
                false
            }
        }
    }

    pub fn up(&mut self) -> bool {
        let TransactionState { state, hooks} = self;
        TransactionState::up_state(state, hooks)
    }

    fn down_state(state: &mut State) -> Option<BTreeSet<GraphId>> {
        match state {
            State::Idle => {
                log::error!("You cannot call 'down' for a state 'TransactionState::Idle'");

                None
            },
            State::Modification { level, edges } => {
                *level -= 1;

                if *level == 0 {
                    let edges_copy = std::mem::take(edges);
                    *state = State::Refreshing;
                    return Some(edges_copy);
                }

                None
            },
            State::Refreshing => {
                log::error!("You cannot change the source value while the dependency graph is being refreshed");
                None
            }
        }
    }

    pub fn down(&mut self) -> Option<BTreeSet<GraphId>> {
        TransactionState::down_state(&mut self.state)
    }

    fn move_to_idle_state(state: &mut State, hooks: &mut Hooks) {
        match state {
            State::Idle => {
                log::error!("you cannot go from 'TransactionState::Idle' to 'TransactionState::Idle'");
            },
            State::Modification { .. } => {
                log::error!("you cannot go from 'TransactionState::Modification' to 'TransactionState::Idle'");
            },
            State::Refreshing => {
                *state = State::Idle;
                hooks.fire_end();
            }
        }
    }

    pub fn move_to_idle(&mut self) {
        let TransactionState { state, hooks} = self;
        TransactionState::move_to_idle_state(state, hooks)
    }

    fn add_edge_to_refresh_state(state: &mut State, new_edge: GraphId) {
        match state {
            State::Modification { edges, .. } => {
                edges.insert(new_edge);
            },
            _ => {
                log::error!("You can only call the trigger if you are in a transaction block");
            }
        }
    }

    pub fn add_edge_to_refresh(&mut self, new_edge: GraphId) {
        TransactionState::add_edge_to_refresh_state(&mut self.state, new_edge);
    }

    pub fn set_hook(&mut self, before_start: Box<dyn Fn()>, after_end: Box<dyn Fn()>) {
        self.hooks.add(before_start, after_end);
    }
}
