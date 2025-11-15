// src/main.rs
use std::env;
use std::path::PathBuf;

// import lines_editor_module lines_editor_module w/ these 2 lines:
mod lines_editor_module;
use lines_editor_module::{
    LinesError, get_default_filepath, is_in_home_directory, lines_full_file_editor,
    memo_mode_mini_editor_loop, print_help, prompt_for_filename, stack_format_it,
};

mod buttons_reversible_edit_changelog_module;
mod toggle_comment_indent_module;

// To make a smaller binary, you can remove source-it.
/// "Source-It" allows build source code transparency: --source
mod source_it_module;
use source_it_module::{SourcedFile, handle_sourceit_command};

mod buffy_format_write_module;
use buffy_format_write_module::{FormatArg, buffy_print, buffy_println};

// To make a smaller binary, you can remove source-it.
/// Source-It: Developer explicitly lists files to embed w/
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
    SourcedFile::new(
        "src/buttons_reversible_edit_changelog_module.rs",
        include_str!("buttons_reversible_edit_changelog_module.rs"),
    ),
    SourcedFile::new(
        "src/toggle_comment_indent_module.rs",
        include_str!("toggle_comment_indent_module.rs"),
    ),
    SourcedFile::new(
        "src/buffy_format_write_module.rs",
        include_str!("buffy_format_write_module.rs"),
    ),
    SourcedFile::new("src/tests.rs", include_str!("tests.rs")),
    // SourcedFile::new("src/lib.rs", include_str!("lib.rs")),
    SourcedFile::new("README.md", include_str!("../README.md")),
    SourcedFile::new("LICENSE", include_str!("../LICENSE")),
    SourcedFile::new(".gitignore", include_str!("../.gitignore")),
];

// Cargo-tests in tests.rs // run: cargo test
#[cfg(test)]
mod tests;

/// Parsed command line arguments for the editor
///
/// # Purpose
/// Holds the structured result of parsing command line arguments,
/// separating concerns of file path, line number, and session recovery.
///
/// # Fields
/// * `file_path` - Optional path to file to edit
/// * `starting_line` - Optional line number to jump to (from file:123 syntax)
/// * `session_path` - Optional path to existing session directory for crash recovery
/// * `mode` - Special mode flags (help, version, source, append)
#[derive(Debug)]
struct ParsedArgs {
    file_path: Option<PathBuf>,
    starting_line: Option<usize>,
    session_path: Option<PathBuf>,
    mode: ArgMode,
}

/// Special argument modes that don't start the editor
#[derive(Debug, PartialEq)]
enum ArgMode {
    Normal,     // Start editor normally
    Help,       // Print help and exit
    Version,    // Print version and exit
    Source,     // Extract source and exit, // To make a smaller binary, you can remove source-it.
    AppendMode, // Memo mode (append-only)
}

/// Parses command line arguments into structured format
///
/// # Purpose
/// Processes raw command line arguments and extracts:
/// - File path with optional :line_number suffix
/// - --session flag with path argument
/// - -a/--append flag for memo mode
/// - Special flags (--help, --version, --source)
///
/// # Argument Patterns Supported
/// ```text
/// lines
/// lines file.txt
/// lines file.txt:123
/// lines --session <path>
/// lines --session <path> file.txt
/// lines file.txt --session <path>
/// lines file.txt:123 --session <path>
/// lines -a file.txt
/// lines --help
/// ```
///
/// # Arguments
/// * `args` - Raw command line arguments (including program name at index 0)
///
/// # Returns
/// * `Ok(ParsedArgs)` - Successfully parsed arguments
/// * `Err(String)` - Parse error with user-friendly message
///
/// # Error Cases
/// - `--session` flag without path argument
/// - Unknown flags
/// - Too many non-flag arguments
fn parse_arguments(args: &[String]) -> Result<ParsedArgs, String> {
    let mut file_path: Option<PathBuf> = None;
    let mut starting_line: Option<usize> = None;
    let mut session_path: Option<PathBuf> = None;
    let mut mode = ArgMode::Normal;

    // Skip program name (args[0])
    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];

        // Check for flags
        match arg.as_str() {
            // Special mode flags
            "--help" | "-h" => {
                mode = ArgMode::Help;
                i += 1;
            }
            "--version" | "-v" | "-V" => {
                mode = ArgMode::Version;
                i += 1;
            }

            // To make a smaller binary, you can remove source-it.
            "--source" | "--source_it" => {
                mode = ArgMode::Source;
                i += 1;
            }

            "-a" | "--append" => {
                mode = ArgMode::AppendMode;
                i += 1;
            }
            // Session flag with path argument
            "--session" | "-s" => {
                // Next argument should be the session path
                if i + 1 >= args.len() {
                    return Err("Error: --session flag requires a path argument".to_string());
                }
                i += 1;
                session_path = Some(PathBuf::from(&args[i]));
                i += 1;
            }
            // Unknown flag
            arg_str if arg_str.starts_with("--") || arg_str.starts_with('-') => {
                return Err(stack_format_it(
                    "Error: Unknown flag '{}'",
                    &[&arg_str],
                    "Error: Unknown flag",
                ));
            }
            // Non-flag argument (file path)
            _ => {
                if file_path.is_some() {
                    return Err("Error: Multiple file paths specified".to_string());
                }

                // Parse "filename:line" format
                let (file_path_str, line_num) = if let Some(colon_pos) = arg.rfind(':') {
                    let file_part = &arg[..colon_pos];
                    let line_part = &arg[colon_pos + 1..];

                    match line_part.parse::<usize>() {
                        Ok(line) if line > 0 => (file_part.to_string(), Some(line)),
                        _ => (arg.to_string(), None), // Invalid line, treat as filename
                    }
                } else {
                    (arg.to_string(), None)
                };

                file_path = Some(PathBuf::from(file_path_str));
                starting_line = line_num;
                i += 1;
            }
        }
    }

    Ok(ParsedArgs {
        file_path,
        starting_line,
        session_path,
        mode,
    })
}

/// Main entry point - routes between memo mode and full editor mode
///
/// # Purpose
/// Determines which mode to use based on current directory and arguments.
///
/// # Command Line Usage
/// ```text
/// lines                                    # Memo mode (if in home) or prompt
/// lines file.txt                          # Full editor with file
/// lines file.txt:123                      # Full editor, jump to line 123
/// lines --session ./sessions/20250103/    # Full editor with session recovery
/// lines file.txt --session <path>         # Full editor with file and session
/// lines -a file.txt                       # Memo mode (append-only)
/// lines --help                            # Print help
/// lines --version                         # Print version
/// lines --source                          # Extract source code
/// ```
///
/// # Mode Selection Logic
/// 1. If CWD is home directory -> memo mode available
/// 2. Otherwise -> full editor mode (requires file argument)
///
/// # Session Recovery
/// Use `--session <path>` to continue an interrupted editing session.
/// The path can be relative or absolute:
/// - Relative: `lines --session sessions/20250103_143022 file.txt`
/// - Absolute: `lines --session /full/path/to/sessions/20250103_143022 file.txt`
///
/// # Exit Codes
/// - 0: Success
/// - 1: General error
/// - 2: Invalid arguments
fn main() -> Result<(), LinesError> {
    let args: Vec<String> = std::env::args().collect();

    // Parse command line arguments
    let parsed = match parse_arguments(&args) {
        Ok(parsed) => parsed,
        Err(err_msg) => {
            eprintln!("{}", err_msg);
            eprintln!();
            eprintln!("Usage: lines [OPTIONS] [FILE[:LINE]]");
            eprintln!("Options:");
            eprintln!("  -h, --help              Print help information");
            eprintln!("  -v, --version           Print version information");
            eprintln!("  --source                Extract source code");
            eprintln!("  -a, --append FILE       Memo mode (append-only)");
            eprintln!("  -s, --session PATH      Use existing session directory");
            eprintln!();
            eprintln!("Examples:");
            eprintln!("  lines                               # Quick-Edit: new Documents/ file");
            eprintln!("  lines notes.txt                     # Edit / create-&-edit file");
            eprintln!("  lines notes.txt:42                  # Edit file, jump to line 42");
            eprintln!("  lines -a notes.txt                  # Quick-Edit: Memo-Append Mode");
            eprintln!("  lines --session ./sessions/2025../  # Recover session");
            eprintln!("  lines notes.txt --session <path>    # Edit with session");
            std::process::exit(2);
        }
    };

    // Check if we're in home directory
    let in_home = is_in_home_directory()?;

    // Handle special modes that don't start the editor
    match parsed.mode {
        ArgMode::Help => {
            print_help();
            return Ok(());
        }
        ArgMode::Version => {
            buffy_print(
                "Lines-Editor Version: {}",
                &[FormatArg::Str(env!("CARGO_PKG_VERSION"))],
            )?;

            return Ok(());
        }
        ArgMode::Source => {
            // To make a smaller binary, you can remove source-it.
            match handle_sourceit_command("lines_editor", None, SOURCE_FILES) {
                Ok(path) => buffy_print("Source extracted to: {}", &[FormatArg::Path(&path)])?,
                Err(e) => eprintln!("Failed to extract source: {}", e),
            }
            return Ok(());
        }
        ArgMode::AppendMode => {
            // Memo mode (append-only) - requires file path
            if let Some(file_path) = parsed.file_path {
                buffy_print(
                    "Starting memo mode (append-only) with file: {}",
                    &[FormatArg::Path(&file_path)],
                )?;

                return memo_mode_mini_editor_loop(&file_path);
            } else {
                eprintln!("Error: --append flag requires a file path");
                std::process::exit(2);
            }
        }
        ArgMode::Normal => {
            // Continue to normal editor mode logic below
        }
    }

    // Normal editor mode - determine whether to use memo mode or full editor
    match parsed.file_path {
        None => {
            // No file specified
            if in_home {
                // Memo mode: create today's file
                buffy_println("Starting memo mode...", &[])?;
                let original_file_path = get_default_filepath(None)?;
                memo_mode_mini_editor_loop(&original_file_path)
            } else {
                // Full editor mode - prompt for filename in current directory
                buffy_println(
                    "No file specified. Creating new file in current directory.",
                    &[],
                )?;
                let filename = prompt_for_filename()?;
                let current_dir = env::current_dir()?;
                let original_file_path = current_dir.join(filename);

                // Call full editor with session path if provided
                /*
                pub fn lines_full_file_editor(
                    original_file_path: Option<PathBuf>,
                    starting_line: Option<usize>,
                    use_this_session: Option<PathBuf>,
                    state_persists: bool,
                ) -> Result<()> {
                */
                lines_full_file_editor(Some(original_file_path), None, parsed.session_path, false)
            }
        }
        Some(file_path) => {
            // File path provided
            let file_path_str = file_path.to_string_lossy();

            // Check if this is a simple filename in home directory (memo mode)
            if in_home
                && !file_path_str.contains('/')
                && !file_path_str.contains('\\')
                && parsed.session_path.is_none()
            // Only memo mode if no session specified
            {
                buffy_print(
                    "Starting memo mode with custom file: {}",
                    &[FormatArg::Str(&file_path_str)],
                )?;

                let original_file_path = get_default_filepath(Some(&file_path_str))?;
                memo_mode_mini_editor_loop(&original_file_path)
            } else {
                /*
                pub fn lines_full_file_editor(
                    original_file_path: Option<PathBuf>,
                    starting_line: Option<usize>,
                    use_this_session: Option<PathBuf>,
                    state_persists: bool,
                ) -> Result<()> {
                */
                // Full editor mode with file
                lines_full_file_editor(
                    Some(file_path),
                    parsed.starting_line,
                    parsed.session_path,
                    false,
                )
            }
        }
    }
}
