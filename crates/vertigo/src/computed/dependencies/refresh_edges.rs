use std::collections::BTreeMap;
use crate::computed::{Dependencies, GraphId, graph_value::GraphValueRefresh};

enum RefreshState {
    Unknown,
    NewValue,
    PreviousValue,
}

enum FinalState {
    NewValue,
    PreviousValue,
}

fn calculate_level(deps: &Dependencies, status: &mut BTreeMap::<GraphId, RefreshState>, edges_to_refresh: &Vec<GraphValueRefresh>) -> Vec<GraphValueRefresh> {
    let mut result: Vec<GraphValueRefresh> = Vec::new();

    for item in edges_to_refresh {

        let mut counter_unknown_value: u32 = 0;
        let mut counter_new_value: u32 = 0;

        let parents = deps.get_parents(item.id);

        for parent_id in parents {
            match status.get(&parent_id) {
                Some(RefreshState::Unknown) => {
                    counter_unknown_value += 1;
                },
                Some(RefreshState::NewValue) => {
                    counter_new_value += 1;
                },
                Some(RefreshState::PreviousValue) => {},
                None => {
                    counter_new_value += 1;
                },
            }
        }

        if counter_unknown_value != 0 {
            result.push(item.clone());
            continue;
        }

        let new_status = if counter_new_value > 0 {
            if item.refresh() {
                RefreshState::NewValue
            } else {
                RefreshState::PreviousValue
            }
        } else {
            RefreshState::PreviousValue
        };

        status.insert(item.id, new_status);
    }

    result
}

fn convert_refresh_state(state: RefreshState) -> FinalState {
    match state {
        RefreshState::NewValue => FinalState::NewValue,
        RefreshState::PreviousValue => FinalState::PreviousValue,
        RefreshState::Unknown => {
            panic!("Problem with refreshing");
        }
    }
}

fn convert_state(state: BTreeMap<GraphId, RefreshState>) -> BTreeMap<GraphId, FinalState> {
    let mut final_state = BTreeMap::new();

    for (id, item) in state {
        final_state.insert(id, convert_refresh_state(item));
    }

    final_state
}

fn refresh_edges_computed(deps: &Dependencies, mut edges_to_refresh: Vec<GraphValueRefresh>) -> BTreeMap<GraphId, RefreshState> {

    let mut status = BTreeMap::<GraphId, RefreshState>::new();
    
    for refresh_item in edges_to_refresh.iter() {
        status.insert(refresh_item.id, RefreshState::Unknown);
    }

    loop {
        let new_edges = calculate_level(deps, &mut status, &edges_to_refresh);

        if new_edges.len() == 0 {
            return status;
        }

        if new_edges.len() < edges_to_refresh.len() {
            edges_to_refresh = new_edges;
        } else {
            panic!("Recurency error");
        }
    }
}

fn refresh_edges_client(deps: &Dependencies, state_refresh: BTreeMap<GraphId, FinalState>, edges_to_refresh: Vec<GraphValueRefresh>) {
    for item in edges_to_refresh {

        let mut counter_new_value: u32 = 0;

        let parents = deps.get_parents(item.id);

        for parent_id in parents {
            if let Some(FinalState::NewValue) = state_refresh.get(&parent_id) {
                counter_new_value += 1;
            }
        }

        if counter_new_value > 0 {
            item.refresh();
        }
    }
}

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

    let state_refresh = refresh_edges_computed(deps, edges_computed);

    let state_refresh = convert_state(state_refresh);

    refresh_edges_client(deps, state_refresh, edges_client);
}
