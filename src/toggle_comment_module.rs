//! # toggle_comment_module.rs
//!
//! A simple, safe Rust crate to toggle comment flags in source code files.
//!
//! ## Overview
//! This crate provides thread-safe, memory-efficient utilities to toggle comments on source code
//! without loading entire files into memory. Three operating modes support different use cases:
//! - **Single-line toggle**: Toggle basic comments (`//` or `#`) on one line
//! - **Docstring toggle**: Toggle Rust doc comments (`///`) on one line
//! - **Block comment toggle**: Add/remove block comment markers around line ranges (`/* */` or `"""`)
//! - **Batch operations**: Toggle comments on multiple lines in one pass (max 128 lines)
//!
//! ## Supported Languages & Comment Types
//!
//! ### Double-Slash Comments (`//`)
//! Rust, C, C++, C#, Java, JavaScript, TypeScript, Go, Swift
//! - Extensions: `rs`, `c`, `cpp`, `cc`, `cxx`, `h`, `hpp`, `js`, `ts`, `java`, `go`, `swift`
//!
//! ### Hash Comments (`#`)
//! Python, Shell, Bash, TOML, YAML, Ruby, Perl, R
//! - Extensions: `py`, `sh`, `bash`, `toml`, `yaml`, `yml`, `rb`, `pl`, `r`
//!
//! ### Block Comments (`/* */`)
//! Rust, C, C++, C#, Java, JavaScript, TypeScript, Go, Swift
//! - Supported for same languages as `//`
//!
//! ### Block Comments (`""" """`)
//! Python (triple-quoted strings as docblocks)
//! - Supported for `.py` files
//!
//! ### Rust Documentation (`///`)
//! Rust doc comments (dedicated function)
//! - Supported for `.rs` files
//!
//! ## Operating Modes
//!
//! ### Single-Line Comment Toggle
//! Toggles comment flag at start of line (after leading whitespace):
//!
//! ```text
//! Input:  "    println!(...);"
//! Output: "// println!(...);"
//!
//! Input:  "// println!(...);"
//! Output: "println!(...);"
//! ```
//!
//! Pattern detection: `{0+ spaces}{flag}{1 space}{content}`
//!
//! ### Block Comment Toggle
//! Automatically detects whether to add or remove markers:
//!
//! **Add Mode** (markers not present):
//! ```text
//! Input line 0:   "code line 1"
//! Input line 1:   "code line 2"
//!
//! Output line 0:  "/*"
//! Output line 1:  "code line 1"
//! Output line 2:  "code line 2"
//! Output line 3:  "*/"
//! ```
//!
//! **Remove Mode** (markers present at specified lines):
//! ```text
//! Input line 0:   "/*"
//! Input line 1:   "code line 1"
//! Input line 2:   "code line 2"
//! Input line 3:   "*/"
//!
//! Output line 0:  "code line 1"
//! Output line 1:  "code line 2"
//! ```
//!
//! ### Batch Operations
//! Toggle comments on multiple lines with single backup and file pass:
//! - Max 128 lines per operation
//! - Input order doesn't matter (automatically sorted)
//! - Duplicate lines handled automatically
//! - More efficient than repeated single-line calls
//!
//! ## Safety & Reliability Features
//!
//! ### Memory Safety
//! - **No heap allocation during processing**: Fixed pre-allocated buffers only
//! - **Bounded operations**: All loops have upper limits to prevent hangs
//! - **Line length limits**: Rejects lines exceeding 1MB (MAX_LINE_LENGTH)
//! - **Batch size limits**: Max 128 lines per batch operation (MAX_BATCH_LINES)
//!
//! ### File Safety
//! - **Atomic operations**: Original file only replaced on complete success
//! - **Single backup**: Creates `backup_toggle_comment_{filename}` before modifications
//! - **Temp files**: Uses process-ID in temp filename to avoid collisions
//! - **Preserve file endings**: Maintains original line endings (LF, CRLF, or none)
//!
//! ### Error Handling
//! - **All errors returned as `Result`**: No panics in production code
//! - **Specific error types**: `ToggleError` enum provides detailed failure reasons
//! - **I/O operation tracking**: Errors specify which operation failed (open, read, write, etc.)
//! - **Recoverable**: Failed operations leave backups intact; original file untouched
//!
//! ## Usage Examples
//!
//! ### Toggle Single Line
//! ```rust
//! use toggle_basic_singleline_comment::toggle_basic_singleline_comment;
//!
//! match toggle_basic_singleline_comment("./src/main.rs", 5) {
//!     Ok(()) => println!("Line 5 toggled"),
//!     Err(e) => eprintln!("Failed: {}", e),
//! }
//! ```
//!
//! ### Toggle Rust Docstring
//! ```rust
//! use toggle_basic_singleline_comment::toggle_rust_docstring_singleline_comment;
//!
//! match toggle_rust_docstring_singleline_comment("./src/lib.rs", 10) {
//!     Ok(()) => println!("Docstring toggled"),
//!     Err(e) => eprintln!("Failed: {}", e),
//! }
//! ```
//!
//! ### Toggle Block Comment
//! ```rust
//! use toggle_basic_singleline_comment::toggle_block_comment;
//!
//! // Toggle block markers around lines 5-10
//! match toggle_block_comment("./src/lib.rs", 5, 10) {
//!     Ok(()) => println!("Block comment toggled"),
//!     Err(e) => eprintln!("Failed: {}", e),
//! }
//! ```
//!
//! ### Batch Toggle Multiple Lines
//! ```rust
//! use toggle_basic_singleline_comment::toggle_multiple_basic_comments;
//!
//! let lines = [5, 10, 15, 20];
//! match toggle_multiple_basic_comments("./src/main.rs", &lines) {
//!     Ok(()) => println!("All 4 lines toggled in one pass"),
//!     Err(e) => eprintln!("Failed: {}", e),
//! }
//! ```
//!
//! ## Limitations & Edge Cases
//!
//! ### Limitations
//! - **Max file line length**: 1,000,000 bytes per line (rejects longer lines)
//! - **Max batch lines**: 128 lines per batch operation
//! - **Extension-based**: Comment type determined by file extension (case-insensitive)
//! - **Simple pattern matching**: Only detects `{spaces}{flag}{space}` pattern
//! - **Line-based**: Does not modify content within lines, only toggle markers
//!
//! ### Supported Edge Cases
//! - ✓ Empty lines: `\n` → `// \n` (blank comments allowed)
//! - ✓ Whitespace-only lines: `    \n` → `//     \n` (preserves internal spaces)
//! - ✓ Tab indentation: Treated as any other character
//! - ✓ No newline at EOF: Last line without `\n` preserved as-is
//! - ✓ Mixed line endings: CRLF (`\r\n`) and LF (`\n`) both preserved
//! - ✓ Last line of file: Can be toggled like any other line
//! - ✓ Only whitespace + flag: `    // \n` → `\n` (removes all)
//!
//! ### Not Supported (By Design)
//! - ✗ Inline comments: `code // comment` on same line (flag must start line)
//! - ✗ Partial line modification: Comments must be at line start
//! - ✗ Nested block comments: Detection assumes non-nested markers
//! - ✗ Smart content parsing: No language-aware syntax analysis
//! - ✗ Format preservation: Line formatting outside flag not preserved/modified
//!
//! ## Implementation Notes
//!
//! ### Design Principles
//! - **Stateless**: Each operation independent; no persistent state
//! - **Simple**: Narrow scope, single responsibility per function
//! - **Safe**: All operations atomic and recoverable
//! - **Efficient**: Single file pass, bounded buffers, minimal allocations
//! - **Communicable**: Clear errors, explicit limits, documented behavior
//!
//! ### Internal Details
//! - Reads source file line-by-line with 8KB buffer
//! - Maintains sorted array of target lines for O(1) lookup in batch operations
//! - Writes to temp file in same directory; replaces original only on success
//! - Backup file overwritten (not versioned) on each operation
//! - All path operations use canonical absolute paths
//!
//! ## Error Handling
//!
//! The `ToggleError` enum provides specific error information:
//! - `FileNotFound`: Specified file does not exist
//! - `NoExtension`: File has no extension (cannot determine comment type)
//! - `UnsupportedExtension`: Extension not recognized for any comment mode
//! - `LineNotFound { requested, file_lines }`: Target line beyond EOF
//! - `IoError(operation)`: I/O failure during backup, read, write, etc.
//! - `PathError`: Filesystem path manipulation failed
//! - `LineTooLong { line_number, length }`: Line exceeds 1MB limit
//! - `InconsistentBlockMarkers`: Only one block marker found (not both)
//! - `InvalidLineRange`: Block range invalid (start >= end)
//!
//! ## Performance Characteristics
//!
//! | Operation | Time Complexity | Space |
//! |-----------|-----------------|-------|
//! | Single-line toggle | O(n) | O(1) |
//! | Batch toggle (128 lines) | O(n log m) | O(1) |
//! | Block toggle | O(n) | O(1) |
//!
//! Where `n` = file lines, `m` = batch size ≤ 128
//!
//! All operations use constant stack space regardless of file size.
//!
//! ## Safety & Policy
//! - Never loads entire file into memory
//! - Pre-allocated fixed buffers only (no heap dynamic allocation during processing)
//! - Creates one static (not timestamped or versioned or unique) backup before any modifications
//!   Simple create/overwrite upon each operation
//! - Atomic file replacement (original only replaced on success)
//! - All errors returned as Result - no panics in production code
//! - Not swiss-army-knife functions: keep it simple, maintainable, communicable, plannable
//! - See more rules in comments below
//! - No event will result in a panic-crash, everything is handled.

/*
# Notes:

```
////////////
```
will be ignored, that is by design.

the goal is to be able to turn-on, turn off, commments
reliably in this system, not to micro-manage the rest of the universe.
*/

/*
# Example main function:

```rust
//! # main.rs
//!
//! Command-line interface for toggle_basic_singleline_comment crate
//!
//! # Usage Modes
//!
//! ## Basic single-line toggle (auto-detect comment type from extension)
//! ```text
//! toggle_comment <file_path> <line_number>
//! ```
//!
//! ## Rust docstring single-line toggle (///)
//! ```text
//! toggle_comment --rust-doc-string <file_path> <line_number>
//! ```
//!
//! ## Block comment toggle (insert/remove /* */ or """ """)
//! ```text
//! toggle_comment --block <file_path> <start_line> <end_line>
//! ```
//!
//! ## Batch toggle - basic comments
//! ```text
//! toggle_comment --list-basic <file_path> <line1> <line2> ... <lineN>
//! ```
//!
//! ## Batch toggle - rust docstrings
//! ```text
//! toggle_comment --list-docstring <file_path> <line1> <line2> ... <lineN>
//! ```

use std::env;
use std::process;
mod toggle_comment_module;
use toggle_comment_module::{
    IndentError, ToggleError, indent_line, indent_range, toggle_basic_singleline_comment,
    toggle_block_comment, toggle_multiple_basic_comments, toggle_multiple_singline_docstrings,
    toggle_rust_docstring_singleline_comment, unindent_line, unindent_range,
};

/// Maximum number of lines that can be toggled in batch mode
/// Prevents unbounded memory usage while still being practical
const MAX_BATCH_LINES: usize = 512;

/// Print comprehensive usage information and exit
fn print_usage() {
    eprintln!("toggle_comment - Toggle comments in source code files");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  toggle_comment <file_path> <line_number>");
    eprintln!("  toggle_comment --rust-doc-string <file_path> <line_number>");
    eprintln!("  toggle_comment --block <file_path> <start_line> <end_line>");
    eprintln!("  toggle_comment --list-basic <file_path> <line1> <line2> ...");
    eprintln!("  toggle_comment --list-docstring <file_path> <line1> <line2> ...");
    eprintln!("  toggle_comment --indent <file_path> <line_number>");
    eprintln!("  toggle_comment --unindent <file_path> <line_number>");
    eprintln!("  toggle_comment --indent-range <file_path> <start_line> <end_line>");
    eprintln!("  toggle_comment --unindent-range <file_path> <start_line> <end_line>");
    eprintln!();

    eprintln!("MODES:");
    eprintln!("  Basic mode:");
    eprintln!("    Auto-detects comment type from file extension");
    eprintln!("    Toggles // or # on a single line");
    eprintln!();
    eprintln!("  --rust-doc-string:");
    eprintln!("    Toggles Rust documentation comment (///) on a single line");
    eprintln!();
    eprintln!("  --block:");
    eprintln!("    Toggles block comments around a range of lines");
    eprintln!("    Inserts /* before start_line and */
 after end_line (or removes them)");
    eprintln!("    For Python: uses \"\"\" instead");
    eprintln!();
    eprintln!("  --list-basic:");
    eprintln!("    Toggle basic comments on multiple lines in one operation");
    eprintln!("    Maximum {} lines per batch", MAX_BATCH_LINES);
    eprintln!();
    eprintln!("  --list-docstring:");
    eprintln!("    Toggle /// comments on multiple lines in one operation");
    eprintln!("    Maximum {} lines per batch", MAX_BATCH_LINES);
    eprintln!();
    eprintln!("  --indent:");
    eprintln!("    Add 4 spaces to the start of a line");
    eprintln!();
    eprintln!("  --unindent:");
    eprintln!("    Remove up to 4 spaces from the start of a line");
    eprintln!("  --indent-range:");
    eprintln!("    Add 4 spaces to the start of multiple lines (inclusive range)");
    eprintln!();
    eprintln!("  --unindent-range:");
    eprintln!("    Remove up to 4 spaces from multiple lines (inclusive range)");
    eprintln!();

    eprintln!("ARGUMENTS:");
    eprintln!("  file_path    - Path to source code file");
    eprintln!("  line_number  - Line number to toggle (zero-indexed)");
    eprintln!("  start_line   - First line of range/block (zero-indexed, inclusive)");
    eprintln!("  end_line     - Last line of range/block (zero-indexed, inclusive)");
    eprintln!();

    eprintln!("EXAMPLES:");
    eprintln!("  toggle_comment hello_world.py 5");
    eprintln!("  toggle_comment --rust-doc-string hello_world.py 10");
    eprintln!("  toggle_comment --block hello_world.rs 5 15");
    eprintln!("  toggle_comment --list-basic hello_world.py 1 10 12");
    eprintln!("  toggle_comment --list-docstring hello_world.toml 1 2 3");
    eprintln!("  toggle_comment --indent hello_world.py 10");
    eprintln!("  toggle_comment --unindent hello_world.py 10");
    eprintln!("  toggle_comment --indent-range hello_world.py 10 12");
    eprintln!("  toggle_comment --unindent-range hello_world.py 10 12");
    eprintln!();

    eprintln!("SUPPORTED EXTENSIONS:");
    eprintln!("  //  : rs, c, cpp, js, ts, java, go, swift");
    eprintln!("  #   : py, sh, toml, yaml, rb, pl, r");
    eprintln!();

    eprintln!("EXIT CODES:");
    eprintln!("  0 - Success");
    eprintln!("  1 - Invalid arguments");
    eprintln!("  2 - File not found");
    eprintln!("  3 - No extension");
    eprintln!("  4 - Unsupported extension");
    eprintln!("  5 - Line not found");
    eprintln!("  6 - I/O error");
    eprintln!("  7 - Path error");
    eprintln!("  8 - Line too long");
}

/// Parse a line number argument, returning error on invalid input
///
/// # Arguments
/// * `arg` - String slice to parse
/// * `arg_name` - Name of argument for error messages
///
/// # Returns
/// * `Ok(usize)` - Successfully parsed line number
/// * `Err(())` - Parse failed (error already printed to stderr)
fn parse_line_number(arg: &str, arg_name: &str) -> Result<usize, ()> {
    match arg.parse::<usize>() {
        Ok(n) => Ok(n),
        Err(_) => {
            eprintln!("Error: {} must be a valid integer", arg_name);
            eprintln!();
            Err(())
        }
    }
}

/// Parse multiple line number arguments into a fixed-size array
///
/// # Arguments
/// * `args` - Slice of string arguments to parse
///
/// # Returns
/// * `Ok((count, array))` - Successfully parsed line numbers
/// * `Err(())` - Parse failed or too many lines
///
/// # Safety
/// - Bounded to MAX_BATCH_LINES (512)
/// - Pre-allocated fixed array on stack
fn parse_line_list(args: &[String]) -> Result<(usize, [usize; MAX_BATCH_LINES]), ()> {
// Check bounds
    if args.is_empty() {
        eprintln!("Error: No line numbers provided");
        return Err(());
    }

    if args.len() > MAX_BATCH_LINES {
        eprintln!("Error: Too many lines (max {})", MAX_BATCH_LINES);
        return Err(());
    }

// Pre-allocate fixed array
    let mut line_array: [usize; MAX_BATCH_LINES] = [0; MAX_BATCH_LINES];
    let count = args.len();

// Parse each line number
    for (i, arg) in args.iter().enumerate() {
        match arg.parse::<usize>() {
            Ok(n) => line_array[i] = n,
            Err(_) => {
                eprintln!("Error: Invalid line number: {}", arg);
                return Err(());
            }
        }
    }

    Ok((count, line_array))
}

/// Convert ToggleError to exit code
///
/// # Arguments
/// * `error` - The error to convert
///
/// # Returns
/// * Exit code (2-10)
fn error_to_exit_code(error: ToggleError) -> i32 {
    match error {
        ToggleError::FileNotFound => 2,
        ToggleError::NoExtension => 3,
        ToggleError::UnsupportedExtension => 4,
        ToggleError::LineNotFound { .. } => 5,
        ToggleError::IoError(_) => 6,
        ToggleError::PathError => 7,
        ToggleError::LineTooLong { .. } => 8,
        ToggleError::InconsistentBlockMarkers => 9,
        ToggleError::InvalidLineRange => 10,
    }
}

/// Convert IndentError to exit code
///
/// # Arguments
/// * `error` - The error to convert
///
/// # Returns
/// * Exit code (2-10, same mapping as ToggleError where applicable)
fn indent_error_to_exit_code(error: IndentError) -> i32 {
    match error {
        IndentError::FileNotFound => 2,
        IndentError::LineNotFound { .. } => 5,
        IndentError::IoError(_) => 6,
        IndentError::PathError => 7,
        IndentError::LineTooLong { .. } => 8,
        IndentError::InvalidLineRange => 10,
    }
}

/// Execute indent on a single line
fn execute_indent(file_path: &str, line_number: usize) -> i32 {
    match indent_line(file_path, line_number) {
        Ok(()) => {
            println!("Successfully indented line {}", line_number);
            0
        }
        Err(e) => {
            eprintln!("Error indenting {}: {}", file_path, e);
            indent_error_to_exit_code(e)
        }
    }
}

/// Execute unindent on a single line
fn execute_unindent(file_path: &str, line_number: usize) -> i32 {
    match unindent_line(file_path, line_number) {
        Ok(()) => {
            println!("Successfully unindented line {}", line_number);
            0
        }
        Err(e) => {
            eprintln!("Error unindenting {}: {}", file_path, e);
            indent_error_to_exit_code(e)
        }
    }
}

/// Execute basic single-line comment toggle
fn execute_basic_toggle(file_path: &str, line_number: usize) -> i32 {
    match toggle_basic_singleline_comment(file_path, line_number) {
        Ok(()) => {
            println!("Successfully toggled comment on line {}", line_number);
            0
        }
        Err(e) => {
            eprintln!("Error toggling {}: {}", file_path, e);
            error_to_exit_code(e)
        }
    }
}

/// Execute Rust docstring single-line comment toggle
fn execute_docstring_toggle(file_path: &str, line_number: usize) -> i32 {
    match toggle_rust_docstring_singleline_comment(file_path, line_number) {
        Ok(()) => {
            println!("Successfully toggled docstring on line {}", line_number);
            0
        }
        Err(e) => {
            eprintln!("Error toggling docstring {}: {}", file_path, e);
            error_to_exit_code(e)
        }
    }
}

/// Execute block comment toggle
fn execute_block_toggle(file_path: &str, start_line: usize, end_line: usize) -> i32 {
    match toggle_block_comment(file_path, start_line, end_line) {
        Ok(()) => {
            println!(
                "Successfully toggled block comment (lines {}-{})",
                start_line, end_line
            );
            0
        }
        Err(e) => {
            eprintln!("Error toggling block {}: {}", file_path, e);
            error_to_exit_code(e)
        }
    }
}

/// Execute indent on a range of lines
fn execute_indent_range(file_path: &str, start_line: usize, end_line: usize) -> i32 {
    match indent_range(file_path, start_line, end_line) {
        Ok(()) => {
            println!("Successfully indented lines {} to {}", start_line, end_line);
            0
        }
        Err(e) => {
            eprintln!("Error indenting range {}: {}", file_path, e);
            indent_error_to_exit_code(e)
        }
    }
}

/// Execute unindent on a range of lines
fn execute_unindent_range(file_path: &str, start_line: usize, end_line: usize) -> i32 {
    match unindent_range(file_path, start_line, end_line) {
        Ok(()) => {
            println!(
                "Successfully unindented lines {} to {}",
                start_line, end_line
            );
            0
        }
        Err(e) => {
            eprintln!("Error unindenting range {}: {}", file_path, e);
            indent_error_to_exit_code(e)
        }
    }
}

/// Execute batch toggle - basic comments
fn execute_batch_toggle_standard(
    file_path: &str,
    count: usize,
    lines: &[usize; MAX_BATCH_LINES],
) -> i32 {
// Use only the valid portion of array
    let line_slice = &lines[..count];

    match toggle_multiple_basic_comments(file_path, line_slice) {
        Ok(()) => {
            println!("Successfully toggled {} lines", count);
            0
        }
        Err(e) => {
            eprintln!("Error batch toggling {}: {}", file_path, e);
            error_to_exit_code(e)
        }
    }
}

/// Execute batch toggle - docstrings
fn execute_batch_toggle_docstring(
    file_path: &str,
    count: usize,
    lines: &[usize; MAX_BATCH_LINES],
) -> i32 {
// Use only the valid portion of array
    let line_slice = &lines[..count];

    match toggle_multiple_singline_docstrings(file_path, line_slice) {
        Ok(()) => {
            println!("Successfully toggled {} docstrings", count);
            0
        }
        Err(e) => {
            eprintln!("Error batch toggling docstrings {}: {}", file_path, e);
            error_to_exit_code(e)
        }
    }
}

fn main() {
// Collect command line arguments
    let args: Vec<String> = env::args().collect();

// Minimum: program name + at least 2 args
    if args.len() < 3 {
        eprintln!("Error: Invalid number of arguments");
        eprintln!();
        print_usage();
        process::exit(1);
    }

// Determine mode based on first argument
    let exit_code = if args[1].starts_with("--") {
// Flag-based mode
        let flag = &args[1];

        match flag.as_str() {
            "--rust-doc-string" => {
// Expect: --rust-doc-string <file> <line>
                if args.len() != 4 {
                    eprintln!("Error: --rust-doc-string requires <file_path> <line_number>");
                    eprintln!();
                    print_usage();
                    process::exit(1);
                }

                let file_path = &args[2];
                let line_number = match parse_line_number(&args[3], "line_number") {
                    Ok(n) => n,
                    Err(_) => {
                        print_usage();
                        process::exit(1);
                    }
                };

                execute_docstring_toggle(file_path, line_number)
            }

            "--block" => {
// Expect: --block <file> <start_line> <end_line>
                if args.len() != 5 {
                    eprintln!("Error: --block requires <file_path> <start_line> <end_line>");
                    eprintln!();
                    print_usage();
                    process::exit(1);
                }

                let file_path = &args[2];
                let start_line = match parse_line_number(&args[3], "start_line") {
                    Ok(n) => n,
                    Err(_) => {
                        print_usage();
                        process::exit(1);
                    }
                };
                let end_line = match parse_line_number(&args[4], "end_line") {
                    Ok(n) => n,
                    Err(_) => {
                        print_usage();
                        process::exit(1);
                    }
                };

// Validate line order
                if start_line >= end_line {
                    eprintln!("Error: start_line must be less than end_line");
                    process::exit(1);
                }

                execute_block_toggle(file_path, start_line, end_line)
            }

            "--list-basic" => {
// Expect: --list-basic <file> <line1> <line2> ...
                if args.len() < 4 {
                    eprintln!("Error: --list-basic requires <file_path> <line1> [line2] ...");
                    eprintln!();
                    print_usage();
                    process::exit(1);
                }

                let file_path = &args[2];
                let line_args = &args[3..];

                let (count, line_array) = match parse_line_list(line_args) {
                    Ok(result) => result,
                    Err(_) => {
                        print_usage();
                        process::exit(1);
                    }
                };

                execute_batch_toggle_standard(file_path, count, &line_array)
            }

            "--list-docstring" => {
// Expect: --list-docstring <file> <line1> <line2> ...
                if args.len() < 4 {
                    eprintln!("Error: --list-docstring requires <file_path> <line1> [line2] ...");
                    eprintln!();
                    print_usage();
                    process::exit(1);
                }

                let file_path = &args[2];
                let line_args = &args[3..];

                let (count, line_array) = match parse_line_list(line_args) {
                    Ok(result) => result,
                    Err(_) => {
                        print_usage();
                        process::exit(1);
                    }
                };

                execute_batch_toggle_docstring(file_path, count, &line_array)
            }
            "--indent" => {
// Expect: --indent <file> <line>
                if args.len() != 4 {
                    eprintln!("Error: --indent requires <file_path> <line_number>");
                    eprintln!();
                    print_usage();
                    process::exit(1);
                }

                let file_path = &args[2];
                let line_number = match parse_line_number(&args[3], "line_number") {
                    Ok(n) => n,
                    Err(_) => {
                        print_usage();
                        process::exit(1);
                    }
                };

                execute_indent(file_path, line_number)
            }

            "--unindent" => {
// Expect: --unindent <file> <line>
                if args.len() != 4 {
                    eprintln!("Error: --unindent requires <file_path> <line_number>");
                    eprintln!();
                    print_usage();
                    process::exit(1);
                }

                let file_path = &args[2];
                let line_number = match parse_line_number(&args[3], "line_number") {
                    Ok(n) => n,
                    Err(_) => {
                        print_usage();
                        process::exit(1);
                    }
                };

                execute_unindent(file_path, line_number)
            }
            "--indent-range" => {
// Expect: --indent-range <file> <start_line> <end_line>
                if args.len() != 5 {
                    eprintln!("Error: --indent-range requires <file_path> <start_line> <end_line>");
                    eprintln!();
                    print_usage();
                    process::exit(1);
                }

                let file_path = &args[2];
                let start_line = match parse_line_number(&args[3], "start_line") {
                    Ok(n) => n,
                    Err(_) => {
                        print_usage();
                        process::exit(1);
                    }
                };
                let end_line = match parse_line_number(&args[4], "end_line") {
                    Ok(n) => n,
                    Err(_) => {
                        print_usage();
                        process::exit(1);
                    }
                };

// Validate line order
                if start_line > end_line {
                    eprintln!("Error: start_line must be less than or equal to end_line");
                    process::exit(1);
                }

                execute_indent_range(file_path, start_line, end_line)
            }

            "--unindent-range" => {
// Expect: --unindent-range <file> <start_line> <end_line>
                if args.len() != 5 {
                    eprintln!(
                        "Error: --unindent-range requires <file_path> <start_line> <end_line>"
                    );
                    eprintln!();
                    print_usage();
                    process::exit(1);
                }

                let file_path = &args[2];
                let start_line = match parse_line_number(&args[3], "start_line") {
                    Ok(n) => n,
                    Err(_) => {
                        print_usage();
                        process::exit(1);
                    }
                };
                let end_line = match parse_line_number(&args[4], "end_line") {
                    Ok(n) => n,
                    Err(_) => {
                        print_usage();
                        process::exit(1);
                    }
                };

// Validate line order
                if start_line > end_line {
                    eprintln!("Error: start_line must be less than or equal to end_line");
                    process::exit(1);
                }

                execute_unindent_range(file_path, start_line, end_line)
            }
            _ => {
                eprintln!("Error: Unknown flag: {}", flag);
                eprintln!();
                print_usage();
                process::exit(1);
            }
        }
    } else {
// Basic mode: <file> <line>
        if args.len() != 3 {
            eprintln!("Error: Basic mode requires <file_path> <line_number>");
            eprintln!();
            print_usage();
            process::exit(1);
        }

        let file_path = &args[1];
        let line_number = match parse_line_number(&args[2], "line_number") {
            Ok(n) => n,
            Err(_) => {
                print_usage();
                process::exit(1);
            }
        };

        execute_basic_toggle(file_path, line_number)
    };

// Exit with appropriate code
    process::exit(exit_code);
}

```

*/

/*

# Rust rules:
-Always best practice.
-Always extensive doc strings.
-Always comments.
- Always cargo tests (where possible).
- Never remove documentation.
- Always clear, meaningful, unique names (e.g. variables, functions).
- Always absolute file paths.
- Always error handling.
- Never unsafe code.
- Never use unwrap.

- Load what is needed when it is needed: Do not ever load a whole file, rarely load a whole anything. increment and load only what is required pragmatically.

- Always defensive best practice:
- Always error handling: everything will fail at some point, if only because of cosmic-ray bit-flips (which are actually common), there must always be fail-safe error handling.

Safety, reliability, maintainability, fail-safe, communication-documentation, are the goals.

## No third party libraries (or very strictly avoid third party libraries where possible).

## Every part of code will eventually fail if only due to hardware failure, power supply failures, hard radiation bit flips, security attacks, etc. Every failure must be handled smoothly: let it fail and move on.

## Rule of Thumb, ideals not absolute rules: Follow NASA's 'Power of 10 rules' where possible and sensible (updated for 2025 and Rust):
1. no unsafe stuff:
- no recursion
- no goto
- no pointers
- no preprocessor

2. upper bound on all normal-loops, failsafe for all always-loops

3. Pre-allocate all memory (no dynamic memory allocation)

4. Clear function scope and Data Ownership: Part of having a function be 'focused' means knowing if the function is in scope. Functions should be neither swiss-army-knife functions that do too many things, nor scope-less micro-functions that may be doing something that should not be done. Many functions should have a narrow focus and a short length, but definition of actual-project scope functionality must be explicit. Replacing one long clear in-scope function with 50 scope-agnostic generic sub-functions with no clear way of telling if they are in scope or how they interact (e.g. hidden indirect recursion) is unsafe. Rust's ownership and borrowing rules focus on Data ownership and hidden dependencies, making it even less appropriate to scatter borrowing and ownership over a spray of microfunctions purely for the ideology of turning every operation into a microfunction just for the sake of doing so. (See more in rule 9.)

5. Defensive programming: debug-assert, test-assert, prod safely check & handle, not 'assert!' panic
For production-release code:
1. check and handle without panic/halt in production
2. return result (such as Result<T, E>) and smoothly handle errors (not halt-panic stopping the application): no assert!() outside of test-only code
3. test assert: use #[cfg(test)] assert!() to test production binaries (not in prod)
4. debug assert: use debug_assert to test debug builds/runs (not in prod)
5. use defensive programming with recovery of all issues at all times
- use cargo tests
- use debug_asserts
- do not leave assertions in production code.
- use no-panic error handling
- use Option
- use enums and structs
- check bounds
- check returns
- note: a test-flagged assert can test a production release build (whereas debug_assert cannot); cargo test --release
```
#[cfg(test)]
assert!(
```

e.g.
# "Assert & Catch-Handle" 3-part System

// template/example for check/assert format
//    =================================================
// // Debug-Assert, Test-Asset, Production-Catch-Handle
//    =================================================
// This is not included in production builds
// assert: only when running in a debug-build: will panic
debug_assert!(
    INFOBAR_MESSAGE_BUFFER_SIZE > 0,
    "Info bar buffer must have non-zero capacity"
);
// This is not included in production builds
// assert: only when running cargo test: will panic
#[cfg(test)]
assert!(
    INFOBAR_MESSAGE_BUFFER_SIZE > 0,
    "Info bar buffer must have non-zero capacity"
);
// Catch & Handle without panic in production
// This IS included in production to safe-catch
if !INFOBAR_MESSAGE_BUFFER_SIZE == 0 {
    // state.set_info_bar_message("Config error");
    return Err(LinesError::GeneralAssertionCatchViolation(
        "zero buffer size error".into(),
    ));
}


Avoid heap for error messages and for all things.
   Is heap used because that is THE best way
   the most secure, the most efficient, proper separate of debug testing vs. secure production code?
   Or is heap used because "it's future dev's problem, let's party"
   Can we use  heap in debug/test modes/builds only?
   The lack of clarity on this is not acceptable.
   Production software must not be a shrug punt.
   Is debug information being included in production builds?
   That is NOT supposed to happen.

This is central to the question about testing vs. a pedantic ban on conditional compilation; not putting full traceback insecurity into production code is not a different operational process logic tree for process operations.

Just like with the pedantic "a loops being bounded" rule, there is a fundamental exception: always-on loops must be the opposite.
With conditional compilations: code NEVER to EVER be in production-builds MUST be "conditionally" excluded. This is not an OS condition or a hardware condition. This is an 'unsafe-testing or not' condition.

Error messages and error outcomes in 'production' 'release' (real-use, not debug/testing) must not ever contain any information that could be a security vulnerability or attack surface. Failing to remove debugging inspection is a major category of security and hygiene problems.

Security: Error messages in production must NOT contain:
- File paths (can reveal system structure)
- File contents
- environment variables
- data
- Internal implementation details

Production output following an error must be managed and defined, not not open to whatever some api or OS call wants to dump out.

6. Manage ownership and borrowing

7. Manage return values:
- use null-void return values
- check non-void-null returns

8. Navigate debugging and testing on the one hand and not-dangerous conditional compilation on the other hand

9. Communicate:
- use doc strings, use comments,
- Document use-cases, edge-cases, and policies (These are project specific and cannot be telepathed from generic micro-function code. When a Mars satellite failed because one team used SI-metric units and another team did not, that problem could not have been detected by looking at, and auditing, any individual function in isolation without documentation. Breaking a process into innumerable undocumented micro-functions can make scope and policy impossible to track. To paraphrase Jack Welch: "The most dangerous thing in the world is a flawless operation that should never have been done in the first place.")

10. Use state-less operations when possible:
- a seemingly invisibly small increase in state often completely destroys projects
- expanding state destroys projects with unmaintainable over-reach

*/

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

/// Maximum bytes to scan in a line when looking for comment pattern
/// This prevents unbounded operations on malformed files
const MAX_SCAN_BYTES: usize = 64;

/// Buffer size for file I/O operations - pre-allocated, stack-friendly size
const IO_BUFFER_SIZE: usize = 8192;

/// Maximum line length we'll process - safety bound
const MAX_LINE_LENGTH: usize = 1_000_000; // 64KB per line max

/// Errors that can occur during comment toggling operations
///
/// All variants are Copy - no heap allocation, no string storage.
/// Caller provides context (file paths, etc.) they already have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleError {
    /// The specified file was not found
    FileNotFound,

    /// File has no extension
    NoExtension,

    /// File extension not recognized/supported
    UnsupportedExtension,

    /// The requested line index exceeds the file's line count
    LineNotFound { requested: usize, file_lines: usize },

    /// I/O operation failed
    IoError(IoOperation),

    /// Path conversion or manipulation error
    PathError,

    /// Line exceeds maximum safe length
    LineTooLong { line_number: usize, length: usize },

    /// Block comment markers are inconsistent (only one present)
    InconsistentBlockMarkers,

    /// Invalid line range for block comment
    InvalidLineRange,
}

/// Specific I/O operations that can fail
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoOperation {
    /// Creating backup file
    Backup,

    /// Opening source file for reading
    Open,

    /// Creating temporary/destination file
    Create,

    /// Reading line from file
    Read,

    /// Writing line to file
    Write,

    /// Flushing write buffer
    Flush,

    /// Replacing original file with modified version
    Replace,
}

impl std::fmt::Display for ToggleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToggleError::FileNotFound => write!(f, "File not found"),
            ToggleError::NoExtension => write!(f, "No file extension"),
            ToggleError::UnsupportedExtension => write!(f, "Unsupported extension"),
            ToggleError::LineNotFound {
                requested,
                file_lines,
            } => {
                write!(
                    f,
                    "Line {} not found (file has {} lines)",
                    requested, file_lines
                )
            }
            ToggleError::IoError(op) => write!(f, "IO error: {:?}", op),
            ToggleError::PathError => write!(f, "Path error"),
            ToggleError::LineTooLong {
                line_number,
                length,
            } => {
                write!(f, "Line {} too long: {} bytes", line_number, length)
            }
            ToggleError::InconsistentBlockMarkers => {
                write!(f, "Inconsistent block markers (only one found)")
            }
            ToggleError::InvalidLineRange => {
                write!(f, "Invalid line range")
            }
        }
    }
}

impl std::error::Error for ToggleError {}

/// Comment flag type for different language syntaxes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommentFlag {
    /// Tripple Slash for Rust-Docstrings
    TripppleSlash,

    /// Double-slash comments (Rust, C, C++, JavaScript, etc.)
    DoubleSlash,

    /// Hash/pound comments (Python, Shell, TOML, etc.)
    Hash,
}

impl CommentFlag {
    /// Get the byte slice representation of the comment flag
    fn as_bytes(&self) -> &'static [u8] {
        match self {
            CommentFlag::TripppleSlash => b"///",
            CommentFlag::DoubleSlash => b"//",
            CommentFlag::Hash => b"#",
        }
    }

    /// Get the string representation of the comment flag
    fn as_str(&self) -> &'static str {
        match self {
            CommentFlag::TripppleSlash => "///",
            CommentFlag::DoubleSlash => "//",
            CommentFlag::Hash => "#",
        }
    }
}

/// Determine comment flag based on file extension
///
/// # Arguments
/// * `extension` - File extension without the dot (e.g., "rs", "py")
///
/// # Returns
/// * `Some(CommentFlag)` if extension is supported
/// * `None` if extension is not recognized
///
/// # Supported Extensions
/// - `//` : rs, c, cpp, cc, cxx, h, hpp, js, ts, java, go, swift
/// - `#`  : py, sh, bash, toml, yaml, yml, rb, pl, r
fn determine_comment_flag(extension: &str) -> Option<CommentFlag> {
    match extension.to_lowercase().as_str() {
        // Double-slash languages
        "rs" | "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" | "js" | "ts" | "java" | "go" | "swift" => {
            Some(CommentFlag::DoubleSlash)
        }

        // Hash languages
        "py" | "sh" | "bash" | "toml" | "yaml" | "yml" | "rb" | "pl" | "r" => {
            Some(CommentFlag::Hash)
        }

        // Unknown extension
        _ => None,
    }
}

/// Analyze a line to determine if it has a comment flag that should be removed
///
/// Scans the first MAX_SCAN_BYTES of the line looking for pattern:
/// `{0+ spaces}{comment_flag}{1 space}`
///
/// # Arguments
/// * `line_bytes` - The line content as bytes (without newline)
/// * `flag` - The comment flag to search for
///
/// # Returns
/// * `Some(remove_count)` - Number of bytes to skip if pattern found (flag + space)
/// * `None` - If pattern not found in first MAX_SCAN_BYTES
///
/// # Safety
/// - Bounded scan (max MAX_SCAN_BYTES bytes)
/// - No allocations
/// - Handles malformed UTF-8 safely (works on bytes)
fn should_remove_comment(line_bytes: &[u8], flag: CommentFlag) -> Option<usize> {
    let flag_bytes = flag.as_bytes();
    let flag_len = flag_bytes.len();

    // Scan limit: don't go beyond line length or MAX_SCAN_BYTES
    let scan_limit = std::cmp::min(line_bytes.len(), MAX_SCAN_BYTES);

    // Track position in line
    let mut pos = 0;

    // Skip leading spaces - bounded loop with upper limit
    let mut spaces_skipped = 0;
    while pos < scan_limit && line_bytes[pos] == b' ' {
        pos += 1;
        spaces_skipped += 1;

        // Safety: prevent infinite loop on pathological input
        if spaces_skipped > MAX_SCAN_BYTES {
            return None;
        }
    }

    // Check if we have room for flag + space
    if pos + flag_len + 1 > scan_limit {
        return None;
    }

    // Check if next bytes match the flag
    if &line_bytes[pos..pos + flag_len] != flag_bytes {
        return None;
    }

    // Check if next byte after flag is a space
    if line_bytes[pos + flag_len] != b' ' {
        return None;
    }

    // Pattern found: return how many bytes to skip (flag + one space)
    Some(flag_len + 1)
}
/// Toggle Rust documentation comment (///) on a specific line
///
/// # Overview
/// Operates identically to `toggle_basic_singleline_comment` but uses
/// the Rust documentation comment marker `///` instead of determining
/// the comment type from file extension.
///
/// # Arguments
/// * `file_path` - Path to the source file
/// * `row_line_zeroindex` - Zero-indexed line number to toggle
///
/// # Returns
/// * `Ok(())` - Comment toggled successfully
/// * `Err(ToggleError)` - Specific error code
///
/// # Example
/// ```no_run
/// use toggle_basic_singleline_comment::toggle_rust_docstring_singleline_comment;
///
/// match toggle_rust_docstring_singleline_comment("./src/lib.rs", 0) {
///     Ok(()) => println!("Docstring toggled"),
///     Err(e) => eprintln!("Failed: {:?}", e),
/// }
/// ```
pub fn toggle_rust_docstring_singleline_comment(
    file_path: &str,
    row_line_zeroindex: usize,
) -> Result<(), ToggleError> {
    // Convert to absolute path
    let absolute_path = match Path::new(file_path).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(ToggleError::FileNotFound);
            }
            return Err(ToggleError::PathError);
        }
    };

    // Get filename for backup naming
    let filename = match absolute_path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => return Err(ToggleError::PathError),
    };

    // Use TripleSlash flag (no extension lookup needed)
    let comment_flag = CommentFlag::TripppleSlash;

    // Create backup path in CWD
    let backup_filename = format!("backup_toggle_comment_{}", filename);
    let backup_path = PathBuf::from(&backup_filename);

    // Create backup copy
    if let Err(_) = std::fs::copy(&absolute_path, &backup_path) {
        return Err(ToggleError::IoError(IoOperation::Backup));
    }

    // Create working temp file in CWD
    let temp_filename = format!("temp_toggle_docstring_{}_{}", std::process::id(), filename);
    let temp_path = PathBuf::from(&temp_filename);

    // Process file and toggle comment on target line
    let process_result =
        process_file_toggle(&absolute_path, &temp_path, row_line_zeroindex, comment_flag);

    // Handle result
    match process_result {
        Ok(()) => {
            // Success: replace original
            if let Err(_) = std::fs::copy(&temp_path, &absolute_path) {
                let _ = std::fs::remove_file(&temp_path);
                return Err(ToggleError::IoError(IoOperation::Replace));
            }

            // Clean up temp file
            if let Err(_) = std::fs::remove_file(&temp_path) {
                // Non-fatal: temp file left behind but operation succeeded
                #[cfg(debug_assertions)]
                eprintln!("Warning: Failed to clean up temp file");
            }

            Ok(())
        }
        Err(e) => {
            // Failed: clean up and return error
            let _ = std::fs::remove_file(&temp_path);
            Err(e)
        }
    }
}

// ================
// Block Party Mode
// ================

/// Block comment markers for different languages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BlockMarkers {
    start: &'static [u8],
    end: &'static [u8],
}

/// Determine block comment markers from file extension
///
/// # Arguments
/// * `extension` - File extension without dot
///
/// # Returns
/// * `Some(BlockMarkers)` - Start and end markers for this language
/// * `None` - Extension not supported for block comments
fn determine_block_markers(extension: &str) -> Option<BlockMarkers> {
    match extension.to_lowercase().as_str() {
        // C-style block comments: /* */
        "rs" | "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" | "js" | "ts" | "java" | "go" | "swift" => {
            Some(BlockMarkers {
                start: b"/*\n",
                end: b"*/\n",
            })
        }

        // Python triple-quote: """ """
        "py" => Some(BlockMarkers {
            start: b"\"\"\"\n",
            end: b"\"\"\"\n",
        }),

        // Shell/TOML/YAML don't have block comments
        _ => None,
    }
}

/// Mode for block comment operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockMode {
    /// Add block markers (markers not present)
    Add,
    /// Remove block markers (markers present)
    Remove,
}

/// Detect whether block markers are present
///
/// # Arguments
/// * `file_path` - Path to file
/// * `start_line` - Expected position of start marker (or first content line)
/// * `end_line` - Expected position of end marker (or last content line + 1)
/// * `markers` - Block markers to check for
///
/// # Returns
/// * `Ok(BlockMode::Add)` - Markers not present
/// * `Ok(BlockMode::Remove)` - Markers present
/// * `Err(InconsistentBlockMarkers)` - Only one marker present
/// * `Err(LineNotFound)` - File too short
///
/// # Detection Logic
/// Checks if:
/// - Line at start_line is EXACTLY the start marker
/// - Line at end_line is EXACTLY the end marker
/// Both must match for Remove mode, neither for Add mode
fn detect_block_mode(
    file_path: &Path,
    start_line: usize,
    end_line: usize,
    markers: BlockMarkers,
) -> Result<BlockMode, ToggleError> {
    // Open file for reading
    let file = match File::open(file_path) {
        Ok(f) => f,
        Err(_) => return Err(ToggleError::IoError(IoOperation::Open)),
    };

    let mut reader = BufReader::with_capacity(IO_BUFFER_SIZE, file);
    let mut line_buffer = Vec::with_capacity(MAX_LINE_LENGTH);

    let mut current_line: usize = 0;
    let mut start_is_marker = false;
    let mut end_is_marker = false;
    let mut found_start_line = false;
    let mut found_end_line = false;

    // Safety limit for loop
    let line_limit = end_line.saturating_add(1000);

    // Read until we've checked both positions
    loop {
        // Safety: prevent unbounded loop
        if current_line > line_limit {
            return Err(ToggleError::IoError(IoOperation::Read));
        }

        line_buffer.clear();

        let bytes_read = match reader.read_until(b'\n', &mut line_buffer) {
            Ok(n) => n,
            Err(_) => return Err(ToggleError::IoError(IoOperation::Read)),
        };

        // End of file
        if bytes_read == 0 {
            break;
        }

        // Check line length
        if line_buffer.len() > MAX_LINE_LENGTH {
            return Err(ToggleError::LineTooLong {
                line_number: current_line,
                length: line_buffer.len(),
            });
        }

        // Check if this is start_line
        if current_line == start_line {
            found_start_line = true;
            start_is_marker = line_buffer.as_slice() == markers.start;
        }

        // Check if this is end_line (the line after the content range)
        if current_line == end_line {
            found_end_line = true;
            end_is_marker = line_buffer.as_slice() == markers.end;
            // Found both - can stop reading
            break;
        }

        current_line += 1;
    }

    // Verify we found both line positions
    if !found_start_line {
        return Err(ToggleError::LineNotFound {
            requested: start_line,
            file_lines: current_line,
        });
    }

    if !found_end_line {
        return Err(ToggleError::LineNotFound {
            requested: end_line,
            file_lines: current_line,
        });
    }

    // Determine mode based on marker presence
    match (start_is_marker, end_is_marker) {
        (true, true) => Ok(BlockMode::Remove),
        (false, false) => Ok(BlockMode::Add),
        _ => Err(ToggleError::InconsistentBlockMarkers),
    }
}
/// Toggle block comment around a range of lines
///
/// # Arguments
/// * `file_path` - Path to source file
/// * `start_line` - For Add: first content line. For Remove: the start marker line
/// * `end_line` - For Add: last content line. For Remove: the end marker line
///
/// # Logic
/// **Detection:**
/// - Check if line[start_line] == start_marker AND line[end_line] == end_marker
/// - If both match: Remove mode (skip those lines)
/// - If neither match: Add mode (insert new lines adjacent)
/// - If only one matches: Error (inconsistent)
///
/// **Add Mode:** Insert markers around content
/// - At start_line: INSERT marker before, then write content
/// - At end_line: write content, then INSERT marker after
///
/// **Remove Mode:** Delete marker lines
/// - At start_line: SKIP (don't write - this deletes the marker)
/// - At end_line: SKIP (don't write - this deletes the marker)
pub fn toggle_block_comment(
    file_path: &str,
    start_line: usize,
    end_line: usize,
) -> Result<(), ToggleError> {
    // Validate line range
    if start_line >= end_line {
        return Err(ToggleError::InvalidLineRange);
    }

    // Convert to absolute path
    let absolute_path = match Path::new(file_path).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(ToggleError::FileNotFound);
            }
            return Err(ToggleError::PathError);
        }
    };

    // Extract and validate extension
    let extension = match absolute_path.extension() {
        Some(ext) => ext.to_string_lossy().to_string(),
        None => return Err(ToggleError::NoExtension),
    };

    // Determine block markers from extension
    let markers = match determine_block_markers(&extension) {
        Some(m) => m,
        None => return Err(ToggleError::UnsupportedExtension),
    };

    // Detect current state - are markers present AT these line positions?
    let mode = detect_block_mode(&absolute_path, start_line, end_line, markers)?;

    // Get filename for backup
    let filename = match absolute_path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => return Err(ToggleError::PathError),
    };

    // Create backup path
    let backup_filename = format!("backup_toggle_comment_{}", filename);
    let backup_path = PathBuf::from(&backup_filename);

    // Create backup
    if let Err(_) = std::fs::copy(&absolute_path, &backup_path) {
        return Err(ToggleError::IoError(IoOperation::Backup));
    }

    // Create temp file
    let temp_filename = format!("temp_toggle_block_{}_{}", std::process::id(), filename);
    let temp_path = PathBuf::from(&temp_filename);

    // Process file
    let process_result = process_block_toggle(
        &absolute_path,
        &temp_path,
        start_line,
        end_line,
        markers,
        mode,
    );

    // Handle result
    match process_result {
        Ok(()) => {
            // Success: replace original
            if let Err(_) = std::fs::copy(&temp_path, &absolute_path) {
                let _ = std::fs::remove_file(&temp_path);
                return Err(ToggleError::IoError(IoOperation::Replace));
            }

            // Clean up temp
            if let Err(_) = std::fs::remove_file(&temp_path) {
                #[cfg(debug_assertions)]
                eprintln!("Warning: Failed to clean up temp file");
            }

            Ok(())
        }
        Err(e) => {
            // Failed: clean up and return error
            let _ = std::fs::remove_file(&temp_path);
            Err(e)
        }
    }
}

/// Process file to add or remove block comment markers
///
/// # Add Mode Logic (Insert new marker lines)
/// ```text
/// Source:           Output:
/// line 4            line 4
/// line 5 (start) -> """         <- INSERTED
///                   line 5      <- copied
/// line 6            line 6
/// line 7 (end)   -> line 7      <- copied
///                   """         <- INSERTED
/// line 8            line 8
/// ```
///
/// # Remove Mode Logic (Delete marker lines)
/// ```text
/// Source:           Output:
/// line 4            line 4
/// line 5 (marker) -> [SKIP]     <- DELETED
/// line 6            line 6
/// line 7 (marker) -> [SKIP]     <- DELETED
/// line 8            line 8
/// ```
fn process_block_toggle(
    source_path: &Path,
    dest_path: &Path,
    start_line: usize,
    end_line: usize,
    markers: BlockMarkers,
    mode: BlockMode,
) -> Result<(), ToggleError> {
    // Open source file
    let source_file = match File::open(source_path) {
        Ok(f) => f,
        Err(_) => return Err(ToggleError::IoError(IoOperation::Open)),
    };

    let mut reader = BufReader::with_capacity(IO_BUFFER_SIZE, source_file);

    // Create destination file
    let dest_file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dest_path)
    {
        Ok(f) => f,
        Err(_) => return Err(ToggleError::IoError(IoOperation::Create)),
    };

    let mut writer = BufWriter::with_capacity(IO_BUFFER_SIZE, dest_file);

    let mut line_buffer = Vec::with_capacity(MAX_LINE_LENGTH);
    let mut current_line: usize = 0;
    let line_limit = end_line.saturating_add(1000000);

    // Process file line by line
    loop {
        // Safety check
        if current_line > line_limit {
            return Err(ToggleError::IoError(IoOperation::Read));
        }

        line_buffer.clear();

        let bytes_read = match reader.read_until(b'\n', &mut line_buffer) {
            Ok(n) => n,
            Err(_) => return Err(ToggleError::IoError(IoOperation::Read)),
        };

        // End of file
        if bytes_read == 0 {
            break;
        }

        // Safety: check line length
        if line_buffer.len() > MAX_LINE_LENGTH {
            return Err(ToggleError::LineTooLong {
                line_number: current_line,
                length: line_buffer.len(),
            });
        }

        // Handle based on mode
        match mode {
            BlockMode::Add => {
                // ADD mode: INSERT new marker lines adjacent to content

                if current_line == start_line {
                    // INSERT start marker before this content line
                    if let Err(_) = writer.write_all(markers.start) {
                        return Err(ToggleError::IoError(IoOperation::Write));
                    }
                    // Then write the content line itself
                    if let Err(_) = writer.write_all(&line_buffer) {
                        return Err(ToggleError::IoError(IoOperation::Write));
                    }
                } else if current_line == end_line {
                    // Write the content line first
                    if let Err(_) = writer.write_all(&line_buffer) {
                        return Err(ToggleError::IoError(IoOperation::Write));
                    }
                    // Then INSERT end marker after this content line
                    if let Err(_) = writer.write_all(markers.end) {
                        return Err(ToggleError::IoError(IoOperation::Write));
                    }
                } else {
                    // All other lines: copy unchanged
                    if let Err(_) = writer.write_all(&line_buffer) {
                        return Err(ToggleError::IoError(IoOperation::Write));
                    }
                }
            }

            BlockMode::Remove => {
                // REMOVE mode: DELETE marker lines by skipping them

                if current_line == start_line {
                    // This line IS the start marker - SKIP it (don't write)
                    // This deletes the marker line
                } else if current_line == end_line {
                    // This line IS the end marker - SKIP it (don't write)
                    // This deletes the marker line
                } else {
                    // All other lines: copy unchanged
                    if let Err(_) = writer.write_all(&line_buffer) {
                        return Err(ToggleError::IoError(IoOperation::Write));
                    }
                }
            }
        }

        current_line += 1;
    }

    // Flush writer
    if let Err(_) = writer.flush() {
        return Err(ToggleError::IoError(IoOperation::Flush));
    }

    Ok(())
}

#[cfg(test)]
mod block_comment_tests {
    use super::*;

    /// Helper: create a temporary test file with given content
    fn create_test_file(filename: &str, content: &str) -> PathBuf {
        let path = PathBuf::from(filename);
        let mut file = File::create(&path).expect("Failed to create test file");
        file.write_all(content.as_bytes())
            .expect("Failed to write test file");
        path
    }

    /// Helper: read file content as string
    fn read_file_content(path: &Path) -> String {
        std::fs::read_to_string(path).expect("Failed to read file")
    }

    /// Helper: cleanup test files
    fn cleanup_files(paths: &[&Path]) {
        for path in paths {
            let _ = std::fs::remove_file(path);
        }
    }

    #[test]
    fn test_block_comment_add_rust() {
        let content = "fn main() {\n    println!(\"hello\");\n}\n";
        let test_file = create_test_file("test_block_add.rs", content);

        // Comment lines 0-2 (entire file)
        let result = toggle_block_comment(test_file.to_str().unwrap(), 0, 2);

        #[cfg(test)]
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        let expected = "/*\nfn main() {\n    println!(\"hello\");\n}\n*/\n";

        #[cfg(test)]
        assert_eq!(new_content, expected);

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_block_test_block_add.rs"),
        ]);
    }

    #[test]
    fn test_block_comment_remove_rust() {
        let content = "/*\nfn main() {\n    println!(\"hello\");\n}\n*/\n";
        let test_file = create_test_file("test_block_remove.rs", content);

        // Remove markers at lines 0 and 4
        let result = toggle_block_comment(test_file.to_str().unwrap(), 0, 4);

        #[cfg(test)]
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        let expected = "fn main() {\n    println!(\"hello\");\n}\n";

        #[cfg(test)]
        assert_eq!(new_content, expected);

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_block_test_block_remove.rs"),
        ]);
    }

    #[test]
    fn test_block_comment_inconsistent() {
        // Only start marker present
        let content = "/*\nfn main() {}\n";
        let test_file = create_test_file("test_block_inconsistent.rs", content);

        let result = toggle_block_comment(test_file.to_str().unwrap(), 0, 1);

        #[cfg(test)]
        assert!(matches!(result, Err(ToggleError::InconsistentBlockMarkers)));

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_block_test_block_inconsistent.rs"),
        ]);
    }

    #[test]
    fn test_block_comment_python() {
        let content = "def hello():\n    print('world')\n";
        let test_file = create_test_file("test_block.py", content);

        let result = toggle_block_comment(test_file.to_str().unwrap(), 0, 1);

        #[cfg(test)]
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        let expected = "\"\"\"\ndef hello():\n    print('world')\n\"\"\"\n";

        #[cfg(test)]
        assert_eq!(new_content, expected);

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_block_test_block.py"),
        ]);
    }
}

// ================
// List Batch Toggle
// =================

/// Maximum number of lines in batch operations
/// 128 lines = 1KB of stack space (8 bytes per usize on 64-bit)
const MAX_BATCH_LINES: usize = 128;

/// Toggle comments on multiple lines in a single operation
///
/// # Overview
/// Process multiple lines with one backup and one file pass.
/// More efficient than calling toggle function N times.
///
/// # Arguments
/// * `file_path` - Path to source file
/// * `line_numbers` - Slice of line numbers to toggle (zero-indexed)
/// * `comment_flag` - Which comment type to use
///
/// # Returns
/// * `Ok(())` - All lines toggled successfully
/// * `Err(ToggleError)` - Processing failed
///
/// # Safety
/// - Bounded to MAX_BATCH_LINES (128 lines)
/// - Single backup, single file pass
/// - Stack-only array (no heap)
/// - Sorted for O(1) lookup per line
///
/// # Example
/// ```no_run
/// let lines = [5, 10, 15, 20];
/// toggle_multiple_lines("./src/main.rs", &lines, CommentFlag::DoubleSlash)?;
/// ```
fn toggle_multiple_lines(
    file_path: &str,
    line_numbers: &[usize],
    comment_flag: CommentFlag,
) -> Result<(), ToggleError> {
    // Validate input
    if line_numbers.is_empty() {
        return Ok(()); // Nothing to do
    }

    if line_numbers.len() > MAX_BATCH_LINES {
        // Too many lines - could return error or just process first N
        return Err(ToggleError::IoError(IoOperation::Read)); // Reuse error for now
    }

    // Convert to absolute path
    let absolute_path = match Path::new(file_path).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(ToggleError::FileNotFound);
            }
            return Err(ToggleError::PathError);
        }
    };

    // Get filename for backup
    let filename = match absolute_path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => return Err(ToggleError::PathError),
    };

    // Copy line numbers to fixed array and sort
    let mut sorted_lines: [usize; MAX_BATCH_LINES] = [0; MAX_BATCH_LINES];
    let count = line_numbers.len();

    // Copy input to our array
    for i in 0..count {
        sorted_lines[i] = line_numbers[i];
    }

    // Sort the array (only the valid portion)
    // This enables O(1) lookup during file processing
    sorted_lines[..count].sort_unstable();

    // Remove duplicates by compacting array
    let mut unique_count = 0;
    if count > 0 {
        sorted_lines[0] = sorted_lines[0]; // First element stays
        unique_count = 1;

        for i in 1..count {
            if sorted_lines[i] != sorted_lines[unique_count - 1] {
                sorted_lines[unique_count] = sorted_lines[i];
                unique_count += 1;
            }
        }
    }

    // If no valid lines after dedup, nothing to do
    if unique_count == 0 {
        return Ok(());
    }

    // Create backup path in CWD
    let backup_filename = format!("backup_toggle_comment_{}", filename);
    let backup_path = PathBuf::from(&backup_filename);

    // Create backup copy
    if let Err(_) = std::fs::copy(&absolute_path, &backup_path) {
        return Err(ToggleError::IoError(IoOperation::Backup));
    }

    // Create temp file
    let temp_filename = format!("temp_toggle_batch_{}_{}", std::process::id(), filename);
    let temp_path = PathBuf::from(&temp_filename);

    // Process file with batch toggle
    let process_result = process_batch_toggle(
        &absolute_path,
        &temp_path,
        &sorted_lines[..unique_count],
        comment_flag,
    );

    // Handle result
    match process_result {
        Ok(()) => {
            // Success: replace original
            if let Err(_) = std::fs::copy(&temp_path, &absolute_path) {
                let _ = std::fs::remove_file(&temp_path);
                return Err(ToggleError::IoError(IoOperation::Replace));
            }

            // Clean up temp
            if let Err(_) = std::fs::remove_file(&temp_path) {
                #[cfg(debug_assertions)]
                eprintln!("Warning: Failed to clean up temp file");
            }

            Ok(())
        }
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            Err(e)
        }
    }
}

/// Process file toggling comments on multiple lines
///
/// # Arguments
/// * `source_path` - Original file
/// * `dest_path` - Temporary output file
/// * `target_lines` - SORTED array of line numbers to toggle
/// * `flag` - Comment flag to use
///
/// # Algorithm
/// Uses sorted array for O(1) lookup:
/// - Keep index into sorted array
/// - For each line, check if current_line == sorted_array[index]
/// - If match: toggle and increment index
/// - If no match: copy unchanged
///
/// # Returns
/// * `Ok(())` - Processing succeeded
/// * `Err(ToggleError)` - Processing failed
fn process_batch_toggle(
    source_path: &Path,
    dest_path: &Path,
    target_lines: &[usize], // Pre-sorted, no duplicates
    flag: CommentFlag,
) -> Result<(), ToggleError> {
    // Open source file
    let source_file = match File::open(source_path) {
        Ok(f) => f,
        Err(_) => return Err(ToggleError::IoError(IoOperation::Open)),
    };

    let mut reader = BufReader::with_capacity(IO_BUFFER_SIZE, source_file);

    // Create destination file
    let dest_file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dest_path)
    {
        Ok(f) => f,
        Err(_) => return Err(ToggleError::IoError(IoOperation::Create)),
    };

    let mut writer = BufWriter::with_capacity(IO_BUFFER_SIZE, dest_file);

    let mut line_buffer = Vec::with_capacity(MAX_LINE_LENGTH);
    let mut current_line: usize = 0;
    let mut target_index: usize = 0; // Index into sorted target_lines array

    // Get last target line for safety limit
    let max_target_line = target_lines[target_lines.len() - 1];
    let line_limit = max_target_line.saturating_add(1000000);

    // Track how many lines we actually toggled
    let mut toggled_count: usize = 0;

    // Process file line by line
    loop {
        // Safety check
        if current_line > line_limit {
            return Err(ToggleError::IoError(IoOperation::Read));
        }

        line_buffer.clear();

        let bytes_read = match reader.read_until(b'\n', &mut line_buffer) {
            Ok(n) => n,
            Err(_) => return Err(ToggleError::IoError(IoOperation::Read)),
        };

        // End of file
        if bytes_read == 0 {
            break;
        }

        // Safety: check line length
        if line_buffer.len() > MAX_LINE_LENGTH {
            return Err(ToggleError::LineTooLong {
                line_number: current_line,
                length: line_buffer.len(),
            });
        }

        // Check if this is a target line (O(1) because array is sorted)
        if target_index < target_lines.len() && current_line == target_lines[target_index] {
            // This is a target line - toggle it
            if let Err(e) = toggle_line(&mut writer, &line_buffer, flag) {
                return Err(e);
            }

            // Move to next target
            target_index += 1;
            toggled_count += 1;
        } else {
            // Not a target line - copy unchanged
            if let Err(_) = writer.write_all(&line_buffer) {
                return Err(ToggleError::IoError(IoOperation::Write));
            }
        }

        current_line += 1;
    }

    // Flush writer
    if let Err(_) = writer.flush() {
        return Err(ToggleError::IoError(IoOperation::Flush));
    }

    // Verify we found all target lines
    if toggled_count < target_lines.len() {
        // Some target lines were beyond EOF
        let first_missing = target_lines[toggled_count];
        return Err(ToggleError::LineNotFound {
            requested: first_missing,
            file_lines: current_line,
        });
    }

    Ok(())
}

/// Toggle basic comments on multiple lines (extension-based)
///
/// # Arguments
/// * `file_path` - Path to source file
/// * `line_numbers` - Slice of line numbers to toggle
///
/// # Returns
/// * `Ok(())` - All lines toggled successfully
/// * `Err(ToggleError)` - Processing failed
///
/// # Example
/// ```no_run
/// let lines = [5, 10, 15];
/// toggle_multiple_basic_comments("./src/main.rs", &lines)?;
/// ```
pub fn toggle_multiple_basic_comments(
    file_path: &str,
    line_numbers: &[usize],
) -> Result<(), ToggleError> {
    // Determine comment flag from extension
    let path = Path::new(file_path);
    let extension = match path.extension() {
        Some(ext) => ext.to_string_lossy().to_string(),
        None => return Err(ToggleError::NoExtension),
    };

    let comment_flag = match determine_comment_flag(&extension) {
        Some(flag) => flag,
        None => return Err(ToggleError::UnsupportedExtension),
    };

    toggle_multiple_lines(file_path, line_numbers, comment_flag)
}

/// Toggle Rust docstrings on multiple lines
///
/// # Arguments
/// * `file_path` - Path to source file
/// * `line_numbers` - Slice of line numbers to toggle
///
/// # Returns
/// * `Ok(())` - All docstrings toggled successfully
/// * `Err(ToggleError)` - Processing failed
///
/// # Example
/// ```no_run
/// let lines = [10, 20, 30];
/// toggle_multiple_singline_docstrings("./src/lib.rs", &lines)?;
/// ```
pub fn toggle_multiple_singline_docstrings(
    file_path: &str,
    line_numbers_list: &[usize],
) -> Result<(), ToggleError> {
    toggle_multiple_lines(file_path, line_numbers_list, CommentFlag::TripppleSlash)
}

#[cfg(test)]
mod batch_tests {
    use super::*;

    /// Helper: create a temporary test file with given content
    fn create_test_file(filename: &str, content: &str) -> PathBuf {
        let path = PathBuf::from(filename);
        let mut file = File::create(&path).expect("Failed to create test file");
        file.write_all(content.as_bytes())
            .expect("Failed to write test file");
        path
    }

    /// Helper: read file content as string
    fn read_file_content(path: &Path) -> String {
        std::fs::read_to_string(path).expect("Failed to read file")
    }

    /// Helper: cleanup test files
    fn cleanup_files(paths: &[&Path]) {
        for path in paths {
            let _ = std::fs::remove_file(path);
        }
    }

    #[test]
    fn test_batch_toggle_multiple_lines() {
        let content = "line 0\nline 1\nline 2\nline 3\nline 4\n";
        let test_file = create_test_file("test_batch.rs", content);

        let lines = [1, 3]; // Toggle lines 1 and 3
        let result = toggle_multiple_basic_comments(test_file.to_str().unwrap(), &lines);

        #[cfg(test)]
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        let expected = "line 0\n// line 1\nline 2\n// line 3\nline 4\n";

        #[cfg(test)]
        assert_eq!(new_content, expected);

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_batch_test_batch.rs"),
        ]);
    }

    #[test]
    fn test_batch_toggle_unsorted_input() {
        let content = "line 0\nline 1\nline 2\nline 3\n";
        let test_file = create_test_file("test_batch_unsort.rs", content);

        let lines = [3, 1, 2]; // Unsorted input
        let result = toggle_multiple_basic_comments(test_file.to_str().unwrap(), &lines);

        #[cfg(test)]
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        let expected = "line 0\n// line 1\n// line 2\n// line 3\n";

        #[cfg(test)]
        assert_eq!(new_content, expected);

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_batch_test_batch_unsort.rs"),
        ]);
    }

    #[test]
    fn test_batch_toggle_duplicates() {
        let content = "line 0\nline 1\nline 2\n";
        let test_file = create_test_file("test_batch_dup.rs", content);

        let lines = [1, 1, 1]; // Duplicates
        let result = toggle_multiple_basic_comments(test_file.to_str().unwrap(), &lines);

        #[cfg(test)]
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        let expected = "line 0\n// line 1\nline 2\n";

        #[cfg(test)]
        assert_eq!(new_content, expected);

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_batch_test_batch_dup.rs"),
        ]);
    }
}

/// Toggle comment on a specific line in a source code file
///
/// # Overview
/// This function safely toggles a comment flag on a single line without loading
/// the entire file into memory. It creates a backup before any modifications and
/// only replaces the original file on success.
///
/// Comment type is determined by file extension.
///
/// # Arguments
/// * `file_path` - Path to the source file
/// * `row_line_zeroindex` - Zero-indexed line number to toggle
///
/// # Returns
/// * `Ok(())` - Comment toggled successfully
/// * `Err(ToggleError)` - Specific error code
///
/// # Example
/// ```no_run
/// use toggle_basic_singleline_comment::toggle_basic_singleline_comment;
///
/// match toggle_basic_singleline_comment("./src/main.rs", 0) {
///     Ok(()) => println!("Toggled"),
///     Err(e) => eprintln!("Failed: {:?}", e),
/// }
/// ```
pub fn toggle_basic_singleline_comment(
    file_path: &str,
    row_line_zeroindex: usize,
) -> Result<(), ToggleError> {
    // Convert to absolute path
    let absolute_path = match Path::new(file_path).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(ToggleError::FileNotFound);
            }
            return Err(ToggleError::PathError);
        }
    };

    // Extract and validate file extension
    let extension = match absolute_path.extension() {
        Some(ext) => ext.to_string_lossy().to_string(),
        None => return Err(ToggleError::NoExtension),
    };

    // Determine comment flag from extension
    let comment_flag = match determine_comment_flag(&extension) {
        Some(flag) => flag,
        None => return Err(ToggleError::UnsupportedExtension),
    };

    // Get filename for backup naming
    let filename = match absolute_path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => return Err(ToggleError::PathError),
    };

    // Create backup path in CWD
    let backup_filename = format!("backup_toggle_comment_{}", filename);
    let backup_path = PathBuf::from(&backup_filename);

    // Create backup copy of original file
    if let Err(_) = std::fs::copy(&absolute_path, &backup_path) {
        return Err(ToggleError::IoError(IoOperation::Backup));
    }

    // Create working temp file in CWD
    let temp_filename = format!("temp_toggle_{}_{}", std::process::id(), filename);
    let temp_path = PathBuf::from(&temp_filename);

    // Process file and toggle comment on target line
    let process_result =
        process_file_toggle(&absolute_path, &temp_path, row_line_zeroindex, comment_flag);

    // Handle processing result
    match process_result {
        Ok(()) => {
            // Success: replace original with temp file
            if let Err(_) = std::fs::copy(&temp_path, &absolute_path) {
                // Failed to replace - clean up and error
                let _ = std::fs::remove_file(&temp_path);
                return Err(ToggleError::IoError(IoOperation::Replace));
            }

            // Clean up temp file
            if let Err(_) = std::fs::remove_file(&temp_path) {
                // Non-fatal: temp file left behind but operation succeeded
                #[cfg(debug_assertions)]
                eprintln!("Warning: Failed to clean up temp file");
            }

            Ok(())
        }
        Err(e) => {
            // Failed: clean up temp file and return error
            let _ = std::fs::remove_file(&temp_path);
            Err(e)
        }
    }
}

/// Process file line-by-line, toggling comment on target line
///
/// # Arguments
/// * `source_path` - Original file to read from
/// * `dest_path` - Temporary file to write modified content to
/// * `target_line` - Zero-indexed line number to toggle
/// * `flag` - Comment flag to use
///
/// # Returns
/// * `Ok(())` - Processing succeeded, target line was found and toggled
/// * `Err(ToggleError)` - Processing failed
///
/// # Safety
/// - Pre-allocated buffers only
/// - Bounded line length checks
/// - No dynamic allocation during loop
fn process_file_toggle(
    source_path: &Path,
    dest_path: &Path,
    target_line: usize,
    flag: CommentFlag,
) -> Result<(), ToggleError> {
    // Open source file for reading
    let source_file = match File::open(source_path) {
        Ok(f) => f,
        Err(_) => return Err(ToggleError::IoError(IoOperation::Open)),
    };

    // Create buffered reader with pre-allocated buffer
    let mut reader = BufReader::with_capacity(IO_BUFFER_SIZE, source_file);

    // Create destination file
    let dest_file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dest_path)
    {
        Ok(f) => f,
        Err(_) => return Err(ToggleError::IoError(IoOperation::Create)),
    };

    // Create buffered writer with pre-allocated buffer
    let mut writer = BufWriter::with_capacity(IO_BUFFER_SIZE, dest_file);

    // Pre-allocate line buffer - fixed size, reused for all lines
    let mut line_buffer = Vec::with_capacity(MAX_LINE_LENGTH);

    // Track current line number (zero-indexed)
    let mut current_line: usize = 0;

    // Track if we found the target line
    let mut found_target = false;

    // Line counter safety limit - prevent infinite loops
    let line_limit = target_line.saturating_add(1000000);

    // Process file line by line
    loop {
        // Safety check: prevent unbounded loop
        if current_line > line_limit {
            return Err(ToggleError::IoError(IoOperation::Read));
        }

        // Clear buffer for reuse
        line_buffer.clear();

        // Read next line into pre-allocated buffer
        let bytes_read = match reader.read_until(b'\n', &mut line_buffer) {
            Ok(n) => n,
            Err(_) => return Err(ToggleError::IoError(IoOperation::Read)),
        };

        // End of file reached
        if bytes_read == 0 {
            break;
        }

        // Safety: check line length
        if line_buffer.len() > MAX_LINE_LENGTH {
            return Err(ToggleError::LineTooLong {
                line_number: current_line,
                length: line_buffer.len(),
            });
        }

        // Check if this is our target line
        if current_line == target_line {
            found_target = true;

            // Toggle comment on this line
            if let Err(e) = toggle_line(&mut writer, &line_buffer, flag) {
                return Err(e);
            }
        } else {
            // Copy line unchanged
            if let Err(_) = writer.write_all(&line_buffer) {
                return Err(ToggleError::IoError(IoOperation::Write));
            }
        }

        current_line += 1;
    }

    // Flush writer to ensure all data written
    if let Err(_) = writer.flush() {
        return Err(ToggleError::IoError(IoOperation::Flush));
    }

    // Check if we found the target line
    if !found_target {
        return Err(ToggleError::LineNotFound {
            requested: target_line,
            file_lines: current_line,
        });
    }

    Ok(())
}

/// Toggle comment flag on a single line
///
/// # Arguments
/// * `writer` - Buffered writer to output modified line
/// * `line_buffer` - Complete line including newline
/// * `flag` - Comment flag to toggle
///
/// # Returns
/// * `Ok(())` - Line written successfully
/// * `Err(ToggleError)` - Write failed
///
/// # Logic
/// - Extract line content (without trailing newline)
/// - Check if comment should be removed
/// - If yes: write line without flag+space
/// - If no: write flag+space at start, then rest of line
/// - Always preserve original newline
fn toggle_line(
    writer: &mut BufWriter<File>,
    line_buffer: &[u8],
    flag: CommentFlag,
) -> Result<(), ToggleError> {
    // Separate line content from newline
    let (content, newline) = if line_buffer.ends_with(b"\r\n") {
        (&line_buffer[..line_buffer.len() - 2], &b"\r\n"[..])
    } else if line_buffer.ends_with(b"\n") {
        (&line_buffer[..line_buffer.len() - 1], &b"\n"[..])
    } else {
        // No newline (last line of file might not have one)
        (line_buffer, &b""[..])
    };

    // Check if we should remove comment
    if let Some(skip_count) = should_remove_comment(content, flag) {
        // REMOVE mode: write content skipping flag+space
        if let Err(_) = writer.write_all(&content[skip_count..]) {
            return Err(ToggleError::IoError(IoOperation::Write));
        }
    } else {
        // ADD mode: write flag+space, then content
        let flag_with_space = format!("{} ", flag.as_str());

        if let Err(_) = writer.write_all(flag_with_space.as_bytes()) {
            return Err(ToggleError::IoError(IoOperation::Write));
        }

        if let Err(_) = writer.write_all(content) {
            return Err(ToggleError::IoError(IoOperation::Write));
        }
    }

    // Write newline back (preserve original line ending)
    if !newline.is_empty() {
        if let Err(_) = writer.write_all(newline) {
            return Err(ToggleError::IoError(IoOperation::Write));
        }
    }

    Ok(())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod toggle_comment_tests {
    use super::*;
    use std::io::Write;

    /// Helper: create a temporary test file with given content
    fn create_test_file(filename: &str, content: &str) -> PathBuf {
        let path = PathBuf::from(filename);
        let mut file = File::create(&path).expect("Failed to create test file");
        file.write_all(content.as_bytes())
            .expect("Failed to write test file");
        path
    }

    /// Helper: read file content as string
    fn read_file_content(path: &Path) -> String {
        std::fs::read_to_string(path).expect("Failed to read file")
    }

    /// Helper: cleanup test files
    fn cleanup_files(paths: &[&Path]) {
        for path in paths {
            let _ = std::fs::remove_file(path);
        }
    }

    #[test]
    fn test_determine_comment_flag() {
        // Double-slash languages
        assert_eq!(determine_comment_flag("rs"), Some(CommentFlag::DoubleSlash));
        assert_eq!(
            determine_comment_flag("cpp"),
            Some(CommentFlag::DoubleSlash)
        );
        assert_eq!(determine_comment_flag("js"), Some(CommentFlag::DoubleSlash));

        // Hash languages
        assert_eq!(determine_comment_flag("py"), Some(CommentFlag::Hash));
        assert_eq!(determine_comment_flag("toml"), Some(CommentFlag::Hash));
        assert_eq!(determine_comment_flag("sh"), Some(CommentFlag::Hash));

        // Unknown
        assert_eq!(determine_comment_flag("txt"), None);
        assert_eq!(determine_comment_flag("unknown"), None);
    }

    #[test]
    fn test_should_remove_comment_rust() {
        let flag = CommentFlag::DoubleSlash;

        // Should remove: "// code"
        assert_eq!(should_remove_comment(b"// code", flag), Some(3));

        // Should remove: "  // code"
        assert_eq!(should_remove_comment(b"  // code", flag), Some(3));

        // Should NOT remove: "//code" (no space after flag)
        assert_eq!(should_remove_comment(b"//code", flag), None);

        // Should NOT remove: "code // comment"
        assert_eq!(should_remove_comment(b"code // comment", flag), None);

        // Should NOT remove: empty line
        assert_eq!(should_remove_comment(b"", flag), None);

        // Should NOT remove: only spaces
        assert_eq!(should_remove_comment(b"    ", flag), None);
    }

    #[test]
    fn test_should_remove_comment_python() {
        let flag = CommentFlag::Hash;

        // Should remove: "# code"
        assert_eq!(should_remove_comment(b"# code", flag), Some(2));

        // Should remove: "  # code"
        assert_eq!(should_remove_comment(b"  # code", flag), Some(2));

        // Should NOT remove: "#code" (no space after flag)
        assert_eq!(should_remove_comment(b"#code", flag), None);

        // Should NOT remove: "code # comment"
        assert_eq!(should_remove_comment(b"code # comment", flag), None);
    }

    #[test]
    fn test_toggle_comment_add_rust() {
        let test_file = create_test_file("test_add.rs", "fn main() {}\n");

        let result = toggle_basic_singleline_comment(test_file.to_str().unwrap(), 0);

        #[cfg(test)]
        assert!(result.is_ok());

        let content = read_file_content(&test_file);

        #[cfg(test)]
        assert_eq!(content, "// fn main() {}\n");

        // Cleanup
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_add.rs"),
        ]);
    }

    #[test]
    fn test_toggle_comment_remove_rust() {
        let test_file = create_test_file("test_remove.rs", "// fn main() {}\n");

        let result = toggle_basic_singleline_comment(test_file.to_str().unwrap(), 0);

        #[cfg(test)]
        assert!(result.is_ok());

        let content = read_file_content(&test_file);

        #[cfg(test)]
        assert_eq!(content, "fn main() {}\n");

        // Cleanup
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_remove.rs"),
        ]);
    }

    #[test]
    fn test_toggle_comment_add_python() {
        let test_file = create_test_file("test_add.py", "print('hello')\n");

        let result = toggle_basic_singleline_comment(test_file.to_str().unwrap(), 0);

        #[cfg(test)]
        assert!(result.is_ok());

        let content = read_file_content(&test_file);

        #[cfg(test)]
        assert_eq!(content, "# print('hello')\n");

        // Cleanup
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_add.py"),
        ]);
    }

    #[test]
    fn test_toggle_comment_remove_python() {
        let test_file = create_test_file("test_remove.py", "# print('hello')\n");

        let result = toggle_basic_singleline_comment(test_file.to_str().unwrap(), 0);

        #[cfg(test)]
        assert!(result.is_ok());

        let content = read_file_content(&test_file);

        #[cfg(test)]
        assert_eq!(content, "print('hello')\n");

        // Cleanup
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_remove.py"),
        ]);
    }

    #[test]
    fn test_toggle_comment_line_not_found() {
        let test_file = create_test_file("test_notfound.rs", "fn main() {}\n");

        let result = toggle_basic_singleline_comment(test_file.to_str().unwrap(), 10);

        #[cfg(test)]
        assert!(matches!(result, Err(ToggleError::LineNotFound { .. })));

        // Cleanup
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_notfound.rs"),
        ]);
    }

    #[test]
    fn test_toggle_comment_no_extension() {
        let test_file = create_test_file("test_noext", "some content\n");

        let result = toggle_basic_singleline_comment(test_file.to_str().unwrap(), 0);

        #[cfg(test)]
        assert!(matches!(result, Err(ToggleError::NoExtension)));

        // Cleanup
        cleanup_files(&[&test_file]);
    }

    #[test]
    fn test_toggle_comment_unsupported_extension() {
        let test_file = create_test_file("test.txt", "some content\n");

        let result = toggle_basic_singleline_comment(test_file.to_str().unwrap(), 0);

        #[cfg(test)]
        assert!(matches!(result, Err(ToggleError::UnsupportedExtension)));

        // Cleanup
        cleanup_files(&[&test_file]);
    }

    #[test]
    fn test_toggle_preserves_other_lines() {
        let content = "line 0\nline 1\nline 2\n";
        let test_file = create_test_file("test_preserve.rs", content);

        // Toggle line 1
        let result = toggle_basic_singleline_comment(test_file.to_str().unwrap(), 1);

        #[cfg(test)]
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);

        #[cfg(test)]
        assert_eq!(new_content, "line 0\n// line 1\nline 2\n");

        // Cleanup
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_preserve.rs"),
        ]);
    }

    #[test]
    fn test_toggle_rust_docstring_add() {
        let test_file = create_test_file("test_docstring1.rs", "/// Some docs\n");
        let result = toggle_rust_docstring_singleline_comment(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        let content = read_file_content(&test_file);
        assert_eq!(content, "Some docs\n"); // Should remove ///
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_docstring.rs"),
        ]);
    }

    #[test]
    fn test_toggle_rust_docstring_remove() {
        let test_file = create_test_file("test_docstring2.rs", "Some docs\n");
        let result = toggle_rust_docstring_singleline_comment(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        let content = read_file_content(&test_file);
        assert_eq!(content, "/// Some docs\n"); // Should add ///
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_docstring.rs"),
        ]);
    }

    #[test]
    fn test_toggle_comment_no_newline_at_eof() {
        // Last line without newline
        let test_file = create_test_file("test_no_newline.rs", "fn main() {}");
        let result = toggle_basic_singleline_comment(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        let content = read_file_content(&test_file);
        assert_eq!(content, "// fn main() {}"); // No newline added
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_no_newline.rs"),
        ]);
    }

    #[test]
    fn test_toggle_comment_crlf_line_ending() {
        // Windows-style line ending
        let test_file = create_test_file("test_crlf.rs", "fn main() {}\r\n");
        let result = toggle_basic_singleline_comment(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        let content = read_file_content(&test_file);
        assert_eq!(content, "// fn main() {}\r\n"); // Preserve CRLF
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_crlf.rs"),
        ]);
    }

    #[test]
    fn test_toggle_comment_with_tabs() {
        let test_file = create_test_file("test_tabs.rs", "\t\tfn main() {}\n");
        let result = toggle_basic_singleline_comment(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        let content = read_file_content(&test_file);
        assert_eq!(content, "// \t\tfn main() {}\n");
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_tabs.rs"),
        ]);
    }

    #[test]
    fn test_toggle_comment_only_whitespace_line() {
        // Line with only spaces/tabs
        let test_file = create_test_file("test_whitespace.rs", "    \n");
        let result = toggle_basic_singleline_comment(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        let content = read_file_content(&test_file);
        assert_eq!(content, "//     \n"); // Comment even whitespace-only lines
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_whitespace.rs"),
        ]);
    }

    #[test]
    fn test_toggle_comment_empty_line() {
        let test_file = create_test_file("test_empty.rs", "\n");
        let result = toggle_basic_singleline_comment(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        let content = read_file_content(&test_file);
        assert_eq!(content, "// \n");
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_empty.rs"),
        ]);
    }

    #[test]
    fn test_toggle_comment_idempotent() {
        // Toggle on, then off - should return to original
        let original = "fn main() {}\n";
        let test_file = create_test_file("test_idempotent.rs", original);

        let result1 = toggle_basic_singleline_comment(test_file.to_str().unwrap(), 0);
        assert!(result1.is_ok());

        let result2 = toggle_basic_singleline_comment(test_file.to_str().unwrap(), 0);
        assert!(result2.is_ok());

        let content = read_file_content(&test_file);
        assert_eq!(content, original);
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_idempotent.rs"),
        ]);
    }
    #[test]
    fn test_toggle_comment_double_comment() {
        // Line that's "// // code"
        let test_file = create_test_file("test_double.rs", "// // code\n");
        let result = toggle_basic_singleline_comment(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        let content = read_file_content(&test_file);
        // Should remove outer comment only
        assert_eq!(content, "// code\n");
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_double.rs"),
        ]);
    }

    #[test]
    fn test_toggle_comment_just_flag_and_space() {
        // Line with only "// "
        let test_file = create_test_file("test_just_flag.rs", "// \n");
        let result = toggle_basic_singleline_comment(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        let content = read_file_content(&test_file);
        assert_eq!(content, "\n"); // Just whitespace removed
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_just_flag.rs"),
        ]);
    }
    #[test]
    fn test_toggle_comment_last_line() {
        let content = "line 0\nline 1\nline 2\n";
        let test_file = create_test_file("test_last_line.rs", content);
        let result = toggle_basic_singleline_comment(test_file.to_str().unwrap(), 2);
        assert!(result.is_ok());
        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "line 0\nline 1\n// line 2\n");
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_last_line.rs"),
        ]);
    }

    #[test]
    fn test_toggle_comment_single_line_file() {
        let test_file = create_test_file("test_single.rs", "code\n");
        let result = toggle_basic_singleline_comment(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        let content = read_file_content(&test_file);
        assert_eq!(content, "// code\n");
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_single.rs"),
        ]);
    }

    #[test]
    fn test_file_not_found() {
        let result = toggle_basic_singleline_comment("/nonexistent/path/file.rs", 0);
        assert!(matches!(result, Err(ToggleError::FileNotFound)));
    }
    #[test]
    fn test_extension_case_insensitive() {
        let test_file = create_test_file("test_upper.RS", "code\n");
        let result = toggle_basic_singleline_comment(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_upper.RS"),
        ]);
    }
    #[test]
    fn test_batch_empty_array() {
        let test_file = create_test_file("test_batch_empty.rs", "line 0\n");
        let lines: [usize; 0] = [];
        let result = toggle_multiple_basic_comments(test_file.to_str().unwrap(), &lines);
        assert!(result.is_ok()); // Should succeed (no-op)
        cleanup_files(&[&test_file]);
    }

    #[test]
    fn test_batch_out_of_range() {
        let test_file = create_test_file("test_batch_oob.rs", "line 0\n");
        let lines = [100]; // Way beyond file
        let result = toggle_multiple_basic_comments(test_file.to_str().unwrap(), &lines);
        assert!(matches!(result, Err(ToggleError::LineNotFound { .. })));
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_batch_test_batch_oob.rs"),
        ]);
    }

    #[test]
    fn test_batch_exceeds_max() {
        let test_file = create_test_file("test_batch_max.rs", "line\n".repeat(150).as_str());
        let lines: Vec<usize> = (0..130).collect(); // > MAX_BATCH_LINES
        let result = toggle_multiple_basic_comments(test_file.to_str().unwrap(), &lines);
        assert!(result.is_err()); // Should reject
        cleanup_files(&[&test_file]);
    }
    #[test]
    fn test_determine_block_markers() {
        assert!(determine_block_markers("rs").is_some());
        assert!(determine_block_markers("py").is_some());
        assert!(determine_block_markers("sh").is_none()); // No block comments
    }
    #[test]
    fn test_toggle_preserves_trailing_whitespace() {
        let test_file = create_test_file("test_trailing.rs", "code   \n");
        let result = toggle_basic_singleline_comment(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        let content = read_file_content(&test_file);
        assert_eq!(content, "// code   \n"); // Trailing spaces preserved
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_trailing.rs"),
        ]);
    }
}

// =================
// Indent / Unindent
// =================
/*

The standard
indent/unindent functionality
-- ctrl+brackets is standard --
is simpliar to but simpler than
comment-toggling
-- ctrl + / is standard --

Two functionalities:
- single line
- range of lines

 */
// ============================================================================
// INDENT/UNINDENT FUNCTIONALITY
// ============================================================================

/// Number of spaces to add/remove for indent/unindent operations
const INDENT_SPACES: usize = 4;

/// Errors that can occur during indent/unindent operations
///
/// All variants are Copy - no heap allocation, no string storage.
/// Reuses same design pattern as ToggleError for consistency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentError {
    /// The specified file was not found
    FileNotFound,

    /// The requested line index exceeds the file's line count
    LineNotFound { requested: usize, file_lines: usize },

    /// I/O operation failed
    IoError(IoOperation),

    /// Path conversion or manipulation error
    PathError,

    /// Line exceeds maximum safe length
    LineTooLong { line_number: usize, length: usize },

    /// Invalid line range (start >= end)
    InvalidLineRange,
}

impl std::fmt::Display for IndentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndentError::FileNotFound => write!(f, "File not found"),
            IndentError::LineNotFound {
                requested,
                file_lines,
            } => {
                write!(
                    f,
                    "Line {} not found (file has {} lines)",
                    requested, file_lines
                )
            }
            IndentError::IoError(op) => write!(f, "IO error: {:?}", op),
            IndentError::PathError => write!(f, "Path error"),
            IndentError::LineTooLong {
                line_number,
                length,
            } => {
                write!(f, "Line {} too long: {} bytes", line_number, length)
            }
            IndentError::InvalidLineRange => {
                write!(f, "Invalid line range (start must be < end)")
            }
        }
    }
}

impl std::error::Error for IndentError {}

/// Add 4 spaces to the start of a specific line
///
/// # Overview
/// Adds exactly 4 spaces at the beginning of the target line, regardless of
/// existing indentation. Works on any file type (language-agnostic).
///
/// # Arguments
/// * `file_path` - Path to the source file
/// * `line_number` - Zero-indexed line number to indent
///
/// # Returns
/// * `Ok(())` - Line indented successfully
/// * `Err(IndentError)` - Specific error code
///
/// # Safety
/// - Uses same backup system as toggle_comment
/// - Atomic file operations
/// - No heap allocation during processing
/// - Preserves line endings (LF/CRLF/none)
///
/// # Example
/// ```no_run
/// use toggle_comment_module::indent_line;
///
/// match indent_line("./src/main.rs", 5) {
///     Ok(()) => println!("Line indented"),
///     Err(e) => eprintln!("Failed: {:?}", e),
/// }
/// ```
///
/// # Behavior
/// ```text
/// Before: "code"
/// After:  "    code"
///
/// Before: "  code"  (already indented 2 spaces)
/// After:  "      code"  (now indented 6 spaces)
/// ```
pub fn indent_line(file_path: &str, line_number: usize) -> Result<(), IndentError> {
    // Convert to absolute path
    let absolute_path = match Path::new(file_path).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(IndentError::FileNotFound);
            }
            return Err(IndentError::PathError);
        }
    };

    // Get filename for backup naming
    let filename = match absolute_path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => return Err(IndentError::PathError),
    };

    // Create backup path in CWD (reuse same backup name as toggle_comment)
    let backup_filename = format!("backup_toggle_comment_{}", filename);
    let backup_path = PathBuf::from(&backup_filename);

    // Create backup copy of original file
    if let Err(_) = std::fs::copy(&absolute_path, &backup_path) {
        return Err(IndentError::IoError(IoOperation::Backup));
    }

    // Create working temp file in CWD
    let temp_filename = format!("temp_indent_{}_{}", std::process::id(), filename);
    let temp_path = PathBuf::from(&temp_filename);

    // Process file and indent target line
    let process_result = process_file_indent(&absolute_path, &temp_path, line_number);

    // Handle processing result
    match process_result {
        Ok(()) => {
            // Success: replace original with temp file
            if let Err(_) = std::fs::copy(&temp_path, &absolute_path) {
                let _ = std::fs::remove_file(&temp_path);
                return Err(IndentError::IoError(IoOperation::Replace));
            }

            // Clean up temp file
            if let Err(_) = std::fs::remove_file(&temp_path) {
                #[cfg(debug_assertions)]
                eprintln!("Warning: Failed to clean up temp file");
            }

            Ok(())
        }
        Err(e) => {
            // Failed: clean up temp file and return error
            let _ = std::fs::remove_file(&temp_path);
            Err(e)
        }
    }
}

/// Remove up to 4 spaces from the start of a specific line
///
/// # Overview
/// Removes up to 4 leading spaces from the target line. If the line has fewer
/// than 4 leading spaces, removes only what's there (0, 1, 2, or 3 spaces).
/// Non-space characters at line start are left unchanged.
///
/// # Arguments
/// * `file_path` - Path to the source file
/// * `line_number` - Zero-indexed line number to unindent
///
/// # Returns
/// * `Ok(())` - Line unindented successfully (even if no spaces removed)
/// * `Err(IndentError)` - Specific error code
///
/// # Safety
/// - Uses same backup system as toggle_comment
/// - Atomic file operations
/// - No heap allocation during processing
/// - Preserves line endings (LF/CRLF/none)
///
/// # Example
/// ```no_run
/// use toggle_comment_module::unindent_line;
///
/// match unindent_line("./src/main.rs", 5) {
///     Ok(()) => println!("Line unindented"),
///     Err(e) => eprintln!("Failed: {:?}", e),
/// }
/// ```
///
/// # Behavior
/// ```text
/// Before: "    code"  (4 spaces)
/// After:  "code"      (removed 4)
///
/// Before: "  code"    (2 spaces)
/// After:  "code"      (removed 2)
///
/// Before: "code"      (0 spaces)
/// After:  "code"      (removed 0 - no-op success)
///
/// Before: "\tcode"    (tab, not spaces)
/// After:  "\tcode"    (unchanged - only removes spaces)
/// ```
pub fn unindent_line(file_path: &str, line_number: usize) -> Result<(), IndentError> {
    // Convert to absolute path
    let absolute_path = match Path::new(file_path).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(IndentError::FileNotFound);
            }
            return Err(IndentError::PathError);
        }
    };

    // Get filename for backup naming
    let filename = match absolute_path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => return Err(IndentError::PathError),
    };

    // Create backup path in CWD
    let backup_filename = format!("backup_toggle_comment_{}", filename);
    let backup_path = PathBuf::from(&backup_filename);

    // Create backup copy
    if let Err(_) = std::fs::copy(&absolute_path, &backup_path) {
        return Err(IndentError::IoError(IoOperation::Backup));
    }

    // Create temp file
    let temp_filename = format!("temp_unindent_{}_{}", std::process::id(), filename);
    let temp_path = PathBuf::from(&temp_filename);

    // Process file and unindent target line
    let process_result = process_file_unindent(&absolute_path, &temp_path, line_number);

    // Handle result
    match process_result {
        Ok(()) => {
            // Success: replace original
            if let Err(_) = std::fs::copy(&temp_path, &absolute_path) {
                let _ = std::fs::remove_file(&temp_path);
                return Err(IndentError::IoError(IoOperation::Replace));
            }

            // Clean up temp
            if let Err(_) = std::fs::remove_file(&temp_path) {
                #[cfg(debug_assertions)]
                eprintln!("Warning: Failed to clean up temp file");
            }

            Ok(())
        }
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            Err(e)
        }
    }
}

/// Process file line-by-line, adding 4 spaces to target line
///
/// # Arguments
/// * `source_path` - Original file to read from
/// * `dest_path` - Temporary file to write modified content to
/// * `target_line` - Zero-indexed line number to indent
///
/// # Returns
/// * `Ok(())` - Processing succeeded, target line was found and indented
/// * `Err(IndentError)` - Processing failed
///
/// # Safety
/// - Pre-allocated buffers only
/// - Bounded line length checks
/// - No dynamic allocation during loop
fn process_file_indent(
    source_path: &Path,
    dest_path: &Path,
    target_line: usize,
) -> Result<(), IndentError> {
    // Open source file for reading
    let source_file = match File::open(source_path) {
        Ok(f) => f,
        Err(_) => return Err(IndentError::IoError(IoOperation::Open)),
    };

    let mut reader = BufReader::with_capacity(IO_BUFFER_SIZE, source_file);

    // Create destination file
    let dest_file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dest_path)
    {
        Ok(f) => f,
        Err(_) => return Err(IndentError::IoError(IoOperation::Create)),
    };

    let mut writer = BufWriter::with_capacity(IO_BUFFER_SIZE, dest_file);

    // Pre-allocate line buffer
    let mut line_buffer = Vec::with_capacity(MAX_LINE_LENGTH);

    // Track current line number
    let mut current_line: usize = 0;
    let mut found_target = false;

    // Safety limit for loop
    let line_limit = target_line.saturating_add(1000000);

    // Process file line by line
    loop {
        // Safety check: prevent unbounded loop
        if current_line > line_limit {
            return Err(IndentError::IoError(IoOperation::Read));
        }

        line_buffer.clear();

        let bytes_read = match reader.read_until(b'\n', &mut line_buffer) {
            Ok(n) => n,
            Err(_) => return Err(IndentError::IoError(IoOperation::Read)),
        };

        // End of file reached
        if bytes_read == 0 {
            break;
        }

        // Safety: check line length
        if line_buffer.len() > MAX_LINE_LENGTH {
            return Err(IndentError::LineTooLong {
                line_number: current_line,
                length: line_buffer.len(),
            });
        }

        // Check if this is our target line
        if current_line == target_line {
            found_target = true;

            // Add 4 spaces to start of line
            if let Err(e) = indent_single_line(&mut writer, &line_buffer) {
                return Err(e);
            }
        } else {
            // Copy line unchanged
            if let Err(_) = writer.write_all(&line_buffer) {
                return Err(IndentError::IoError(IoOperation::Write));
            }
        }

        current_line += 1;
    }

    // Flush writer
    if let Err(_) = writer.flush() {
        return Err(IndentError::IoError(IoOperation::Flush));
    }

    // Check if we found the target line
    if !found_target {
        return Err(IndentError::LineNotFound {
            requested: target_line,
            file_lines: current_line,
        });
    }

    Ok(())
}

/// Process file line-by-line, removing up to 4 spaces from target line
///
/// # Arguments
/// * `source_path` - Original file to read from
/// * `dest_path` - Temporary file to write modified content to
/// * `target_line` - Zero-indexed line number to unindent
///
/// # Returns
/// * `Ok(())` - Processing succeeded, target line was found and unindented
/// * `Err(IndentError)` - Processing failed
fn process_file_unindent(
    source_path: &Path,
    dest_path: &Path,
    target_line: usize,
) -> Result<(), IndentError> {
    // Open source file
    let source_file = match File::open(source_path) {
        Ok(f) => f,
        Err(_) => return Err(IndentError::IoError(IoOperation::Open)),
    };

    let mut reader = BufReader::with_capacity(IO_BUFFER_SIZE, source_file);

    // Create destination file
    let dest_file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dest_path)
    {
        Ok(f) => f,
        Err(_) => return Err(IndentError::IoError(IoOperation::Create)),
    };

    let mut writer = BufWriter::with_capacity(IO_BUFFER_SIZE, dest_file);

    let mut line_buffer = Vec::with_capacity(MAX_LINE_LENGTH);
    let mut current_line: usize = 0;
    let mut found_target = false;

    // Safety limit
    let line_limit = target_line.saturating_add(1000000);

    // Process file
    loop {
        // Safety check
        if current_line > line_limit {
            return Err(IndentError::IoError(IoOperation::Read));
        }

        line_buffer.clear();

        let bytes_read = match reader.read_until(b'\n', &mut line_buffer) {
            Ok(n) => n,
            Err(_) => return Err(IndentError::IoError(IoOperation::Read)),
        };

        if bytes_read == 0 {
            break;
        }

        // Safety: check line length
        if line_buffer.len() > MAX_LINE_LENGTH {
            return Err(IndentError::LineTooLong {
                line_number: current_line,
                length: line_buffer.len(),
            });
        }

        // Check if this is our target line
        if current_line == target_line {
            found_target = true;

            // Remove up to 4 spaces from start
            if let Err(e) = unindent_single_line(&mut writer, &line_buffer) {
                return Err(e);
            }
        } else {
            // Copy unchanged
            if let Err(_) = writer.write_all(&line_buffer) {
                return Err(IndentError::IoError(IoOperation::Write));
            }
        }

        current_line += 1;
    }

    // Flush writer
    if let Err(_) = writer.flush() {
        return Err(IndentError::IoError(IoOperation::Flush));
    }

    // Check if we found target
    if !found_target {
        return Err(IndentError::LineNotFound {
            requested: target_line,
            file_lines: current_line,
        });
    }

    Ok(())
}

/// Add 4 spaces to the start of a single line
///
/// # Arguments
/// * `writer` - Buffered writer to output modified line
/// * `line_buffer` - Complete line including newline
///
/// # Returns
/// * `Ok(())` - Line written successfully
/// * `Err(IndentError)` - Write failed
///
/// # Logic
/// - Separate line content from trailing newline
/// - Write 4 spaces
/// - Write original content
/// - Write original newline
pub fn indent_single_line(
    writer: &mut BufWriter<File>,
    line_buffer: &[u8],
) -> Result<(), IndentError> {
    // Separate content from newline
    let (content, newline) = if line_buffer.ends_with(b"\r\n") {
        (&line_buffer[..line_buffer.len() - 2], &b"\r\n"[..])
    } else if line_buffer.ends_with(b"\n") {
        (&line_buffer[..line_buffer.len() - 1], &b"\n"[..])
    } else {
        // No newline (last line of file)
        (line_buffer, &b""[..])
    };

    // Write 4 spaces
    if let Err(_) = writer.write_all(b"    ") {
        return Err(IndentError::IoError(IoOperation::Write));
    }

    // Write original content
    if let Err(_) = writer.write_all(content) {
        return Err(IndentError::IoError(IoOperation::Write));
    }

    // Write newline
    if !newline.is_empty() {
        if let Err(_) = writer.write_all(newline) {
            return Err(IndentError::IoError(IoOperation::Write));
        }
    }

    Ok(())
}

/// Remove up to 4 leading spaces from a single line
///
/// # Arguments
/// * `writer` - Buffered writer to output modified line
/// * `line_buffer` - Complete line including newline
///
/// # Returns
/// * `Ok(())` - Line written successfully
/// * `Err(IndentError)` - Write failed
///
/// # Logic
/// - Separate content from newline
/// - Count leading spaces (max 4)
/// - Write content starting after counted spaces
/// - Write original newline
pub fn unindent_single_line(
    writer: &mut BufWriter<File>,
    line_buffer: &[u8],
) -> Result<(), IndentError> {
    // Separate content from newline
    let (content, newline) = if line_buffer.ends_with(b"\r\n") {
        (&line_buffer[..line_buffer.len() - 2], &b"\r\n"[..])
    } else if line_buffer.ends_with(b"\n") {
        (&line_buffer[..line_buffer.len() - 1], &b"\n"[..])
    } else {
        (line_buffer, &b""[..])
    };

    // Count leading spaces (up to 4)
    let mut spaces_to_remove = 0;
    for &byte in content.iter() {
        if byte == b' ' && spaces_to_remove < INDENT_SPACES {
            spaces_to_remove += 1;
        } else {
            break;
        }
    }

    // Write content starting after the spaces we're removing
    if let Err(_) = writer.write_all(&content[spaces_to_remove..]) {
        return Err(IndentError::IoError(IoOperation::Write));
    }

    // Write newline
    if !newline.is_empty() {
        if let Err(_) = writer.write_all(newline) {
            return Err(IndentError::IoError(IoOperation::Write));
        }
    }

    Ok(())
}

// ============================================================================
// TESTS - Indent/Unindent Single Line
// ============================================================================

#[cfg(test)]
mod indent_tests {
    use super::*;

    /// Helper: create a temporary test file with given content
    fn create_test_file(filename: &str, content: &str) -> PathBuf {
        let path = PathBuf::from(filename);
        let mut file = File::create(&path).expect("Failed to create test file");
        file.write_all(content.as_bytes())
            .expect("Failed to write test file");
        path
    }

    /// Helper: read file content as string
    fn read_file_content(path: &Path) -> String {
        std::fs::read_to_string(path).expect("Failed to read file")
    }

    /// Helper: cleanup test files
    fn cleanup_files(paths: &[&Path]) {
        for path in paths {
            let _ = std::fs::remove_file(path);
        }
    }

    // ========================================
    // Indent Tests
    // ========================================

    #[test]
    fn test_indent_line_basic() {
        let test_file = create_test_file("test_indent_basic.txt", "code\n");
        let result = indent_line(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        let content = read_file_content(&test_file);
        assert_eq!(content, "    code\n");
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_indent_basic.txt"),
        ]);
    }

    #[test]
    fn test_indent_line_already_indented() {
        let test_file = create_test_file("test_indent_existing.txt", "  code\n");
        let result = indent_line(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        let content = read_file_content(&test_file);
        assert_eq!(content, "      code\n"); // 2 + 4 = 6 spaces
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_indent_existing.txt"),
        ]);
    }

    #[test]
    fn test_indent_line_empty() {
        let test_file = create_test_file("test_indent_empty.txt", "\n");
        let result = indent_line(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        let content = read_file_content(&test_file);
        assert_eq!(content, "    \n");
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_indent_empty.txt"),
        ]);
    }

    #[test]
    fn test_indent_line_no_newline() {
        let test_file = create_test_file("test_indent_no_newline.txt", "code");
        let result = indent_line(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        let content = read_file_content(&test_file);
        assert_eq!(content, "    code");
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_indent_no_newline.txt"),
        ]);
    }

    #[test]
    fn test_indent_line_crlf() {
        let test_file = create_test_file("test_indent_crlf.txt", "code\r\n");
        let result = indent_line(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        let content = read_file_content(&test_file);
        assert_eq!(content, "    code\r\n");
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_indent_crlf.txt"),
        ]);
    }

    #[test]
    fn test_indent_line_middle_of_file() {
        let content = "line 0\nline 1\nline 2\n";
        let test_file = create_test_file("test_indent_middle.txt", content);
        let result = indent_line(test_file.to_str().unwrap(), 1);
        assert!(result.is_ok());
        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "line 0\n    line 1\nline 2\n");
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_indent_middle.txt"),
        ]);
    }

    #[test]
    fn test_indent_line_not_found() {
        let test_file = create_test_file("test_indent_notfound.txt", "code\n");
        let result = indent_line(test_file.to_str().unwrap(), 10);
        assert!(matches!(result, Err(IndentError::LineNotFound { .. })));
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_indent_notfound.txt"),
        ]);
    }

    #[test]
    fn test_indent_line_file_not_found() {
        let result = indent_line("/nonexistent/file.txt", 0);
        assert!(matches!(result, Err(IndentError::FileNotFound)));
    }

    // ========================================
    // Unindent Tests
    // ========================================

    #[test]
    fn test_unindent_line_four_spaces() {
        let test_file = create_test_file("test_unindent_four.txt", "    code\n");
        let result = unindent_line(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        let content = read_file_content(&test_file);
        assert_eq!(content, "code\n");
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_unindent_four.txt"),
        ]);
    }

    #[test]
    fn test_unindent_line_two_spaces() {
        let test_file = create_test_file("test_unindent_two.txt", "  code\n");
        let result = unindent_line(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        let content = read_file_content(&test_file);
        assert_eq!(content, "code\n"); // Removed 2 spaces (all available)
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_unindent_two.txt"),
        ]);
    }

    #[test]
    fn test_unindent_line_no_spaces() {
        let test_file = create_test_file("test_unindent_none.txt", "code\n");
        let result = unindent_line(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok()); // Success (no-op)
        let content = read_file_content(&test_file);
        assert_eq!(content, "code\n"); // Unchanged
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_unindent_none.txt"),
        ]);
    }

    #[test]
    fn test_unindent_line_six_spaces() {
        let test_file = create_test_file("test_unindent_six.txt", "      code\n");
        let result = unindent_line(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        let content = read_file_content(&test_file);
        assert_eq!(content, "  code\n"); // Removed 4, left 2
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_unindent_six.txt"),
        ]);
    }

    #[test]
    fn test_unindent_line_tab() {
        let test_file = create_test_file("test_unindent_tab.txt", "\tcode\n");
        let result = unindent_line(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        let content = read_file_content(&test_file);
        assert_eq!(content, "\tcode\n"); // Tab unchanged (only removes spaces)
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_unindent_tab.txt"),
        ]);
    }

    #[test]
    fn test_unindent_line_spaces_and_tab() {
        let test_file = create_test_file("test_unindent_mixed.txt", "  \tcode\n");
        let result = unindent_line(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        let content = read_file_content(&test_file);
        assert_eq!(content, "\tcode\n"); // Removed 2 spaces, left tab
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_unindent_mixed.txt"),
        ]);
    }

    #[test]
    fn test_unindent_line_middle_of_file() {
        let content = "line 0\n    line 1\nline 2\n";
        let test_file = create_test_file("test_unindent_middle.txt", content);
        let result = unindent_line(test_file.to_str().unwrap(), 1);
        assert!(result.is_ok());
        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "line 0\nline 1\nline 2\n");
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_unindent_middle.txt"),
        ]);
    }

    #[test]
    fn test_unindent_line_empty() {
        let test_file = create_test_file("test_unindent_empty.txt", "    \n");
        let result = unindent_line(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());
        let content = read_file_content(&test_file);
        assert_eq!(content, "\n"); // All spaces removed
        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_unindent_empty.txt"),
        ]);
    }

    // ========================================
    // Round-trip Tests
    // ========================================

    #[test]
    fn test_indent_unindent_roundtrip() {
        let original = "code\n";
        let test_file = create_test_file("test_roundtrip.txt", original);

        // Indent
        let result1 = indent_line(test_file.to_str().unwrap(), 0);
        assert!(result1.is_ok());
        let content1 = read_file_content(&test_file);
        assert_eq!(content1, "    code\n");

        // Unindent back
        let result2 = unindent_line(test_file.to_str().unwrap(), 0);
        assert!(result2.is_ok());
        let content2 = read_file_content(&test_file);
        assert_eq!(content2, original);

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_roundtrip.txt"),
        ]);
    }

    #[test]
    fn test_unindent_idempotent() {
        // Unindenting line with no spaces should be no-op
        let original = "code\n";
        let test_file = create_test_file("test_unindent_idempotent.txt", original);

        let result1 = unindent_line(test_file.to_str().unwrap(), 0);
        assert!(result1.is_ok());

        let result2 = unindent_line(test_file.to_str().unwrap(), 0);
        assert!(result2.is_ok());

        let content = read_file_content(&test_file);
        assert_eq!(content, original);

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_unindent_idempotent.txt"),
        ]);
    }
}

// ============================================================================
// INDENT/UNINDENT RANGE FUNCTIONS
// ============================================================================

/// Add 4 spaces to the start of multiple lines (range, inclusive)
///
/// # Overview
/// Adds exactly 4 spaces at the beginning of each line in the specified range.
/// Both start_line and end_line are included in the operation.
///
/// # Arguments
/// * `file_path` - Path to the source file
/// * `start_line` - Zero-indexed first line to indent (inclusive)
/// * `end_line` - Zero-indexed last line to indent (inclusive)
///
/// # Returns
/// * `Ok(())` - Lines indented successfully
/// * `Err(IndentError)` - Specific error code
///
/// # Safety
/// - Uses same backup system as toggle_comment
/// - Atomic file operations
/// - No heap allocation during processing
/// - Preserves line endings (LF/CRLF/none)
///
/// # Example
/// ```no_run
/// use toggle_comment_module::indent_range;
///
/// // Indent lines 5 through 10 (inclusive)
/// match indent_range("./src/main.rs", 5, 10) {
///     Ok(()) => println!("Lines indented"),
///     Err(e) => eprintln!("Failed: {:?}", e),
/// }
/// ```
///
/// # Behavior
/// ```text
/// Input (lines 0-2):
/// line 0
/// line 1
/// line 2
///
/// After indent_range(path, 0, 2):
///     line 0
///     line 1
///     line 2
/// ```
pub fn indent_range(
    file_path: &str,
    start_line: usize,
    end_line: usize,
) -> Result<(), IndentError> {
    // Validate range
    if start_line > end_line {
        return Err(IndentError::InvalidLineRange);
    }

    // Convert to absolute path
    let absolute_path = match Path::new(file_path).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(IndentError::FileNotFound);
            }
            return Err(IndentError::PathError);
        }
    };

    // Get filename for backup naming
    let filename = match absolute_path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => return Err(IndentError::PathError),
    };

    // Create backup path in CWD
    let backup_filename = format!("backup_toggle_comment_{}", filename);
    let backup_path = PathBuf::from(&backup_filename);

    // Create backup copy
    if let Err(_) = std::fs::copy(&absolute_path, &backup_path) {
        return Err(IndentError::IoError(IoOperation::Backup));
    }

    // Create temp file
    let temp_filename = format!("temp_indent_range_{}_{}", std::process::id(), filename);
    let temp_path = PathBuf::from(&temp_filename);

    // Process file and indent range
    let process_result =
        process_file_indent_range(&absolute_path, &temp_path, start_line, end_line);

    // Handle result
    match process_result {
        Ok(()) => {
            // Success: replace original
            if let Err(_) = std::fs::copy(&temp_path, &absolute_path) {
                let _ = std::fs::remove_file(&temp_path);
                return Err(IndentError::IoError(IoOperation::Replace));
            }

            // Clean up temp
            if let Err(_) = std::fs::remove_file(&temp_path) {
                #[cfg(debug_assertions)]
                eprintln!("Warning: Failed to clean up temp file");
            }

            Ok(())
        }
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            Err(e)
        }
    }
}

/// Remove up to 4 spaces from the start of multiple lines (range, inclusive)
///
/// # Overview
/// Removes up to 4 leading spaces from each line in the specified range.
/// Both start_line and end_line are included in the operation.
///
/// # Arguments
/// * `file_path` - Path to the source file
/// * `start_line` - Zero-indexed first line to unindent (inclusive)
/// * `end_line` - Zero-indexed last line to unindent (inclusive)
///
/// # Returns
/// * `Ok(())` - Lines unindented successfully
/// * `Err(IndentError)` - Specific error code
///
/// # Safety
/// - Uses same backup system as toggle_comment
/// - Atomic file operations
/// - No heap allocation during processing
/// - Preserves line endings (LF/CRLF/none)
///
/// # Example
/// ```no_run
/// use toggle_comment_module::unindent_range;
///
/// // Unindent lines 5 through 10 (inclusive)
/// match unindent_range("./src/main.rs", 5, 10) {
///     Ok(()) => println!("Lines unindented"),
///     Err(e) => eprintln!("Failed: {:?}", e),
/// }
/// ```
///
/// # Behavior
/// ```text
/// Input (lines 0-2):
///     line 0
///     line 1
///     line 2
///
/// After unindent_range(path, 0, 2):
/// line 0
/// line 1
/// line 2
/// ```
pub fn unindent_range(
    file_path: &str,
    start_line: usize,
    end_line: usize,
) -> Result<(), IndentError> {
    // Validate range
    if start_line > end_line {
        return Err(IndentError::InvalidLineRange);
    }

    // Convert to absolute path
    let absolute_path = match Path::new(file_path).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(IndentError::FileNotFound);
            }
            return Err(IndentError::PathError);
        }
    };

    // Get filename for backup naming
    let filename = match absolute_path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => return Err(IndentError::PathError),
    };

    // Create backup path in CWD
    let backup_filename = format!("backup_toggle_comment_{}", filename);
    let backup_path = PathBuf::from(&backup_filename);

    // Create backup copy
    if let Err(_) = std::fs::copy(&absolute_path, &backup_path) {
        return Err(IndentError::IoError(IoOperation::Backup));
    }

    // Create temp file
    let temp_filename = format!("temp_unindent_range_{}_{}", std::process::id(), filename);
    let temp_path = PathBuf::from(&temp_filename);

    // Process file and unindent range
    let process_result =
        process_file_unindent_range(&absolute_path, &temp_path, start_line, end_line);

    // Handle result
    match process_result {
        Ok(()) => {
            // Success: replace original
            if let Err(_) = std::fs::copy(&temp_path, &absolute_path) {
                let _ = std::fs::remove_file(&temp_path);
                return Err(IndentError::IoError(IoOperation::Replace));
            }

            // Clean up temp
            if let Err(_) = std::fs::remove_file(&temp_path) {
                #[cfg(debug_assertions)]
                eprintln!("Warning: Failed to clean up temp file");
            }

            Ok(())
        }
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            Err(e)
        }
    }
}

/// Process file line-by-line, adding 4 spaces to lines in range
///
/// # Arguments
/// * `source_path` - Original file to read from
/// * `dest_path` - Temporary file to write modified content to
/// * `start_line` - First line to indent (inclusive)
/// * `end_line` - Last line to indent (inclusive)
///
/// # Returns
/// * `Ok(())` - Processing succeeded, all lines in range indented
/// * `Err(IndentError)` - Processing failed
///
/// # Safety
/// - Pre-allocated buffers only
/// - Bounded line length checks
/// - No dynamic allocation during loop
fn process_file_indent_range(
    source_path: &Path,
    dest_path: &Path,
    start_line: usize,
    end_line: usize,
) -> Result<(), IndentError> {
    // Open source file
    let source_file = match File::open(source_path) {
        Ok(f) => f,
        Err(_) => return Err(IndentError::IoError(IoOperation::Open)),
    };

    let mut reader = BufReader::with_capacity(IO_BUFFER_SIZE, source_file);

    // Create destination file
    let dest_file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dest_path)
    {
        Ok(f) => f,
        Err(_) => return Err(IndentError::IoError(IoOperation::Create)),
    };

    let mut writer = BufWriter::with_capacity(IO_BUFFER_SIZE, dest_file);

    let mut line_buffer = Vec::with_capacity(MAX_LINE_LENGTH);
    let mut current_line: usize = 0;
    let mut found_end_line = false;

    // Safety limit
    let line_limit = end_line.saturating_add(1000000);

    // Process file
    loop {
        // Safety check
        if current_line > line_limit {
            return Err(IndentError::IoError(IoOperation::Read));
        }

        line_buffer.clear();

        let bytes_read = match reader.read_until(b'\n', &mut line_buffer) {
            Ok(n) => n,
            Err(_) => return Err(IndentError::IoError(IoOperation::Read)),
        };

        if bytes_read == 0 {
            break;
        }

        // Safety: check line length
        if line_buffer.len() > MAX_LINE_LENGTH {
            return Err(IndentError::LineTooLong {
                line_number: current_line,
                length: line_buffer.len(),
            });
        }

        // Check if this line is in our target range
        if current_line >= start_line && current_line <= end_line {
            // This line is in range - indent it
            if let Err(e) = indent_single_line(&mut writer, &line_buffer) {
                return Err(e);
            }

            // Track if we've seen the end line
            if current_line == end_line {
                found_end_line = true;
            }
        } else {
            // Not in range - copy unchanged
            if let Err(_) = writer.write_all(&line_buffer) {
                return Err(IndentError::IoError(IoOperation::Write));
            }
        }

        current_line += 1;
    }

    // Flush writer
    if let Err(_) = writer.flush() {
        return Err(IndentError::IoError(IoOperation::Flush));
    }

    // Verify we found the end line
    if !found_end_line {
        return Err(IndentError::LineNotFound {
            requested: end_line,
            file_lines: current_line,
        });
    }

    Ok(())
}

/// Process file line-by-line, removing up to 4 spaces from lines in range
///
/// # Arguments
/// * `source_path` - Original file to read from
/// * `dest_path` - Temporary file to write modified content to
/// * `start_line` - First line to unindent (inclusive)
/// * `end_line` - Last line to unindent (inclusive)
///
/// # Returns
/// * `Ok(())` - Processing succeeded, all lines in range unindented
/// * `Err(IndentError)` - Processing failed
fn process_file_unindent_range(
    source_path: &Path,
    dest_path: &Path,
    start_line: usize,
    end_line: usize,
) -> Result<(), IndentError> {
    // Open source file
    let source_file = match File::open(source_path) {
        Ok(f) => f,
        Err(_) => return Err(IndentError::IoError(IoOperation::Open)),
    };

    let mut reader = BufReader::with_capacity(IO_BUFFER_SIZE, source_file);

    // Create destination file
    let dest_file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dest_path)
    {
        Ok(f) => f,
        Err(_) => return Err(IndentError::IoError(IoOperation::Create)),
    };

    let mut writer = BufWriter::with_capacity(IO_BUFFER_SIZE, dest_file);

    let mut line_buffer = Vec::with_capacity(MAX_LINE_LENGTH);
    let mut current_line: usize = 0;
    let mut found_end_line = false;

    // Safety limit
    let line_limit = end_line.saturating_add(1000000);

    // Process file
    loop {
        // Safety check
        if current_line > line_limit {
            return Err(IndentError::IoError(IoOperation::Read));
        }

        line_buffer.clear();

        let bytes_read = match reader.read_until(b'\n', &mut line_buffer) {
            Ok(n) => n,
            Err(_) => return Err(IndentError::IoError(IoOperation::Read)),
        };

        if bytes_read == 0 {
            break;
        }

        // Safety: check line length
        if line_buffer.len() > MAX_LINE_LENGTH {
            return Err(IndentError::LineTooLong {
                line_number: current_line,
                length: line_buffer.len(),
            });
        }

        // Check if this line is in our target range
        if current_line >= start_line && current_line <= end_line {
            // This line is in range - unindent it
            if let Err(e) = unindent_single_line(&mut writer, &line_buffer) {
                return Err(e);
            }

            // Track if we've seen the end line
            if current_line == end_line {
                found_end_line = true;
            }
        } else {
            // Not in range - copy unchanged
            if let Err(_) = writer.write_all(&line_buffer) {
                return Err(IndentError::IoError(IoOperation::Write));
            }
        }

        current_line += 1;
    }

    // Flush writer
    if let Err(_) = writer.flush() {
        return Err(IndentError::IoError(IoOperation::Flush));
    }

    // Verify we found the end line
    if !found_end_line {
        return Err(IndentError::LineNotFound {
            requested: end_line,
            file_lines: current_line,
        });
    }

    Ok(())
}

// ============================================================================
// TESTS - Indent/Unindent Range
// ============================================================================

#[cfg(test)]
mod indent_range_tests {
    use super::*;

    /// Helper: create a temporary test file with given content
    fn create_test_file(filename: &str, content: &str) -> PathBuf {
        let path = PathBuf::from(filename);
        let mut file = File::create(&path).expect("Failed to create test file");
        file.write_all(content.as_bytes())
            .expect("Failed to write test file");
        path
    }

    /// Helper: read file content as string
    fn read_file_content(path: &Path) -> String {
        std::fs::read_to_string(path).expect("Failed to read file")
    }

    /// Helper: cleanup test files
    fn cleanup_files(paths: &[&Path]) {
        for path in paths {
            let _ = std::fs::remove_file(path);
        }
    }

    // ========================================
    // Indent Range Tests
    // ========================================

    #[test]
    fn test_indent_range_basic() {
        let content = "line 0\nline 1\nline 2\n";
        let test_file = create_test_file("test_indent_range_basic.txt", content);

        let result = indent_range(test_file.to_str().unwrap(), 0, 2);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "    line 0\n    line 1\n    line 2\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_indent_range_basic.txt"),
        ]);
    }

    #[test]
    fn test_indent_range_middle() {
        let content = "line 0\nline 1\nline 2\nline 3\n";
        let test_file = create_test_file("test_indent_range_middle.txt", content);

        let result = indent_range(test_file.to_str().unwrap(), 1, 2);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "line 0\n    line 1\n    line 2\nline 3\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_indent_range_middle.txt"),
        ]);
    }

    #[test]
    fn test_indent_range_single_line() {
        let content = "line 0\nline 1\nline 2\n";
        let test_file = create_test_file("test_indent_range_single.txt", content);

        // Range of just one line (start == end)
        let result = indent_range(test_file.to_str().unwrap(), 1, 1);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "line 0\n    line 1\nline 2\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_indent_range_single.txt"),
        ]);
    }

    #[test]
    fn test_indent_range_invalid_range() {
        let content = "line 0\nline 1\n";
        let test_file = create_test_file("test_indent_range_invalid.txt", content);

        // start > end
        let result = indent_range(test_file.to_str().unwrap(), 2, 1);
        assert!(matches!(result, Err(IndentError::InvalidLineRange)));

        cleanup_files(&[&test_file]);
    }

    #[test]
    fn test_indent_range_beyond_eof() {
        let content = "line 0\nline 1\n";
        let test_file = create_test_file("test_indent_range_eof.txt", content);

        // end_line beyond file
        let result = indent_range(test_file.to_str().unwrap(), 0, 10);
        assert!(matches!(result, Err(IndentError::LineNotFound { .. })));

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_indent_range_eof.txt"),
        ]);
    }

    #[test]
    fn test_indent_range_preserves_outside() {
        let content = "line 0\nline 1\nline 2\nline 3\nline 4\n";
        let test_file = create_test_file("test_indent_range_preserve.txt", content);

        let result = indent_range(test_file.to_str().unwrap(), 1, 3);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(
            new_content,
            "line 0\n    line 1\n    line 2\n    line 3\nline 4\n"
        );

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_indent_range_preserve.txt"),
        ]);
    }

    // ========================================
    // Unindent Range Tests
    // ========================================

    #[test]
    fn test_unindent_range_basic() {
        let content = "    line 0\n    line 1\n    line 2\n";
        let test_file = create_test_file("test_unindent_range_basic.txt", content);

        let result = unindent_range(test_file.to_str().unwrap(), 0, 2);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "line 0\nline 1\nline 2\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_unindent_range_basic.txt"),
        ]);
    }

    #[test]
    fn test_unindent_range_middle() {
        let content = "line 0\n    line 1\n    line 2\nline 3\n";
        let test_file = create_test_file("test_unindent_range_middle.txt", content);

        let result = unindent_range(test_file.to_str().unwrap(), 1, 2);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "line 0\nline 1\nline 2\nline 3\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_unindent_range_middle.txt"),
        ]);
    }

    #[test]
    fn test_unindent_range_mixed_indents() {
        let content = "    line 0\n  line 1\nline 2\n      line 3\n";
        let test_file = create_test_file("test_unindent_range_mixed.txt", content);

        let result = unindent_range(test_file.to_str().unwrap(), 0, 3);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        // Removes: 4, 2, 0, 4 spaces respectively
        assert_eq!(new_content, "line 0\nline 1\nline 2\n  line 3\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_unindent_range_mixed.txt"),
        ]);
    }

    #[test]
    fn test_unindent_range_single_line() {
        let content = "line 0\n    line 1\nline 2\n";
        let test_file = create_test_file("test_unindent_range_single.txt", content);

        // Range of just one line
        let result = unindent_range(test_file.to_str().unwrap(), 1, 1);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "line 0\nline 1\nline 2\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_unindent_range_single.txt"),
        ]);
    }

    #[test]
    fn test_unindent_range_invalid_range() {
        let content = "line 0\nline 1\n";
        let test_file = create_test_file("test_unindent_range_invalid.txt", content);

        // start > end
        let result = unindent_range(test_file.to_str().unwrap(), 2, 1);
        assert!(matches!(result, Err(IndentError::InvalidLineRange)));

        cleanup_files(&[&test_file]);
    }

    // ========================================
    // Round-trip Tests
    // ========================================

    #[test]
    fn test_indent_unindent_range_roundtrip() {
        let original = "line 0\nline 1\nline 2\n";
        let test_file = create_test_file("test_range_roundtrip.txt", original);

        // Indent range
        let result1 = indent_range(test_file.to_str().unwrap(), 0, 2);
        assert!(result1.is_ok());

        let content1 = read_file_content(&test_file);
        assert_eq!(content1, "    line 0\n    line 1\n    line 2\n");

        // Unindent back
        let result2 = unindent_range(test_file.to_str().unwrap(), 0, 2);
        assert!(result2.is_ok());

        let content2 = read_file_content(&test_file);
        assert_eq!(content2, original);

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_range_roundtrip.txt"),
        ]);
    }

    #[test]
    fn test_indent_range_preserves_line_endings() {
        let content = "line 0\r\nline 1\r\nline 2\r\n";
        let test_file = create_test_file("test_indent_range_crlf.txt", content);

        let result = indent_range(test_file.to_str().unwrap(), 0, 2);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "    line 0\r\n    line 1\r\n    line 2\r\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_indent_range_crlf.txt"),
        ]);
    }

    #[test]
    fn test_unindent_range_no_spaces() {
        let content = "line 0\nline 1\nline 2\n";
        let test_file = create_test_file("test_unindent_range_noop.txt", content);

        // Should succeed (no-op)
        let result = unindent_range(test_file.to_str().unwrap(), 0, 2);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, content);

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_unindent_range_noop.txt"),
        ]);
    }
}
