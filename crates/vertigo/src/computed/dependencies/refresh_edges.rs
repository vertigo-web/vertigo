use std::collections::{BTreeMap, BTreeSet};
use crate::computed::{Dependencies, GraphId, graph_value::GraphValueRefresh};

pub enum RefreshState {
    CalculationPending,
    NewValue,
    PreviousValue,
}

// impl RefreshState {
//     fn to_string(&self) -> &'static str {
//         match self {
//             RefreshState::Unknown => "Unknown",
//             RefreshState::NewValue => "NewValue",
//             RefreshState::PreviousValue => "PreviousValue",
//         }
//     }
// }

fn calculate_level(
    deps: &Dependencies,
    state_refreshing: &mut BTreeMap::<GraphId, RefreshState>,
    edges_to_refresh: &Vec<GraphValueRefresh>
) -> Vec<GraphValueRefresh> {
    let mut result: Vec<GraphValueRefresh> = Vec::new();

    for item in edges_to_refresh {

        let mut counter_unknown_value: u32 = 0;
        let mut counter_new_value: u32 = 0;

        for parent_id in deps.get_parents(item.id) {
            match state_refreshing.get(&parent_id) {
                Some(RefreshState::CalculationPending) => {
                    counter_unknown_value += 1;
                },
                Some(RefreshState::NewValue) => {
                    counter_new_value += 1;
                },
                Some(RefreshState::PreviousValue) => {},
                None => {},
            }
        }

        if counter_unknown_value > 0 {
            result.push(item.clone());
            continue;
        }

        let new_status = if counter_new_value > 0 {
            match item.refresh(&state_refreshing) {
                RefreshState::NewValue => RefreshState::NewValue,
                RefreshState::PreviousValue => RefreshState::PreviousValue,
                RefreshState::CalculationPending => {
                    log::warn!("continue computation ...");
                    result.push(item.clone());
                    continue;
                }
            }
        } else {
            RefreshState::PreviousValue
        };

        state_refreshing.insert(item.id, new_status);
    }

    result
}

// fn show_state(state_refreshing: &BTreeMap::<GraphId, RefreshState>) {
//     log::info!("------------------------------");
//     for (id, item) in state_refreshing {
//         log::info!("item ---> {:?} {}", id, item.to_string());
//     }
// }

fn drop_edges(deps: &Dependencies) {
    loop {
        let edges = deps.drain_removables();

        if edges.len() == 0 {
            return;
        }

        for dropped_id in edges {
            deps.drop_value(&dropped_id);
        }
    }
}

fn refresh_edges_computed(
    deps: &Dependencies,
    state_refreshing: &mut BTreeMap::<GraphId, RefreshState>,
    mut edges_to_refresh: Vec<GraphValueRefresh>
) {

    loop {
        let new_edges = calculate_level(deps, state_refreshing, &edges_to_refresh);

        if new_edges.len() == 0 {
            return;
        }

        if new_edges.len() < edges_to_refresh.len() {
            edges_to_refresh = new_edges;
        } else {
            panic!("Recurency error");
        }
    }
}

fn refresh_edges_client(deps: &Dependencies, state_refreshing: &mut BTreeMap<GraphId, RefreshState>, edges_to_refresh: Vec<GraphValueRefresh>) {

    let empty_state_refreshing = BTreeMap::new();

    for item in edges_to_refresh {

        let mut counter_new_value: u32 = 0;

        let parents = deps.get_parents(item.id);

        for parent_id in parents {
            match state_refreshing.get(&parent_id) {
                Some(RefreshState::CalculationPending) => {
                    //panic!("Incorrect graph condition {:?}", parent_id);
                                            //TODO - trzeba sprawdzić, jesli odwolujemy sie do innego klienta, to pomiń to połaczenie
                },
                Some(RefreshState::NewValue) => {
                    counter_new_value += 1;
                },
                Some(RefreshState::PreviousValue) => {},
                None => {},
            }
        }

        if counter_new_value > 0 {
            item.refresh(&empty_state_refreshing);
        }
    }
}

fn crete_state_refreshing(edges_values: &BTreeSet<GraphId>, edges_to_refresh: &Vec<GraphValueRefresh>) -> BTreeMap::<GraphId, RefreshState> {
    let mut state_refreshing = BTreeMap::<GraphId, RefreshState>::new();
    
    for valute_id in edges_values {
        state_refreshing.insert(valute_id.clone(), RefreshState::NewValue);
    }

    for refresh_item in edges_to_refresh.iter() {
        state_refreshing.insert(refresh_item.id, RefreshState::CalculationPending);
    }

    state_refreshing
}

pub fn refresh_edges(deps: &Dependencies, edges_values: &BTreeSet<GraphId>, edges_to_refresh: Vec<GraphValueRefresh>) {
    let mut state_refreshing = crete_state_refreshing(&edges_values, &edges_to_refresh);

    let mut edges_computed = Vec::new();
    let mut edges_client = Vec::new();

    for item in edges_to_refresh {
        if item.is_computed() {
            edges_computed.push(item);
        } else {
            edges_client.push(item);
        }
    }

    refresh_edges_computed(deps, &mut state_refreshing, edges_computed);

    refresh_edges_client(deps, &mut state_refreshing, edges_client);

    drop_edges(deps);
}
