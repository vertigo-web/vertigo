#[derive(Eq, PartialEq, Debug)]
enum RowSideState {
    Left,
    Right,
    BracketsOpened(u16),
}

// Splits rules into Vec
//
// "rule1; rule2;" -> ["rule1", "rule2"]
//
// "rule1 { subrule1; subrule2; }; rule2 { subrules3; subrule4; }" -> ["rule1 { subrule1; subrule2; }", "rule2 { ... }"]
//
// "rule { subrule1 { sub-subrule1; sub-subrule2 } }" -> ["rule { subrule1 { sub-subrule1; sub-subrule2 } }"]
//
pub fn css_split_rows(css: &str) -> Vec<&str> {
    let mut out: Vec<&str> = Vec::new();

    let mut row_side = RowSideState::Left;
    let mut start = 0;

    for (index, char) in css.char_indices() {
        if char == '{' {
            if row_side == RowSideState::Right {
                row_side = RowSideState::BracketsOpened(1);
            } else if let RowSideState::BracketsOpened(counter) = &mut row_side {
                *counter += 1;
            } else {
                log::info!("css input: {css}");
                panic!("unsupported use case at opening bracket");
            }
        }

        if char == '}' {
            let should_close = if let RowSideState::BracketsOpened(counter) = &mut row_side {
                if *counter > 1 {
                    *counter -= 1;
                    false
                } else {
                    true
                }
            } else {
                log::info!("css input: {css}");
                panic!("unsupported use case at closing bracket");
            };

            if should_close {
                row_side = RowSideState::Right;
            }
        }

        if (char == ':' || char == '.') && row_side == RowSideState::Left {
            row_side = RowSideState::Right;
        }

        if char == ';' && row_side == RowSideState::Right {
            out.push(css[start..index].trim());
            start = index + 1;
            row_side = RowSideState::Left;
        }
    }

    out.push(css[start..css.len()].trim());

    out.into_iter().filter(|item| item.trim() != "").collect()
}

// Split rule into key and value
pub fn css_row_split_to_pair(row: &str) -> Option<(&str, String)> {
    let chunks: Vec<&str> = row.split(':').collect();

    if chunks.len() < 2 {
        return None;
    }

    let key = chunks[0].trim();
    let attr: String = chunks[1..].join(":").trim().into();

    Some((key, attr))
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
        let central_word = &line[start + 1..end];
        let end_word = &line[end + 1..];
        return Some((start_word.trim(), central_word.trim(), end_word.trim()));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{css_row_split_to_pair, css_split_rows, find_brackets};

    #[test]
    fn test_css_split_rows_simple() {
        let css = "border-right: 3px solid #666;
            border-bottom: 4px solid #444;";

        let rows = css_split_rows(css);

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], "border-right: 3px solid #666");
        assert_eq!(rows[1], "border-bottom: 4px solid #444");
    }

    #[test]
    fn test_css_split_rows_hover() {
        let css = "transition: all .2s ease-in-out;

        :hover {
            transform: scale(1.2);
            box-shadow: 54px 54px 14px rgba(0, 0, 0, 0.3), 58px 58px 14px rgba(0, 0, 0, 0.2), 62px 62px 14px rgba(0, 0, 0, 0.1);
        };
        ";

        let rows = css_split_rows(css);

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], "transition: all .2s ease-in-out");

        assert_eq!(rows[1], ":hover {
            transform: scale(1.2);
            box-shadow: 54px 54px 14px rgba(0, 0, 0, 0.3), 58px 58px 14px rgba(0, 0, 0, 0.2), 62px 62px 14px rgba(0, 0, 0, 0.1);
        }");
    }

    #[test]
    fn test_css_split_rows_media_query_1() {
        let css = ":hover {
            transform: scale(1.2);
        };

        @media screen and (min-width: 600px) {
            :hover {
                transform: scale(1.5);
            }
        };
        ";

        let rows = css_split_rows(css);

        assert_eq!(rows.len(), 2);

        assert_eq!(
            rows[0],
            ":hover {
            transform: scale(1.2);
        }"
        );

        assert_eq!(
            rows[1],
            "@media screen and (min-width: 600px) {
            :hover {
                transform: scale(1.5);
            }
        }"
        );
    }

    #[test]
    fn test_css_split_rows_media_query_2() {
        let css = "color: black;

        :hover {
            color: white;
        };

        @media screen and (min-width: 600px) {
            color: red;

            :hover {
                green: green;
            }
        };

        @media screen and (min-width: 1200px) {
            color: blue;

            :hover {
                green: cyan;
            }
        };
        ";

        let rows = css_split_rows(css);

        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0], "color: black");

        assert_eq!(
            rows[1],
            ":hover {
            color: white;
        }"
        );

        assert_eq!(
            rows[2],
            "@media screen and (min-width: 600px) {
            color: red;

            :hover {
                green: green;
            }
        }"
        );

        assert_eq!(
            rows[3],
            "@media screen and (min-width: 1200px) {
            color: blue;

            :hover {
                green: cyan;
            }
        }"
        );
    }

    fn css_split_rows_pair(css: &str) -> Vec<(&str, String)> {
        let mut out = Vec::new();

        for row in css_split_rows(css) {
            let pair = css_row_split_to_pair(row);

            if let Some(pair) = pair {
                out.push(pair);
            } else {
                log::error!("Problem with parsing this css line: {row}");
            }
        }

        out
    }

    #[test]
    fn test_css_split_rows1() {
        let css = "cursor: pointer;";

        assert_eq!(css_split_rows(css), vec!("cursor: pointer"));

        let rows_pairs = css_split_rows_pair(css);

        assert_eq!(rows_pairs, vec!(("cursor", "pointer".into()),));
    }

    #[test]
    fn test_css_split_rows2() {
        let css = "border: 1px solid black; padding: 10px; background-color: #e0e0e0; margin-bottom: 10px ; ";

        assert_eq!(
            css_split_rows(css),
            vec!(
                "border: 1px solid black",
                "padding: 10px",
                "background-color: #e0e0e0",
                "margin-bottom: 10px"
            )
        );

        let rows_pairs = css_split_rows_pair(css);
        assert_eq!(
            rows_pairs,
            vec!(
                ("border", "1px solid black".into()),
                ("padding", "10px".into()),
                ("background-color", "#e0e0e0".into()),
                ("margin-bottom", "10px".into())
            )
        )
    }

    #[test]
    fn test_css_split_rows3() {
        let css = "border: 1px solid black; padding: 10px; background-color: #e0e0e0; margin-bottom: 10px   ";

        let rows = css_split_rows(css);
        assert_eq!(
            &rows,
            &vec!(
                "border: 1px solid black",
                "padding: 10px",
                "background-color: #e0e0e0",
                "margin-bottom: 10px"
            )
        );

        let rows_pairs = css_split_rows_pair(css);
        assert_eq!(
            rows_pairs,
            vec!(
                ("border", "1px solid black".into()),
                ("padding", "10px".into()),
                ("background-color", "#e0e0e0".into()),
                ("margin-bottom", "10px".into())
            )
        )
    }

    #[test]
    fn test_find_brackets() {
        let css = "";
        assert_eq!(find_brackets(css), None);
        let css = "1.0s infinite ease-in-out";
        assert_eq!(find_brackets(css), None);
        let css = "1.0s infinite ease-in-out { dsd }";
        assert_eq!(
            find_brackets(css),
            Some(("1.0s infinite ease-in-out", "dsd", ""))
        );
        let css = "1.0s infinite ease-in-out { dsd } fff";
        assert_eq!(
            find_brackets(css),
            Some(("1.0s infinite ease-in-out", "dsd", "fff"))
        );

        let css = "@media screen { color: white }";
        assert_eq!(
            find_brackets(css),
            Some(("@media screen", "color: white", ""))
        );
    }

    #[test]
    fn test_square_brackets() {
        let css1 = ".autocss_1 { color: white; }";
        assert_eq!(css_split_rows(css1), vec![".autocss_1 { color: white; }",]);

        let css2 = ":hover .autocss_2 { visibility: visible; }";
        assert_eq!(css_split_rows(css2), vec![":hover .autocss_2 { visibility: visible; }",]);

        let css3 = "color: red;
            .autocss_3 { color: white; }
        ";

        assert_eq!(
            css_split_rows(css3),
            vec!["color: red", ".autocss_3 { color: white; }",]
        );
    }
}
