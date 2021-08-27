use vertigo::{VDomNode, VDomElement, VDomText, VDomComponent, Css, CssGroup};

#[derive(Debug, PartialEq)]
pub enum EqResult {
    Equal,
    NotEqual(String),
}

pub fn eq_nodes(node1: &VDomNode, node2: &VDomNode) -> EqResult {
    match (node1, node2) {
        (VDomNode::Element { node: el1 }, VDomNode::Element { node: el2 }) => eq_els(el1, el2),
        (VDomNode::Text { node: txt1 }, VDomNode::Text { node: txt2 }) => eq_txts(txt1, txt2),
        (VDomNode::Component { node: cmp1 }, VDomNode::Component { node: cmp2 }) => eq_cmps(cmp1, cmp2),
        (_, _) => EqResult::NotEqual("Type mismatch".to_string())
    }
}

pub fn eq_els(el1: &VDomElement, el2: &VDomElement) -> EqResult {
    if el1.name != el2.name {
        return EqResult::NotEqual(format!("{} != {}", el1.name, el2.name))
    }

    if el1.attr != el2.attr {
        return EqResult::NotEqual(format!("{} and {} have different attrs", el1.name, el2.name))
    }

    if el1.on_click.is_some() != el2.on_click.is_some() {
        return EqResult::NotEqual(format!("Out of {} and {}, one has on_click and one not", el1.name, el2.name))
    }

    if el1.on_input.is_some() != el2.on_input.is_some() {
        return EqResult::NotEqual(format!("Out of {} and {}, one has on_input and one not", el1.name, el2.name))
    }

    if el1.on_input.is_some() != el2.on_input.is_some() {
        return EqResult::NotEqual(format!("Out of {} and {}, one has on_input and one not", el1.name, el2.name))
    }

    if el1.on_mouse_enter.is_some() != el2.on_mouse_enter.is_some() {
        return EqResult::NotEqual(format!("Out of {} and {}, one has on_mouse_enter and one not", el1.name, el2.name))
    }

    if el1.on_mouse_leave.is_some() != el2.on_mouse_leave.is_some() {
        return EqResult::NotEqual(format!("Out of {} and {}, one has on_mouse_leave and one not", el1.name, el2.name))
    }

    if el1.on_key_down.is_some() != el2.on_key_down.is_some() {
        return EqResult::NotEqual(format!("Out of {} and {}, one has on_key_down and one not", el1.name, el2.name))
    }

    if el1.css.is_some() != el2.css.is_some() {
        return EqResult::NotEqual(format!("Out of {} and {}, one has css and one not", el1.name, el2.name))
    }

    if let (Some(css1), Some(css2)) = (el1.css.as_ref(), el2.css.as_ref()) {
        if let EqResult::NotEqual(reason) = eq_css(css1, css2) {
            return EqResult::NotEqual(reason)
        }
    }

    if let EqResult::NotEqual(reason) = eq_nodes_vec(&el1.children, &el2.children) {
        return EqResult::NotEqual(reason)
    }

    EqResult::Equal
}

pub fn eq_txts(txt1: &VDomText, txt2: &VDomText) -> EqResult {
    if txt1.value != txt2.value {
        return EqResult::NotEqual(format!("Text not equal: {} != {}", txt1.value, txt2.value))
    }

    EqResult::Equal
}

pub fn eq_cmps(cmp1: &VDomComponent, cmp2: &VDomComponent) -> EqResult {
    // if cmp1.id != cmp2.id

    eq_els(&cmp1.view.get_value(), &cmp2.view.get_value())
}

pub fn eq_nodes_vec(vec1: &[VDomNode], vec2: &[VDomNode]) -> EqResult {
    if vec1.len() != vec2.len() {
        return EqResult::NotEqual(format!("Nodes vectors lengths not equal: {} != {}", vec1.len(), vec2.len()))
    }
    let finding = vec1.iter()
        .zip(vec2.iter())
        .enumerate()
        .find_map(|(x, (node1, node2))| {
            match eq_nodes(node1, node2) {
                EqResult::Equal => None,
                EqResult::NotEqual(reason) => Some((x, reason))
            }
        });
    if let Some((x, reason)) = finding {
        return EqResult::NotEqual(format!("Nodes vectors not equal in position {}: {}", x, reason))
    }
    EqResult::Equal
}

pub fn eq_css(css1: &Css, css2: &Css) -> EqResult {
    let finding = css1.groups.iter()
        .zip(css2.groups.iter())
        .enumerate()
        .find_map(|(x, (css_group1, css_group2))|
            match eq_css_groups(css_group1, css_group2) {
                EqResult::Equal => None,
                EqResult::NotEqual(reason) => Some((x, reason))
            }
        );
    if let Some((x, reason)) = finding {
        return EqResult::NotEqual(format!("Css groups vector not equal in position {}: {}", x, reason))
    }
    EqResult::Equal
}

pub fn eq_css_groups(css_group1: &CssGroup, css_group2: &CssGroup) -> EqResult {
    match (css_group1, css_group2) {
        (CssGroup::CssStatic { value: val1 }, CssGroup::CssStatic { value: val2 }) =>
            if val1 != val2 {
                EqResult::NotEqual("Css static groups not equal".to_string())
            } else {
                EqResult::Equal
            }
        (CssGroup::CssDynamic { value: val1 }, CssGroup::CssDynamic { value: val2 }) =>
            if val1 != val2 {
                EqResult::NotEqual("Css dynamic groups not equal".to_string())
            } else {
                EqResult::Equal
            }
        (_, _) => EqResult::NotEqual("Css groups type mismatch".to_string()),
    }
}

#[cfg(test)]
mod tests;
