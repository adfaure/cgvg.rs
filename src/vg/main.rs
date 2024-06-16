use clap::Parser;
use log::debug;
use rgvg::common::{expand_paths, load};
use std::env;
use std::fs;
use std::process::{Command, ExitCode};

use std::ffi::CString;
use std::ptr;

extern "C" {
    fn execvp(path: *const libc::c_char, argv: *const *const libc::c_char) -> libc::c_int;
}

/// vg edit code mathing previous rg research
///
/// The program reads your $EDITOR environment variable.
/// The editors vim, nvim, emacs, code, codium should be handled by default.
/// If your editor is not in the liste you can use the --format option to describe the command line that shoul open your editor.
///
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Selection number from previous rg command
    seletion: u32,
    /// Format to describe how to open your editor.
    /// Simple format to tune how `vg` will open your editor.
    /// The format should use the placeholders: {LINE}, {EDITOR} and {PATH}.
    ///
    /// The default for vim for instance is "{EDITOR} +{LINE} {PATH}".
    /// It will be resolved as "nvim +21 /path/to/file"
    #[arg(short, long)]
    format: Option<String>,

    /// Specify the path to your editor
    #[arg(short, long)]
    editor: Option<String>,

    /// path to index state file of rgvg
    #[arg(short, long, default_value = "~/.cgvg.idx")]
    index_file: String,
    /// Place match file of rgvg
    #[arg(short, long, default_value = "~/.cgvg.match")]
    match_file: String,
}

fn main() -> ExitCode {
    env_logger::init();

    let args = Args::parse();
    debug!("{args:?}");

    // Find a text editor
    let editor_path = match args.editor {
        Some(editor) => editor,
        None => match env::var("EDITOR") {
            Ok(val) => val,
            Err(_) => {
                eprintln!("Failed to find and editor. Check the content of your $EDITOR environment variable or use the command line option `--editor`.");
                std::process::exit(1);
            }
        },
    };

    // Using `which` to check that the editor is in the path
    let find_editor = Command::new("which")
        .arg(&editor_path)
        .output()
        .expect("failed to execute process");

    if !find_editor.status.success() {
        eprintln!("Could not find editor ($EDITOR={editor_path:}) in path.");
        return ExitCode::from(1);
    }

    // Finding the editor name to choose the command to open the file
    let editor_name = if editor_path.starts_with('/') || editor_path.starts_with("./") {
        editor_path.split('/').last().expect("Editor name")
    } else {
        &editor_path
    };

    let open_format = match args.format {
        Some(format) => format,
        None => match editor_name {
            "vim" | "vi" | "nvim" | "emacs" => String::from("{EDITOR} +{LINE} {PATH}"),
            "code" | "codium" => String::from("{EDITOR} -g {PATH}:{LINE}"),
            _ => {
                panic!("No rule for editor: {editor_name:?}. You can use the `--format` option.");
            }
        },
    };

    let (match_file, index_file) = expand_paths(&args.match_file, &args.index_file).unwrap();

    match (fs::metadata(&match_file), fs::metadata(&index_file)) {
        (Ok(_), Ok(_)) => {}
        (Err(_), _) | (_, Err(_)) => {
            eprintln!(
                "Could not find state files {} or {}. Did you use vg without rg?",
                index_file, match_file
            );
            return ExitCode::from(1);
        }
    }

    let selected = args.seletion;

    let result = load(selected, &match_file, &index_file).unwrap();

    // Replacing the placeholders
    let mut command_args: String = open_format.replace("{LINE}", &result.1.to_string());
    command_args = command_args.replace("{EDITOR}", &editor_path);
    command_args = command_args.replace("{PATH}", &result.0);

    debug!("command_args: {}", command_args);

    // Argument for excv, the first arg is the command name
    let splitted_args: Vec<CString> = command_args
        .split_whitespace()
        .map(|arg| CString::new(arg).expect("CString Failed to create"))
        .collect();

    let mut args_ptrs: Vec<*const libc::c_char> =
        splitted_args.iter().map(|arg| arg.as_ptr()).collect();

    args_ptrs.push(ptr::null());

    let command = CString::new(editor_path).expect("cannot create cstring for program");

    unsafe {
        // Execvp looks for the path if the binary name is given
        execvp(command.as_ptr(), args_ptrs.as_ptr());
    }

    return ExitCode::from(1);
}
