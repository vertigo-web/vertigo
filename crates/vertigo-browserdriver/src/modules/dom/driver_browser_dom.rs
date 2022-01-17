use std::rc::Rc;
use vertigo::{
    dev::{EventCallback, RealDomId, RefsContext},
    Dependencies, KeyDownEvent,
};
use vertigo::struct_mut::VecMut;

use crate::api::ApiImport;

use super::{
    driver_data::DriverData,
    driver_dom_command::DriverDomCommand,
    visited_node_manager::VisitedNodeManager,
};


struct DriverDomInner {
    api: Rc<ApiImport>,
    data: Rc<DriverData>,
    refs: VecMut<RefsContext>,
    commands: VecMut<DriverDomCommand>,
    current_visited: VisitedNodeManager,
}

#[derive(Clone)]
pub struct DriverBrowserDom {
    inner: Rc<DriverDomInner>,
}

impl DriverBrowserDom {
    pub fn new(dependencies: &Dependencies, api: &Rc<ApiImport>) -> DriverBrowserDom {
        let data = DriverData::new();
        let current_visited = VisitedNodeManager::new(&data, dependencies);

        let driver_browser = DriverBrowserDom {
            inner: Rc::new(DriverDomInner {
                api: api.clone(),
                data,
                refs: VecMut::new(),
                commands: VecMut::new(),
                current_visited,
            }),
        };

        let root_id = RealDomId::root();

        driver_browser.create_node(root_id, "div");
        driver_browser.mount_node(root_id);

        dependencies.set_hook(
            Box::new(|| {}),
            {
                let driver_browser = driver_browser.clone();
                Box::new(move || {
                    driver_browser.flush_dom_changes();
                })
            }
        );

        driver_browser
    }
}

impl DriverBrowserDom {

    pub fn export_dom_mousedown(&self, dom_id: u64) {
        let event_to_run = self.inner.data.find_event_click(RealDomId::from_u64(dom_id));

        if let Some(event_to_run) = event_to_run {
            event_to_run();
        }
    }

    pub fn export_dom_mouseover(&self, dom_id: Option<u64>) {
        match dom_id {
            None => {
                self.inner.current_visited.clear();
            },
            Some(dom_id) => {
                let nodes = self.inner.data.find_all_nodes(RealDomId::from_u64(dom_id));
                self.inner.current_visited.push_new_nodes(nodes);
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

        let id = match dom_id {
            None => RealDomId::root(),
            Some(id) => RealDomId::from_u64(id),
        };

        match self.inner.data.find_event_keydown(id) {
            Some(event_to_run) => event_to_run(event),
            None => false,
        }
    }

    pub fn export_dom_oninput(&self, dom_id: u64, text: String) {
        let event_to_run = self.inner.data.find_event_on_input(RealDomId::from_u64(dom_id));

        if let Some(event_to_run) = event_to_run {
            event_to_run(text);
        }
    }

    fn mount_node(&self, id: RealDomId) {
        self.inner.commands.push(DriverDomCommand::MountNode { id });
    }

    fn add_command(&self, command: DriverDomCommand) {
        self.inner.commands.push(command);
    }

    pub fn create_node(&self, id: RealDomId, name: &'static str) {
        self.inner.data.create_node(id);
        self.add_command(DriverDomCommand::CreateNode { id, name });
    }

    pub fn rename_node(&self, id: RealDomId, name: &'static str) {
        self.add_command(DriverDomCommand::RenameNode { id, new_name: name })
    }

    pub fn create_text(&self, id: RealDomId, value: &str) {
        self.add_command(DriverDomCommand::CreateText {
            id,
            value: value.into(),
        })
    }

    pub fn update_text(&self, id: RealDomId, value: &str) {
        self.add_command(DriverDomCommand::UpdateText {
            id,
            value: value.into(),
        });
    }

    pub fn set_attr(&self, id: RealDomId, key: &'static str, value: &str) {
        self.add_command(DriverDomCommand::SetAttr {
            id,
            key,
            value: value.into(),
        });
    }

    pub fn remove_attr(&self, id: RealDomId, name: &'static str) {
        self.add_command(DriverDomCommand::RemoveAttr { id, name });
    }

    pub fn remove_text(&self, id: RealDomId) {
        self.inner.data.remove_text(id);
        self.add_command(DriverDomCommand::RemoveText { id });
    }

    pub fn remove_node(&self, id: RealDomId) {
        self.inner.data.remove_node(id);
        self.add_command(DriverDomCommand::RemoveNode { id });
    }

    pub fn insert_before(&self, parent: RealDomId, child: RealDomId, ref_id: Option<RealDomId>) {
        self.inner.data.set_parent(child, parent);
        self.add_command(DriverDomCommand::InsertBefore { parent, child, ref_id });
    }

    pub fn insert_css(&self, selector: &str, value: &str) {
        self.add_command(DriverDomCommand::InsertCss {
            selector: selector.into(),
            value: value.into(),
        });
    }

    pub fn flush_dom_changes(&self) {
        let state = self.inner.commands.take();

        if !state.is_empty() {
            let mut out = Vec::<String>::new();

            for command in state {
                out.push(command.into_string());
            }
    
            let command_str = format!("[{}]", out.join(","));
            self.inner.api.dom_bulk_update(command_str.as_str());
        }

        let refs = self.inner.refs.take();

        for context in refs {
            context.run();
        }
    }

    pub fn set_event(&self, id: RealDomId, callback: EventCallback) {
        self.inner.data.set_event(id, callback);
    }

    pub fn get_bounding_client_rect_x(&self, id: RealDomId) -> i32 {
        self.inner.api.dom_get_bounding_client_rect_x(id.to_u64())
    }

    pub fn get_bounding_client_rect_y(&self, id: RealDomId) -> i32 {
        self.inner.api.dom_get_bounding_client_rect_y(id.to_u64())
    }

    pub fn get_bounding_client_rect_width(&self, id: RealDomId) -> u32 {
        self.inner.api.dom_get_bounding_client_rect_width(id.to_u64())
    }

    pub fn get_bounding_client_rect_height(&self, id: RealDomId) -> u32 {
        self.inner.api.dom_get_bounding_client_rect_height(id.to_u64())
    }

    pub fn scroll_top(&self, id: RealDomId) -> i32 {
        self.inner.api.dom_scroll_top(id.to_u64())
    }

    pub fn set_scroll_top(&self, id: RealDomId, value: i32) {
        self.inner.api.dom_set_scroll_top(id.to_u64(), value)
    }

    pub fn scroll_left(&self, id: RealDomId) -> i32 {
        self.inner.api.dom_scroll_left(id.to_u64())
    }

    pub fn set_scroll_left(&self, id: RealDomId, value: i32) {
        self.inner.api.dom_set_scroll_left(id.to_u64(), value)
    }

    pub fn scroll_width(&self, id: RealDomId) -> u32 {
        self.inner.api.dom_scroll_width(id.to_u64())
    }

    pub fn scroll_height(&self, id: RealDomId) -> u32 {
        self.inner.api.dom_scroll_height(id.to_u64())
    }

    pub(crate) fn push_ref_context(&self, context: RefsContext) {
        self.inner.refs.push(context);
    }
}
