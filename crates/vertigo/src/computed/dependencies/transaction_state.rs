use crate::{computed::graph_id::GraphId, struct_mut::ValueMut};

use super::hook::Hooks;

type SetValue = Box<dyn FnOnce() -> Option<GraphId> + 'static>;

enum State {
    Idle,
    Modification {
        //Modifying the first layer
        level: u16,               //current transacion level
        set_func_list: Vec<SetValue>,
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
                        set_func_list: Vec::new(),
                    }
                }
                State::Modification { mut level, set_func_list } => {
                    level += 1;
                    State::Modification { level, set_func_list }
                }
                State::Refreshing => {
                    panic!("You cannot change the source value while the dependency graph is being refreshed");
                }
            }
        })
    }

    pub fn down(&self) -> Option<Vec<SetValue>> {
        self.state.move_to(|state| -> (State, Option<Vec<SetValue>>) {
            match state {
                State::Idle => {
                    log::error!("You cannot call 'down' for a state 'TransactionState::Idle'");

                    (State::Idle, None)
                }
                State::Modification { mut level, set_func_list } => {
                    level -= 1;

                    if level == 0 {
                        return (State::Refreshing, Some(set_func_list));
                    }

                    (State::Modification { level, set_func_list }, None)
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
                State::Modification { level, set_func_list } => {
                    log::error!("you cannot go from 'TransactionState::Modification' to 'TransactionState::Idle'");
                    State::Modification { level, set_func_list }
                }
                State::Refreshing => {
                    hooks.fire_end();
                    State::Idle
                }
            }
        });
    }

    pub fn add_edge_to_refresh(&self, set_func: impl FnOnce() -> Option<GraphId> + 'static) {
        self.state.change(move |mut state| {
            match &mut state {
                State::Modification { set_func_list, .. } => {
                    set_func_list.push(Box::new(set_func));
                }
                _ => {
                    log::error!("You can only call the trigger if you are in a transaction block");
                }
            }
        })
    }
}
