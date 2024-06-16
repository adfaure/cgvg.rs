use rgvg::common::load;
use std::env;
use clap::Parser;
use log::debug;

use std::ffi::CString;
use std::ptr;

extern "C" {
    fn execvp(path: *const libc::c_char, argv: *const *const libc::c_char) -> libc::c_int;
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Selection number from previous rg command
    seletion: u32,
}

fn main() {
    env_logger::init();

    let args = Args::parse();
    debug!("{args:?}");

    let key = "EDITOR";
    let editor = match env::var(key) {
        Ok(val) => val,
        Err(_) => panic!("couldn't find EDITOR"),
    };

    let data_file = "file.bin";
    let index_file = "index.bin";

    let args: Vec<String> = env::args().skip(1).collect();
    let selected = args[0].parse::<u32>().unwrap();

    let result = load(selected, data_file, index_file).unwrap();
    debug!("retrieved tuple: {result:?}");

    let open_format = match editor.as_str() {
        "vim" | "vi" | "nvim" | "emacs" => {
            String::from("{EDITOR} +{LINE} {PATH}")
        },
        "code" | "codium" => {
            String::from("{EDITOR} -g {PATH}:{LINE}")
        },
        _ => {
            panic!("No rule for editor: {editor:?}");
        }
    };

    let mut command_args: String = open_format.replace("{LINE}", &result.1.to_string());
    command_args = command_args.replace("{EDITOR}", &editor);
    command_args = command_args.replace("{PATH}", &result.0);

    debug!("command_args: {}", command_args);

    // Argument for excv, the first arg is the command name
    let splitted_args: Vec<CString> =
        command_args
        .split_whitespace()
        .map(|arg| CString::new(arg).expect("CString Failed to create"))
        .collect();

    let mut args_ptrs: Vec<*const libc::c_char> =
        splitted_args.iter().map(|arg| arg.as_ptr()).collect();

    args_ptrs.push(ptr::null());

    let command = CString::new(editor).expect("cannot create cstring for program");

    let res;
    unsafe {
        // Execvp looks for the path if the binary name is given
        res = execvp(command.as_ptr(), args_ptrs.as_ptr());
    }

    std::process::exit(res);
}
