use vertigo::{dom, component};

#[component]
pub fn Row<'a>(id: String, label: String) {
    dom! {
        <div id={id}>{label}</div>
    }
}
