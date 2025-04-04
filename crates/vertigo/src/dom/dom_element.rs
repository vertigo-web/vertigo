use std::rc::Rc;

use crate::{
    dom::{dom_id::DomId, dom_node::DomNode},
    driver_module::{driver::Driver, StaticString},
    get_driver,
    struct_mut::VecMut,
    AttrGroup, AttrGroupValue, Computed, Css, DomText, DropFileItem, DropResource, JsValue,
};

use crate::struct_mut::VecDequeMut;

use super::{
    attr_value::{AttrValue, CssAttrValue},
    callback::{Callback, Callback1},
    dom_element_class::DomElementClassMerge,
    dom_element_ref::DomElementRef,
    types::{DropFileEvent, KeyDownEvent},
};

/// A Real DOM representative - element kind
pub struct DomElement {
    driver: Driver,
    id_dom: DomId,
    child_node: VecDequeMut<DomNode>,
    subscriptions: VecMut<DropResource>,
    class_manager: DomElementClassMerge,
}

impl DomElement {
    pub fn new(name: impl Into<StaticString>) -> Self {
        let name = name.into();
        let id_dom = DomId::from_name(name.as_str());

        let driver = get_driver();

        driver.inner.dom.create_node(id_dom, name);

        let class_manager = DomElementClassMerge::new(driver, id_dom);

        Self {
            driver,
            id_dom,
            child_node: VecDequeMut::new(),
            subscriptions: VecMut::new(),
            class_manager,
        }
    }

    pub fn add_attr(&self, name: impl Into<StaticString>, value: impl Into<AttrValue>) {
        let name = name.into();
        let value = value.into();

        match value {
            AttrValue::String(value) => {
                self.class_manager.set_attr_value(name, Some(value));
            }
            AttrValue::Computed(computed) => {
                let class_manager = self.class_manager.clone();

                self.subscribe(computed, move |value| {
                    class_manager.set_attr_value(name.clone(), Some(value));
                });
            }
            AttrValue::ComputedOpt(computed) => {
                let class_manager = self.class_manager.clone();

                self.subscribe(computed, move |value| {
                    class_manager.set_attr_value(name.clone(), value);
                });
            }
            AttrValue::Value(value) => {
                let class_manager = self.class_manager.clone();

                self.subscribe(value.to_computed(), move |value| {
                    class_manager.set_attr_value(name.clone(), Some(value));
                });
            }
            AttrValue::ValueOpt(value) => {
                let class_manager = self.class_manager.clone();

                self.subscribe(value.to_computed(), move |value| {
                    class_manager.set_attr_value(name.clone(), value);
                });
            }
        };
    }

    pub fn add_attr_group(mut self, values: AttrGroup) -> Self {
        for (key, value) in values {
            self = self.add_attr_group_item(key, value);
        }
        self
    }

    pub fn add_attr_group_item(self, key: String, value: AttrGroupValue) -> Self {
        match (key.as_str(), value) {
            ("css", AttrGroupValue::Css(css)) => self.css(css), // tu on_click i inne warianty
            ("hook_key_down", AttrGroupValue::HookKeyDown(on_hook_key_down)) => {
                self.hook_key_down(on_hook_key_down)
            }
            ("on_blur", AttrGroupValue::OnBlur(on_blur)) => self.on_blur(on_blur),
            ("on_change", AttrGroupValue::OnChange(on_change)) => self.on_change(on_change),
            ("on_click", AttrGroupValue::OnClick(on_click)) => self.on_click(on_click),
            ("on_dropfile", AttrGroupValue::OnDropfile(on_dropfile)) => {
                self.on_dropfile(on_dropfile)
            }
            ("on_input", AttrGroupValue::OnInput(on_input)) => self.on_input(on_input),
            ("on_key_down", AttrGroupValue::OnKeyDown(on_key_down)) => {
                self.on_key_down(on_key_down)
            }
            ("on_load", AttrGroupValue::OnLoad(on_load)) => self.on_load(on_load),
            ("on_mouse_down", AttrGroupValue::OnMouseDown(on_mouse_down)) => {
                self.on_mouse_down(on_mouse_down)
            }
            ("on_mouse_enter", AttrGroupValue::OnMouseEnter(on_mouse_enter)) => {
                self.on_mouse_enter(on_mouse_enter)
            }
            ("on_mouse_leave", AttrGroupValue::OnMouseLeave(on_mouse_leave)) => {
                self.on_mouse_leave(on_mouse_leave)
            }
            ("on_mouse_up", AttrGroupValue::OnMouseUp(on_mouse_up)) => {
                self.on_mouse_up(on_mouse_up)
            }
            ("on_submit", AttrGroupValue::OnSubmit(on_submit))
            | ("form", AttrGroupValue::OnSubmit(on_submit)) => self.on_submit(on_submit),
            ("vertigo-suspense", AttrGroupValue::Suspense(callback)) => {
                self.suspense(Some(callback))
            }
            (_, AttrGroupValue::AttrValue(value)) => self.attr(key, value),
            (_, _) => {
                crate::log::error!("Invalid attribute type for key {key}");
                self
            }
        }
    }

    pub fn add_child(&self, child_node: impl Into<DomNode>) {
        let child_node = child_node.into();

        let child_id = child_node.id_dom();
        self.driver
            .inner
            .dom
            .insert_before(self.id_dom, child_id, None);

        self.child_node.push(child_node);
    }

    pub fn add_child_text(&self, text: impl Into<String>) {
        let text = text.into();
        self.add_child(DomNode::Text {
            node: DomText::new(text),
        });
    }

    pub fn attr(self, name: impl Into<StaticString>, value: impl Into<AttrValue>) -> Self {
        self.add_attr(name, value);
        self
    }

    pub fn attrs<T: Into<AttrValue>>(self, attrs: Vec<(impl Into<StaticString>, T)>) -> Self {
        for (name, value) in attrs.into_iter() {
            self.add_attr(name, value)
        }
        self
    }

    pub fn child(self, child_node: impl Into<DomNode>) -> Self {
        self.add_child(child_node);
        self
    }

    pub fn child_text(self, text: impl Into<String>) -> Self {
        self.add_child_text(text);
        self
    }

    pub fn children<C: Into<DomNode>>(self, children: Vec<C>) -> Self {
        for child_node in children.into_iter() {
            self.add_child(child_node)
        }
        self
    }

    pub fn css(self, css: impl Into<CssAttrValue>) -> Self {
        let css = css.into();

        match css {
            CssAttrValue::Css(css) => {
                self.class_manager.set_css(css);
            }
            CssAttrValue::Computed(css) => {
                let class_manager = self.class_manager.clone();

                self.subscribe(css, move |css| {
                    class_manager.set_css(css);
                });
            }
        }
        self
    }

    pub fn get_ref(&self) -> DomElementRef {
        DomElementRef::new(self.driver.inner.api.clone(), self.id_dom)
    }

    #[cfg(test)]
    pub fn get_children(&self) -> &VecDequeMut<DomNode> {
        &self.child_node
    }

    pub fn hook_key_down(self, on_hook_key_down: impl Into<Callback1<KeyDownEvent, bool>>) -> Self {
        let on_hook_key_down = self.install_callback1(on_hook_key_down);

        self.add_event_listener("hook_keydown", move |data| match get_key_down_event(data) {
            Ok(event) => {
                let prevent_default = on_hook_key_down(event);

                match prevent_default {
                    true => JsValue::True,
                    false => JsValue::False,
                }
            }
            Err(error) => {
                log::error!("export_websocket_callback_message -> params decode error -> {error}");
                JsValue::False
            }
        })
    }

    pub fn id_dom(&self) -> DomId {
        self.id_dom
    }

    pub fn on_blur(self, on_blur: impl Into<Callback<()>>) -> Self {
        let on_blur = self.install_callback(on_blur);

        self.add_event_listener("blur", move |_data| {
            on_blur();
            JsValue::Undefined
        })
    }

    pub fn on_change(self, on_change: impl Into<Callback1<String, ()>>) -> Self {
        let on_change = self.install_callback1(on_change);

        self.add_event_listener("change", move |data| {
            if let JsValue::String(text) = data {
                on_change(text);
            } else {
                log::error!("Invalid data: on_change: {data:?}");
            }

            JsValue::Undefined
        })
    }

    pub fn on_click(self, on_click: impl Into<Callback<()>>) -> Self {
        let on_click = self.install_callback(on_click);

        self.add_event_listener("click", move |_data| {
            on_click();
            JsValue::Undefined
        })
    }

    pub fn on_dropfile(self, on_dropfile: impl Into<Callback1<DropFileEvent, ()>>) -> Self {
        let on_dropfile = self.install_callback1(on_dropfile);

        self.add_event_listener("drop", move |data| {
            let params = data.convert(|mut params| {
                let files = params.get_vec("drop file", |item| {
                    item.convert(|mut item| {
                        let name = item.get_string("name")?;
                        let data = item.get_buffer("data")?;

                        Ok(DropFileItem::new(name, data))
                    })
                })?;

                Ok(DropFileEvent::new(files))
            });

            match params {
                Ok(params) => {
                    on_dropfile(params);
                }
                Err(error) => {
                    log::error!("on_dropfile -> params decode error -> {error}");
                }
            };

            JsValue::Undefined
        })
    }

    pub fn on_input(self, on_input: impl Into<Callback1<String, ()>>) -> Self {
        let on_input = self.install_callback1(on_input);

        self.add_event_listener("input", move |data| {
            if let JsValue::String(text) = data {
                on_input(text);
            } else {
                log::error!("Invalid data: on_input: {data:?}");
            }

            JsValue::Undefined
        })
    }

    pub fn on_key_down(self, on_key_down: impl Into<Callback1<KeyDownEvent, bool>>) -> Self {
        let on_key_down = self.install_callback1(on_key_down);

        self.add_event_listener("keydown", move |data| match get_key_down_event(data) {
            Ok(event) => {
                let prevent_default = on_key_down(event);

                match prevent_default {
                    true => JsValue::True,
                    false => JsValue::False,
                }
            }
            Err(error) => {
                log::error!("export_websocket_callback_message -> params decode error -> {error}");
                JsValue::False
            }
        })
    }

    pub fn on_load(self, on_load: impl Into<Callback<()>>) -> Self {
        let on_load = self.install_callback(on_load);

        self.add_event_listener("load", move |_data| {
            on_load();
            JsValue::Undefined
        })
    }

    pub fn on_mouse_down(self, on_mouse_down: impl Into<Callback<bool>>) -> Self {
        let on_mouse_down = self.install_callback(on_mouse_down);

        self.add_event_listener("mousedown", move |_data| {
            if on_mouse_down() {
                JsValue::True
            } else {
                JsValue::False
            }
        })
    }

    pub fn on_mouse_enter(self, on_mouse_enter: impl Into<Callback<()>>) -> Self {
        let on_mouse_enter = self.install_callback(on_mouse_enter);

        self.add_event_listener("mouseenter", move |_data| {
            on_mouse_enter();
            JsValue::Undefined
        })
    }

    pub fn on_mouse_leave(self, on_mouse_leave: impl Into<Callback<()>>) -> Self {
        let on_mouse_leave = self.install_callback(on_mouse_leave);

        self.add_event_listener("mouseleave", move |_data| {
            on_mouse_leave();
            JsValue::Undefined
        })
    }

    pub fn on_mouse_up(self, on_mouse_up: impl Into<Callback<bool>>) -> Self {
        let on_mouse_up = self.install_callback(on_mouse_up);

        self.add_event_listener("mouseup", move |_data| {
            if on_mouse_up() {
                JsValue::True
            } else {
                JsValue::False
            }
        })
    }

    pub fn on_submit(self, on_submit: impl Into<Callback<()>>) -> Self {
        let on_submit = self.install_callback(on_submit);

        self.add_event_listener("submit", move |_data| {
            on_submit();
            JsValue::Undefined
        })
    }

    pub fn suspense(self, callback: Option<fn(bool) -> Css>) -> Self {
        self.class_manager.set_suspense_attr(callback);
        self
    }

    fn subscribe<T: Clone + PartialEq + 'static>(
        &self,
        value: Computed<T>,
        call: impl Fn(T) + 'static,
    ) {
        let client = value.subscribe(call);
        self.subscriptions.push(client);
    }

    fn add_event_listener(
        self,
        name: &'static str,
        callback: impl Fn(JsValue) -> JsValue + 'static,
    ) -> Self {
        let (callback_id, drop) = self.driver.inner.api.callback_store.register(callback);

        let drop_event = DropResource::new(move || {
            self.driver
                .inner
                .dom
                .callback_remove(self.id_dom, name, callback_id);
            drop.off();
        });

        self.driver
            .inner
            .dom
            .callback_add(self.id_dom, name, callback_id);
        self.subscriptions.push(drop_event);
        self
    }

    fn install_callback<R: 'static>(
        &self,
        callback: impl Into<Callback<R>>,
    ) -> Rc<dyn Fn() -> R + 'static> {
        let callback: Callback<R> = callback.into();
        let (callback, drop) = callback.subscribe();
        if let Some(drop) = drop {
            self.subscriptions.push(drop);
        }
        callback
    }

    fn install_callback1<T: 'static, R: 'static>(
        &self,
        callback: impl Into<Callback1<T, R>>,
    ) -> Rc<dyn Fn(T) -> R + 'static> {
        let callback: Callback1<T, R> = callback.into();
        let (callback, drop) = callback.subscribe();
        if let Some(drop) = drop {
            self.subscriptions.push(drop);
        }
        callback
    }
}

impl Drop for DomElement {
    fn drop(&mut self) {
        self.driver.inner.dom.remove_node(self.id_dom);
    }
}

fn get_key_down_event(data: JsValue) -> Result<KeyDownEvent, String> {
    data.convert(|mut params| {
        let key = params.get_string("key")?;
        let code = params.get_string("code")?;
        let alt_key = params.get_bool("altKey")?;
        let ctrl_key = params.get_bool("ctrlKey")?;
        let shift_key = params.get_bool("shiftKey")?;
        let meta_key = params.get_bool("metaKey")?;
        params.expect_no_more()?;

        Ok(KeyDownEvent {
            key,
            code,
            alt_key,
            ctrl_key,
            shift_key,
            meta_key,
        })
    })
}
