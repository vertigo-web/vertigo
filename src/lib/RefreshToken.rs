use std::rc::Rc;

use crate::lib::BoxRefCell::BoxRefCell;

pub enum RefreshToken {
    Computed {
        isFreshCell: Rc<BoxRefCell<bool>>,
    },
    Client {
        refresh: Rc<Box<dyn Fn()>>,
    }
}

impl RefreshToken {
    pub fn newComputed(isFreshCell: Rc<BoxRefCell<bool>>) -> RefreshToken {
        RefreshToken::Computed {
            isFreshCell,
        }
    }

    pub fn newClient<F: Fn() + 'static>(refresh: F) -> RefreshToken {
        RefreshToken::Client {
            refresh: Rc::new(Box::new(refresh)),
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
            RefreshToken::Computed { isFreshCell, .. } => {
                RefreshToken::Computed {
                    isFreshCell: isFreshCell.clone(),
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
