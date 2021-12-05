use crate::computed::{Dependencies, graph_value::GraphValueRefresh};

pub fn refresh_edges(deps: &Dependencies, edges_to_refresh: Vec<GraphValueRefresh>) {
    let mut edges_computed = Vec::new();
    let mut edges_client = Vec::new();

    for item in edges_to_refresh {
        if item.is_computed() {
            edges_computed.push(item);
        } else {
            edges_client.push(item);
        }
    }

    for item in edges_computed {
        item.drop_value();
    }

    for item in edges_client {
        item.refresh();
    }

    deps.external_connections.refresh_connect();
}
