use log::info;
use std::env;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Text {
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SubMatch {
    #[serde(rename = "match")]
    submatch: Text,
    start: u32,
    end: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", content = "data")]
enum Match {
    Begin {
        path: Text,
    },
    Match {
        path: Text,
        lines: Text,
        line_number: u32,
        absolute_offset: u32,
        submatches: Vec<SubMatch>,
    },
    End {
        path: Text,
    },
}

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

    let mut cmd = Command::new(command)
        .args(command_args)
        // Instead check if it is already
        .arg("--json")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("failed to execute process");

    // Ensure we have a handle to stdout
    let stdout = cmd.stdout.take().expect("Failed to open stdout");

    // Use a buffered reader to read the lines asynchronously
    let mut reader = BufReader::new(stdout).lines();

    // 
    while let Some(line) = reader.next_line().await.expect("Failed to read line") {
        println!("Received line: {}", line);
        println!("{:?}", serde_json::from_str::<Match>(&line))
    }

    // Ensure the command completes
    let status = cmd.wait().await.expect(""); //cmd.await.expect("Command wasn't running");
    println!("Command finished with status: {}", status);
}
