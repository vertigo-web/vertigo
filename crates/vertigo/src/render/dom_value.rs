use std::rc::Rc;

use crate::{get_driver, struct_mut::ValueMut, Computed, DomComment, DomNode};

pub fn render_value_option<T: Clone + PartialEq + 'static>(
    computed: Computed<T>,
    render: impl Fn(T) -> Option<DomNode> + 'static,
) -> DomNode {
    let render = Rc::new(render);

    DomComment::new_marker("value element", move |parent_id, comment_id| {
        let current_node: ValueMut<Option<DomNode>> = ValueMut::new(None);

        Some(computed.clone().subscribe({
            let render = render.clone();

            move |value| {
                let new_element = render(value).inspect(|new_element| {
                    get_driver().inner.dom.insert_before(
                        parent_id,
                        new_element.id_dom(),
                        Some(comment_id),
                    );
                });

                current_node.change(|current| {
                    *current = new_element;
                });
            }
        }))
    })
    .into()
}

pub fn render_value<T: Clone + PartialEq + 'static>(
    computed: Computed<T>,
    render: impl Fn(T) -> DomNode + 'static,
) -> DomNode {
    render_value_option(computed, move |value| -> Option<DomNode> {
        Some(render(value))
    })
}
