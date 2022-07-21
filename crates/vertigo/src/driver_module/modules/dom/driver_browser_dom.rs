use std::{
    rc::Rc,
};
use crate::{
    DomId,
    KeyDownEvent, DropFileEvent,
};
use crate::struct_mut::VecMut;

use crate::driver_module::api::ApiImport;

use super::{
    driver_data::DriverData,
    driver_dom_command::DriverDomCommand,
    visited_node_manager::VisitedNodeManager,
};


pub struct DriverDom {
    api: Rc<ApiImport>,
    pub(crate) data: DriverData,
    commands: VecMut<DriverDomCommand>,
    current_visited: VisitedNodeManager,
}

impl DriverDom {
    pub fn new(
        api: &Rc<ApiImport>,
    ) -> DriverDom {
        let data = DriverData::new();
        let current_visited = VisitedNodeManager::new();

        let driver_browser = DriverDom {
            api: api.clone(),
            data,
            commands: VecMut::new(),
            current_visited,
        };

        let root_id = DomId::root();

        driver_browser.create_node(root_id, "div");
        driver_browser.mount_node(root_id);

        driver_browser
    }

    pub fn export_dom_mousedown(&self, dom_id: u64) {
        let event_to_run = self.data.find_event_click(DomId::from_u64(dom_id));

        if let Some(callback) = event_to_run {
            callback();
        }
    }

    pub fn export_dom_mouseover(&self, dom_id: Option<u64>) {
        match dom_id {
            None => {
                self.current_visited.clear();
            },
            Some(dom_id) => {
                let nodes = self.data.find_all_nodes(DomId::from_u64(dom_id));
                self.current_visited.push_new_nodes(&self.data, nodes);
            }
        }
    }

    pub fn export_dom_keydown(&self, dom_id: Option<u64>, key: String, code: String, alt_key: bool, ctrl_key: bool, shift_key: bool, meta_key: bool) -> bool {
        let event = KeyDownEvent {
            key,
            code,
            alt_key,
            ctrl_key,
            shift_key,
            meta_key,
        };

        for callback in self.data.find_hook_keydown() {
            let stop_propagate = callback(event.clone());

            if stop_propagate {
                return true;
            }
        }

        let id = match dom_id {
            None => DomId::root(),
            Some(id) => DomId::from_u64(id),
        };

        match self.data.find_event_keydown(id) {
            Some(event_to_run) => event_to_run(event),
            None => false,
        }
    }

    pub fn export_dom_oninput(&self, dom_id: u64, text: String) {
        let event_to_run = self.data.find_event_on_input(DomId::from_u64(dom_id));

        if let Some(event_to_run) = event_to_run {
            event_to_run(text);
        }
    }

    pub fn export_dom_ondropfile(&self, dom_id: u64, event: DropFileEvent) {
        let event_to_run = self.data.find_event_on_dropfile(DomId::from_u64(dom_id));

        if let Some(event_to_run) = event_to_run {
            event_to_run(event);
        }
    }

    fn mount_node(&self, id: DomId) {
        self.commands.push(DriverDomCommand::MountNode { id });
    }

    fn add_command(&self, command: DriverDomCommand) {
        self.commands.push(command);
    }

    pub fn create_node(&self, id: DomId, name: &'static str) {
        self.data.create_node(id);
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
        self.data.remove_text(id);
        self.add_command(DriverDomCommand::RemoveText { id });
    }

    pub fn remove_node(&self, id: DomId) {
        self.data.remove_node(id);
        self.add_command(DriverDomCommand::RemoveNode { id });
    }

    pub fn insert_before(&self, parent: DomId, child: DomId, ref_id: Option<DomId>) {
        self.data.set_parent(child, parent);
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
        self.data.remove_text(id);
        self.add_command(DriverDomCommand::RemoveComment { id });
    }
    
    pub fn flush_dom_changes(&self) {
        let state = self.commands.take();

        if !state.is_empty() {
            let mut out = Vec::<String>::new();

            for command in state {
                out.push(command.into_string());
            }

            let command_str = format!("[{}]", out.join(","));
            self.api.dom_bulk_update(command_str.as_str());
        }
    }
}
