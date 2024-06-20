use clap::Parser;
use colored::Colorize;
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
    match matched {
        Match::Begin { path } => {
            return Some(format!("{}", path.text.red()));
        }
        Match::Match {
            path: _,
            lines,
            line_number,
            absolute_offset: _,
            submatches,
        } => {
            let colored_idx = format!("{idx}").yellow();
            let colored_line_number = format!("{line_number}").cyan();

            let mut color_submatches = String::from("");
            let remaining = String::from(lines.text.trim_end_matches('\n'));
            let mut cursor = 0;
            for submatch in submatches.iter() {
                // println!("match: {lines:?}, {}, start: {}", remaining, submatch.start);
                let begin = String::from(&remaining[cursor..submatch.start]);
                let submatch_str =
                    format!("{}", remaining[submatch.start..submatch.end].bright_green().bold());

                cursor = submatch.end;

                color_submatches = format!("{color_submatches}{begin}{submatch_str}");
            }

            color_submatches = format!(
                "{color_submatches}{}",
                &remaining[cursor..].to_string()
            );

            let result = format!("{colored_idx}:{colored_line_number}\t{color_submatches}",);

            return Some(result);
        }
        Match::End { path: _ } => {
            return Some(format!(""));
        }
        _ => {
            return None;
        }
    }
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
