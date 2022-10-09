use std::cell::Cell;
use std::rc::Rc;

use crate::DomId;
use crate::struct_mut::VecMut;

use crate::driver_module::api::ApiImport;
use crate::driver_module::callbacks::CallbackId;
use super::driver_dom_command::{DriverDomCommand, sort_commands};


pub struct DriverDom {
    api: Rc<ApiImport>,
    commands: VecMut<DriverDomCommand>,

    // For testing/debuging purposes
    log_enabled: Cell<bool>,
    log_vec: VecMut<DriverDomCommand>,
}

impl DriverDom {
    pub fn new(api: &Rc<ApiImport>) -> DriverDom {

        let driver_browser = DriverDom {
            api: api.clone(),
            commands: VecMut::new(),
            log_enabled: Cell::new(false),
            log_vec: VecMut::new(),
        };

        let root_id = DomId::root();

        driver_browser.create_node(root_id, "div");
        driver_browser.mount_node(root_id);

        driver_browser
    }

    fn mount_node(&self, id: DomId) {
        let command = DriverDomCommand::MountNode { id };
        if self.log_enabled.get() {
            self.log_vec.push(command.clone());
        }
        self.commands.push(command);
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
            let mut out = Vec::<String>::new();

            let state = sort_commands(state);

            for command in state {
                out.push(command.into_string());
            }

            let command_str = format!("[{}]", out.join(","));
            self.api.dom_bulk_update(command_str.as_str());
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
