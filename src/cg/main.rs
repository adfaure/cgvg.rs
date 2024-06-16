use clap::Parser;
use log::{debug, info};
use std::process::ExitCode;
use std::*;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use rgvg::common::{expand_paths, save, Index};

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
}

#[tokio::main]
async fn main() -> ExitCode {
    env_logger::init();

    let args = Args::parse();

    // The first argument is the command, and the rest are its arguments
    let command = "rg";
    let command_args = vec!["--sort", "path", "dbqezrazr.", "/home/adfaure/code/oar3"];

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
                path,
                lines,
                line_number,
                absolute_offset: _,
                submatches: _,
            } => {
                println!("{} {} {}\n\t{}", idx, path.text, line_number, lines.text);
                file_and_line.push((path.text, line_number));
                idx += 1;
            }
            _ => {}
        }
    }

    // Ensure the command completes
    let status = cmd.wait().await.expect("");
    debug!("Command finished with status: {}", status);

    let (index_file, match_file) = expand_paths(&args.index_file, &args.match_file).unwrap();
    save(file_and_line, &match_file, &index_file);

    return ExitCode::from(0);
}
