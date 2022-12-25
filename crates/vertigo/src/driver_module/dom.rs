use std::cell::Cell;

use crate::{DomId, JsJson};
use crate::struct_mut::VecMut;

use crate::driver_module::api::ApiImport;
use super::{dom_command::{DriverDomCommand, sort_commands}, api::CallbackId};


pub struct DriverDom {
    api: ApiImport,
    commands: VecMut<DriverDomCommand>,

    // For testing/debuging purposes
    log_enabled: Cell<bool>,
    log_vec: VecMut<DriverDomCommand>,
}

impl DriverDom {
    pub fn new(api: &ApiImport) -> DriverDom {
        DriverDom {
            api: api.clone(),
            commands: VecMut::new(),
            log_enabled: Cell::new(false),
            log_vec: VecMut::new(),
        }
    }

    fn add_command(&self, command: DriverDomCommand) {
        if self.log_enabled.get() {
            self.log_vec.push(command.clone());
        }

        self.commands.push(command);
    }

    pub fn create_node(&self, id: DomId, name: &'static str) {
        self.add_command(DriverDomCommand::CreateNode { id, name });
    }

    pub fn create_text(&self, id: DomId, value: &str) {
        self.add_command(DriverDomCommand::CreateText {
            id,
            value: value.into(),
        })
    }

    pub fn update_text(&self, id: DomId, value: &str) {
        self.add_command(DriverDomCommand::UpdateText {
            id,
            value: value.into(),
        });
    }

    pub fn set_attr(&self, id: DomId, name: &'static str, value: &str) {
        self.add_command(DriverDomCommand::SetAttr {
            id,
            name,
            value: value.into(),
        });
    }

    pub fn remove_text(&self, id: DomId) {
        self.add_command(DriverDomCommand::RemoveText { id });
    }

    pub fn remove_node(&self, id: DomId) {
        self.add_command(DriverDomCommand::RemoveNode { id });
    }

    pub fn insert_before(&self, parent: DomId, child: DomId, ref_id: Option<DomId>) {
        self.add_command(DriverDomCommand::InsertBefore { parent, child, ref_id });
    }

    pub fn insert_css(&self, selector: &str, value: &str) {
        self.add_command(DriverDomCommand::InsertCss {
            selector: selector.into(),
            value: value.into(),
        });
    }

    pub fn create_comment(&self, id: DomId, value: String) {
        self.add_command(DriverDomCommand::CreateComment {
            id,
            value,
        })
    }

    pub fn remove_comment(&self, id: DomId) {
        self.add_command(DriverDomCommand::RemoveComment { id });
    }

    pub fn flush_dom_changes(&self) {
        let state = self.commands.take();

        if !state.is_empty() {
            let mut out = Vec::<JsJson>::new();

            let state = sort_commands(state);

            for command in state {
                out.push(command.into_string());
            }

            let out = JsJson::List(out);
            self.api.dom_bulk_update(out);
        }
    }

    pub fn callback_add(&self, id: DomId, event_name: impl Into<String>, callback_id: CallbackId) {
        self.add_command(DriverDomCommand::CallbackAdd {
            id,
            event_name: event_name.into(),
            callback_id
        });
    }

    pub fn callback_remove(&self, id: DomId, event_name: impl Into<String>, callback_id: CallbackId) {
        self.add_command(DriverDomCommand::CallbackRemove {
            id,
            event_name: event_name.into(),
            callback_id
        });
    }

    pub fn log_start(&self) {
        if self.log_enabled.replace(true) {
            println!("log_start: already started");
        }
    }

    pub fn log_take(&self) -> Vec<DriverDomCommand> {
        self.log_enabled.replace(false);
        self.log_vec.take()
    }
}
