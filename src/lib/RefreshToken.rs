use std::rc::Rc;

use crate::lib::BoxRefCell::BoxRefCell;

pub enum RefreshToken {
    Computed {
        id: u64,
        isFreshCell: Rc<BoxRefCell<bool>>,
    },
    Client {
        id: u64,
        refresh: Rc<Box<dyn Fn()>>,
    }
}

impl RefreshToken {
    pub fn newComputed(id: u64, isFreshCell: Rc<BoxRefCell<bool>>) -> RefreshToken {
        RefreshToken::Computed {
            id,
            isFreshCell,
        }
    }

    pub fn newClient(id: u64, refresh: Rc<Box<dyn Fn()>>) -> RefreshToken {
        RefreshToken::Client {
            id,
            refresh,
        }
    }

    pub fn update(&self) {
        match self {
            RefreshToken::Computed { isFreshCell, .. } => {
                isFreshCell.change((), |state, _data| {
                    *state = false;
                });
            },
            RefreshToken::Client { refresh, .. } => {
                refresh();
            } 
        }
    }

    pub fn getId(&self) -> u64 {
        match self {
            RefreshToken::Computed { id, .. } => {
                *id
            },
            RefreshToken::Client { id, .. } => {
                *id
            }
        }
    }

    pub fn isComputed(&self) -> bool {
        match self {
            RefreshToken::Computed { .. } => true,
            RefreshToken::Client { .. } => false,
        }
    }
}

impl Clone for RefreshToken {
    fn clone(&self) -> Self {
        match self {
            RefreshToken::Computed { isFreshCell, id } => {
                RefreshToken::Computed {
                    isFreshCell: isFreshCell.clone(),
                    id: *id
                }
            },
            RefreshToken::Client { id, refresh } => {
                RefreshToken::Client {
                    id: *id,
                    refresh: refresh.clone(),
                }
            }
        }
    }
}
