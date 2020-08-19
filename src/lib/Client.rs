use crate::lib::{
    Dependencies::Dependencies,
    Computed::Computed,
    RefreshToken::RefreshToken,
    GraphId::GraphId,
};

pub struct Client {
    deps: Dependencies,
    id: GraphId,
}

impl Client {
    pub fn new<T: 'static, F: Fn(&T) + 'static>(deps: Dependencies, computed: Computed<T>, call: F) -> Client {

        let id = GraphId::new();
        
        let getValue = deps.wrapGetValue(move || {
            computed.getValue()
        }, id.clone());

        let refresh = move || {
            let value = getValue();
            call(value.as_ref());
        };
        
        refresh();

        deps.registerRefreshToken(id.clone(), RefreshToken::newClient(refresh));

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
        self.deps.removeRelation(&self.id);
    }
}
