mod inline;

pub use inline::Inline;

// Proc-macros can't be defined in the same crate, so all we can do is re-export it here from separate "sub-crate"
pub use vertigo_html_macro::{html_component, html_element};

// For convenience
pub use vertigo::node_attr::NodeAttr;

#[cfg(test)]
mod tests;
