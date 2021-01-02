use alloc::{
    collections::BTreeSet,
};

use crate::computed::graph_id::GraphId;


pub enum TransactionState {
    Idle,
    Modification {                          //Modifying the first layer
        level: u16,                         //current transacion level
        edges: BTreeSet<GraphId>,            //edges to refresh
    },
    Refreshing
}

impl TransactionState {
    pub fn up(&mut self) -> bool {
        match self {
            TransactionState::Idle => {
                *self = TransactionState::Modification {
                    level: 1,
                    edges: BTreeSet::new()
                };

                true
            },
            TransactionState::Modification { level, .. } => {
                *level += 1;
                true
            },
            TransactionState::Refreshing => {
                log::error!("You cannot change the source value while the dependency graph is being refreshed");
                false
            }
        }
    }

    pub fn down(&mut self) -> Option<BTreeSet<GraphId>> {
        match self {
            TransactionState::Idle => {
                log::error!("You cannot call 'down' for a state 'TransactionState::Idle'");

                None
            },
            TransactionState::Modification { level, edges } => {
                *level -= 1;

                if *level == 0 {
                    let edges = core::mem::replace(edges, BTreeSet::new());
                    *self = TransactionState::Refreshing;
                    return Some(edges);
                }

                None
            },
            TransactionState::Refreshing => {
                log::error!("You cannot change the source value while the dependency graph is being refreshed");
                None
            }
        }
    }

    pub fn to_idle(&mut self) {
        match self {
            TransactionState::Idle => {
                log::error!("you cannot go from 'TransactionState::Idle' to 'TransactionState::Idle'");
            },
            TransactionState::Modification { .. } => {
                log::error!("you cannot go from 'TransactionState::Modification' to 'TransactionState::Idle'");
            },
            TransactionState::Refreshing => {
                *self = TransactionState::Idle;
            }
        }
    }

    pub fn add_edge_to_refresh(&mut self, new_edge: GraphId) {
        match self {
            TransactionState::Modification { edges, .. } => {
                edges.insert(new_edge);
            },
            _ => {
                log::error!("You can only call the trigger if you are in a transaction block");
            }
        }
    }
}
