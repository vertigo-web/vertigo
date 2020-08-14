use std::rc::Rc;
use std::fmt::Debug;

use crate::lib::{
    get_unique_id::get_unique_id,
    Dependencies::Dependencies,
    Computed::Computed,
};

pub struct ClientRefresh {
    id: u64,
    refresh: Rc<Box<dyn Fn()>>,
}

impl Clone for ClientRefresh {
    fn clone(&self) -> Self {
        ClientRefresh {
            id: self.id,
            refresh: self.refresh.clone(),
        }
    }
}

impl ClientRefresh {
    fn new(id: u64, refresh: Rc<Box<dyn Fn()>>) -> ClientRefresh {
        ClientRefresh {
            id,
            refresh,
        }
    }

    pub fn recalculate(&self) {
        let ClientRefresh { refresh, .. } = self;
        refresh();
    }

    pub fn getId(&self) -> u64 {
        self.id
    }
}

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

    pub fn getClientRefresh(&self) -> ClientRefresh {
        ClientRefresh::new(self.id, self.refresh.clone())
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
