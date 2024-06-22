use colored::Colorize;

use crate::ripgrep_json::Match;
use crate::{number_of_digits, wrap_text};

/// Print a ripgrep match in the terminal, wrapping the matching text.
/// This function can be used on each individual record from ripgrep,
/// and can be called online. However, doing so may result in less satisfying
/// printing, as we start printing before knowing the final size of the indices and line numbers.
///
/// `matched` A rust representation of RG json format. Can be either a match, a begin, a end, or a
/// summary.
///
/// `idx` The idx to associate the match with (that will be used but the user to open this match).
///
/// `terminal_size` The current terminal width to determine the text size
/// ```
pub fn match_view_online(matched: &Match, idx: &usize, terminal_size: &usize) -> Option<String> {
    let colored_match = color_match(matched);
    let line_number = match matched {
        Match::Match {
            line_number,
            ..
        } => Some(*line_number),
        _ => None,
    };

    let result = (line_number, colored_match);

    match &result {
        (Some(line_number), Some(text)) => {
            return padding_and_wrap(&text, line_number, idx, terminal_size);
        }
        (None, Some(text)) => return Some(text.to_string()),
        _ => None,
    }
}

pub fn padding_and_wrap(colored_text: &String, line_number: &u32, idx: &usize, terminal_size: &usize) -> Option<String> {
            let mut result = "".to_string();

            let line_number_len = number_of_digits(&(*line_number as usize));
            let idx_len = number_of_digits(&idx);

            let prefix = format!(
                "{}    {}    ",
                idx.to_string().cyan(),
                line_number.to_string().magenta()
            );
            let prefix_size = line_number_len + idx_len + 8;

            let padding = std::iter::repeat(" ")
                .take(prefix_size - 1)
                .collect::<String>();

            let text_size = terminal_size - prefix_size;

            for (line, s) in wrap_text(&colored_text, &text_size, &8).iter().enumerate() {
                if line == 0 {
                    result = format!("{prefix}{s}\n");
                } else {
                    result = format!("{result}{padding} {s}\n");
                }
            }

            return Some(result);
}

pub fn color_match(m: &Match) -> Option<String> {
    let result = match m {
        Match::Begin { path } => Some(format!("{}\n", path.text.red())),
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
                let begin = String::from(&matched_text[cursor..submatch.start]).bright_green();
                let submatch_str = format!(
                    "{}",
                    matched_text[submatch.start..submatch.end].yellow().bold()
                );

                cursor = submatch.end;

                color_submatches = format!("{color_submatches}{begin}{submatch_str}");
            }

            color_submatches = format!(
                "{color_submatches}{}",
                &matched_text[cursor..].to_string().bright_green()
            );

            let result = color_submatches;

            Some(result)
        }
        Match::End { path: _ } => Some(format!("")),
        _ => None,
    };

    result
}

pub fn match_view(matched: &Vec<(Match, u32)>, terminal_size: &usize) -> Option<String> {
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

    None
}
