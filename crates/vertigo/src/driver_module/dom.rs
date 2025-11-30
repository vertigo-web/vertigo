use std::rc::Rc;
use vertigo_macro::store;

use crate::{
    computed::{
        struct_mut::{HashMapMut, VecMut},
        DropResource,
    },
    dev::{command::DriverDomCommand, CallbackId},
    driver_module::{api::api_browser_command, event_emitter::EventEmitter},
    DomId,
};

use super::StaticString;

struct Commands {
    commands: VecMut<DriverDomCommand>,
    // For testing/debuging purposes
    new_command: EventEmitter<DriverDomCommand>,
}

impl Commands {
    pub fn new() -> Self {
        Commands {
            commands: VecMut::new(),
            new_command: EventEmitter::default(),
        }
    }

    #[allow(dead_code)]
    fn inspect_command(&self, func: impl Fn(DriverDomCommand) + 'static) -> DropResource {
        self.new_command.add(func)
    }

    fn add_command(&self, command: DriverDomCommand) {
        self.new_command.trigger(&command);
        self.commands.push(command);
    }

    fn flush_dom_changes(&self) {
        let state = self.commands.take();

        if !state.is_empty() {
            let state: Vec<DriverDomCommand> = sort_commands(state);
            api_browser_command().dom_bulk_update(state);
        }
    }
}

pub fn sort_commands(list: Vec<DriverDomCommand>) -> Vec<DriverDomCommand> {
    let mut dom = Vec::new();
    let mut events = Vec::new();

    for command in list {
        if command.is_event() {
            events.push(command);
        } else {
            dom.push(command);
        }
    }

    dom.extend(events);

    dom
}

type Callback = Rc<dyn Fn(DomId) + 'static>;

#[store]
pub fn get_driver_dom() -> Rc<DriverDom> {
    Rc::new(DriverDom::new())
}

pub struct DriverDom {
    commands: Commands,
    node_parent_callback: Rc<HashMapMut<DomId, Callback>>,
}

impl DriverDom {
    fn new() -> DriverDom {
        let commands = Commands::new();

        DriverDom {
            commands,
            node_parent_callback: Rc::new(HashMapMut::new()),
        }
    }

    #[allow(dead_code)]
    pub fn inspect_command(&self, func: impl Fn(DriverDomCommand) + 'static) -> DropResource {
        self.commands.inspect_command(func)
    }

    pub fn create_node(&self, id: DomId, name: impl Into<StaticString>) {
        let name = name.into();

        self.commands
            .add_command(DriverDomCommand::CreateNode { id, name });
    }

    pub fn create_text(&self, id: DomId, value: &str) {
        self.commands.add_command(DriverDomCommand::CreateText {
            id,
            value: value.into(),
        })
    }

    pub fn update_text(&self, id: DomId, value: &str) {
        self.commands.add_command(DriverDomCommand::UpdateText {
            id,
            value: value.into(),
        });
    }

    pub fn set_attr(&self, id: DomId, name: impl Into<StaticString>, value: &str) {
        let name = name.into();

        self.commands.add_command(DriverDomCommand::SetAttr {
            id,
            name,
            value: value.into(),
        });
    }

    pub fn remove_attr(&self, id: DomId, name: impl Into<StaticString>) {
        self.commands.add_command(DriverDomCommand::RemoveAttr {
            id,
            name: name.into(),
        });
    }

    pub fn remove_text(&self, id: DomId) {
        self.commands
            .add_command(DriverDomCommand::RemoveText { id });
    }

    pub fn remove_node(&self, id: DomId) {
        self.commands
            .add_command(DriverDomCommand::RemoveNode { id });
    }

    pub fn insert_before(&self, parent: DomId, child: DomId, ref_id: Option<DomId>) {
        self.commands.add_command(DriverDomCommand::InsertBefore {
            parent,
            child,
            ref_id,
        });

        if let Some(callback) = self.node_parent_callback.get(&child) {
            callback(parent);
        }
    }

    pub fn insert_css(&self, selector: Option<String>, value: String) {
        self.commands
            .add_command(DriverDomCommand::InsertCss { selector, value });
    }

    pub fn create_comment(&self, id: DomId, value: impl Into<String>) {
        self.commands.add_command(DriverDomCommand::CreateComment {
            id,
            value: value.into(),
        })
    }

    pub fn remove_comment(&self, id: DomId) {
        self.commands
            .add_command(DriverDomCommand::RemoveComment { id });
    }

    pub fn callback_add(&self, id: DomId, event_name: impl Into<String>, callback_id: CallbackId) {
        self.commands.add_command(DriverDomCommand::CallbackAdd {
            id,
            event_name: event_name.into(),
            callback_id,
        });
    }

    pub fn callback_remove(
        &self,
        id: DomId,
        event_name: impl Into<String>,
        callback_id: CallbackId,
    ) {
        self.commands.add_command(DriverDomCommand::CallbackRemove {
            id,
            event_name: event_name.into(),
            callback_id,
        });
    }

    pub fn flush_dom_changes(&self) {
        self.commands.flush_dom_changes();
    }

    pub fn node_parent(&self, node_id: DomId, callback: impl Fn(DomId) + 'static) -> DropResource {
        self.node_parent_callback.insert(node_id, Rc::new(callback));

        let node_parent_callback = self.node_parent_callback.clone();

        DropResource::new(move || {
            node_parent_callback.remove(&node_id);
        })
    }
}
