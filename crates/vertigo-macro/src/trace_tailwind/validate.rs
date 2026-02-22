use crate::trace_tailwind::paths::get_tailwind_classes_file_path;
use proc_macro_error::emit_error;
use std::error::Error;

pub(crate) fn validate_tailwind_classes(bundle: &str) -> Result<(), Box<dyn Error>> {
    let classes_path = get_tailwind_classes_file_path()?;

    if !classes_path.exists() {
        return Ok(()); // If the file does not exist, there are no classes to validate
    }

    let classes_content = std::fs::read_to_string(&classes_path)?;

    let mut missing = std::collections::BTreeSet::new();

    for line in classes_content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        for class in line.split_whitespace() {
            let class = class.trim();
            if class.is_empty() {
                continue;
            }
            if !contains_class(bundle, class) {
                missing.insert(class.to_string());
            }
        }
    }

    if !missing.is_empty() {
        let missing_list: Vec<&str> = missing.iter().map(|s| s.as_str()).collect();
        // Warn the developer that these classes were used but not resolved by Tailwind
        emit_error!(
            proc_macro::Span::call_site(),
            "The following Tailwind classes were used but not found in the generated CSS: {}",
            missing_list.join(", ")
        );
    }

    Ok(())
}

fn escape_css_name(name: &str) -> String {
    let mut escaped = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
            escaped.push(c);
        } else {
            escaped.push('\\');
            escaped.push(c);
        }
    }
    escaped
}

fn contains_class(bundle: &str, class: &str) -> bool {
    let escaped = escape_css_name(class);
    let selector = format!(".{}", escaped);

    let mut start = 0;

    while let Some(index) = bundle[start..].find(&selector) {
        let actual_index = start + index;
        let next_char_idx = actual_index + selector.len();
        if next_char_idx < bundle.len() {
            let next_char = bundle[next_char_idx..].chars().next().unwrap_or(' ');
            // CSS selector should end with pseudo class (:), space, comma, brace, combinator, etc.
            if !next_char.is_ascii_alphanumeric() && next_char != '-' && next_char != '_' {
                return true;
            }
        } else {
            return true;
        }
        start = actual_index + selector.len();
    }

    // Also check for attribute selectors logic or complex variants if needed, but above covers 99%
    false
}
