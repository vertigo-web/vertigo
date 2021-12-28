use std::rc::Rc;
use vertigo::{
    dev::{EventCallback, RealDomId, RefsContext},
    Dependencies, KeyDownEvent,
};
use wasm_bindgen::prelude::Closure;

use vertigo::struct_mut::VecMut;

use super::{
    driver_data::DriverData,
    driver_dom_command::DriverDomCommand,
    js_dom::DriverBrowserDomJs,
    visited_node_manager::VisitedNodeManager,
};

type KeydownClosureType = Closure<dyn Fn(Option<u64>, String, String, bool, bool, bool, bool) -> bool>;

struct DriverDomInner {
    data: Rc<DriverData>,
    dom_js: Rc<DriverBrowserDomJs>,
    _mouse_down: Closure<dyn Fn(u64)>,
    _mouse_enter: Closure<dyn Fn(Option<u64>)>,
    _keydown: KeydownClosureType,
    _oninput: Closure<dyn Fn(u64, String)>,
    refs: VecMut<RefsContext>,
    commands: VecMut<DriverDomCommand>,
}

#[derive(Clone)]
pub struct DriverBrowserDom {
    inner: Rc<DriverDomInner>,
}

impl DriverBrowserDom {
    pub fn new(dependencies: &Dependencies) -> DriverBrowserDom {
        let data = DriverData::new();

        let mouse_down = {
            let data = data.clone();

            Closure::new(move |dom_id: u64| {
                let event_to_run = data.find_event_click(RealDomId::from_u64(dom_id));

                if let Some(event_to_run) = event_to_run {
                    event_to_run();
                }
            })
        };

        let mouse_enter: Closure<dyn Fn(Option<u64>)> = {
            let data = data.clone();
            let current_visited = VisitedNodeManager::new(&data, dependencies);

            Closure::new(move |dom_id: Option<u64>| {
                match dom_id {
                    Some(dom_id) => {
                        let nodes = data.find_all_nodes(RealDomId::from_u64(dom_id));
                        current_visited.push_new_nodes(nodes);
                    }
                    None => {
                        current_visited.clear();
                    }
                }
            })
        };

        let keydown: KeydownClosureType = {
            let data = data.clone();

            Closure::new(
                move |
                    dom_id: Option<u64>,
                    key: String,
                    code: String,
                    alt_key: bool,
                    ctrl_key: bool,
                    shift_key: bool,
                    meta_key: bool
                | -> bool {
                    let event = KeyDownEvent {
                        key,
                        code,
                        alt_key,
                        ctrl_key,
                        shift_key,
                        meta_key,
                    };

                    let id = match dom_id {
                        Some(id) => RealDomId::from_u64(id),
                        None => RealDomId::root(),
                    };

                    let event_to_run = data.find_event_keydown(id);

                    if let Some(event_to_run) = event_to_run {
                        let prevent_default = event_to_run(event);

                        if prevent_default {
                            return true;
                        }
                    }

                    false
                }
            )
        };

        let oninput: Closure<dyn Fn(u64, String)> = {
            let data = data.clone();

            Closure::new(move |dom_id: u64, text: String| {
                let event_to_run = data.find_event_on_input(RealDomId::from_u64(dom_id));

                if let Some(event_to_run) = event_to_run {
                    event_to_run(text);
                }
            })
        };

        let dom_js = Rc::new(DriverBrowserDomJs::new(&mouse_down, &mouse_enter, &keydown, &oninput));

        let driver_browser = DriverBrowserDom {
            inner: Rc::new(DriverDomInner {
                data,
                dom_js,
                _mouse_down: mouse_down,
                _mouse_enter: mouse_enter,
                _keydown: keydown,
                _oninput: oninput,
                refs: VecMut::new(),
                commands: VecMut::new(),
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
            self.inner.dom_js.bulk_update(command_str.as_str());
        }

        let refs = self.inner.refs.take();

        for context in refs {
            context.run();
        }
    }

    pub fn set_event(&self, id: RealDomId, callback: EventCallback) {
        self.inner.data.set_event(id, callback);
    }

    pub fn get_bounding_client_rect_x(&self, id: RealDomId) -> f64 {
        self.inner.dom_js.get_bounding_client_rect_x(id.to_u64())
    }

    pub fn get_bounding_client_rect_y(&self, id: RealDomId) -> f64 {
        self.inner.dom_js.get_bounding_client_rect_y(id.to_u64())
    }

    pub fn get_bounding_client_rect_width(&self, id: RealDomId) -> f64 {
        self.inner.dom_js.get_bounding_client_rect_width(id.to_u64())
    }

    pub fn get_bounding_client_rect_height(&self, id: RealDomId) -> f64 {
        self.inner.dom_js.get_bounding_client_rect_height(id.to_u64())
    }

    pub fn scroll_top(&self, id: RealDomId) -> i32 {
        self.inner.dom_js.scroll_top(id.to_u64())
    }

    pub fn set_scroll_top(&self, id: RealDomId, value: i32) {
        self.inner.dom_js.set_scroll_top(id.to_u64(), value)
    }

    pub fn scroll_left(&self, id: RealDomId) -> i32 {
        self.inner.dom_js.scroll_left(id.to_u64())
    }

    pub fn set_scroll_left(&self, id: RealDomId, value: i32) {
        self.inner.dom_js.set_scroll_left(id.to_u64(), value)
    }

    pub fn scroll_width(&self, id: RealDomId) -> i32 {
        self.inner.dom_js.scroll_width(id.to_u64())
    }

    pub fn scroll_height(&self, id: RealDomId) -> i32 {
        self.inner.dom_js.scroll_height(id.to_u64())
    }

    pub(crate) fn push_ref_context(&self, context: RefsContext) {
        self.inner.refs.push(context);
    }
}
