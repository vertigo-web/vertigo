use crate::{CssGroup, Css};

// Get n'th css group as static
pub(super) fn get_ss(css: &Css, idx: usize) -> &'static str {
    match css.groups[idx] {
        CssGroup::CssStatic { value } => {
            value
        },
        _ => panic!("Expected CssStatic")
    }
}

// Get first css group as static
pub(super) fn get_s(css: &Css) -> &'static str {
    get_ss(css, 0)
}

// Get n'th css group as dynamic
pub(super) fn get_dd(css: &'_ Css, idx: usize) -> &'_ String {
    match &css.groups[idx] {
        CssGroup::CssDynamic { value } => {
            value
        },
        _ => panic!("Expected CssDynamic")
    }
}

// Get first css group as dynamic
pub(super) fn get_d(css: &'_ Css) -> &'_ String {
    get_dd(css, 0)
}
