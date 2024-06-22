use clap::{error::ErrorKind, Parser};
use log::{debug, info, warn};
use regex::Regex;
use rgvg::common::{expand_path, save_text, Index};
use std::env;
use std::process::ExitCode;
use terminal_size::terminal_size;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

mod views;
use views::match_view;

mod ripgrep_json;
use ripgrep_json::Match;

mod print_terminal;
use print_terminal::{number_of_digits, wrap_text};

static DEFAULT_MATCH_FILE: &'static str = "~/.cgvg.match";
static DEFAULT_RG: &'static str = "rg";

/// rg find code using ripgrep
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Place match file of rgvg
    #[arg(short, long, default_value = DEFAULT_MATCH_FILE)]
    match_file: String,
    /// Binary name of rg, or path
    #[arg(short, long, default_value = DEFAULT_RG)]
    rg_bin_path: String,

    /// Arguments for rg command. rg needs to be installed and in your PATH for cg to be able to find it.
    ///
    /// Example `cg find_text .` -> Look for find_text in the current directory
    // trailing_var_arg tells clap to stop parsing and collecting
    // everything as if the user would have provided --
    #[arg(trailing_var_arg = true, required = true)]
    rg: Vec<String>,
}

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();

    let size = terminal_size();
    let terminal_size = size.unwrap().0;

    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(err) => {
            // If clap fails to parse the command line we passe everything to rg
            match err.kind() {
                // Display help is considered as an error.
                ErrorKind::DisplayHelp => {
                    println!("{}", err);
                    return ExitCode::from(0);
                }
                _ => {
                    let args: Vec<String> = env::args().skip(1).collect();
                    info!("Fail to parse commandline, falling back to rg command.");
                    info!("error: {err:?}");

                    Args {
                        match_file: DEFAULT_MATCH_FILE.to_string(),
                        rg_bin_path: DEFAULT_RG.to_string(),
                        rg: args.clone(),
                    }
                }
            }
        }
    };

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

    let mut matches = vec![];

    while let Some(line) = reader.next_line().await.expect("Failed to read line") {
        debug!("Received line: {}", line);
        debug!("{:?}", serde_json::from_str::<Match>(&line));

        let matched = match serde_json::from_str::<Match>(&line) {
            Ok(parsed) => parsed,
            Err(e) => {
                warn!("TODO: {e:?}");
                eprintln!("Received record from rg with unsopported format: {}", line);
                return ExitCode::from(1);
            }
        };

        matches.push((matched.clone(), idx));

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

    match_view(&matches, &(terminal_size.0 as usize), Some(&500));

    // Ensure the command completes
    let status = cmd.wait().await.expect("");
    debug!("Command finished with status: {}", status);

    let match_file = expand_path(&args.match_file).unwrap();
    save_text(file_and_line, &match_file);

    return ExitCode::from(0);
}
