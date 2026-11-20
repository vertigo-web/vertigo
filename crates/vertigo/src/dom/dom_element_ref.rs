use crate::{
    dom::dom_id::DomId,
    driver_module::api::{DomAccess, api_browser_command},
};

/// A reference to [DomElement](crate::DomElement).
///
/// Use [DomElement::get_ref](crate::DomElement::get_ref) to obtain. See [js!](crate::js!) macro for an example.
#[derive(Clone)]
pub struct DomElementRef {
    id: DomId,
}

impl DomElementRef {
    pub fn new(id: DomId) -> DomElementRef {
        DomElementRef { id }
    }

    pub(crate) fn id(&self) -> DomId {
        self.id
    }

    pub fn dom_access(&self) -> DomAccess {
        DomAccess::default().element(self.id)
    }

    /// Locks the pointer to this element: the mouse's pointer disappears and stays put, and its
    /// moves come to the element's pointer listeners as [`PointerEvent::movement_x`] and
    /// [`movement_y`](PointerEvent::movement_y) - f. ex. for a game to look around with the mouse.
    ///
    /// The browser locks it only right after a click or a key pressed on the page, so this goes
    /// in their listener. It lets the pointer go on Esc, which the page doesn't hear then, and
    /// wants another click to lock it again. [`DomElement::hook_pointer_lock`] tells when the
    /// pointer is locked and let go, and [`Driver::exit_pointer_lock`] lets it go.
    ///
    /// ```rust
    /// use vertigo::{ClickEvent, PointerEvent, Value, bind, dom, dom_element, transaction};
    ///
    /// let looking = Value::new(false);
    /// let heading = Value::new(0.0);
    ///
    /// let view = dom_element! { <canvas /> };
    /// let view_ref = view.get_ref();
    /// let view = view
    ///     .on_click(move |_: ClickEvent| view_ref.request_pointer_lock())
    ///     .hook_pointer_lock(bind!(looking, |locked: bool| looking.set(locked)))
    ///     .on_pointer_move(bind!(looking, heading, |event: PointerEvent| {
    ///         if transaction(|ctx| looking.get(ctx)) {
    ///             heading.change(|heading| *heading += event.movement_x * 0.2);
    ///         }
    ///     }));
    ///
    /// dom! { <div>{view}</div> };
    /// ```
    ///
    /// [`PointerEvent::movement_x`]: crate::PointerEvent::movement_x
    /// [`movement_y`]: crate::PointerEvent::movement_y
    /// [`DomElement::hook_pointer_lock`]: crate::DomElement::hook_pointer_lock
    /// [`Driver::exit_pointer_lock`]: crate::Driver::exit_pointer_lock
    pub fn request_pointer_lock(&self) {
        api_browser_command().pointer_lock(Some(self.id.to_u64()));
    }

    /// Shows this element on the whole screen, f. ex. a game's canvas with what is over it. The
    /// browser does it only right after a click or a key pressed on the page, so this goes in
    /// their listener, and leaves it on Esc. [`DomElement::hook_fullscreen`] tells when the
    /// element is on the whole screen and no longer, and [`Driver::exit_fullscreen`] leaves it.
    /// The browser's own fullscreen (F11) shows the page, not an element, and isn't told.
    ///
    /// ```rust
    /// use vertigo::{ClickEvent, Value, bind, dom, dom_element, get_driver, transaction};
    ///
    /// let whole = Value::new(false);
    ///
    /// let game = dom_element! { <div>"The game"</div> };
    /// let game_ref = game.get_ref();
    /// let game = game.hook_fullscreen(bind!(whole, |on: bool| whole.set(on)));
    /// let toggle = bind!(whole, |_: ClickEvent| {
    ///     if transaction(|ctx| whole.get(ctx)) {
    ///         get_driver().exit_fullscreen();
    ///     } else {
    ///         game_ref.request_fullscreen();
    ///     }
    /// });
    ///
    /// dom! {
    ///     <div>
    ///         {game}
    ///         <button on_click={toggle}>"Whole screen"</button>
    ///     </div>
    /// };
    /// ```
    ///
    /// [`DomElement::hook_fullscreen`]: crate::DomElement::hook_fullscreen
    /// [`Driver::exit_fullscreen`]: crate::Driver::exit_fullscreen
    pub fn request_fullscreen(&self) {
        api_browser_command().fullscreen(Some(self.id.to_u64()));
    }
}

impl PartialEq for DomElementRef {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
