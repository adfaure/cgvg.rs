use colored::Colorize;

use crate::ripgrep_json::Match;
use crate::{number_of_digits, wrap_text};

pub fn padding_and_wrap(
    colored_text: &String,
    line_number: &u32,
    idx: &usize,
    terminal_size: &usize,
    line_number_max_digits: Option<usize>,
    idx_max_digits: Option<usize>,
) -> Vec<String> {
    let line_number_len = number_of_digits(&(*line_number as usize));
    let idx_len = number_of_digits(&idx);

    let line_number_str = match line_number_max_digits {
        Some(max) if max > line_number_len => {
            let diff = max - line_number_len;
            let padding = std::iter::repeat(" ").take(diff).collect::<String>();
            format!("{}{}", line_number, padding)
        }
        _ => {
            format!("{}", line_number)
        }
    };

    let idx_str = match idx_max_digits {
        Some(max) if max > idx_len => {
            let diff = max - idx_len;
            let padding = std::iter::repeat(" ").take(diff).collect::<String>();
            format!("{}{}", idx, padding)
        }
        _ => {
            format!("{}", idx)
        }
    };

    let prefix = format!(
        "{}    {}    ",
        idx_str.cyan(),
        line_number_str.bright_purple()
    );

    let prefix_size =
        idx_max_digits.unwrap_or(idx_len) + line_number_max_digits.unwrap_or(line_number_len) + 8;

    let padding = std::iter::repeat(" ")
        .take(prefix_size - 1)
        .collect::<String>();

    let text_size = terminal_size - prefix_size;

    let mut result = vec![];
    for (line, s) in wrap_text(&colored_text, &text_size, &8, true)
        .iter()
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

pub fn color_match(m: &Match) -> Option<String> {
    let result = match m {
        Match::Match {
            path: _,
            lines,
            line_number: _,
            absolute_offset: _,
            submatches,
        } => {
            let mut color_submatches = String::from("");
            let mut cursor = 0;

            let matched_text = String::from(lines.text.trim_end_matches('\n'));

            for submatch in submatches.iter() {
                let begin = String::from(&matched_text[cursor..submatch.start]);
                let submatch_str = format!(
                    "{}",
                    matched_text[submatch.start..submatch.end].blue().bold()
                );

                cursor = submatch.end;

                color_submatches = format!("{color_submatches}{begin}{submatch_str}");
            }

            color_submatches = format!("{color_submatches}{}", &matched_text[cursor..].to_string());

            let result = color_submatches;

            Some(result)
        }
        _ => {
            panic!("")
        }
    };

    result
}

pub fn match_view(
    matched: &Vec<(Match, u32)>,
    terminal_size: &usize,
    max_text_size: Option<&usize>,
) {
    let (mut max_idx, mut max_line) = (0, 0);
    for (m, idx) in matched.iter() {
        match m {
            Match::Match { line_number, .. } => {
                max_idx = std::cmp::max(max_idx, number_of_digits(&(*idx as usize)));
                max_line = std::cmp::max(max_line, number_of_digits(&(*line_number as usize)));
            }
            _ => {}
        };
    }
    let mut i = matched.iter();

    while let Some(m) = i.next() {
        let (record, idx) = m;
        match &record {
            Match::Match { lines, .. } => {
                let colored_match = if max_text_size.is_some_and(|max| lines.text.len() > *max) {
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
                    color_match(&record)
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
                        &(*idx as usize),
                        terminal_size,
                        Some(max_line as usize),
                        Some(max_idx as usize),
                    ),
                    _ => panic!(),
                };

                for line in lines_to_print.iter() {
                    println!("{line}");
                }
            }
            Match::Begin { path } => {
                println!("{}", path.text.blue());
            }
            Match::End { .. } => {
                println!("");
            }
            _ => {}
        };
    }
}
