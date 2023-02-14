use crate::dom::dom_comment_create::DomFragment;
use crate::dom::dom_node::DomNodeFragment;
use crate::struct_mut::{ValueMut};
use crate::{Computed, get_driver, DomComment, DomNode};

pub fn render_value_option<T: Clone + PartialEq + 'static, R: Into<DomNodeFragment>>(
    computed: Computed<T>,
    render: impl Fn(T) -> Option<R> + 'static
) -> DomFragment {
    let comment = DomComment::new("value element");
    let comment_id = comment.id_dom();

    DomFragment::new(comment_id, move |parent_id| {
        let driver = get_driver();

        driver.inner.dom.insert_before(parent_id, comment_id, None);
        let current_node: ValueMut<Option<DomNode>> = ValueMut::new(None);

        let client = computed.subscribe(move |value| {

            let new_element = render(value).map(|item| {
                let new_element: DomNodeFragment = item.into();
                driver.inner.dom.insert_before(parent_id, new_element.id(), Some(comment_id));
                new_element.convert_to_node(parent_id)
            });

            current_node.change(|current| {
                *current = new_element;
            });
        });

        comment.add_subscription(client);
        comment
    })
}

pub fn render_value<T: Clone + PartialEq + 'static, R: Into<DomNodeFragment>>(
    computed: Computed<T>,
    render: impl Fn(T) -> R + 'static
) -> DomFragment {
    render_value_option(computed, move |value| -> Option<R> {
        Some(render(value))
    })
}
