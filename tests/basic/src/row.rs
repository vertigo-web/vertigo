use vertigo::{component, dom};

#[component]
pub fn Row<'a>(id: &'a str, label: &'a str) {
    dom! {
        <div id={id}>{label}</div>
    }
}
