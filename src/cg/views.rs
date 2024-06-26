use colored::Colorize;

use crate::ripgrep_json::Match;
use crate::{number_of_digits, pad_number, wrap_text};

pub fn padding_and_wrap(
    colored_text: &String,
    line_number: &u32,
    idx: &u32,
    terminal_size: &u32,
    line_number_max: Option<u32>,
    idx_max: Option<u32>,
) -> Vec<String> {
    let line_number_str = pad_number(
        *line_number,
        number_of_digits(&line_number_max.unwrap_or(*line_number)),
    );
    let idx_str = pad_number(*idx, number_of_digits(&idx_max.unwrap_or(*idx)));

    let prefix = format!(
        "{}    {}    ",
        idx_str.cyan(),
        line_number_str.bright_purple()
    );

    let prefix_size = (line_number_str.len() + idx_str.len()) as u32 + 8;

    let padding = std::iter::repeat(" ")
        .take((prefix_size - 1) as usize)
        .collect::<String>();

    let text_size = terminal_size - prefix_size;

    let mut result = vec![];
    for (line, s) in wrap_text(&colored_text, &text_size, &8, true)
        .enumerate()
    {
        if line == 0 {
            result.push(format!("{prefix}{s}"));
        } else {
            result.push(format!("{padding} {s}"));
        }
    }

    return result;
}

pub fn color_submatch(text: &String, submatches: &Vec<(u32, u32)>) -> Option<String> {
    let mut color_submatches = String::from("");
    let mut cursor = 0;

    // FIXME: Don't trim here
    let matched_text = String::from(text.trim_end_matches('\n'));

    for (start, end) in submatches.iter() {
        assert!(
            (*end as usize) <= matched_text.len(),
            "Cannot color submatches, text is shorter than submatches {end} {}",
            matched_text.len()
        );

        let begin = String::from(&matched_text[(cursor as usize)..(*start as usize)]);
        let submatch_str = format!(
            "{}",
            matched_text[(*start as usize)..(*end as usize)]
                .blue()
                .bold()
        );

        cursor = *end;

        color_submatches = format!("{color_submatches}{begin}{submatch_str}");
    }

    color_submatches = format!(
        "{color_submatches}{}",
        &matched_text[(cursor as usize)..].to_string()
    );

    let result = color_submatches;

    Some(result)
}

pub fn match_view(matched: &Vec<(Match, u32)>, terminal_size: &u32, max_text_size: Option<&u32>) {
    let (mut max_idx, mut max_line) = (0, 0);
    for (m, idx) in matched.iter() {
        match m {
            Match::Match { line_number, .. } => {
                max_idx = std::cmp::max(max_idx, *idx);
                max_line = std::cmp::max(max_line, *line_number);
            }
            _ => {}
        };
    }
    let mut i = matched.iter();

    while let Some(m) = i.next() {
        let (record, idx) = m;
        match &record {
            Match::Match {
                lines, submatches, ..
            } => {
                let colored_match =
                    if max_text_size.is_some_and(|max| lines.text.len() > *max as usize) {
                        Some(
                            format!(
                                "text truncated size({})>{}",
                                lines.text.len(),
                                max_text_size.unwrap()
                            )
                            .red()
                            .to_string(),
                        )
                    } else {
                        color_submatch(
                            &lines.text,
                            &submatches.iter().map(|s| (s.start, s.end)).collect(),
                        )
                    };

                let line_number = match record {
                    Match::Match { line_number, .. } => Some(line_number),
                    _ => None,
                };

                let result = (line_number, colored_match);

                let lines_to_print = match &result {
                    (Some(line_number), Some(text)) => padding_and_wrap(
                        &text,
                        line_number,
                        idx,
                        terminal_size,
                        Some(max_line),
                        Some(max_idx),
                    ),
                    _ => panic!(),
                };

                for line in lines_to_print.iter() {
                    println!("{line}");
                }
            }
            Match::Begin { path } => {
                println!("{}", path.text.red());
            }
            Match::End { .. } => {
                println!("");
            }
            _ => {}
        };
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use std::panic;

    #[test]
    fn test_color_submatch() {
        let text = "aaaaabbbbbcccccdddddeeeee".to_string();
        let submatches = vec![(0, 15)];
        let colored = color_submatch(&text, &submatches);
        println!("{colored:?}");
        assert_eq!(colored.is_some(), true);
        assert_eq!(
            "\u{1b}[1;34maaaaabbbbbccccc\u{1b}[0mdddddeeeee",
            colored.unwrap()
        );

        let submatches = vec![(0, 5)];
        let colored = color_submatch(&text, &submatches);
        assert_eq!(colored.is_some(), true);
        assert_eq!(
            "\u{1b}[1;34maaaaa\u{1b}[0mbbbbbcccccdddddeeeee",
            colored.unwrap()
        );

        let submatches = vec![(10, 25)];
        let colored = color_submatch(&text, &submatches);
        assert_eq!(colored.is_some(), true);
        assert_eq!(
            "aaaaabbbbb\u{1b}[1;34mcccccdddddeeeee\u{1b}[0m",
            colored.unwrap()
        );

        let submatches = vec![(0, 24)];
        let colored = color_submatch(&text, &submatches);
        assert_eq!(colored.is_some(), true);
        assert_eq!(
            "\u{1b}[1;34maaaaabbbbbcccccdddddeeee\u{1b}[0me",
            colored.unwrap()
        );

        let submatches = vec![(0, 26)];
        let result = panic::catch_unwind(|| {
            color_submatch(&text, &submatches);
        });
        assert_eq!(true, result.is_err());
    }
}
