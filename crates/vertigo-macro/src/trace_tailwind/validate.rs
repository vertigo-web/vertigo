use crate::trace_tailwind::paths::get_tailwind_classes_file_path;
use proc_macro_error::emit_error;
use regex::Regex;
use std::collections::BTreeSet;
use std::error::Error;

pub(crate) fn validate_tailwind_classes(bundle: &str) -> Result<(), Box<dyn Error>> {
    let classes_path = get_tailwind_classes_file_path()?;

    if !classes_path.exists() {
        return Ok(()); // If the file does not exist, there are no classes to validate
    }

    let defined_vars = extract_defined_variables(bundle);
    let used_classes = get_all_used_classes(&classes_path)?;

    let (missing_classes, missing_vars) =
        find_missing_elements(bundle, &defined_vars, used_classes);

    emit_validation_errors(missing_classes, missing_vars);

    Ok(())
}

fn get_all_used_classes(
    classes_path: &std::path::Path,
) -> Result<BTreeSet<String>, Box<dyn Error>> {
    let classes_content = std::fs::read_to_string(classes_path)?;
    let mut used_classes = BTreeSet::new();

    for line in classes_content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        for class in line.split_whitespace() {
            let class = class.trim();
            if !class.is_empty() {
                used_classes.insert(class.to_string());
            }
        }
    }

    Ok(used_classes)
}

fn find_missing_elements(
    bundle: &str,
    defined_vars: &BTreeSet<String>,
    used_classes: BTreeSet<String>,
) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut missing_classes = BTreeSet::new();
    let mut missing_vars = BTreeSet::new();

    for class in used_classes {
        // Validate class existence - only if it's not a raw var usage
        if !class.starts_with("var(") && !contains_class(bundle, &class) {
            missing_classes.insert(class.to_string());
        }

        // Validate CSS variables
        for var in extract_used_variables(&class) {
            if !defined_vars.contains(&var) {
                missing_vars.insert(var);
            }
        }
    }

    (missing_classes, missing_vars)
}

fn emit_validation_errors(missing_classes: BTreeSet<String>, missing_vars: BTreeSet<String>) {
    if !missing_classes.is_empty() {
        let missing_list: Vec<&str> = missing_classes.iter().map(|s| s.as_str()).collect();
        emit_error!(
            proc_macro::Span::call_site(),
            "The following Tailwind classes were used but not found in the generated CSS: {}",
            missing_list.join(", ")
        );
    }

    if !missing_vars.is_empty() {
        let missing_list: Vec<&str> = missing_vars.iter().map(|s| s.as_str()).collect();
        emit_error!(
            proc_macro::Span::call_site(),
            "The following CSS variables were used but are not defined in the CSS bundle: {}",
            missing_list.join(", ")
        );
    }
}

fn extract_defined_variables(bundle: &str) -> BTreeSet<String> {
    let mut vars = BTreeSet::new();
    // Matches --var-name:
    let re = Regex::new(r"(?m)^\s+(--[a-zA-Z0-9_-]+):").unwrap();
    for cap in re.captures_iter(bundle) {
        vars.insert(cap[1].to_string());
    }
    vars
}

pub(crate) fn extract_used_variables(class: &str) -> Vec<String> {
    let mut vars = Vec::new();
    // Matches var(--var-name)
    let re = Regex::new(r"var\((--[a-zA-Z0-9_-]+)\)").unwrap();
    for cap in re.captures_iter(class) {
        vars.push(cap[1].to_string());
    }
    vars
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_defined_variables() {
        let bundle = r#"
            :root {
                --token-text-text_dark: #333;
                --bg-primary: #fff;
            }
            .some-class {
                color: var(--token-text-text_dark);
            }
        "#;
        let vars = extract_defined_variables(bundle);
        assert!(vars.contains("--token-text-text_dark"));
        assert!(vars.contains("--bg-primary"));
        assert_eq!(vars.len(), 2);
    }

    #[test]
    fn test_extract_used_variables() {
        let class = "text-[var(--token-text-text_dark)]";
        let vars = extract_used_variables(class);
        assert_eq!(vars, vec!["--token-text-text_dark"]);

        let class_multiple = "bg-[var(--bg)] shadow-[0_0_10px_var(--shadow)]";
        let vars_multiple = extract_used_variables(class_multiple);
        assert_eq!(vars_multiple, vec!["--bg", "--shadow"]);
    }

    #[test]
    fn test_validation_logic() {
        let bundle = r#"
            :root {
                --menu-active: #000;
                --menu-inactivef: #fff;
            }
        "#;
        let defined_vars = extract_defined_variables(bundle);
        let mut used_classes = BTreeSet::new();
        used_classes.insert("bg-[var(--menu-inactive)]".to_string());

        let (_, missing_vars) = find_missing_elements(bundle, &defined_vars, used_classes);
        assert!(missing_vars.contains("--menu-inactive"));
    }
}
