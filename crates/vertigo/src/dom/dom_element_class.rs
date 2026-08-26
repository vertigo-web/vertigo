use std::rc::Rc;

use crate::{
    Css, DomId, css::get_css_manager, dom::attr_value::AttrText, driver_module::get_driver_dom,
    struct_mut::ValueMut,
};

/// The extra state an element needs *once it has css*, and not before.
///
/// Behind a `Box` because most elements never get here: a `class` attribute on its own needs
/// nothing but the two words in [`ClassMergeInner`], and this is the debug hint and cached
/// class name that they would otherwise all carry.
struct CssSource {
    /// What the css resolves to. Cached because `get_class_name` allocates a `String` and
    /// walks the css registry, and without this it would run again every time the *class
    /// attribute* changed - which has nothing to do with it. The registry memoises, so the
    /// answer for a given `Css` does not move.
    resolved: String,
    /// The `v-css` hint. `None` in release builds - the macro only emits one under
    /// `cfg(test)`, which is also the only place it is ever asserted.
    hint: Option<String>,
    hint_sent: Option<String>,
}

/// Merges what the `class` attribute says with what `css` resolves to, and writes the result
/// out once.
///
/// Two sources, one attribute. Keeping the last value written is what stops a repeat reaching
/// the browser.
struct ClassMergeInner {
    id_dom: DomId,
    attr: Option<AttrText>,
    css: Option<Box<CssSource>>,
    last_sent: Option<AttrText>,
}

impl ClassMergeInner {
    /// `already_sent` is the class the element wrote before it needed a merger at all - see
    /// [`ClassState`]. Recording it as both the current attribute and the last thing written
    /// means promotion emits nothing by itself.
    fn new(id_dom: DomId, already_sent: Option<AttrText>) -> Self {
        Self {
            id_dom,
            attr: already_sent.clone(),
            css: None,
            last_sent: already_sent,
        }
    }

    /// What the `class` attribute should say, or `None` when neither source has anything.
    ///
    /// An *empty* class attribute is `Some("")` and not `None`: the two differ in whether
    /// anything is written at all.
    fn merged(&self) -> Option<AttrText> {
        match (&self.attr, &self.css) {
            (None, None) => None,
            // The overwhelmingly common case, and the reason this returns `AttrText` rather
            // than building a `String`: with no css to merge in, the attribute *is* the
            // answer, and cloning it copies a pointer.
            (Some(attr), None) => Some(attr.clone()),
            (None, Some(css)) => Some(AttrText::from(css.resolved.clone())),
            (Some(attr), Some(css)) => Some(AttrText::from(format!(
                "{} {}",
                attr.as_str(),
                css.resolved
            ))),
        }
    }

    fn refresh_dom(&mut self) {
        let merged = self.merged();

        if self.last_sent != merged {
            let value = merged.as_ref().map(AttrText::as_str).unwrap_or("");
            get_driver_dom().set_attr(self.id_dom, "class", value);
            self.last_sent = merged;
        }

        // Tracked separately from the class rather than compared alongside it, so that a
        // class change does not resend a hint that has not moved.
        let id_dom = self.id_dom;
        if let Some(css) = &mut self.css
            && let Some(hint) = css.hint.clone()
            && css.hint_sent.as_ref() != Some(&hint)
        {
            get_driver_dom().set_attr(id_dom, "v-css", &hint);
            css.hint_sent = Some(hint);
        }
    }
}

/// Handle on an element's `class`, shared with whatever subscriptions write to it.
#[derive(Clone)]
pub struct DomElementClassMerge {
    inner: Rc<ValueMut<ClassMergeInner>>,
}

impl DomElementClassMerge {
    fn new(id_dom: DomId, already_sent: Option<AttrText>) -> Self {
        Self {
            inner: Rc::new(ValueMut::new(ClassMergeInner::new(id_dom, already_sent))),
        }
    }

    pub fn set_attribute(&self, new_value: AttrText) {
        self.inner.change(|state| {
            state.attr = Some(new_value);
            state.refresh_dom();
        });
    }

    pub fn remove_attribute(&self) {
        self.inner.change(|state| {
            state.attr = None;
            state.refresh_dom();
        });
    }

    pub fn set_css(&self, new_value: Css, debug_class_name: Option<String>) {
        self.inner.change(|state| {
            let resolved = get_css_manager().get_class_name(&new_value);
            let hint_sent = state.css.as_mut().and_then(|css| css.hint_sent.take());

            state.css = Some(Box::new(CssSource {
                resolved,
                hint: debug_class_name,
                hint_sent,
            }));
            state.refresh_dom();
        });
    }
}

/// What an element knows about its own `class`, and how much it had to allocate to know it.
///
/// The merger exists to reconcile *two* sources - the `class` attribute and `css={..}` - and
/// most elements never have both. `class="a b c"` is a single value, and `tw=` is folded into
/// it by `AttrValue::combine` before the element sees it, so the number of class names is
/// beside the point: what costs an allocation is a second source.
///
/// So a static class writes itself straight out and keeps the value inline, and only a `css`
/// or a *reactive* class - which needs a cell its subscription can write through - pays for
/// a merger. In a table row that is one element of eight rather than six.
pub enum ClassState {
    Empty,
    /// A class attribute, no css. Also the value last written, which is what a later `css`
    /// has to be merged with.
    Plain(AttrText),
    Merged(DomElementClassMerge),
}

impl ClassState {
    pub fn set_attribute(&mut self, id_dom: DomId, value: AttrText) {
        match self {
            ClassState::Merged(merge) => merge.set_attribute(value),
            ClassState::Plain(sent) if *sent == value => {}
            ClassState::Empty | ClassState::Plain(_) => {
                get_driver_dom().set_attr(id_dom, "class", value.as_str());
                *self = ClassState::Plain(value);
            }
        }
    }

    // There is no `remove_attribute` here on purpose: a class only *goes away* when it comes
    // from an optional reactive value, and those own a merger from the moment the element is
    // built. `Plain` is only ever reached from a value that is always present.

    /// The shared merger, building it if this is the first time two sources meet.
    ///
    /// A class already written is carried across as *already sent*, so promoting on its own
    /// emits nothing - the next `set_css` is what produces the merged value.
    pub fn merger(&mut self, id_dom: DomId) -> DomElementClassMerge {
        if let ClassState::Merged(merge) = self {
            return merge.clone();
        }

        let already_sent = match std::mem::replace(self, ClassState::Empty) {
            ClassState::Plain(sent) => Some(sent),
            _ => None,
        };

        let merge = DomElementClassMerge::new(id_dom, already_sent);
        *self = ClassState::Merged(merge.clone());
        merge
    }
}
