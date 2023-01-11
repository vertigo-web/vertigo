//! Vertigo is a library for building reactive web components.
//!
//! It mainly consists of three parts:
//!
//! * **Reactive dependencies** - A graph of values and clients (micro-subscriptions)
//!   that can automatically compute what to refresh after one or more change(s)
//! * **Real DOM operations** - No intermediate Virtual DOM mechanism is necessary
//! * **HTML/CSS macros** - Allows to construct Real DOM nodes using HTML and CSS
//!
//! ## Example 1
//!
//! ```rust
//! use vertigo::{dom, DomElement, Value, bind, start_app};
//!
//! pub fn render(count: &Value<i32>) -> DomElement {
//!     let increment = bind!(count, || {
//!         count.change(|value| {
//!             *value += 1;
//!         });
//!     });
//!
//!     let decrement = bind!(count, || {
//!         count.change(|value| {
//!             *value -= 1;
//!         });
//!     });
//!
//!     dom! {
//!         <div>
//!             <p>"Counter: " { count }</p>
//!             <button on_click={decrement}>"-"</button>
//!             <button on_click={increment}>"+"</button>
//!         </div>
//!     }
//! }
//!
//! #[no_mangle]
//! pub fn start_application() {
//!     let count = Value::new(0);
//!     let view = render(&count);
//!     start_app(count, view);
//! }
//! ```
//!
//! ## Example 2
//!
//! ```rust
//! use vertigo::{css, DomElement, Value, dom};
//!
//! pub struct MyMessage {
//!     pub message: Value<String>,
//! }
//!
//! impl MyMessage {
//!     pub fn mount(self) -> DomElement {
//!         dom! {
//!             <p>
//!                 "Message to the world: "
//!                 { self.message }
//!             </p>
//!         }
//!     }
//! }
//!
//! fn render() -> DomElement {
//!     let message = Value::new("Hello world!".to_string());
//!
//!     let main_div = css!("
//!         color: darkblue;
//!     ");
//!
//!     dom! {
//!         <div css={main_div}>
//!             <MyMessage message={message} />
//!         </div>
//!     }
//! }
//! ```
//!
//! To get started you may consider looking at the
//! [Tutorial](https://github.com/vertigo-web/vertigo/blob/master/tutorial.md).

#![deny(rust_2018_idioms)]
#![feature(try_trait_v2)] // https://github.com/rust-lang/rust/issues/84277

mod computed;
mod css;
mod dom_list;
mod dom_value;
mod dom;
mod driver_module;
mod fetch;
mod future_box;
mod html_macro;
pub mod inspect;
mod instant;
pub mod router;
mod websocket;
mod external_api;

use std::any::Any;
use computed::struct_mut::ValueMut;

pub use computed::{
    AutoMap, Client, Computed, context::Context, Dependencies, DropResource, GraphId, struct_mut, Value
};
pub use dom::{
    css::{Css, CssGroup},
    dom_id::DomId,
    dom_comment::DomComment,
    dom_comment_create::DomCommentCreate,
    dom_element::DomElement,
    dom_node::{DomNode, DomNodeFragment},
    dom_text::DomText,
    types::{KeyDownEvent, DropFileEvent, DropFileItem},
};
pub use dom_list::ListRendered;
use driver_module::{init_env::init_env, api::CallbackId};
pub use driver_module::{
    api::ApiImport,
    driver::{Driver, FetchResult, FetchMethod},
    js_value::{
        MemoryBlock,
        JsValue,
        JsJson,
        JsJsonSerialize,
        JsJsonDeserialize,
        JsJsonContext,
        JsJsonObjectBuilder,
        from_json,
        to_json
    },
    dom_command::DriverDomCommand,
};
pub use fetch::{
    lazy_cache::{self, LazyCache},
    pinboxfut::PinBoxFuture,
    request_builder::{RequestResponse, RequestBuilder, RequestBody},
    resource::Resource,
};
pub use future_box::{FutureBoxSend, FutureBox};
pub use html_macro::{
    EmbedDom
};
pub use instant::{
    Instant, InstantType
};
pub use websocket::{WebsocketConnection, WebsocketMessage};

/// Allows to include a static file
///
/// This will place the file along with the rest of generated files. The macro returns a public path to the file with it's hash in name.
pub use vertigo_macro::include_static;

/// Allows to create an event handler based on provided arguments
pub use vertigo_macro::bind;

/// Allows to create an event handler based on provided arguments which is wrapped in Rc
pub use vertigo_macro::bind_rc;

/// Allows to create an event handler based on provided arguments which launches an asynchronous action
pub use vertigo_macro::bind_spawn;

pub use vertigo_macro::AutoJsJson;
pub use vertigo_macro::component;

// Export log module which can be used in vertigo plugins
pub use log;

/// Allows to create DomElement using HTML tags.
///
/// ```rust
/// use vertigo::dom;
///
/// let value = "world";
///
/// dom! {
///     <div>
///         <h3>"Hello " {value} "!"</h3>
///         <p>"Good morning!"</p>
///     </div>
/// };
/// ```
pub use vertigo_macro::dom;

/// Version of `dom!` macro that additionally emits compiler warning with generated code.
pub use vertigo_macro::dom_debug;

/// Allows to create Css styles for virtual DOM.
///
/// ```rust
/// use vertigo::css;
///
/// let green_on_red = css!("
///     color: green;
///     background-color: red;
/// ");
/// ```
pub use vertigo_macro::css;

/// Constructs a CSS block that can be manually pushed into existing Css styles instance.
///
/// ```rust
/// use vertigo::{css, css_block};
///
/// let mut green_style = css!("
///     color: green;
/// ");
///
/// green_style.push_str(
///     css_block! { "
///         font-style: italic;
///    " }
/// );
/// ```
pub use vertigo_macro::css_block;

pub mod html_entities;


pub struct DriverConstruct {
    driver: Driver,
    state: ValueMut<Option<Box<dyn Any>>>,
    subscription: ValueMut<Option<DomElement>>,
}

impl DriverConstruct {
    fn new() -> DriverConstruct {
        let driver = Driver::default();

        DriverConstruct {
            driver,
            state: ValueMut::new(None),
            subscription: ValueMut::new(None),
        }
    }

    fn set_root(&self, state: Box<dyn Any>, root: DomElement) {
        self.state.set(Some(state));
        self.subscription.set(Some(root));
    }
}

thread_local! {
    static DRIVER_BROWSER: DriverConstruct = DriverConstruct::new();
}

/// Starting point of the app.
pub fn start_app(app_state: impl Any + 'static, view: DomElement) {
    get_driver_state("start_app", |state| {
        init_env(state.driver.inner.api.clone());
        state.driver.inner.api.on_fetch_start.trigger(());

        state.set_root(Box::new(app_state), view);

        state.driver.inner.api.on_fetch_stop.trigger(());
        get_driver().inner.dom.flush_dom_changes();
    });
}

pub fn start_app_fn<S: Any + 'static>(init_app: impl FnOnce() -> (S, DomElement)) {
    get_driver_state("start_app", |state| {
        init_env(state.driver.inner.api.clone());

        let (state, dom) = init_app();
        start_app(state, dom);
    });
}

/// Getter for Driver singleton.
pub fn get_driver() -> Driver {
    DRIVER_BROWSER.with(|state| {
        state.driver.clone()
    })
}

/// Do bunch of operations without triggering anything in between.
pub fn transaction<R, F: FnOnce(&Context) -> R>(f: F) -> R {
    get_driver().transaction(f)
}

//------------------------------------------------------------------------------------------------------------------
// Internals below
//------------------------------------------------------------------------------------------------------------------

fn get_driver_state<R: Default, F: FnOnce(&DriverConstruct) -> R>(label: &'static str, once: F) -> R {
    match DRIVER_BROWSER.try_with(once) {
        Ok(value) => value,
        Err(_) => {
            if label != "free" {
                println!("error access {label}");
            }

            R::default()
        }
    }
}

// Methods for memory allocation

#[no_mangle]
pub fn alloc(size: u32) -> u32 {
    get_driver_state("alloc", |state| {
        state.driver.inner.api.arguments.alloc(size)
    })
}

#[no_mangle]
pub fn free(pointer: u32) {
    get_driver_state("free", |state| {
        state.driver.inner.api.arguments.free(pointer);
    })
}

// Callbacks gateways

#[no_mangle]
pub fn wasm_callback(callback_id: u64, value_ptr: u32) -> u64 {
    get_driver_state("export_dom_callback", |state| {

        let value = state.driver.inner.api.arguments.get_by_ptr(value_ptr);
        let callback_id = CallbackId::from_u64(callback_id);

        let mut result = JsValue::Undefined;

        state.driver.transaction(|_| {
            result = state.driver.inner.api.callback_store.call(callback_id, value);
        });

        if result == JsValue::Undefined {
            return 0;
        }

        let memory_block = result.to_snapshot();
        let (ptr, size) = memory_block.get_ptr_and_size();
        state.driver.inner.api.arguments.set(memory_block);

        let ptr = ptr as u64;
        let size = size as u64;

        (ptr << 32) + size
    })
}
