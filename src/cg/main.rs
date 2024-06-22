use clap::Parser;
use colored::Colorize;
use log::{debug, info};
use regex::Regex;
use rgvg::common::{expand_paths, save, Index};
use std::process::ExitCode;
use terminal_size::terminal_size;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

mod ripgrep_json;
use ripgrep_json::Match;

mod print_terminal;
use print_terminal::{number_of_digits, wrap_text};

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
    /// Binary name of rg, or path
    #[arg(short, long, default_value = "rg")]
    rg_bin_path: String,

    /// rg command to use. rg needs to be installed and in your PATH for cg to be able to find it.
    // trailing_var_arg tells clap to stop parsing and collecting
    // everything as if the user would have provided --
    #[arg(trailing_var_arg = true, required = true)]
    rg: Vec<String>,
}

pub fn match_view(matched: &Match, idx: &usize, terminal_size: &usize) -> Option<String> {
    let result = match matched {
        Match::Begin { path } => Some((None, format!("{}", path.text.red()))),
        Match::Match {
            path: _,
            lines,
            line_number,
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

            Some((Some(*line_number), result))
        }
        Match::End { path: _ } => Some((None, format!(""))),
        _ => None,
    };

    match &result {
        Some((Some(line_number), text)) => {
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

            for (line, s) in wrap_text(text, &text_size, &8).iter().enumerate() {
                if line == 0 {
                    result = format!("{prefix}{s}\n");
                } else {
                    result = format!("{result}{padding} {s}\n");
                }
            }

            return Some(result);
        }
        Some((None, text)) => return Some(text.to_string()),
        None => return None,
    };
}

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();

    let size = terminal_size();
    let terminal_size = size.unwrap().0;

    let args = Args::parse();
    debug!("{:?}", args);

    // Using `which` to check that the editor is in the path
    let find = Command::new("which")
        .arg(&args.rg_bin_path)
        .output()
        .await
        .expect("failed to fin rg");

    if find.status.code().unwrap() != 0 {
        eprintln!(
            "rg not found at path: {}, try to use install rg or `--rg-bin-path`",
            args.rg_bin_path
        );
        return ExitCode::from(1);
    }


    let version = std::process::Command::new(&args.rg_bin_path)
        .arg("--version")
        .output()
        .expect("could not run rg");

    let re = Regex::new(r"ripgrep (\d+\.\d+.\d+)").unwrap();

    let mut binding = version.stdout.lines();
    let check_version = binding.next_line().await.unwrap().unwrap();
    if !re.is_match(&check_version) {
        eprintln!("Binary does not seem to be ripgrep: {check_version}");
    }

    debug!("{check_version:?}");
    info!("rg version: {:?}", version);

    // The first argument is the command, and the rest are its arguments
    let command_args = &args.rg;
    // Log the command and its arguments
    info!(
        "Running command: {} {:?}",
        args.rg_bin_path,
        command_args.join(" ")
    );

    let mut cmd = Command::new(args.rg_bin_path)
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
    debug!("terminal size= {:?}", terminal_size);

    while let Some(line) = reader.next_line().await.expect("Failed to read line") {
        debug!("Received line: {}", line);
        debug!("{:?}", serde_json::from_str::<Match>(&line));
        let matched = serde_json::from_str::<Match>(&line).ok().unwrap();

        match match_view(&matched, &idx, &(terminal_size.0 as usize)) {
            Some(text) => {
                println!("{text}")
            }
            None => {}
        };

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
    }

    // Ensure the command completes
    let status = cmd.wait().await.expect("");
    debug!("Command finished with status: {}", status);

    let (index_file, match_file) = expand_paths(&args.index_file, &args.match_file).unwrap();
    save(file_and_line, &match_file, &index_file);

    return ExitCode::from(0);
}
