use clap::Parser;
use colored::Colorize;
use itertools::FoldWhile::{Continue, Done};
use itertools::Itertools;
use log::{debug, info};
use rgvg::common::{expand_paths, save, Index};
use std::process::ExitCode;
use std::*;
use terminal_size::{terminal_size, Height, Width};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

mod ripgrep_json;
use ripgrep_json::Match;

/// rg find code using ripgrep
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// path to index state file of rgvg
    #[arg(short, long, default_value = "~/.cgvg.idx")]
    index_file: String,
    /// Place match file of rgvg
    #[arg(short, long, default_value = "~/.cgvg.match")]
    match_file: String,
    /// rg command to use. rg needs to be installed and in your PATH for cg to be able to find it.
    // trailing_var_arg tells clap to stop parsing and collecting
    // everything as if the user would have provided --
    #[arg(trailing_var_arg = true, required = true)]
    rg: Vec<String>,
}

pub fn match_view(matched: &Match, idx: &usize) -> Option<String> {
    let result = match matched {
        Match::Begin { path } => Some(format!("{}", path.text.red())),
        Match::Match {
            path: _,
            lines,
            line_number,
            absolute_offset: _,
            submatches,
        } => {
            let colored_line_number = format!("{line_number}").cyan();
            let mut color_submatches = String::from("");
            let remaining = String::from(lines.text.trim_end_matches('\n'));
            let mut cursor = 0;
            for submatch in submatches.iter() {
                let begin = String::from(&remaining[cursor..submatch.start]);
                let submatch_str = format!(
                    "{}",
                    remaining[submatch.start..submatch.end]
                        .bright_green()
                        .bold()
                );

                cursor = submatch.end;

                color_submatches = format!("{color_submatches}{begin}{submatch_str}");
            }

            color_submatches = format!("{color_submatches}{}", &remaining[cursor..].to_string());

            let result = color_submatches;

            Some(result)
        }
        Match::End { path: _ } => Some(format!("")),
        _ => None,
    };

    let colored_idx = format!("{idx}").yellow();

    match &result {
        Some(text) => {
            let size = terminal_size();
            if let Some((Width(w), Height(h))) = size {
                let mut result = "".to_string();
                for s in wrap_text(text, w as usize).iter() {
                    result = format!("{result}{s}\n");
                }
                return Some(result);
            } else {
                debug!("Unable to get terminal size");
            }
        }
        None => {}
    };

    return result;
}

pub fn wrap_text<'a>(text: &'a str, max_length: usize) -> Vec<String> {
    // let array = text.chars().collect::<Vec<_>>();
    let mut memory = None;

    let wrapped = text
        .chars()
        .batching(|it| {
            let (_, temp, _) = it
                .fold_while(
                    (0, vec![], false),
                    |(length, mut acc, wait_end_of_escape_sequence), c| {
                        let mut new_length = length;
                        match memory {
                            Some(c) => {
                                acc.push(c);
                                memory = None;
                                new_length += 1;
                            }
                            _ => {}
                        };

                        // println!("top: {length} {acc:?} {c:?}");

                        if wait_end_of_escape_sequence {
                            acc.push(c);
                            return Continue((length, acc, c != 'm'));
                        }

                        if c == '\u{1b}' {
                            acc.push(c);
                            return Continue((length, acc, true));
                        }

                        new_length += 1;

                        if new_length > max_length {
                            memory = Some(c);
                            Done((new_length, acc, wait_end_of_escape_sequence))
                        } else {
                            acc.push(c);
                            Continue((new_length, acc, wait_end_of_escape_sequence))
                        }
                    },
                )
                .into_inner();

            let line: String = temp.into_iter().collect();

            if line.is_empty() {
                None
            } else {
                Some(line)
            }
        })
        .map(|array| String::from(array))
        .collect_vec();

    wrapped
}

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();

    let size = terminal_size();
    if let Some((Width(w), Height(h))) = size {
        debug!("Your terminal is {} cols wide and {} lines tall", w, h);
    } else {
        debug!("Unable to get terminal size");
    }

    let args = Args::parse();
    debug!("{:?}", args);

    // The first argument is the command, and the rest are its arguments
    let command = &args.rg[0];
    let command_args = &args.rg[1..];

    // Log the command and its arguments
    info!("Running command: {} {:?}", command, command_args.join(" "));

    let mut cmd = Command::new(command)
        .args(command_args)
        //TODO: Instead check if the flag is already present
        .arg("--json")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("failed to execute process");

    // Ensure we have a handle to stdout
    let stdout = cmd.stdout.take().expect("Failed to open stdout");

    // Use a buffered reader to read the lines asynchronously
    let mut reader = BufReader::new(stdout).lines();

    let mut idx = 0;
    let mut file_and_line: Vec<Index> = vec![];

    while let Some(line) = reader.next_line().await.expect("Failed to read line") {
        debug!("Received line: {}", line);
        debug!("{:?}", serde_json::from_str::<Match>(&line));
        let matched = serde_json::from_str::<Match>(&line).ok().unwrap();

        match matched {
            Match::Match {
                ref path,
                lines: _,
                line_number,
                absolute_offset: _,
                submatches: _,
            } => {
                file_and_line.push((path.text.to_string(), line_number));
                idx += 1;
            }
            _ => {}
        };

        match match_view(&matched, &idx) {
            Some(text) => {
                println!("{text}")
            }
            None => {}
        };
    }

    // Ensure the command completes
    let status = cmd.wait().await.expect("");
    debug!("Command finished with status: {}", status);

    let (index_file, match_file) = expand_paths(&args.index_file, &args.match_file).unwrap();
    save(file_and_line, &match_file, &index_file);

    return ExitCode::from(0);
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_wrap_text() {
        // Simple cases
        let res = wrap_text("1234567890abc", 5);
        assert_eq!(vec!["12345", "67890", "abc"], res);

        let res = wrap_text("1234567890abc", 15);
        assert_eq!(vec!["1234567890abc"], res);

        // Got coloring
        let blue = format!("aaaaabbbbbzzzzz").blue().to_string();

        let res = wrap_text(&blue, 5);
        assert_eq!(vec!["\u{1b}[34maaaaa", "bbbbb", "zzzzz\u{1b}[0m"], res);

        let blue_bold_underline = format!("aaaaabbbbbzzzzz").blue().bold().underline().to_string();
        let res = wrap_text(&blue_bold_underline, 5);
        assert_eq!(vec!["\u{1b}[1;4;34maaaaa", "bbbbb", "zzzzz\u{1b}[0m"], res);

        let blue_bold_underline = format!("{}{}{}", "aaaaa".blue(), "zzzzz".bold(), "bbbbb".underline());
        let res = wrap_text(&blue_bold_underline, 5);
        assert_eq!(vec!["\u{1b}[34maaaaa\u{1b}[0m\u{1b}[1m", "zzzzz\u{1b}[0m\u{1b}[4m", "bbbbb\u{1b}[0m" ], res);

    }
}
