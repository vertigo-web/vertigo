use super::{get_selector::get_selector, next_id::NextId};

#[cfg(test)]
mod tests;

fn css_row_split_to_pair(row: &str) -> Option<(&str, String)> {
    let chunks: Vec<&str> = row.split(":").collect();

    if chunks.len() < 2 {
        return None;
    }

    let key = chunks[0].trim();
    let attr: String = chunks[1..].join(":").trim().into();

    Some((key, attr))
}

#[cfg(test)]
pub fn css_split_rows_pair(css: &str) -> Vec<(&str, String)> {
    let mut out = Vec::new();

    for row in css_split_rows(css) {
        let pair = css_row_split_to_pair(row);

        if let Some(pair) = pair {
            out.push(pair);
        } else {
            log::error!("Problem with parsing this css line: {}", row);
        }
    }

    out
}

#[derive(Eq, PartialEq, Debug)]
enum ParsingRowState {
    Left,
    Right,
    RightBracketOpen(u16),
}

pub fn css_split_rows(css: &str) -> Vec<&str> {
    let mut out: Vec<&str> = Vec::new();

    let mut state = ParsingRowState::Left;
    let mut start = 0;

    for (index, char) in css.char_indices() {

        if char == '{' {
            if state == ParsingRowState::Right {
                state = ParsingRowState::RightBracketOpen(1);
            } else if let ParsingRowState::RightBracketOpen(counter) = &mut state {
                *counter += 1;
            } else {
                panic!("unsupported use case");
            }
        }

        if char == '}' {
            let should_clouse = if let ParsingRowState::RightBracketOpen(counter) = &mut state {
                if *counter > 1 {
                    *counter -= 1;
                    false
                } else {
                    true
                }
            } else {
                panic!("unsupported use case");
            };

            if should_clouse {
                state = ParsingRowState::Right;
            }

        }

        if char == ':' {
            if state == ParsingRowState::Left {
                state = ParsingRowState::Right;
            } else if let ParsingRowState::RightBracketOpen(..) = state {
                //ignore
            } else {
                log::error!("css {:?}", css);
                log::error!("stan {:?}", state);
                panic!("unsupported use case");
            }
        }

        if char == ';' {
            if state == ParsingRowState::Right {
                out.push(&css[start..index].trim());
                start = index + 1;
                state = ParsingRowState::Left;
            }
        }
    }

    out.push(&css[start..css.len()].trim());

    out.into_iter().filter(|item|{
        item.trim() != ""
    }).collect()
}

pub fn find_brackets(line: &str) -> Option<(&str, &str, &str)> {
    let mut start: Option<usize> = None;
    let mut end: Option<usize> = None;

    let chars: Vec<char> = line.chars().collect();

    for (index, char) in chars.iter().enumerate() {
        if *char == '{' {
            start = Some(index);
            break;
        }
    }

    for (index, char) in chars.iter().enumerate().rev() {
        if *char == '}' {
            end = Some(index);
            break;
        }
    }

    if let (Some(start), Some(end)) = (start, end) {
        let start_word = &line[0..start];
        let central_word = &line[start+1..end];
        let end_word = &line[end+1..];
        return Some((start_word.trim(), central_word.trim(), end_word.trim()));
    }

    None
}


#[test]
fn test_find_brackets() {
    let css = "";
    assert_eq!(find_brackets(css), None);
    let css = "1.0s infinite ease-in-out";
    assert_eq!(find_brackets(css), None);
    let css = "1.0s infinite ease-in-out { dsd }";
    assert_eq!(find_brackets(css), Some(("1.0s infinite ease-in-out", "dsd", "")));
    let css = "1.0s infinite ease-in-out { dsd } fff";
    assert_eq!(find_brackets(css), Some(("1.0s infinite ease-in-out", "dsd", "fff")));
}

pub fn transform_css_animation_value(css: &str, next_id: &mut NextId) -> (String, Option<(String, String)>) {

    let brackets = find_brackets(css);

    if let Some((start_word, central_word, end_word)) = brackets {
        let id = next_id.get_next_id();
        let selector = get_selector(&id);

        let keyframe_name = format!("@keyframes {}", selector);
        let keyframe_content = central_word;

        let new_css = format!("{} {} {}", start_word, selector, end_word);

        return (new_css, Some((keyframe_name, keyframe_content.into())));
    }

    return (css.into(), None);
}

pub fn transform_css(css: &str, next_id: &mut NextId) -> (u64, Vec<(String, String)>) {
    let class_id = next_id.get_next_id();
    let selector = format!(".{}", get_selector(&class_id));

    let mut css_out: Vec<String> = Vec::new();
    let mut css_documents: Vec<(String, String)> = Vec::new();

    for row in css_split_rows(css) {
        match css_row_split_to_pair(row) {
            Some((name, value)) => {
                let value_parsed = if name.trim() == "animation" {
                    let (value_parsed, extra_animation) = transform_css_animation_value(&value, next_id);

                    if let Some(extra_animation) = extra_animation {
                        css_documents.push(extra_animation);
                    }

                    value_parsed
                } else {
                    value
                };
                
                css_out.push(format!("{}: {}", name, value_parsed));
            },
            None => {
                css_out.push(row.into());
            }
        }
    }

    let css_out: String = css_out.join("; ");

    css_documents.push((selector, css_out));

    (class_id, css_documents)
}
