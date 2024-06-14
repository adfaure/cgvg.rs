use log::{debug, info};
use std::env;
use std::*;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use rgvg::common::{save, Index};

mod ripgrep_json;
use ripgrep_json::Match;

#[tokio::main]
async fn main() {
    env_logger::init();

    // Collect all the command-line arguments passed to the Rust program
    let args: Vec<String> = env::args().collect();

    // Ensure there are arguments
    if args.len() < 2 {
        eprintln!("Usage: {} <command> [<args>...]", args[0]);
        std::process::exit(1);
    }

    // The first argument is the command, and the rest are its arguments
    let command = &args[1];
    let command_args = &args[2..];

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
                idx += 1;
                println!("{} {} {}\n\t{}", idx, path.text, line_number, lines.text);
                file_and_line.push((path.text, line_number));
            }
            _ => {}
        }
    }

    let data_file = "file.bin";
    let index_file = "index.bin";

    save(file_and_line, data_file, index_file);

    // Ensure the command completes
    let status = cmd.wait().await.expect(""); //cmd.await.expect("Command wasn't running");
    debug!("Command finished with status: {}", status);
}
