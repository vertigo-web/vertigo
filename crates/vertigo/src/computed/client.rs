use crate::computed::{
    Dependencies,
    Computed,
    refresh_token::RefreshToken,
    graph_id::GraphId,
};

pub struct Client {
    deps: Dependencies,
    id: GraphId,
}

impl Client {
    pub fn new<T: 'static, F: Fn(&T) + 'static>(deps: Dependencies, computed: Computed<T>, call: F) -> Client {

        let id = GraphId::default();

        let get_value = deps.wrap_get_value(move || {
            computed.get_value()
        }, id.clone());

        let refresh = move || {
            let value = get_value();
            call(value.as_ref());
        };

        refresh();

        deps.register_refresh_token(id.clone(), RefreshToken::new_client(refresh));

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
        self.deps.remove_relation(&self.id);
    }
}
