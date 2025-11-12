use std::cell::Cell;
use std::rc::Rc;

use crate::struct_mut::{HashMapMut, VecMut};
use crate::{DomId, DropResource, JsJson};

use super::StaticString;
use super::{
    api::CallbackId,
    dom_command::{sort_commands, DriverDomCommand},
};
use crate::driver_module::api::{api_import};

struct Commands {
    commands: VecMut<DriverDomCommand>,
    // For testing/debuging purposes
    log_enabled: Cell<bool>,
    log_vec: VecMut<DriverDomCommand>,
}

impl Commands {
    pub fn new() -> &'static Self {
        Box::leak(Box::new(Commands {
            commands: VecMut::new(),
            log_enabled: Cell::new(false),
            log_vec: VecMut::new(),
        }))
    }

    fn log_start(&self) {
        if self.log_enabled.replace(true) {
            println!("log_start: already started");
        }
    }

    fn log_take(&self) -> Vec<DriverDomCommand> {
        self.log_enabled.replace(false);
        self.log_vec.take()
    }

    fn add_command(&self, command: DriverDomCommand) {
        if self.log_enabled.get() {
            self.log_vec.push(command.clone());
        }

        self.commands.push(command);
    }

    fn flush_dom_changes(&self) {
        let state = self.commands.take();

        if !state.is_empty() {
            let mut out = Vec::<JsJson>::new();

            let state = sort_commands(state);

            for command in state {
                out.push(command.into_string());
            }

            let out = JsJson::List(out);
            api_import().dom_bulk_update(out);
        }
    }
}

type Callback = Rc<dyn Fn(DomId) + 'static>;

pub struct DriverDom {
    commands: &'static Commands,
    node_parent_callback: Rc<HashMapMut<DomId, Callback>>,

    #[cfg(test)]
    _callback_id_lock: Cell<Option<std::sync::MutexGuard<'static, ()>>>,
}

impl DriverDom {
    pub fn new() -> &'static DriverDom {
        let commands = Commands::new();

        Box::leak(Box::new(DriverDom {
            commands,
            node_parent_callback: Rc::new(HashMapMut::new()),

            #[cfg(test)]
            _callback_id_lock: Cell::new(None),
        }))
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

    pub fn log_start(&self) {
        #[cfg(test)]
        {
            let lock = SEMAPHORE.lock().unwrap();
            CallbackId::reset();
            self._callback_id_lock.set(Some(lock));
        };

        self.commands.log_start();
    }

    pub fn log_take(&self) -> Vec<DriverDomCommand> {
        let log = self.commands.log_take();
        #[cfg(test)]
        self._callback_id_lock.set(None);
        log
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

#[cfg(test)]
/// Use in tests to block callback id generation in simultaneous async tests
static SEMAPHORE: std::sync::Mutex<()> = std::sync::Mutex::new(());
