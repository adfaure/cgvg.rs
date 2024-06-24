use colored::Colorize;

use crate::ripgrep_json::Match;
use crate::{number_of_digits, wrap_text};

pub fn pad_number(number: u32, max_size: u32) -> String {
    let nb_digits = number_of_digits(&number);
    if nb_digits < max_size {
        let diff = max_size - nb_digits;
        let padding = std::iter::repeat(" ")
            .take(diff as usize)
            .collect::<String>();
        format!("{}{}", number, padding)
    } else {
        format!("{}", number)
    }
}

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
                let begin =
                    String::from(&matched_text[(cursor as usize)..(submatch.start as usize)]);
                let submatch_str = format!(
                    "{}",
                    matched_text[(submatch.start as usize)..(submatch.end as usize)]
                        .blue()
                        .bold()
                );

                cursor = submatch.end;

                color_submatches = format!("{color_submatches}{begin}{submatch_str}");
            }

            color_submatches = format!(
                "{color_submatches}{}",
                &matched_text[(cursor as usize)..].to_string()
            );

            let result = color_submatches;

            Some(result)
        }
        _ => {
            panic!("")
        }
    };

    result
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
            Match::Match { lines, .. } => {
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
