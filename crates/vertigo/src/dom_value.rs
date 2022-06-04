use crate::struct_mut::{ValueMut};
use crate::{Computed, get_driver, DomComment, DomNode};

pub fn render_value_option<T: Clone + PartialEq + 'static, R: Into<DomNode>>(
    computed: Computed<T>,
    render: impl Fn(T) -> Option<R> + 'static
) -> DomComment {
    let driver = get_driver();

    let comment = DomComment::new("value element");
    let comment_id = comment.id_dom();

    comment.set_on_mount(move |parent_id| {
        driver.insert_before(parent_id, comment_id, None);
        let current_node: ValueMut<Option<DomNode>> = ValueMut::new(None);

        computed.subscribe(move |value| {
            let new_element = render(value).map(|item| item.into());

            current_node.change(|current| {
                if let Some(new_element) = &new_element {
                    driver.insert_before(parent_id, new_element.id_dom(), Some(comment_id));
                }

                let _ = std::mem::replace(current, new_element);
            });
        })
    })
}

pub fn render_value<T: Clone + PartialEq + 'static, R: Into<DomNode>,>(
    computed: Computed<T>,
    render: impl Fn(T) -> R + 'static
) -> DomComment {
    render_value_option(computed, move |value| -> Option<R> {
        Some(render(value))
    })
}
