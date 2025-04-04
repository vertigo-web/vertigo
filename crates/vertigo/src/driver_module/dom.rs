use std::cell::Cell;
use std::rc::Rc;

use crate::struct_mut::{HashMapMut, ValueMut, VecMut};
use crate::{DomId, DropResource, JsJson};

use super::dom_suspense::DomSuspense;
use super::StaticString;
use super::{
    api::CallbackId,
    dom_command::{sort_commands, DriverDomCommand},
};
use crate::driver_module::api::ApiImport;

#[derive(PartialEq)]
enum StateInitiation {
    Waiting { counter: u32 },
    Ready,
}

impl StateInitiation {
    pub fn new() -> Self {
        StateInitiation::Waiting { counter: 0 }
    }

    pub fn is_ready(&self) -> bool {
        &Self::Ready == self
    }

    pub fn up(&mut self) {
        if let Self::Waiting { counter } = self {
            *counter += 1;
        }
    }

    ///
    /// Returns true if we have gone to zero while decreasing
    ///
    pub fn down(&mut self) -> bool {
        let send_commands = if let Self::Waiting { counter } = self {
            *counter -= 1;
            *counter == 0
        } else {
            false
        };

        if send_commands {
            *self = Self::Ready;
        }

        send_commands
    }
}

struct Commands {
    state: ValueMut<StateInitiation>,
    api: ApiImport,
    commands: VecMut<DriverDomCommand>,
    // For testing/debuging purposes
    log_enabled: Cell<bool>,
    log_vec: VecMut<DriverDomCommand>,
}

impl Commands {
    pub fn new(api: &ApiImport) -> &'static Self {
        Box::leak(Box::new(Commands {
            state: ValueMut::new(StateInitiation::new()),
            api: api.clone(),
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

    fn flush_dom_changes_inner(&self) {
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

    fn flush_dom_changes(&self) {
        let is_ready = self.state.map(|state| state.is_ready());

        if is_ready {
            self.flush_dom_changes_inner();
        }
    }

    fn fetch_up(&self) {
        self.state.change(|state| {
            state.up();
        });
    }

    fn fetch_down(&self) {
        let send_commands = self.state.change(|state| state.down());

        if send_commands {
            self.flush_dom_changes_inner();
        }
    }
}

type Callback = Rc<dyn Fn(DomId) + 'static>;

pub struct DriverDom {
    commands: &'static Commands,
    _sub1: DropResource,
    _sub2: DropResource,
    node_parent_callback: Rc<HashMapMut<DomId, Callback>>,
    pub dom_suspense: &'static DomSuspense,

    #[cfg(test)]
    _callback_id_lock: Cell<Option<std::sync::MutexGuard<'static, ()>>>,
}

impl DriverDom {
    pub fn new(api: &ApiImport) -> &'static DriverDom {
        let commands = Commands::new(api);

        let sub1 = api.on_fetch_start.add({
            move |_| {
                commands.fetch_up();
            }
        });

        let sub2 = api.on_fetch_stop.add({
            move |_| {
                commands.fetch_down();
            }
        });

        Box::leak(Box::new(DriverDom {
            commands,
            _sub1: sub1,
            _sub2: sub2,
            node_parent_callback: Rc::new(HashMapMut::new()),
            dom_suspense: DomSuspense::new(),

            #[cfg(test)]
            _callback_id_lock: Cell::new(None),
        }))
    }

    pub fn create_node(&self, id: DomId, name: impl Into<StaticString>) {
        let name = name.into();

        if name.as_str() == "vertigo-suspense" {
            self.dom_suspense.set_node_suspense(id);
        }

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
        self.dom_suspense.remove(id);
    }

    pub fn remove_node(&self, id: DomId) {
        self.commands
            .add_command(DriverDomCommand::RemoveNode { id });
        self.dom_suspense.remove(id);
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

        self.dom_suspense.set_parent(child, parent);
    }

    pub fn insert_css(&self, selector: &str, value: &str) {
        self.commands.add_command(DriverDomCommand::InsertCss {
            selector: selector.into(),
            value: value.into(),
        });
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
        self.dom_suspense.remove(id);
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
