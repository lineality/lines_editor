//! # toggle_comment_indent_module.rs
//!
//! https://github.com/lineality/toggle_comment_indent
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
//! - **Specific error types**: `ToggleCommentError` enum provides detailed failure reasons
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
//!  ## Toggle Single Line Comment
//!  ```rust
//!  use toggle_comment_indent_module::toggle_basic_singleline_comment;
//!
//!  // Auto-detects `//` or `#` from file extension
//!  toggle_basic_singleline_comment("./script.py", 5)?;   // → `// code`
//!  toggle_basic_singleline_comment("./script.py", 3)?; // → `# code`
//!  ```
//!
//!  ## Toggle Rust Docstring
//!  ```rust
//!  use toggle_comment_indent_module::toggle_rust_docstring_singleline_comment;
//!
//! // Use `///` instead of `//`
//! toggle_rust_docstring_singleline_comment("./script.py", 10)?;
//! ```
//!
//! ## Toggle Block Comments
//! ```rust
//! use toggle_comment_indent_module::toggle_block_comment;
//!
//! // Automatically add/remove markers around lines 5-10
//! // Detects: /* */ for C/Rust, """ for Python
//! toggle_block_comment("./script.py", 5, 10)?;
//! ```
//!
//! ## Batch Toggle Multiple Lines
//! ```rust
//! use toggle_comment_indent_module::toggle_multiple_basic_comments;
//!
//! // Toggle lines 5, 10, 15, 20 in one pass
//! let lines = [5, 10, 15, 20];
//! toggle_multiple_basic_comments("./script.py", &lines)?;
//! ```
//!
// ## Indent Single Line
//!
//! Add 4 spaces to the start of a line:
//!
//! ```rust
//! use toggle_comment_indent_module::indent_line;
//!
//! // Indent line 5 of a file
//! indent_line("./script.py", 5)?;
//! ```
//!
//! ## Unindent Single Line
//!
//! Remove up to 4 spaces from the start of a line:
//!
//!```rust
//!use toggle_comment_indent_module::unindent_line;
//!
//!// Unindent line 5 of a file
//!unindent_line("./script.py", 5)?;
//!```
//!
//!## Indent Range
//!
//!Add 4 spaces to the start of multiple lines (inclusive range):
//!
//!```rust
//!use toggle_comment_indent_module::indent_range;
//!
//!// Indent lines 5 through 15 (inclusive)
//!indent_range("./script.py", 5, 15)?;
//!```
//!
//!## Unindent Range
//!
//!Remove up to 4 spaces from the start of multiple lines (inclusive range):
//!
//!```rust
//!use toggle_comment_indent_module::unindent_range;
//!
//!// Unindent lines 5 through 15 (inclusive)
//!unindent_range("./script.py", 5, 15)?;
//!```
//!
//!## Toggle Block Range Standard-Comment
//!```rust
//!use toggle_comment_indent_module::execute_range_toggle_basic;
//!
//!execute_range_toggle_basic(file_path, start_line, end_line)
//!```
//!
//!## ~Toggle Block Rust-Docstring
//!```rust
//!use toggle_comment_indent_module::execute_range_toggle_docstring;
//!
//!execute_range_toggle_docstring(file_path, start_line, end_line)
//!```
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
//! The `ToggleCommentError` enum provides specific error information:
//! - `FileNotFound`: Specified file does not exist
//! - `NoExtension`: File has no extension (cannot determine comment type)
//! - `UnsupportedExtension`: Extension not recognized for any comment mode
//! - `LineNotFound { requested, file_lines }`: Target line beyond EOF
//! - `IoError(operation)`: I/O failure during backup, read, write, etc.
//! - `PathError`: Filesystem path manipulation failed
//! - `LineTooLong { line_number, length }`: Line exceeds 1MB limit
//! - `InconsistentBlockMarkers`: Only one block marker found (not both)
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
# Sample main.rs file

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
mod toggle_comment_indent_module;
use toggle_comment_indent_module::{
    ToggleIndentError, ToggleCommentError, indent_line, indent_range, toggle_basic_singleline_comment,
    toggle_block_comment, toggle_multiple_basic_comments, toggle_multiple_singline_docstrings,
    toggle_range_basic_comments, toggle_range_rust_docstring,
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

/// Execute range toggle - basic comments
fn execute_range_toggle_basic(file_path: &str, start_line: usize, end_line: usize) -> i32 {
    match toggle_range_basic_comments(file_path, start_line, end_line) {
        Ok(()) => {
            println!(
                "Successfully toggled comment range (lines {}-{})",
                start_line, end_line
            );
            0
        }
        Err(e) => {
            eprintln!("Error toggling range {}: {}", file_path, e);
            error_to_exit_code(e)
        }
    }
}

/// Execute range toggle - rust docstrings
fn execute_range_toggle_docstring(file_path: &str, start_line: usize, end_line: usize) -> i32 {
    match toggle_range_rust_docstring(file_path, start_line, end_line) {
        Ok(()) => {
            println!(
                "Successfully toggled docstring range (lines {}-{})",
                start_line, end_line
            );
            0
        }
        Err(e) => {
            eprintln!("Error toggling docstring range {}: {}", file_path, e);
            error_to_exit_code(e)
        }
    }
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

/// Convert ToggleCommentError to exit code
///
/// # Arguments
/// * `error` - The error to convert
///
/// # Returns
/// * Exit code (2-10)
fn error_to_exit_code(error: ToggleCommentError) -> i32 {
    match error {
        ToggleCommentError::FileNotFound => 2,
        ToggleCommentError::NoExtension => 3,
        ToggleCommentError::UnsupportedExtension => 4,
        ToggleCommentError::LineNotFound { .. } => 5,
        ToggleCommentError::IoError(_) => 6,
        ToggleCommentError::PathError => 7,
        ToggleCommentError::LineTooLong { .. } => 8,
        ToggleCommentError::InconsistentBlockMarkers => 9,
        ToggleCommentError::RangeTooLarge { .. } => 10,
    }
}

/// Convert ToggleIndentError to exit code
///
/// # Arguments
/// * `error` - The error to convert
///
/// # Returns
/// * Exit code (2-10, same mapping as ToggleCommentError where applicable)
fn indent_error_to_exit_code(error: ToggleIndentError) -> i32 {
    match error {
        ToggleIndentError::FileNotFound => 2,
        ToggleIndentError::LineNotFound { .. } => 5,
        ToggleIndentError::IoError(_) => 6,
        ToggleIndentError::PathError => 7,
        ToggleIndentError::LineTooLong { .. } => 8,
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
            "--toggle-range-comment-basic" => {
// Expect: --toggle-range-comment-basic <file> <start_line> <end_line>
                if args.len() != 5 {
                    eprintln!(
                        "Error: --toggle-range-comment-basic requires <file_path> <start_line> <end_line>"
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

// Note: No validation needed - function auto-sorts and validates
                execute_range_toggle_basic(file_path, start_line, end_line)
            }

            "--toggle-range-rust-docstring" => {
// Expect: --toggle-range-rust-docstring <file> <start_line> <end_line>
                if args.len() != 5 {
                    eprintln!(
                        "Error: --toggle-range-rust-docstring requires <file_path> <start_line> <end_line>"
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

// Note: No validation needed - function auto-sorts and validates
                execute_range_toggle_docstring(file_path, start_line, end_line)
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
- Always best practice.
- Always extensive doc strings: what the code is doing with project context
- Always clear comments.
- Always cargo tests (where possible).
- Never remove documentation.
- Always clear, meaningful, unique names (e.g. variables, functions).
- Always absolute file paths.
- Always error handling.
- Never unsafe code.
- Never use unwrap.

- Load what is needed when it is needed: Do not ever load a whole file or line, rarely load a whole anything. increment and load only what is required pragmatically. Do not fill 'state' with every possible piece of un-used information. Do not insecurity output information broadly in the case of errors and exceptions.

- Always defensive best practice
- Always error and exception handling: Every part of code, every process, function, and operation will fail at some point, if only because of cosmic-ray bit-flips (which are common), hardware failure, power-supply failure, adversarial attacks, etc. There must always be fail-safe error handling where production-release-build code handles issues and moves on without panic-crashing ever. Every failure must be handled smoothly: let it fail and move on.

Somehow there seems to be no clear vocabulary for 'Do not stop.' When you come to something to handle, handle it:
- Handle and move on: Do not halt the program.
- Handle and move on: Do not terminate the program.
- Handle and move on: Do not exit the program.
- Handle and move on: Do not crash the program.
- Handle and move on: Do not panic the program.
- Handle and move on: Do not coredump the program.
- Handle and move on: Do not stop the program.
- Handle and move on: Do not finish the program.

Comments and docs for functions and groups of functions must include project level information: To paraphrase Jack Welch, "The most dangerous thing in the world is a flawless operation that should never have been done in the first place." For projects, functions are not pure platonic abstractions; the project has a need that the function is or is not meeting. It happens constantly that a function does the wrong thing well and so this 'bug' is never detected. Project-level documentation and logic-level documentation are two different things that must both exist such that discrepancies must be identifiable; Project-level documentation, logic-level documentation, and the code, must align and align with user-needs, real conditions, and future conditions.

Safety, reliability, maintainability, fail-safe, communication-documentation, are the goals: not ideology, aesthetics, popularity, momentum-tradition, bad habits, convenience, nihilism, lazyness, lack of impulse control, etc.

## No third party libraries (or very strictly avoid third party libraries where possible).

## Rule of Thumb, ideals not absolute rules: Follow NASA's 'Power of 10 rules' where possible and sensible (as updated for 2025 and Rust (not narrowly 2006 c for embedded systems):
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


Avoid heap for error messages and for all things:
Is heap used for error messages because that is THE best way, the most secure, the most efficient, proper separate of debug testing vs. secure production code?
Or is heap used because of oversights and apathy: "it's future dev's problem, let's party."
We can use heap in debug/test modes/builds only.
Production software must not insecurely output debug diagnostics.
Debug information must not be included in production builds: "developers accidentally left development code in the software" is a classic error (not a desired design spec) that routinely leads to security and other issues. That is NOT supposed to happen. It is not coherent to insist the open ended heap output 'must' or 'should' be in a production build.

This is central to the question about testing vs. a pedantic ban on conditional compilation; not putting full traceback insecurity into production code is not a different operational process logic tree for process operations.

Just like with the pedantic "all loops being bounded" rule, there is a fundamental exception: always-on loops must be the opposite.
With conditional compilations: code NEVER to EVER be in production-builds MUST be always "conditionally" excluded. This is not an OS conditional compilation or a hardware conditional compilation. This is an 'unsafe-testing-only or safe-production-code' condition.

Error messages and error outcomes in 'production' 'release' (real-use, not debug/testing) must not ever contain any information that could be a security vulnerability or attack surface. Failing to remove debugging inspection is a major category of security and hygiene problems.

Security: Error messages in production must NOT contain:
- File paths (can reveal system structure)
- File contents
- environment variables
- user, file, state, data
- internal implementation details
- etc.

All debug-prints not for production must be tagged with
```
#[cfg(debug_assertions)]
```

Production output following an error must be managed and defined, not not open to whatever an api or OS-call wants to dump out.

6. Manage ownership and borrowing

7. Manage return values:
- use null-void return values
- check non-void-null returns

8. Navigate debugging and testing on the one hand and not-dangerous conditional-compilation on the other hand:
- Here 'conditional compilation' is interpreted as significant changes to the overall 'tree' of operation depending on build settings/conditions, such as using different modules and basal functions. E.g. "GDPR compliance mode compilation"
- Any LLVM type compilation or build-flag will modify compilation details, but not the target tree logic of what the software does (arguably).
- 2025+ "compilation" and "conditions" cannot be simplistically compared with single-architecture 1970 pdp-11-only C or similar embedded device compilation.

9. Communicate:
- Use doc strings; use comments.
- Document use-cases, edge-cases, and policies (These are project specific and cannot be telepathed from generic micro-function code. When a Mars satellite failed because one team used SI-metric units and another team did not, that problem could not have been detected by looking at, and auditing, any individual function in isolation without documentation. Breaking a process into innumerable undocumented micro-functions can make scope and policy impossible to track. To paraphrase Jack Welch: "The most dangerous thing in the world is a flawless operation that should never have been done in the first place.")

10. Use state-less operations when possible:
- a seemingly invisibly small increase in state often completely destroys projects
- expanding state destroys projects with unmaintainable over-reach

Vigilance: We should help support users and developers and the people who depend upon maintainable software. Maintainable code supports the future for us all.

*/

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Seek, Write};
use std::path::{Path, PathBuf};

/// Buffer size for file I/O operations - pre-allocated, stack-friendly size
const IO_BUFFER_SIZE: usize = 8192;

// // Maximum line length we'll process - safety bound
// const MAX_LINE_LENGTH: usize = 1_000_000; // 64KB per line max

// ============================================================================
// ERROR SECTION: ERROR HANDLING SYSTEM (start)
// ============================================================================

/// Errors that can occur during comment toggling operations
///
/// All variants are Copy - no heap allocation, no string storage.
/// Caller provides context (file paths, etc.) they already have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleCommentError {
    /// The specified file was not found
    FileNotFound,

    /// File has no extension
    NoExtension,

    /// The requested line index exceeds the file's line count
    LineNotFound { requested: usize, file_lines: usize },

    /// I/O operation failed
    IoError(IoOperation),

    /// Path conversion or manipulation error
    PathError,
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

impl std::fmt::Display for ToggleCommentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToggleCommentError::FileNotFound => write!(f, "File not found"),
            ToggleCommentError::NoExtension => write!(f, "No file extension"),
            ToggleCommentError::LineNotFound {
                requested,
                file_lines,
            } => {
                write!(
                    f,
                    "Line {} not found (file has {} lines)",
                    requested, file_lines
                )
            }
            ToggleCommentError::IoError(op) => write!(f, "IO error: {:?}", op),
            ToggleCommentError::PathError => write!(f, "Path error"),
        }
    }
}

impl std::error::Error for ToggleCommentError {}

// ============================================================================
// ERROR SECTION: ERROR HANDLING SYSTEM (end)
// ============================================================================

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

// impl CommentFlag {
//     /// Get the byte slice representation of the comment flag
//     fn as_bytes(&self) -> &'static [u8] {
//         match self {
//             CommentFlag::TripppleSlash => b"///",
//             CommentFlag::DoubleSlash => b"//",
//             CommentFlag::Hash => b"#",
//         }
//     }

//     /// Get the string representation of the comment flag
//     fn as_str(&self) -> &'static str {
//         match self {
//             CommentFlag::TripppleSlash => "///",
//             CommentFlag::DoubleSlash => "//",
//             CommentFlag::Hash => "#",
//         }
//     }
// }

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

// ============================================================================
// BYTE-WISE TOGGLE OPERATIONS (IMPLEMENTATION)
// ============================================================================
//
// ## Architecture
// This module implements comment toggling using single-byte operations
// with no heap allocation and no line-loading into memory.
//
// Three core operations:
// 1. find_and_detect_tag_state() - Combined find+detect in one pass
// 2. write_toggled_file_bytewise() - Three-part byte-by-byte copy with toggle
// 3. toggle_basic_singleline_comment() - Refactored wrapper using bytewise ops
//
// ## Memory Usage
// - Stack only: single byte buffer [u8; 1]
// - No Vec, no String, no heap allocations during processing
// - Total working memory: 1 byte
//
// ## Safety
// - All loops bounded by MAX_BYTE_ITERATIONS
// - Stops at newline during tag detection
// - Preserves CRLF vs LF line endings
// - Returns Ok(None) for line not found (not an error)
//
// ============================================================================

/// Maximum bytes to read before safety abort
/// Allows ~1GB files while preventing infinite loops
const MAX_BYTE_ITERATIONS: u64 = 1_000_000_000;

/// Combined operation: Find line start position AND detect tag state
///
/// # Overview
/// Performs find+detect in a single file pass. After finding target line,
/// checks COLUMN 0 for comment tag pattern. No indentation handling.
///
/// # Detection Rule (SIMPLE)
/// Read first bytes of line starting at column 0:
/// - "// " → HAS_TAG
/// - "# "  → HAS_TAG
/// - "/// " → HAS_TAG
/// - Anything else → NO_TAG
///
/// No space skipping. No indentation detection. Column 0 only.
fn find_and_detect_tag_state(
    file_path: &str,
    target_line: usize,
    comment_flag: CommentFlag,
) -> Result<Option<(u64, bool)>, ToggleCommentError> {
    // Open file for reading
    let mut file = match File::open(file_path) {
        Ok(f) => f,
        Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Open)),
    };

    let mut byte_bucket: [u8; 1] = [0u8; 1];
    let mut byte_position: u64 = 0;
    let mut current_line: usize = 0;
    let line_start_pos: u64;

    // ===========================================
    // PHASE 1: Find target line start position
    // ===========================================

    if target_line == 0 {
        line_start_pos = 0;
    } else {
        loop {
            if byte_position >= MAX_BYTE_ITERATIONS {
                return Err(ToggleCommentError::IoError(IoOperation::Read));
            }

            let bytes_read = match file.read(&mut byte_bucket) {
                Ok(n) => n,
                Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Read)),
            };

            if bytes_read == 0 {
                return Ok(None); // Line not found
            }

            if byte_bucket[0] == b'\n' {
                current_line += 1;
                if current_line == target_line {
                    line_start_pos = byte_position + 1;
                    break;
                }
            }

            byte_position += 1;
        }
    }

    // ===========================================
    // PHASE 2: Detect tag at column 0 ONLY
    // ===========================================

    // Seek to line start
    if let Err(_) = file.seek(std::io::SeekFrom::Start(line_start_pos)) {
        return Err(ToggleCommentError::IoError(IoOperation::Read));
    }

    // Read first byte at column 0
    let bytes_read = match file.read(&mut byte_bucket) {
        Ok(n) => n,
        Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Read)),
    };

    // EOF or empty line
    if bytes_read == 0 || byte_bucket[0] == b'\n' {
        return Ok(Some((line_start_pos, false)));
    }

    // Check if first byte matches start of tag
    let has_tag = match comment_flag {
        CommentFlag::Hash => {
            // Pattern: "# " at column 0
            if byte_bucket[0] != b'#' {
                false
            } else {
                // Read next byte - should be space
                match file.read(&mut byte_bucket) {
                    Ok(0) => false,
                    Ok(_) => byte_bucket[0] == b' ',
                    Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Read)),
                }
            }
        }

        CommentFlag::DoubleSlash => {
            // Pattern: "// " at column 0
            if byte_bucket[0] != b'/' {
                false
            } else {
                // Read second '/'
                match file.read(&mut byte_bucket) {
                    Ok(0) => false,
                    Ok(_) => {
                        if byte_bucket[0] != b'/' {
                            false
                        } else {
                            // Read space
                            match file.read(&mut byte_bucket) {
                                Ok(0) => false,
                                Ok(_) => byte_bucket[0] == b' ',
                                Err(_) => {
                                    return Err(ToggleCommentError::IoError(IoOperation::Read));
                                }
                            }
                        }
                    }
                    Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Read)),
                }
            }
        }

        CommentFlag::TripppleSlash => {
            // Pattern: "/// " at column 0
            if byte_bucket[0] != b'/' {
                false
            } else {
                // Read second '/'
                match file.read(&mut byte_bucket) {
                    Ok(0) => false,
                    Ok(_) => {
                        if byte_bucket[0] != b'/' {
                            false
                        } else {
                            // Read third '/'
                            match file.read(&mut byte_bucket) {
                                Ok(0) => false,
                                Ok(_) => {
                                    if byte_bucket[0] != b'/' {
                                        false
                                    } else {
                                        // Read space
                                        match file.read(&mut byte_bucket) {
                                            Ok(0) => false,
                                            Ok(_) => byte_bucket[0] == b' ',
                                            Err(_) => {
                                                return Err(ToggleCommentError::IoError(
                                                    IoOperation::Read,
                                                ));
                                            }
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(ToggleCommentError::IoError(IoOperation::Read));
                                }
                            }
                        }
                    }
                    Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Read)),
                }
            }
        }
    };

    Ok(Some((line_start_pos, has_tag)))
}

/// Write file with one line toggled using byte-by-byte operations
///
/// # Overview
/// Copies source file to destination with single line modified.
/// All tag operations at COLUMN 0 only. No indentation handling.
///
/// # Add Mode (has_tag = false)
/// Write tag at column 0, then copy rest of line
///
/// # Remove Mode (has_tag = true)
/// Skip tag bytes at column 0, then copy rest of line
fn write_toggled_file_bytewise(
    source_path: &Path,
    dest_path: &Path,
    target_line: usize,
    line_start_pos: u64,
    has_tag: bool,
    comment_flag: CommentFlag,
) -> Result<(), ToggleCommentError> {
    let mut source_file = match File::open(source_path) {
        Ok(f) => f,
        Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Open)),
    };

    let dest_file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dest_path)
    {
        Ok(f) => f,
        Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Create)),
    };

    let mut writer = BufWriter::with_capacity(IO_BUFFER_SIZE, dest_file);
    let mut byte_bucket: [u8; 1] = [0u8; 1];
    let mut byte_position: u64 = 0;

    // ===========================================
    // PART A: Copy bytes before target line
    // ===========================================

    while byte_position < line_start_pos {
        if byte_position >= MAX_BYTE_ITERATIONS {
            return Err(ToggleCommentError::IoError(IoOperation::Read));
        }

        let bytes_read = match source_file.read(&mut byte_bucket) {
            Ok(n) => n,
            Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Read)),
        };

        if bytes_read == 0 {
            return Err(ToggleCommentError::LineNotFound {
                requested: target_line,
                file_lines: 0,
            });
        }

        if let Err(_) = writer.write_all(&byte_bucket) {
            return Err(ToggleCommentError::IoError(IoOperation::Write));
        }

        byte_position += 1;
    }

    // ===========================================
    // PART B: Toggle target line at column 0
    // ===========================================

    if has_tag {
        // REMOVE MODE: Skip tag bytes at column 0
        let bytes_to_skip = match comment_flag {
            CommentFlag::Hash => 2,          // "# "
            CommentFlag::DoubleSlash => 3,   // "// "
            CommentFlag::TripppleSlash => 4, // "/// "
        };

        // Skip the tag bytes
        for _ in 0..bytes_to_skip {
            if let Err(_) = source_file.read(&mut byte_bucket) {
                return Err(ToggleCommentError::IoError(IoOperation::Read));
            }
        }

        // Copy rest of line
        loop {
            let bytes_read = match source_file.read(&mut byte_bucket) {
                Ok(n) => n,
                Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Read)),
            };

            if bytes_read == 0 {
                break; // EOF
            }

            if let Err(_) = writer.write_all(&byte_bucket) {
                return Err(ToggleCommentError::IoError(IoOperation::Write));
            }

            if byte_bucket[0] == b'\n' {
                break; // End of line
            }
        }
    } else {
        // ADD MODE: Write tag at column 0, then copy rest of line
        match comment_flag {
            CommentFlag::Hash => {
                if let Err(_) = writer.write_all(b"#") {
                    return Err(ToggleCommentError::IoError(IoOperation::Write));
                }
                if let Err(_) = writer.write_all(b" ") {
                    return Err(ToggleCommentError::IoError(IoOperation::Write));
                }
            }
            CommentFlag::DoubleSlash => {
                if let Err(_) = writer.write_all(b"//") {
                    return Err(ToggleCommentError::IoError(IoOperation::Write));
                }
                if let Err(_) = writer.write_all(b" ") {
                    return Err(ToggleCommentError::IoError(IoOperation::Write));
                }
            }
            CommentFlag::TripppleSlash => {
                if let Err(_) = writer.write_all(b"///") {
                    return Err(ToggleCommentError::IoError(IoOperation::Write));
                }
                if let Err(_) = writer.write_all(b" ") {
                    return Err(ToggleCommentError::IoError(IoOperation::Write));
                }
            }
        }

        // Copy rest of line
        loop {
            let bytes_read = match source_file.read(&mut byte_bucket) {
                Ok(n) => n,
                Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Read)),
            };

            if bytes_read == 0 {
                break; // EOF
            }

            if let Err(_) = writer.write_all(&byte_bucket) {
                return Err(ToggleCommentError::IoError(IoOperation::Write));
            }

            if byte_bucket[0] == b'\n' {
                break; // End of line
            }
        }
    }

    // ===========================================
    // PART C: Copy rest of file
    // ===========================================

    let mut remaining_bytes: u64 = 0;
    loop {
        if remaining_bytes >= MAX_BYTE_ITERATIONS {
            return Err(ToggleCommentError::IoError(IoOperation::Read));
        }

        let bytes_read = match source_file.read(&mut byte_bucket) {
            Ok(n) => n,
            Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Read)),
        };

        if bytes_read == 0 {
            break;
        }

        if let Err(_) = writer.write_all(&byte_bucket) {
            return Err(ToggleCommentError::IoError(IoOperation::Write));
        }

        remaining_bytes += 1;
    }

    if let Err(_) = writer.flush() {
        return Err(ToggleCommentError::IoError(IoOperation::Flush));
    }

    Ok(())
}

// ============================================================================
// REFACTORED WRAPPER - Uses Bytewise Operations
// ============================================================================

/// Toggle comment on a specific line using byte-wise operations (REFACTORED)
///
/// # Overview
/// **This function has been refactored to use byte-wise operations with no
/// line buffering or heap allocation.** The public API remains unchanged for
/// backward compatibility.
///
/// Previous implementation loaded entire lines into Vec<u8>. New implementation
/// uses single-byte buffer and separate find/detect/write operations.
///
/// # New Implementation
/// 1. Calls `find_and_detect_tag_state()` - finds line and detects tag in one pass
/// 2. Handles `Ok(None)` gracefully (line not found - not treated as error initially)
/// 3. Calls `write_toggled_file_bytewise()` - performs byte-wise toggle
/// 4. Preserves existing backup/temp file behavior
///
/// # Arguments
/// * `file_path` - Path to the source file
/// * `row_line_zeroindex` - Zero-indexed line number to toggle
///
/// # Returns
/// * `Ok(())` - Comment toggled successfully
/// * `Err(ToggleCommentError)` - Specific error code
///
/// # Example
/// ```no_run
/// use toggle_comment_indent_module::toggle_basic_singleline_comment;
///
/// match toggle_basic_singleline_comment("./src/main.rs", 5) {
///     Ok(()) => println!("Line 5 toggled"),
///     Err(e) => eprintln!("Failed: {:?}", e),
/// }
/// ```
///
/// # Memory Usage
/// - Previous: Up to MAX_LINE_LENGTH (10KB) per line
/// - New: 1 byte working buffer
/// - Reduction: ~10,000x less memory
///
/// # Performance
/// - Previous: Read entire line into Vec
/// - New: Single-byte operations, may be slower but more memory-safe
/// - Trade-off: Safety and bounded memory over raw speed
//
// NOTE: This is the refactored version using bytewise operations
pub fn toggle_basic_singleline_comment_bytewise(
    file_path: &str,
    row_line_zeroindex: usize,
) -> Result<(), ToggleCommentError> {
    // Convert to absolute path
    let absolute_path = match Path::new(file_path).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(ToggleCommentError::FileNotFound);
            }
            return Err(ToggleCommentError::PathError);
        }
    };

    // Extract and validate file extension
    let extension = match absolute_path.extension() {
        Some(ext) => ext.to_string_lossy().to_string(),
        None => return Err(ToggleCommentError::NoExtension),
    };

    // Determine comment flag from extension
    let comment_flag = match determine_comment_flag(&extension) {
        Some(flag) => flag,
        None => return Ok(()), // Unsupported extension - no-op
    };

    // ==================================================
    // NEW: Combined find and detect in single pass
    // ==================================================
    let (line_start_pos, has_tag) =
        match find_and_detect_tag_state(file_path, row_line_zeroindex, comment_flag)? {
            Some((pos, tag_state)) => (pos, tag_state),
            None => {
                // Line not found - return appropriate error
                return Err(ToggleCommentError::LineNotFound {
                    requested: row_line_zeroindex,
                    file_lines: 0, // Unknown in bytewise mode
                });
            }
        };

    // Get filename for backup naming
    let filename = match absolute_path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => return Err(ToggleCommentError::PathError),
    };

    // Create backup path in CWD
    let backup_filename = format!("backup_toggle_comment_{}", filename);
    let backup_path = PathBuf::from(&backup_filename);

    // Create backup copy of original file
    if let Err(_) = std::fs::copy(&absolute_path, &backup_path) {
        return Err(ToggleCommentError::IoError(IoOperation::Backup));
    }

    // Create working temp file in CWD
    let temp_filename = format!("temp_toggle_bytewise_{}_{}", std::process::id(), filename);
    let temp_path = PathBuf::from(&temp_filename);

    // ==================================================
    // NEW: Byte-wise write operation
    // ==================================================
    let process_result = write_toggled_file_bytewise(
        &absolute_path,
        &temp_path,
        row_line_zeroindex,
        line_start_pos,
        has_tag,
        comment_flag,
    );

    // Handle processing result (same as before)
    match process_result {
        Ok(()) => {
            // Success: replace original with temp file
            if let Err(_) = std::fs::copy(&temp_path, &absolute_path) {
                let _ = std::fs::remove_file(&temp_path);
                return Err(ToggleCommentError::IoError(IoOperation::Replace));
            }

            // Clean up temp file
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

// ============================================================================
// TESTS - Byte-Wise Operations
// ============================================================================

#[cfg(test)]
mod bytewise_tests {
    use super::*;

    // ========================================
    // find_and_detect_tag_state() Tests
    // ========================================

    #[test]
    fn test_find_and_detect_line_0() {
        let content = "code\nmore\n";
        let test_file = create_test_file("test_bytewise_find_0.rs", content);

        let result =
            find_and_detect_tag_state(test_file.to_str().unwrap(), 0, CommentFlag::DoubleSlash);

        assert!(result.is_ok());
        let found = result.unwrap();
        assert!(found.is_some());
        let (pos, has_tag) = found.unwrap();
        assert_eq!(pos, 0); // Line 0 starts at position 0
        assert_eq!(has_tag, false); // "code" has no "//" tag

        cleanup_files(&[&test_file]);
    }

    #[test]
    fn test_find_and_detect_line_1() {
        let content = "line 0\n// line 1\nline 2\n";
        let test_file = create_test_file("test_bytewise_find_1.rs", content);

        let result =
            find_and_detect_tag_state(test_file.to_str().unwrap(), 1, CommentFlag::DoubleSlash);

        assert!(result.is_ok());
        let found = result.unwrap();
        assert!(found.is_some());
        let (pos, has_tag) = found.unwrap();
        assert_eq!(pos, 7); // After "line 0\n"
        assert_eq!(has_tag, true); // Has "// " pattern

        cleanup_files(&[&test_file]);
    }

    #[test]
    fn test_find_and_detect_line_not_found() {
        let content = "line 0\nline 1\n";
        let test_file = create_test_file("test_bytewise_notfound.rs", content);

        let result =
            find_and_detect_tag_state(test_file.to_str().unwrap(), 10, CommentFlag::DoubleSlash);

        assert!(result.is_ok());
        let found = result.unwrap();
        assert!(found.is_none()); // Line 10 doesn't exist - Ok(None)

        cleanup_files(&[&test_file]);
    }

    #[test]
    fn test_find_and_detect_no_space_after_tag() {
        let content = "line 0\n//code\nline 2\n";
        let test_file = create_test_file("test_bytewise_nospace.rs", content);

        let result =
            find_and_detect_tag_state(test_file.to_str().unwrap(), 1, CommentFlag::DoubleSlash);

        assert!(result.is_ok());
        let found = result.unwrap();
        assert!(found.is_some());
        let (_pos, has_tag) = found.unwrap();
        assert_eq!(has_tag, false); // "//code" doesn't match "// " pattern

        cleanup_files(&[&test_file]);
    }

    #[test]
    fn test_find_and_detect_empty_line() {
        let content = "line 0\n\nline 2\n";
        let test_file = create_test_file("test_bytewise_empty.rs", content);

        let result =
            find_and_detect_tag_state(test_file.to_str().unwrap(), 1, CommentFlag::DoubleSlash);

        assert!(result.is_ok());
        let found = result.unwrap();
        assert!(found.is_some());
        let (_pos, has_tag) = found.unwrap();
        assert_eq!(has_tag, false); // Empty line has no tag

        cleanup_files(&[&test_file]);
    }

    #[test]
    fn test_find_and_detect_hash_comment() {
        let content = "line 0\n# code\nline 2\n";
        let test_file = create_test_file("test_bytewise_hash.py", content);

        let result = find_and_detect_tag_state(test_file.to_str().unwrap(), 1, CommentFlag::Hash);

        assert!(result.is_ok());
        let found = result.unwrap();
        assert!(found.is_some());
        let (_pos, has_tag) = found.unwrap();
        assert_eq!(has_tag, true); // "# " pattern found

        cleanup_files(&[&test_file]);
    }

    #[test]
    fn test_find_and_detect_triple_slash() {
        let content = "line 0\n/// docs\nline 2\n";
        let test_file = create_test_file("test_bytewise_triple.rs", content);

        let result =
            find_and_detect_tag_state(test_file.to_str().unwrap(), 1, CommentFlag::TripppleSlash);

        assert!(result.is_ok());
        let found = result.unwrap();
        assert!(found.is_some());
        let (_pos, has_tag) = found.unwrap();
        assert_eq!(has_tag, true); // "/// " pattern found

        cleanup_files(&[&test_file]);
    }

    // ========================================
    // write_toggled_file_bytewise() Tests
    // ========================================

    #[test]
    fn test_write_bytewise_add_tag() {
        let content = "line 0\ncode\nline 2\n";
        let test_file = create_test_file("test_bytewise_write_add.rs", content);

        let temp_file = PathBuf::from("temp_test_bytewise_add.rs");

        let result = write_toggled_file_bytewise(
            &test_file,
            &temp_file,
            1,
            7,     // Position after "line 0\n"
            false, // has_tag = false (ADD mode)
            CommentFlag::DoubleSlash,
        );

        assert!(result.is_ok());

        let new_content = read_file_content(&temp_file);
        assert_eq!(new_content, "line 0\n// code\nline 2\n");

        cleanup_files(&[&test_file, &temp_file]);
    }

    #[test]
    fn test_write_bytewise_remove_tag() {
        let content = "line 0\n// code\nline 2\n";
        let test_file = create_test_file("test_bytewise_write_remove.rs", content);

        let temp_file = PathBuf::from("temp_test_bytewise_remove.rs");

        let result = write_toggled_file_bytewise(
            &test_file,
            &temp_file,
            1,
            7,    // Position after "line 0\n"
            true, // has_tag = true (REMOVE mode)
            CommentFlag::DoubleSlash,
        );

        assert!(result.is_ok());

        let new_content = read_file_content(&temp_file);
        assert_eq!(new_content, "line 0\ncode\nline 2\n");

        cleanup_files(&[&test_file, &temp_file]);
    }

    #[test]
    fn test_write_bytewise_preserves_other_lines() {
        let content = "line 0\nline 1\nline 2\nline 3\n";
        let test_file = create_test_file("test_bytewise_preserve.rs", content);

        let temp_file = PathBuf::from("temp_test_bytewise_preserve.rs");

        let result = write_toggled_file_bytewise(
            &test_file,
            &temp_file,
            1,
            7,     // After "line 0\n"
            false, // ADD mode
            CommentFlag::DoubleSlash,
        );

        assert!(result.is_ok());

        let new_content = read_file_content(&temp_file);
        assert_eq!(new_content, "line 0\n// line 1\nline 2\nline 3\n");

        cleanup_files(&[&test_file, &temp_file]);
    }

    #[test]
    fn test_write_bytewise_empty_line_add() {
        let content = "line 0\n\nline 2\n";
        let test_file = create_test_file("test_bytewise_empty_add.rs", content);

        let temp_file = PathBuf::from("temp_test_bytewise_empty_add.rs");

        let result = write_toggled_file_bytewise(
            &test_file,
            &temp_file,
            1,
            7,     // After "line 0\n"
            false, // ADD mode
            CommentFlag::DoubleSlash,
        );

        assert!(result.is_ok());

        let new_content = read_file_content(&temp_file);
        assert_eq!(new_content, "line 0\n// \nline 2\n"); // Tag added to empty line

        cleanup_files(&[&test_file, &temp_file]);
    }

    #[test]
    fn test_write_bytewise_crlf_preservation() {
        let content = "line 0\r\ncode\r\nline 2\r\n";
        let test_file = create_test_file("test_bytewise_crlf.rs", content);

        let temp_file = PathBuf::from("temp_test_bytewise_crlf.rs");

        let result = write_toggled_file_bytewise(
            &test_file,
            &temp_file,
            1,
            8,     // After "line 0\r\n"
            false, // ADD mode
            CommentFlag::DoubleSlash,
        );

        assert!(result.is_ok());

        let new_content = read_file_content(&temp_file);
        // CRLF should be preserved
        assert_eq!(new_content, "line 0\r\n// code\r\nline 2\r\n");

        cleanup_files(&[&test_file, &temp_file]);
    }

    // ========================================
    // Integration Tests - Full Toggle
    // ========================================

    #[test]
    fn test_bytewise_toggle_full_add() {
        let content = "fn main() {}\n";
        let test_file = create_test_file("test_bytewise_full_add.rs", content);

        let result = toggle_basic_singleline_comment_bytewise(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "// fn main() {}\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_bytewise_full_add.rs"),
        ]);
    }

    #[test]
    fn test_bytewise_toggle_full_remove() {
        let content = "// fn main() {}\n";
        let test_file = create_test_file("test_bytewise_full_remove.rs", content);

        let result = toggle_basic_singleline_comment_bytewise(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "fn main() {}\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_bytewise_full_remove.rs"),
        ]);
    }

    #[test]
    fn test_bytewise_toggle_roundtrip() {
        let original = "code\n";
        let test_file = create_test_file("test_bytewise_roundtrip.rs", original);

        // Toggle on
        let result1 = toggle_basic_singleline_comment_bytewise(test_file.to_str().unwrap(), 0);
        assert!(result1.is_ok());

        let content1 = read_file_content(&test_file);
        assert_eq!(content1, "// code\n");

        // Toggle off
        let result2 = toggle_basic_singleline_comment_bytewise(test_file.to_str().unwrap(), 0);
        assert!(result2.is_ok());

        let content2 = read_file_content(&test_file);
        assert_eq!(content2, original);

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_bytewise_roundtrip.rs"),
        ]);
    }

    #[test]
    fn test_bytewise_toggle_middle_line() {
        let content = "line 0\nline 1\nline 2\n";
        let test_file = create_test_file("test_bytewise_middle.rs", content);

        let result = toggle_basic_singleline_comment_bytewise(test_file.to_str().unwrap(), 1);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "line 0\n// line 1\nline 2\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_bytewise_middle.rs"),
        ]);
    }

    #[test]
    fn test_bytewise_toggle_last_line() {
        let content = "line 0\nline 1\nline 2";
        let test_file = create_test_file("test_bytewise_last.rs", content);

        let result = toggle_basic_singleline_comment_bytewise(test_file.to_str().unwrap(), 2);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "line 0\nline 1\n// line 2");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_bytewise_last.rs"),
        ]);
    }

    #[test]
    fn test_bytewise_toggle_python() {
        let content = "print('hello')\n";
        let test_file = create_test_file("test_bytewise_python.py", content);

        let result = toggle_basic_singleline_comment_bytewise(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "# print('hello')\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_bytewise_python.py"),
        ]);
    }

    #[test]
    fn test_bytewise_line_not_found_error() {
        let content = "line 0\nline 1\n";
        let test_file = create_test_file("test_bytewise_notfound_err.rs", content);

        let result = toggle_basic_singleline_comment_bytewise(test_file.to_str().unwrap(), 10);
        assert!(matches!(
            result,
            Err(ToggleCommentError::LineNotFound { .. })
        ));

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_bytewise_notfound_err.rs"),
        ]);
    }
}

// ============================================================================
// TESTS
// ============================================================================
#[cfg(test)]
/// Helper: create a temporary test file with given content
fn create_test_file(filename: &str, content: &str) -> PathBuf {
    use std::io::Write;
    // Create tests directory if it doesn't exist
    let tests_dir = PathBuf::from("./tests");
    std::fs::create_dir_all(&tests_dir).expect("Failed to create tests directory");

    // Create file path within tests directory
    let path = tests_dir.join(filename);
    let mut file = File::create(&path).expect("Failed to create test file");
    file.write_all(content.as_bytes())
        .expect("Failed to write test file");
    path
}
#[cfg(test)]
/// Helper: read file content as string
fn read_file_content(path: &Path) -> String {
    std::fs::read_to_string(path).expect("Failed to read file")
}
#[cfg(test)]
/// Helper: cleanup test files
fn cleanup_files(paths: &[&Path]) {
    for path in paths {
        let _ = std::fs::remove_file(path);
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

// ============================================================================
// ERROR SECTION: ERROR HANDLING SYSTEM (start)
// ============================================================================

/// Errors that can occur during indent/unindent operations
///
/// All variants are Copy - no heap allocation, no string storage.
/// Reuses same design pattern as ToggleCommentError for consistency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleIndentError {
    /// The specified file was not found
    FileNotFound,

    /// The requested line index exceeds the file's line count
    LineNotFound { requested: usize, file_lines: usize },

    /// I/O operation failed
    IoError(IoOperation),

    /// Path conversion or manipulation error
    PathError,
}

impl std::fmt::Display for ToggleIndentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToggleIndentError::FileNotFound => write!(f, "File not found"),
            ToggleIndentError::LineNotFound {
                requested,
                file_lines,
            } => {
                write!(
                    f,
                    "Line {} not found (file has {} lines)",
                    requested, file_lines
                )
            }
            ToggleIndentError::IoError(op) => write!(f, "IO error: {:?}", op),
            ToggleIndentError::PathError => write!(f, "Path error"),
        }
    }
}

impl std::error::Error for ToggleIndentError {}

// ============================================================================
// ERROR SECTION: ERROR HANDLING SYSTEM (end)
// ============================================================================

// ============================================================================
// PHASE 2: INDENT/UNINDENT - BYTEWISE OPERATIONS
// ============================================================================

/// Helper: Find line start position (extracted for reuse)
///
/// # Overview
/// Reads file byte-by-byte counting newlines until target line is found.
/// Returns byte offset where target line begins.
///
/// # Arguments
/// * `file_path` - Path to source file
/// * `target_line` - Zero-indexed line number to find
///
/// # Returns
/// * `Ok(Some(position))` - Line found at byte offset
/// * `Ok(None)` - Line not found (file has fewer lines)
/// * `Err(ToggleIndentError)` - File operation failed
///
/// # Example
/// ```text
/// File:  "line 0\nline 1\nline 2\n"
///
/// find_line_start_position(path, 0) → Ok(Some(0))
/// find_line_start_position(path, 1) → Ok(Some(7))
/// find_line_start_position(path, 2) → Ok(Some(14))
/// find_line_start_position(path, 10) → Ok(None)
/// ```
pub fn find_line_start_position(
    file_path: &str,
    target_line: usize,
) -> Result<Option<u64>, ToggleIndentError> {
    let mut file = match File::open(file_path) {
        Ok(f) => f,
        Err(_) => return Err(ToggleIndentError::IoError(IoOperation::Open)),
    };

    // Special case: line 0 always starts at position 0
    if target_line == 0 {
        return Ok(Some(0));
    }

    let mut byte_bucket: [u8; 1] = [0u8; 1];
    let mut byte_position: u64 = 0;
    let mut current_line: usize = 0;

    loop {
        // Safety check
        if byte_position >= MAX_BYTE_ITERATIONS {
            return Err(ToggleIndentError::IoError(IoOperation::Read));
        }

        let bytes_read = match file.read(&mut byte_bucket) {
            Ok(n) => n,
            Err(_) => return Err(ToggleIndentError::IoError(IoOperation::Read)),
        };

        if bytes_read == 0 {
            return Ok(None); // Line not found
        }

        if byte_bucket[0] == b'\n' {
            current_line += 1;
            if current_line == target_line {
                return Ok(Some(byte_position + 1));
            }
        }

        byte_position += 1;
    }
}

/// Write file with 4 spaces added at start of target line
///
/// # Overview
/// Copies source to dest, inserting 4 spaces at column 0 of target line.
/// Simple three-part copy: before, modified line, after.
///
/// # Arguments
/// * `source_path` - Original file
/// * `dest_path` - Temp file for output
/// * `line_start_pos` - Byte offset where target line begins
///
/// # Returns
/// * `Ok(())` - File written successfully
/// * `Err(ToggleIndentError)` - Write operation failed
///
/// # Memory
/// - 1 byte buffer only
/// - No heap allocation
pub fn write_indented_file_bytewise(
    source_path: &Path,
    dest_path: &Path,
    line_start_pos: u64,
) -> Result<(), ToggleIndentError> {
    let mut source_file = match File::open(source_path) {
        Ok(f) => f,
        Err(_) => return Err(ToggleIndentError::IoError(IoOperation::Open)),
    };

    let dest_file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dest_path)
    {
        Ok(f) => f,
        Err(_) => return Err(ToggleIndentError::IoError(IoOperation::Create)),
    };

    let mut writer = BufWriter::with_capacity(IO_BUFFER_SIZE, dest_file);
    let mut byte_bucket: [u8; 1] = [0u8; 1];
    let mut byte_position: u64 = 0;

    // PART A: Copy before target line
    while byte_position < line_start_pos {
        if byte_position >= MAX_BYTE_ITERATIONS {
            return Err(ToggleIndentError::IoError(IoOperation::Read));
        }

        let bytes_read = match source_file.read(&mut byte_bucket) {
            Ok(n) => n,
            Err(_) => return Err(ToggleIndentError::IoError(IoOperation::Read)),
        };

        if bytes_read == 0 {
            return Err(ToggleIndentError::IoError(IoOperation::Read));
        }

        if let Err(_) = writer.write_all(&byte_bucket) {
            return Err(ToggleIndentError::IoError(IoOperation::Write));
        }

        byte_position += 1;
    }

    // PART B: Add 4 spaces at column 0, then copy rest of line
    if let Err(_) = writer.write_all(b"    ") {
        return Err(ToggleIndentError::IoError(IoOperation::Write));
    }

    // Copy rest of line
    loop {
        let bytes_read = match source_file.read(&mut byte_bucket) {
            Ok(n) => n,
            Err(_) => return Err(ToggleIndentError::IoError(IoOperation::Read)),
        };

        if bytes_read == 0 {
            break; // EOF
        }

        if let Err(_) = writer.write_all(&byte_bucket) {
            return Err(ToggleIndentError::IoError(IoOperation::Write));
        }

        if byte_bucket[0] == b'\n' {
            break; // End of line
        }
    }

    // PART C: Copy rest of file
    let mut remaining_bytes: u64 = 0;
    loop {
        if remaining_bytes >= MAX_BYTE_ITERATIONS {
            return Err(ToggleIndentError::IoError(IoOperation::Read));
        }

        let bytes_read = match source_file.read(&mut byte_bucket) {
            Ok(n) => n,
            Err(_) => return Err(ToggleIndentError::IoError(IoOperation::Read)),
        };

        if bytes_read == 0 {
            break;
        }

        if let Err(_) = writer.write_all(&byte_bucket) {
            return Err(ToggleIndentError::IoError(IoOperation::Write));
        }

        remaining_bytes += 1;
    }

    if let Err(_) = writer.flush() {
        return Err(ToggleIndentError::IoError(IoOperation::Flush));
    }

    Ok(())
}

/// Write file with up to 4 spaces removed from start of target line
///
/// # Overview
/// Copies source to dest, removing up to 4 spaces from column 0 of target line.
/// If line has fewer than 4 spaces, removes only what's there.
///
/// # Arguments
/// * `source_path` - Original file
/// * `dest_path` - Temp file for output
/// * `line_start_pos` - Byte offset where target line begins
///
/// # Returns
/// * `Ok(())` - File written successfully
/// * `Err(ToggleIndentError)` - Write operation failed
///
/// # Memory
/// - 1 byte buffer only
/// - No heap allocation
pub fn write_unindented_file_bytewise(
    source_path: &Path,
    dest_path: &Path,
    line_start_pos: u64,
) -> Result<(), ToggleIndentError> {
    let mut source_file = match File::open(source_path) {
        Ok(f) => f,
        Err(_) => return Err(ToggleIndentError::IoError(IoOperation::Open)),
    };

    let dest_file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dest_path)
    {
        Ok(f) => f,
        Err(_) => return Err(ToggleIndentError::IoError(IoOperation::Create)),
    };

    let mut writer = BufWriter::with_capacity(IO_BUFFER_SIZE, dest_file);
    let mut byte_bucket: [u8; 1] = [0u8; 1];
    let mut byte_position: u64 = 0;

    // PART A: Copy before target line
    while byte_position < line_start_pos {
        if byte_position >= MAX_BYTE_ITERATIONS {
            return Err(ToggleIndentError::IoError(IoOperation::Read));
        }

        let bytes_read = match source_file.read(&mut byte_bucket) {
            Ok(n) => n,
            Err(_) => return Err(ToggleIndentError::IoError(IoOperation::Read)),
        };

        if bytes_read == 0 {
            return Err(ToggleIndentError::IoError(IoOperation::Read));
        }

        if let Err(_) = writer.write_all(&byte_bucket) {
            return Err(ToggleIndentError::IoError(IoOperation::Write));
        }

        byte_position += 1;
    }

    // PART B: Skip up to 4 spaces at column 0, then copy rest of line
    let mut spaces_skipped: usize = 0;
    loop {
        let bytes_read = match source_file.read(&mut byte_bucket) {
            Ok(n) => n,
            Err(_) => return Err(ToggleIndentError::IoError(IoOperation::Read)),
        };

        if bytes_read == 0 {
            break; // EOF
        }

        // If it's a space and we haven't skipped 4 yet, skip it
        if byte_bucket[0] == b' ' && spaces_skipped < INDENT_SPACES {
            spaces_skipped += 1;
            continue; // Skip this space, don't write it
        }

        // Not a space, or already skipped 4 - write rest of line
        if let Err(_) = writer.write_all(&byte_bucket) {
            return Err(ToggleIndentError::IoError(IoOperation::Write));
        }

        if byte_bucket[0] == b'\n' {
            break; // End of line
        }

        // Copy rest of line
        loop {
            let bytes_read = match source_file.read(&mut byte_bucket) {
                Ok(n) => n,
                Err(_) => return Err(ToggleIndentError::IoError(IoOperation::Read)),
            };

            if bytes_read == 0 {
                break; // EOF
            }

            if let Err(_) = writer.write_all(&byte_bucket) {
                return Err(ToggleIndentError::IoError(IoOperation::Write));
            }

            if byte_bucket[0] == b'\n' {
                break; // End of line
            }
        }

        break;
    }

    // PART C: Copy rest of file
    let mut remaining_bytes: u64 = 0;
    loop {
        if remaining_bytes >= MAX_BYTE_ITERATIONS {
            return Err(ToggleIndentError::IoError(IoOperation::Read));
        }

        let bytes_read = match source_file.read(&mut byte_bucket) {
            Ok(n) => n,
            Err(_) => return Err(ToggleIndentError::IoError(IoOperation::Read)),
        };

        if bytes_read == 0 {
            break;
        }

        if let Err(_) = writer.write_all(&byte_bucket) {
            return Err(ToggleIndentError::IoError(IoOperation::Write));
        }

        remaining_bytes += 1;
    }

    if let Err(_) = writer.flush() {
        return Err(ToggleIndentError::IoError(IoOperation::Flush));
    }

    Ok(())
}

/// Add 4 spaces to the start of a specific line (bytewise)
///
/// # Overview
/// Bytewise implementation - adds exactly 4 spaces at column 0 of target line.
/// No heap allocation, single byte buffer.
///
/// # Arguments
/// * `file_path` - Path to the source file
/// * `line_number` - Zero-indexed line number to indent
///
/// # Returns
/// * `Ok(())` - Line indented successfully
/// * `Err(ToggleIndentError)` - Specific error code
///
/// # Example
/// ```no_run
/// use toggle_comment_indent_module::indent_line_bytewise;
///
/// match indent_line_bytewise("./src/main.rs", 5) {
///     Ok(()) => println!("Line 5 indented"),
///     Err(e) => eprintln!("Failed: {:?}", e),
/// }
/// ```
///
/// # Behavior
/// ```text
/// Before: "code"
/// After:  "    code"
///
/// Before: "  code"  (already indented 2)
/// After:  "      code"  (now indented 6)
/// ```
pub fn indent_line_bytewise(file_path: &str, line_number: usize) -> Result<(), ToggleIndentError> {
    // Convert to absolute path
    let absolute_path = match Path::new(file_path).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(ToggleIndentError::FileNotFound);
            }
            return Err(ToggleIndentError::PathError);
        }
    };

    // Find line start position
    let line_start_pos = match find_line_start_position(file_path, line_number)? {
        Some(pos) => pos,
        None => {
            return Err(ToggleIndentError::LineNotFound {
                requested: line_number,
                file_lines: 0,
            });
        }
    };

    // Get filename for backup
    let filename = match absolute_path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => return Err(ToggleIndentError::PathError),
    };

    // Create backup
    let backup_filename = format!("backup_toggle_comment_{}", filename);
    let backup_path = PathBuf::from(&backup_filename);

    if let Err(_) = std::fs::copy(&absolute_path, &backup_path) {
        return Err(ToggleIndentError::IoError(IoOperation::Backup));
    }

    // Create temp file
    let temp_filename = format!("temp_indent_bytewise_{}_{}", std::process::id(), filename);
    let temp_path = PathBuf::from(&temp_filename);

    // Write indented file
    let process_result = write_indented_file_bytewise(&absolute_path, &temp_path, line_start_pos);

    // Handle result
    match process_result {
        Ok(()) => {
            if let Err(_) = std::fs::copy(&temp_path, &absolute_path) {
                let _ = std::fs::remove_file(&temp_path);
                return Err(ToggleIndentError::IoError(IoOperation::Replace));
            }

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

/// Remove up to 4 spaces from the start of a specific line (bytewise)
///
/// # Overview
/// Bytewise implementation - removes up to 4 spaces from column 0.
/// If line has fewer than 4 spaces, removes only what's there.
/// No heap allocation, single byte buffer.
///
/// # Arguments
/// * `file_path` - Path to the source file
/// * `line_number` - Zero-indexed line number to unindent
///
/// # Returns
/// * `Ok(())` - Line unindented successfully (even if no spaces removed)
/// * `Err(ToggleIndentError)` - Specific error code
///
/// # Example
/// ```no_run
/// use toggle_comment_indent_module::unindent_line_bytewise;
///
/// match unindent_line_bytewise("./src/main.rs", 5) {
///     Ok(()) => println!("Line 5 unindented"),
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
/// After:  "code"      (removed 0 - no-op)
/// ```
pub fn unindent_line_bytewise(
    file_path: &str,
    line_number: usize,
) -> Result<(), ToggleIndentError> {
    // Convert to absolute path
    let absolute_path = match Path::new(file_path).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(ToggleIndentError::FileNotFound);
            }
            return Err(ToggleIndentError::PathError);
        }
    };

    // Find line start position
    let line_start_pos = match find_line_start_position(file_path, line_number)? {
        Some(pos) => pos,
        None => {
            return Err(ToggleIndentError::LineNotFound {
                requested: line_number,
                file_lines: 0,
            });
        }
    };

    // Get filename for backup
    let filename = match absolute_path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => return Err(ToggleIndentError::PathError),
    };

    // Create backup
    let backup_filename = format!("backup_toggle_comment_{}", filename);
    let backup_path = PathBuf::from(&backup_filename);

    if let Err(_) = std::fs::copy(&absolute_path, &backup_path) {
        return Err(ToggleIndentError::IoError(IoOperation::Backup));
    }

    // Create temp file
    let temp_filename = format!("temp_unindent_bytewise_{}_{}", std::process::id(), filename);
    let temp_path = PathBuf::from(&temp_filename);

    // Write unindented file
    let process_result = write_unindented_file_bytewise(&absolute_path, &temp_path, line_start_pos);

    // Handle result
    match process_result {
        Ok(()) => {
            if let Err(_) = std::fs::copy(&temp_path, &absolute_path) {
                let _ = std::fs::remove_file(&temp_path);
                return Err(ToggleIndentError::IoError(IoOperation::Replace));
            }

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

/// Add 4 spaces to multiple lines using simple loop (bytewise)
///
/// # Overview
/// **Simple implementation:** Calls `indent_line_bytewise()` once for each
/// line in the range. File opened/closed multiple times - intentional for simplicity.
///
/// # Arguments
/// * `file_path` - Path to the source file
/// * `start_line` - First line to indent (will be sorted with end_line)
/// * `end_line` - Last line to indent (will be sorted with start_line)
///
/// # Returns
/// * `Ok(())` - All lines indented successfully
/// * `Err(ToggleIndentError)` - Specific error code
///
/// # Example
/// ```no_run
/// use toggle_comment_indent_module::indent_range_bytewise;
///
/// // Indent lines 5-10 (order doesn't matter)
/// match indent_range_bytewise("./src/main.rs", 5, 10) {
///     Ok(()) => println!("Range indented"),
///     Err(e) => eprintln!("Failed: {:?}", e),
/// }
/// ```
pub fn indent_range_bytewise(
    file_path: &str,
    start_line: usize,
    end_line: usize,
) -> Result<(), ToggleIndentError> {
    let (start, end) = sort_range(start_line, end_line);

    // Safety check
    let range_size = end.saturating_sub(start).saturating_add(1);
    if range_size > 10000 {
        #[cfg(debug_assertions)]
        eprintln!(
            "Warning: Large range ({} lines) - this will be slow",
            range_size
        );
    }

    // Simple loop: indent each line independently
    for line_num in start..=end {
        indent_line_bytewise(file_path, line_num)?;
    }

    Ok(())
}

/// Remove up to 4 spaces from multiple lines using simple loop (bytewise)
///
/// # Overview
/// **Simple implementation:** Calls `unindent_line_bytewise()` once for each
/// line in the range. File opened/closed multiple times - intentional for simplicity.
///
/// # Arguments
/// * `file_path` - Path to the source file
/// * `start_line` - First line to unindent (will be sorted with end_line)
/// * `end_line` - Last line to unindent (will be sorted with start_line)
///
/// # Returns
/// * `Ok(())` - All lines unindented successfully
/// * `Err(ToggleIndentError)` - Specific error code
///
/// # Example
/// ```no_run
/// use toggle_comment_indent_module::unindent_range_bytewise;
///
/// // Unindent lines 5-10 (order doesn't matter)
/// match unindent_range_bytewise("./src/main.rs", 5, 10) {
///     Ok(()) => println!("Range unindented"),
///     Err(e) => eprintln!("Failed: {:?}", e),
/// }
/// ```
pub fn unindent_range_bytewise(
    file_path: &str,
    start_line: usize,
    end_line: usize,
) -> Result<(), ToggleIndentError> {
    let (start, end) = sort_range(start_line, end_line);

    // Safety check
    let range_size = end.saturating_sub(start).saturating_add(1);
    if range_size > 10000 {
        #[cfg(debug_assertions)]
        eprintln!(
            "Warning: Large range ({} lines) - this will be slow",
            range_size
        );
    }

    // Simple loop: unindent each line independently
    for line_num in start..=end {
        unindent_line_bytewise(file_path, line_num)?;
    }

    Ok(())
}

// ============================================================================
// TESTS - PHASE 2: INDENT/UNINDENT BYTEWISE
// ============================================================================

#[cfg(test)]
mod indent_bytewise_tests {
    use super::*;

    // ========================================
    // Single Line Indent Tests
    // ========================================

    #[test]
    fn test_indent_bytewise_basic() {
        let content = "code\n";
        let test_file = create_test_file("test_indent_bw_basic.txt", content);

        let result = indent_line_bytewise(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "    code\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_indent_bw_basic.txt"),
        ]);
    }

    #[test]
    fn test_indent_bytewise_already_indented() {
        let content = "  code\n";
        let test_file = create_test_file("test_indent_bw_existing.txt", content);

        let result = indent_line_bytewise(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "      code\n"); // 2 + 4 = 6 spaces

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_indent_bw_existing.txt"),
        ]);
    }

    #[test]
    fn test_indent_bytewise_empty_line() {
        let content = "\n";
        let test_file = create_test_file("test_indent_bw_empty.txt", content);

        let result = indent_line_bytewise(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "    \n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_indent_bw_empty.txt"),
        ]);
    }

    // ========================================
    // Single Line Unindent Tests
    // ========================================

    #[test]
    fn test_unindent_bytewise_four_spaces() {
        let content = "    code\n";
        let test_file = create_test_file("test_unindent_bw_four.txt", content);

        let result = unindent_line_bytewise(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "code\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_unindent_bw_four.txt"),
        ]);
    }

    #[test]
    fn test_unindent_bytewise_two_spaces() {
        let content = "  code\n";
        let test_file = create_test_file("test_unindent_bw_two.txt", content);

        let result = unindent_line_bytewise(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "code\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_unindent_bw_two.txt"),
        ]);
    }

    #[test]
    fn test_unindent_bytewise_no_spaces() {
        let content = "code\n";
        let test_file = create_test_file("test_unindent_bw_none.txt", content);

        let result = unindent_line_bytewise(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "code\n"); // Unchanged

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_unindent_bw_none.txt"),
        ]);
    }

    #[test]
    fn test_unindent_bytewise_six_spaces() {
        let content = "      code\n";
        let test_file = create_test_file("test_unindent_bw_six.txt", content);

        let result = unindent_line_bytewise(test_file.to_str().unwrap(), 0);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "  code\n"); // Removed 4, left 2

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_unindent_bw_six.txt"),
        ]);
    }

    // ========================================
    // Range Tests
    // ========================================

    #[test]
    fn test_indent_range_bytewise_basic() {
        let content = "line 0\nline 1\nline 2\n";
        let test_file = create_test_file("test_indent_range_bw.txt", content);

        let result = indent_range_bytewise(test_file.to_str().unwrap(), 0, 2);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "    line 0\n    line 1\n    line 2\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_indent_range_bw.txt"),
        ]);
    }

    #[test]
    fn test_unindent_range_bytewise_basic() {
        let content = "    line 0\n    line 1\n    line 2\n";
        let test_file = create_test_file("test_unindent_range_bw.txt", content);

        let result = unindent_range_bytewise(test_file.to_str().unwrap(), 0, 2);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "line 0\nline 1\nline 2\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_unindent_range_bw.txt"),
        ]);
    }

    #[test]
    fn test_indent_unindent_roundtrip() {
        let original = "code\n";
        let test_file = create_test_file("test_indent_roundtrip_bw.txt", original);

        // Indent
        let result1 = indent_line_bytewise(test_file.to_str().unwrap(), 0);
        assert!(result1.is_ok());

        let content1 = read_file_content(&test_file);
        assert_eq!(content1, "    code\n");

        // Unindent back
        let result2 = unindent_line_bytewise(test_file.to_str().unwrap(), 0);
        assert!(result2.is_ok());

        let content2 = read_file_content(&test_file);
        assert_eq!(content2, original);

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_indent_roundtrip_bw.txt"),
        ]);
    }
}

// ============================================================================
// RANGE UTILITIES
// ============================================================================

/// Sort and validate a line range to ensure start ≤ end
///
/// # Overview
/// Universal helper for all range-based operations. Always returns a valid
/// sorted tuple where the first value is ≤ the second value.
///
/// # Arguments
/// * `from` - First line number (may be larger than `to`)
/// * `to` - Second line number (may be smaller than `from`)
///
/// # Returns
/// * `(start, end)` - Sorted tuple where start ≤ end
///
/// # Behavior
/// ```text
/// Input: (5, 10) → Output: (5, 10)  [already sorted]
/// Input: (10, 5) → Output: (5, 10)  [flipped]
/// Input: (7, 7)  → Output: (7, 7)   [single line, valid]
/// ```
///
/// # Usage
/// ```rust
/// let (start, end) = sort_range(user_input_from, user_input_to);
/// // Now guaranteed: start <= end
/// ```
///
/// # Safety
/// - Always succeeds (no Result needed)
/// - No allocations
/// - Copy semantics only
/// - Works for any usize values (no overflow - uses min/max)
fn sort_range(from: usize, to: usize) -> (usize, usize) {
    // Use built-in min/max for clarity and safety
    let start = std::cmp::min(from, to);
    let end = std::cmp::max(from, to);

    (start, end)
}

// ============================================================================
// INDENT/UNINDENT RANGE FUNCTIONS
// ============================================================================

// ============================================================================
// PHASE 1: RANGE TOGGLE - SIMPLE LOOP APPROACH (BYTEWISE)
// ============================================================================

/// Toggle Rust documentation comment (///) on a specific line - bytewise version
///
/// # Overview
/// Bytewise implementation of docstring toggle. Identical to
/// `toggle_basic_singleline_comment_bytewise()` but uses TripleSlash flag.
///
/// # Arguments
/// * `file_path` - Path to the source file
/// * `row_line_zeroindex` - Zero-indexed line number to toggle
///
/// # Returns
/// * `Ok(())` - Comment toggled successfully
/// * `Err(ToggleCommentError)` - Specific error code
///
/// # Memory Usage
/// - Single byte buffer [u8; 1]
/// - No heap allocation during processing
///
/// # Example
/// ```no_run
/// use toggle_comment_indent_module::toggle_rust_docstring_singleline_comment_bytewise;
///
/// match toggle_rust_docstring_singleline_comment_bytewise("./src/lib.rs", 5) {
///     Ok(()) => println!("Docstring toggled"),
///     Err(e) => eprintln!("Failed: {:?}", e),
/// }
/// ```
pub fn toggle_rust_docstring_singleline_comment_bytewise(
    file_path: &str,
    row_line_zeroindex: usize,
) -> Result<(), ToggleCommentError> {
    // Convert to absolute path
    let absolute_path = match Path::new(file_path).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(ToggleCommentError::FileNotFound);
            }
            return Err(ToggleCommentError::PathError);
        }
    };

    // Use TripleSlash flag (no extension check needed)
    let comment_flag = CommentFlag::TripppleSlash;

    // Combined find and detect in single pass
    let (line_start_pos, has_tag) =
        match find_and_detect_tag_state(file_path, row_line_zeroindex, comment_flag)? {
            Some((pos, tag_state)) => (pos, tag_state),
            None => {
                return Err(ToggleCommentError::LineNotFound {
                    requested: row_line_zeroindex,
                    file_lines: 0,
                });
            }
        };

    // Get filename for backup naming
    let filename = match absolute_path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => return Err(ToggleCommentError::PathError),
    };

    // Create backup path in CWD
    let backup_filename = format!("backup_toggle_comment_{}", filename);
    let backup_path = PathBuf::from(&backup_filename);

    // Create backup copy
    if let Err(_) = std::fs::copy(&absolute_path, &backup_path) {
        return Err(ToggleCommentError::IoError(IoOperation::Backup));
    }

    // Create temp file
    let temp_filename = format!(
        "temp_toggle_docstring_bytewise_{}_{}",
        std::process::id(),
        filename
    );
    let temp_path = PathBuf::from(&temp_filename);

    // Byte-wise write operation
    let process_result = write_toggled_file_bytewise(
        &absolute_path,
        &temp_path,
        row_line_zeroindex,
        line_start_pos,
        has_tag,
        comment_flag,
    );

    // Handle result
    match process_result {
        Ok(()) => {
            // Success: replace original
            if let Err(_) = std::fs::copy(&temp_path, &absolute_path) {
                let _ = std::fs::remove_file(&temp_path);
                return Err(ToggleCommentError::IoError(IoOperation::Replace));
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

/// Toggle basic comments on a range of lines using simple loop (bytewise)
///
/// # Overview
/// **Simple implementation:** Calls `toggle_basic_singleline_comment_bytewise()`
/// once for each line in the range. No optimization, no single-pass complexity.
/// File is opened/closed multiple times - this is intentional for simplicity.
///
/// Each line is toggled independently based on its current state.
/// Range is automatically sorted - argument order doesn't matter.
///
/// # Arguments
/// * `file_path` - Path to the source file
/// * `start_line` - First line to toggle (will be sorted with end_line)
/// * `end_line` - Last line to toggle (will be sorted with start_line)
///
/// # Returns
/// * `Ok(())` - All lines in range toggled successfully
/// * `Err(ToggleCommentError)` - Specific error code
///
/// # Algorithm
/// ```text
/// for each line in start..=end:
///     toggle_basic_singleline_comment_bytewise(file, line)
/// ```
///
/// # Memory Usage
/// - Per line: 1 byte buffer
/// - Total: 1 byte (reused for each line)
/// - No accumulation, no line counting needed
///
/// # Example
/// ```no_run
/// use toggle_comment_indent_module::toggle_range_basic_comments_bytewise;
///
/// // Toggle lines 5-10 (both inclusive, order doesn't matter)
/// match toggle_range_basic_comments_bytewise("./src/main.rs", 5, 10) {
///     Ok(()) => println!("Range toggled"),
///     Err(e) => eprintln!("Failed: {:?}", e),
/// }
///
/// // Same result (auto-sorted):
/// toggle_range_basic_comments_bytewise("./src/main.rs", 10, 5)?;
/// ```
///
/// # Behavior
/// Each line toggled independently:
/// ```text
/// Input range [5-7]:
/// line 5          →  // line 5
/// // line 6       →  line 6
/// line 7          →  // line 7
/// ```
///
/// # Performance Note
/// This is NOT optimized for performance - it's optimized for:
/// - Simplicity (easy to understand and maintain)
/// - Safety (reuses proven single-line bytewise code)
/// - Memory bounds (constant memory regardless of range size)
///
/// For a 100-line range, this opens the file 100 times. That's OK.
/// Simple is better than clever.
pub fn toggle_range_basic_comments_bytewise(
    file_path: &str,
    start_line: usize,
    end_line: usize,
) -> Result<(), ToggleCommentError> {
    // Sort range automatically
    let (start, end) = sort_range(start_line, end_line);

    // Safety check: reasonable range size
    // This is just a sanity check, not a hard limit
    let range_size = end.saturating_sub(start).saturating_add(1);
    if range_size > 10000 {
        #[cfg(debug_assertions)]
        eprintln!(
            "Warning: Large range ({} lines) - this will be slow",
            range_size
        );
    }

    // Simple loop: toggle each line independently
    for line_num in start..=end {
        // If any line fails, propagate the error immediately
        toggle_basic_singleline_comment_bytewise(file_path, line_num)?;
    }

    Ok(())
}

/// Toggle Rust docstrings on a range of lines using simple loop (bytewise)
///
/// # Overview
/// **Simple implementation:** Calls `toggle_rust_docstring_singleline_comment_bytewise()`
/// once for each line in the range. No optimization, no single-pass complexity.
/// File is opened/closed multiple times - this is intentional for simplicity.
///
/// Each line is toggled independently based on its current state.
/// Range is automatically sorted - argument order doesn't matter.
///
/// # Arguments
/// * `file_path` - Path to the source file
/// * `start_line` - First line to toggle (will be sorted with end_line)
/// * `end_line` - Last line to toggle (will be sorted with start_line)
///
/// # Returns
/// * `Ok(())` - All lines in range toggled successfully
/// * `Err(ToggleCommentError)` - Specific error code
///
/// # Algorithm
/// ```text
/// for each line in start..=end:
///     toggle_rust_docstring_singleline_comment_bytewise(file, line)
/// ```
///
/// # Memory Usage
/// - Per line: 1 byte buffer
/// - Total: 1 byte (reused for each line)
///
/// # Example
/// ```no_run
/// use toggle_comment_indent_module::toggle_range_rust_docstring_bytewise;
///
/// // Toggle docstrings on lines 5-10
/// match toggle_range_rust_docstring_bytewise("./src/lib.rs", 5, 10) {
///     Ok(()) => println!("Docstrings toggled"),
///     Err(e) => eprintln!("Failed: {:?}", e),
/// }
/// ```
///
/// # Note
/// No file extension validation - works on any file type.
/// Caller responsible for using on appropriate files.
pub fn toggle_range_rust_docstring_bytewise(
    file_path: &str,
    start_line: usize,
    end_line: usize,
) -> Result<(), ToggleCommentError> {
    // Sort range automatically
    let (start, end) = sort_range(start_line, end_line);

    // Safety check: reasonable range size
    let range_size = end.saturating_sub(start).saturating_add(1);
    if range_size > 10000 {
        #[cfg(debug_assertions)]
        eprintln!(
            "Warning: Large range ({} lines) - this will be slow",
            range_size
        );
    }

    // Simple loop: toggle each line independently
    for line_num in start..=end {
        toggle_rust_docstring_singleline_comment_bytewise(file_path, line_num)?;
    }

    Ok(())
}

// ============================================================================
// TESTS - PHASE 1: RANGE TOGGLE BYTEWISE
// ============================================================================

#[cfg(test)]
mod range_toggle_bytewise_tests {
    use super::*;

    // ========================================
    // Basic Range Toggle Tests
    // ========================================

    #[test]
    fn test_range_bytewise_basic_all_off() {
        let content = "line 0\nline 1\nline 2\nline 3\n";
        let test_file = create_test_file("test_range_bw_all_off.rs", content);

        let result = toggle_range_basic_comments_bytewise(test_file.to_str().unwrap(), 1, 2);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "line 0\n// line 1\n// line 2\nline 3\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_range_bw_all_off.rs"),
        ]);
    }

    #[test]
    fn test_range_bytewise_basic_all_on() {
        let content = "line 0\n// line 1\n// line 2\nline 3\n";
        let test_file = create_test_file("test_range_bw_all_on.rs", content);

        let result = toggle_range_basic_comments_bytewise(test_file.to_str().unwrap(), 1, 2);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "line 0\nline 1\nline 2\nline 3\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_range_bw_all_on.rs"),
        ]);
    }

    #[test]
    fn test_range_bytewise_basic_mixed() {
        let content = "line 0\n// line 1\nline 2\n// line 3\nline 4\n";
        let test_file = create_test_file("test_range_bw_mixed.rs", content);

        let result = toggle_range_basic_comments_bytewise(test_file.to_str().unwrap(), 1, 3);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        // Each line toggled independently
        assert_eq!(new_content, "line 0\nline 1\n// line 2\nline 3\nline 4\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_range_bw_mixed.rs"),
        ]);
    }

    #[test]
    fn test_range_bytewise_reversed_input() {
        let content = "line 0\nline 1\nline 2\nline 3\n";
        let test_file = create_test_file("test_range_bw_reversed.rs", content);

        // Reversed input: 2, 1 (should auto-sort to 1, 2)
        let result = toggle_range_basic_comments_bytewise(test_file.to_str().unwrap(), 2, 1);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "line 0\n// line 1\n// line 2\nline 3\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_range_bw_reversed.rs"),
        ]);
    }

    #[test]
    fn test_range_bytewise_single_line() {
        let content = "line 0\nline 1\nline 2\n";
        let test_file = create_test_file("test_range_bw_single.rs", content);

        // Range of one line (1, 1)
        let result = toggle_range_basic_comments_bytewise(test_file.to_str().unwrap(), 1, 1);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "line 0\n// line 1\nline 2\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_range_bw_single.rs"),
        ]);
    }

    #[test]
    fn test_range_bytewise_preserves_outside() {
        let content = "line 0\nline 1\nline 2\nline 3\nline 4\n";
        let test_file = create_test_file("test_range_bw_preserve.rs", content);

        // Toggle only middle lines
        let result = toggle_range_basic_comments_bytewise(test_file.to_str().unwrap(), 1, 3);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(
            new_content,
            "line 0\n// line 1\n// line 2\n// line 3\nline 4\n"
        );

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_range_bw_preserve.rs"),
        ]);
    }

    #[test]
    fn test_range_bytewise_roundtrip() {
        let original = "line 0\nline 1\nline 2\n";
        let test_file = create_test_file("test_range_bw_roundtrip.rs", original);

        // Toggle on
        let result1 = toggle_range_basic_comments_bytewise(test_file.to_str().unwrap(), 0, 2);
        assert!(result1.is_ok());

        let content1 = read_file_content(&test_file);
        assert_eq!(content1, "// line 0\n// line 1\n// line 2\n");

        // Toggle off
        let result2 = toggle_range_basic_comments_bytewise(test_file.to_str().unwrap(), 0, 2);
        assert!(result2.is_ok());

        let content2 = read_file_content(&test_file);
        assert_eq!(content2, original);

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_range_bw_roundtrip.rs"),
        ]);
    }

    #[test]
    fn test_range_bytewise_python() {
        let content = "line 0\nline 1\nline 2\n";
        let test_file = create_test_file("test_range_bw_python.py", content);

        let result = toggle_range_basic_comments_bytewise(test_file.to_str().unwrap(), 0, 1);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "# line 0\n# line 1\nline 2\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_range_bw_python.py"),
        ]);
    }

    #[test]
    fn test_range_bytewise_line_not_found() {
        let content = "line 0\nline 1\n";
        let test_file = create_test_file("test_range_bw_notfound.rs", content);

        // Range extends beyond file
        let result = toggle_range_basic_comments_bytewise(test_file.to_str().unwrap(), 0, 10);
        assert!(matches!(
            result,
            Err(ToggleCommentError::LineNotFound { .. })
        ));

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_range_bw_notfound.rs"),
        ]);
    }

    // ========================================
    // Docstring Range Toggle Tests
    // ========================================

    #[test]
    fn test_range_bytewise_docstring_basic() {
        let content = "line 0\nline 1\nline 2\nline 3\n";
        let test_file = create_test_file("test_range_bw_doc.rs", content);

        let result = toggle_range_rust_docstring_bytewise(test_file.to_str().unwrap(), 1, 2);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "line 0\n/// line 1\n/// line 2\nline 3\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_range_bw_doc.rs"),
        ]);
    }

    #[test]
    fn test_range_bytewise_docstring_mixed() {
        let content = "line 0\n/// line 1\nline 2\n/// line 3\n";
        let test_file = create_test_file("test_range_bw_doc_mixed.rs", content);

        let result = toggle_range_rust_docstring_bytewise(test_file.to_str().unwrap(), 1, 3);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        // Each line toggled independently
        assert_eq!(new_content, "line 0\nline 1\n/// line 2\nline 3\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_range_bw_doc_mixed.rs"),
        ]);
    }

    #[test]
    fn test_range_bytewise_docstring_roundtrip() {
        let original = "line 0\nline 1\nline 2\n";
        let test_file = create_test_file("test_range_bw_doc_roundtrip.rs", original);

        // Toggle on
        let result1 = toggle_range_rust_docstring_bytewise(test_file.to_str().unwrap(), 0, 2);
        assert!(result1.is_ok());

        let content1 = read_file_content(&test_file);
        assert_eq!(content1, "/// line 0\n/// line 1\n/// line 2\n");

        // Toggle off
        let result2 = toggle_range_rust_docstring_bytewise(test_file.to_str().unwrap(), 0, 2);
        assert!(result2.is_ok());

        let content2 = read_file_content(&test_file);
        assert_eq!(content2, original);

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_range_bw_doc_roundtrip.rs"),
        ]);
    }
}

// ============================================================================
// PHASE 3: BLOCK COMMENTS - BYTEWISE OPERATIONS
// ============================================================================

/// Detect if specific line starts with exact byte pattern at column 0
///
/// # Overview
/// Reads first bytes of target line and compares against expected pattern.
/// Used for block comment marker detection.
///
/// # Arguments
/// * `file_path` - Path to source file
/// * `line_number` - Zero-indexed line to check
/// * `pattern` - Exact byte sequence to match (e.g., b"/*\n")
///
/// # Returns
/// * `Ok(true)` - Line starts with exact pattern at column 0
/// * `Ok(false)` - Line doesn't match pattern
/// * `Err(ToggleCommentError)` - File operation failed or line not found
///
/// # Example
/// ```text
/// File line 5: "/*\n"
/// detect_line_pattern(path, 5, b"/*\n") → Ok(true)
///
/// File line 5: "  /*\n"  (indented)
/// detect_line_pattern(path, 5, b"/*\n") → Ok(false)
///
/// File line 5: "/* comment\n"  (has content after)
/// detect_line_pattern(path, 5, b"/*\n") → Ok(false)
/// ```
pub fn detect_line_pattern(
    file_path: &str,
    line_number: usize,
    pattern: &[u8],
) -> Result<bool, ToggleCommentError> {
    // Find line start position
    let line_start_pos =
        match find_line_start_position(file_path, line_number).map_err(|e| match e {
            ToggleIndentError::FileNotFound => ToggleCommentError::FileNotFound,
            ToggleIndentError::LineNotFound {
                requested,
                file_lines,
            } => ToggleCommentError::LineNotFound {
                requested,
                file_lines,
            },
            ToggleIndentError::IoError(op) => ToggleCommentError::IoError(op),
            ToggleIndentError::PathError => ToggleCommentError::PathError,
        })? {
            Some(pos) => pos,
            None => return Ok(false), // Line not found = doesn't match
        };

    let mut file = match File::open(file_path) {
        Ok(f) => f,
        Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Open)),
    };

    // Seek to line start
    if let Err(_) = file.seek(std::io::SeekFrom::Start(line_start_pos)) {
        return Err(ToggleCommentError::IoError(IoOperation::Read));
    }

    let mut byte_bucket: [u8; 1] = [0u8; 1];

    // Compare each byte in pattern
    for &expected_byte in pattern.iter() {
        let bytes_read = match file.read(&mut byte_bucket) {
            Ok(n) => n,
            Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Read)),
        };

        if bytes_read == 0 {
            return Ok(false); // EOF before pattern complete
        }

        if byte_bucket[0] != expected_byte {
            return Ok(false); // Mismatch
        }
    }

    // All bytes matched
    Ok(true)
}

/// Delete entire line from file (bytewise copy, skip target line)
///
/// # Overview
/// Copies source to dest, skipping all bytes of target line including newline.
/// Simple three-part copy: before, skip, after.
///
/// # Arguments
/// * `source_path` - Original file
/// * `dest_path` - Temp file for output
/// * `line_number` - Zero-indexed line to delete
///
/// # Returns
/// * `Ok(())` - Line deleted successfully
/// * `Err(ToggleCommentError)` - Operation failed
///
/// # Memory
/// - 1 byte buffer only
/// - No heap allocation
pub fn delete_line_bytewise(
    source_path: &Path,
    dest_path: &Path,
    line_number: usize,
) -> Result<(), ToggleCommentError> {
    // Find line start position
    let line_start_pos =
        match find_line_start_position(source_path.to_str().unwrap_or(""), line_number).map_err(
            |e| match e {
                ToggleIndentError::FileNotFound => ToggleCommentError::FileNotFound,
                ToggleIndentError::LineNotFound {
                    requested,
                    file_lines,
                } => ToggleCommentError::LineNotFound {
                    requested,
                    file_lines,
                },
                ToggleIndentError::IoError(op) => ToggleCommentError::IoError(op),
                ToggleIndentError::PathError => ToggleCommentError::PathError,
            },
        )? {
            Some(pos) => pos,
            None => {
                return Err(ToggleCommentError::LineNotFound {
                    requested: line_number,
                    file_lines: 0,
                });
            }
        };

    let mut source_file = match File::open(source_path) {
        Ok(f) => f,
        Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Open)),
    };

    let dest_file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dest_path)
    {
        Ok(f) => f,
        Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Create)),
    };

    let mut writer = BufWriter::with_capacity(IO_BUFFER_SIZE, dest_file);
    let mut byte_bucket: [u8; 1] = [0u8; 1];
    let mut byte_position: u64 = 0;

    // PART A: Copy before target line
    while byte_position < line_start_pos {
        if byte_position >= MAX_BYTE_ITERATIONS {
            return Err(ToggleCommentError::IoError(IoOperation::Read));
        }

        let bytes_read = match source_file.read(&mut byte_bucket) {
            Ok(n) => n,
            Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Read)),
        };

        if bytes_read == 0 {
            return Err(ToggleCommentError::IoError(IoOperation::Read));
        }

        if let Err(_) = writer.write_all(&byte_bucket) {
            return Err(ToggleCommentError::IoError(IoOperation::Write));
        }

        byte_position += 1;
    }

    // PART B: Skip entire target line (don't write anything)
    loop {
        let bytes_read = match source_file.read(&mut byte_bucket) {
            Ok(n) => n,
            Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Read)),
        };

        if bytes_read == 0 {
            break; // EOF - line had no newline
        }

        if byte_bucket[0] == b'\n' {
            break; // Found newline - skip it too, line is deleted
        }
    }

    // PART C: Copy rest of file
    let mut remaining_bytes: u64 = 0;
    loop {
        if remaining_bytes >= MAX_BYTE_ITERATIONS {
            return Err(ToggleCommentError::IoError(IoOperation::Read));
        }

        let bytes_read = match source_file.read(&mut byte_bucket) {
            Ok(n) => n,
            Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Read)),
        };

        if bytes_read == 0 {
            break;
        }

        if let Err(_) = writer.write_all(&byte_bucket) {
            return Err(ToggleCommentError::IoError(IoOperation::Write));
        }

        remaining_bytes += 1;
    }

    if let Err(_) = writer.flush() {
        return Err(ToggleCommentError::IoError(IoOperation::Flush));
    }

    Ok(())
}

/// Insert new line before target line (bytewise)
///
/// # Overview
/// Copies source to dest, inserting new line content before target line.
///
/// # Arguments
/// * `source_path` - Original file
/// * `dest_path` - Temp file for output
/// * `line_number` - Zero-indexed line to insert before
/// * `content` - Bytes to insert (should include newline)
///
/// # Returns
/// * `Ok(())` - Line inserted successfully
/// * `Err(ToggleCommentError)` - Operation failed
pub fn insert_line_before_bytewise(
    source_path: &Path,
    dest_path: &Path,
    line_number: usize,
    content: &[u8],
) -> Result<(), ToggleCommentError> {
    // Find line start position
    let line_start_pos =
        match find_line_start_position(source_path.to_str().unwrap_or(""), line_number).map_err(
            |e| match e {
                ToggleIndentError::FileNotFound => ToggleCommentError::FileNotFound,
                ToggleIndentError::LineNotFound {
                    requested,
                    file_lines,
                } => ToggleCommentError::LineNotFound {
                    requested,
                    file_lines,
                },
                ToggleIndentError::IoError(op) => ToggleCommentError::IoError(op),
                ToggleIndentError::PathError => ToggleCommentError::PathError,
            },
        )? {
            Some(pos) => pos,
            None => {
                return Err(ToggleCommentError::LineNotFound {
                    requested: line_number,
                    file_lines: 0,
                });
            }
        };

    let mut source_file = match File::open(source_path) {
        Ok(f) => f,
        Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Open)),
    };

    let dest_file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dest_path)
    {
        Ok(f) => f,
        Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Create)),
    };

    let mut writer = BufWriter::with_capacity(IO_BUFFER_SIZE, dest_file);
    let mut byte_bucket: [u8; 1] = [0u8; 1];
    let mut byte_position: u64 = 0;

    // PART A: Copy before target line
    while byte_position < line_start_pos {
        if byte_position >= MAX_BYTE_ITERATIONS {
            return Err(ToggleCommentError::IoError(IoOperation::Read));
        }

        let bytes_read = match source_file.read(&mut byte_bucket) {
            Ok(n) => n,
            Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Read)),
        };

        if bytes_read == 0 {
            return Err(ToggleCommentError::IoError(IoOperation::Read));
        }

        if let Err(_) = writer.write_all(&byte_bucket) {
            return Err(ToggleCommentError::IoError(IoOperation::Write));
        }

        byte_position += 1;
    }

    // PART B: Insert new content
    if let Err(_) = writer.write_all(content) {
        return Err(ToggleCommentError::IoError(IoOperation::Write));
    }

    // PART C: Copy rest of file
    let mut remaining_bytes: u64 = 0;
    loop {
        if remaining_bytes >= MAX_BYTE_ITERATIONS {
            return Err(ToggleCommentError::IoError(IoOperation::Read));
        }

        let bytes_read = match source_file.read(&mut byte_bucket) {
            Ok(n) => n,
            Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Read)),
        };

        if bytes_read == 0 {
            break;
        }

        if let Err(_) = writer.write_all(&byte_bucket) {
            return Err(ToggleCommentError::IoError(IoOperation::Write));
        }

        remaining_bytes += 1;
    }

    if let Err(_) = writer.flush() {
        return Err(ToggleCommentError::IoError(IoOperation::Flush));
    }

    Ok(())
}

/// Insert new line after target line (bytewise)
///
/// # Overview
/// Copies source to dest, inserting new line content after target line's newline.
///
/// # Arguments
/// * `source_path` - Original file
/// * `dest_path` - Temp file for output
/// * `line_number` - Zero-indexed line to insert after
/// * `content` - Bytes to insert (should include newline)
///
/// # Returns
/// * `Ok(())` - Line inserted successfully
/// * `Err(ToggleCommentError)` - Operation failed
pub fn insert_line_after_bytewise(
    source_path: &Path,
    dest_path: &Path,
    line_number: usize,
    content: &[u8],
) -> Result<(), ToggleCommentError> {
    // Find line start position
    let line_start_pos =
        match find_line_start_position(source_path.to_str().unwrap_or(""), line_number).map_err(
            |e| match e {
                ToggleIndentError::FileNotFound => ToggleCommentError::FileNotFound,
                ToggleIndentError::LineNotFound {
                    requested,
                    file_lines,
                } => ToggleCommentError::LineNotFound {
                    requested,
                    file_lines,
                },
                ToggleIndentError::IoError(op) => ToggleCommentError::IoError(op),
                ToggleIndentError::PathError => ToggleCommentError::PathError,
            },
        )? {
            Some(pos) => pos,
            None => {
                return Err(ToggleCommentError::LineNotFound {
                    requested: line_number,
                    file_lines: 0,
                });
            }
        };

    let mut source_file = match File::open(source_path) {
        Ok(f) => f,
        Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Open)),
    };

    let dest_file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dest_path)
    {
        Ok(f) => f,
        Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Create)),
    };

    let mut writer = BufWriter::with_capacity(IO_BUFFER_SIZE, dest_file);
    let mut byte_bucket: [u8; 1] = [0u8; 1];
    let mut byte_position: u64 = 0;

    // PART A: Copy up to and including target line
    while byte_position < line_start_pos {
        if byte_position >= MAX_BYTE_ITERATIONS {
            return Err(ToggleCommentError::IoError(IoOperation::Read));
        }

        let bytes_read = match source_file.read(&mut byte_bucket) {
            Ok(n) => n,
            Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Read)),
        };

        if bytes_read == 0 {
            return Err(ToggleCommentError::IoError(IoOperation::Read));
        }

        if let Err(_) = writer.write_all(&byte_bucket) {
            return Err(ToggleCommentError::IoError(IoOperation::Write));
        }

        byte_position += 1;
    }

    // PART B: Copy target line until newline
    loop {
        let bytes_read = match source_file.read(&mut byte_bucket) {
            Ok(n) => n,
            Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Read)),
        };

        if bytes_read == 0 {
            // EOF - no newline at end, insert content here
            if let Err(_) = writer.write_all(b"\n") {
                return Err(ToggleCommentError::IoError(IoOperation::Write));
            }
            if let Err(_) = writer.write_all(content) {
                return Err(ToggleCommentError::IoError(IoOperation::Write));
            }
            break;
        }

        if let Err(_) = writer.write_all(&byte_bucket) {
            return Err(ToggleCommentError::IoError(IoOperation::Write));
        }

        if byte_bucket[0] == b'\n' {
            // Found newline - insert after it
            if let Err(_) = writer.write_all(content) {
                return Err(ToggleCommentError::IoError(IoOperation::Write));
            }
            break;
        }
    }

    // PART C: Copy rest of file
    let mut remaining_bytes: u64 = 0;
    loop {
        if remaining_bytes >= MAX_BYTE_ITERATIONS {
            return Err(ToggleCommentError::IoError(IoOperation::Read));
        }

        let bytes_read = match source_file.read(&mut byte_bucket) {
            Ok(n) => n,
            Err(_) => return Err(ToggleCommentError::IoError(IoOperation::Read)),
        };

        if bytes_read == 0 {
            break;
        }

        if let Err(_) = writer.write_all(&byte_bucket) {
            return Err(ToggleCommentError::IoError(IoOperation::Write));
        }

        remaining_bytes += 1;
    }

    if let Err(_) = writer.flush() {
        return Err(ToggleCommentError::IoError(IoOperation::Flush));
    }

    Ok(())
}

/// Toggle block comment markers around range of lines (bytewise)
///
/// # Overview
/// Adds or removes block comment markers (`/* */` or `"""`) at column 0.
/// Markers placed on their own lines before/after content.
///
/// # Rules
/// - **Column 0 only** - no indentation handling
/// - **Single line (start == end)** - always ADD mode
/// - **Detection** - if BOTH lines start with markers → REMOVE, else ADD
/// - **ADD mode** - insert new line before start, new line after end
/// - **REMOVE mode** - delete end line first, then start line
///
/// # Arguments
/// * `file_path` - Path to source file
/// * `start_line` - First line of content range (zero-indexed)
/// * `end_line` - Last line of content range (zero-indexed)
///
/// # Returns
/// * `Ok(())` - Block comment toggled successfully
/// * `Err(ToggleCommentError)` - Specific error code
///
/// # Example (Rust - ADD mode)
/// ```text
/// Before:
/// line 5: code line 1
/// line 6: code line 2
///
/// After toggle_block_comment_bytewise(path, 5, 6):
/// line 5: /*
/// line 6: code line 1
/// line 7: code line 2
/// line 8: */
/// ```
///
/// # Example (Rust - REMOVE mode)
/// ```text
/// Before:
/// line 5: /*
/// line 6: code line 1
/// line 7: code line 2
/// line 8: */
///
/// After toggle_block_comment_bytewise(path, 5, 8):
/// line 5: code line 1
/// line 6: code line 2
/// ```
///
/// # Example (Python)
/// ```text
/// Uses """ markers instead of /* */
/// ```
pub fn toggle_block_comment_bytewise(
    file_path: &str,
    start_line: usize,
    end_line: usize,
) -> Result<(), ToggleCommentError> {
    // Sort range
    let (start, end) = sort_range(start_line, end_line);

    // Convert to absolute path
    let absolute_path = match Path::new(file_path).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err(ToggleCommentError::FileNotFound);
            }
            return Err(ToggleCommentError::PathError);
        }
    };

    // Determine block markers from extension
    let extension = match absolute_path.extension() {
        Some(ext) => ext.to_string_lossy().to_string(),
        None => return Err(ToggleCommentError::NoExtension),
    };

    let markers = match determine_block_markers(&extension) {
        Some(m) => m,
        None => return Ok(()), // Unsupported - no-op
    };

    // Get filename
    let filename = match absolute_path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => return Err(ToggleCommentError::PathError),
    };

    // EDGE CASE: Single line always ADD mode
    if start == end {
        // Create backup
        let backup_filename = format!("backup_toggle_comment_{}", filename);
        let backup_path = PathBuf::from(&backup_filename);
        if let Err(_) = std::fs::copy(&absolute_path, &backup_path) {
            return Err(ToggleCommentError::IoError(IoOperation::Backup));
        }

        // Insert closing marker after line (do this first so line numbers don't shift)
        let temp1_filename = format!("temp_block_1_{}_{}", std::process::id(), filename);
        let temp1_path = PathBuf::from(&temp1_filename);
        insert_line_after_bytewise(&absolute_path, &temp1_path, start, markers.end)?;

        // Replace original with temp1
        if let Err(_) = std::fs::copy(&temp1_path, &absolute_path) {
            let _ = std::fs::remove_file(&temp1_path);
            return Err(ToggleCommentError::IoError(IoOperation::Replace));
        }

        // Insert opening marker before line
        let temp2_filename = format!("temp_block_2_{}_{}", std::process::id(), filename);
        let temp2_path = PathBuf::from(&temp2_filename);
        insert_line_before_bytewise(&absolute_path, &temp2_path, start, markers.start)?;

        // Replace original with temp2
        if let Err(_) = std::fs::copy(&temp2_path, &absolute_path) {
            let _ = std::fs::remove_file(&temp2_path);
            return Err(ToggleCommentError::IoError(IoOperation::Replace));
        }

        // Cleanup temps
        let _ = std::fs::remove_file(&temp1_path);
        let _ = std::fs::remove_file(&temp2_path);

        return Ok(());
    }

    // DETECT MODE: Check if both markers present at column 0
    let start_has_marker = detect_line_pattern(file_path, start, markers.start)?;
    let end_has_marker = detect_line_pattern(file_path, end, markers.end)?;

    let mode = if start_has_marker && end_has_marker {
        BlockMode::Remove
    } else {
        BlockMode::Add
    };

    // Create backup
    let backup_filename = format!("backup_toggle_comment_{}", filename);
    let backup_path = PathBuf::from(&backup_filename);
    if let Err(_) = std::fs::copy(&absolute_path, &backup_path) {
        return Err(ToggleCommentError::IoError(IoOperation::Backup));
    }

    match mode {
        BlockMode::Remove => {
            // DELETE end_line FIRST (so start_line number stays valid)
            let temp1_filename = format!("temp_block_1_{}_{}", std::process::id(), filename);
            let temp1_path = PathBuf::from(&temp1_filename);
            delete_line_bytewise(&absolute_path, &temp1_path, end)?;

            // Replace original
            if let Err(_) = std::fs::copy(&temp1_path, &absolute_path) {
                let _ = std::fs::remove_file(&temp1_path);
                return Err(ToggleCommentError::IoError(IoOperation::Replace));
            }

            // DELETE start_line
            let temp2_filename = format!("temp_block_2_{}_{}", std::process::id(), filename);
            let temp2_path = PathBuf::from(&temp2_filename);
            delete_line_bytewise(&absolute_path, &temp2_path, start)?;

            // Replace original
            if let Err(_) = std::fs::copy(&temp2_path, &absolute_path) {
                let _ = std::fs::remove_file(&temp2_path);
                return Err(ToggleCommentError::IoError(IoOperation::Replace));
            }

            // Cleanup temps
            let _ = std::fs::remove_file(&temp1_path);
            let _ = std::fs::remove_file(&temp2_path);
        }

        BlockMode::Add => {
            // INSERT closing marker after end_line (do this first)
            let temp1_filename = format!("temp_block_1_{}_{}", std::process::id(), filename);
            let temp1_path = PathBuf::from(&temp1_filename);
            insert_line_after_bytewise(&absolute_path, &temp1_path, end, markers.end)?;

            // Replace original
            if let Err(_) = std::fs::copy(&temp1_path, &absolute_path) {
                let _ = std::fs::remove_file(&temp1_path);
                return Err(ToggleCommentError::IoError(IoOperation::Replace));
            }

            // INSERT opening marker before start_line
            let temp2_filename = format!("temp_block_2_{}_{}", std::process::id(), filename);
            let temp2_path = PathBuf::from(&temp2_filename);
            insert_line_before_bytewise(&absolute_path, &temp2_path, start, markers.start)?;

            // Replace original
            if let Err(_) = std::fs::copy(&temp2_path, &absolute_path) {
                let _ = std::fs::remove_file(&temp2_path);
                return Err(ToggleCommentError::IoError(IoOperation::Replace));
            }

            // Cleanup temps
            let _ = std::fs::remove_file(&temp1_path);
            let _ = std::fs::remove_file(&temp2_path);
        }
    }

    Ok(())
}

// ============================================================================
// TESTS - PHASE 3: BLOCK COMMENTS BYTEWISE
// ============================================================================

#[cfg(test)]
mod block_comment_bytewise_tests {
    use super::*;

    // ========================================
    // Detection Tests
    // ========================================

    #[test]
    fn test_detect_line_pattern_match() {
        let content = "line 0\n/*\nline 2\n";
        let test_file = create_test_file("test_detect_match.rs", content);

        let result = detect_line_pattern(test_file.to_str().unwrap(), 1, b"/*\n");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);

        cleanup_files(&[&test_file]);
    }

    #[test]
    fn test_detect_line_pattern_no_match() {
        let content = "line 0\ncode\nline 2\n";
        let test_file = create_test_file("test_detect_no_match.rs", content);

        let result = detect_line_pattern(test_file.to_str().unwrap(), 1, b"/*\n");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);

        cleanup_files(&[&test_file]);
    }

    #[test]
    fn test_detect_line_pattern_indented() {
        let content = "line 0\n  /*\nline 2\n";
        let test_file = create_test_file("test_detect_indented.rs", content);

        // Should NOT match - pattern must be at column 0
        let result = detect_line_pattern(test_file.to_str().unwrap(), 1, b"/*\n");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);

        cleanup_files(&[&test_file]);
    }

    // ========================================
    // Delete Line Tests
    // ========================================

    #[test]
    fn test_delete_line_bytewise_middle() {
        let content = "line 0\nline 1\nline 2\n";
        let test_file = create_test_file("test_delete_middle.txt", content);
        let temp_file = PathBuf::from("temp_delete_test.txt");

        let result = delete_line_bytewise(&test_file, &temp_file, 1);
        assert!(result.is_ok());

        let new_content = read_file_content(&temp_file);
        assert_eq!(new_content, "line 0\nline 2\n");

        cleanup_files(&[&test_file, &temp_file]);
    }

    #[test]
    fn test_delete_line_bytewise_first() {
        let content = "line 0\nline 1\nline 2\n";
        let test_file = create_test_file("test_delete_first.txt", content);
        let temp_file = PathBuf::from("temp_delete_first.txt");

        let result = delete_line_bytewise(&test_file, &temp_file, 0);
        assert!(result.is_ok());

        let new_content = read_file_content(&temp_file);
        assert_eq!(new_content, "line 1\nline 2\n");

        cleanup_files(&[&test_file, &temp_file]);
    }

    // ========================================
    // Insert Line Tests
    // ========================================

    #[test]
    fn test_insert_line_before_bytewise() {
        let content = "line 0\nline 1\nline 2\n";
        let test_file = create_test_file("test_insert_before.txt", content);
        let temp_file = PathBuf::from("temp_insert_before.txt");

        let result = insert_line_before_bytewise(&test_file, &temp_file, 1, b"NEW\n");
        assert!(result.is_ok());

        let new_content = read_file_content(&temp_file);
        assert_eq!(new_content, "line 0\nNEW\nline 1\nline 2\n");

        cleanup_files(&[&test_file, &temp_file]);
    }

    #[test]
    fn test_insert_line_after_bytewise() {
        let content = "line 0\nline 1\nline 2\n";
        let test_file = create_test_file("test_insert_after.txt", content);
        let temp_file = PathBuf::from("temp_insert_after.txt");

        let result = insert_line_after_bytewise(&test_file, &temp_file, 1, b"NEW\n");
        assert!(result.is_ok());

        let new_content = read_file_content(&temp_file);
        assert_eq!(new_content, "line 0\nline 1\nNEW\nline 2\n");

        cleanup_files(&[&test_file, &temp_file]);
    }

    // ========================================
    // Block Comment Toggle Tests
    // ========================================

    #[test]
    fn test_block_comment_bytewise_add_rust() {
        let content = "code line 1\ncode line 2\n";
        let test_file = create_test_file("test_block_add.rs", content);

        let result = toggle_block_comment_bytewise(test_file.to_str().unwrap(), 0, 1);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "/*\ncode line 1\ncode line 2\n*/\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_block_add.rs"),
        ]);
    }

    #[test]
    fn test_block_comment_bytewise_remove_rust() {
        let content = "/*\ncode line 1\ncode line 2\n*/\n";
        let test_file = create_test_file("test_block_remove.rs", content);

        let result = toggle_block_comment_bytewise(test_file.to_str().unwrap(), 0, 3);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "code line 1\ncode line 2\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_block_remove.rs"),
        ]);
    }

    #[test]
    fn test_block_comment_bytewise_single_line() {
        let content = "code line\n";
        let test_file = create_test_file("test_block_single.rs", content);

        // Single line always ADD
        let result = toggle_block_comment_bytewise(test_file.to_str().unwrap(), 0, 0);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "/*\ncode line\n*/\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_block_single.rs"),
        ]);
    }

    #[test]
    fn test_block_comment_bytewise_python() {
        let content = "code line 1\ncode line 2\n";
        let test_file = create_test_file("test_block_python.py", content);

        let result = toggle_block_comment_bytewise(test_file.to_str().unwrap(), 0, 1);
        assert!(result.is_ok());

        let new_content = read_file_content(&test_file);
        assert_eq!(new_content, "\"\"\"\ncode line 1\ncode line 2\n\"\"\"\n");

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_block_python.py"),
        ]);
    }

    #[test]
    fn test_block_comment_bytewise_roundtrip() {
        let original = "code line 1\ncode line 2\n";
        let test_file = create_test_file("test_block_roundtrip.rs", original);

        // Add
        let result1 = toggle_block_comment_bytewise(test_file.to_str().unwrap(), 0, 1);
        assert!(result1.is_ok());

        let content1 = read_file_content(&test_file);
        assert_eq!(content1, "/*\ncode line 1\ncode line 2\n*/\n");

        // Remove
        let result2 = toggle_block_comment_bytewise(test_file.to_str().unwrap(), 0, 3);
        assert!(result2.is_ok());

        let content2 = read_file_content(&test_file);
        assert_eq!(content2, original);

        cleanup_files(&[
            &test_file,
            &PathBuf::from("backup_toggle_comment_test_block_roundtrip.rs"),
        ]);
    }
}
