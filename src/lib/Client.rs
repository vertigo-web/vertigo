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
}

impl Client {
    pub fn new<T: Debug + 'static, F: Fn(&T) + 'static>(deps: Dependencies, computed: Computed<T>, call: F) -> Client {

        let id = get_unique_id();
        
        let getValue = deps.wrapGetValue(move || {
            computed.getValue()
        }, id);

        let refresh = move || {
            let value = getValue();
            call(value.as_ref());
        };
        
        refresh();

        deps.registerRefreshToken(id, RefreshToken::newClient(refresh));

        Client {
            deps,
            id,
        }
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
