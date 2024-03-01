use super::{get_selector::get_selector, next_id::NextId};

mod splits;
use splits::{css_row_split_to_pair, css_split_rows, find_brackets};

#[cfg(test)]
mod tests;

pub fn transform_css_animation_value(
    css: &str,
    next_id: &NextId,
) -> (String, Option<(String, String)>) {
    let brackets = find_brackets(css);

    if let Some((start_word, central_word, end_word)) = brackets {
        let id = next_id.get_next_id();
        let selector = get_selector(&id);

        let keyframe_name = ["@keyframes ", &selector].concat();
        let keyframe_content = central_word;

        let new_css = [start_word, &selector, end_word].join(" ");

        return (new_css, Some((keyframe_name, keyframe_content.into())));
    }

    (css.into(), None)
}

pub fn transform_css_selector_value(row: &str, parent_selector: &str) -> Option<(String, String)> {
    let brackets = find_brackets(row);

    if let Some((pseudo_selector, rules, trash)) = brackets {
        if !trash.trim().is_empty() {
            log::error!("Unexpected input after pseudo-selector rule set, missing semi-colon?");
        }
        let new_selector = [parent_selector, pseudo_selector].concat();
        return Some((new_selector, rules.into()));
    }

    None
}

pub fn transform_css_media_query(
    row: &str,
    parent_selector: &str,
    css_documents: &mut Vec<(String, String)>,
) {
    let brackets = find_brackets(row);

    if let Some((query, rules_input, trash)) = brackets {
        if !trash.trim().is_empty() {
            log::error!("Unexpected input after media query, missing semicolon?");
        }

        // Collected rules
        let mut regular_rules = vec![];
        // Collected sets of rules inside pseudo-selectors
        let mut rules_in_pseudo = vec![];

        for row in css_split_rows(rules_input) {
            if row.starts_with(':') {
                // Pseudo selectors inside media query, collect it into separate vector
                let extra_rule = transform_css_selector_value(row, parent_selector);

                if let Some(extra_rule) = extra_rule {
                    rules_in_pseudo.push([&extra_rule.0, " { ", &extra_rule.1, " }"].concat());
                }
            } else {
                // Regular rule in media query
                // TODO: Handle animation here
                regular_rules.push(row);
            }
        }

        if !regular_rules.is_empty() || !rules_in_pseudo.is_empty() {
            let mut media_content = "".to_string();
            // Insert regular rules first
            if !regular_rules.is_empty() {
                // selector { rules; }
                media_content.push_str(parent_selector);
                media_content.push_str(" { ");
                media_content.push_str(&regular_rules.join(";"));
                media_content.push_str(" }");
            }
            // Add sets of rules in pseudo selectors
            for set in &rules_in_pseudo {
                if !media_content.is_empty() {
                    media_content.push('\n');
                }
                media_content.push_str(set);
            }
            css_documents.push((query.into(), media_content));
        }
    }
}

pub fn transform_css(css: &str, next_id: &NextId) -> (u64, Vec<(String, String)>) {
    let class_id = next_id.get_next_id();
    let selector = [".", &get_selector(&class_id)].concat();

    let mut css_out: Vec<String> = Vec::new();
    let mut css_documents: Vec<(String, String)> = Vec::new();

    for row in css_split_rows(css) {
        if row.starts_with(':') {
            // It's a pseudo-selector
            let extra_rule = transform_css_selector_value(row, &selector);

            if let Some(extra_rule) = extra_rule {
                css_documents.push(extra_rule);
            }
        } else if row.starts_with("@media") {
            // It's a set of rules inside media query
            transform_css_media_query(row, &selector, &mut css_documents);
        } else {
            // Single rule
            match css_row_split_to_pair(row) {
                Some((name, value)) => {
                    let value_parsed = if name.trim() == "animation" {
                        // Animation rule
                        let (value_parsed, extra_animation) =
                            transform_css_animation_value(&value, next_id);

                        if let Some(extra_animation) = extra_animation {
                            css_documents.push(extra_animation);
                        }

                        value_parsed
                    } else {
                        // Regular rule
                        value
                    };

                    css_out.push([name, ": ", &value_parsed].concat());
                }
                None => {
                    css_out.push(row.into());
                }
            }
        }
    }

    let css_out: String = css_out.join("; ");

    css_documents.push((selector, css_out));

    (class_id, css_documents)
}
