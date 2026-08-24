use std::cell::Cell;

use crate::{
    Computed, Context, DropResource, ToComputed, driver_module::get_driver_dom, struct_mut::VecMut,
};

use super::dom_id::DomId;

/// A Real DOM representative - text kind
pub struct DomText {
    id_dom: DomId,
    subscriptions: VecMut<DropResource>,
}

impl DomText {
    pub fn new(value: impl Into<String>) -> DomText {
        let value = value.into();
        let id = DomId::default();

        get_driver_dom().create_text(id, &value);

        DomText {
            id_dom: id,
            subscriptions: VecMut::new(),
        }
    }

    /// A text node that keeps itself current by **patching its own content**.
    ///
    /// One `UpdateText` per change, and the node keeps its identity - which means anything
    /// holding a reference to it, and any browser state attached to it, survives.
    ///
    /// The alternative shape, a `render_value` that builds a fresh [`DomText`] each time,
    /// costs three commands per change (create, insert, remove), leaves a marker comment
    /// behind, and hands out a new [`DomId`] every time.
    pub fn new_computed<T: Into<String> + Clone + PartialEq + 'static>(
        computed: impl ToComputed<T>,
    ) -> Self {
        Self::patched(computed.to_computed(), Into::into)
    }

    /// As [`Self::new_computed`], but for anything *printable* rather than convertible into
    /// a `String` - `u32`, `bool`, a [`Display`](std::fmt::Display) type of your own.
    pub fn new_computed_display<T: ToString + Clone + PartialEq + 'static>(
        computed: impl ToComputed<T>,
    ) -> Self {
        Self::patched(computed.to_computed(), |value| value.to_string())
    }

    fn patched<T: Clone + PartialEq + 'static>(
        computed: Computed<T>,
        print: impl Fn(T) -> String + 'static,
    ) -> Self {
        // Created with the value already in it, rather than created empty and patched by
        // the subscription's first run. That keeps the mount stream a single `CreateText`,
        // which matters for more than the one saved command: the hydration pass builds its
        // virtual tree from `CreateText` and ignores `UpdateText` entirely
        // (`src_js/api/command/dom/hydration.ts`), so a node created empty would hydrate
        // as empty and then have to be corrected.
        let initial = print(computed.get(&Context::read()));
        let text_node = DomText::new(initial);
        let id_dom = text_node.id_dom;

        // `subscribe` replays the current value immediately, and that first call would
        // write the very string the node was just created with. Skip it; every later call
        // is a real change, because a `Computed` only notifies when its value differs.
        let is_first_call = Cell::new(true);
        let client = computed.subscribe(move |value| {
            if is_first_call.replace(false) {
                return;
            }
            get_driver_dom().update_text(id_dom, &print(value));
        });

        text_node.subscriptions.push(client);
        text_node
    }

    pub fn id_dom(&self) -> DomId {
        self.id_dom
    }

    pub fn append_drop_resource(&self, resource: DropResource) {
        self.subscriptions.push(resource);
    }
}

impl Drop for DomText {
    fn drop(&mut self) {
        get_driver_dom().remove_text(self.id_dom);
    }
}
