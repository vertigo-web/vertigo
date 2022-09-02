use crate::dom::dom_comment_create::DomCommentCreate;
use crate::dom::dom_node::DomNodeFragment;
use crate::struct_mut::{ValueMut};
use crate::{Computed, get_driver, DomComment, DomNode};

pub fn render_value_option<T: Clone + PartialEq + 'static, R: Into<DomNodeFragment>>(
    computed: Computed<T>,
    render: impl Fn(T) -> Option<R> + 'static
) -> DomCommentCreate {
    DomCommentCreate::new(move |parent_id| {
        let driver = get_driver();

        let comment = DomComment::new("value element");
        let comment_id = comment.id_dom();

        driver.insert_before(parent_id, comment_id, None);
        let current_node: ValueMut<Option<DomNode>> = ValueMut::new(None);

        let client = computed.subscribe(move |value| {
            let new_element = render(value).map(|item| item.into().convert_to_node(parent_id));

            current_node.change(|current| {
                if let Some(new_element) = &new_element {
                    driver.insert_before(parent_id, new_element.id_dom(), Some(comment_id));
                }

                let _ = std::mem::replace(current, new_element);
            });
        });

        comment.add_subscription(client);
        comment
    })
}

pub fn render_value<T: Clone + PartialEq + 'static, R: Into<DomNodeFragment>>(
    computed: Computed<T>,
    render: impl Fn(T) -> R + 'static
) -> DomCommentCreate {
    render_value_option(computed, move |value| -> Option<R> {
        Some(render(value))
    })
}
