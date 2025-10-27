// src/main.rs
use std::env;
use std::path::PathBuf;

// import lines_editor_module lines_editor_module w/ these 2 lines:
mod lines_editor_module;
use lines_editor_module::{
    LinesError, full_lines_editor, get_default_filepath, is_in_home_directory,
    memo_mode_mini_editor_loop, print_help, prompt_for_filename,
};

// "Source-It" allows build source code transparency: --source
mod source_it_module;
use source_it_module::{SourcedFile, handle_sourceit_command};
// Source-It: Developer explicitly lists files to embed w/
const SOURCE_FILES: &[SourcedFile] = &[
    SourcedFile::new("Cargo.toml", include_str!("../Cargo.toml")),
    SourcedFile::new("src/main.rs", include_str!("main.rs")),
    SourcedFile::new("src/tests.rs", include_str!("tests.rs")),
    SourcedFile::new(
        "src/source_it_module.rs",
        include_str!("source_it_module.rs"),
    ),
    SourcedFile::new(
        "src/lines_editor_module.rs",
        include_str!("lines_editor_module.rs"),
    ),
    // SourcedFile::new("src/lib.rs", include_str!("lib.rs")),
    SourcedFile::new("README.md", include_str!("../README.md")),
    SourcedFile::new("LICENSE", include_str!("../LICENSE")),
    SourcedFile::new(".gitignore", include_str!("../.gitignore")),
];

// Cargo-tests in tests.rs // run: cargo test
#[cfg(test)]
mod tests;

/// Main entry point - routes between memo mode and full editor mode
///
/// # Purpose
/// Determines which mode to use based on current directory and arguments.
///
/// # Command Line Usage
/// - `lines` - Memo mode (if in home) or error (if elsewhere)
/// - `lines file.txt` - Full editor mode with file
/// - `lines /path/to/dir/` - Full editor mode, prompts for filename
///
/// # Mode Selection Logic
/// 1. If CWD is home directory -> memo mode available
/// 2. Otherwise -> full editor mode (requires file argument)
///
/// # Exit Codes
/// - 0: Success
/// - 1: General error
/// - 2: Invalid arguments
fn main() -> Result<(), LinesError> {
    let args: Vec<String> = std::env::args().collect();
    // Check if we're in home directory
    let in_home = is_in_home_directory()?;

    // // Diagnostics
    // println!("=== Lines Text Editor ===");
    // println!("Current directory: {}", env::current_dir()?.display());
    // if in_home {
    //     println!("Mode: Memo mode available (in home directory)");
    // } else {
    //     println!("Mode: Full editor (not in home directory)");
    // }
    // println!();

    // Parse command line arguments
    match args.len() {
        1 => {
            // No arguments provided
            if in_home {
                // Memo mode: create today's file
                println!("Starting memo mode...");
                let original_file_path = get_default_filepath(None)?;
                memo_mode_mini_editor_loop(&original_file_path)
            } else {
                // Full editor mode - prompt for filename in current directory
                println!("No file specified. Creating new file in current directory.");
                let filename = prompt_for_filename()?;
                let current_dir = env::current_dir()?;
                let original_file_path = current_dir.join(filename);
                full_lines_editor(Some(original_file_path))
            }
        }
        2 => {
            // One argument provided
            let arg = &args[1];

            // Check for special commands
            match arg.as_str() {
                "--help" | "-h" => {
                    print_help();
                    Ok(())
                }
                "--version" | "-v" | "-V" => {
                    println!("Lines-Editor Version: {}", env!("CARGO_PKG_VERSION"));
                    Ok(())
                }
                "--source" | "--source_it" => {
                    match handle_sourceit_command("lines_editor", None, SOURCE_FILES) {
                        Ok(path) => println!("Source extracted to: {}", path.display()),
                        Err(e) => eprintln!("Failed to extract source: {}", e),
                    }
                    Ok(())
                }
                _ => {
                    // Treat as file/directory path
                    if in_home && !arg.contains('/') && !arg.contains('\\') {
                        // In home + simple filename = memo mode with custom name
                        println!("Starting memo mode with custom file: {}", arg);
                        let original_file_path = get_default_filepath(Some(arg))?;
                        memo_mode_mini_editor_loop(&original_file_path)
                    } else {
                        // Full editor mode with specified path
                        let path = PathBuf::from(arg);
                        full_lines_editor(Some(path))
                    }
                }
            }
        }
        3 => {
            // Two arguments provided
            let flag = &args[1];
            let filepath_arg = &args[2];

            // Check if first arg is append flag
            match flag.as_str() {
                "-a" | "--append" => {
                    // Memo mode (append-only) with specified file path
                    let file_path = PathBuf::from(filepath_arg);
                    println!(
                        "Starting memo mode (append-only) with file: {}",
                        file_path.display()
                    );
                    memo_mode_mini_editor_loop(&file_path)
                }
                _ => {
                    // Unknown flag combination
                    eprintln!("Error: Invalid arguments");
                    eprintln!("Usage: lines [filename | -a <filepath> | --help]");
                    eprintln!("Examples:");
                    eprintln!("  lines notes.txt          # Full editor mode");
                    eprintln!("  lines -a notes.txt       # Append-only mode");
                    eprintln!("  lines --append /tmp/log  # Append-only mode");
                    std::process::exit(2);
                }
            }
        }
        _ => {
            // Multiple arguments - currently not supported
            eprintln!("Error: It's The Too many arguments!");
            eprintln!("Try Usage: lines [filename | -a <filepath> | --help]");
            std::process::exit(2);
        }
    }
}
