use std::rc::Rc;
use std::fmt::Debug;

use crate::lib::{
    get_unique_id::get_unique_id,
    Dependencies::Dependencies,
    Computed::Computed,
    RefreshToken::RefreshToken,
};
pub struct Client {
    deps: Dependencies,
    id: u64,
    refresh: Rc<Box<dyn Fn()>>,
}

impl Client {
    pub fn new<T: Debug + 'static>(deps: Dependencies, computed: Computed<T>, call: Box<dyn Fn(&T) + 'static>) -> Client {
        let refresh = Box::new(move || {
            let value = computed.getValue();
            call(value.as_ref());
        });
        
        refresh();

        Client {
            deps,
            id: get_unique_id(),
            refresh: Rc::new(refresh)
        }
    }

    pub fn getClientRefresh(&self) -> RefreshToken {
        RefreshToken::newClient(self.id, self.refresh.clone())
    }

    pub fn off(self: Client) {
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        println!("Client ----> DROP");
        self.deps.removeRelation(self.id);
    }
}
