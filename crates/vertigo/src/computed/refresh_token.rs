use std::rc::Rc;

use crate::computed::BoxRefCell;

pub enum RefreshToken {
    Computed {
        is_fresh_cell: Rc<BoxRefCell<bool>>,
    },
    Client {
        refresh: Rc<dyn Fn()>,
    }
}

impl RefreshToken {
    pub fn new_computed(is_fresh_cell: Rc<BoxRefCell<bool>>) -> RefreshToken {
        RefreshToken::Computed {
            is_fresh_cell: is_fresh_cell,
        }
    }

    pub fn new_client<F: Fn() + 'static>(refresh: F) -> RefreshToken {
        RefreshToken::Client {
            refresh: Rc::new(Box::new(refresh)),
        }
    }

    pub fn update(&self) {
        match self {
            RefreshToken::Computed { is_fresh_cell, .. } => {
                is_fresh_cell.change((), |state, _data| {
                    *state = false;
                });
            },
            RefreshToken::Client { refresh, .. } => {
                refresh();
            }
        }
    }

    pub fn is_computed(&self) -> bool {
        match self {
            RefreshToken::Computed { .. } => true,
            RefreshToken::Client { .. } => false,
        }
    }
}

impl Clone for RefreshToken {
    fn clone(&self) -> Self {
        match self {
            RefreshToken::Computed { is_fresh_cell, .. } => {
                RefreshToken::Computed {
                    is_fresh_cell: is_fresh_cell.clone(),
                }
            },
            RefreshToken::Client { refresh } => {
                RefreshToken::Client {
                    refresh: refresh.clone(),
                }
            }
        }
    }
}
