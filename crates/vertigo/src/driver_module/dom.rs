use std::rc::Rc;
use vertigo_macro::store;

use crate::{
    DomId, DropResource,
    dev::{CallbackId, ValueMut, command::DriverDomCommand},
    driver_module::{
        api::{api_browser_command, api_dom_snapshot},
        event_emitter::EventEmitter,
        hydration::{Reconciled, discard, reconcile, split_buffer},
    },
    struct_mut::{HashMapMut, VecMut},
};

use super::StaticString;

struct Commands {
    commands: VecMut<DriverDomCommand>,
    /// Opt-in tap on the command stream, used by [`crate::dev::inspect`]. Nothing
    /// subscribes to it unless a debugging session asks for it.
    new_command: EventEmitter<DriverDomCommand>,
    /// When armed, `flush_dom_changes` sends nothing.
    ///
    /// Flush fires during mount twice: once from the `on_after_transaction` hook after
    /// closing the mount transaction, once explicitly after `flush_watch`. The comparison
    /// must cover the complete tree, so both those moments must be silenced, and the send
    /// is done by `flush_hydration`.
    hydration: ValueMut<bool>,
    /// Inspection of the actually sent batches. `new_command` fires when a command is
    /// queued, which is the wrong moment for anything that wants to see what the browser
    /// received: hydration replaces the queued stream with the reconciled one.
    new_batch: EventEmitter<Vec<DriverDomCommand>>,
}

impl Commands {
    pub fn new() -> Self {
        Commands {
            commands: VecMut::new(),
            new_command: EventEmitter::default(),
            hydration: ValueMut::new(false),
            new_batch: EventEmitter::default(),
        }
    }

    fn inspect_command(&self, func: impl Fn(DriverDomCommand) + 'static) -> DropResource {
        self.new_command.add(func)
    }

    fn inspect_batch(&self, func: impl Fn(Vec<DriverDomCommand>) + 'static) -> DropResource {
        self.new_batch.add(func)
    }

    fn add_command(&self, command: DriverDomCommand) {
        self.new_command.trigger(&command);
        self.commands.push(command);
    }

    fn send(&self, commands: Vec<DriverDomCommand>) {
        if commands.is_empty() {
            return;
        }

        let commands = sort_commands(commands);
        self.new_batch.trigger(&commands);
        api_browser_command().dom_bulk_update(commands);
    }

    fn flush_dom_changes(&self) {
        if self.hydration.get() {
            return;
        }

        self.send(self.commands.take());
    }

    fn arm_hydration(&self) {
        self.hydration.set(true);
    }

    /// Ends mount: fetches snapshot, reconciles buffer against browser state, sends
    /// and disarms the mode. From this moment everything returns to ordinary flushing
    /// after transaction.
    fn flush_hydration(&self) {
        self.hydration.set(false);

        let commands = self.commands.take();

        if commands.is_empty() {
            return;
        }

        let commands = match api_dom_snapshot().get() {
            Some(snapshot) => {
                if hydration_disabled() {
                    discard(split_buffer(commands), &snapshot)
                } else {
                    let Reconciled { commands, report } =
                        reconcile(split_buffer(commands), &snapshot);
                    report.publish();
                    commands
                }
            }
            None => commands,
        };

        self.send(commands);
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

    /// Watch every DOM command as it is produced. For debugging and tests only - each
    /// subscriber gets its own clone of every command.
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

    pub(crate) fn arm_hydration(&self) {
        self.commands.arm_hydration();
    }

    pub(crate) fn flush_hydration(&self) {
        self.commands.flush_hydration();
    }

    pub fn inspect_batch(&self, func: impl Fn(Vec<DriverDomCommand>) + 'static) -> DropResource {
        self.commands.inspect_batch(func)
    }

    pub fn node_parent(&self, node_id: DomId, callback: impl Fn(DomId) + 'static) -> DropResource {
        self.node_parent_callback.insert(node_id, Rc::new(callback));

        let node_parent_callback = self.node_parent_callback.clone();

        DropResource::new(move || {
            node_parent_callback.remove(&node_id);
        })
    }
}

/// Command-line flag `--disable-hydration`, inserted by cli as
/// `data-env-disable-hydration` and read the same way as other environment variables.
///
/// The policy belongs to rust: js returns the snapshot regardless of the flag, and the
/// decision whether to adopt anything is made by this branch.
fn hydration_disabled() -> bool {
    api_browser_command().get_env("disable-hydration") == Some("true".to_string())
}
