use std::rc::Rc;
use vertigo_macro::store;

use crate::{
    JsJson,
    dev::command::decode_json,
    driver_module::{api::api_browser_command, hydration::DomSnapshot},
};

#[cfg(test)]
use crate::dev::ValueMut;

#[store]
pub fn api_dom_snapshot() -> Rc<ApiDomSnapshot> {
    Rc::new(ApiDomSnapshot {
        #[cfg(test)]
        mock: ValueMut::new(None),
    })
}

/// Browser DOM state, fetched once during application mount.
///
/// A separate store rather than a method on `DriverDom` for the same reason `api_fetch_cache`
/// is separate: an application that never hydrates does not pull the snapshot decoder into wasm.
pub struct ApiDomSnapshot {
    #[cfg(test)]
    mock: ValueMut<Option<Rc<DomSnapshot>>>,
}

impl ApiDomSnapshot {
    #[cfg(test)]
    pub fn set_mock(&self, snapshot: DomSnapshot) {
        self.mock.set(Some(Rc::new(snapshot)));
    }

    pub fn get(&self) -> Option<DomSnapshot> {
        #[cfg(test)]
        if let Some(mock) = self.mock.get() {
            return Some((*mock).clone());
        }

        let json = api_browser_command().dom_snapshot_get();

        if let JsJson::Null = json {
            return None;
        }

        match decode_json::<DomSnapshot>(json) {
            Ok(snapshot) => Some(snapshot),
            Err(err) => {
                log::error!("dom snapshot decode error = {err}, returning None");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver_module::hydration::{DomSnapshot, SnapshotNode};

    #[test]
    fn without_a_browser_there_is_no_snapshot() {
        assert!(api_dom_snapshot().get().is_none());
    }

    #[test]
    fn a_mocked_snapshot_is_returned_as_is() {
        api_dom_snapshot().set_mock(DomSnapshot {
            nodes: vec![SnapshotNode::Element {
                name: "html".to_string(),
                attrs: vec![],
                children: vec![],
            }],
            head: None,
            body: None,
        });

        let Some(snapshot) = api_dom_snapshot().get() else {
            panic!("the mocked snapshot should come back");
        };

        assert_eq!(snapshot.nodes.len(), 1);
    }
}
