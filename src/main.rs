// lines is minimal text editor
// test files in: src/tests.rs

/*
Policy and Rust Rules

1. Lines is to be a 'no load' 'load only what is needed only when it is needed' application.

2. Any operation of unknown size is broken up into chunks and handled modularly.

3. Let it fail and try again. This is production software that must not crash/panic ever.
Every part of every function will fail at some point, if only due to cosmic ray bitflips
which are common on a larger scape. When, not if, when something fails
the application must gracefully let it fail and move on, e.g.
returning to the last non-failed step. It is the user's choice to try again or not.

4. Error messages use pre-allocated buffers are are very terse for terminal,
longer errors can be appended to log files. No heap.


Rust rules:
Always best practice.
Always extensive doc strings.
Always comments.
Always cargo tests (where possible).
Never remove documentation.
Always clear, meaningful, unique names (e.g. variables, functions).
Always absolute file paths.
Always error handling.
Never unsafe code.
Never use unwrap.

Load what is needed when it is needed: Do not ever load a whole file,
rarely load a whole anything. increment and load only what is required pragmatically.

Following NASA's 'Power of 10 rules' (updated for 2025 and Rust):
1. no unsafe stuff:
- no recursion
- no goto
- no pointers
- no preprocessor

2. upper bound on normal-loops, failsafe for always-loops

3. Pre-allocate all memory (no dynamic memory allocation)

4. functions have narrow focus
s
5. Defensive programming:
- average to a minimum of two assertions per function
- cargo tests
- error handling details
- uses of Option


6. ? Is this about ownership of variables?

7. manage return values:
null-void return values & checking non-void-null returns

8.
9. Communicate:
doc strings, comments, use case, edge case,


Always defensive best practice:
Always error handling: everything will fail at some point,
if only because of cosmic-ray bit-flips (which are actually common),
there must always be fail-safe error handling.

Safety, reliability, maintainability, fail-safe, communication-documentation, are the goals.

No third party libraries (or strictly avoid third party libraries where possible).

*/

/*


This code is under construction! Some code below may not be correct.
Any code that violates the roles and policies is wrong or placeholder-code.



# Build-Plan A


Basic features to be use-able, with a design such that planned scope can be modularly added (without a need to go back and re-design & rebuild everything).

1. open a read-copy file: call lines from cli with a file-path
- use function input path from argument handled by wrapper
- make timestamped session directory in exe relative abs path directory lines_data/tmp/sessions/{timestamp}/
- save read-copy of file

main() is a wrapper that handles arguments, e.g. get path from argument and feed it in, later help menu, etc.

lines_editor_module(option(path)) is the main lines application-wrapper, called by main (or called by another application into which the lines editor is added as a module)


lines_data/tmp/sessions/{timestamp}/{timestamp}_{filename}
2. save paths in state:
- original file path
[done] - readcopy file path

[done] - added timestamps (made timestamp crate) (from ff)
[done] - added abs exe parent-relative paths (From ff)

3. modular command handling:
[done]- modes ( look at ff mode/command-modules )
[done]- --help (super mvp)
?done? - any new command modules added
- add source-it

4. Cursor system: "Plus Enter" cursor system.
[done] 1. Add cursor etc. (from POC)
- int+letter, move N spaces (in POC, but backwards from vim, use vim int+letter)
2. **Add scroll down** - Increment line number, rebuild window
3. **Add scroll up** - Decrement line number, rebuild window
4. **Test** - Verify line numbers track correctly
5. scroll right (see unwrapped long lines)
6. scroll back to left
7. w,e,b, normal mode move
8. select code (even if select doesn't do anything now) ( visual mode, works in POC)

5. insert:
- start of insert mode: user types into buffer
- user types into input buffer,
- add input buffer byte to file at mapped cursor location: changes made to file
- reload window from newly changed file
maybe add another item into state:
( maybe step to store in state last number of cursor spaces in input buffer
- move cursor to end of new addition,
- back to start of insert mode
- probably leave insert mode with reserved command strings: -n -v --normal --visual -s --save (saves and returns to normal?) (or -w --write, or -wq --writequit

6. s/w command to save changes to original file
- first makes a backup of the original file in same parent /archive/{timestamp}_{filename} (or, thought of differently, moves the original file as is and re-names it)
- replaces the old file with the new one (copies the read-copy to the original location

7. works on: linux, android-termux, BSD-MacOS, windows, redox, etc.

8. For MVP, calling Lines with path argument in home directory launches the ultra-minimal legacy Memo-Mode which is simple and stable (that does not need to launch full-lines with state etc.)

9. calling lines not in home directory should...first ask for file name?

10. calling lines with a path that does not yet exit, make those dirs and or file and launch full-lines


...

TODO:
fix bug, at some point a move*2 bug was introduced
one space is 2, 10l move 20
uniform across up down left right... odd

...

## In Scope:
- modular structure (lines IS a module)
- Opening files (creating new, opening existing)
- Viewing file contents in a "sliding window" (80×24 default, up to 320×96)
- Navigation (hjkl, word boundaries, goto line)
- Basic editing (insert characters & newlines, delete characters)
- Line operations (delete line, comment/uncomment)
- Save operations (save, save-as)
- File safety (read-copies, timestamped archives)
- Line wrapping toggle
- Visual mode (selection)
- Line numbers (absolute/relative)
- Multi-cursor/ctrl+d functionality
~ "Plugin" architecture: modular system for commands
- 'Memo' Mode: (quick-start exists in original Lines)
- open to line
- go to end of file (just iterate, must see line number)


## Future/Probably Scope:
- relative lines
- fancier delete options: array slice of char, array slice of lines (relative) (absolute?)
- simple raw hex bytes view
- raw text view (seeing escape characters)
- comment-line / un-comment line (command is '/+enter'), file extension for standard languages should make this simple .py .rs .js .toml etc.
- Character encoding awareness
- extended delete (line array slice)
- Undo (optional, with constrained history buffers)
- Encoding conversion (write in different encodings)
- Hex editor dual-view
- Search,
- fuzzy search,
- regex search,
- some Extended goto commands
- Configuration files
- help menu
- build .rs for --version
- source-it (see File Fantastic)
- super-mini directory file manager, for if "lines ." open in dir (list file/dir by number, if select file open in lines, if select dir show fiies, option 1 is back, option)
- Byte viewing mode
- Byte editing
- some way to 'view raw characters' e.g. so that escape characters are shown and ideally entered to

*/

/*
POC cursor system

// ANSI escape codes
const CLEAR_SCREEN: &str = "\x1b[2J";
const RESET_CURSOR: &str = "\x1b[H";
const RESET_STYLE: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const BLUE: &str = "\x1b[34m";
const YELLOW: &str = "\x1b[33m";
// const CYAN: &str = "\x1b[36m";
const BG_WHITE: &str = "\x1b[47m";
// const BG_BLACK: &str = "\x1b[40m";
// const BG_YELLOW: &str = "\x1b[43m";
const BG_CYAN: &str = "\x1b[46m";

fn move_cursor_to(x: usize, y: usize) {
    print!("\x1b[{};{}H", y + 1, x + 1);
}

fn parse_command(input: &str) -> (Option<char>, usize) {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return (None, 1);
    }

    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() == 1 {
        return (Some(chars[0]), 1);
    }

    if let Some(first_char) = chars.first() {
        if "hjklweb".contains(*first_char) {
            let number_part: String = chars[1..].iter().collect();
            if let Ok(count) = number_part.parse::<usize>() {
                return (Some(*first_char), count.min(100));
            }
        }
    }

    (Some(chars[0]), 1)
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn find_word_end(line: &[char], start_x: usize) -> usize {
    if start_x >= line.len() {
        return start_x;
    }

    let mut x = start_x;

    // If we're on a word character, skip to end of current word
    if x < line.len() && is_word_char(line[x]) {
        while x < line.len() && is_word_char(line[x]) {
            x += 1;
        }
        if x < line.len() {
            return x - 1; // Position on last char of word
        }
        return x.saturating_sub(1);
    }

    // Skip non-word characters
    while x < line.len() && !is_word_char(line[x]) {
        x += 1;
    }

    // Find end of next word
    while x < line.len() && is_word_char(line[x]) {
        x += 1;
    }

    x.saturating_sub(1).min(line.len().saturating_sub(1))
}

fn find_word_beginning(line: &[char], start_x: usize) -> usize {
    if line.is_empty() || start_x == 0 {
        return 0;
    }

    let mut x = start_x.min(line.len().saturating_sub(1));

    // If we're on a word character and not at the beginning of a word, go to current word's beginning
    if x > 0 && x < line.len() && is_word_char(line[x]) && is_word_char(line[x - 1]) {
        while x > 0 && is_word_char(line[x - 1]) {
            x -= 1;
        }
        return x;
    }

    // Skip backward over non-word characters
    while x > 0 && !is_word_char(line[x.min(line.len() - 1)]) {
        x -= 1;
    }

    // Find beginning of previous word
    while x > 0 && is_word_char(line[x]) {
        x -= 1;
    }

    if x > 0 || !is_word_char(line[0]) {
        x + 1
    } else {
        0
    }
}

fn find_next_word_start(line: &[char], start_x: usize) -> usize {
    if start_x >= line.len() {
        return line.len();
    }

    let mut x = start_x;

    // Skip current word
    while x < line.len() && is_word_char(line[x]) {
        x += 1;
    }

    // Skip spaces/punctuation
    while x < line.len() && !is_word_char(line[x]) {
        x += 1;
    }

    x
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Position {
    x: usize,
    y: usize,
}

fn main() {
    let mut cursor = Position { x: 0, y: 0 };
    let mut selection_start: Option<Position> = None;
    let mut selection_end: Option<Position> = None;
    let mut visual_mode = false;

    let mut content: Vec<Vec<char>> = vec![
        "Hello, World! This is a test.".chars().collect(),
        "This is a minimal editor with selections".chars().collect(),
        "Use hjkl to move cursor, w/e/b for words".chars().collect(),
        "Press v to start/stop visual mode".chars().collect(),
        "Press q + Enter to quit".chars().collect(),
        "foo_bar baz-qux some.text here".chars().collect(),
        "".chars().collect(),
    ];

    while content.len() < 24 {
        content.push(Vec::new());
    }

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut last_command_info = String::new();

    loop {
        print!("{}{}", CLEAR_SCREEN, RESET_CURSOR);

        // Draw the content with selection highlighting
        for (y, line) in content.iter().enumerate() {
            if y >= 24 {
                break;
            }

            move_cursor_to(0, y);

            for (x, &ch) in line.iter().enumerate() {
                if x >= 80 {
                    break;
                }

                let pos = Position { x, y };
                let is_cursor = x == cursor.x && y == cursor.y;

                // Check if position is in selection
                let in_selection =
                    if let (Some(start), Some(end)) = (selection_start, selection_end) {
                        // Handle multi-line selection
                        if start.y == end.y {
                            y == start.y && x >= start.x.min(end.x) && x <= start.x.max(end.x)
                        } else if start.y < end.y {
                            (y == start.y && x >= start.x)
                                || (y > start.y && y < end.y)
                                || (y == end.y && x <= end.x)
                        } else {
                            (y == end.y && x >= end.x)
                                || (y > end.y && y < start.y)
                                || (y == start.y && x <= start.x)
                        }
                    } else {
                        false
                    };

                if is_cursor {
                    print!("{}{}{}{}{}", BOLD, RED, BG_WHITE, ch, RESET_STYLE);
                } else if in_selection {
                    print!("{}{}{}{}{}", BOLD, YELLOW, BG_CYAN, ch, RESET_STYLE);
                } else {
                    print!("{}", ch);
                }
            }

            // Show cursor if beyond line end
            if cursor.x >= line.len() && y == cursor.y {
                move_cursor_to(cursor.x, y);
                print!("{}{}{} {}", BOLD, RED, BG_WHITE, RESET_STYLE);
            }

            // Highlight empty space in selection
            if let (Some(start), Some(end)) = (selection_start, selection_end) {
                let line_len = line.len();
                if y >= start.y.min(end.y) && y <= start.y.max(end.y) {
                    let selection_end_x = if y == end.y.max(start.y) {
                        end.x.max(start.x)
                    } else {
                        79
                    };

                    for x in line_len..=selection_end_x.min(79) {
                        if x == cursor.x && y == cursor.y {
                            continue; // Already drawn cursor
                        }
                        move_cursor_to(x, y);
                        print!("{}{}{} {}", BOLD, YELLOW, BG_CYAN, RESET_STYLE);
                    }
                }
            }
        }

        // Status lines
        move_cursor_to(0, 22);
        print!(
            "{}{}Pos: ({},{}) | Mode: {} | Last: {} {}",
            BOLD,
            BLUE,
            cursor.x,
            cursor.y,
            if visual_mode { "VISUAL" } else { "NORMAL" },
            last_command_info,
            RESET_STYLE
        );

        move_cursor_to(0, 23);
        print!(
            "{}{}Commands: hjkl=move, w/e/b=word, v=visual, q=quit | Enter: {}",
            BOLD, GREEN, RESET_STYLE
        );

        io::stdout().flush().unwrap();

        let mut input = String::new();
        handle.read_line(&mut input).unwrap();
        let (command, repeat_count) = parse_command(&input);

        if let Some(cmd) = command {
            let old_cursor = cursor;

            match cmd {
                'h' => {
                    let moves = repeat_count.min(cursor.x);
                    cursor.x -= moves;
                    last_command_info = format!("Left {}", moves);
                }
                'l' => {
                    let moves = repeat_count.min(79 - cursor.x);
                    cursor.x += moves;
                    last_command_info = format!("Right {}", moves);
                }
                'k' => {
                    let moves = repeat_count.min(cursor.y);
                    cursor.y -= moves;
                    last_command_info = format!("Up {}", moves);
                }
                'j' => {
                    let moves = repeat_count.min(23 - cursor.y);
                    cursor.y += moves;
                    last_command_info = format!("Down {}", moves);
                }
                'w' => {
                    // Move to next word start
                    for _ in 0..repeat_count {
                        if cursor.y < content.len() {
                            let new_x = find_next_word_start(&content[cursor.y], cursor.x);
                            cursor.x = new_x.min(79);
                        }
                    }
                    last_command_info = format!("Word forward {}", repeat_count);
                }
                'e' => {
                    // Move to word end
                    for _ in 0..repeat_count {
                        if cursor.y < content.len() {
                            let new_x = find_word_end(&content[cursor.y], cursor.x);
                            cursor.x = new_x.min(79);
                        }
                    }
                    last_command_info = format!("Word end {}", repeat_count);
                }
                'b' => {
                    // Move to word beginning
                    for _ in 0..repeat_count {
                        if cursor.y < content.len() {
                            cursor.x = find_word_beginning(&content[cursor.y], cursor.x);
                        }
                    }
                    last_command_info = format!("Word back {}", repeat_count);
                }
                'v' => {
                    // Toggle visual mode
                    visual_mode = !visual_mode;
                    if visual_mode {
                        selection_start = Some(cursor);
                        selection_end = Some(cursor);
                        last_command_info = "Visual mode ON".to_string();
                    } else {
                        selection_start = None;
                        selection_end = None;
                        last_command_info = "Visual mode OFF".to_string();
                    }
                }
                'q' => {
                    print!("{}{}", CLEAR_SCREEN, RESET_CURSOR);
                    println!("Goodbye!");
                    break;
                }
                _ => {
                    last_command_info = format!("Unknown: {}", cmd);
                }
            }

            // Update selection end if in visual mode and cursor moved
            if visual_mode && cursor != old_cursor {
                selection_end = Some(cursor);
            }
        }
    }
}
 */

/*
* Spec Notes:

1. copy() in Rust uses stack: So no additional preallocations needed ...right??? TODO: check this

2. Input System Architecture:

- Lines uses Rust's standard stdin().read_line() which reads until Enter is pressed
    uses a String (which DOES use heap - that's the OS/Rust input buffer)
- NO direct keypress detection (no raw terminal)
- Everything is "command + Enter"
*/

use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Tests are in src/tests.rs
#[cfg(test)]
mod tests;

/// state.rs - Core editor state management with pre-allocated buffers
///
/// This module manages all editor state using only pre-allocated memory.
/// No dynamic allocation is performed after initialization.
/// All operations are bounds-checked and error-handled.
///
/// Maximum buffer size for window content is (2^13 = 8192 bytes)
///
/// Sorry: The calculation is not simple,
/// a 80x24 terminal size 3 minus header, 3 footer, line-number * 4
/// is (rows, cols) 45*157 TUI-text size * 3-bytes UTF-8
///
/// default normal size is 77*21 = 1617 bytes
/// but most of that is empty space, likely ~800 character bytes.
///
/// Also, as line number grows, more columns lost to that
///
const FILE_TUI_WINDOW_MAP_BUFFER_SIZE: usize = 8192; // 2**13=8192

/// Maximum number of rows (lines) in largest supported terminal
/// of which 45 can be file rows (there are 45 tui line buffers)
const MAX_TUI_ROWS: usize = 48;

/// Maximum number of columns (utf-8 char across) in largest supported TUI
/// of which 157 can be file text
const MAX_TUI_COLS: usize = 160;

/// Default terminal is 24 x 80
/// Default TUI text dimensions will be
/// +/- 3 header footer,
/// +/- at least 3 for line numbers
const DEFAULT_ROWS: usize = 24;
const DEFAULT_COLS: usize = 80;

const RESET: &str = "\x1b[0m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
// const GREEN: &str = "\x1b[32m";
// const BLUE: &str = "\x1b[34m";
// const BOLD: &str = "\x1b[1m";
// const ITALIC: &str = "\x1b[3m";
// const UNDERLINE: &str = "\x1b[4m";

/// Defensive programming limits to prevent infinite loops and resource exhaustion
/// Following NASA Power of 10 rules: all loops must have explicit upper bounds
mod limits {
    /// Maximum iterations for binary search through double-width character ranges
    /// Based on: log2(128) = 7, rounded up to 8 for safety margin
    pub const DOUBLE_WIDTH_BINARY_SEARCH: usize = 8;

    /// Maximum bytes to scan when seeking to a line number
    /// Prevents infinite loops on corrupted files or extremely large files
    /// 10 million bytes = ~10MB, reasonable for text files
    pub const FILE_SEEK_BYTES: usize = 10_000_000;

    /// Maximum lines to process when building window display
    /// Should match or exceed MAX_TUI_ROWS (45) with generous margin
    pub const WINDOW_BUILD_LINES: usize = 1000;

    /// Maximum bytes to read when processing a single line
    /// Matches the line buffer size
    pub const LINE_READ_BYTES: usize = 4096;

    /// Maximum iterations when skipping characters for horizontal offset
    /// Allows scrolling very far right in long lines
    pub const HORIZONTAL_SCROLL_CHARS: usize = 10_000;

    /// Maximum cursor movement iterations in a single command
    /// Allows "1000j" type commands while preventing integer overflow issues
    pub const CURSOR_MOVEMENT_STEPS: usize = 1_000_000;

    /// Maximum iterations in main editor loop
    /// Effectively unlimited (100k commands per session is very generous)
    pub const MAIN_EDITOR_LOOP_COMMANDS: usize = 100_000;

    /// Maximum bytes to scan when finding UTF-8 character boundaries
    /// UTF-8 characters are at most 4 bytes
    pub const MAX_UTF8_BOUNDARY_SCAN: usize = 4;

    /// Maximum iterations when parsing command input strings
    /// Allows up to 20-digit repeat counts (e.g., "12345678901234567890j")
    pub const COMMAND_PARSE_MAX_CHARS: usize = 20;
}

/// Creates a timestamp string specifically for archive file naming
///
/// # Purpose
/// Generates a consistent, sortable timestamp string for archive filenames
/// that works identically across all platforms (Windows, Linux, macOS).
///
/// # Arguments
/// * `time` - The SystemTime to format (typically SystemTime::now())
///
/// # Returns
/// * `String` - Timestamp in format: "YY_MM_DD_HH_MM_SS"
///
/// # Format Specification
/// - YY: Two-digit year (00-99)
/// - MM: Two-digit month (01-12)
/// - DD: Two-digit day (01-31)
/// - HH: Two-digit hour in 24-hour format (00-23)
/// - MM: Two-digit minute (00-59)
/// - SS: Two-digit second (00-59)
///
/// # Examples
/// - "24_01_15_14_30_45" for January 15, 2024 at 2:30:45 PM
/// - "23_12_31_23_59_59" for December 31, 2023 at 11:59:59 PM
///
/// # Platform Consistency
/// This function produces identical output on all platforms by using
/// epoch-based calculations rather than platform-specific date commands.
fn create_archive_timestamp(time: SystemTime) -> String {
    // Get duration since Unix epoch
    let duration_since_epoch = match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration,
        Err(_) => {
            // System time before Unix epoch - use fallback
            eprintln!("Warning: System time is before Unix epoch, using fallback timestamp");
            return String::from("70_01_01_00_00_00");
        }
    };

    let total_seconds = duration_since_epoch.as_secs();

    // Use the accurate date calculation
    let (year, month, day, hour, minute, second) =
        epoch_seconds_to_datetime_components(total_seconds);

    // Assertion 1: Validate year range
    const MAX_REASONABLE_YEAR: u32 = 9999;
    if year > MAX_REASONABLE_YEAR {
        eprintln!(
            "Warning: Year {} exceeds maximum reasonable value {}. Using fallback.",
            year, MAX_REASONABLE_YEAR
        );
        return String::from("99_12_31_23_59_59");
    }

    // Assertion 2: Validate all components are in expected ranges
    if month < 1 || month > 12 || day < 1 || day > 31 || hour > 23 || minute > 59 || second > 59 {
        eprintln!(
            "Warning: Invalid date/time components: {}-{:02}-{:02} {:02}:{:02}:{:02}",
            year, month, day, hour, minute, second
        );
        return String::from("70_01_01_00_00_00"); // Safe fallback
    }

    // Format as YY_MM_DD_HH_MM_SS
    format!(
        "{:02}_{:02}_{:02}_{:02}_{:02}_{:02}",
        year % 100, // Two-digit year
        month,
        day,
        hour,
        minute,
        second
    )
}

/// Converts Unix epoch seconds to accurate date/time components
///
/// # Purpose
/// Provides accurate date/time calculation that properly handles:
/// - Leap years (including century rules)
/// - Correct days per month
/// - Time zones (UTC)
///
/// # Arguments
/// * `epoch_seconds` - Seconds since Unix epoch (1970-01-01 00:00:00 UTC)
///
/// # Returns
/// * `(year, month, day, hour, minute, second)` - All as u32 values
///
/// # Algorithm
/// Uses proper calendar arithmetic to convert epoch seconds to date/time
/// components, accounting for leap years and varying month lengths.
fn epoch_seconds_to_datetime_components(epoch_seconds: u64) -> (u32, u32, u32, u32, u32, u32) {
    // Time component calculations
    const SECONDS_PER_MINUTE: u64 = 60;
    const SECONDS_PER_HOUR: u64 = 3600;
    const SECONDS_PER_DAY: u64 = 86400;

    // Calculate time of day components
    let seconds_today = epoch_seconds % SECONDS_PER_DAY;
    let hour = (seconds_today / SECONDS_PER_HOUR) as u32;
    let minute = ((seconds_today % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE) as u32;
    let second = (seconds_today % SECONDS_PER_MINUTE) as u32;

    // Calculate date components
    let days_since_epoch = epoch_seconds / SECONDS_PER_DAY;
    let (year, month, day) = days_to_ymd(days_since_epoch);

    (year, month, day, hour, minute, second)
}

/// Converts days since Unix epoch to year, month, day
///
/// # Purpose
/// Accurate calendar calculation that properly handles leap years
/// and correct month lengths.
///
/// # Arguments
/// * `days_since_epoch` - Days since 1970-01-01
///
/// # Returns
/// * `(year, month, day)` - Calendar date components
///
/// # Leap Year Rules
/// - Divisible by 4: leap year
/// - Divisible by 100: not a leap year
/// - Divisible by 400: leap year
///
/// # Safety Bounds
/// - Maximum year: 9999 (bounded loop with MAX_YEAR_ITERATIONS)
/// - If bounds exceeded, returns safe fallback date
fn days_to_ymd(days_since_epoch: u64) -> (u32, u32, u32) {
    // Constants for loop bounds and validation
    const EPOCH_YEAR: u32 = 1970;
    const MAX_YEAR: u32 = 9999;
    const MAX_YEAR_ITERATIONS: u32 = MAX_YEAR - EPOCH_YEAR; // 8029 iterations max

    // Start from 1970-01-01
    let mut year = EPOCH_YEAR;
    let mut remaining_days = days_since_epoch;

    // Helper function to check if a year is a leap year
    let is_leap_year = |y: u32| -> bool { (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0) };

    // BOUNDED LOOP - Subtract complete years with explicit upper limit
    let mut iteration_count = 0u32;
    while remaining_days > 0 && iteration_count < MAX_YEAR_ITERATIONS {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };

        if remaining_days >= days_in_year {
            remaining_days -= days_in_year;
            year += 1;
            iteration_count += 1;
        } else {
            break;
        }
    }

    // Assertion 1: Check if we hit the iteration limit (defensive programming)
    if iteration_count >= MAX_YEAR_ITERATIONS {
        eprintln!(
            "Warning: Year calculation exceeded maximum iterations ({}). Input may be corrupted.",
            MAX_YEAR_ITERATIONS
        );
        eprintln!(
            "Debug: days_since_epoch={}, remaining_days={}, year={}",
            days_since_epoch, remaining_days, year
        );
        // Return safe fallback date: 9999-12-31
        return (9999, 12, 31);
    }

    // Assertion 2: Year should be in reasonable range
    if year > MAX_YEAR {
        eprintln!(
            "Warning: Calculated year {} exceeds maximum {}",
            year, MAX_YEAR
        );
        return (9999, 12, 31);
    }

    // Days in each month for normal and leap years
    const DAYS_IN_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    const DAYS_IN_MONTH_LEAP: [u32; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    let days_in_months = if is_leap_year(year) {
        &DAYS_IN_MONTH_LEAP
    } else {
        &DAYS_IN_MONTH
    };

    // BOUNDED LOOP - Find the month and day (max 12 iterations)
    let mut month = 1u32;
    let mut days_left = remaining_days as u32;

    // Explicit bound: maximum 12 months
    for month_index in 0..12 {
        let days_in_month = days_in_months[month_index];

        if days_left >= days_in_month {
            days_left -= days_in_month;
            month += 1;
        } else {
            break;
        }
    }

    // Assertion 3: Month should be in valid range
    if month < 1 || month > 12 {
        eprintln!(
            "Warning: Calculated month {} is invalid. Defaulting to December.",
            month
        );
        month = 12;
    }

    // Day of month (1-based), add 1 because we want 1-31, not 0-30
    let day = days_left + 1;

    // Assertion 4: Day should be in valid range for the month
    let max_day_for_month = days_in_months[(month - 1) as usize];
    if day < 1 || day > max_day_for_month {
        eprintln!(
            "Warning: Calculated day {} is invalid for month {}. Using last valid day.",
            day, month
        );
        return (year, month, max_day_for_month);
    }

    (year, month, day)
}

/// Creates a timestamp with optional microsecond precision for uniqueness
///
/// # Purpose
/// When multiple archives might be created in the same second, this
/// adds microsecond precision to ensure unique filenames.
///
/// # Arguments
/// * `time` - The SystemTime to format
/// * `include_microseconds` - Whether to append microseconds
///
/// # Returns
/// * `String` - Timestamp, optionally with microseconds appended
///
/// # Format
/// - Without microseconds: "YY_MM_DD_HH_MM_SS"
/// - With microseconds: "YY_MM_DD_HH_MM_SS_UUUUUU"
pub fn createarchive_timestamp_with_precision(
    time: SystemTime,
    include_microseconds: bool,
) -> String {
    let base_timestamp = create_archive_timestamp(time);

    if !include_microseconds {
        return base_timestamp;
    }

    // Get microseconds component
    let duration_since_epoch = match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration,
        Err(_) => return base_timestamp, // Fall back to base timestamp
    };

    let microseconds = duration_since_epoch.as_micros() % 1_000_000;

    format!("{}_{:06}", base_timestamp, microseconds)
}

/*
 * Solution 2
 * The attempt is to follow NASA's only-preallocated-memory rule.
 */

use std::fmt;
// use std::io;

/// Fixed-size timestamp type - stack allocated, no heap
#[derive(Copy, Clone)]
pub struct FixedSize32Timestamp {
    data: [u8; 32],
    len: usize,
}

impl FixedSize32Timestamp {
    /// Creates a FixedSize32Timestamp from a string slice
    ///
    /// # Arguments
    /// * `s` - String slice to convert (max 31 bytes)
    ///
    /// # Returns
    /// * `Result<Self>` - FixedSize32Timestamp or LinesError
    ///
    /// # Errors
    /// Returns error if string exceeds 31 bytes
    pub fn from_str(s: &str) -> Result<Self> {
        const MAX_LEN: usize = 31;

        // Assertion 1: Check length
        if s.len() > MAX_LEN {
            return Err(LinesError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("String too long: {} bytes, max: {}", s.len(), MAX_LEN),
            )));
        }

        // Assertion 2: Verify valid UTF-8 (already guaranteed by &str type)
        let mut data = [0u8; 32];
        let bytes = s.as_bytes();

        // Bounded copy loop
        for i in 0..s.len().min(MAX_LEN) {
            data[i] = bytes[i];
        }

        Ok(FixedSize32Timestamp { data, len: s.len() })
    }

    /// Gets the timestamp as a string slice
    ///
    /// # Returns
    /// * `Result<&str>` - String slice view of timestamp
    ///
    /// # Errors
    /// Returns error if internal data is not valid UTF-8
    pub fn as_str(&self) -> Result<&str> {
        // Assertion: Internal invariant check
        assert!(
            self.len <= 32,
            "Internal invariant violated: length exceeds buffer size"
        );

        std::str::from_utf8(&self.data[..self.len]).map_err(|e| {
            LinesError::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid UTF-8 in FixedSize32Timestamp: {}", e),
            ))
        })
    }
}

impl fmt::Display for FixedSize32Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.as_str() {
            Ok(s) => write!(f, "{}", s),
            Err(e) => {
                // Log the error before returning placeholder
                eprintln!("Warning: Invalid UTF-8 in FixedSize32Timestamp: {}", e);
                write!(f, "[invalid UTF-8]")
            }
        }
    }
}

/// Splits a timestamp into two independent copies - NO HEAP, NO UNSAFE
///
/// # Purpose
/// Creates two stack-allocated copies of a timestamp string without any heap allocation.
/// This follows NASA's pre-allocated memory rule by using only fixed-size stack arrays.
///
/// # Arguments
/// * `input` - String slice containing timestamp (17-31 characters)
///
/// # Returns
/// * `Result<(FixedSize32Timestamp, FixedSize32Timestamp)>` - Two independent copies or error
///
/// # Memory Allocation
/// Uses only stack-allocated fixed arrays ([u8; 32]). No heap allocation.
/// Both copies are completely independent and stored on the stack.
///
/// # Constraints
/// - Minimum length: 17 characters (YY_MM_DD_HH_MM_SS)
/// - Maximum length: 31 characters (YY_MM_DD_HH_MM_SS_UUUUUU)
///
/// # Errors
/// Returns error if:
/// - Input is too short (< 17 chars)
/// - Input is too long (> 31 chars)
/// - Input contains invalid UTF-8
///
/// # Example
/// ```
/// let timestamp = "24_10_12_12_08_13_656800";
/// match split_timestamp_no_heap(timestamp) {
///     Ok((copy1, copy2)) => {
///         // copy1 and copy2 are independent stack copies
///         println!("Copy 1: {}", copy1);
///         println!("Copy 2: {}", copy2);
///     }
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn split_timestamp_no_heap(
    input: &str,
) -> Result<(FixedSize32Timestamp, FixedSize32Timestamp)> {
    // Assertion 1: Length check - minimum
    const MIN_LEN: usize = 17;
    const MAX_LEN: usize = 31;

    if input.len() < MIN_LEN {
        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Timestamp too short: {} chars, minimum required: {}",
                input.len(),
                MIN_LEN
            ),
        )));
    }

    // Assertion 2: Length check - maximum
    if input.len() > MAX_LEN {
        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Timestamp too long: {} chars, maximum allowed: {}",
                input.len(),
                MAX_LEN
            ),
        )));
    }

    // Create the timestamp (this validates and copies to stack)
    let timestamp = FixedSize32Timestamp::from_str(input)?;

    // These are true copies on the stack, no heap allocation
    // The Copy trait creates bitwise copies
    let copy1 = timestamp;
    let copy2 = timestamp;

    // Assertion 3: Verify copies maintain data integrity
    assert_eq!(
        copy1.len, timestamp.len,
        "Copy 1 length mismatch: expected {}, got {}",
        timestamp.len, copy1.len
    );
    assert_eq!(
        copy2.len, timestamp.len,
        "Copy 2 length mismatch: expected {}, got {}",
        timestamp.len, copy2.len
    );

    // Assertion 4: Verify both copies have identical content
    // Note: In safe Rust, we cannot verify different stack addresses,
    // but we can verify the content is identical
    match (copy1.as_str(), copy2.as_str()) {
        (Ok(s1), Ok(s2)) => {
            assert_eq!(s1, s2, "Copies should have identical content");
        }
        (Err(e), _) | (_, Err(e)) => {
            return Err(LinesError::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Copy validation failed: {}", e),
            )));
        }
    }

    Ok((copy1, copy2))
}

/// Formats the navigation legend with color-coded keyboard shortcuts
///
/// # Purpose
/// Creates a formatted legend string showing all available keyboard commands
/// with color highlighting (RED for command keys, YELLOW for descriptions).
///
/// # Returns
/// * `Ok(String)` - The formatted legend string with ANSI color codes
/// * `Err(FileFantasticError)` - If string formatting fails (defensive programming)
///
/// # Color Scheme
/// - RED: Single letter command keys (q, b, t, d, f, n, s, m, g, v, y, p)
/// - YELLOW: Command descriptions and separators
/// - RESET: Applied at end to restore terminal defaults
///
/// # Legend Commands
/// - q: quit the application
/// - b: navigate back/parent directory
/// - t: open terminal in current directory
/// - d: filter to show directories only
/// - f: filter to show files only
/// - n: sort by name
/// - s: sort by size
/// - m: sort by modified date
/// - g: get-send file operations
/// - v,y,p: additional file operations
/// - str: search functionality
/// - enter: reset filters/search
///
/// # Example
/// ```rust
/// match format_navigation_legend() {
///     Ok(legend) => println!("{}", legend),
///     Err(e) => eprintln!("Failed to format legend: {}", e),
/// }
/// ```
fn format_navigation_legend() -> Result<String> {
    // Pre-allocate string capacity based on expected legend size
    // Legend is approximately 200 characters plus color codes
    let mut legend = String::with_capacity(300);

    // Build the legend string with error handling for format operations
    // quit save undo norm ins vis del wrap relative raw byt wrd,b,end /commnt hjkl
    let formatted = format!(
        "{}{}q{}uit {}s{}ave {}u{}ndo {}d{}el|{}n{}orm {}i{}ns {}v{}is|{}wrap{} {}raw{} {}r{}lativ {}b{}yte|{}w{}rd,{}b{},{}e{}nd {}/{}cmmnt {}[]{}rpt {}hjkl{}{}",
        YELLOW, // Overall legend color
        RED,
        YELLOW, // RED q + YELLOW uit
        RED,
        YELLOW, // RED b + YELLOW ack
        RED,
        YELLOW, // RED t + YELLOW erm
        RED,
        YELLOW, // RED d + YELLOW ir
        RED,
        YELLOW, // RED f + YELLOW ile
        RED,
        YELLOW, // RED n + YELLOW ame
        RED,
        YELLOW, // RED s + YELLOW ize
        RED,
        YELLOW, // RED m + YELLOW od
        RED,
        YELLOW, // RED g + YELLOW et
        RED,
        YELLOW, // RED v + YELLOW ,
        RED,
        YELLOW, // RED y + YELLOW ,
        RED,
        YELLOW, // RED p + YELLOW ,
        RED,
        YELLOW, // RED str + YELLOW ...
        RED,
        YELLOW, // RED enter + YELLOW ...
        RED,
        YELLOW, // RED enter + YELLOW ...
        RED,
        YELLOW, // RED enter + YELLOW ...
        RED,
        YELLOW, // RED enter + YELLOW ...
        RESET
    );

    // Check if the formatted string is reasonable
    // (defensive programming against format! macro issues)
    if formatted.is_empty() {
        return Err(LinesError::FormatError(String::from(
            "Legend formatting produced empty string",
        )));
    }

    legend.push_str(&formatted);
    Ok(legend)
}

// ============================================================================
// ERROR HANDLING SYSTEM
// ============================================================================

/// Error types for the Lines text editor
///
/// # Design Principles
/// - Simple enum covering main error categories
/// - Wraps std::io::Error for file operations
/// - Provides context-specific error messages
/// - Supports conversion from common error types
#[derive(Debug)]
pub enum LinesError {
    /// File system or I/O operation failed
    Io(io::Error),

    /// Invalid user input or argument
    InvalidInput(String),

    /// String formatting or display error
    FormatError(String),

    /// UTF-8 encoding/decoding error
    Utf8Error(String),

    /// Terminal or display rendering error
    DisplayError(String),

    /// Configuration or state error
    StateError(String),
}

impl std::fmt::Display for LinesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinesError::Io(e) => write!(f, "IO error: {}", e),
            LinesError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            LinesError::FormatError(msg) => write!(f, "Format error: {}", msg),
            LinesError::Utf8Error(msg) => write!(f, "UTF-8 error: {}", msg),
            LinesError::DisplayError(msg) => write!(f, "Display error: {}", msg),
            LinesError::StateError(msg) => write!(f, "State error: {}", msg),
        }
    }
}

impl std::error::Error for LinesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LinesError::Io(e) => Some(e),
            _ => None,
        }
    }
}

/// Automatic conversion from io::Error to LinesError
impl From<io::Error> for LinesError {
    fn from(err: io::Error) -> Self {
        LinesError::Io(err)
    }
}

/// Result type alias for Lines editor operations
pub type Result<T> = std::result::Result<T, LinesError>;

/// Appends an error message to the error log file
///
/// # Purpose
/// Provides fail-safe error logging that never interrupts normal operation.
/// Errors are logged to `~/Documents/lines_editor/lines_data/error_logs/yyyy_mm_dd.log`
///
/// # Arguments
/// * `error_msg` - The error message to log
/// * `context` - Optional context string (e.g., function name, operation)
///
/// # Behavior
/// - Creates log directory if it doesn't exist
/// - Appends to daily log file with timestamp
/// - If logging fails, prints to stderr but doesn't return error
/// - Never interrupts normal program flow
pub fn log_error(error_msg: &str, context: Option<&str>) {
    // Build error log path - if this fails, just print to stderr
    let log_path = match get_error_log_path() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("WARNING: Cannot determine error log path: {}", e);
            eprintln!("ERROR: {}", error_msg);
            if let Some(ctx) = context {
                eprintln!("CONTEXT: {}", ctx);
            }
            return;
        }
    };

    // Ensure parent directory exists
    if let Some(parent) = log_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("WARNING: Cannot create error log directory: {}", e);
            eprintln!("ERROR: {}", error_msg);
            return;
        }
    }

    // Get current timestamp
    let timestamp = match get_timestamp() {
        Ok(ts) => ts,
        Err(_) => String::from("UNKNOWN_TIME"),
    };

    // Build log entry
    let log_entry = if let Some(ctx) = context {
        format!("[{}] [{}] {}\n", timestamp, ctx, error_msg)
    } else {
        format!("[{}] {}\n", timestamp, error_msg)
    };

    // Attempt to write to log file
    match OpenOptions::new().create(true).append(true).open(&log_path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(log_entry.as_bytes()) {
                eprintln!("WARNING: Cannot write to error log: {}", e);
                eprintln!("ERROR: {}", error_msg);
            }
            // Explicitly ignore flush errors - we tried our best
            let _ = file.flush();
        }
        Err(e) => {
            eprintln!("WARNING: Cannot open error log: {}", e);
            eprintln!("ERROR: {}", error_msg);
        }
    }
}

/// Gets the path to today's error log file
fn get_error_log_path() -> io::Result<PathBuf> {
    let home = get_home_directory()?;
    let timestamp = get_timestamp()?;

    let mut log_path = home;
    log_path.push("Documents");
    log_path.push("lines_editor");
    log_path.push("lines_data");
    log_path.push("error_logs");
    log_path.push(format!("{}.log", timestamp));

    Ok(log_path)
}

/// Makes, verifies, or creates a directory path relative to the executable directory location.
///
/// This function performs the following sequential steps:
/// 1. Converts the provided directory path string to an absolute path relative to the executable directory
/// 2. Checks if the directory exists at the calculated absolute path location
/// 3. If the directory does not exist, creates it and all necessary parent directories
/// 4. Returns the canonicalized (absolute path with all symlinks resolved) path to the directory
///
/// # Arguments
///
/// * `dir_path_string` - A string representing the directory path relative to the executable directory
///
/// # Returns
///
/// * `Result<PathBuf, std::io::Error>` - The canonicalized absolute path to the directory if successful,
///   or an error if any step fails (executable path determination, directory creation, or canonicalization)
///
/// # Errors
///
/// This function may return an error in the following situations:
/// - If the executable's directory cannot be determined
/// - If directory creation fails due to permissions or other I/O errors
/// - If path canonicalization fails
///
/// use example:
/// // Ensure the project graph data directory exists relative to the executable
/// let project_graph_directory_result = make_verify_or_create_executabledirectoryrelative_canonicalized_dir_path("project_graph_data");

/// // Handle any errors that might occur during directory creation or verification
/// let project_graph_directory = match project_graph_directory_result {
///     Ok(directory_path) => directory_path,
///     Err(io_error) => {
///         // Log the error and handle appropriately for your application
///         return Err(format!("Failed to ensure project graph directory exists: {}", io_error).into());
///     }
/// };
///
pub fn make_verify_or_create_executabledirectoryrelative_canonicalized_dir_path(
    dir_path_string: &str,
) -> Result<PathBuf> {
    // Step 1: Convert the provided directory path to an absolute path relative to the executable
    let absolute_dir_path =
        make_input_path_name_abs_executabledirectoryrelative_nocheck(dir_path_string)?;

    // Step 2: Check if the directory exists at the calculated absolute path
    let directory_exists = abs_executable_directory_relative_exists(&absolute_dir_path)?;

    if !directory_exists {
        // Step 3: Directory doesn't exist, create it and all parent directories
        // Note: mkdir_new_abs_executabledirectoryrelative_canonicalized will also canonicalize the path
        mkdir_new_abs_executabledirectoryrelative_canonicalized(dir_path_string)
    } else {
        absolute_dir_path
            .canonicalize()
            .map_err(|canonicalization_error| {
                LinesError::Io(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "Failed to canonicalize existing directory path: {}",
                        canonicalization_error
                    ),
                ))
            })
    }
}

/// Creates a new directory at the specified path relative to the executable directory.
/// Returns an error if the directory already exists.
///
/// # Arguments
///
/// * `dir_path` - The directory path relative to the executable directory
///
/// # Returns
///
/// * `Result<PathBuf, io::Error>` - The absolute, canonicalized path to the newly created directory
pub fn mkdir_new_abs_executabledirectoryrelative_canonicalized<P: AsRef<Path>>(
    dir_path: P,
) -> Result<PathBuf> {
    // Get the absolute path without checking existence
    let abs_path = make_input_path_name_abs_executabledirectoryrelative_nocheck(dir_path)?;

    // Check if the directory already exists
    if abs_executable_directory_relative_exists(&abs_path)? {
        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Directory already exists",
        )));
    }

    // Create the directory and all parent directories
    std::fs::create_dir_all(&abs_path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to create directory: {}", e),
        )
    })?;

    // Canonicalize the path (should succeed because we just created it)
    abs_path.canonicalize().map_err(|e| {
        LinesError::Io(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to canonicalize newly created directory path: {}", e),
        ))
    })
}

/// Checks if a path exists (either as a file or directory).
///
/// # Arguments
///
/// * `path_to_check` - The path to check for existence
///
/// # Returns
///
/// * `Result<bool, io::Error>` - Whether the path exists or an error
pub fn abs_executable_directory_relative_exists<P: AsRef<Path>>(path_to_check: P) -> Result<bool> {
    let path = path_to_check.as_ref();
    Ok(path.exists())
}

/// Converts a path to an absolute path based on the executable's directory location.
/// Does NOT check if the path exists or attempt to create anything.
///
/// # Arguments
///
/// * `path_to_make_absolute` - A path to convert to an absolute path relative to
///   the executable's directory location.
///
/// # Returns
///
/// * `Result<PathBuf, io::Error>` - The absolute path based on the executable's directory or an error
///   if the executable's path cannot be determined or if the path cannot be resolved.
///
/// # Examples
///
/// ```
/// use manage_absolute_executable_directory_relative_paths::make_input_path_name_abs_executabledirectoryrelative_nocheck;
///
/// // Get an absolute path for "data/config.json" relative to the executable directory
/// let abs_path = make_input_path_name_abs_executabledirectoryrelative_nocheck("data/config.json").unwrap();
/// println!("Absolute path: {}", abs_path.display());
/// ```
pub fn make_input_path_name_abs_executabledirectoryrelative_nocheck<P: AsRef<Path>>(
    path_to_make_absolute: P,
) -> Result<PathBuf> {
    // Get the directory where the executable is located
    let executable_directory = get_absolute_path_to_executable_parentdirectory()?;

    // Create a path by joining the executable directory with the provided path
    let target_path = executable_directory.join(path_to_make_absolute);

    // If the path doesn't exist, we still return the absolute path without trying to canonicalize
    if !abs_executable_directory_relative_exists(&target_path)? {
        // Ensure the path is absolute (it should be since we joined with executable_directory)
        if target_path.is_absolute() {
            return Ok(target_path);
        } else {
            // Wrap io::Error in LinesError::Io
            return Err(LinesError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Failed to create absolute path",
            )));
        }
    }

    // Path exists, so we can canonicalize it to resolve any ".." or "." segments
    target_path.canonicalize().map_err(|e| {
        // Wrap in LinesError::Io
        LinesError::Io(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to canonicalize path: {}", e),
        ))
    })
}

/// Gets the directory where the current executable is located.
///
/// # Returns
///
/// * `Result<PathBuf, io::Error>` - The absolute directory path containing the executable or an error
///   if it cannot be determined.
pub fn get_absolute_path_to_executable_parentdirectory() -> Result<PathBuf> {
    // Get the path to the current executable
    let executable_path = std::env::current_exe().map_err(|e| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Failed to determine current executable path: {}", e),
        )
    })?;

    // Get the directory containing the executable
    let executable_directory = executable_path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Failed to determine parent directory of executable",
        )
    })?;

    Ok(executable_directory.to_path_buf())
}

/// Represents a position in the file (not in the window)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilePosition {
    /// Byte offset from start of file
    pub byte_offset: u64,
    /// Line number (0-indexed)
    pub line_number: usize,
    /// Byte offset within the line
    pub byte_in_line: usize,
}

/// Represents a position in the terminal window
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowPosition {
    /// Row in terminal (0-indexed, 0-95 max)
    pub row: usize,
    /// Column in terminal (0-indexed, 0-319 max)
    pub col: usize,
}

/// Maps window positions to file positions
pub struct WindowMap {
    /// Pre-allocated mapping array [row][col] -> Option<FilePosition>
    /// None means this position is empty/padding
    positions: [[Option<FilePosition>; MAX_TUI_COLS]; MAX_TUI_ROWS],
    /// Number of valid rows in current window
    valid_rows: usize,
    /// Number of valid columns in current window
    valid_cols: usize,
}

impl WindowMap {
    /// Creates a new WindowMap with all positions set to None
    pub fn new() -> Self {
        WindowMap {
            positions: [[None; MAX_TUI_COLS]; MAX_TUI_ROWS],
            valid_rows: DEFAULT_ROWS,
            valid_cols: DEFAULT_COLS,
        }
    }

    /// Gets the file position for a window position
    ///
    /// # Arguments
    /// * `row` - Terminal row (0-indexed)
    /// * `col` - Terminal column (0-indexed)
    ///
    /// # Returns
    /// * `Ok(Option<FilePosition>)` - File position if valid, None if empty
    /// * `Err(io::Error)` - If row/col out of bounds
    pub fn get_file_position(&self, row: usize, col: usize) -> io::Result<Option<FilePosition>> {
        // Defensive: Check bounds
        if row >= self.valid_rows {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Row {} exceeds valid rows {}", row, self.valid_rows),
            ));
        }
        if col >= self.valid_cols {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Column {} exceeds valid columns {}", col, self.valid_cols),
            ));
        }

        Ok(self.positions[row][col])
    }

    /// Sets the file position for a window position
    ///
    /// # Arguments
    /// * `row` - Terminal row (0-indexed)
    /// * `col` - Terminal column (0-indexed)
    /// * `file_pos` - File position to map to (None for empty)
    ///
    /// # Returns
    /// * `Ok(())` - Successfully set
    /// * `Err(io::Error)` - If row/col out of bounds
    pub fn set_file_position(
        &mut self,
        row: usize,
        col: usize,
        file_pos: Option<FilePosition>,
    ) -> io::Result<()> {
        // Defensive: Check bounds
        if row >= MAX_TUI_ROWS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Row {} exceeds maximum {}", row, MAX_TUI_ROWS),
            ));
        }
        if col >= MAX_TUI_COLS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Column {} exceeds maximum {}", col, MAX_TUI_COLS),
            ));
        }

        self.positions[row][col] = file_pos;
        Ok(())
    }

    /// Clears all mappings
    pub fn clear(&mut self) {
        // Defensive: explicit loop with bounds
        for row in 0..MAX_TUI_ROWS {
            for col in 0..MAX_TUI_COLS {
                self.positions[row][col] = None;
            }
        }
    }
}

/// Current editor mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorMode {
    /// Normal/command mode (like vim normal mode)
    Normal,
    /// Insert mode for typing
    Insert,
    /// Visual selection mode
    Visual,
    /// Multi-cursor mode (ctrl+d equivalent)
    MultiCursor,
}

/// Line wrap mode setting
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WrapMode {
    /// Lines wrap at terminal width
    Wrap,
    /// Lines extend beyond terminal width (horizontal scroll)
    NoWrap,
}

/// Two-Purpose Buffer (alternate plan is )
/// A. processes command-input that does not go to file
/// B. processes chunks to stdin to write to files (read-copy, change-log)
///
/// Maximum size for insert mode input buffer (512 or 256 bytes)
/// Allows ~512 ASCII chars or ~170 3-byte UTF-8 chars per insert
///
/// This is for a chunked process of moving std-in
/// text from the user to
/// A: file
/// B: change-log
/// without whole-loading things into buffers.
/// Towers of Hanoy
const TOFILE_INSERTBUFFER_CHUNK_SIZE: usize = 256;

/// Main editor state structure with all pre-allocated buffers
pub struct EditorState {
    /// It's...The Last Command!
    /// or the most recent past command
    /// that the user asked for / was executed
    /// (so you can do stuff again easily with
    /// an empty enter on norma/visual mode)
    /// "When you got nothing, then your on...
    /// The Last Command!"
    /// None if no command has been executed yet
    ///
    pub the_last_command: Option<Command>,

    ///where lines files for this session are stored
    pub session_directory_path: Option<PathBuf>,

    /// Current editor mode
    pub mode: EditorMode,

    /// Line wrap setting
    pub wrap_mode: WrapMode,

    /// Absolute path to the file being edited
    pub original_file_path: Option<PathBuf>,

    /// Absolute path to read-copy of file
    pub read_copy_path: Option<PathBuf>,

    /// Terminal dimensions
    pub terminal_rows: usize,
    pub terminal_cols: usize,

    /// Effective editing area (minus headers/footers/line numbers)
    pub effective_rows: usize,
    pub effective_cols: usize,

    /// Current window buffer containing visible text
    /// Pre-allocated to FILE_TUI_WINDOW_MAP_BUFFER_SIZE
    pub state_file_tui_window_map_buffer: [u8; FILE_TUI_WINDOW_MAP_BUFFER_SIZE],

    /// Number of valid bytes in state_file_tui_window_map_buffer
    pub filetui_windowmap_buffer_used: usize,

    /// Window to file position mapping
    pub window_map: WindowMap,

    /// Cursor position in window
    pub cursor: WindowPosition,

    /// File position of top-left corner of window
    pub window_start: FilePosition,

    /// Visual mode selection start (if in visual mode)
    pub selection_start: Option<FilePosition>,

    /// Path to .changelog file
    pub changelog_path: Option<PathBuf>,

    /// Flag indicating if file has unsaved changes
    pub is_modified: bool,

    // === WINDOW POSITION TRACKING ===
    /// Line number of file that appears at top of terminal window
    /// Example: If window shows from line 500, this is 500
    pub line_count_at_top_of_window: usize,

    /// Byte position in file where the top display line starts
    /// Example: Line 500 starts at byte 12048 in the file
    pub file_position_of_topline_start: u64,

    /// For LineWrap mode: byte offset within the line where window starts
    /// Example: If line 500 has 300 chars and we're showing the 3rd wrap, this might be 211
    pub linewrap_window_topline_startbyte_position: u64,

    /// For LineWrap mode: character offset within the line where window starts
    /// Example: Starting at character 70 of line 500
    pub linewrap_window_topline_char_offset: usize,

    /// For NoWrap mode: horizontal character offset for all displayed lines
    /// Example: Showing characters 20-97 of each line
    pub horizontal_line_char_offset: usize,

    // === DISPLAY BUFFERS ===
    /// Pre-allocated buffers for each display row (45 rows × 80 chars)
    /// Each buffer holds one terminal row including line number and text
    pub display_buffers: [[u8; 182]; 45],

    /// Actual bytes used in each display buffer
    /// Since lines can be shorter than 80 chars, we track actual usage
    pub display_buffer_lengths: [usize; 45],

    /// TODO: Should there be a clear-buffer method?
    /// Pre-allocated buffer for insert mode text input
    /// Used to capture user input before inserting into file
    pub tofile_insert_input_chunk_buffer: [u8; TOFILE_INSERTBUFFER_CHUNK_SIZE],

    /// TODO is this needed?
    /// Number of valid bytes in tofile_insert_input_chunk_buffer
    pub tofile_insertinput_chunkbuffer_used: usize,
}

impl EditorState {
    /// Creates a new EditorState with all memory pre-allocated
    ///
    /// # Returns
    /// * `EditorState` - Initialized state with default values
    pub fn new() -> Self {
        // Calculate effective area (3 cols for line numbers, 3 rows for header/footer)
        let effective_cols = DEFAULT_COLS.saturating_sub(3);
        let effective_rows = DEFAULT_ROWS.saturating_sub(3);

        EditorState {
            the_last_command: None,
            session_directory_path: None,
            mode: EditorMode::Normal,
            wrap_mode: WrapMode::Wrap,
            original_file_path: None,
            read_copy_path: None,
            terminal_rows: DEFAULT_ROWS,
            terminal_cols: DEFAULT_COLS,
            effective_rows,
            effective_cols,
            state_file_tui_window_map_buffer: [0u8; FILE_TUI_WINDOW_MAP_BUFFER_SIZE],
            filetui_windowmap_buffer_used: 0,
            window_map: WindowMap::new(),
            cursor: WindowPosition { row: 0, col: 0 },
            window_start: FilePosition {
                byte_offset: 0,
                line_number: 0,
                byte_in_line: 0,
            },
            selection_start: None,
            changelog_path: None,
            is_modified: false,

            // === NEW FIELD INITIALIZATION ===
            // Window position tracking - start at beginning of file
            line_count_at_top_of_window: 0,
            file_position_of_topline_start: 0,
            linewrap_window_topline_startbyte_position: 0,
            linewrap_window_topline_char_offset: 0,
            horizontal_line_char_offset: 0,

            // Display buffers - initialized to zero
            display_buffers: [[0u8; 182]; 45],
            display_buffer_lengths: [0usize; 45],

            tofile_insert_input_chunk_buffer: [0u8; TOFILE_INSERTBUFFER_CHUNK_SIZE],
            tofile_insertinput_chunkbuffer_used: 0,
        }
    }

    /*
    // When opening a file:
    state.init_changelog(&original_file_path)?;

    // When user types/edits:
    state.log_edit(&format!("INSERT {} {}: {}", line_num, byte_pos, text))?;

    // When user deletes:
    state.log_edit(&format!("DELETE {} {}: {}", line_num, byte_pos, deleted_text))?;
     */

    /// Clears all display buffers and resets their lengths
    ///
    /// # Purpose
    /// Called before rebuilding window content to ensure clean slate
    /// Defensive programming: explicitly zeros all buffers
    pub fn clear_display_buffers(&mut self) {
        // Defensive: Clear each buffer completely
        for row_idx in 0..45 {
            for col_idx in 0..80 {
                self.display_buffers[row_idx][col_idx] = 0;
            }
            self.display_buffer_lengths[row_idx] = 0;
        }
    }

    /// Initialize changelog for the current file
    pub fn init_changelog(&mut self, original_file_path: &Path) -> io::Result<()> {
        // Put changelog next to the file: "document.txt.changelog"
        self.changelog_path = Some(original_file_path.with_extension("txt.changelog"));
        Ok(())
    }

    /// Append an edit operation to the changelog
    pub fn log_edit(&self, operation: &str) -> io::Result<()> {
        if let Some(ref log_path) = self.changelog_path {
            use std::fs::OpenOptions;
            use std::io::Write;

            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)?;

            writeln!(file, "{}", operation)?;
        }
        Ok(())
    }

    /// Writes a line number into a display buffer
    ///
    /// # Format
    /// Just the line number and ONE space. No padding, no alignment.
    /// - Line 1: "1 " (2 bytes)
    /// - Line 42: "42 " (3 bytes)
    /// - Line 999: "999 " (4 bytes)
    pub fn write_line_number(&mut self, row_idx: usize, line_num: usize) -> io::Result<usize> {
        // Defensive: Validate row index
        if row_idx >= 45 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Row index {} exceeds maximum 44", row_idx),
            ));
        }

        // Format: just the number and one space, NO PADDING
        let line_str = format!("{} ", line_num);
        let line_bytes = line_str.as_bytes();

        // Copy to buffer
        let bytes_to_write = line_bytes.len().min(182); // Safety check against buffer size
        self.display_buffers[row_idx][..bytes_to_write]
            .copy_from_slice(&line_bytes[..bytes_to_write]);

        Ok(bytes_to_write)
    }

    /// Updates terminal dimensions and recalculates effective area
    ///
    /// # Arguments
    /// * `rows` - New terminal row count
    /// * `cols` - New terminal column count
    ///
    /// # Returns
    /// * `Ok(())` - Successfully updated
    /// * `Err(io::Error)` - If dimensions exceed maximums
    pub fn resize_terminal(&mut self, rows: usize, cols: usize) -> io::Result<()> {
        // Defensive: Validate dimensions
        if rows > MAX_TUI_ROWS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Rows {} exceeds maximum {}", rows, MAX_TUI_ROWS),
            ));
        }
        if cols > MAX_TUI_COLS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Columns {} exceeds maximum {}", cols, MAX_TUI_COLS),
            ));
        }

        self.terminal_rows = rows;
        self.terminal_cols = cols;

        // Recalculate effective area (reserve space for UI elements)
        self.effective_rows = rows.saturating_sub(3);
        self.effective_cols = cols.saturating_sub(3);

        // Update window map dimensions
        self.window_map.valid_rows = self.effective_rows;
        self.window_map.valid_cols = self.effective_cols;

        Ok(())
    }

    /// Clears the window buffer and map
    pub fn clear_window(&mut self) {
        // Clear buffer
        for i in 0..FILE_TUI_WINDOW_MAP_BUFFER_SIZE {
            self.state_file_tui_window_map_buffer[i] = 0;
        }
        self.filetui_windowmap_buffer_used = 0;

        // Clear map
        self.window_map.clear();
    }
}

/// Gets a timestamp string in yyyy_mm_dd format using only standard library
fn get_timestamp() -> io::Result<String> {
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let secs = time.as_secs();
    let days_since_epoch = secs / (24 * 60 * 60);

    // These arrays help us handle different month lengths
    let days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    let mut year = 1970;
    let mut remaining_days = days_since_epoch;

    // Calculate year
    loop {
        let year_length = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < year_length {
            break;
        }
        remaining_days -= year_length;
        year += 1;
    }

    // Calculate month and day
    let mut month = 1;
    for (month_idx, &days) in days_in_month.iter().enumerate() {
        let month_length = if month_idx == 1 && is_leap_year(year) {
            29
        } else {
            days
        };

        if remaining_days < month_length {
            break;
        }
        remaining_days -= month_length;
        month += 1;
    }

    let day = remaining_days + 1;

    Ok(format!("{:04}_{:02}_{:02}", year, month, day))
}

/// Helper function to determine if a year is a leap year
/// Determines if a given year is a leap year using the Gregorian calendar rules
///
/// # Arguments
/// * `year` - Year to check (CE/AD)
///
/// # Returns
/// * `bool` - true if leap year, false if not
///
/// # Rules
/// - Year is leap year if divisible by 4
/// - Exception: century years must be divisible by 400
/// - Years divisible by 100 but not 400 are not leap years
fn is_leap_year(year: u64) -> bool {
    if year % 4 != 0 {
        false
    } else if year % 100 != 0 {
        true
    } else if year % 400 != 0 {
        false
    } else {
        true
    }
}

/// Displays the last n lines of a file to standard output
/// Returns an IO Result to properly handle potential file reading errors
///
/// # Arguments
/// * `original_file_path` - Path to the file to display
/// * `num_lines` - Number of lines to show from end of file
///
/// # Returns
/// * `io::Result<()>` - Success or error status of the display operation
///
/// # Errors
/// Returns error if:
/// - File cannot be opened
/// - File cannot be read
/// - File content cannot be parsed as valid UTF-8
fn memo_mode_display_file_tail(original_file_path: &Path, num_lines: usize) -> io::Result<()> {
    /*
     * TODO: this maybe needs to be revised to not use heap dynamic memory
     */
    let file = File::open(original_file_path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<io::Result<_>>()?;

    let start = if lines.len() > num_lines {
        lines.len() - num_lines
    } else {
        0
    };

    for line in &lines[start..] {
        println!("{}", line);
    }
    Ok(())
}

/// Gets the header string for a new file
/// Combines timestamp with optional header.txt content
///
/// # Returns
/// - `Ok(String)` - Header string containing timestamp and optional header.txt content
/// - `Err(io::Error)` - If there's an error reading header.txt (if it exists)
/// Gets the header string for a new file          // <-- Duplicated line
/// Combines timestamp with optional header.txt content  // <-- Duplicated line
fn memo_mode_get_header_text() -> io::Result<String> {
    /*
     * TODO: this maybe needs to be revised to not use heap dynamic memory
     */
    let timestamp = get_timestamp()?;
    let mut header = format!("# {}", timestamp);

    // Get the executable's directory
    let exe_path = env::current_exe()?;
    let exe_dir = exe_path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine executable directory",
        )
    })?;

    // Check for header.txt in the executable's directory
    let header_path = exe_dir.join("header.txt");

    // Also check in the current working directory as fallback
    let current_dir_header = Path::new("header.txt");

    if header_path.exists() {
        let header_content = fs::read_to_string(header_path)?;
        header.push_str("  ");
        header.push_str(&header_content);
    } else if current_dir_header.exists() {
        let header_content = fs::read_to_string(current_dir_header)?;
        header.push_str("  ");
        header.push_str(&header_content);
    }

    Ok(header)
}

/// Appends a single line to the file with temporary backup protection
///
/// # Arguments
/// * `original_file_path` - Path to the file being appended to
/// * `line` - Text line to append
///
/// # Behavior
/// 1. Creates temporary backup
/// 2. Appends the line
/// 3. Removes backup if successful
/// 4. Restores from backup if append fails
fn memo_mode_append_line(original_file_path: &Path, line: &str) -> io::Result<()> {
    /*
     * TODO: this maybe needs to be revised to not use heap dynamic memory
     */
    // Create temporary backup before modification
    let backup_path = if original_file_path.exists() {
        let bak_path = original_file_path.with_extension("bak");
        fs::copy(original_file_path, &bak_path)?;
        Some(bak_path)
    } else {
        None
    };

    // Attempt to append the line
    let result = OpenOptions::new()
        .create(true)
        .append(true)
        .open(original_file_path)
        .and_then(|mut file| writeln!(file, "{}", line));

    // Handle the result
    match result {
        Ok(_) => {
            // Success: remove backup if it exists
            if let Some(bak_path) = backup_path {
                fs::remove_file(bak_path)?;
            }
            Ok(())
        }
        Err(e) => {
            // Failure: restore from backup if it exists
            if let Some(bak_path) = backup_path {
                fs::copy(&bak_path, original_file_path)?;
                fs::remove_file(bak_path)?;
            }
            Err(e)
        }
    }
}

/// Main editing loop for the lines text editor
///
/// # Arguments
/// * `original_file_path` - Path to the file being edited
///
/// # Returns
/// * `io::Result<()>` - Success or error status of the editing session
///
/// # Behavior
/// 1. Displays file path and basic commands
/// 2. If file doesn't exist, creates it with timestamp header
/// 3. Shows last 10 lines of current file content
/// 4. Enters input loop where user can:
///    - Type text and press enter to append a line
///    - Enter 'q', 'quit', or 'exit' to close editor
/// 5. After each append, displays updated last 10 lines
///
/// # Errors
/// Returns error if:
/// - Cannot create/access the file
/// - Cannot read user input
/// - Cannot append to file
/// - Cannot display file contents
///
/// # Example
/// ```no_run
/// let path = Path::new("notes.txt");
/// memo_mode_mini_editor_loop(&path)?;
/// ```
fn memo_mode_mini_editor_loop(original_file_path: &Path) -> io::Result<()> {
    /*
     * TODO: this maybe needs to be revised to not use heap dynamic memory
     */
    print!("\x1B[2J\x1B[1;1H");
    println!("lines text editor: Type 'q' to (q)uit");
    println!("file path -> {}", original_file_path.display());

    let stdin = io::stdin();
    let mut input = String::new();

    // Create file with header if it doesn't exist
    if !original_file_path.exists() {
        let header = memo_mode_get_header_text()?;
        memo_mode_append_line(original_file_path, &header)?;
        memo_mode_append_line(original_file_path, "")?; // blank line after header
    }

    // Display initial tail of file
    println!("Current file (last 10 lines) ->\n");
    if let Err(e) = memo_mode_display_file_tail(original_file_path, 10) {
        eprintln!("Error displaying file: {}", e);
    }

    loop {
        input.clear();
        print!("\n> "); // Add a prompt
        io::stdout().flush()?; // Ensure prompt is displayed

        if let Err(e) = stdin.read_line(&mut input) {
            eprintln!("Error reading input: {}", e);
            continue;
        }

        let trimmed = input.trim();

        if trimmed == "q" || trimmed == "quit" || trimmed == "exit" || trimmed == "exit()" {
            println!("Exiting editor...");
            break;
        }

        // Clear screen
        print!("\x1B[2J\x1B[1;1H");

        // Append the line with temporary backup protection
        if let Err(e) = memo_mode_append_line(original_file_path, trimmed) {
            eprintln!("Error writing to file: {}", e);
            continue;
        }

        // Display the tail of the file after append
        println!("\nLast 10 lines of file ->");
        if let Err(e) = memo_mode_display_file_tail(original_file_path, 10) {
            eprintln!("Error displaying file: {}", e);
        }
    }

    Ok(())
}

/// Gets or creates the default file path for the line editor.
/// If a custom filename is provided, appends the date to it.
///
/// # Arguments
/// * `custom_name` - Optional custom filename to use as prefix
///
/// # Returns
/// - For default: `{home}/Documents/lines_editor/yyyy_mm_dd.txt`
/// - For custom: `{home}/Documents/lines_editor/custom_name_yyyy_mm_dd.txt`
fn get_default_filepath(custom_name: Option<&str>) -> io::Result<PathBuf> {
    // Try to get home directory from environment variables
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map_err(|e| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Could not find home directory: {}", e),
            )
        })?;

    // Build the base directory path
    let mut base_path = PathBuf::from(home);
    base_path.push("Documents");
    base_path.push("lines_editor");

    // Create all directories in the path if they don't exist
    fs::create_dir_all(&base_path)?;

    // Get timestamp for filename
    let timestamp = get_timestamp()?;

    // Create filename based on whether custom_name is provided
    let filename = match custom_name {
        Some(name) => format!("{}_{}.txt", name, timestamp),
        None => format!("{}.txt", timestamp),
    };

    // Join the base path with the filename
    Ok(base_path.join(filename))
}

/// Module for detecting double-width (full-width) UTF-8 characters in terminal display.
///
/// This module provides fast, reliable detection of characters that occupy two columns
/// in terminal display, primarily East Asian characters (CJK) and full-width variants.
///
/// # Implementation Notes
/// - Uses a pre-compiled lookup table for O(1) performance
/// - No third-party dependencies
/// - Memory-safe with no dynamic allocation
/// - Based on Unicode 15.0 East Asian Width property
pub mod double_width {
    use crate::limits;

    /// Maximum number of double-width character ranges we support.
    /// Pre-allocated to avoid dynamic memory allocation per NASA Power of 10 rules.
    const MAX_RANGES: usize = 128;

    /// Compiled lookup table of Unicode ranges for double-width characters.
    /// Each tuple represents (start, end) of an inclusive range.
    ///
    /// # Source
    /// Based on Unicode East Asian Width property categories:
    /// - F (Fullwidth): Always double-width
    /// - W (Wide): Always double-width in East Asian contexts
    ///
    /// # Memory Layout
    /// Pre-allocated array avoids dynamic allocation.
    /// Unused slots are filled with (0, 0) which won't match valid characters.
    const DOUBLE_WIDTH_RANGES: [(u32, u32); MAX_RANGES] = [
        // CJK Symbols and Punctuation
        (0x3000, 0x303F),
        // Hiragana
        (0x3040, 0x309F),
        // Katakana
        (0x30A0, 0x30FF),
        // CJK Strokes
        (0x31C0, 0x31EF),
        // Katakana Phonetic Extensions
        (0x31F0, 0x31FF),
        // Enclosed CJK Letters and Months
        (0x3200, 0x32FF),
        // CJK Compatibility
        (0x3300, 0x33FF),
        // CJK Unified Ideographs Extension A
        (0x3400, 0x4DBF),
        // CJK Unified Ideographs (main block)
        (0x4E00, 0x9FFF),
        // Yi Syllables
        (0xA000, 0xA48F),
        // Yi Radicals
        (0xA490, 0xA4CF),
        // Hangul Jamo Extended-A
        (0xA960, 0xA97F),
        // Hangul Syllables
        (0xAC00, 0xD7AF),
        // CJK Compatibility Ideographs
        (0xF900, 0xFAFF),
        // Vertical Forms
        (0xFE10, 0xFE1F),
        // CJK Compatibility Forms
        (0xFE30, 0xFE4F),
        // Small Form Variants
        (0xFE50, 0xFE6F),
        // Halfwidth and Fullwidth Forms (fullwidth part)
        (0xFF01, 0xFF60),
        (0xFFE0, 0xFFE6),
        // Kana Supplement
        (0x1B000, 0x1B0FF),
        // Kana Extended-A
        (0x1B100, 0x1B12F),
        // Small Kana Extension
        (0x1B130, 0x1B16F),
        // CJK Unified Ideographs Extension B
        (0x20000, 0x2A6DF),
        // CJK Unified Ideographs Extension C
        (0x2A700, 0x2B73F),
        // CJK Unified Ideographs Extension D
        (0x2B740, 0x2B81F),
        // CJK Unified Ideographs Extension E
        (0x2B820, 0x2CEAF),
        // CJK Unified Ideographs Extension F
        (0x2CEB0, 0x2EBEF),
        // CJK Compatibility Ideographs Supplement
        (0x2F800, 0x2FA1F),
        // CJK Unified Ideographs Extension G
        (0x30000, 0x3134F),
        // Fill remaining slots with (0, 0) - won't match any valid character
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
        (0, 0),
    ];

    /// Determines if a UTF-8 character is double-width in terminal display.
    ///
    /// # Arguments
    /// * `c` - The character to check
    ///
    /// # Returns
    /// * `true` if the character occupies two columns in terminal display
    /// * `false` if the character occupies one column (or zero for combining marks)
    ///
    /// # Performance
    /// O(n) where n is the number of ranges (currently ~27), but with early exit
    /// optimization for ASCII characters which are the most common case.
    ///
    /// # Examples
    /// ```
    /// assert_eq!(is_double_width('A'), false);  // ASCII - single width
    /// assert_eq!(is_double_width('中'), true);  // Chinese - double width
    /// assert_eq!(is_double_width('あ'), true);  // Hiragana - double width
    /// assert_eq!(is_double_width('ア'), true);  // Katakana - double width
    /// assert_eq!(is_double_width('한'), true);  // Hangul - double width
    /// assert_eq!(is_double_width('Ａ'), true);  // Fullwidth Latin - double width
    /// ```
    ///
    /// # Edge Cases
    /// - Control characters: returns false
    /// - Combining marks: returns false (they don't advance cursor)
    /// - Emoji: most return false (emoji width is complex and font-dependent)
    /// - Invalid Unicode: returns false
    pub fn is_double_width(c: char) -> bool {
        let code_point = c as u32;

        // Fast path: ASCII characters are never double-width
        // This catches the most common case immediately
        if code_point < 0x80 {
            debug_assert!(code_point < 0x80, "ASCII range check failed");
            return false;
        }

        // Fast path: Characters below the first CJK block are mostly single-width
        // Exception: Some symbols in the 0x3000-0x303F range
        if code_point < 0x3000 {
            debug_assert!(code_point < 0x3000, "Pre-CJK range check failed");
            return false;
        }

        // Binary search through our sorted ranges
        // Loop counter for NASA Power of 10 rule #2
        let mut iterations = 0;

        let mut left = 0;
        let mut right = MAX_RANGES;

        while left < right && iterations < limits::DOUBLE_WIDTH_BINARY_SEARCH {
            iterations += 1;

            let mid = left + (right - left) / 2;
            let (range_start, range_end) = DOUBLE_WIDTH_RANGES[mid];

            // Skip empty slots (0, 0)
            if range_start == 0 && range_end == 0 {
                right = mid;
                continue;
            }

            if code_point < range_start {
                right = mid;
            } else if code_point > range_end {
                left = mid + 1;
            } else {
                // Found in range
                debug_assert!(
                    code_point >= range_start && code_point <= range_end,
                    "Character should be within found range"
                );
                return true;
            }
        }

        // Defensive assertion: we should have checked all relevant ranges
        debug_assert!(
            iterations <= limits::DOUBLE_WIDTH_BINARY_SEARCH,
            "Binary search exceeded maximum iterations"
        );

        false
    }

    /// Calculates the display width of a string in terminal columns.
    ///
    /// # Arguments
    /// * `text` - The text to measure
    ///
    /// # Returns
    /// * `Option<usize>` - The width in terminal columns, or None if calculation fails
    ///
    /// # Examples
    /// ```
    /// assert_eq!(calculate_display_width("Hello"), Some(5));
    /// assert_eq!(calculate_display_width("你好"), Some(4)); // Two double-width characters
    /// assert_eq!(calculate_display_width("Hello世界"), Some(9)); // 5 + 2*2
    /// ```
    ///
    /// # Error Handling
    /// Returns `None` if:
    /// - The string contains invalid UTF-8 (shouldn't happen with Rust strings)
    /// - Integer overflow occurs (extremely long strings)
    pub fn calculate_display_width(text: &str) -> Option<usize> {
        let mut width = 0usize;
        let mut char_count = 0;
        const MAX_CHARS: usize = 1_000_000; // Upper bound per NASA rule #2 TODO: math other MAX_CHARS?

        for c in text.chars() {
            // Prevent infinite loops with character count limit
            if char_count >= MAX_CHARS {
                return None;
            }
            char_count += 1;

            // Add 2 for double-width, 1 for single-width
            let char_width = if is_double_width(c) { 2 } else { 1 };

            // Check for overflow before adding
            width = width.checked_add(char_width)?;
        }

        // Defensive assertion: result should be reasonable
        debug_assert!(
            width <= text.len() * 2,
            "Display width should not exceed twice the byte length"
        );

        Some(width)
    }
}

/// Seeks to a specific line number in the file and returns the byte position
///
/// # Purpose
/// Efficiently finds the byte offset where a specific line starts in the file.
/// This allows us to seek directly to that position for display.
///
/// # Arguments
/// * `file` - Open file handle to read from
/// * `target_line` - Line number to seek to (0-indexed)
///
/// # Returns
/// * `Ok(byte_position)` - Byte offset where the target line starts
/// * `Err(io::Error)` - If file operations fail
///
/// # Defensive Programming
/// - Limits iterations to prevent infinite loops
/// - Returns error if target line exceeds file length
/// - Handles EOF gracefully
fn seek_to_line_number(file: &mut File, target_line: usize) -> io::Result<u64> {
    use std::io::{Read, Seek, SeekFrom};

    // Start at beginning of file
    file.seek(SeekFrom::Start(0))?;

    if target_line == 0 {
        return Ok(0); // Already at start
    }

    let mut current_line = 0usize;
    let mut byte_position = 0u64;
    let mut buffer = [0u8; 1];

    // Defensive: Limit iterations
    let mut iterations = 0;

    // Read byte by byte looking for newlines
    while current_line < target_line && iterations < limits::FILE_SEEK_BYTES {
        iterations += 1;

        match file.read(&mut buffer)? {
            0 => {
                // EOF before reaching target line
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    format!(
                        "File only has {} lines, requested line {}",
                        current_line, target_line
                    ),
                ));
            }
            1 => {
                byte_position += 1;
                if buffer[0] == b'\n' {
                    current_line += 1;
                }
            }
            _ => unreachable!("Single byte read returned unexpected count"),
        }
    }

    // Defensive: Check iteration limit
    if iterations >= limits::FILE_SEEK_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Maximum iterations exceeded while seeking to line",
        ));
    }

    // Assertion: We should have found the target line
    debug_assert_eq!(current_line, target_line, "Should have reached target line");

    Ok(byte_position)
}

/// Builds the window-to-file mapping for NoWrap mode
/// Note: this should be using a read-copy file at all times.
///
/// # Purpose
/// Reads file content and populates display buffers with proper line numbers
/// and text, while maintaining a complete mapping of which file byte each
/// terminal cell corresponds to.
///
/// # NoWrap Mode Behavior
/// - Each file line maps to exactly one display row (no wrapping)
/// - Lines longer than terminal width are truncated at display edge
/// - Horizontal scrolling is controlled by state.horizontal_line_char_offset
/// - Empty file lines still consume a display row
///
/// # Arguments
/// * `state` - Editor state containing buffers and window position info
/// * `original_file_path` - Absolute path to the file being displayed
///
/// # Returns
/// * `Ok(lines_processed)` - Number of file lines successfully processed
/// * `Err(io::Error)` - If file operations fail or invalid UTF-8 encountered
///
/// # State Modified
/// - `state.display_buffers` - Filled with line numbers and visible text
/// - `state.display_buffer_lengths` - Set to actual bytes used per row
/// - `state.window_map` - Updated with file position for each display cell
///
/// # Defensive Programming
/// - Validates file exists before reading
/// - Bounds checks all buffer accesses
/// - Handles invalid UTF-8 gracefully
/// - Limits iteration counts to prevent infinite loops
///
/// # Example
/// For a file starting "Hello\nWorld\n" with window at line 1:
/// - Row 0: "1 Hello"
/// - Row 1: "2 World"
/// WindowMap will map each character to its file byte position.
pub fn build_windowmap_nowrap(
    state: &mut EditorState,
    readcopy_file_path: &Path,
) -> io::Result<usize> {
    // Defensive: Validate inputs
    if !readcopy_file_path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "File path must be absolute",
        ));
    }

    if !readcopy_file_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File not found: {:?}", readcopy_file_path),
        ));
    }

    // Assertion: State should have valid dimensions
    debug_assert!(state.effective_rows > 0, "Effective rows must be positive");
    debug_assert!(state.effective_cols > 0, "Effective cols must be positive");

    // Clear existing buffers and map before building
    state.clear_display_buffers();
    state.window_map.clear();

    // Open file for reading
    let mut file = File::open(readcopy_file_path)?;

    // // Seek to window start position
    // use std::io::{Seek, SeekFrom};
    // file.seek(SeekFrom::Start(state.file_position_of_topline_start))?;

    // *** FIX: Calculate byte position for the target line ***
    let byte_position = seek_to_line_number(&mut file, state.line_count_at_top_of_window)?;

    // Update state with the correct byte position
    state.file_position_of_topline_start = byte_position;

    // Pre-allocate read buffer for one line at a time
    // Max line we'll try to read (defensive limit)
    let mut line_buffer = [0u8; limits::LINE_READ_BYTES];

    let mut current_display_row = 0usize;
    let mut current_line_number = state.line_count_at_top_of_window;
    let mut lines_processed = 0usize;
    let mut file_byte_position = state.file_position_of_topline_start;

    // Defensive: Limit iterations to prevent infinite loops
    let mut iteration_count = 0;

    // Process lines until display is full or file ends
    while current_display_row < state.effective_rows && iteration_count < limits::WINDOW_BUILD_LINES
    {
        iteration_count += 1;

        // Assertion: We should not exceed our display buffer count
        debug_assert!(current_display_row < 45, "Display row exceeds maximum");

        // Read one line from file (up to newline or MAX_LINE_BYTES)
        let (line_bytes, line_length, found_newline) =
            read_single_line(&mut file, &mut line_buffer)?;

        // Check for end of file
        if line_length == 0 && !found_newline {
            break; // End of file reached
        }

        // Write line number to display buffer
        let line_number_display = current_line_number + 1; // Convert 0-indexed to 1-indexed
        let line_num_bytes_written =
            state.write_line_number(current_display_row, line_number_display)?;

        // Calculate how many columns remain after line number
        let remaining_cols = state.effective_cols.saturating_sub(line_num_bytes_written);

        // Process the line text with horizontal offset
        let text_bytes_written = process_line_with_offset(
            state,
            current_display_row,
            line_num_bytes_written, // Column position after line number
            &line_bytes[..line_length],
            state.horizontal_line_char_offset,
            remaining_cols,
            file_byte_position,
        )?;

        // Update total buffer length for this row
        state.display_buffer_lengths[current_display_row] =
            line_num_bytes_written + text_bytes_written;

        // Advance to next line
        current_display_row += 1;
        current_line_number += 1;
        lines_processed += 1;

        // Update file position for next line
        file_byte_position += line_length as u64;
        if found_newline {
            file_byte_position += 1; // Account for newline character
        }
    }

    // Defensive: Check we didn't hit iteration limit
    if iteration_count >= limits::WINDOW_BUILD_LINES {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Maximum iterations exceeded in build_windowmap_nowrap",
        ));
    }

    // Assertion: Verify our line count makes sense
    debug_assert!(
        lines_processed <= state.effective_rows,
        "Processed more lines than display rows available"
    );

    Ok(lines_processed)
}

/// Saves the current read-copy back to the original file with backup
///
/// # Purpose
/// Safely saves changes by:
/// 1. Creating timestamped backup of original file
/// 2. Copying read-copy content to original file
/// 3. Marking state as unmodified
///
/// # Arguments
/// * `state` - Editor state with file paths
///
/// # Returns
/// * `Ok(())` - Save successful
/// * `Err(io::Error)` - Save operation failed
///
/// # Safety
/// - Original file backed up before overwrite
/// - Backup kept in archive directory
/// - If save fails, original file unchanged
fn save_file(state: &mut EditorState) -> io::Result<()> {
    // Defensive: Check we have both paths
    let original_path = state
        .original_file_path
        .as_ref()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "No original file path"))?;

    let read_copy_path = state
        .read_copy_path
        .as_ref()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "No read-copy path"))?;

    // Step 1: Create archive directory if it doesn't exist
    let archive_dir = original_path
        .parent()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot determine parent directory",
            )
        })?
        .join("archive");

    fs::create_dir_all(&archive_dir)?;

    // Step 2: Create timestamped backup of original
    let timestamp = createarchive_timestamp_with_precision(SystemTime::now(), true);
    let original_filename = original_path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Cannot determine filename"))?;

    let backup_path = archive_dir.join(format!(
        "{}_{}",
        timestamp,
        original_filename.to_string_lossy()
    ));

    // Step 3: Copy original to backup (if original exists)
    if original_path.exists() {
        fs::copy(original_path, &backup_path)?;
        println!("Backup created: {}", backup_path.display());
    }

    // Step 4: Copy read-copy to original location
    fs::copy(read_copy_path, original_path)?;

    // Step 5: Mark as unmodified
    state.is_modified = false;

    println!("File saved: {}", original_path.display());

    Ok(())
}

/// Reads a single line from file into buffer
///
/// # Purpose
/// Reads bytes from current file position until newline or buffer limit.
/// Does NOT include the newline character in the returned bytes.
///
/// # Arguments
/// * `file` - Open file handle positioned at start of line
/// * `buffer` - Pre-allocated buffer to read into
///
/// # Returns
/// * `Ok((buffer, bytes_read, found_newline))` - The buffer, bytes read, and whether newline was found
/// * `Err(io::Error)` - If read operation fails
///
/// # Defensive Notes
/// - Stops at newline character (0x0A)
/// - Stops if buffer is full
/// - Returns found_newline flag to distinguish EOF from empty line
fn read_single_line<'a>(
    file: &'a mut File,
    buffer: &'a mut [u8; 4096],
) -> io::Result<(&'a [u8; 4096], usize, bool)> {
    use std::io::Read;

    let mut bytes_read = 0usize;
    let mut found_newline = false;
    let mut single_byte = [0u8; 1];

    // Defensive: Limit iterations
    let mut iterations = 0;

    while bytes_read < buffer.len() && iterations < limits::LINE_READ_BYTES {
        iterations += 1;

        // Diagnostic: print bytes read so far
        if iterations % 10 == 0 {
            println!("Iterations: {}, Bytes read: {}", iterations, bytes_read);
        }

        // Read one byte at a time (inefficient but simple for MVP)
        match file.read(&mut single_byte)? {
            0 => break, // EOF reached
            1 => {
                if single_byte[0] == b'\n' {
                    found_newline = true;
                    break; // Don't include newline in buffer
                }

                buffer[bytes_read] = single_byte[0];
                bytes_read += 1;
            }
            _ => {
                // Should not happen with single byte read
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Unexpected read result",
                ));
            }
        }
    }
    println!(
        "Final bytes read: {}, Found newline: {}",
        bytes_read, found_newline
    );

    // Assertion: We should have stayed within bounds
    debug_assert!(bytes_read <= buffer.len(), "Read exceeded buffer size");
    debug_assert!(
        iterations <= limits::LINE_READ_BYTES,
        "Too many iterations in line read"
    );

    Ok((buffer, bytes_read, found_newline))
}

/// Processes a line with horizontal offset and writes visible portion to display
///
/// # Purpose
/// Takes a line's bytes, skips horizontal_offset characters, then writes
/// the visible portion to the display buffer while updating WindowMap.
///
/// # Arguments
/// * `state` - Editor state for buffers and map
/// * `row` - Display row index
/// * `col_start` - Starting column (after line number)
/// * `line_bytes` - The complete line text as bytes
/// * `horizontal_offset` - Number of characters to skip from line start
/// * `max_cols` - Maximum columns available for text
/// * `file_line_start` - Byte position where this line starts in file
///
/// # Returns
/// * `Ok(bytes_written)` - Number of bytes written to display buffer
/// * `Err(io::Error)` - If UTF-8 parsing fails or buffer access fails
///
/// # UTF-8 Handling
/// - Properly skips complete characters, not bytes
/// - Handles multi-byte UTF-8 sequences correctly
/// - Maps double-width characters to two display columns
fn process_line_with_offset(
    state: &mut EditorState,
    row: usize,
    col_start: usize,
    line_bytes: &[u8],
    horizontal_offset: usize,
    max_cols: usize,
    file_line_start: u64,
) -> io::Result<usize> {
    // Defensive: Validate row index
    if row >= 45 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Row {} exceeds maximum display rows", row),
        ));
    }

    // First pass: Skip horizontal_offset characters (not bytes!)
    let mut byte_index = 0usize;
    let mut chars_skipped = 0usize;

    // Defensive: Limit iterations
    let mut iterations = 0;

    while byte_index < line_bytes.len()
        && chars_skipped < horizontal_offset
        && iterations < limits::HORIZONTAL_SCROLL_CHARS
    {
        iterations += 1;

        // Assertion 4: byte_index should never exceed line length
        debug_assert!(
            byte_index < line_bytes.len(),
            "byte_index {} exceeds line length {}",
            byte_index,
            line_bytes.len()
        );

        // Determine character byte length from first byte
        let byte_val = line_bytes[byte_index];
        let char_len = if byte_val & 0b1000_0000 == 0 {
            1 // ASCII
        } else if byte_val & 0b1110_0000 == 0b1100_0000 {
            2 // 2-byte UTF-8
        } else if byte_val & 0b1111_0000 == 0b1110_0000 {
            3 // 3-byte UTF-8
        } else if byte_val & 0b1111_1000 == 0b1111_0000 {
            4 // 4-byte UTF-8
        } else {
            // Invalid UTF-8 or continuation byte - skip single byte
            1
        };

        // Skip this character
        byte_index = (byte_index + char_len).min(line_bytes.len());
        chars_skipped += 1;
    }

    // Assertion 5: We should have skipped exactly the requested amount or hit end
    debug_assert!(
        chars_skipped <= horizontal_offset,
        "Skipped {} characters but only {} requested",
        chars_skipped,
        horizontal_offset
    );

    // Second pass: Write visible characters to display buffer
    let mut display_col = col_start;
    let mut bytes_written = 0usize;
    iterations = 0; // Reset iteration counter

    // Reserve 1 or 2 columns at line end to prevent double-width characters from overflowing
    // A double-width char starting at max_cols-1 would extend to max_cols+1, causing overflow
    while byte_index < line_bytes.len()
        // Reserve 1 or 2 columns to prevent double-width overflow
        && display_col < col_start + max_cols - 1
        && iterations < limits::HORIZONTAL_SCROLL_CHARS
    {
        iterations += 1;

        // Assertion 6: byte_index should be within bounds
        debug_assert!(
            byte_index < line_bytes.len(),
            "byte_index {} exceeds line length {} in write phase",
            byte_index,
            line_bytes.len()
        );

        // Parse next UTF-8 character
        let byte_val = line_bytes[byte_index];
        let char_len = if byte_val & 0b1000_0000 == 0 {
            1 // ASCII
        } else if byte_val & 0b1110_0000 == 0b1100_0000 {
            2 // 2-byte UTF-8
        } else if byte_val & 0b1111_0000 == 0b1110_0000 {
            3 // 3-byte UTF-8
        } else if byte_val & 0b1111_1000 == 0b1111_0000 {
            4 // 4-byte UTF-8
        } else {
            // Skip invalid bytes
            byte_index += 1;
            continue;
        };

        // Check if complete character is available
        if byte_index + char_len > line_bytes.len() {
            break; // Incomplete character at end
        }

        // Get the character bytes
        let char_bytes = &line_bytes[byte_index..byte_index + char_len];

        // Assertion 7: Character bytes should be exactly char_len
        debug_assert_eq!(
            char_bytes.len(),
            char_len,
            "Character byte slice length mismatch"
        );

        // Check how many display columns this character needs
        let display_width = if char_len == 1 {
            1 // ASCII is always single-width
        } else {
            // Parse character to check if double-width
            match std::str::from_utf8(char_bytes) {
                Ok(s) => {
                    if let Some(ch) = s.chars().next() {
                        if double_width::is_double_width(ch) {
                            2
                        } else {
                            1
                        }
                    } else {
                        1 // Default to single-width
                    }
                }
                Err(_) => 1, // Invalid UTF-8, treat as single-width
            }
        };

        // Check if character fits in remaining space
        if display_col + display_width > col_start + max_cols {
            break; // Character would exceed display width
        }

        // Write character to display buffer
        if col_start + bytes_written + char_len <= 182 {
            // Copy bytes to display buffer
            for i in 0..char_len {
                state.display_buffers[row][col_start + bytes_written + i] = char_bytes[i];
            }

            // Update WindowMap for this character position
            let file_pos = FilePosition {
                byte_offset: file_line_start + byte_index as u64,
                line_number: state.line_count_at_top_of_window,
                byte_in_line: byte_index,
            };

            // Map all display columns this character occupies
            for i in 0..display_width {
                state
                    .window_map
                    .set_file_position(row, display_col + i, Some(file_pos))?;
            }

            bytes_written += char_len;
            display_col += display_width;
        } else {
            break; // Buffer full
        }

        byte_index += char_len;
    }

    // Defensive: Check iteration limit
    if iterations >= limits::HORIZONTAL_SCROLL_CHARS {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Maximum iterations exceeded in line processing",
        ));
    }

    // Assertion 9: Verify we stayed within display buffer bounds
    debug_assert!(
        bytes_written <= 182,
        "Wrote {} bytes but buffer is only 182 bytes",
        bytes_written
    );

    // Assertion 10: Verify display column stayed within bounds
    debug_assert!(
        display_col <= col_start + max_cols,
        "Display column {} exceeds limit {}",
        display_col,
        col_start + max_cols
    );

    Ok(bytes_written)
}

/// Determines if the current working directory is the user's home directory
///
/// # Purpose
/// Used to decide whether to enter memo mode (when in home) or require
/// a file path (when elsewhere).
///
/// # Returns
/// * `Ok(true)` - Currently in home directory
/// * `Ok(false)` - Not in home directory
/// * `Err(io::Error)` - Cannot determine home or current directory
///
/// # Platform Support
/// - Linux/macOS: Uses $HOME environment variable
/// - Windows: Uses %USERPROFILE% environment variable
///
/// # Errors
/// - Missing HOME/USERPROFILE environment variable
/// - Cannot determine current working directory
fn is_in_home_directory() -> io::Result<bool> {
    // Get current working directory
    let cwd = env::current_dir().map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Cannot determine current directory: {}", e),
        )
    })?;

    // Get home directory
    let home = get_home_directory()?;

    // Compare canonical paths to handle symlinks
    let canonical_cwd = fs::canonicalize(&cwd).unwrap_or_else(|_| cwd.clone());
    let canonical_home = fs::canonicalize(&home).unwrap_or_else(|_| home.clone());

    Ok(canonical_cwd == canonical_home)
}

/// Gets the user's home directory path
///
/// # Purpose
/// Cross-platform function to reliably find user's home directory.
/// Used for memo mode detection and default file location.
///
/// # Returns
/// * `Ok(PathBuf)` - Absolute path to user's home directory
/// * `Err(io::Error)` - Cannot determine home directory
///
/// # Platform Behavior
/// - Linux/macOS: Reads $HOME environment variable
/// - Windows: Reads %USERPROFILE% environment variable
///
/// # Fallback Strategy
/// If primary variable missing, tries alternative methods before failing.
fn get_home_directory() -> io::Result<PathBuf> {
    // Try primary home variable for platform
    let home_result = env::var("HOME").or_else(|_| env::var("USERPROFILE"));

    match home_result {
        Ok(home_str) => {
            let home_path = PathBuf::from(home_str);

            // Defensive: Verify the directory exists
            if !home_path.exists() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Home directory does not exist: {:?}", home_path),
                ));
            }

            // Defensive: Verify it's actually a directory
            if !home_path.is_dir() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Home path is not a directory: {:?}", home_path),
                ));
            }

            Ok(home_path)
        }
        Err(_) => {
            // Fallback: try USER environment variable with common paths
            if let Ok(user) = env::var("USER") {
                let possible_home = PathBuf::from(format!("/home/{}", user));
                if possible_home.exists() && possible_home.is_dir() {
                    return Ok(possible_home);
                }
            }

            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Cannot determine home directory: neither HOME nor USERPROFILE set",
            ))
        }
    }
}

/// Prompts user for a filename when a directory path is provided
///
/// # Purpose
/// Interactive input when user specifies a directory but not a filename.
/// Validates input to ensure safe filename creation.
///
/// # Returns
/// * `Ok(String)` - Valid filename entered by user
/// * `Err(io::Error)` - User cancelled or invalid input
///
/// # Input Validation
/// - Rejects empty input
/// - Rejects path separators (/, \)
/// - Rejects parent directory references (..)
/// - Limits filename length to 255 characters
fn prompt_for_filename() -> io::Result<String> {
    use std::io::{Write, stdin, stdout};

    println!("\n=== Create New File ===");
    println!("Enter filename (or 'q' to quit):");
    print!("> ");
    stdout().flush()?;

    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let trimmed = input.trim();

    // Check for quit command
    if trimmed == "q" || trimmed == "quit" || trimmed == "exit" {
        return Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "User cancelled file creation",
        ));
    }

    // Validate filename
    if trimmed.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Filename cannot be empty",
        ));
    }

    // Defensive: Reject path separators
    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Filename cannot contain path separators",
        ));
    }

    // Defensive: Reject parent directory reference
    if trimmed.contains("..") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Filename cannot contain parent directory references",
        ));
    }

    // Defensive: Check filename length (most filesystems limit to 255)
    if trimmed.len() > 255 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Filename too long: {} characters (max 255)", trimmed.len()),
        ));
    }

    // Add .txt extension if no extension provided
    let filename = if trimmed.contains('.') {
        trimmed.to_string()
    } else {
        format!("{}.txt", trimmed)
    };

    Ok(filename)
}

// ============================================================================
// COMMAND SYSTEM - Modular command handling
// ============================================================================

/// Represents all possible editor commands
/// Defensive: Explicit enum prevents arbitrary command injection
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    // Navigation
    MoveUp(usize),    // k - repeat count
    MoveDown(usize),  // j - repeat count
    MoveLeft(usize),  // h - repeat count
    MoveRight(usize), // l - repeat count

    // Mode changes
    EnterInsertMode, // i
    EnterVisualMode, // v
    EnterNormalMode, // n or Esc or ??? -> Ctrl-[

    // Text editing
    InsertNewline(char), // Insert single \n at cursor's file-position
    InsertText(String),  // Insert input buffer string at cursor
    DeleteChar,          // Delete character at cursor
    Backspace,           // Delete character before cursor

    // Select? up down left right byte count? or... to position?

    // File operations
    Save,        // s
    Quit,        // q
    SaveAndQuit, // w (write-quit)

    // Display
    ToggleWrap, // w (in normal mode)

    // No operation
    None,
}

/// Parses user input into a command
///
/// # Arguments
/// * `input` - Raw input string from user
/// * `current_mode` - Current editor mode for context-aware parsing
///
/// # Returns
/// * `Command` - Parsed command or Command::None if invalid
///
/// # Format
/// - Single char: `j` -> MoveDown(1)
/// - With count: `5j` -> MoveDown(5)
/// - Count then command: `10l` -> MoveRight(10)
/// - Mode commands: `i` -> EnterInsertMode
///
/// # Examples
/// - "j" -> MoveDown(1)
/// - "5j" -> MoveDown(5)
/// - "10k" -> MoveUp(10)
/// - "3h" -> MoveLeft(3)
/// - "7l" -> MoveRight(7)
pub fn parse_command(input: &str, current_mode: EditorMode) -> Command {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Command::None;
    }

    // In insert mode, most keys are text, not commands
    if current_mode == EditorMode::Insert {
        // Check for escape sequences to exit insert mode
        if trimmed == "\x1b" || trimmed == "ESC" || trimmed == "n" {
            return Command::EnterNormalMode;
        }
        // Everything else is text input (handled separately)
        return Command::None;
    }

    // Parse potential repeat count
    let mut chars = trimmed.chars();
    let mut count = 0usize;
    let mut command_char = None;
    // let mut found_command = false;

    // Defensive: Limit iteration on input parsing (not movement)
    let mut iterations = 0;

    while let Some(ch) = chars.next() {
        // COMMAND_PARSE_MAX_CHARS is the max allowed use do*N
        if iterations >= limits::COMMAND_PARSE_MAX_CHARS {
            return Command::None; // Too long to be valid command
        }
        iterations += 1;

        if ch.is_ascii_digit() && command_char.is_none() {
            // Build up count
            let digit = (ch as usize) - ('0' as usize);
            count = count.saturating_mul(10).saturating_add(digit);
        } else {
            command_char = Some(ch);
            break; // Found the command character
        }
    }

    // Default count to 1 if not specified
    if count == 0 {
        count = 1;
    }

    if current_mode == EditorMode::Normal {
        // Match command character
        match command_char {
            // if normal mode, move, if visial select?
            // TODO wq? two letters? word commands?
            Some('h') => Command::MoveLeft(count),
            Some('j') => Command::MoveDown(count),
            Some('k') => Command::MoveUp(count),
            Some('l') => Command::MoveRight(count),

            Some('i') => Command::EnterInsertMode,
            Some('v') => Command::EnterVisualMode,
            Some('q') => Command::Quit,
            Some('s') => Command::Save,
            Some('w') => Command::Save,
            // Some('wrap') => {
            //     if current_mode == EditorMode::Normal {
            //         Command::ToggleWrap
            //     } else {
            //         // Command::SaveAndQuit
            //         print("");
            //     }
            // }
            _ => Command::None,
        }
    } else if current_mode == EditorMode::Visual {
        match command_char {
            Some('i') => Command::EnterInsertMode,
            Some('v') => Command::EnterVisualMode,
            Some('q') => Command::Quit,
            Some('s') => Command::Save,
            Some('n') => Command::EnterNormalMode,
            // Some('v') => Command::EnterVisualMode,
            // Some('w') => Command::SelectNextWord,
            // Some('b') => Command::SelectPreviousWordBeginning,
            // Some('e') => Command::SelectNextWordEnd,
            //
            // Some('h') => Command::SelectLeft(count),
            // Some('j') => Command::SelectDown(count),
            // Some('k') => Command::SelectUp(count),
            // Some('l') => Command::SelectRight(count),
            _ => Command::None,
        }
    } else {
        // if current_mode == EditorMode::Insert {
        match command_char {
            // TODO: --flag commands? not letters?
            // Some('i') => Command::EnterInsertMode,
            // Some('v') => Command::EnterVisualMode,
            // Some('q') => Command::Quit,
            // Some('s') => Command::Save,
            // Some('n') => Command::EnterNormalMode,
            // Some('v') => Command::EnterVisualMode,
            // Some('w') => Command::SelectNextWord,
            // Some('b') => Command::SelectPreviousWordBeginning,
            // Some('e') => Command::SelectNextWordEnd,
            _ => Command::None,
        }
    }
}

/// Executes a command and updates editor state
///
/// # Arguments
/// * `state` - Current editor state to modify
/// * `command` - Command to execute
/// * `original_file_path` - Path to the file being edited
///
/// # Returns
/// * `Ok(true)` - Continue editor loop
/// * `Ok(false)` - Exit editor loop
/// * `Err(io::Error)` - Command execution failed
pub fn execute_command(state: &mut EditorState, command: Command) -> io::Result<bool> {
    // let base_edit_filepath: PathBuf = state
    //     .read_copy_path
    //     .as_ref()
    //     .map(|p| p.clone()) // Clone the PathBuf
    //     .unwrap_or_else(|| original_file_path.to_path_buf()); // path -> buff
    let base_edit_filepath: PathBuf = state
        .read_copy_path
        .as_ref()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "CRITICAL: No read-copy path available - cannot edit",
            )
        })?
        .clone();

    let edit_file_path = Path::new(&base_edit_filepath); // buff -> path!!

    match command {
        Command::MoveLeft(count) => {
            // Vim-like behavior: move cursor left, scroll window if at edge

            let mut remaining_moves = count;
            let mut needs_rebuild = false;

            // Defensive: Limit iterations to prevent infinite loops
            let mut iterations = 0;

            while remaining_moves > 0 && iterations < limits::CURSOR_MOVEMENT_STEPS {
                iterations += 1;

                if state.cursor.col > 0 {
                    // Cursor can move left within visible window
                    let cursor_moves = remaining_moves.min(state.cursor.col);
                    state.cursor.col -= cursor_moves;
                    remaining_moves -= cursor_moves;
                } else if state.horizontal_line_char_offset > 0 {
                    // Cursor at left edge, scroll window left
                    let scroll_amount = remaining_moves.min(state.horizontal_line_char_offset);
                    state.horizontal_line_char_offset -= scroll_amount;
                    remaining_moves -= scroll_amount;
                    needs_rebuild = true;
                } else {
                    // At absolute left edge - can't move further
                    break;
                }
            }

            // Defensive: Check iteration limit
            if iterations >= limits::CURSOR_MOVEMENT_STEPS {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Maximum iterations exceeded in MoveLeft",
                ));
            }

            // Only rebuild if we scrolled the window
            if needs_rebuild {
                // Rebuild window to show the change from read-copy file
                build_windowmap_nowrap(state, &edit_file_path)?;
            }

            Ok(true)
        }

        Command::MoveRight(count) => {
            // Vim-like behavior: move cursor right, scroll window if at edge

            let mut remaining_moves = count;
            let mut needs_rebuild = false;

            // Defensive: Limit iterations
            let mut iterations = 0;

            while remaining_moves > 0 && iterations < limits::CURSOR_MOVEMENT_STEPS {
                iterations += 1;

                // Calculate space available before right edge
                // Reserve 1 column to prevent display overflow
                let right_edge = state.effective_cols.saturating_sub(1);

                if state.cursor.col < right_edge {
                    // Cursor can move right within visible window
                    let space_available = right_edge - state.cursor.col;
                    let cursor_moves = remaining_moves.min(space_available);

                    // inspection
                    println!("Inspection cursor_moves-> {:?}", &cursor_moves);

                    state.cursor.col += cursor_moves;
                    remaining_moves -= cursor_moves;
                } else {
                    // Cursor at right edge, scroll window right
                    // Cap scroll to prevent excessive horizontal offset

                    if state.horizontal_line_char_offset < limits::CURSOR_MOVEMENT_STEPS {
                        let max_scroll =
                            limits::CURSOR_MOVEMENT_STEPS - state.horizontal_line_char_offset;
                        let scroll_amount = remaining_moves.min(max_scroll);
                        state.horizontal_line_char_offset += scroll_amount;
                        remaining_moves -= scroll_amount;
                        needs_rebuild = true;
                    } else {
                        // Hit maximum horizontal scroll
                        break;
                    }
                }
            }

            // Defensive: Check iteration limit
            if iterations >= limits::CURSOR_MOVEMENT_STEPS {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Maximum iterations exceeded in MoveRight",
                ));
            }

            // Only rebuild if we scrolled the window
            if needs_rebuild {
                // Rebuild window to show the change from read-copy file
                build_windowmap_nowrap(state, &edit_file_path)?;
            }

            Ok(true)
        }

        Command::MoveDown(count) => {
            // Vim-like behavior: move cursor down, scroll window if at bottom edge

            let mut remaining_moves = count;
            let mut needs_rebuild = false;

            // Defensive: Limit iterations
            let mut iterations = 0;

            while remaining_moves > 0 && iterations < limits::CURSOR_MOVEMENT_STEPS {
                iterations += 1;

                // Calculate space available before bottom edge
                let bottom_edge = state.effective_rows.saturating_sub(1);

                if state.cursor.row < bottom_edge {
                    // Cursor can move down within visible window
                    let space_available = bottom_edge - state.cursor.row;
                    let cursor_moves = remaining_moves.min(space_available);
                    state.cursor.row += cursor_moves;
                    remaining_moves -= cursor_moves;
                } else {
                    // Cursor at bottom edge, scroll window down
                    // Note: We should track total file lines to prevent scrolling past EOF
                    // For now, scroll unconditionally (will show empty lines at EOF)
                    state.line_count_at_top_of_window += remaining_moves;
                    remaining_moves = 0;
                    needs_rebuild = true;
                }
            }

            // Defensive: Check iteration limit
            if iterations >= limits::CURSOR_MOVEMENT_STEPS {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Maximum iterations exceeded in MoveDown",
                ));
            }

            // Rebuild window if we scrolled
            if needs_rebuild {
                // Rebuild window to show the change from read-copy file
                build_windowmap_nowrap(state, &edit_file_path)?;
            }

            Ok(true)
        }

        Command::MoveUp(count) => {
            // Vim-like behavior: move cursor up, scroll window if at top edge

            let mut remaining_moves = count;
            let mut needs_rebuild = false;

            // Defensive: Limit iterations
            let mut iterations = 0;

            while remaining_moves > 0 && iterations < limits::CURSOR_MOVEMENT_STEPS {
                iterations += 1;

                if state.cursor.row > 0 {
                    // Cursor can move up within visible window
                    let cursor_moves = remaining_moves.min(state.cursor.row);
                    state.cursor.row -= cursor_moves;
                    remaining_moves -= cursor_moves;
                } else if state.line_count_at_top_of_window > 0 {
                    // Cursor at top edge, scroll window up
                    let scroll_amount = remaining_moves.min(state.line_count_at_top_of_window);
                    state.line_count_at_top_of_window -= scroll_amount;
                    remaining_moves -= scroll_amount;
                    needs_rebuild = true;
                } else {
                    // At absolute top of file - can't move further
                    break;
                }
            }

            // Defensive: Check iteration limit
            if iterations >= limits::CURSOR_MOVEMENT_STEPS {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Maximum iterations exceeded in MoveUp",
                ));
            }

            // Rebuild window if we scrolled
            if needs_rebuild {
                // Rebuild window to show the change from read-copy file
                build_windowmap_nowrap(state, &edit_file_path)?;
            }

            Ok(true)
        }

        Command::InsertNewline(_) => {
            insert_newline_at_cursor(state, edit_file_path)?;

            // Rebuild window to show the change
            build_windowmap_nowrap(state, edit_file_path)?;

            Ok(true)
        }

        // Todo: why is there a phantom command?
        Command::InsertText(_) => {
            // This command is not used in normal flow
            // Text insertion happens via read_stdin_and_insert_to_file()
            eprintln!("Warning: InsertText command called directly (unexpected)");
            Ok(true)
        }
        Command::DeleteChar => {
            // Delete character at cursor position
            delete_char_at_cursor(state, edit_file_path)?;

            // Rebuild window to show the change from read-copy file
            build_windowmap_nowrap(state, &edit_file_path)?;

            Ok(true)
        }

        Command::Backspace => {
            // Delete character before cursor position
            backspace_at_cursor(state, edit_file_path)?;

            // Rebuild window to show the change from read-copy file
            build_windowmap_nowrap(state, &edit_file_path)?;

            Ok(true)
        }

        Command::EnterInsertMode => {
            state.mode = EditorMode::Insert;
            Ok(true)
        }

        Command::EnterNormalMode => {
            state.mode = EditorMode::Normal;
            Ok(true)
        }

        Command::EnterVisualMode => {
            state.mode = EditorMode::Visual;
            // Set selection start at current cursor position
            if let Ok(Some(file_pos)) = state
                .window_map
                .get_file_position(state.cursor.row, state.cursor.col)
            {
                state.selection_start = Some(file_pos);
            }
            Ok(true)
        }

        Command::Save => {
            save_file(state)?;
            Ok(true)
            // Save doesn't need rebuild (no content change in display)
        }

        Command::Quit => {
            if state.is_modified {
                // Todo, maybe have a press enter to proceed thing...
                println!("Warning: Unsaved changes! Use 'w' to save.");
                Ok(true)
            } else {
                Ok(false) // Signal to exit loop
            }
        }

        Command::SaveAndQuit => {
            save_file(state)?; // save file
            Ok(false) // Signal to exit after save
        }

        Command::ToggleWrap => {
            state.wrap_mode = match state.wrap_mode {
                WrapMode::Wrap => WrapMode::NoWrap,
                WrapMode::NoWrap => WrapMode::Wrap,
            };

            // Rebuild window with new wrap mode
            build_windowmap_nowrap(state, &edit_file_path)?;
            Ok(true)
        }

        Command::None => Ok(true),
    }
}

/// Deletes the character at the cursor position (like vim's 'x')
///
/// # Purpose
/// Deletes the character under the cursor. Cursor stays at same position.
/// If at end of line, does nothing.
///
/// # Arguments
/// * `state` - Editor state with cursor position
/// * `file_path` - Path to the file being edited (read-copy)
///
/// # Returns
/// * `Ok(())` - Character deleted successfully (or nothing to delete)
/// * `Err(io::Error)` - File operations failed
///
/// # Behavior
/// - Deletes UTF-8 character at cursor (handles multi-byte)
/// - If cursor at end of line, does nothing
/// - If deleting last char on line, cursor stays
/// - Marks file as modified
fn delete_char_at_cursor(state: &mut EditorState, file_path: &Path) -> io::Result<()> {
    // Step 1: Get file position from cursor
    let file_pos = match state
        .window_map
        .get_file_position(state.cursor.row, state.cursor.col)?
    {
        Some(pos) => pos,
        None => {
            // Cursor not on valid position (e.g., past end of line)
            return Ok(()); // Nothing to delete
        }
    };

    // Step 2: Read entire file (MVP approach)
    let mut content = fs::read_to_string(file_path)?;

    // Step 3: Validate byte offset
    let delete_position = file_pos.byte_offset as usize;
    if delete_position >= content.len() {
        // At or past end of file
        return Ok(()); // Nothing to delete
    }

    // Step 4: Find the character at this position and its byte length
    let char_at_pos = content[delete_position..].chars().next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid UTF-8 at delete position",
        )
    })?;

    let char_byte_len = char_at_pos.len_utf8();

    // Defensive: Ensure we don't go past end
    let end_position = (delete_position + char_byte_len).min(content.len());

    // Step 5: Remove the character
    content.drain(delete_position..end_position);

    // Step 6: Write modified content back
    fs::write(file_path, content)?;

    // Step 7: Mark file as modified
    state.is_modified = true;

    // Step 8: Log the edit
    state.log_edit(&format!(
        "DELETE_CHAR line:{} byte:{} char:'{}'",
        file_pos.line_number, file_pos.byte_offset, char_at_pos
    ))?;

    // Step 9: Cursor stays at same position
    // (the character after deleted one is now under cursor)

    Ok(())
}

/// Deletes the character before the cursor position (like vim's backspace)
///
/// # Purpose
/// Deletes the character before the cursor and moves cursor back one position.
/// If at start of line, deletes the newline (joins with previous line).
///
/// # Arguments
/// * `state` - Editor state with cursor position
/// * `file_path` - Path to the file being edited (read-copy)
///
/// # Returns
/// * `Ok(())` - Character deleted successfully (or nothing to delete)
/// * `Err(io::Error)` - File operations failed
///
/// # Behavior
/// - Deletes UTF-8 character before cursor (handles multi-byte)
/// - Cursor moves back one position
/// - If at start of line, joins with previous line
/// - If at start of file, does nothing
fn backspace_at_cursor(state: &mut EditorState, file_path: &Path) -> io::Result<()> {
    // Step 1: Check if we're at start of file
    if state.cursor.row == 0 && state.cursor.col == 0 {
        return Ok(()); // Nothing before cursor
    }

    // Step 2: Get file position from cursor
    let file_pos = match state
        .window_map
        .get_file_position(state.cursor.row, state.cursor.col)?
    {
        Some(pos) => pos,
        None => {
            // If cursor is past end of line, move back to end of line first
            if state.cursor.col > 0 {
                state.cursor.col -= 1;
            }
            return Ok(());
        }
    };

    // Step 3: Read entire file (MVP approach)
    let mut content = fs::read_to_string(file_path)?;

    // Step 4: Find the character BEFORE cursor position
    let current_position = file_pos.byte_offset as usize;

    if current_position == 0 {
        return Ok(()); // At start of file
    }

    // Find the start of the previous UTF-8 character
    let mut prev_char_start = current_position - 1;

    // Defensive: Limit iterations for finding UTF-8 boundary
    let mut iterations = 0;

    // Walk backward to find UTF-8 character boundary
    while prev_char_start > 0
        && iterations < limits::MAX_UTF8_BOUNDARY_SCAN
        && (content.as_bytes()[prev_char_start] & 0b1100_0000) == 0b1000_0000
    {
        // This is a UTF-8 continuation byte, keep going back
        prev_char_start -= 1;
        iterations += 1;
    }

    // Get the character we're about to delete (for logging)
    let char_to_delete = content[prev_char_start..]
        .chars()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 before cursor"))?;

    // Step 5: Remove the character
    content.drain(prev_char_start..current_position);

    // Step 6: Write modified content back
    fs::write(file_path, content)?;

    // Step 7: Mark file as modified
    state.is_modified = true;

    // Step 8: Log the edit
    state.log_edit(&format!(
        "BACKSPACE line:{} byte:{} char:'{}'",
        file_pos.line_number, prev_char_start, char_to_delete
    ))?;

    // Step 9: Move cursor back
    if char_to_delete == '\n' {
        // Deleted a newline - move to end of previous line
        if state.cursor.row > 0 {
            state.cursor.row -= 1;
            // TODO: Set cursor.col to end of previous line
            // For now, just move to a reasonable position
            if state.cursor.col > 0 {
                state.cursor.col -= 1;
            }
        }
    } else {
        // Deleted a regular character - move back one column
        if state.cursor.col > 0 {
            state.cursor.col -= 1;
        }
    }

    Ok(())
}

/// Inserts a newline character at the cursor position
///
/// # Purpose
/// MVP implementation of newline insertion. This is the simplest insertion
/// case - just insert a single '\n' character at cursor position.
///
/// # Arguments
/// * `state` - Editor state with cursor position
/// * `file_path` - Path to the file being edited (read-copy)
///
/// # Returns
/// * `Ok(())` - Newline inserted successfully
/// * `Err(io::Error)` - File operations failed
///
/// # Process
/// 1. Get byte position from cursor
/// 2. Read file into memory
/// 3. Insert '\n' at position
/// 4. Write back to file
/// 5. Update cursor position (move to start of new line)
///
/// # Note
/// This reads entire file into memory (MVP approach).
/// For large files, this will be slow. Future: use gap buffer.
fn insert_newline_at_cursor(state: &mut EditorState, file_path: &Path) -> io::Result<()> {
    // Step 1: Get file position from cursor
    let file_pos = state
        .window_map
        .get_file_position(state.cursor.row, state.cursor.col)?
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cursor not on valid file position",
            )
        })?;

    // Step 2: Read entire file (MVP approach)
    let mut content = fs::read_to_string(file_path)?;

    // Step 3: Validate byte offset
    let insert_position = file_pos.byte_offset as usize;
    if insert_position > content.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Insert position {} exceeds file length {}",
                insert_position,
                content.len()
            ),
        ));
    }

    // Step 4: Insert newline at position
    content.insert(insert_position, '\n');

    // Step 5: Write modified content back
    fs::write(file_path, content)?;

    // Step 6: Mark file as modified
    state.is_modified = true;

    // Step 7: Log the edit
    state.log_edit(&format!(
        "INSERT_NEWLINE line:{} byte:{}",
        file_pos.line_number, file_pos.byte_offset
    ))?;

    // Step 8: Update cursor - move to start of new line
    state.cursor.row += 1;
    state.cursor.col = 0;
    state.line_count_at_top_of_window += 1;

    Ok(())
}

/// Inserts a chunk of text at cursor position using file operations
///
/// # Arguments
/// * `state` - Editor state with cursor position
/// * `file_path` - Path to the read-copy file
/// * `text_bytes` - The bytes to insert
///
/// # Returns
/// * `Ok(())` - Text inserted successfully
/// * `Err(io::Error)` - File operation failed
fn insert_text_chunk_at_cursor_position(
    state: &mut EditorState,
    file_path: &Path,
    text_bytes: &[u8],
) -> io::Result<()> {
    use std::io::{Read, Seek, SeekFrom, Write};

    // Get cursor file position
    let file_pos = state
        .window_map
        .get_file_position(state.cursor.row, state.cursor.col)?
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cursor not on valid file position",
            )
        })?;

    let insert_position = file_pos.byte_offset;

    // Open file for read+write
    let mut file = OpenOptions::new().read(true).write(true).open(file_path)?;

    // Read bytes after insertion point into 8K buffer
    let mut after_buffer = [0u8; 8192];
    file.seek(SeekFrom::Start(insert_position))?;
    let bytes_after = file.read(&mut after_buffer)?;

    // Write new text at insertion position
    file.seek(SeekFrom::Start(insert_position))?;
    file.write_all(text_bytes)?;

    // Write the shifted bytes
    file.write_all(&after_buffer[..bytes_after])?;
    file.flush()?;

    // Update state
    state.is_modified = true;

    // Log the edit
    let text_str = std::str::from_utf8(text_bytes).unwrap_or("[invalid UTF-8]");
    state.log_edit(&format!(
        "INSERT line:{} byte:{} text:'{}'",
        file_pos.line_number, file_pos.byte_offset, text_str
    ))?;

    // Update cursor position
    let char_count = text_str.chars().count();
    state.cursor.col += char_count;

    Ok(())
}

/// Full-featured editor mode for editing files
///
/// # Purpose
/// Main entry point for full editor functionality (not memo mode).
/// Handles file creation, opening, and launching the editor loop.
///
/// # Arguments
/// * `original_file_path` - Optional path to file or directory
///
/// # Returns
/// * `Ok(())` - Editor session completed successfully
/// * `Err(io::Error)` - File operations failed
///
/// # Behavior
/// - `None` - Error (requires path in full editor mode)
/// - `Some(file)` - Opens existing or creates new file
/// - `Some(dir)` - Prompts for filename, creates in directory
///
/// # File Creation
/// - Creates parent directories if needed
/// - Initializes new files with timestamp header
/// - Creates read-copy for safety
pub fn full_lines_editor(original_file_path: Option<PathBuf>) -> io::Result<()> {
    /*
    Workflow:
    A: when cwd is os home dir: always memo-mode (old version mode)
    when not in cwd is os home dir
    B: when provided a valid exsint file file path, open file
    C: when provided a new path, make that path
    D: when given new file name, make that file
    E: when given a path but no file name, ask user for file name
    */

    // Determine the target file path
    let target_path = match original_file_path {
        None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "File path required in full editor mode. Usage: lines <filename>",
            ));
        }
        Some(path) => {
            // Convert to absolute path
            let absolute_path = if path.is_absolute() {
                path.clone()
            } else {
                env::current_dir()?.join(&path)
            };

            // Check if path exists and what type it is
            if absolute_path.exists() {
                if absolute_path.is_dir() {
                    // Directory: prompt for filename
                    println!("Directory specified: {}", absolute_path.display());
                    let filename = prompt_for_filename()?;
                    absolute_path.join(filename)
                } else {
                    // Existing file: use as-is
                    absolute_path
                }
            } else {
                // Path doesn't exist: treat as new file
                // Check if parent looks like a directory (ends with separator)
                let path_str = path.to_string_lossy();
                if path_str.ends_with('/') || path_str.ends_with('\\') {
                    // Treat as directory that needs creating
                    fs::create_dir_all(&absolute_path)?;
                    println!("Created directory: {}", absolute_path.display());
                    let filename = prompt_for_filename()?;
                    absolute_path.join(filename)
                } else {
                    // Treat as file that needs creating
                    // Create parent directories if needed
                    if let Some(parent) = absolute_path.parent() {
                        if !parent.exists() {
                            println!("Creating parent directories: {}", parent.display());
                            fs::create_dir_all(parent)?;
                        }
                    }
                    absolute_path
                }
            }
        }
    };

    // Defensive: Final validation
    if target_path.to_string_lossy().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid file path: empty path",
        ));
    }

    println!("\n=== Opening Lines Editor ===");
    println!("File: {}", target_path.display());

    // Create file if it doesn't exist
    if !target_path.exists() {
        println!("Creating new file...");
        let header = memo_mode_get_header_text()?;

        // Create with header
        let mut file = File::create(&target_path)?;
        writeln!(file, "{}", header)?;
        writeln!(file)?; // Empty line after header
        file.flush()?;

        println!("Created new file with header");
    }
    // Initialize editor state
    let session_time_base = createarchive_timestamp_with_precision(SystemTime::now(), true);

    let (session_time_stamp1, session_time_stamp2) =
        match split_timestamp_no_heap(&session_time_base) {
            Ok((ts4, ts5)) => (ts4, ts5),
            Err(e) => {
                eprintln!("Error: {}", e);
                // Create two empty FixedSize32Timestamp structs as defaults
                let empty =
                    FixedSize32Timestamp::from_str("err01_01_01_01_01").unwrap_or_else(|_| {
                        // If even the fallback fails, create manually
                        FixedSize32Timestamp {
                            data: [0u8; 32],
                            len: 0,
                        }
                    });
                (empty, empty)
            }
        };

    let mut state = EditorState::new();
    state.original_file_path = Some(target_path.clone());

    // Initialize session directory FIRST
    initialize_session_directory(&mut state, session_time_stamp1)?;

    // Get session directory path (we just initialized it)
    let session_dir = state
        .session_directory_path
        .as_ref()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Session directory not initialized"))?;

    // Create read-copy for safety
    let read_copy_path =
        create_a_readcopy_of_file(&target_path, session_dir, session_time_stamp2.to_string())?;
    println!("Read-copy: {}", read_copy_path.display());

    // Initialize editor state
    let mut state = EditorState::new();
    state.original_file_path = Some(target_path.clone());
    state.read_copy_path = Some(read_copy_path);

    // // TODO: Replace with full editor loop when ready
    // // For now, use the test display functionality
    // println!("\n--- File Preview (first 21 lines) ---");
    // state.line_count_at_top_of_window = 0;
    // state.file_position_of_topline_start = 0;
    // state.horizontal_line_char_offset = 0;

    // let lines_processed = build_windowmap_nowrap(&mut state, &target_path)?;
    // println!("Lines in window: {}", lines_processed);
    // println!("{}", "-".repeat(40));

    // display_window(&state)?;

    // println!("\n[Full editor mode will be implemented here]");

    // // Initialize editor state
    // let mut state = EditorState::new();
    // state.original_file_path = Some(target_path.clone());
    // state.read_copy_path = Some(read_copy_path);

    // Initialize window position
    state.line_count_at_top_of_window = 0;
    state.file_position_of_topline_start = 0;
    state.horizontal_line_char_offset = 0;

    // Build initial window content
    // Get the read_copy path BEFORE the mutable borrow
    let read_copy = state
        .read_copy_path
        .clone()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No read copy path"))?;

    // Now we can mutably borrow state
    let lines_processed = build_windowmap_nowrap(&mut state, &read_copy)?;
    // build_windowmap_nowrap(&mut state, &target_path)?;
    println!("Loaded {} lines", lines_processed);

    // ADD THESE TWO LINES:
    state.cursor.row = 0;
    state.cursor.col = 0;

    // Main editor loop
    let stdin = io::stdin();
    let mut input_buffer = String::new();
    let mut continue_editing = true;

    // Defensive: Limit loop iterations to prevent infinite loops
    let mut iteration_count = 0;

    while continue_editing && iteration_count < limits::MAIN_EDITOR_LOOP_COMMANDS {
        iteration_count += 1;

        // Clear input buffer for new command
        input_buffer.clear();

        // Render TUI (convert LinesError to io::Error)
        render_tui(&state, &input_buffer)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Display error: {}", e)))?;

        // Read user input
        // TODO: this is against spec, against scope
        // this MUST be changed
        //
        stdin.read_line(&mut input_buffer)?;

        // Handle input based on mode
        // TODO: these should be tested
        // to find the least-bad way to implement
        // fewer-collisions with normal text entry
        // balanced with being memorable
        // possibly an added info-line blurb hint -n -s -v -wq
        if state.mode == EditorMode::Insert {
            let trimmed = input_buffer.trim();

            // Check for exit insert mode commands
            if trimmed == "-n" || trimmed == "\x1b" {
                // // Exit insert mode
                // continue_editing =
                //     execute_command(&mut state, Command::EnterNormalMode, &target_path)?;
                // // Rebuild window to refresh the TUI display
                // build_windowmap_nowrap(&mut state, &target_path)?;
                continue_editing = execute_command(&mut state, Command::EnterNormalMode)?;

                // Get the read_copy path BEFORE the mutable borrow
                let read_copy = state
                    .read_copy_path
                    .clone()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No read copy path"))?;

                // Now we can mutably borrow state
                build_windowmap_nowrap(&mut state, &read_copy)?;
            } else if trimmed == "-v" {
                continue_editing = execute_command(&mut state, Command::EnterVisualMode)?;

                // Get the read_copy path BEFORE the mutable borrow
                let read_copy = state
                    .read_copy_path
                    .clone()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No read copy path"))?;

                // Now we can mutably borrow state
                build_windowmap_nowrap(&mut state, &read_copy)?;
            } else if trimmed == "-s" || trimmed == "-w" {
                // Exit insert mode
                continue_editing = execute_command(&mut state, Command::Save)?;
            } else if trimmed == "-wq" {
                // Exit insert mode
                continue_editing = execute_command(&mut state, Command::SaveAndQuit)?;
            } else if trimmed.is_empty() {
                // Empty line = newline insertion
                continue_editing = execute_command(&mut state, Command::InsertNewline('\n'))?;
            } else {
                // Read stdin and insert text
                // match read_stdin_and_insert_to_file(&mut state, &read_copy)? {
                //     Some(command) => {
                //         // Input was a command - execute it
                //         continue_editing = execute_command(&mut state, command)?;
                //     }
                //     None => {
                //         // Text was inserted - rebuild window
                //         build_windowmap_nowrap(&mut state, &read_copy)?;
                //     }
                // }
                // Read and process stdin directly
                // It's text - insert it directly without re-reading stdin
                let text_bytes = trimmed.as_bytes();
                insert_text_chunk_at_cursor_position(&mut state, &read_copy, text_bytes)?;
                build_windowmap_nowrap(&mut state, &read_copy)?;
            }
        } else {
            // IF in  Normal/Visual mode: parse as command
            // let command = parse_command(&input_buffer, state.mode);
            // continue_editing = execute_command(&mut state, command, &target_path)?;

            // Normal/Visual mode: parse as command
            let trimmed = input_buffer.trim();

            let command = if trimmed.is_empty() {
                // Empty enter: repeat last command
                match state.the_last_command.clone() {
                    Some(cmd) => cmd,
                    None => Command::None, // No previous command
                }
            } else {
                // Parse new command
                parse_command(&input_buffer, state.mode)
            };

            // Execute command
            continue_editing = execute_command(&mut state, command.clone())?;

            // Store command for repeat (only if it's not Command::None)
            if command != Command::None {
                state.the_last_command = Some(command);
            }
        }
    }

    // Defensive: Check if we hit iteration limit
    if iteration_count >= limits::MAIN_EDITOR_LOOP_COMMANDS {
        eprintln!("Warning: Editor loop exceeded maximum iterations");
    }

    // Clean exit
    println!("\nExciting Lines Editor!");

    // Clean up read-copy if it exists
    if let Some(read_copy) = state.read_copy_path {
        if read_copy.exists() {
            fs::remove_file(read_copy).ok(); // Ignore errors on cleanup
        }
    }

    Ok(())
}

/// Creates a read-only copy of the file in the session directory
///
/// # Purpose
/// Creates a timestamped copy in the session directory that won't be modified
/// during editing. This prevents corruption if the editor crashes while writing.
/// Read-copies are VISIBLE (no hidden files) and located in session directory
/// for easy access and crash recovery.
///
/// # Arguments
/// * `original_path` - Path to the original file
/// * `session_dir` - Path to this session's directory (from EditorState)
///
/// # Returns
/// * `Ok(PathBuf)` - Path to the read-copy in session directory
/// * `Err(io::Error)` - Copy operation failed
///
/// # File Naming
/// Original: `/path/to/file.txt`
/// Session dir: `{executable_dir}/lines_data/tmp/sessions/2025_01_15_14_30_45/`
/// Read-copy: `{session_dir}/2025_01_15_14_30_45_file.txt`
///
/// # Design Notes
/// - NO hidden files (no leading dot) - files should be visible to user
/// - Stored in session directory for crash recovery
/// - Timestamp prefix ensures uniqueness
/// - Session directory persists after exit for recovery
fn create_a_readcopy_of_file(
    original_path: &Path,
    session_dir: &Path,
    session_time_stamp: String,
) -> io::Result<PathBuf> {
    // Defensive: Validate inputs
    if !original_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Original file does not exist: {:?}", original_path),
        ));
    }

    if !session_dir.exists() || !session_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Session directory does not exist: {:?}", session_dir),
        ));
    }

    // Get original filename
    let file_name = original_path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Cannot determine filename"))?
        .to_string_lossy();

    // Build read-copy filename: {timestamp}_{original_filename}
    // NO leading dot - file should be visible
    let read_copy_name = format!("{}_{}", session_time_stamp, file_name);
    let read_copy_path = session_dir.join(&read_copy_name);

    // Defensive: Check if read-copy already exists (shouldn't happen with timestamp)
    if read_copy_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Read-copy already exists: {:?}", read_copy_path),
        ));
    }

    // Copy the file to session directory
    fs::copy(original_path, &read_copy_path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to create read-copy: {}", e),
        )
    })?;

    // Defensive: Verify copy succeeded
    if !read_copy_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Read-copy creation reported success but file not found",
        ));
    }

    // Log success for user visibility
    println!("Read-copy created: {}", read_copy_path.display());

    // Assertion: Verify result is valid
    debug_assert!(
        read_copy_path.is_absolute(),
        "Read-copy path should be absolute"
    );
    debug_assert!(
        read_copy_path.exists(),
        "Read-copy should exist after creation"
    );

    Ok(read_copy_path)
}

/// Prints help message to stdout
///
/// # Purpose
/// Displays usage information and available commands.
/// Called when user runs `lines --help`.
fn print_help() {
    println!("Lines Editor - The Helps");
    println!("USAGE:");
    println!("    lines [OPTIONS] [FILE]");
    println!("OPTIONS:");
    println!("    --help, -h      Show this help message");
    println!("    --version, -v   Show version information");
    println!("    --files         Open file manager");
    println!("MODES:");
    println!("    Memo Mode:      Run from home directory, Append-only quickie");
    println!("                    Creates dated files in ~/Documents/lines_editor/");
    println!("    Full Editor:    Run from any other directory");
    println!("                    Requires file path argument");
    println!("NAVIGATION:");
    println!("    hjkl        Move cursor");
    println!("    5j, 10l     Move with repeat count");
    println!("    [Empty Enter]     Repeat last command (Normal/Visual/ ...?)");
    println!("EXAMPLES in terminal/shell");
    println!("  lines                  Memo mode (if in home)");
    println!("  lines notes.txt        Create/open notes.txt");
    println!("  lines ./mydir/         Create new file in directory");
}

/// Formats the bottom info bar with current editor state
///
/// # Purpose
/// Shows critical state info: mode, cursor position, filename, and input buffer.
/// All info on ONE line to minimize vertical space usage.
///
/// # Arguments
/// * `state` - Current editor state
/// * `input_buffer` - Current command/insert input (if any)
///
/// # Returns
/// * `Ok(String)` - Formatted info bar string
/// * `Err(LinesError)` - If formatting fails
///
/// # Format
/// "NORMAL line 42, col 7 document.txt > command_here"
/// "INSERT line 42, col 7 document.txt > text being typed"
///
/// # Design
/// - Mode in caps for visibility
/// - Line/col for cursor tracking
/// - Filename for context
/// - Input buffer shows what user is typing
fn format_info_bar(state: &EditorState, input_buffer: &str) -> Result<String> {
    // Get mode string
    let mode_str = match state.mode {
        EditorMode::Normal => "NORMAL",
        EditorMode::Insert => "INSERT",
        EditorMode::Visual => "VISUAL",
        EditorMode::MultiCursor => "MULTI",
    };

    // Get current line and column
    // Line is 1-indexed for display (humans count from 1)
    let line_display = state.line_count_at_top_of_window + state.cursor.row + 1;
    let col_display = state.cursor.col + 1;

    // Get filename (or "unnamed" if none)
    let filename = state
        .original_file_path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unnamed");

    // Truncate input buffer if too long (leave room for other info)
    let max_input_len = 40;
    let display_input = if input_buffer.len() > max_input_len {
        format!("{}...", &input_buffer[..max_input_len - 3])
    } else {
        input_buffer.to_string()
    };

    // Build the info bar
    let info = format!(
        "{}{} {}line {} col {} {}{} > {}{}",
        YELLOW, mode_str, RED, line_display, col_display, YELLOW, filename, display_input, RESET
    );

    Ok(info)
}

/// Renders the complete TUI to terminal: legend + content + info bar
///
/// # Purpose
/// Displays the minimal 3-section TUI:
/// 1. Top: Command legend (1 line)
/// 2. Middle: File content (effective_rows lines)
/// 3. Bottom: Info bar with command input (1 line)
///
/// # Arguments
/// * `state` - Current editor state with display buffers
/// * `input_buffer` - Current user input/command
///
/// # Returns
/// * `Ok(())` - Successfully rendered
/// * `Err(LinesError)` - Display operation failed
///
/// # Layout
/// ```text
/// quit ins vis save undo hjkl wb /search       <- Legend (1 line)
/// 1 First line of file content                 <- Content start
/// 2 Second line of file content
/// ...
/// N Last visible line                          <- Content end
/// NORMAL line 42, col 7 doc.txt > cmd_         <- Info bar (1 line)
/// ```
///
/// # Design Goals
/// - Only 2 non-content lines (legend + info)
/// - No wasted space, no filler lines
/// - All essential info visible
/// - Clean, minimal aesthetic
pub fn render_tui(state: &EditorState, input_buffer: &str) -> Result<()> {
    // Clear screen
    print!("\x1B[2J\x1B[H");
    io::stdout()
        .flush()
        .map_err(|e| LinesError::DisplayError(format!("Failed to flush stdout: {}", e)))?;

    // === TOP LINE: LEGEND ===
    let legend = format_navigation_legend()?;
    println!("{}", legend);

    // // === MIDDLE: FILE CONTENT ===
    // // Render each content row
    // for row in 0..state.effective_rows {
    //     if state.display_buffer_lengths[row] > 0 {
    //         let row_content = &state.display_buffers[row][..state.display_buffer_lengths[row]];

    //         match std::str::from_utf8(row_content) {
    //             Ok(row_str) => println!("{}", row_str),
    //             Err(_) => println!("�"), // Invalid UTF-8 fallback
    //         }
    //     } else {
    //         // Empty row - print newline to maintain spacing
    //         println!();
    //     }
    // }

    // === MIDDLE: FILE CONTENT WITH CURSOR ===
    for row in 0..state.effective_rows {
        if state.display_buffer_lengths[row] > 0 {
            let row_content = &state.display_buffers[row][..state.display_buffer_lengths[row]];

            match std::str::from_utf8(row_content) {
                Ok(row_str) => {
                    // ADD CURSOR HIGHLIGHTING HERE (was missing!)
                    let display_str = render_row_with_cursor(state, row, row_str);
                    println!("{}", display_str);
                }
                Err(_) => println!("�"),
            }
        } else {
            // Show cursor on empty rows if cursor is here
            if row == state.cursor.row {
                println!("{}{}{}█{}", "\x1b[1m", "\x1b[31m", "\x1b[47m", "\x1b[0m");
            } else {
                println!();
            }
        }
    }

    // === BOTTOM LINE: INFO BAR ===
    let info_bar = format_info_bar(state, input_buffer)?;
    print!("{}", info_bar);

    io::stdout()
        .flush()
        .map_err(|e| LinesError::DisplayError(format!("Failed to flush stdout: {}", e)))?;

    Ok(())
}

/// Renders one row of the display with cursor highlighting
///
/// # Purpose
/// Takes a display buffer row and adds cursor highlighting if cursor is on this row.
/// Uses ANSI escape codes to show cursor position visually.
///
/// # Arguments
/// * `state` - Editor state with cursor position
/// * `row_index` - Which display row we're rendering (0-indexed)
/// * `row_content` - The text content for this row
///
/// # Returns
/// * `String` - The row with cursor highlighting applied
fn render_row_with_cursor(state: &EditorState, row_index: usize, row_content: &str) -> String {
    // Not on cursor row - return as-is
    if row_index != state.cursor.row {
        return row_content.to_string();
    }

    // On cursor row - highlight the cursor position
    const BOLD: &str = "\x1b[1m";
    const RED: &str = "\x1b[31m";
    const BG_WHITE: &str = "\x1b[47m";
    const RESET: &str = "\x1b[0m";

    let chars: Vec<char> = row_content.chars().collect();
    let mut result = String::with_capacity(row_content.len() + 20); // Extra for ANSI codes

    // Defensive: Handle cursor beyond line end
    let cursor_col = state.cursor.col.min(chars.len());

    // Build string with cursor highlighting
    for (i, &ch) in chars.iter().enumerate() {
        if i == cursor_col {
            // This is the cursor position - highlight it
            result.push_str(&format!("{}{}{}{}{}", BOLD, RED, BG_WHITE, ch, RESET));
        } else {
            result.push(ch);
        }
    }

    // If cursor is at/past end of line, show a space character as cursor
    if cursor_col >= chars.len() {
        result.push_str(&format!("{}{}{}█{}", BOLD, RED, BG_WHITE, RESET));
    }

    result
}

/// Renders TUI to a test writer (for testing without terminal)
///
/// # Purpose
/// Same as render_tui but writes to provided writer instead of stdout.
/// Allows testing TUI layout without actual terminal.
///
/// # Arguments
/// * `state` - Current editor state
/// * `input_buffer` - Current user input
/// * `writer` - Where to write output (e.g., test buffer)
///
/// # Returns
/// * `Ok(())` - Successfully rendered
/// * `Err(LinesError)` - Display operation failed
pub fn render_tui_to_writer<W: Write>(
    state: &EditorState,
    input_buffer: &str,
    writer: &mut W,
) -> Result<()> {
    // Top legend
    let legend = format_navigation_legend()?;
    writeln!(writer, "{}", legend)
        .map_err(|e| LinesError::DisplayError(format!("Write failed: {}", e)))?;

    // Content rows
    for row in 0..state.effective_rows {
        if state.display_buffer_lengths[row] > 0 {
            let row_content = &state.display_buffers[row][..state.display_buffer_lengths[row]];

            match std::str::from_utf8(row_content) {
                Ok(row_str) => {
                    // ONLY CHANGE: Apply cursor highlighting if cursor is on this row
                    let display_str = render_row_with_cursor(state, row, row_str);
                    writeln!(writer, "{}", display_str)
                }
                Err(_) => writeln!(writer, "�"),
            }
            .map_err(|e| LinesError::DisplayError(format!("Write failed: {}", e)))?;
        } else {
            // ONLY CHANGE: Show cursor on empty rows if cursor is here
            if row == state.cursor.row {
                writeln!(
                    writer,
                    "{}{}{}█{}",
                    "\x1b[1m", "\x1b[31m", "\x1b[47m", "\x1b[0m"
                )
            } else {
                writeln!(writer)
            }
            .map_err(|e| LinesError::DisplayError(format!("Write failed: {}", e)))?;
        }
    }

    // Bottom info bar
    let info_bar = format_info_bar(state, input_buffer)?;
    write!(writer, "{}", info_bar)
        .map_err(|e| LinesError::DisplayError(format!("Write failed: {}", e)))?;

    writer
        .flush()
        .map_err(|e| LinesError::DisplayError(format!("Flush failed: {}", e)))?;

    Ok(())
}

// /// Renders TUI to a test writer (for testing without terminal)
// ///
// /// # Purpose
// /// Same as render_tui but writes to provided writer instead of stdout.
// /// Allows testing TUI layout without actual terminal.
// ///
// /// # Arguments
// /// * `state` - Current editor state
// /// * `input_buffer` - Current user input
// /// * `writer` - Where to write output (e.g., test buffer)
// ///
// /// # Returns
// /// * `Ok(())` - Successfully rendered
// /// * `Err(LinesError)` - Display operation failed
// pub fn render_tui_to_writer<W: Write>(
//     state: &EditorState,
//     input_buffer: &str,
//     writer: &mut W,
// ) -> Result<()> {
//     // Top legend
//     let legend = format_navigation_legend()?;
//     writeln!(writer, "{}", legend)
//         .map_err(|e| LinesError::DisplayError(format!("Write failed: {}", e)))?;

//     // Content rows
//     for row in 0..state.effective_rows {
//         if state.display_buffer_lengths[row] > 0 {
//             let row_content = &state.display_buffers[row][..state.display_buffer_lengths[row]];

//             match std::str::from_utf8(row_content) {
//                 Ok(row_str) => writeln!(writer, "{}", row_str),
//                 Err(_) => writeln!(writer, "�"),
//             }
//             .map_err(|e| LinesError::DisplayError(format!("Write failed: {}", e)))?;
//         } else {
//             writeln!(writer)
//                 .map_err(|e| LinesError::DisplayError(format!("Write failed: {}", e)))?;
//         }
//     }

//     // Bottom info bar
//     let info_bar = format_info_bar(state, input_buffer)?;
//     write!(writer, "{}", info_bar)
//         .map_err(|e| LinesError::DisplayError(format!("Write failed: {}", e)))?;

//     writer
//         .flush()
//         .map_err(|e| LinesError::DisplayError(format!("Flush failed: {}", e)))?;

//     Ok(())
// }

/// Initializes the session directory structure for this editing session
///
/// # Purpose
/// Creates the lines_data infrastructure and a unique session directory
/// for this run of the editor. Session directories persist after exit
/// for crash recovery purposes.
///
/// # Directory Structure Created
/// ```text
/// {executable_dir}/
///   lines_data/
///     tmp/
///       sessions/
///         {timestamp}/          <- This session's directory
/// ```
///
/// # Arguments
/// * `state` - Editor state to update with session directory path
///
/// # Returns
/// * `Ok(())` - Session directory created and path stored in state
/// * `Err(io::Error)` - If directory creation fails
///
/// # State Modified
/// - `state.session_directory_path` - Set to absolute path of session directory
///
/// # Crash Recovery
/// Session directories are NOT deleted on exit. This allows recovery of
/// read-copy files if the editor crashes or is interrupted.
///
/// # Future Enhancement
/// TODO: On startup, detect existing session directories and offer user
/// the option to recover interrupted sessions. Display path to recovery files.
///
/// # Error Handling
/// If session directory creation fails, editor should not continue as
/// read-copy operations depend on this directory existing.
fn initialize_session_directory(
    state: &mut EditorState,
    session_time_stamp: FixedSize32Timestamp,
) -> io::Result<()> {
    // Defensive: Verify state is in clean initial state
    debug_assert!(
        state.session_directory_path.is_none(),
        "Session directory should not be initialized twice"
    );

    // Step 1: Ensure base directory structure exists
    // Creates: {executable_dir}/lines_data/tmp/sessions/
    let base_sessions_path = "lines_data/tmp/sessions";

    let sessions_dir = make_verify_or_create_executabledirectoryrelative_canonicalized_dir_path(
        base_sessions_path,
    )
    .map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to create sessions directory structure: {}", e),
        )
    })?;

    // Defensive: Verify the path is a directory
    if !sessions_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Sessions path exists but is not a directory",
        ));
    }

    // Step 2: Get timestamp for this session
    // let session_time_stamp = get_session_timestamp()?;
    // let session_time_stamp = createarchive_timestamp_with_precision(SystemTime::now(), true);

    // Step 3: Create this session's directory
    // let session_dir_name = format!("{}/", session_time_stamp);
    let session_path = sessions_dir.join(session_time_stamp.to_string());

    // Create the session directory
    fs::create_dir(&session_path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Failed to create session directory {}: {}",
                session_time_stamp, e
            ),
        )
    })?;

    // Defensive: Verify creation succeeded
    if !session_path.exists() || !session_path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Session directory creation reported success but directory not found",
        ));
    }

    // Step 4: Store path in state
    state.session_directory_path = Some(session_path.clone());

    // Log success for user visibility
    println!("Session directory created: {}", session_path.display());
    println!("(Session files persist for crash recovery)");

    // Assertion: Verify state was updated
    debug_assert!(
        state.session_directory_path.is_some(),
        "Session directory path should be set in state"
    );

    Ok(())
}

// Keep this code to re-use / reference later as 'memo-mode' where no path argument and cwd is home
// /// Lines - A minimal text editor for quick append-only notes
// ///
// /// # Usage
// ///   lines [FILENAME | COMMAND]
// ///
// /// # Commands
// ///   --files     Open file manager at notes directory
// ///
// /// # File Handling
// /// - Without arguments: Creates/opens yyyy_mm_dd.txt in ~/Documents/lines_editor/
// /// - With filename: Creates/opens filename_yyyy_mm_dd.txt in ~/Documents/lines_editor/
// /// - With path: Uses exact path if file exists
// fn main() -> io::Result<()> {
//     let args: Vec<String> = env::args().collect();

//     if args.len() > 1 {
//         match args[1].as_str() {
//             "files" | "--files" => {
//                 let dir_path = if args.len() > 2 {
//                     PathBuf::from(&args[2])
//                 } else {
//                     get_default_filepath(None)?
//                         .parent()
//                         .ok_or_else(|| {
//                             io::Error::new(
//                                 io::ErrorKind::NotFound,
//                                 "Could not determine parent directory",
//                             )
//                         })?
//                         .to_path_buf()
//                 };
//                 return memo_mode_open_in_file_manager(&dir_path, None);
//             }
//             _ => {}
//         }
//     }

//     // Original file editing logic...
//     let original_file_path = if args.len() > 1 {
//         let arg_path = PathBuf::from(&args[1]);
//         if arg_path.exists() {
//             arg_path
//         } else {
//             get_default_filepath(Some(&args[1]))?
//         }
//     } else {
//         get_default_filepath(None)?
//     };

//     memo_mode_mini_editor_loop(&original_file_path)
// }

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
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Check if we're in home directory
    let in_home = is_in_home_directory()?;

    println!("=== Lines Text Editor ===");
    println!("Current directory: {}", env::current_dir()?.display());
    if in_home {
        println!("Mode: Memo mode available (in home directory)");
    } else {
        println!("Mode: Full editor (not in home directory)");
    }
    println!();

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
                "--help" | "-h" | "help" => {
                    print_help();
                    Ok(())
                }
                "--version" | "-v" | "version" => {
                    println!("lines editor v0.1.0");
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
        _ => {
            // Multiple arguments - currently not supported
            eprintln!("Error: Too many arguments");
            eprintln!("Usage: lines [filename | --help]");
            std::process::exit(2);
        }
    }
}

/*
Build Notes:
Current todo steps:


All clone() heap use, or read_lie() that can be re-done in a stack based should/must be:
- Replace Clone() with Copy Trait
- Use References and Borrowing // Instead of cloning command, pass references
e.g.     original_file_path: &Path,

The main edge-case exception is readline stdin, which in theory could be replaced
by a modular system that handles a stream of input bytes. This Lines design is
so far built around the Enter terminated input, which uses heap. Ok.


? "Use SmallVec or arrayvec for Small Collections For small, bounded collections, use stack-based alternatives:"
? String Handling: For small strings, use fixed-size buffers:
? Replace String with &str or Fixed Buffers
? // Use static or pre-allocated error messages, Minimize Heap Allocations in Error Handling
? Prefer From Trait for Error Conversion

Q: can there be debug-only verbose errors, and for appliation only terse stack use?

note: messages printed to terminal are a total waste, the user never sees
them because they are lost when the TUI refreshes. terminal prints for debugging-dev
are useful (to be removed later or commented out)
user-facing invisble text is trash-code.

There should be a seriously small optional message window in the info bar


make a check and add tests and power-of-10 items
any heap allocations that can be pre-allocated?

note: line numbers not included in file-map: fskip {int /s (space)} for ...map tui to file

Q: (Is clone a heap action?) Should every clone have a pre-allocated buffer?

TODO:
it looks like save is not implemented yet
or not detected...


TODO:
there needs to be a blurb-section
in the info bar
messages are not showing on TUI,
they get scrolled past

TODO:
when you change modes,
you need to refresh the window

refresh window when:
1. you just made a change
2. entering a new N V I mode

TODO:
maybe some kind of smart-status
for not letting cursor go into the number lines?
or... if red... not writing?
(maybe latter)

todo: put a small message bar in the info bar

Todo:
check for redundant standard libraries

...
error handling reorganization...
no heap strings for production... if terse errors.

probably skip file manager...
put lines in to ff


*/
