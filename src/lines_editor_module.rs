// lines is minimal text editor
// test files in: src/tests.rs

/*
See: "diagnostic" flag for debugging inspection

# Abstract
Lines is a minimal terminal text/hex file-editor,
written from scratch in vanilla 2024-Rust with no third party crates or unsafe code,
designed for long term reliability, maintainability, adaptable modularity,
memory-footprint minimalism, safety, and security.
The scope is intentionally constrained to a few essential uses and file operations
such as insert, and delete at the character/byte level. Lines is,
by design and policy, not a "fully-featured," "feature-rich," "responsive,"
full IDE competing with Zed, Helix, vsCode, etc.

# Policies
- slim, modular, scalable, customizable, maintainable; not bloated, not inflexible, not unmaintainable, not incomprehensible, not ideological; not low-code no-code.
- line numbers (absolute or relative) are always shown
- ready-copy: for safety, there is always a read-copy of a file, even for large files
- memory and file safety is a priority
- memory-use is strongly optimized/minimized: RAM is a precious resource
- UI/TUI is strongly optimized/minimized: UI space is a precious resource
- disk-space use is not optimized/minimized: disk space is not a rare  precious resource.
- Goal: 100% memory pre-allocated
- get-needed-when-needed: only load what is needed when it is needed; never blindly whole-pre-load anything
- modular iterative chunks: Anything (potentially including posix epoch time-stamps) that do not have a 100% tautologically forever-known size must be handled in modular chunks, not blindly loaded whole.
- vanilla-Rust with no 3rd party dependencies
- no unsafe code
- Lines require safe file backups. 3x the file-size may/will be used. There is no cowboy in-place altering of only-copies of files.
- 'defensive' coding
- modularity and simplicity for maintainability and customization
- Always work with absolute paths (defensive programming)
- executable parent relative absolute paths?
- Where possible, as in legacy-mini-lines-editor, do not leave a file 'open' to read/write/append. Read what you need, when you need to, then stop reading the file, close out the read/write process so that the file is not locked or conflicted for another application or process (outside or inside of Lines).


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


6. ? Is this about ownership of variables?
- maybe: manage rust ownership to avoid heap or memory-bloat

7. manage return values:
- use null-void return values
- check non-void-null returns

8. Navigate debugging and testing on the one hand and not-dangerous conditional compilation on the other hand

9. Communicate:
- use doc strings, use comments,
- Document use-cases, edge-cases, and policies (These are project specific and cannot be telepathed from generic micro-function code. When a Mars satellite failed because one team used SI-metric units and another team did not, that problem could not have been detected by looking at, and auditing, any individual function in isolation without documentation. Breaking a process into innumerable undocumented micro-functions can make scope and policy impossible to track. To paraphrase Jack Welch: "The most dangerous thing in the world is a flawless operation that should never have been done in the first place.")


## code requires communication


*/

/*
This code is under construction! Some code below may not be correct.
Any code that violates the roles and policies is wrong or placeholder-code.


# Build plan A


Basic features to be use-able, with a design such that planned scope can be modularly added (without a need to go back and re-design & rebuild everything).

0. Forever testing and cleanup:
- works on:
-- linux,
-- android-termux,
-- BSD-MacOS,
-- windows,
-- redox,
-- etc.

- Find and remove un-used code

- Comment and document functionality

- Error handling: 'Fail and try again' aka 'catch it, log it, and move on.'
Don't crash, just move on (optional, terminal print message
-- (info-bar message)) Are there any places where an error/exception will cause a crash rather than 'catch it, log it, and move on.'

- 'Get (what is) needed, when (it is) needed.) Is anything being loaded into memory/state that does not need to be?

- tests: Is there anything that can and should be tested but is not?

- asserts: Is there anything that can and should be asserted but is not?

- reduce redundant libraries: Are there redundant library imports?

- find arbitrarily elaborate solutions -> make more maintainable

- balancing modular 'simplicity' and efficiency

- Other NASA Power-of-10-rules areas to try or test for?


1. open a read-copy file: call lines from cli with a file-path
[Done]- use function input path from argument handled by wrapper
[Done]- make timestamped session directory in exe relative abs path directory lines_data/tmp/sessions/{timestamp}/
[Done]- save read-copy of file

[Done]main() is a wrapper that handles arguments, e.g. get path from argument and feed it in, later help menu, etc.

[Done]lines_editor_module(option(path)) is the main lines application-wrapper, called by main (or called by another application into which the lines editor is added as a module)

[Done]lines_data/tmp/sessions/{timestamp}/{timestamp}_{filename}


2. save paths in state: [Done]
[Done]- original file path
[done] - readcopy file path
[done] - added timestamps (made timestamp crate) (from ff)
[done] - added abs exe parent-relative paths (From ff)


3. modular command handling:
[done]- modes ( look at ff mode/command-modules )
[done]- --help (super mvp)
?done? - any new command modules added
- add source-it
[done]- single character commands
[done]- add multiple-letter commands in Normal/visual mode: wq/~Esc, arrows, etc.

4. Cursor system: "Plus Enter" cursor system.
[done] 1. Add cursor etc. (from POC)
- int+letter, move N spaces (in POC, but backwards from vim, use vim int+letter)
[Done]2. **Add scroll down** - Increment line number, rebuild window
[Done]3. **Add scroll up** - Decrement line number, rebuild window
[Done]4. **Test** - Verify line numbers track correctly
[Done]5. scroll right (see unwrapped long lines)
[Done]6. scroll back to left
- bump boostrap starting cursor position to +3 positions ahead on the line (not 0,0) TODO

Note: for Move-Cursor & Select Characters:
- advanced move and go-to are not needed for select to work
- there may NOT be any need for fancy move and any select

5. Moving Cursor: step-move and Go-To/go to
[Done]- hjkl
[Done]- int+hjkl
- w,e,b, normal mode move
- ge (go to file end)
- gg (go to file start)
- g{line number}, absolute or relative

(maybe future, maybe out of scope)
- gh (go to the beginning of the line)
- gl (go to the end of the line)
- gw (go to word...super cool if out of scope, see Helix)

6. select (even if select doesn't do anything now) ( visual mode, works in POC)
- Select Next
- Select to delete
- Select to copy paste
- hjkl
- int+hjkl
- w,e,b, normal mode move
- v or y, c or p

7. Delete
- figure out a coherent plan for defaults and options
- mvp d delete... a char?
maybe:
MVP:[Done]
[Done]- make backspace_style_delete_noload()
[Done]- make delete_current_line_noload()

[Done]- normal mode: 'd' is deletes current line
[Done]- visual mode: 'd' acts like backspace (deletes character before cursor)
[Done]- insert mode: '-d' is also backspace-style
After MVP: When selection is available:
- visual mode: 'd' deletes selection

Note: TUI refreshes after delete action (just like inserting)
Note: there is no planned support for 'legacy delete' (deleting forward characters)
Note: no whole-file loading.


[] file & multi-file replace-all...


8. insert:
[Done]- start of insert mode:
[Done]- user types into input buffer,
[Done]- add input buffer bytes to file at mapped cursor location: changes made to file
[Done] - reload window from newly changed file
maybe add another item into state:
( maybe step to store in state last number of cursor spaces in input buffer
- move cursor to end of new addition,
- back to start of insert mode
[Done] Empty insert is newline add \n
[Done] - commands in visual -n -v -s -w -wq


9. Saving File
[Done] s/w command to save changes to original file
[Done]- first makes a backup of the original file in same parent /archive/{timestamp}_{filename} (or, thought of differently, moves the original file as is and re-names it)
[Done]- replaces the old file with the new one (copies the read-copy to the original location
[Done] - '-wq' in insert mode
- double check that this is not loading the whole file


[Done] - 'wq' in Normal/Visual mode
- 'sa' save as (important!)


10. help/instructions system
[done] - super MVP version
- effectively all possible commands should be A. in the header or, B. in the info-bar, C. in a help-menu

11. Restore/Integrate/Fix Legacy 'Memo-Mode & Append functionality'
Very practical, very useful.
- legacy mode does whole-file loading: must do by chunks
- For MVP, calling Lines with path argument in home directory launches the ultra-minimal legacy Memo-Mode which is simple and stable (that does not need to launch full-lines with state etc.)
- add 'a' append mode, which is like memo-mode
- jump to end of file
- show last part of last lines of text (last 5-10 lines, or whatever fits in top 5 lines of TUI
- whatever user types, append as a new line
- cli argument -a --append to open file into append mode


12. new file name prompt
[Done]- calling lines not in home directory should...first ask for file name?


13. new paths
[Almost Done] - calling lines with a path that does not yet exit, make those dirs and or file and launch full-lines
- Why is it making/saving an archive directory in the new directory???
- oh...archiving... new new file? ok... but check this... save-archives... maybe.


14. comment/uncomment (simple?)
- up to ~24 space
- is there comment-tag + space?
- yes? no?, flip it


15. File-insert and Extract-to-File
- export row/line (slice) of one file to a new file
- insert file-bytes into a file (rather than input-buffer, e.g. instead of big cut and paste)
- export selection to a file
- "screen shot" print TUI to file instead of terminal

16. integrate into ff (...It's File Fantastic...)

19. Byte/Hex
simple byte mode?
raw...? simple view change?
wrap...lower priority

17. Tester-Bot
- Try to make an automated testing thing-bot, that runs through what a person would input and checks the status (read-copy-file state, original file state, archived file state) at each step or otherwise routinely.
- Automate actions that are routine.
- Automate actions that are potentially error-prone, such as repeated edits, large files, large inputs, etc.

18. maybe out of scope: undo & changelogs
- MVP maybe to save a change-log, but not have undo-feature... for mvp
- z or u
- idea: save reverse operations to files that can be run back.

19. simple view bytes
How difficult would it be to do a direct hex-byte version with no character handling?


20. alternating hex & character lines


21: Wrap
- maybe wrap is out of scope for now...

22: raw...maybe out of scope

23: backend-api for another wrapper or front-end.




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
- save-as


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

fn parse_commands_for_normal_visualselect_modes(input: &str) -> (Option<char>, usize) {
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
        let (command, repeat_count) = parse_commands_for_normal_visualselect_modes(&input);

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
                full_lines_editor(Some(original_file_path), None)
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
                    // Parse "filename:line" format
                    let (file_path_str, starting_line) = if let Some(colon_pos) = arg.rfind(':') {
                        let file_part = &arg[..colon_pos];
                        let line_part = &arg[colon_pos + 1..];

                        match line_part.parse::<usize>() {
                            Ok(line_num) if line_num > 0 => (file_part.to_string(), Some(line_num)),
                            _ => (arg.to_string(), None), // Invalid line, treat whole thing as filename
                        }
                    } else {
                        (arg.to_string(), None)
                    };

                    // Treat as file/directory path
                    if in_home && !file_path_str.contains('/') && !file_path_str.contains('\\') {
                        println!("Starting memo mode with custom file: {}", file_path_str);
                        let original_file_path = get_default_filepath(Some(&file_path_str))?;
                        memo_mode_mini_editor_loop(&original_file_path)
                    } else {
                        let path = PathBuf::from(file_path_str);
                        full_lines_editor(Some(path), starting_line) // Pass starting_line
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
*/

/*
# Spec Notes:

1. copy() in Rust uses stack: So no additional preallocations needed ...right??? TODO: check this

2. Input System Architecture:

"+ Enter" System
- Lines uses Rust's standard stdin().read() on stack, not .read_line() on heap
- No direct keypress detection (no raw terminal)
- Everything is "command + Enter"
*/

use std::env;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, StdinLock, Write, stdin, stdout};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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

// for commands such as "n"
const WHOLE_COMMAND_BUFFER_SIZE: usize = 16; //

// for iterating chunks of text to be inserted into file
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
const TEXT_BUCKET_BRIGADE_CHUNKING_BUFFER_SIZE: usize = 256;

const INFOBAR_MESSAGE_BUFFER_SIZE: usize = 32;

/// Maximum number of rows (lines) in largest supported terminal
/// of which 45 can be file rows (there are 45 tui line buffers)
pub const MAX_TUI_ROWS: usize = 48;

/// Maximum number of columns (utf-8 char across) in largest supported TUI
/// of which 157 can be file text
const MAX_TUI_COLS: usize = 160;

/// Default terminal is 24 x 80
/// Default TUI text dimensions will be
/// +/- 3 header footer,
/// +/- at least 3 for line numbers
pub const DEFAULT_ROWS: usize = 24;
pub const DEFAULT_COLS: usize = 80;

const RESET: &str = "\x1b[0m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
// const GREEN: &str = "\x1b[32m";
// const BLUE: &str = "\x1b[34m";
// const BOLD: &str = "\x1b[1m";
// const ITALIC: &str = "\x1b[3m";
// const UNDERLINE: &str = "\x1b[4m";

// ============================================================================
// ERROR SECTION: ERROR HANDLING SYSTEM (start)
// ============================================================================
/*
Error Policy:
- Do not panic-crash ever.
- 'Let it fail and try again.'
- Every line in every function WILL fail eventually
if only due to hardware, radiation, power-supply, attacks, etc.
- Every failure must be handled smoothly, returning to the last
stable state so the user can choose what to do next, trying
again or not.


# Notes on Converting to LinesError:

return Err(e);
->
return Err(LinesError::Io(e));


return Err(io::Error::new(
->
return Err(LinesError::Io(io::Error::new(


let file = File::open(path)?;
->
let file = File::open(path).map_err(|e| LinesError::Io(e))?;


# Notes on 'Power of Ten Rules' Inspired
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

*/

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

    /// For use with suite of
    /// Debug-Assert, Test-Asset, Production-Catch-Handle
    GeneralAssertionCatchViolation(String),
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
            LinesError::GeneralAssertionCatchViolation(msg) => write!(f, "State error: {}", msg),
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

// ============================================================================
// (end) ERROR HANDLING SYSTEM
// ============================================================================

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

    pub const TEXT_INPUT_CHUNKS: usize = 1024;
}

// STEM values ensuring reproducibility
// Get the source that built a binary: source_it
/*
// In main.rs:
mod source_it_module;
use source_it_module::{SourcedFile, handle_sourceit_command};

// Developer explicitly lists files to embed
const SOURCE_FILES: &[SourcedFile] = &[
    SourcedFile::new("Cargo.toml", include_str!("../Cargo.toml")),
    SourcedFile::new("src/main.rs", include_str!("main.rs")),
    SourcedFile::new(
        "src/source_it_module.rs",
        include_str!("source_it_module.rs"),
    ),
    // SourcedFile::new("src/lib.rs", include_str!("lib.rs")),
    SourcedFile::new("README.md", include_str!("../README.md")),
    // SourcedFile::new("LICENSE", include_str!("../LICENSE")),
    SourcedFile::new(".gitignore", include_str!("../.gitignore")),
];

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--source".to_string()) {
        match handle_sourceit_command("my_fft_tool", None, SOURCE_FILES) {
            Ok(path) => println!("Source extracted to: {}", path.display()),
            Err(e) => eprintln!("Failed to extract source: {}", e),
        }
        return;
    }

    // Normal application logic...
}
*/

// ==================
// Movement Functions
// ==================
const WORD_MOVE_MAX_ITERATIONS: usize = 64;
// move section
// movement section

/// Checks if a byte is a syntax (non-word) character
///
/// # Syntax chars (ASCII only):
/// - Whitespace: space (0x20), tab (0x09), newline (0x0A)
/// - Symbols: ( ) , . { } < > \ / : ! #
///
/// # Safety:
/// Only checks single byte. Safe because all syntax chars are ASCII (< 0x80).
/// Multi-byte UTF-8 chars (>= 0x80) are always non-syntax (word chars).
fn is_syntax_char(byte: u8) -> Result<bool> {
    match byte {
        b' ' | b'\t' | b'\n' => Ok(true),
        b'(' | b')' | b',' | b'.' => Ok(true),
        b'{' | b'}' | b'<' | b'>' => Ok(true),
        b'\\' | b'/' | b':' | b'!' | b'#' => Ok(true),
        _ => Ok(false),
    }
}

/// Moves cursor to position BEFORE next syntax character (Vim-style 'e' command)
///
/// # Purpose
/// Implements 'e' command for word navigation. Moves cursor forward to the
/// position BEFORE the next syntax character. This positions cursor at the
/// last non-syntax character of current word.
/// Also counts newlines to allow window scrolling without rebuild.
///
/// # Algorithm
/// 1. Move forward 2 bytes (assumption: next byte is syntax, skip it)
/// 2. Loop (max 64 iterations):
///    - Peek ahead 1 byte (look at next position)
///    - If next byte is syntax OR EOF → STOP (cursor is positioned)
///    - If next byte is non-syntax → move cursor forward 1 byte, continue
///    - If newline encountered during move: increment counter
/// 3. Return final byte offset AND newline count
///
/// # Arguments
/// * `file_path` - Absolute path to file being edited
/// * `current_byte_offset` - Current cursor position (0-indexed byte offset)
/// * `file_size` - Total file size in bytes (for EOF detection)
///
/// # Returns
/// * `Ok((new_byte_offset, newlines_crossed))` where:
///   - `new_byte_offset` - Position before next syntax char (or EOF)
///   - `newlines_crossed` - Number of 0x0A bytes encountered during move
/// * `Err(LinesError)` - File read error
///
/// # Edge Cases
/// - Cursor at EOF: returns `(EOF_pos, 0)` (no movement possible)
/// - Cursor already before syntax: moves past it to next word end
/// - Multiple syntax chars in row: skips first, stops at next
/// - Crosses newline: counts it, positions before next syntax
/// - File with no syntax: moves 64+ bytes forward (iteration limit)
///
/// # Memory Safety
/// - Stack-only: 1-byte read buffer
/// - No dynamic allocation
/// - Bounded iterations (max 64)
///
/// # Defensive Programming
/// - Iteration limit prevents infinite loops (NASA Rule #2)
/// - All read errors propagated
/// - Saturating arithmetic prevents underflow/overflow
/// - Peek-ahead safely handles EOF (no buffer overflow)
/// - Byte-level syntax check safe for UTF-8
///
/// # Example
/// ```ignore
/// // File: "hello world"
/// // Cursor at byte 0 (on 'h')
/// let (new_pos, newlines) = move_word_end(path, 0, 11)?;
/// // Moves forward 2 (to 'l'), then peeks at 'l' (non-syntax)
/// // Continues moving: 'l'->'o'->space (stop before space)
/// // Returns (4, 0) - positioned on 'o'
/// ```
pub fn move_word_end(
    file_path: &Path,
    current_byte_offset: u64,
    file_size: u64,
) -> Result<(u64, usize)> {
    // Returns: (new_byte_offset, newlines_crossed)

    // =========================================================================
    // INPUT VALIDATION
    // =========================================================================

    // Debug assert: path should be valid
    debug_assert!(
        !file_path.as_os_str().is_empty(),
        "File path cannot be empty"
    );

    // Test assert: path should be valid
    #[cfg(test)]
    assert!(
        !file_path.as_os_str().is_empty(),
        "File path cannot be empty"
    );

    // Production check: path empty
    if file_path.as_os_str().is_empty() {
        return Err(LinesError::InvalidInput("File path cannot be empty".into()));
    }

    // Debug assert: offset should not exceed file size
    debug_assert!(
        current_byte_offset <= file_size,
        "Cursor offset {} exceeds file size {}",
        current_byte_offset,
        file_size
    );

    // Test assert: offset should not exceed file size
    #[cfg(test)]
    assert!(
        current_byte_offset <= file_size,
        "Cursor offset {} exceeds file size {}",
        current_byte_offset,
        file_size
    );

    // Production check: offset exceeds file size
    if current_byte_offset > file_size {
        return Err(LinesError::InvalidInput(format!(
            "Cursor offset {} exceeds file size {}",
            current_byte_offset, file_size
        )));
    }

    // =========================================================================
    // EARLY RETURN: ALREADY AT EOF
    // =========================================================================

    // If cursor already at EOF, nowhere to move to
    if current_byte_offset >= file_size {
        return Ok((current_byte_offset, 0)); // Stay at EOF, no newlines
    }

    // =========================================================================
    // OPEN FILE FOR READING
    // =========================================================================

    let mut file = File::open(file_path).map_err(|e| {
        log_error(
            &format!("Cannot open file for word end movement: {}", e),
            Some("move_word_end"),
        );
        LinesError::Io(e)
    })?;

    // =========================================================================
    // INITIALIZE STATE
    // =========================================================================

    // Pre-allocated 1-byte buffer (stack only, no allocation)
    let mut byte_buffer: [u8; 1] = [0];

    // Current position during iteration
    let mut current_pos: u64 = current_byte_offset;

    // Newline counter (for window scrolling)
    let mut newlines_crossed: usize = 0;

    // =========================================================================
    // STEP 1: MOVE FORWARD 2 BYTES (ASSUMPTION: NEXT IS SYNTAX, SKIP IT)
    // =========================================================================

    // First move: skip 1 byte
    current_pos = current_pos.saturating_add(1);
    if current_pos >= file_size {
        return Ok((current_pos, newlines_crossed));
    }

    // Second move: skip another byte (assumption: it's syntax)
    // But we need to track if it's a newline
    file.seek(io::SeekFrom::Start(current_pos)).map_err(|e| {
        log_error(
            &format!(
                "Cannot seek to byte {} for word end initial move: {}",
                current_pos, e
            ),
            Some("move_word_end"),
        );
        LinesError::Io(e)
    })?;

    match file.read(&mut byte_buffer) {
        Ok(0) => {
            // EOF at this position
            return Ok((current_pos, newlines_crossed));
        }
        Ok(1) => {
            // Track if we're moving past a newline
            if byte_buffer[0] == b'\n' {
                newlines_crossed += 1;
            }
            current_pos = current_pos.saturating_add(1);
        }
        Ok(n) => {
            let error_msg = format!(
                "Unexpected read count {} at byte {} (expected 0 or 1)",
                n, current_pos
            );
            log_error(&error_msg, Some("move_word_end"));
            return Err(LinesError::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                error_msg,
            )));
        }
        Err(e) => {
            log_error(
                &format!("Read error at byte {}: {}", current_pos, e),
                Some("move_word_end"),
            );
            return Err(LinesError::Io(e));
        }
    }

    // Check if we've gone past EOF after second move
    if current_pos >= file_size {
        return Ok((current_pos, newlines_crossed));
    }

    // =========================================================================
    // MAIN LOOP: PEEK AHEAD UNTIL NEXT BYTE IS SYNTAX
    // =========================================================================

    let mut iteration: usize = 0;

    loop {
        // Defensive: Check iteration limit
        if iteration >= WORD_MOVE_MAX_ITERATIONS {
            // Hit iteration limit - stop here
            log_error(
                &format!(
                    "Word end movement hit iteration limit at byte {}",
                    current_pos
                ),
                Some("move_word_end"),
            );
            return Ok((current_pos, newlines_crossed));
        }

        iteration += 1;

        // ===================================================================
        // PEEK AHEAD TO NEXT BYTE (BEFORE MOVING)
        // ===================================================================

        // Calculate next position
        let next_pos = current_pos.saturating_add(1);

        // Check if next position would be past EOF
        if next_pos >= file_size {
            // Next byte would be past EOF - stop here (cursor at current position)
            return Ok((current_pos, newlines_crossed));
        }

        // ===================================================================
        // READ NEXT BYTE (PEEK AHEAD)
        // ===================================================================

        // Seek to next position
        file.seek(io::SeekFrom::Start(next_pos)).map_err(|e| {
            log_error(
                &format!("Cannot seek to byte {} for word end peek: {}", next_pos, e),
                Some("move_word_end"),
            );
            LinesError::Io(e)
        })?;

        // Read one byte
        match file.read(&mut byte_buffer) {
            Ok(0) => {
                // EOF at next position - stop here
                return Ok((current_pos, newlines_crossed));
            }
            Ok(1) => {
                // Got one byte - check if it's syntax
                let next_byte = byte_buffer[0];

                match is_syntax_char(next_byte) {
                    Ok(true) => {
                        // Next byte IS syntax - STOP HERE (cursor stays before it)
                        return Ok((current_pos, newlines_crossed));
                    }
                    Ok(false) => {
                        // Next byte is non-syntax - move cursor forward to it
                        current_pos = next_pos;

                        // Check if we just moved through a newline
                        if next_byte == b'\n' {
                            newlines_crossed += 1;
                        }

                        // Continue loop to peek at the byte after this one
                        continue;
                    }
                    Err(e) => {
                        // Error checking syntax (shouldn't happen, but handle it)
                        log_error(
                            &format!("Error checking syntax at byte {}: {}", next_pos, e),
                            Some("move_word_end"),
                        );
                        return Err(e);
                    }
                }
            }
            Ok(n) => {
                // Unexpected: read() returned more than 1 byte for 1-byte buffer
                let error_msg = format!(
                    "Unexpected read count {} at byte {} (expected 0 or 1)",
                    n, next_pos
                );
                log_error(&error_msg, Some("move_word_end"));
                return Err(LinesError::Io(io::Error::new(
                    io::ErrorKind::InvalidData,
                    error_msg,
                )));
            }
            Err(e) => {
                // Read error - propagate
                log_error(
                    &format!("Read error at byte {}: {}", next_pos, e),
                    Some("move_word_end"),
                );
                return Err(LinesError::Io(e));
            }
        }
    }
}
/// Checks if the byte at a specific file position is a newline character
///
/// # Purpose
/// Safe, defensive helper function for peek-ahead operations. Used by movement
/// commands to detect when cursor would cross a line boundary without actually
/// moving or modifying state.
///
/// # Design Philosophy
/// - Single responsibility: just peek at one byte, answer yes/no
/// - Fail-safe: returns false on any error (doesn't panic or halt)
/// - Memory safe: no allocation, bounded I/O
/// - Used by: move_word_forward, move_word_end, move_word_back, and any other
///   movement that needs to detect line boundaries
///
/// # Arguments
/// * `file_path` - Absolute path to file being checked
///   - Must exist and be readable
///   - Defensive: caller responsible for validation
/// * `byte_pos` - Byte offset to check (0-indexed)
///   - Can be any value including >= file_size
///   - Defensive: out-of-bounds positions safely return false
/// * `file_size` - Total file size in bytes
///   - Used to validate byte_pos is in range
///   - If byte_pos >= file_size, returns false (EOF, not newline)
///
/// # Returns
/// * `Ok(true)` - Byte at position is 0x0A (newline)
/// * `Ok(false)` - Byte at position is not newline (any other byte or out-of-bounds)
/// * `Err(LinesError::Io)` - File operations failed (open, seek, read errors)
///
/// # Edge Cases Handled
/// - `byte_pos >= file_size` → returns `Ok(false)` (EOF is not newline)
/// - `byte_pos == file_size - 1` → reads last byte correctly
/// - File read returns 0 bytes → returns `Ok(false)` (EOF, not newline)
/// - File seek fails → returns `Err` (I/O error, logged)
/// - File read fails mid-operation → returns `Err` (I/O error, logged)
/// - Empty file (size 0) → returns `Ok(false)` (no bytes to read)
///
/// # Memory Safety
/// - Stack-only: 1-byte buffer allocated on stack
/// - No dynamic allocation
/// - No heap growth
/// - Single file open/seek/read/close per call
/// - Safe for UTF-8: newline is ASCII 0x0A, no multi-byte collision possible
///
/// # Performance
/// - Time: O(1) - single byte read (or fail fast)
/// - Space: O(1) - fixed 1-byte buffer
/// - I/O: 1 file open, 1 seek, 1 read, 1 close
/// - Suitable for calling in loops (bounded, cached pattern: peek before move)
///
/// # Defensive Programming
/// - All I/O errors logged with context
/// - No unwrap() or panic() calls
/// - Saturating arithmetic prevents overflow
/// - Early returns for out-of-bounds
/// - No assumptions about file state
///
/// # Use Cases
///
/// **Case 1: Peek-ahead in word movement**
/// ```ignore
/// // Before moving cursor, check if next byte is newline
/// if is_newline_at_position(&file_path, current_pos + 1, file_size)? {
///     // Crossed line boundary - use line nav instead
///     execute_command(state, Command::GotoLineStart)?;
///     execute_command(state, Command::MoveDown(1))?;
/// } else {
///     // Normal character movement
///     cursor.col += 1;
/// }
/// ```
///
/// **Case 2: Loop detection in move_word_forward**
/// ```ignore
/// while remaining_moves > 0 {
///     let next_pos = current_pos + 1;
///
///     if is_newline_at_position(&file_path, next_pos, file_size)? {
///         // Hit newline - stop or handle line crossing
///         return Ok((current_pos, newlines_crossed));
///     }
///
///     current_pos = next_pos;
///     remaining_moves -= 1;
/// }
/// ```
///
/// **Case 3: Integration with MoveRight command**
/// ```ignore
/// // In MoveRight loop: peek before scrolling right
/// if is_newline_at_position(&file_path, next_byte_pos, file_size)? {
///     execute_command(state, Command::GotoLineStart)?;
///     execute_command(state, Command::MoveDown(1))?;
/// } else {
///     state.horizontal_utf8txt_line_char_offset += 1;
/// }
/// ```
///
/// # Integration Points
/// - `move_word_forward()`: Peek ahead to detect line crossing
/// - `move_word_end()`: Peek in loop to stop at syntax on next line
/// - `move_word_back()`: Peek backward to detect line crossing (reverse)
/// - `Command::MoveRight`: Peek before scrolling right (existing code)
/// - Any future movement command that needs line boundary detection
///
/// # Testing Strategy
/// Test with:
/// - File with newlines at various positions
/// - Empty file (no bytes)
/// - File with no newlines (single line)
/// - File with multiple consecutive newlines
/// - Position at EOF
/// - Position past EOF
/// - Position 0 (start of file)
/// - Last byte of file (EOF-1)
/// - Read-only files
/// - Binary files containing 0x0A bytes (not text)
///
/// # Performance Characteristics
/// - Suitable for per-move peek operations
/// - Not suitable for bulk newline scanning (use count_lines_in_file for that)
/// - Cost-benefit: single byte I/O vs. avoiding line-crossing bugs
///
/// # Error Policy
/// Following project error handling philosophy:
/// - File errors propagated (caller handles via ?)
/// - Logging happens before returning error
/// - Never silently swallows errors
/// - Caller responsible for retry/recovery logic
///
/// # Future Enhancement
/// Could cache results if same file peeked repeatedly, but:
/// - File may change between edits
/// - Invalidation logic complex
/// - Current per-call overhead minimal
/// - Premature optimization: keep simple unless profiling shows issue
///
pub fn is_newline_at_position(file_path: &Path, byte_pos: u64, file_size: u64) -> Result<bool> {
    // =========================================================================
    // INPUT VALIDATION
    // =========================================================================

    // Debug assert: path should be valid
    debug_assert!(
        !file_path.as_os_str().is_empty(),
        "File path cannot be empty in is_newline_at_position"
    );

    // Test assert: path should be valid
    #[cfg(test)]
    assert!(
        !file_path.as_os_str().is_empty(),
        "File path cannot be empty in is_newline_at_position"
    );

    // Production check: path empty
    if file_path.as_os_str().is_empty() {
        return Err(LinesError::InvalidInput("File path cannot be empty".into()));
    }

    // =========================================================================
    // EARLY RETURN: OUT OF BOUNDS
    // =========================================================================

    // Defensive: If position is at or past EOF, it's not a newline
    // This is the most common "false" case
    if byte_pos >= file_size {
        return Ok(false);
    }

    // =========================================================================
    // OPEN FILE FOR READING
    // =========================================================================

    let mut file = File::open(file_path).map_err(|e| {
        log_error(
            &format!(
                "Cannot open file to check for newline at byte {}: {}",
                byte_pos, e
            ),
            Some("is_newline_at_position"),
        );
        LinesError::Io(e)
    })?;

    // =========================================================================
    // SEEK TO POSITION AND READ SINGLE BYTE
    // =========================================================================

    // Pre-allocated 1-byte buffer (stack only, no allocation)
    let mut byte_buffer: [u8; 1] = [0];

    // Seek to the position we want to check
    file.seek(io::SeekFrom::Start(byte_pos)).map_err(|e| {
        log_error(
            &format!(
                "Cannot seek to byte {} in is_newline_at_position: {}",
                byte_pos, e
            ),
            Some("is_newline_at_position"),
        );
        LinesError::Io(e)
    })?;

    // Read one byte
    match file.read(&mut byte_buffer) {
        Ok(0) => {
            // EOF reached - no byte at this position (shouldn't happen after bounds check)
            log_error(
                &format!(
                    "Unexpected EOF when reading at byte {} (file_size was {})",
                    byte_pos, file_size
                ),
                Some("is_newline_at_position"),
            );
            Ok(false)
        }
        Ok(1) => {
            // Got one byte - check if it's a newline
            // Newline is ASCII 0x0A - safe single-byte check (no UTF-8 collision)
            Ok(byte_buffer[0] == b'\n')
        }
        Ok(n) => {
            // Unexpected: read() returned more than 1 byte for 1-byte buffer
            // This should never happen in safe Rust
            let error_msg = format!(
                "Unexpected read count {} at byte {} (expected 0 or 1)",
                n, byte_pos
            );
            log_error(&error_msg, Some("is_newline_at_position"));
            Err(LinesError::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                error_msg,
            )))
        }
        Err(e) => {
            // Read error - propagate with context
            log_error(
                &format!(
                    "Read error at byte {} in is_newline_at_position: {}",
                    byte_pos, e
                ),
                Some("is_newline_at_position"),
            );
            Err(LinesError::Io(e))
        }
    }
}

// =========================
// End of Movement Functions
// =========================

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
pub fn days_to_ymd(days_since_epoch: u64) -> (u32, u32, u32) {
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
 * The attempt is to follow NASA's only-preallocated-memory rule.
 */

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
        // Internal invariant check
        //    =================================================
        // // Debug-Assert, Test-Asset, Production-Catch-Handle
        //    =================================================
        // This is not included in production builds
        // assert: only when running in a debug-build: will panic
        debug_assert!(
            self.len <= 32,
            "Internal invariant violated: length exceeds buffer size"
        );
        // This is not included in production builds
        // assert: only when running cargo test: will panic
        #[cfg(test)]
        assert!(
            self.len <= 32,
            "Internal invariant violated: length exceeds buffer size"
        );
        // Catch & Handle without panic in production
        // This IS included in production to safe-catch
        if !self.len <= 32 {
            // state.set_info_bar_message("Config error");
            return Err(LinesError::GeneralAssertionCatchViolation(
                "NOTlen<=32buf".into(),
            ));
        }

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

/// Counts total lines in file by scanning for newline characters
///
/// # Purpose
/// Single-pass linear scan through file to count newlines.
/// Used to find total line count for GotoFileEnd command.
/// Does not load entire file into memory.
///
/// # Arguments
/// * `file_path` - Absolute path to file (must exist and be readable)
///
/// # Returns
/// * `Ok((line_count, last_newline_byte_pos))` where:
///   - `line_count` - Total lines (1-indexed, 0 for empty file)
///   - `last_newline_byte_pos` - Byte offset of final \n (0-indexed), or 0 if no newlines
/// * `Err(LinesError)` - File open, read, or seek failed
///
/// # Memory Safety
/// - Stack-only: single 1-byte buffer
/// - No heap allocation during scan
/// - No file pre-loading
///
/// # Defensive Programming
/// - Bounded iteration (file size is finite)
/// - All I/O errors propagated
/// - No unwrap() calls
/// - Handles empty files gracefully
///
/// # Edge Cases
/// - Empty file (0 bytes): returns `Ok((0, 0))`
/// - File with no newlines: returns `Ok((0, 0))`
/// - File ending with newline: counted correctly
/// - File ending without newline: counted correctly (last line still exists)
///
/// # Example
/// ```ignore
/// let (total_lines, _) = count_lines_in_file(Path::new("/path/to/file.txt"))?;
/// // Now jump to last line
/// execute_command(state, Command::GotoLine(total_lines))?;
/// ```
pub fn count_lines_in_file(file_path: &Path) -> Result<(usize, u64)> {
    // =========================================================================
    // STEP 1: DEFENSIVE INPUT VALIDATION
    // =========================================================================

    // Debug assert: path should not be empty
    debug_assert!(
        !file_path.as_os_str().is_empty(),
        "File path cannot be empty"
    );

    // Test assert: path should not be empty
    #[cfg(test)]
    assert!(
        !file_path.as_os_str().is_empty(),
        "File path cannot be empty"
    );

    // Production check: path empty
    if file_path.as_os_str().is_empty() {
        return Err(LinesError::InvalidInput("File path cannot be empty".into()));
    }

    // =========================================================================
    // STEP 2: OPEN FILE FOR READING
    // =========================================================================

    let mut file = File::open(file_path).map_err(|e| {
        log_error(
            &format!("Cannot open file for line count: {}", e),
            Some("count_lines_in_file"),
        );
        LinesError::Io(e)
    })?;

    // =========================================================================
    // STEP 3: INITIALIZE STATE
    // =========================================================================

    // Pre-allocated 1-byte buffer on stack (no dynamic allocation)
    let mut byte_buffer: [u8; 1] = [0];

    // Counters for line tracking
    let mut line_count: usize = 0;
    let mut last_newline_position: u64 = 0;
    let mut current_byte_position: u64 = 0;

    // Loop iteration counter (NASA Rule #2: upper bound on loops)
    let mut iterations: usize = 0;

    // Safety limit: prevent infinite loops from filesystem corruption
    // Reasonable upper bound: 10GB file = 10,737,418,240 bytes
    // With defensive checking, we'll catch runaway loops long before this
    const MAX_ITERATIONS: usize = 10_737_418_240;

    // =========================================================================
    // STEP 4: LINEAR SCAN - READ BYTE BY BYTE
    // =========================================================================

    loop {
        // Defensive: Check iteration limit (cosmic ray protection)
        if iterations >= MAX_ITERATIONS {
            let error_msg = format!(
                "Line count exceeded maximum iterations ({}). File may be corrupted.",
                MAX_ITERATIONS
            );
            log_error(&error_msg, Some("count_lines_in_file"));
            return Err(LinesError::Io(io::Error::new(
                io::ErrorKind::Other,
                error_msg,
            )));
        }

        iterations += 1;

        // Read one byte
        match file.read(&mut byte_buffer) {
            Ok(0) => {
                // EOF reached - exit loop normally
                break;
            }
            Ok(1) => {
                // Got one byte - check if it's newline
                if byte_buffer[0] == b'\n' {
                    line_count += 1;
                    last_newline_position = current_byte_position;
                }
                current_byte_position += 1;
            }
            Ok(n) => {
                // Unexpected: read() should return 0 or 1 for 1-byte buffer
                let error_msg = format!(
                    "read() returned unexpected byte count: {} (expected 0 or 1)",
                    n
                );
                log_error(&error_msg, Some("count_lines_in_file"));
                return Err(LinesError::Io(io::Error::new(
                    io::ErrorKind::InvalidData,
                    error_msg,
                )));
            }
            Err(e) => {
                // Read error - propagate
                log_error(
                    &format!("Read error at byte {}: {}", current_byte_position, e),
                    Some("count_lines_in_file"),
                );
                return Err(LinesError::Io(e));
            }
        }
    }

    // =========================================================================
    // STEP 5: RETURN RESULTS
    // =========================================================================

    // Defensive assertion: line_count should never be negative (usize is unsigned)
    // But verify it's reasonable
    debug_assert!(
        line_count <= MAX_ITERATIONS,
        "Line count {} exceeds reasonable maximum",
        line_count
    );

    #[cfg(test)]
    assert!(
        line_count <= MAX_ITERATIONS,
        "Line count {} exceeds reasonable maximum",
        line_count
    );

    if line_count > MAX_ITERATIONS {
        return Err(LinesError::GeneralAssertionCatchViolation(
            "Line count exceeded maximum iterations".into(),
        ));
    }

    Ok((line_count, last_newline_position))
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
        "{}{}q{}uit {}s{}ave {}u{}ndo {}d{}el|{}n{}orm {}i{}ns {}v{}is {}hex{}{}{}{} r{}aw|{}cvy p{}asty|{}w{}rd,{}b{},{}e{}nd {}/{}//cmmnt {}[]{}rpt {}hjkl{}{}",
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
pub struct WindowMapStruct {
    /// Pre-allocated mapping array [row][col] -> Option<FilePosition>
    /// None means this position is empty/padding
    pub positions: [[Option<FilePosition>; MAX_TUI_COLS]; MAX_TUI_ROWS],
    /// Number of valid rows in current window
    pub valid_rows: usize,
    /// Number of valid columns in current window
    pub valid_cols: usize,
}

impl WindowMapStruct {
    /// Creates a new WindowMapStruct with all positions set to None
    pub fn new() -> Self {
        WindowMapStruct {
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
    VisualSelectMode,
    /// Visual selection mode
    PastyMode,
    /// Multi-cursor mode (ctrl+d equivalent)
    MultiCursor,
    /// Hex Edict!
    HexMode,
}

// /// Line wrap mode setting
// #[derive(Debug, Clone, Copy, PartialEq)]
// pub enum WrapMode {
//     /// Lines wrap at terminal width
//     Wrap,
//     /// Lines extend beyond terminal width (horizontal scroll)
//     NoWrap,
// }

// const TOFILE_INSERTBUFFER_CHUNK_SIZE: usize = 256;// not used

/// Represents valid user input commands and selections in Pasty mode
///
/// # Design Principle
/// This enum contains ONLY valid operations. It does NOT contain error states,
/// invalid input variants, or exception cases. Per project error handling policy:
/// - Invalid input → `Err(io::Error)`
/// - Parse failures → `Err(io::Error)`
/// - All errors propagate via `io::Result<T>`
///
/// # Variants
/// * `SelectRank(usize)` - User entered a number to select clipboard item by rank (e.g., "3")
/// * `SelectPath(PathBuf)` - User entered a filepath (e.g., "home/user/file.txt")
/// * `PageUp` - User entered "k" or "up" to page up
/// * `PageDown` - User entered "j" or "down" to page down
/// * `ClearAll` - User entered "clear" to clear entire clipboard
/// * `ClearRank(usize)` - User entered "clearN" to clear specific clipboard item (e.g., "clear3")
/// * `Back` - User entered "b" to exit Pasty mode
/// * `Empty` - User pressed Enter with no input (select most recent clipboard item)
#[derive(Debug, Clone, PartialEq)]
pub enum PastyInputPathOrCommand {
    SelectRank(usize),
    SelectPath(PathBuf),
    PageUp,
    PageDown,
    ClearAll,
    ClearRank(usize),
    Back,
    EmptyEnterFirstItem,
}

/// Renders the Pasty mode TUI display
///
/// # Purpose
/// Displays the clipboard interface with:
/// - Legend showing available commands
/// - Clipboard items with rank numbers
/// - Info bar with pagination state and messages
///
/// # Responsibilities
/// - Display ONLY, no input handling
/// - No command execution
/// - No state modification (except reading from state)
///
/// # Arguments
/// * `state` - Editor state (for info bar message, effective_rows)
/// * `sorted_files` - Pre-sorted clipboard files (newest first)
/// * `offset` - Starting index for pagination
/// * `items_per_page` - Number of items to display per page
///
/// # Returns
/// * `Ok(())` - Display rendered successfully
/// * `Err(io::Error)` - Display operation failed (e.g., stdout write error)
fn render_pasty_tui(
    state: &EditorState,
    sorted_files: &[PathBuf],
    offset: usize,
    items_per_page: usize,
) -> io::Result<()> {
    let total_count = sorted_files.len();

    // Clear screen and move cursor to top-left
    print!("\x1b[2J\x1b[H");

    // Draw legend (using existing helper)
    println!("{}", format_pasty_tui_legend()?);

    // // Padding, print 3 lines // under construction...
    // for _ in 0..3 {
    //     println!();
    // }

    // Draw clipboard items with rank numbers
    let end = (offset + items_per_page).min(total_count);

    for idx in offset..end {
        let rank = idx + 1; // 1-indexed display
        let file_path = &sorted_files[idx];
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("???");
        // println!("{}{}. {}{}", RED, rank, YELLOW, filename); // alt
        println!("{}{}. {}{}", RED, rank, RESET, filename);
    }

    // Fill remaining space with blank lines
    let items_displayed = end - offset;
    let padding_lines = items_per_page.saturating_sub(items_displayed);
    for _ in 0..padding_lines {
        println!();
    }

    // Draw info bar (using existing helper)
    let first_count_visible = if total_count > 0 { offset + 1 } else { 0 };
    let last_count_visible = end;

    // Extract message from buffer (find null terminator or use full buffer)
    let message_len = state
        .info_bar_message_buffer
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(state.info_bar_message_buffer.len());

    let message_for_infobar =
        std::str::from_utf8(&state.info_bar_message_buffer[..message_len]).unwrap_or(""); // Empty string if invalid UTF-8

    print!(
        "{}",
        format_pasty_info_bar(
            total_count,
            first_count_visible,
            last_count_visible,
            message_for_infobar // Use info_bar_message from state
        )?
    );

    io::stdout().flush()?;

    Ok(())
}

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

    // /// Line wrap setting
    // pub wrap_mode: WrapMode,
    /// Absolute path to the file being edited
    pub original_file_path: Option<PathBuf>,

    /// Absolute path to read-copy of file
    pub read_copy_path: Option<PathBuf>,

    /// Effective editing area (minus headers/footers/line numbers)
    pub effective_rows: usize,
    pub effective_cols: usize,

    // to force-reset manually clear overwrite buffers
    pub security_mode: bool,

    /// Current window buffer containing visible text
    /// Pre-allocated to FILE_TUI_WINDOW_MAP_BUFFER_SIZE
    // pub state_file_tui_window_map_buffer: [u8; FILE_TUI_WINDOW_MAP_BUFFER_SIZE],

    // pub general_use_medium_buffer: [u8; MEDIUMSIZE_GENERAL_USE_BUFFER_SIZE],

    /// Number of valid bytes in state_file_tui_window_map_buffer
    // pub filetui_windowmap_buffer_used: usize,

    /// Window to file position mapping
    pub window_map: WindowMapStruct,

    /// Cursor position in window
    pub cursor: WindowPosition,

    /// File position of top-left corner of window
    // pub window_start: FilePosition,

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

    // TODO is u64 enough?
    // TODO: Should file-position use Ribbon-external-values?
    /// Byte position in file where the top display line starts
    /// Example: Line 500 starts at byte 12048 in the file
    pub file_position_of_topline_start: u64,
    // start end for visual-mode selection
    pub file_position_of_vis_select_start: u64,
    pub file_position_of_vis_select_end: u64,
    /// For LineWrap mode: byte offset within the line where window starts
    /// Example: If line 500 has 300 chars and we're showing the 3rd wrap, this might be 211
    // pub linewrap_window_topline_startbyte_position: u64,

    /// For LineWrap mode: character offset within the line where window starts
    /// Example: Starting at character 70 of line 500
    // pub linewrap_window_topline_char_offset: usize,

    /// For NoWrap mode: horizontal character offset for all displayed lines
    /// Example: Showing characters 20-97 of each line
    pub horizontal_utf8txt_line_char_offset: usize,

    // === DISPLAY BUFFERS ===
    /// Pre-allocated buffers for each display row (45 rows × 80 chars)
    /// Each buffer holds one terminal row including line number and text
    pub utf8_txt_display_buffers: [[u8; 182]; 45],

    /// Bytes used in each display buffer
    /// Since lines can be shorter than 80 chars, we track usage
    pub display_utf8txt_buffer_lengths: [usize; 45],

    /// Hex mode cursor (byte position in file)
    /// Only used when mode == EditorMode::HexMode
    pub hex_cursor: HexCursor,

    // /// TODO: Should there be a clear-buffer method?
    // /// Pre-allocated buffer for insert mode text input
    // /// Used to capture user input before inserting into file
    // pub tofile_insert_input_chunk_buffer: [u8; TOFILE_INSERTBUFFER_CHUNK_SIZE], // not used
    /// EOF information for the currently displayed window
    /// None = EOF not visible in current window
    /// Some((file_line_of_eof, eof_tui_display_row)) = EOF position
    eof_fileline_tuirow_tuple: Option<(usize, usize)>,

    // /// TODO is this needed?
    // /// Number of valid bytes in tofile_insert_input_chunk_buffer
    // pub tofile_insertinput_chunkbuffer_used: usize, // not used
    /// short message to display in TUI, bottom bar
    pub info_bar_message_buffer: [u8; INFOBAR_MESSAGE_BUFFER_SIZE], // not used
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
            // wrap_mode: WrapMode::Wrap,
            original_file_path: None,
            read_copy_path: None,
            effective_rows,
            effective_cols,
            security_mode: false, // default setting, purpose: to force-reset manually clear overwrite buffers

            window_map: WindowMapStruct::new(),
            cursor: WindowPosition { row: 0, col: 0 },
            // window_start: FilePosition {
            //     // for Wrap mode, if that happens
            //     byte_offset: 0,
            //     line_number: 0,
            //     byte_in_line: 0,
            // },
            selection_start: None,
            changelog_path: None,
            is_modified: false,

            // === NEW FIELD INITIALIZATION ===
            // Window position tracking - start at beginning of file
            line_count_at_top_of_window: 0,
            file_position_of_topline_start: 0,

            // Clipboard/Pasty
            file_position_of_vis_select_start: 0,
            file_position_of_vis_select_end: 0,

            // linewrap_window_topline_startbyte_position: 0,
            // linewrap_window_topline_char_offset: 0,
            horizontal_utf8txt_line_char_offset: 0,

            // Display buffers - initialized to zero
            utf8_txt_display_buffers: [[0u8; 182]; 45],
            display_utf8txt_buffer_lengths: [0usize; 45],
            hex_cursor: HexCursor::new(),
            eof_fileline_tuirow_tuple: None, // Time is like a banana, it had no end...
            // total_file_lines: None,
            info_bar_message_buffer: [0u8; INFOBAR_MESSAGE_BUFFER_SIZE],
        }
    }

    /// Handles user input in Pasty mode using bucket-brigade accumulation
    ///
    /// # Purpose
    /// Reads user input from stdin (which may be longer than buffer size), accumulates
    /// it using bucket-brigade technique, and parses it into a PastyInputPathOrCommand.
    ///
    /// # Differences from Insert Mode Input Handler
    /// **Similarities:**
    /// - Uses bucket-brigade for inputs larger than buffer
    /// - Uses pre-allocated buffers only (no heap)
    /// - Bounded iteration loops for safety
    /// - Defensive error handling
    ///
    /// **Key Differences:**
    /// - **Single-line only**: No multi-line content processing (paths are single line)
    /// - **Simpler delimiter handling**: Final `\n` is ALWAYS stdin delimiter, never content
    /// - **No immediate processing**: Accumulate ALL chunks first, THEN parse complete string
    /// - **Accumulates to state buffer**: Uses `state_file_tui_window_map_buffer` for accumulation
    ///
    /// # Bucket Brigade for Pasty Mode
    ///
    /// Since file paths can exceed 256 bytes (TEXT_BUCKET_BRIGADE_CHUNKING_BUFFER_SIZE),
    /// we use bucket-brigade to accumulate input:
    ///
    /// 1. Clear `state_file_tui_window_map_buffer` (8192 bytes available)
    /// 2. Read chunks from stdin into `text_buffer` (256 bytes)
    /// 3. Copy each chunk into accumulation buffer
    /// 4. Continue until: delimiter found, EOF, buffer full, or iteration limit
    /// 5. Parse complete accumulated string
    ///
    /// **Why this works for Pasty:**
    /// - `state_file_tui_window_map_buffer` gets cleared on every TUI render
    /// - Pasty mode doesn't conflict with file window rendering
    /// - 8192 bytes is plenty for any reasonable filesystem path
    ///
    /// # Input Parsing Priority (Fixed Order)
    ///
    /// After accumulation completes, input is parsed in this EXACT order:
    ///
    /// 1. **Empty input** → `Empty` (select most recent clipboard item)
    /// 2. **Explicit commands** (checked first, take absolute priority):
    ///    - "b" → `Back`
    ///    - "k" or "up" → `PageUp`
    ///    - "j" or "down" → `PageDown`
    ///    - "clear" → `ClearAll`
    ///    - "clearN" (where N is digits) → `ClearRank(N)`
    /// 3. **Number parsing** → Try `parse::<usize>()` → If success → `SelectRank(n)`
    /// 4. **Fallback** → Treat as filepath → `SelectPath(PathBuf::from(input))`
    ///
    /// **Important:** Commands take absolute priority. User CANNOT select files
    /// literally named "b", "clear", "k", etc. This is acceptable tradeoff for
    /// command clarity.
    ///
    /// # Error Handling Policy
    ///
    /// Per project guidelines, this function does NOT return error variants in the enum.
    /// All failures return `Err(io::Error)`:
    ///
    /// * **Input too long** (exceeds 8192 bytes) → `Err(io::Error::new(InvalidInput, "input too long"))`
    /// * **Invalid UTF-8** → `Err(io::Error::new(InvalidData, "invalid UTF-8"))`
    /// * **Stdin read failure** → `Err(io::Error)` (propagated from read)
    /// * **Any unexpected failure** → `Err(io::Error::new(Other, "operation failed"))`
    ///
    /// Caller is responsible for:
    /// - Catching errors
    /// - Setting info bar message
    /// - Returning to stable state (typically stay in Pasty mode loop, re-prompt user)
    ///
    /// # Return Value
    ///
    /// * `Ok(PastyInputPathOrCommand)` - Successfully parsed valid input
    /// * `Err(io::Error)` - Input invalid, too long, or read failure occurred
    ///
    /// # Arguments
    ///
    /// * `stdin_handle` - Locked stdin for reading (mutable to read)
    /// * `text_buffer` - Pre-allocated chunk buffer (256 bytes, reused per chunk)
    ///
    /// # Safety Bounds
    ///
    /// * **Bucket brigade iterations**: Limited to `limits::TEXT_INPUT_CHUNKS`
    /// * **Accumulation buffer size**: Limited to `FILE_TUI_WINDOW_MAP_BUFFER_SIZE` (8192 bytes)
    /// * **Input validation**: All strings validated before PathBuf creation
    ///
    /// # Example Usage
    ///
    /// ```ignore
    /// // In pasty_mode() loop:
    /// match self.handle_pasty_mode_input(&mut stdin_handle, &mut text_buffer) {
    ///     Ok(PastyInputPathOrCommand::Back) => {
    ///         // Exit Pasty mode
    ///         return Ok(true);
    ///     }
    ///     Ok(PastyInputPathOrCommand::SelectPath(path)) => {
    ///         // Insert file at cursor
    ///         insert_file_at_cursor(self, &path)?;
    ///         return Ok(true);
    ///     }
    ///     Ok(other_command) => {
    ///         // Handle pagination, clear, etc.
    ///         // Stay in loop
    ///     }
    ///     Err(e) => {
    ///         self.set_info_bar_message("invalid input");
    ///         // Stay in loop, re-prompt user
    ///     }
    /// }
    /// ```
    ///
    /// # Edge Cases
    ///
    /// **Empty input (just Enter key):**
    /// - Returns `Ok(Empty)` to select most recent clipboard item
    ///
    /// **Whitespace-only input:**
    /// - Treated same as empty after trim()
    ///
    /// **Input exactly equals buffer size:**
    /// - Not overflow, processed normally
    ///
    /// **Path with spaces:**
    /// - Spaces preserved, treated as filepath
    /// - Example: "my file.txt" → `SelectPath("my file.txt")`
    ///
    /// **Ambiguous input like "123":**
    /// - Parsed as number first → `SelectRank(123)`
    /// - To force filepath interpretation, not currently supported
    /// - Future: could require prefix like "/" or "./" for paths
    ///
    /// **Files named after commands:**
    /// - Commands take priority
    /// - File named "b" cannot be selected (command "b" matches first)
    /// - This is acceptable tradeoff for command simplicity
    ///
    /// # Buffer Reuse Safety
    ///
    /// This method reuses `state_file_tui_window_map_buffer` which is also used by TUI rendering.
    /// This is safe because:
    /// - Buffer is cleared at start of this function
    /// - Buffer is cleared on every TUI render (which happens AFTER we return)
    /// - Pasty mode doesn't display file content (no conflict with window map)
    /// - Buffer size (8192) is sufficient for both uses
    ///
    /// # Defensive Programming
    ///
    /// - Pre-allocated buffers only (no dynamic allocation)
    /// - Bounded iteration loops (prevent infinite loops)
    /// - Explicit buffer overflow checks
    /// - All strings validated before use
    /// - No unwrap() or panic!() calls
    /// - Early returns on error conditions
    /// - Clear documentation of assumptions
    ///
    /// # Future Enhancements
    ///
    /// Possible improvements (out of current scope):
    /// - Path prefix requirement (`/` or `./`) to disambiguate from numbers/commands
    /// - Tab completion for file paths
    /// - History of recently used paths
    /// - Validation that path exists (currently accepted without validation)
    fn handle_pasty_mode_input(
        &mut self,
        stdin_handle: &mut StdinLock,
        text_buffer: &mut [u8; TEXT_BUCKET_BRIGADE_CHUNKING_BUFFER_SIZE],
    ) -> io::Result<PastyInputPathOrCommand> {
        // // Clear accumulation buffer before use
        // // (Defensive: ensure no stale data from previous operations)
        // for i in 0..FILE_TUI_WINDOW_MAP_BUFFER_SIZE {
        //     self.state_file_tui_window_map_buffer[i] = 0;
        // }

        // Create local accumulation buffer on stack
        // Buffer name matches FILE_TUI_WINDOW_MAP_BUFFER_SIZE constant
        // No clearing needed - accumulated_bytes bounds valid data
        let mut file_tui_windowmap_buffer = [0u8; FILE_TUI_WINDOW_MAP_BUFFER_SIZE];

        // to force-reset manually clear overwrite buffers
        if self.security_mode {
            // Clear buffer before reading
            for i in 0..FILE_TUI_WINDOW_MAP_BUFFER_SIZE {
                file_tui_windowmap_buffer[i] = 0;
            }
        }

        let mut accumulated_bytes: usize = 0;
        let mut found_delimiter = false;
        let mut chunk_count = 0;

        //  ///////////////////
        //  Bucket Brigade Loop
        //  ///////////////////

        loop {
            chunk_count += 1;

            // Safety bound: prevent infinite loops from malformed stdin
            if chunk_count > limits::TEXT_INPUT_CHUNKS {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "input too long (iteration limit)",
                ));
            }

            // Clear chunk buffer before reading
            for i in 0..TEXT_BUCKET_BRIGADE_CHUNKING_BUFFER_SIZE {
                text_buffer[i] = 0;
            }

            // Read next chunk from stdin
            let bytes_read = stdin_handle.read(text_buffer)?;

            // EOF detected
            if bytes_read == 0 {
                break;
            }

            // Check if this chunk contains the delimiter (newline)
            // In Pasty mode, final \n is ALWAYS the stdin delimiter, never content
            if text_buffer[..bytes_read].contains(&b'\n') {
                found_delimiter = true;
            }

            // Calculate how much we can safely copy to accumulation buffer
            let space_remaining = FILE_TUI_WINDOW_MAP_BUFFER_SIZE - accumulated_bytes;

            // Check for buffer overflow
            if space_remaining == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "input too long (buffer full)",
                ));
            }

            let copy_len = bytes_read.min(space_remaining);

            // Copy chunk into accumulation buffer
            for i in 0..copy_len {
                file_tui_windowmap_buffer[accumulated_bytes + i] = text_buffer[i];
            }

            accumulated_bytes += copy_len;

            // Check if we've copied less than read (buffer full)
            if copy_len < bytes_read {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "input too long (truncated)",
                ));
            }

            // Stop accumulating if:
            // 1. Delimiter found (complete input received)
            // 2. Buffer full (no more space)
            // 3. Partial read (stdin has no more immediate data)
            if found_delimiter
                || accumulated_bytes >= FILE_TUI_WINDOW_MAP_BUFFER_SIZE
                || bytes_read < TEXT_BUCKET_BRIGADE_CHUNKING_BUFFER_SIZE
            {
                break;
            }
        }

        //  ////////////////////////////
        //  Parse Accumulated Input
        //  ////////////////////////////

        // Convert bytes to UTF-8 string
        // Only process valid bytes [0..accumulated_bytes], rest is unused
        let input_str = std::str::from_utf8(&file_tui_windowmap_buffer[..accumulated_bytes])
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid UTF-8"))?;
        // Trim whitespace and newline delimiter
        let trimmed = input_str.trim();

        //  ////////////////////////////
        //  Parse with Priority Order:
        //  1. Empty
        //  2. Explicit commands
        //  3. Numbers (rank selection)
        //  4. Paths (fallback)
        //  ////////////////////////////

        // 1. Empty input → Select most recent clipboard item
        if trimmed.is_empty() {
            return Ok(PastyInputPathOrCommand::EmptyEnterFirstItem);
        }

        // 2. Explicit commands (take absolute priority)

        if trimmed == "b" || trimmed == "q" || trimmed == "n" || trimmed == "\x1b" {
            return Ok(PastyInputPathOrCommand::Back);
        }

        if trimmed == "k" || trimmed == "up" || trimmed == "\x1b[A" {
            return Ok(PastyInputPathOrCommand::PageUp);
        }

        if trimmed == "j" || trimmed == "down" || trimmed == "\x1b[B" {
            return Ok(PastyInputPathOrCommand::PageDown);
        }

        if trimmed == "clear" {
            return Ok(PastyInputPathOrCommand::ClearAll);
        }

        // Check for "clearN" pattern (e.g., "clear3")
        if trimmed.starts_with("clear") && trimmed.len() > 5 {
            let num_str = &trimmed[5..];
            if let Ok(rank) = num_str.parse::<usize>() {
                return Ok(PastyInputPathOrCommand::ClearRank(rank));
            }
            // If parse fails, fall through to path handling
            // (maybe they want a file named "clearxyz")
        }

        // 3. Try parsing as rank number
        if let Ok(rank) = trimmed.parse::<usize>() {
            return Ok(PastyInputPathOrCommand::SelectRank(rank));
        }

        // 4. Fallback: treat as filepath
        // Note: No validation that path exists - caller's responsibility
        // Note: Relative paths accepted - conversion to absolute happens elsewhere
        Ok(PastyInputPathOrCommand::SelectPath(PathBuf::from(trimmed)))
    }

    /// Handles Pasty mode - clipboard management and file insertion interface
    ///
    /// # Purpose
    /// Pasty mode provides an interactive TUI for:
    /// - Viewing clipboard items (files in clipboard directory, sorted by modified time)
    /// - Selecting clipboard item by rank number
    /// - Entering custom filepath to insert
    /// - Paginating through clipboard items
    /// - Clearing clipboard items (all or specific rank)
    ///
    /// # Control Flow
    /// ```text
    /// Loop (bounded):
    ///   1. Get clipboard files (fresh scan each iteration - no stored list)
    ///   2. Render Pasty TUI (legend, items, info bar)
    ///   3. Get user input via handle_pasty_mode_input()
    ///   4. Process input:
    ///      - Commands: execute (page, clear), stay in loop
    ///      - Back: exit mode, return to editor
    ///      - Path/Rank: insert file, exit mode
    ///   5. Handle errors: show message, stay in loop
    /// ```
    ///
    /// # Design Philosophy: No Stored List
    ///
    /// **Critical:** Clipboard files are NOT stored in a list between render and selection.
    /// Each iteration:
    /// 1. Scans clipboard directory fresh
    /// 2. Sorts by modified time
    /// 3. Displays with rank numbers
    /// 4. If user selects rank N:
    ///    - Scan again
    ///    - Sort again
    ///    - Count to item #N
    ///    - Return that path
    ///
    /// **Rationale:** Defensive programming. Files may be added/removed/modified
    /// between display and selection. Always use fresh data.
    ///
    /// # Pagination State
    ///
    /// `offset` is a local variable in this function, NOT stored in EditorState.
    /// It's transient to this Pasty mode session. When user exits and re-enters
    /// Pasty mode, pagination resets to top.
    ///
    /// **Calculated values:**
    /// - `items_per_page` = `self.effective_rows` (from editor state)
    /// - `offset` = start index for display (increments by items_per_page)
    ///
    /// # Commands and Actions
    ///
    /// | Input | Parsed As | Action | Loop Control |
    /// |-------|-----------|--------|--------------|
    /// | `""` | Empty | Select rank 1 (most recent), insert file | Exit loop |
    /// | `"3"` | SelectRank(3) | Select rank 3, insert file | Exit loop |
    /// | `"path/to/file"` | SelectPath(...) | Insert file at path | Exit loop |
    /// | `"b"` | Back | Exit Pasty mode, no insertion | Exit loop |
    /// | `"k"` or `"up"` | PageUp | Decrement offset, refresh display | Stay in loop |
    /// | `"j"` or `"down"` | PageDown | Increment offset, refresh display | Stay in loop |
    /// | `"clear"` | ClearAll | Delete all clipboard files | Stay in loop |
    /// | `"clear3"` | ClearRank(3) | Delete clipboard item rank 3 | Stay in loop |
    ///
    /// # Return Value Semantics
    ///
    /// * `Ok(true)` → Keep main editor loop running
    ///   - After successful file insertion
    ///   - After user presses 'b' (back to editor)
    ///   - Most common case
    ///
    /// * `Ok(false)` → Stop main editor loop (quit editor)
    ///   - Currently not used in Pasty mode
    ///   - Reserved for future quit commands
    ///
    /// * `Err(io::Error)` → Fatal error occurred
    ///   - Propagates to main loop
    ///   - Editor will likely exit or show error
    ///
    /// # Error Handling
    ///
    /// Follows project error handling policy: **stability over diagnostics**
    ///
    /// **Input errors** (invalid input, too long, parse failure):
    /// - Caught in match Err branch
    /// - Info bar message set: "invalid input"
    /// - Loop continues (stay in Pasty mode, re-prompt user)
    ///
    /// **File operation errors** (clipboard scan, delete, insert):
    /// - Caught and converted to info bar message
    /// - Loop continues when possible
    /// - Fatal errors (can't read clipboard dir) propagate
    ///
    /// **Rank out of range:**
    /// - Info bar message: "invalid rank"
    /// - Loop continues
    ///
    /// # Arguments
    ///
    /// * `stdin_handle` - Locked stdin for reading user input
    /// * `text_buffer` - Pre-allocated buffer for bucket-brigade input accumulation
    ///
    /// # Safety Bounds
    ///
    /// * **Loop iterations**: Limited to `limits::MAIN_EDITOR_LOOP_COMMANDS` (reuse editor limit)
    /// * **File operations**: All use io::Result error handling
    /// * **Path validation**: Paths converted to absolute before use
    /// * **Buffer management**: All buffers pre-allocated, no heap
    ///
    /// # Example Session
    ///
    /// ```text
    /// [User enters Pasty mode]
    ///
    /// Have a Pasty!! str-filepath clearcllipboard back Empty(freshest pasty)
    /// 1. shopping_list.txt
    /// 2. notes.md
    /// 3. config.toml
    /// Pasties: 3 showing 1-3 k/j:page >
    ///
    /// [User types: 2]
    /// → Selects notes.md, inserts at cursor, exits to editor
    ///
    /// [User types: k]
    /// → Pages up (offset decreases), stays in Pasty mode
    ///
    /// [User types: clear2]
    /// → Deletes notes.md, stays in Pasty mode, re-displays
    ///
    /// [User types: b]
    /// → Exits Pasty mode, no insertion
    /// ```
    ///
    /// # Integration Points
    ///
    /// **Called by:** Main editor loop when `mode == EditorMode::PastyMode`
    ///
    /// **Calls:**
    /// - `render_pasty_tui()` - Display clipboard items and legend
    /// - `handle_pasty_mode_input()` - Get and parse user input
    /// - `read_and_sort_pasty_clipboard()` - Fresh scan of clipboard directory
    /// - `insert_file_at_cursor()` - Insert selected file into document
    /// - `clear_pasty_file_clipboard()` - Delete all clipboard files
    ///
    /// **Modifies:**
    /// - `self.info_bar_message` - Error/status messages
    /// - Clipboard directory contents (via clear operations)
    /// - Document content (via insert_file_at_cursor)
    ///
    /// # Edge Cases
    ///
    /// **Empty clipboard:**
    /// - If user presses Enter or selects rank → "clipboard empty" message
    /// - Loop continues
    ///
    /// **Rank out of range:**
    /// - User types "99" but only 3 items exist → "invalid rank" message
    /// - Loop continues
    ///
    /// **File deleted between display and selection:**
    /// - User sees item, selects it, but file gone → insert fails gracefully
    /// - Error message shown, loop continues
    ///
    /// **Path doesn't exist:**
    /// - User types custom path that doesn't exist → insert_file_at_cursor handles
    /// - May show error or create file (depends on insert logic)
    ///
    /// **Pagination beyond end:**
    /// - Page down when already at last page → offset doesn't change
    /// - Page up when at top → offset stays at 0 (saturating_sub)
    ///
    /// # Future Enhancements
    ///
    /// Possible improvements (out of current scope):
    /// - Refresh command to re-scan without command
    /// - Search/filter clipboard items
    /// - Preview file contents
    /// - Multi-select for batch operations
    /// - Clipboard item metadata display (size, date)
    fn pasty_mode(
        &mut self,
        stdin_handle: &mut StdinLock,
        text_buffer: &mut [u8; TEXT_BUCKET_BRIGADE_CHUNKING_BUFFER_SIZE],
    ) -> io::Result<bool> {
        // Set mode to normal so leaving Pasty will not restart Pasty!
        // Have another Pasty!!
        self.mode = EditorMode::Normal;

        // Get clipboard directory path
        let session_dir = self.session_directory_path.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "Session directory not initialized")
        })?;
        let clipboard_dir = session_dir.join("clipboard");

        // Create clipboard directory if it doesn't exist
        if !clipboard_dir.exists() {
            fs::create_dir_all(&clipboard_dir)?;
        }

        // Pagination state (transient to this Pasty session)
        let mut offset: usize = 0;
        let items_per_page = self.effective_rows;

        // Loop iteration counter (defensive bounds)
        let mut pasty_iteration = 0;

        //  /////////////////
        //  Pasty Mode Loop
        //  /////////////////

        loop {
            pasty_iteration += 1;

            // Safety bound: prevent infinite loops
            if pasty_iteration > limits::MAIN_EDITOR_LOOP_COMMANDS {
                let _ = self.set_info_bar_message("pasty mode iteration limit");

                return Ok(true); // Exit gracefully, return to normal mode
            }

            //  ///////////////////
            //  Get Clipboard Files
            //  ///////////////////

            // Fresh scan each iteration (defensive: no stale cached list)
            let sorted_files = match read_and_sort_pasty_clipboard(&clipboard_dir) {
                Ok(files) => files,
                Err(_) => {
                    let _ = self.set_info_bar_message("clipboard read failed");
                    // Try to continue anyway with empty list
                    Vec::new()
                }
            };

            let total_count = sorted_files.len();

            // Adjust offset if it's beyond the current list
            // (Defensive: files may have been deleted since last iteration)
            if offset >= total_count && total_count > 0 {
                offset = total_count.saturating_sub(items_per_page).max(0);
            }

            //  //////////////////
            //  Render Pasty TUI
            //  //////////////////

            if let Err(_) = render_pasty_tui(self, &sorted_files, offset, items_per_page) {
                let _ = self.set_info_bar_message("display error");
                // Try to continue anyway
            }

            //  ////////////////////
            //  Get User Input
            //  ////////////////////

            let input_result = self.handle_pasty_mode_input(stdin_handle, text_buffer);

            //  ////////////////////
            //  Process Input
            //  ////////////////////

            match input_result {
                //  ////////
                //  Back Command - Exit Pasty Mode
                //  ////////
                Ok(PastyInputPathOrCommand::Back) => {
                    let _ = self.set_info_bar_message(""); // Clear any error messages
                    return Ok(true); // Exit Pasty mode, back to editor
                }

                //  ////////
                //  Empty Input - Select Most Recent (Rank 1)
                //  ////////
                Ok(PastyInputPathOrCommand::EmptyEnterFirstItem) => {
                    if sorted_files.is_empty() {
                        let _ = self.set_info_bar_message("*clipboard empty*");
                        continue; // Stay in loop
                    }

                    // Select rank 1 (most recent file)
                    let selected_path = &sorted_files[0];

                    // Insert file at cursor
                    if let Err(_) = insert_file_at_cursor(self, selected_path) {
                        let _ = self.set_info_bar_message("*insert fail*");
                        continue; // Stay in loop
                    }

                    let _ = self.set_info_bar_message(""); // Clear messages
                    return Ok(true); // Exit Pasty mode
                }

                //  ////////
                //  Select by Rank Number
                //  ////////
                Ok(PastyInputPathOrCommand::SelectRank(rank)) => {
                    // Validate rank is in range (1-indexed display, 0-indexed array)
                    if rank == 0 || rank > total_count {
                        let _ = self.set_info_bar_message("invalid rank");
                        continue; // Stay in loop
                    }

                    // Get file at this rank (convert to 0-indexed)
                    let selected_path = &sorted_files[rank - 1];

                    // Insert file at cursor
                    if let Err(_) = insert_file_at_cursor(self, selected_path) {
                        let _ = self.set_info_bar_message("*insert fail*");
                        continue; // Stay in loop
                    }

                    let _ = self.set_info_bar_message(""); // Clear messages
                    return Ok(true); // Exit Pasty mode
                }

                //  ////////
                //  Select by Path
                //  ////////
                Ok(PastyInputPathOrCommand::SelectPath(path)) => {
                    // Convert to absolute path (defensive)
                    let absolute_path = if path.is_absolute() {
                        path
                    } else {
                        // Make relative to current working directory
                        match std::env::current_dir() {
                            Ok(cwd) => cwd.join(&path),
                            Err(_) => {
                                let _ = self.set_info_bar_message("*path resolution failed*");
                                continue; // Stay in loop
                            }
                        }
                    };

                    // Insert file at cursor
                    if let Err(_) = insert_file_at_cursor(self, &absolute_path) {
                        let _ = self.set_info_bar_message("*insert failed*");
                        continue; // Stay in loop
                    }

                    let _ = self.set_info_bar_message(""); // Clear messages
                    return Ok(true); // Exit Pasty mode
                }

                //  ////////
                //  Page Up
                //  ////////
                Ok(PastyInputPathOrCommand::PageUp) => {
                    offset = offset.saturating_sub(items_per_page);
                    let _ = self.set_info_bar_message(""); // Clear any previous messages
                    continue; // Stay in loop, refresh display
                }

                //  ////////
                //  Page Down
                //  ////////
                Ok(PastyInputPathOrCommand::PageDown) => {
                    let new_offset = offset + items_per_page;
                    // Only advance if there are more items to show
                    if new_offset < total_count {
                        offset = new_offset;
                    }
                    let _ = self.set_info_bar_message(""); // Clear any previous messages
                    continue; // Stay in loop, refresh display
                }

                //  ////////
                //  Clear All Clipboard
                //  ////////
                Ok(PastyInputPathOrCommand::ClearAll) => {
                    if let Err(_) = clear_pasty_file_clipboard(&clipboard_dir) {
                        let _ = self.set_info_bar_message("*clear failed*");
                        continue; // Stay in loop
                    }

                    offset = 0; // Reset pagination
                    let _ = self.set_info_bar_message("^clipboard cleared^");
                    continue; // Stay in loop, refresh display
                }

                //  ////////
                //  Clear Specific Rank
                //  ////////
                Ok(PastyInputPathOrCommand::ClearRank(rank)) => {
                    // Validate rank is in range
                    if rank == 0 || rank > total_count {
                        let _ = self.set_info_bar_message("invalid rank");
                        continue; // Stay in loop
                    }

                    // Get file at this rank (convert to 0-indexed)
                    let file_to_delete = &sorted_files[rank - 1];

                    // Delete the file
                    if let Err(_) = fs::remove_file(file_to_delete) {
                        let _ = self.set_info_bar_message("delete failed");
                        continue; // Stay in loop
                    }

                    // Adjust offset if we deleted last item on current page
                    if offset >= total_count.saturating_sub(1) && offset > 0 {
                        offset = offset.saturating_sub(items_per_page);
                    }

                    let _ = self.set_info_bar_message("item cleared");
                    continue; // Stay in loop, refresh display
                }

                //  ////////
                //  Input Error (invalid, too long, parse failure)
                //  ////////
                Err(_) => {
                    let _ = self.set_info_bar_message("invalid input");
                    continue; // Stay in loop, re-prompt user
                }
            }
        }
        // Ok(keep_editor_loop_running)
    }

    /// Handles all input when the editor is in Hex mode.
    ///
    /// # Overview
    ///
    /// This method handles hex editor navigation and commands:
    /// 1. **Navigation** - Moving through file by bytes (h,j,k,l)
    /// 2. **Mode switching** - Return to normal/insert mode
    /// 3. **Commands** - Save, quit, etc.
    ///
    /// # Design Philosophy
    ///
    /// Hex mode is SIMPLER than insert mode because:
    /// - No text insertion (read-only for MVP)
    /// - No bucket brigade (single command per input)
    /// - No newline ambiguity (just byte navigation)
    /// - Movement is always 1 byte at a time
    ///
    /// # Navigation Commands
    ///
    /// | Input | Command | Action |
    /// |-------|---------|--------|
    /// | `h` | Move left | Previous byte |
    /// | `l` | Move right | Next byte |
    /// | `j` | Move down | Next row (26 bytes forward) |
    /// | `k` | Move up | Previous row (26 bytes backward) |
    /// | `w` | Word forward | Next 8 bytes (word boundary) |
    /// | `b` | Word backward | Previous 8 bytes |
    /// | `0` | Line start | First byte of current row |
    /// | `$` | Line end | Last byte of current row |
    /// | `gg` | File start | Byte 0 |
    /// | `G` | File end | Last byte |
    ///
    /// # Mode Commands
    ///
    /// | Input | Action | Loop Control |
    /// |-------|--------|--------------|
    /// | `-n` or `ESC` | Normal mode | Continue |
    /// | `-i` | Insert mode | Continue |
    /// | `-s` or `-w` | Save | Continue |
    /// | `-q` | Quit | **Stop** |
    /// | `-wq` | Save & Quit | **Stop** |
    ///
    /// # Return Value Semantics
    ///
    /// * `Ok(true)` → Keep editor loop running
    /// * `Ok(false)` → Exit editor (quit command)
    /// * `Err(e)` → IO error, propagate to main
    ///
    /// # Arguments
    ///
    /// * `stdin_handle` - Locked stdin for reading commands
    /// * `command_buffer` - Pre-allocated buffer for command input
    ///
    /// # Future Enhancements
    ///
    /// For full hex editor (beyond MVP):
    /// - Byte editing (type hex digits to change bytes)
    /// - Search (find byte sequences)
    /// - Copy/paste byte ranges
    /// - Undo/redo for byte changes
    ///
    /// # Defensive Programming
    ///
    /// - All movements bounded by file size
    /// - Cursor clamped to valid byte positions
    /// - No movement command can cause overflow
    /// - File size checked before each navigation
    fn handle_parse_hex_mode_input_and_commands(
        &mut self,
        stdin_handle: &mut StdinLock,
        command_buffer: &mut [u8; WHOLE_COMMAND_BUFFER_SIZE],
    ) -> Result<bool> {
        // Default: keep editor loop running
        let mut keep_editor_loop_running: bool = true;

        /*
        // === HEX DIGIT INPUT (0-9, A-F) ===
                input if input.len() == 1 && input.chars().next().unwrap().is_ascii_hexdigit() => {
                    // Single hex digit - edit byte at cursor
                    let hex_char = input.chars().next().unwrap();
                    handle_hex_digit_input(self, hex_char)?;
                }
         */

        // Clear command buffer before reading
        for i in 0..WHOLE_COMMAND_BUFFER_SIZE {
            command_buffer[i] = 0;
        }

        // Read single command (no chunking needed in hex mode)
        let bytes_read = stdin_handle.read(command_buffer)?;

        if bytes_read == 0 {
            // Empty input - just continue
            let _ = self.set_info_bar_message("*no input*");
            return Ok(true);
        }

        // Parse command from bytes
        let command_input = std::str::from_utf8(&command_buffer[..bytes_read]).unwrap_or("");
        let trimmed = command_input.trim();

        // Get file size for boundary checking
        let file_size = match &self.read_copy_path {
            Some(path) => match fs::metadata(path) {
                Ok(metadata) => metadata.len() as usize,
                Err(_) => {
                    let _ = self.set_info_bar_message("Error: Cannot read file size");
                    return Ok(true);
                }
            },
            None => {
                let _ = self.set_info_bar_message("Error: No file open");
                return Ok(true);
            }
        };

        // Defensive: Ensure cursor doesn't exceed file bounds
        if self.hex_cursor.byte_offset >= file_size && file_size > 0 {
            self.hex_cursor.byte_offset = file_size - 1;
        }

        //  ////////////////////////
        //  Parse Hex Mode Commands
        //  ////////////////////////

        match trimmed {
            // === HEX BYTE REPLACEMENT: Two hex digits ===
            trimmed
                if trimmed.len() == 2
                    && trimmed.as_bytes()[0].is_ascii_hexdigit()
                    && trimmed.as_bytes()[1].is_ascii_hexdigit() =>
            {
                let bytes = trimmed.as_bytes();
                let high = parse_hex_digit(bytes[0])?;
                let low = parse_hex_digit(bytes[1])?;
                let byte_value = (high << 4) | low;

                let file_path = self
                    .read_copy_path
                    .as_ref()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No file path"))?;

                if self.hex_cursor.byte_offset >= file_size {
                    let _ = self.set_info_bar_message("Cannot edit past EOF");
                    return Ok(true);
                }

                replace_byte_in_place(file_path, self.hex_cursor.byte_offset, byte_value)?;

                self.is_modified = true;
                if self.hex_cursor.byte_offset + 1 < file_size {
                    self.hex_cursor.byte_offset += 1;
                }

                let _ = self.set_info_bar_message("Byte written");
            }

            // "" => {
            //     // Empty enter: repeat last command
            //     match self.the_last_command.clone() {
            //         Some(cmd) => {
            //             keep_editor_loop_running = execute_command(self, cmd)?;
            //         }
            //         None => {
            //             self.set_info_bar_message("No previous command");
            //         }
            //     }
            // }
            // === MODE SWITCHING ===
            "n" | "\x1b" => {
                // Exit to normal mode
                keep_editor_loop_running = execute_command(self, Command::EnterNormalMode)?;
            }

            "i" => {
                // Exit to insert mode
                keep_editor_loop_running = execute_command(self, Command::EnterInsertMode)?;
            }

            "v" => {
                // Exit to visual mode
                keep_editor_loop_running = execute_command(self, Command::EnterVisualMode)?;
            }

            "p" => {
                // Exit to visual mode
                keep_editor_loop_running = execute_command(self, Command::EnterPastyClipboardMode)?;
            }

            // === FILE COMMANDS ===
            "s" | "w" => {
                // Save file
                keep_editor_loop_running = execute_command(self, Command::Save)?;
            }

            "wq" => {
                // Save and quit
                keep_editor_loop_running = execute_command(self, Command::SaveAndQuit)?;
            }

            "q" => {
                // Quit without saving
                keep_editor_loop_running = execute_command(self, Command::Quit)?;
            }

            // === NAVIGATION: LEFT/RIGHT (single byte) ===
            "h" => {
                // Move left (previous byte)
                if self.hex_cursor.byte_offset > 0 {
                    self.hex_cursor.byte_offset -= 1;
                } else {
                    let _ = self.set_info_bar_message("Already at start of file");
                }
            }

            "l" => {
                // Move right (next byte)
                if self.hex_cursor.byte_offset + 1 < file_size {
                    self.hex_cursor.byte_offset += 1;
                } else {
                    let _ = self.set_info_bar_message("Already at end of file");
                }
            }

            // === NAVIGATION: UP/DOWN (by newlines ===
            "k" => {
                // Move up to previous newline (backward in file)
                let file_path = self
                    .read_copy_path
                    .as_ref()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No file path"))?;

                match find_previous_newline(file_path, self.hex_cursor.byte_offset) {
                    Ok(Some(newline_pos)) => {
                        // Found a newline - move cursor to it
                        self.hex_cursor.byte_offset = newline_pos;
                        let _ = self.set_info_bar_message("Previous line");
                    }
                    Ok(None) => {
                        // No newline found - go to start of file
                        self.hex_cursor.byte_offset = 0;
                        let _ = self.set_info_bar_message("At start of file");
                    }
                    Err(e) => {
                        let _ = self.set_info_bar_message(&format!("Search error: {}", e));
                    }
                }
            }

            "j" => {
                // Move down to next newline (forward in file)
                let file_path = self
                    .read_copy_path
                    .as_ref()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No file path"))?;

                match find_next_newline(file_path, self.hex_cursor.byte_offset, file_size) {
                    Ok(Some(newline_pos)) => {
                        // Found a newline - move cursor to it
                        self.hex_cursor.byte_offset = newline_pos;
                        let _ = self.set_info_bar_message("Next line");
                    }
                    Ok(None) => {
                        // No newline found - go to end of file
                        if file_size > 0 {
                            self.hex_cursor.byte_offset = file_size - 1;
                        }
                        let _ = self.set_info_bar_message("At end of file");
                    }
                    Err(e) => {
                        let _ = self.set_info_bar_message(&format!("Search error: {}", e));
                    }
                }
            }

            // === NAVIGATION: LINE START/END ===
            "0" | "gh" => {
                // Go to start of current row
                let row = self.hex_cursor.current_row();
                self.hex_cursor.byte_offset = row * self.hex_cursor.bytes_per_row;
            }

            "$" | "gl" => {
                // Go to end of current row (or last byte if row incomplete)
                let row = self.hex_cursor.current_row();
                let row_end = (row + 1) * self.hex_cursor.bytes_per_row - 1;

                if row_end < file_size {
                    self.hex_cursor.byte_offset = row_end;
                } else if file_size > 0 {
                    // Row is incomplete - go to last byte
                    self.hex_cursor.byte_offset = file_size - 1;
                }
            }

            // === NAVIGATION: FILE START/END ===
            "gg" => {
                // Go to start of file
                self.hex_cursor.byte_offset = 0;
                let _ = self.set_info_bar_message("Start of file");
            }

            "ge" | "G" => {
                // TODO? ge is hexlic, what is G?
                // Go to end of file
                if file_size > 0 {
                    self.hex_cursor.byte_offset = file_size - 1;
                    let _ = self.set_info_bar_message("End of file");
                }
            }

            // === UNKNOWN COMMAND ===
            _ => {
                let _ = self.set_info_bar_message(&format!("Unknown hex command: {}", trimmed));
            }
        }

        Ok(keep_editor_loop_running)
    }

    /// Handles all input when the editor is in Insert mode.
    ///
    /// # Overview
    ///
    /// This method is responsible for ALL insert mode input handling, including:
    /// 1. **Command detection** - Recognizing special commands (-n, -v, -s, -w, -wq, -q, etc.)
    /// 2. **Text insertion** - Inserting user-typed text at cursor position
    /// 3. **Bucket brigade** - Handling large text input that exceeds buffer size
    /// 4. **Newline handling** - Distinguishing between content newlines and stdin delimiters
    ///
    /// # Current Design Status
    ///
    /// **This is a transitional design.** This method currently handles multiple concerns
    /// in one place to maintain clarity of the overall insert mode workflow. Future versions
    /// will likely split this into focused sub-methods:
    ///
    /// - `check_insert_mode_commands()` - Command detection and execution
    /// - `handle_text_insertion()` - Single chunk text processing
    /// - `handle_bucket_brigade()` - Multi-chunk large input processing
    /// - `process_chunk_with_newlines()` - Newline delimiter detection logic
    ///
    /// **Rationale for current single-method design:**
    /// - Insert mode workflow is complex and interconnected
    /// - Premature splitting could obscure control flow
    /// - All logic operates on same state and read_copy
    /// - Better to understand the whole before dividing safely
    ///
    /// # Bucket Brigade Mechanism
    ///
    /// The "bucket brigade" is how we handle text input larger than the buffer size
    /// (TEXT_BUCKET_BRIGADE_CHUNKING_BUFFER_SIZE). Named after firefighting bucket brigades,
    /// we pass chunks of data through a chain:
    ///
    /// 1. Read first chunk into buffer
    /// 2. Process it (insert text, handle newlines)
    /// 3. If buffer was full AND didn't end with delimiter, read another chunk
    /// 4. Repeat until: (a) buffer not full, (b) ends with delimiter, or (c) iteration limit hit
    ///
    /// **Why needed:** Users may paste large blocks of text, or pipe data from files.
    /// Without bucket brigade, we'd truncate input at buffer boundary.
    ///
    /// **Safety bound:** Limited to `limits::TEXT_INPUT_CHUNKS` iterations to prevent
    /// infinite loops from malformed stdin streams.
    ///
    /// # Stdin Delimiter Detection
    ///
    /// **Critical distinction:** When user types "hello" and presses Enter, stdin delivers "hello\n"
    /// The final `\n` is the command delimiter (Enter key), NOT part of the content.
    ///
    /// **However:** When user pastes "line1\nline2\n", those newlines ARE content.
    ///
    /// **Solution:** Skip ONLY the final newline IF:
    /// - It's at the last byte position of this chunk, AND
    /// - We're NOT continuing bucket brigade (meaning: this is end of input)
    ///
    /// # Return Value Semantics
    ///
    /// * `Ok(true)` → Keep main editor loop running (continue editing)
    ///   - Most commands (mode switches, save, empty input)
    ///   - All text insertion
    ///
    /// * `Ok(false)` → Stop main editor loop (exit editor)
    ///   - Quit command (-q)
    ///   - Save-and-quit command (-wq)
    ///
    /// * `Err(e)` → IO error occurred, propagates to main
    ///   - stdin read failure
    ///   - File operation failure
    ///   - UTF-8 parsing failure (currently ignored with unwrap_or)
    ///
    /// # Arguments
    ///
    /// * `stdin_handle` - Locked stdin for reading user input (mutable to read)
    /// * `text_buffer` - Pre-allocated buffer for text input (mutable for reuse, zeroed each read)
    ///
    /// # Insert Mode Commands
    ///
    /// These commands are recognized and executed in insert mode:
    ///
    /// | Input | Command | Action | Loop Control |
    /// |-------|---------|--------|--------------|
    /// | `-n` or `ESC` | Enter Normal Mode | Switch to normal mode | Continue |
    /// | `-v` | Enter Visual Mode | Switch to visual mode | Continue |
    /// | `-s` or `-w` | Save | Write changes to disk | Continue |
    /// | `-wq` | Save and Quit | Save and exit editor | **Stop** |
    /// | `-q` | Quit | Exit without saving | **Stop** |
    /// | `Delete key` | Delete Backspace | Delete character | Continue |
    /// | `\n` or `\r\n` | Insert Newline | Add new line | Continue |
    /// | Other text | Insert Text | Add text at cursor | Continue |
    ///
    /// # Windowmap Rebuilding
    ///
    /// After most operations, we rebuild the windowmap with `build_windowmap_nowrap()`.
    /// This updates the display mapping between file lines and screen display.
    ///
    /// **When rebuild happens:**
    /// - After mode switches (Normal/Visual)
    /// - After newline insertion (changes line count)
    /// - After every text chunk insertion (updates display)
    ///
    /// **Why immediate rebuilds:** Display must reflect edits in real-time for TUI rendering.
    ///
    /// # Control Flow
    ///
    /// ```text
    /// 1. Initialize: Set defaults, get read_copy path, clear buffer
    /// 2. Read first chunk from stdin
    /// 3. Check if it's a command:
    ///    - If command → execute and return
    ///    - If text → proceed to insertion
    /// 4. Process text chunk:
    ///    - Split on newlines
    ///    - Insert text segments
    ///    - Insert newlines (except stdin delimiter)
    ///    - Rebuild windowmap after each operation
    /// 5. Check if bucket brigade needed:
    ///    - If buffer full AND no delimiter → read more chunks
    ///    - Loop with iteration limit
    /// 6. Return loop control flag
    /// ```
    ///
    /// # Edge Cases
    ///
    /// **Empty input (bytes_read == 0):**
    /// - Return Ok(true) - equivalent to old 'continue' in main loop
    /// - Skip processing, go to next iteration
    ///
    /// **Invalid UTF-8:**
    /// - Currently: `unwrap_or("")` treats as empty string
    /// - Future: May want explicit error handling
    ///
    /// **Buffer overflow in bucket brigade:**
    /// - Protected by `limits::TEXT_INPUT_CHUNKS` bound
    /// - Prevents infinite loops from malformed stdin
    ///
    /// **Newline ambiguity (content vs delimiter):**
    /// - See "Stdin Delimiter Detection" section above
    /// - Logic in chunk processing handles this
    ///
    /// # Defensive Programming
    ///
    /// - **Pre-allocated buffers:** No dynamic allocation during input processing
    /// - **Iteration bounds:** Bucket brigade limited to prevent infinite loops
    /// - **Error propagation:** All I/O errors bubble up to caller
    /// - **Path validation:** read_copy path checked at method start (fail fast)
    /// - **Buffer clearing:** Zero buffer before each read (prevent data leakage)
    ///
    /// # Future Refactoring Considerations
    ///
    /// When splitting this method, preserve these properties:
    ///
    /// 1. **Clear ownership:** read_copy is cloned once at top, used throughout
    /// 2. **Explicit control flow:** Return values clearly indicate loop control
    /// 3. **Bounded loops:** All loops have explicit upper bounds
    /// 4. **Documented edge cases:** Delimiter detection logic must stay documented
    /// 5. **Testability:** Each sub-method should be independently testable
    ///
    /// **Warning:** The bucket brigade logic has subtle interdependencies with
    /// newline detection. When splitting, ensure the "will_continue_brigade" flag
    /// is correctly threaded through any sub-methods.
    ///
    /// # Example Usage (from main loop)
    ///
    /// ```ignore
    /// if state.mode == EditorMode::Insert {
    ///     keep_editor_loop_running = state.handle_utf8txt_insert_mode_input(
    ///         &mut stdin_handle,
    ///         &mut text_buffer
    ///     )?;
    /// }
    /// ```
    ///
    /// # See Also
    ///
    /// * `handle_normalmode_and_visualmode_input()` - Parallel method for Normal/Visual modes
    /// * `execute_command()` - Executes parsed commands
    /// * `insert_text_chunk_at_cursor_position()` - Core text insertion
    /// * `build_windowmap_nowrap()` - Display update after edits
    ///
    /// # Ownership/Borrowing
    ///
    /// * Takes `&mut self` to update editor state (mode, cursor, etc.)
    /// * Takes `&mut stdin_handle` to read input
    /// * Takes `&mut text_buffer` to reuse pre-allocated buffer
    /// * All three borrows are independent - no ownership conflicts
    /// * `read_copy` is cloned from `self.read_copy_path` to avoid borrow conflicts
    fn handle_utf8txt_insert_mode_input(
        &mut self,
        stdin_handle: &mut StdinLock,
        text_buffer: &mut [u8; TEXT_BUCKET_BRIGADE_CHUNKING_BUFFER_SIZE],
    ) -> Result<bool> {
        //  ///////////
        //  Insert Mode
        //  ///////////
        /*
        Workflow:
        A: when cwd is os home dir: always memo-mode (old version mode)
        when not in cwd is os home dir
        B: when provided a valid exsint file file path, open file
        C: when provided a new path, make that path
        D: when given new file name, make that file
        E: when given a path but no file name, ask user for file name

        // Study output of this to understand the Bucket Brigade
        fn main() -> io::Result<()> {
            println!("Type something and press Enter:");
            println!("(Program will read in 2-byte chunks)\n");

            let mut stdin = io::stdin();
            let mut buffer = [0u8; 2]; // TWO BYTE BUFFER
            let mut chunk_number = 0;
            let mut total_bytes = 0;

            loop {
                chunk_number += 1;
                let bytes_read = stdin.read(&mut buffer)?;
                if bytes_read == 0 {
                    println!("\n[EOF detected]");
                    break;
                }
                total_bytes += bytes_read;
                println!(
                    "Chunk {}: read {} bytes: {:?}",
                    chunk_number,
                    bytes_read,
                    &buffer[..bytes_read]
                );
                // Show as string if valid UTF-8
                if let Ok(s) = std::str::from_utf8(&buffer[..bytes_read]) {
                    println!("  As text: {:?}", s);
                }
                // Stop after newline
                if buffer[..bytes_read].contains(&b'\n') {
                    println!("\n[Newline detected - end of input]");
                    // break; with this commented out, prints all.
                }
            }
            println!("\nTotal bytes read: {}", total_bytes);
            println!("Total chunks: {}", chunk_number);
            Ok(())
        }
        */

        /* For another command area, also see:
        fn parsed_commands(){
        if current_mode == ... */

        // Default: keep editor loop running (will be set to false by quit commands)
        let mut keep_editor_loop_running: bool = true;

        // Get the read_copy path BEFORE the mutable borrow
        let read_copy = self
            .read_copy_path
            .clone()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No read copy path"))?;

        // Clear buffer before reading
        for i in 0..TEXT_BUCKET_BRIGADE_CHUNKING_BUFFER_SIZE {
            text_buffer[i] = 0;
        }

        // Read single command (no chunking)
        let bytes_read = stdin_handle.read(text_buffer)?;

        if bytes_read == 0 {
            // tell user: too long
            let _ = self.set_info_bar_message("*no input*");

            // continue; // as this would appear in a nested loop
            return Ok(true); // Skip/ignore oversized input, continue editing
        }

        // Parse command from bytes
        let text_input_str = std::str::from_utf8(&text_buffer[..bytes_read]).unwrap_or(""); // Ignore invalid UTF-8

        // Normal/Visual mode: parse as command
        let trimmed = text_input_str.trim();

        //  ////////////////////////
        //  Check for Commands First
        //  ////////////////////////

        // Check for exit insert mode commands
        if trimmed == "-n" || trimmed == "\x1b" {
            // \x1b is Esc key
            // Exit insert mode
            keep_editor_loop_running = execute_command(self, Command::EnterNormalMode)?;

            // Now we can mutably borrow state
            build_windowmap_nowrap(self, &read_copy)?;
        } else if trimmed == "-v" {
            keep_editor_loop_running = execute_command(self, Command::EnterVisualMode)?;

            // Now we can mutably borrow state
            build_windowmap_nowrap(self, &read_copy)?;
        } else if trimmed == "-s" || trimmed == "-w" {
            // Exit insert mode
            keep_editor_loop_running = execute_command(self, Command::Save)?;
        } else if trimmed == "-wq" {
            // Exit insert mode
            keep_editor_loop_running = execute_command(self, Command::SaveAndQuit)?;
        } else if trimmed == "-q" {
            // Exit insert mode
            keep_editor_loop_running = execute_command(self, Command::Quit)?;
        } else if trimmed == "\x1b[3~" {
            // Do nothing if delete key entered...
            keep_editor_loop_running = execute_command(self, Command::DeleteBackspace)?;
        } else if text_input_str == "\n" || text_input_str == "\r\n" {
            // note: empty isn't empty, it contains a newline
            // Empty line = newline insertion
            keep_editor_loop_running = execute_command(self, Command::InsertNewline('\n'))?;
            build_windowmap_nowrap(self, &read_copy)?; // Rebuild immediately after newline
        } else {
            //  ///////////////
            //  Text to Insert
            //  ///////////////

            // Determine if bucket brigade will continue after this chunk
            // If the chunk ends with a newline, that newline is the stdin delimiter (Enter key)
            // and we should NOT continue reading more chunks
            let ends_with_newline = bytes_read > 0 && text_buffer[bytes_read - 1] == b'\n';
            let will_continue_brigade =
                !ends_with_newline && bytes_read == TEXT_BUCKET_BRIGADE_CHUNKING_BUFFER_SIZE;

            // Process the chunk, handling multiple newlines
            let mut chunk_start = 0;

            while chunk_start < bytes_read {
                // Find next newline
                let remaining = &text_buffer[chunk_start..bytes_read];

                // STDIN DELIMITER DETECTION:
                // When user types text and presses Enter, stdin delivers: "text\n"
                // The final \n is NOT part of the intended text - it's the command delimiter
                //
                // We handle multiple newlines within a chunk (e.g., paste with \n characters)
                // but skip the FINAL newline if:
                // 1. It's at the last byte position of this chunk, AND
                // 2. We're NOT continuing to read more chunks (bucket brigade)
                //
                // Examples:
                //   "fish\n" → insert "fish", skip final \n (stdin delimiter)
                //   "a\nb\n" → insert "a", \n, "b", skip final \n
                //   "a\nb" (buffer full) → insert "a", \n, "b", continue reading

                if let Some(newline_offset) = remaining.iter().position(|&b| b == b'\n') {
                    // Calculate absolute position of this newline in the chunk
                    let newline_absolute_pos = chunk_start + newline_offset;

                    // Determine if this specific newline should be skipped
                    // (Is it the stdin delimiter at the end of input?)
                    let is_final_byte = newline_absolute_pos == (bytes_read - 1);
                    let should_skip_newline = is_final_byte && !will_continue_brigade;

                    // Found newline - insert text before it
                    if newline_offset > 0 {
                        insert_text_chunk_at_cursor_position(
                            self,
                            &read_copy,
                            &remaining[..newline_offset],
                        )?;
                        // ? Is this to res
                        build_windowmap_nowrap(self, &read_copy)?; // ← Rebuild IMMEDIATELY
                    }

                    // Insert newline ONLY if it's not the stdin delimiter
                    if !should_skip_newline {
                        execute_command(self, Command::InsertNewline('\n'))?;
                        build_windowmap_nowrap(self, &read_copy)?; // ← Rebuild IMMEDIATELY
                    }

                    // Move past the newline for next iteration
                    chunk_start += newline_offset + 1;
                } else {
                    // No more newlines - insert rest of chunk
                    if remaining.len() > 0 {
                        insert_text_chunk_at_cursor_position(self, &read_copy, remaining)?;
                        build_windowmap_nowrap(self, &read_copy)?; // ← Rebuild IMMEDIATELY
                    }
                    break;
                }
            }

            // Continue bucket-brigade if buffer is full and doesn't end with delimiter
            if will_continue_brigade {
                let mut bucket_iteration = 1;

                loop {
                    bucket_iteration += 1;

                    if bucket_iteration > limits::TEXT_INPUT_CHUNKS {
                        break;
                    }

                    let more_bytes = stdin_handle.read(text_buffer)?;

                    if more_bytes == 0 {
                        break;
                    }

                    // Process this chunk's newlines
                    let mut chunk_start = 0;

                    while chunk_start < more_bytes {
                        let remaining = &text_buffer[chunk_start..more_bytes];

                        if let Some(newline_offset) = remaining.iter().position(|&b| b == b'\n') {
                            if newline_offset > 0 {
                                insert_text_chunk_at_cursor_position(
                                    self,
                                    &read_copy,
                                    &remaining[..newline_offset],
                                )?;
                                build_windowmap_nowrap(self, &read_copy)?; // ← Rebuild
                            }

                            execute_command(self, Command::InsertNewline('\n'))?;
                            build_windowmap_nowrap(self, &read_copy)?; // ← Rebuild

                            chunk_start += newline_offset + 1;
                        } else {
                            if remaining.len() > 0 {
                                insert_text_chunk_at_cursor_position(self, &read_copy, remaining)?;
                                build_windowmap_nowrap(self, &read_copy)?; // ← Rebuild
                            }
                            break;
                        }
                    }

                    // A kind of ~halting problem
                    // Use a specific exit command: -q -n -v
                    // when changing mode or quitting, etc.
                    // stdin is a process has no end to predict
                }
            }
        }

        // clear info-bar blurbiness
        // self.set_info_bar_message("");

        Ok(keep_editor_loop_running)
    }

    // /// Parses user input into a command for Normal-Mode and Visual-Select Mode
    // ///
    // /// # Arguments
    // /// * `input` - Raw input string from user
    // /// * `current_mode` - Current editor mode for context-aware parsing
    // ///
    // /// # Returns
    // /// * `Command` - Parsed command or Command::None if invalid
    // ///
    // /// # Format
    // /// - Single char: `j` -> MoveDown(1)
    // /// - With count: `5j` -> MoveDown(5)
    // /// - Count then command: `10l` -> MoveRight(10)
    // /// - Mode commands: `i` -> EnterInsertMode
    // ///
    // /// # Examples
    // /// - "j" -> MoveDown(1)
    // /// - "5j" -> MoveDown(5)
    // /// - "10k" -> MoveUp(10)
    // /// - "3h" -> MoveLeft(3)
    // /// - "7l" -> MoveRight(7)
    // ///
    // /// Note: For other command handling, also see: full_lines_editor()
    // ///
    // pub fn parse_commands_for_normal_visualselect_modes(
    //     &mut self,
    //     input: &str,
    //     current_mode: EditorMode,
    // ) -> Command {
    //     let trimmed = input.trim();

    //     if trimmed.is_empty() {
    //         return Command::None;
    //     }

    //     // In insert mode, most keys are text, not commands
    //     if current_mode == EditorMode::Insert {
    //         // Check for escape sequences to exit insert mode
    //         if trimmed == "\x1b" || trimmed == "ESC" || trimmed == "-n" {
    //             return Command::EnterNormalMode;
    //         }

    //         // delete key
    //         if trimmed == "\x1b[3~" {
    //             return Command::None;
    //         }
    //         // // Check for special commands in insert mode
    //         // if trimmed == "-d" || trimmed == "\x1b[3~" {
    //         //     return Command::DeleteBackspace;
    //         // }
    //         // Everything else is text input (handled separately)
    //         return Command::None;
    //     }

    //     // Parse potential repeat count and command
    //     let mut chars = trimmed.chars().peekable();
    //     let mut count = 0usize;
    //     let mut command_start = 0;

    //     // // Defensive: Limit iteration on input parsing (not movement)
    //     let mut iterations = 0;

    //     // Parse numeric prefix
    //     while let Some(&ch) = chars.peek() {
    //         // Check for size of number for actions:
    //         // this might be done more cleanly but is maybe ok.
    //         // COMMAND_PARSE_MAX_CHARS is the max allowed use do*N
    //         if iterations >= limits::COMMAND_PARSE_MAX_CHARS {
    //             return Command::None; // Too long to be valid command
    //         }
    //         iterations += 1;

    //         if ch.is_ascii_digit() {
    //             count = count
    //                 .saturating_mul(10)
    //                 .saturating_add((ch as usize) - ('0' as usize));
    //             chars.next();
    //             command_start += 1;
    //         } else {
    //             break;
    //         }
    //     }

    //     // Default count to 1 if not specified
    //     if count == 0 {
    //         count = 1;
    //     }

    //     // Get the command string (everything after the number)
    //     let command_str = &trimmed[command_start..];

    //     /*
    //     For another command area, also see:
    //     ```rust
    //     fn full_lines_editor(){
    //     ...
    //     if state.mode == ...
    //     ```
    //      */
    //     if current_mode == EditorMode::Normal {
    //         match command_str {
    //             // Single character commands
    //             "h" => Command::MoveLeft(count),
    //             "\x1b[D" => Command::MoveLeft(count), // left over arrow
    //             "j" => Command::MoveDown(count),
    //             "\x1b[B" => Command::MoveDown(count), // down cast arrow -> \x1b[B
    //             "l" => Command::MoveRight(count),
    //             "\x1b[C" => Command::MoveRight(count), // starboard arrow
    //             "k" => Command::MoveUp(count),
    //             "\x1b[A" => Command::MoveUp(count), // up arrow -> \x1b[A
    //             "i" => Command::EnterInsertMode,
    //             "v" => Command::EnterVisualMode,
    //             // "c" | "y" => Command::Copyank,
    //             // Multi-character commands
    //             "wq" => Command::SaveAndQuit,
    //             "s" | "w" => Command::Save,
    //             "q" => Command::Quit,
    //             "p" | "pasty" => Command::EnterPastyClipboardMode,
    //             "hex" | "bytes" | "byte" => Command::EnterHexEditMode,
    //             // "wrap" => Command::ToggleWrap,
    //             // "gg" => Command::MoveToTop,
    //             "d" => Command::DeleteLine,
    //             "\x1b[3~" => Command::DeleteLine, // delete key -> \x1b[3~
    //             _ => Command::None,
    //         }
    //     } else if current_mode == EditorMode::VisualSelectMode {
    //         match command_str {
    //             // same moves for selection:
    //             "h" => Command::MoveLeft(count),
    //             "\x1b[D" => Command::MoveLeft(count), // left over arrow
    //             "j" => Command::MoveDown(count),
    //             "\x1b[B" => Command::MoveDown(count), // down cast arrow -> \x1b[B
    //             "l" => Command::MoveRight(count),
    //             "\x1b[C" => Command::MoveRight(count), // starboard arrow
    //             "k" => Command::MoveUp(count),
    //             "\x1b[A" => Command::MoveUp(count), // up arrow -> \x1b[A
    //             "i" => Command::EnterInsertMode,
    //             "q" => Command::Quit,
    //             "c" | "y" => Command::Copyank,
    //             "s" | "w" => Command::Save,
    //             "n" | "\x1b" => Command::EnterNormalMode,
    //             "wq" => Command::SaveAndQuit,
    //             "d" => Command::DeleteBackspace,
    //             "\x1b[3~" => Command::DeleteBackspace, // delete key -> \x1b[3~

    //             "v" | "p" | "pasty" => Command::EnterPastyClipboardMode,
    //             "hex" | "bytes" | "byte" => Command::EnterHexEditMode,
    //             // Some('p') => Command::PastyClipboard(count),
    //             // // TODO: Make These, Command::Select...
    //             // Some('w') => Command::SelectNextWord,
    //             // Some('b') => Command::SelectPreviousWordBeginning,
    //             // Some('e') => Command::SelectNextWordEnd,
    //             //
    //             // Some('h') => Command::SelectLeft(count),
    //             // Some('j') => Command::SelectDown(count),
    //             // Some('k') => Command::SelectUp(count),
    //             // Some('l') => Command::SelectRight(count),
    //             _ => Command::None,
    //         }
    //     } else {
    //         match command_str {
    //             // if current_mode == EditorMode::Insert {
    //             // This is an edge case, see above
    //             // (length limit not apply?)
    //             _ => Command::None,
    //         }
    //     }
    // }

    /// Parses user input into a command for Normal-Mode and Visual-Select Mode
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
    /// - Line jump: `g45` -> GotoLine(45)
    ///
    /// # Special Parsing: g-commands
    /// - `g` followed by digits = line jump (e.g., `g10` = line 10)
    /// - `gg`, `ge`, `gh`, `gl` = special navigation
    /// - Leading count is IGNORED for g-commands (e.g., `5g10` still goes to line 10)
    ///
    /// # Examples
    /// - "j" -> MoveDown(1)
    /// - "5j" -> MoveDown(5)
    /// - "g45" -> GotoLine(45)
    /// - "5g10" -> GotoLine(10) [count ignored]
    /// - "gg" -> GotoFileStart
    ///
    /// Note: For other command handling, also see: full_lines_editor()
    ///
    pub fn parse_commands_for_normal_visualselect_modes(
        &mut self,
        input: &str,
        current_mode: EditorMode,
    ) -> Command {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Command::None;
        }

        // In insert mode, most keys are text, not commands
        if current_mode == EditorMode::Insert {
            // Check for escape sequences to exit insert mode
            if trimmed == "\x1b" || trimmed == "ESC" || trimmed == "-n" {
                return Command::EnterNormalMode;
            }

            // delete key
            if trimmed == "\x1b[3~" {
                return Command::None;
            }
            // Everything else is text input (handled separately)
            return Command::None;
        }

        // Parse potential repeat count and command
        let mut chars = trimmed.chars().peekable();
        let mut count = 0usize;
        let mut command_start = 0;

        // Defensive: Limit iteration on input parsing (not movement)
        let mut iterations = 0;

        // Parse numeric prefix
        while let Some(&ch) = chars.peek() {
            // Check for size of number for actions:
            // this might be done more cleanly but is maybe ok.
            // COMMAND_PARSE_MAX_CHARS is the max allowed use do*N
            if iterations >= limits::COMMAND_PARSE_MAX_CHARS {
                return Command::None; // Too long to be valid command
            }
            iterations += 1;

            if ch.is_ascii_digit() {
                count = count
                    .saturating_mul(10)
                    .saturating_add((ch as usize) - ('0' as usize));
                chars.next();
                command_start += 1;
            } else {
                break;
            }
        }

        // Default count to 1 if not specified
        if count == 0 {
            count = 1;
        }

        // Get the command string (everything after the number)
        let command_str = &trimmed[command_start..];

        // =========================================================================
        // SPECIAL CASE: g-commands (line jumps and navigation)
        // =========================================================================
        // Handle 'g' prefix commands BEFORE mode-specific parsing
        // This allows both Normal and Visual modes to use same g-command logic
        //
        // g-commands:
        // - g{digits} = jump to line number (e.g., g45)
        // - gg = jump to file start
        // - ge = jump to file end
        // - gh = jump to line start
        // - gl = jump to line end
        //
        // NOTE: Leading count is IGNORED for all g-commands
        // Example: "5g10" -> GotoLine(10), not some multiple
        if command_str.starts_with('g') && command_str.len() > 1 {
            let rest = &command_str[1..];

            // Check if rest is all digits (line number jump)
            if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
                // Parse line number (defensive: use saturating operations)
                let mut line_number = 0usize;
                let mut digit_iterations = 0;

                for ch in rest.chars() {
                    // Defensive: prevent infinite loop on malformed input
                    if digit_iterations >= limits::COMMAND_PARSE_MAX_CHARS {
                        let _ = self.set_info_bar_message("Line number too long");
                        return Command::None;
                    }
                    digit_iterations += 1;

                    let digit_value = (ch as usize) - ('0' as usize);
                    line_number = line_number.saturating_mul(10).saturating_add(digit_value);
                }

                // Defensive: reject line 0 (lines are 1-indexed)
                if line_number == 0 {
                    let _ = self.set_info_bar_message("Line numbers start at 1");
                    return Command::None;
                }

                // Valid line jump command
                return Command::GotoLine(line_number);
            }

            // Check for multi-character g-commands
            match command_str {
                "gg" => return Command::GotoFileStart,
                "ge" => return Command::GotoFileLastLine,
                "gh" => return Command::GotoLineStart,
                "gl" => return Command::GotoLineEnd,
                _ => {
                    // Unknown g-command
                    let _ = self.set_info_bar_message(&format!("Unknown command: {}", command_str));
                    return Command::None;
                }
            }
        }

        /*
        For another command area, also see:
        ```rust
        fn full_lines_editor(){
        ...
        if state.mode == ...
        ```
         */

        if current_mode == EditorMode::Normal {
            match command_str {
                // Single character commands
                "h" => Command::MoveLeft(count),
                "\x1b[D" => Command::MoveLeft(count), // left over arrow
                "j" => Command::MoveDown(count),
                "\x1b[B" => Command::MoveDown(count), // down cast arrow -> \x1b[B
                "l" => Command::MoveRight(count),
                "\x1b[C" => Command::MoveRight(count), // starboard arrow
                "k" => Command::MoveUp(count),
                "\x1b[A" => Command::MoveUp(count), // up arrow -> \x1b[A

                "w" => Command::MoveWordForward(count),
                "e" => Command::MoveWordEnd(count),
                "b" => Command::MoveWordBack(count),

                "i" => Command::EnterInsertMode,
                "v" => Command::EnterVisualMode,
                // "c" | "y" => Command::Copyank,
                // Multi-character commands
                "wq" => Command::SaveAndQuit,
                "s" => Command::Save,
                "q" => Command::Quit,
                "p" | "pasty" => Command::EnterPastyClipboardMode,
                "hex" | "bytes" | "byte" => Command::EnterHexEditMode,
                // "wrap" => Command::ToggleWrap,
                // "gg" => Command::MoveToTop, // Now handled above
                "d" => Command::DeleteLine,
                "\x1b[3~" => Command::DeleteLine, // delete key -> \x1b[3~
                _ => Command::None,
            }
        } else if current_mode == EditorMode::VisualSelectMode {
            match command_str {
                // same moves for selection:
                "h" => Command::MoveLeft(count),
                "\x1b[D" => Command::MoveLeft(count), // left over arrow
                "j" => Command::MoveDown(count),
                "\x1b[B" => Command::MoveDown(count), // down cast arrow -> \x1b[B
                "l" => Command::MoveRight(count),
                "\x1b[C" => Command::MoveRight(count), // starboard arrow
                "k" => Command::MoveUp(count),
                "\x1b[A" => Command::MoveUp(count), // up arrow -> \x1b[A

                "w" => Command::MoveWordForward(count),
                "e" => Command::MoveWordEnd(count),
                "b" => Command::MoveWordBack(count),

                "i" => Command::EnterInsertMode,
                "q" => Command::Quit,
                "c" | "y" => Command::Copyank,
                "s" => Command::Save,
                "n" | "\x1b" => Command::EnterNormalMode,
                "wq" => Command::SaveAndQuit,
                "d" => Command::DeleteBackspace,
                "\x1b[3~" => Command::DeleteBackspace, // delete key -> \x1b[3~

                "v" | "p" | "pasty" => Command::EnterPastyClipboardMode,
                "hex" | "bytes" | "byte" => Command::EnterHexEditMode,
                _ => Command::None,
            }
        } else {
            match command_str {
                // if current_mode == EditorMode::Insert {
                // This is an edge case, see above
                // (length limit not apply?)
                _ => Command::None,
            }
        }
    }
    /// Handles input when in Normal or Visual mode: a wrapper for parse_commands_for_normal_visualselect_modes()
    ///
    /// Reads a command from stdin, parses it, executes it, and stores it for repeat.
    ///
    /// # Return Value Semantics
    ///
    /// This method returns a boolean that controls the main editor loop:
    ///
    /// * `Ok(true)` → keep_editor_loop_running = true → **loop continues**
    ///   - Used when: command executed normally
    ///   - Used when: input skipped (overflow, invalid)
    ///   - Used when: any case where editor should keep running
    ///
    /// * `Ok(false)` → keep_editor_loop_running = false → **loop STOPS**
    ///   - Used when: quit command executed
    ///   - Used when: editor should exit
    ///
    /// * `Err(e)` → IO error occurred, propagates to caller
    ///
    /// # Important Note
    ///
    /// Unlike the old inline code, there is no `continue` vs normal flow distinction.
    /// Every code path returns to the main loop. The boolean simply answers:
    /// "Should the editor keep running?" (true = yes, false = no)
    ///
    /// Overflow example:
    /// ```ignore
    /// // OLD: continue; (skip to next iteration)
    /// // NEW: return Ok(true); (keep loop running → goes to next iteration)
    /// ```
    ///
    /// # Input Overflow and Stdin Draining
    ///
    /// **Critical Edge Case:** When user input exceeds `WHOLE_COMMAND_BUFFER_SIZE`,
    /// stdin.read() fills the buffer and stops, leaving remaining bytes in the stdin buffer.
    ///
    /// **Example:**
    /// ```text
    /// User types: "agsgpijgpsjgpsjgpsjs\n" (30 bytes)
    /// Buffer size: 16 bytes
    ///
    /// Iteration N:
    ///   stdin.read() → "agsgpijgpsjgpsj" (16 bytes, buffer full)
    ///   Overflow detected! Set message "*input too long*"
    ///
    /// Iteration N+1 (IMMEDIATE - stdin still has data!):
    ///   stdin.read() → "gpsjs\n" (remaining 6 bytes)
    ///   NOT overflow (6 < 16)
    ///   Processes as normal command, CLEARS error message!
    /// ```
    ///
    /// **Solution:** After detecting overflow, DRAIN all remaining stdin bytes until:
    /// - We find the newline delimiter (end of user's input), OR
    /// - stdin.read() returns 0 (EOF), OR
    /// - Safety limit reached (prevent infinite drain)
    ///
    /// This ensures:
    /// 1. Error message persists (next iteration blocks on stdin, not immediate)
    /// 2. Garbage input doesn't get processed as commands
    /// 3. Editor state stays clean
    ///
    /// **Safety bound:** Drain limited to 1024 total bytes to prevent malicious/malformed
    /// stdin from causing infinite loops.
    ///
    /// ( wrapper for parse_commands_for_normal_visualselect_modes() )
    fn handle_normalmode_and_visualmode_input(
        &mut self,
        stdin_handle: &mut StdinLock,
        command_buffer: &mut [u8; WHOLE_COMMAND_BUFFER_SIZE],
    ) -> Result<bool> {
        // Clear command-buffer before reading
        for i in 0..WHOLE_COMMAND_BUFFER_SIZE {
            command_buffer[i] = 0;
        }

        // Read single command (no chunking)
        let bytes_read = stdin_handle.read(command_buffer)?;

        // clear info-bar blurbiness
        let _ = self.set_info_bar_message("");

        // If overflow, ignore and continue/skip
        // this is equivalent to loop{if X {continue};}
        if bytes_read >= WHOLE_COMMAND_BUFFER_SIZE {
            // eprintln!("OVERFLOW: bytes_read={}", bytes_read);

            // DRAIN remaining input until we hit newline or EOF
            let mut total_drained = bytes_read;
            loop {
                let more_bytes = stdin_handle.read(command_buffer)?;
                total_drained += more_bytes;

                // Stop if: no more data, or we found the newline delimiter
                if more_bytes == 0 || command_buffer[..more_bytes].contains(&b'\n') {
                    break;
                }

                // Safety: limit drain iterations
                if total_drained > 1024 {
                    break;
                }
            }

            // eprintln!("DRAINED total {} bytes", total_drained);
            let _ = self.set_info_bar_message("*input too long*");
            return Ok(true);
        }

        // Parse command as utf-8 from bytes
        // // Ignore invalid UTF-8
        let command_str = std::str::from_utf8(&command_buffer[..bytes_read]).unwrap_or("");

        // Normal/Visual mode: parse as command
        let trimmed = command_str.trim();

        let command = if trimmed.is_empty() {
            // Empty enter: repeat last command
            match self.the_last_command.clone() {
                Some(cmd) => cmd,
                None => Command::None, // No previous command
            }
        } else {
            // Normal/Visual mode: Parse this command
            self.parse_commands_for_normal_visualselect_modes(command_str, self.mode)
        };

        // Normal/Visual mode: Execute command
        let keep_editor_loop_running = execute_command(self, command.clone())?;

        // Store command for repeat (only if it's not null -> Command::None)
        if command != Command::None {
            self.the_last_command = Some(command);
        }

        Ok(keep_editor_loop_running)
    }

    /// Gets the first valid text column for cursor's current row
    ///
    /// # Returns
    /// * Column position where text starts (after line number)
    pub fn get_text_start_column(&self) -> usize {
        let line_number = self.line_count_at_top_of_window + self.cursor.row;
        calculate_line_number_width(line_number + 1) // +1 for 1-indexed display
    }

    /// Writes a message into the info bar message buffer
    ///
    /// # Purpose
    /// Safely copies a string message into the pre-allocated info bar buffer.
    /// Used to display short status messages, errors, or notifications to the user.
    ///
    /// # Arguments
    /// * `state` - Mutable reference to editor state containing the buffer
    /// * `message` - The message string to display (will be truncated if too long)
    ///
    /// # Behavior
    /// - Clears the entire buffer to zeros first (ensures null termination)
    /// - Copies message bytes up to buffer capacity
    /// - Truncates message if it exceeds buffer size
    /// - Always null-terminated (buffer pre-cleared)
    /// - Non-UTF8 bytes handled gracefully (copied as-is)
    ///
    /// # Safety
    /// - No dynamic allocation
    /// - Bounded copy prevents buffer overflow
    /// - Always leaves buffer in valid state
    ///
    /// # Example
    /// ```rust
    /// set_info_bar_message(&mut state, "File saved successfully");
    /// set_info_bar_message(&mut state, "Error: Cannot open file");
    /// set_info_bar_message(&mut state, ""); // Clear message
    /// ```
    ///
    /// # Edge Cases
    /// - Empty string: clears the message
    /// - Message too long: truncates to fit buffer
    /// - Non-ASCII: UTF-8 bytes copied directly
    fn set_info_bar_message(&mut self, message: &str) -> Result<()> {
        // ensure buffer exists and has known capacity
        //
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

        // Clear entire buffer (ensures null termination)
        self.info_bar_message_buffer = [0u8; INFOBAR_MESSAGE_BUFFER_SIZE];

        // Get message bytes
        let message_bytes = message.as_bytes();

        // Calculate how many bytes we can safely copy
        // Must leave room for null terminator (already have it from clear)
        let copy_len = message_bytes.len().min(INFOBAR_MESSAGE_BUFFER_SIZE - 1);

        // Defensive: verify we're not exceeding bounds
        debug_assert!(
            copy_len < INFOBAR_MESSAGE_BUFFER_SIZE,
            "Copy length must be less than buffer size"
        );

        // Copy message bytes into buffer
        // Upper bound on loop: copy_len is bounded by buffer size
        for i in 0..copy_len {
            self.info_bar_message_buffer[i] = message_bytes[i];
        }

        // Buffer is already null-terminated from the clear operation
        // Byte at index copy_len and beyond are guaranteed to be 0
        Ok(())
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
    pub fn clear_utf8_displaybuffers(&mut self) {
        // Defensive: Clear each buffer completely
        for row_idx in 0..45 {
            for col_idx in 0..80 {
                self.utf8_txt_display_buffers[row_idx][col_idx] = 0;
            }
            self.display_utf8txt_buffer_lengths[row_idx] = 0;
        }
    }

    /// Is this for undo change-log?
    /// Initialize changelog for the current file
    pub fn init_changelog(&mut self, original_file_path: &Path) -> io::Result<()> {
        // Put changelog next to the file: "document.txt.changelog"
        self.changelog_path = Some(original_file_path.with_extension("txt.changelog"));
        Ok(())
    }

    /// Append an edit operation to the changelog
    pub fn log_edit(&self, operation: &str) -> io::Result<()> {
        if let Some(ref log_path) = self.changelog_path {
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
        self.utf8_txt_display_buffers[row_idx][..bytes_to_write]
            .copy_from_slice(&line_bytes[..bytes_to_write]);

        Ok(bytes_to_write)
    }
}

/// Gets a timestamp string in yyyy_mm_dd format using only standard library
fn get_timestamp() -> io::Result<String> {
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let secs = time.as_secs();
    let days_since_epoch = secs / (24 * 60 * 60);

    // These arrays to handle different month lengths
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

/// Main editing loop for the lines text editor (pre-allocated buffer version)
///
/// # Arguments
/// * `original_file_path` - Path to the file being edited
///
/// # Returns
/// * `io::Result<()>` - Success or error status of the editing session
///
/// # Memory Safety
/// - Uses pre-allocated 256-byte buffer for stdin chunks
/// - Never loads entire file into memory
/// - Processes input chunk-by-chunk using bucket brigade pattern
///
/// # Behavior
/// 1. Creates file with timestamp if it doesn't exist
/// 2. Displays TUI with file path and last ~10 lines
/// 3. Enters input loop where user can:
///    - Type text and press enter - appended immediately
///    - Enter 'q', 'quit', 'exit', or 'exit()' to close editor
/// 4. After each append, refreshes TUI display
///
/// # Errors
/// Returns error if:
/// - Cannot create/access the file
/// - Cannot read user input
/// - Cannot append to file
/// - Cannot display TUI
///
/// # Example
/// ```no_run
/// let path = Path::new("notes.txt");
/// memo_mode_mini_editor_loop(&path)?;
/// ```
pub fn memo_mode_mini_editor_loop(original_file_path: &Path) -> Result<()> {
    // Pre-allocated buffer for bucket brigade stdin reading
    const STDIN_CHUNK_SIZE: usize = 256;
    const MAX_CHUNKS: usize = 1_000_000; // Safety limit to prevent infinite loops

    let mut stdin_chunk_buffer = [0u8; STDIN_CHUNK_SIZE];

    let stdin = io::stdin();
    let mut stdin_handle = stdin.lock(); // Lock stdin once for entire session

    // Create file with simple timestamp header if it doesn't exist
    if !original_file_path.exists() {
        // Simple timestamp header (no external file loading)
        let timestamp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => {
                let secs = duration.as_secs();
                format!("# Created: {} (unix timestamp)\n", secs)
            }
            Err(_) => String::from("# Created: [timestamp unavailable]\n"),
        };

        // Create file with timestamp header
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(original_file_path)?;

        file.write_all(timestamp.as_bytes())?;
        file.write_all(b"\n")?; // Blank line after header
        file.flush()?;
    }

    // Open file in append mode once (keeps handle open for session)
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(original_file_path)?;

    // Bootstrap: Display initial TUI
    build_memo_mode_tui(original_file_path)?;

    let mut chunk_counter = 0;

    // Main editor loop
    loop {
        // Defensive: prevent infinite loop
        chunk_counter += 1;
        if chunk_counter > MAX_CHUNKS {
            return Err(LinesError::Io(io::Error::new(
                io::ErrorKind::Other,
                "Maximum iteration limit exceeded",
            )));
        }

        // Clear buffer before reading (defensive: prevent data leakage)
        for i in 0..STDIN_CHUNK_SIZE {
            stdin_chunk_buffer[i] = 0;
        }

        // Read next chunk from stdin
        let bytes_read = match stdin_handle.read(&mut stdin_chunk_buffer) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                continue;
            }
        };

        //    =================================================
        // // Debug-Assert, Test-Asset, Production-Catch-Handle
        //    =================================================
        // This is not included in production builds
        // assert: only when running in a debug-build: will panic
        debug_assert!(
            bytes_read <= STDIN_CHUNK_SIZE,
            "bytes_read ({}) exceeded buffer size ({})",
            bytes_read,
            STDIN_CHUNK_SIZE
        );
        // This is not included in production builds
        // assert: only when running cargo test: will panic
        #[cfg(test)]
        assert!(
            bytes_read <= STDIN_CHUNK_SIZE,
            "bytes_read ({}) exceeded buffer size ({})",
            bytes_read,
            STDIN_CHUNK_SIZE
        );
        // Catch & Handle without panic in production
        // This IS included in production to safe-catch
        if !bytes_read <= STDIN_CHUNK_SIZE {
            // state.set_info_bar_message("Config error");
            return Err(LinesError::GeneralAssertionCatchViolation(
                "bytes_read <= STDIN_CHUNK_SIZE".into(),
            ));
        }

        // Check for exit command before writing to file
        // Only check if valid UTF-8 (don't fail on binary data)
        if let Ok(text_input_str) = std::str::from_utf8(&stdin_chunk_buffer[..bytes_read]) {
            let trimmed = text_input_str.trim();

            // Exit commands: q, quit, exit, exit()
            if trimmed == "q" || trimmed == "quit" || trimmed == "exit" || trimmed == "exit()" {
                println!("Exiting editor...");
                break;
            }
        }

        // Write chunk directly to file (bucket brigade pattern)
        let bytes_written = file.write(&stdin_chunk_buffer[..bytes_read])?;

        // Defensive assertion: all bytes should be written
        assert_eq!(
            bytes_written, bytes_read,
            "File write incomplete: wrote {} of {} bytes",
            bytes_written, bytes_read
        );

        // Flush to disk immediately (durability)
        file.flush()?;

        // Refresh TUI after append
        build_memo_mode_tui(original_file_path)?;
    }

    // Final flush before exit
    file.flush()?;

    Ok(())
}

/// Builds and displays the memo mode TUI (Text User Interface)
///
/// # Arguments
/// * `file_path` - Path to the file being edited
///
/// # Display Format
/// ```text
/// lines text editor: Type 'q' to (q)uit
/// file path -> /path/to/file.txt
///
/// [last ~10 lines of file content]
///
/// >
/// ```
///
/// # Memory Safety
/// - Uses pre-allocated 512-byte buffer
/// - Never loads entire file into memory
/// - Seeks to end of file and reads backwards
///
/// # Algorithm
/// 1. Clear screen and display header (editor name, file path)
/// 2. Read last 512 bytes of file (or entire file if smaller)
/// 3. Scan forward through buffer to find newline positions
/// 4. If ≥10 lines: display from 10th-to-last line to end
/// 5. If <10 lines: display entire buffer content
/// 6. Display prompt `> `
///
/// # Edge Cases
/// - Empty file: Shows only header and prompt
/// - File < 512 bytes: Shows entire file content
/// - Invalid UTF-8: Uses lossy conversion (shows � for invalid bytes)
/// - No newlines in buffer: Displays entire buffer as single "line"
///
/// # Returns
/// * `Ok(())` on successful display
/// * `Err(io::Error)` if file cannot be opened or read
///
/// # Example
/// ```no_run
/// # use std::path::Path;
/// # fn build_memo_mode_tui(p: &Path) -> std::io::Result<()> { Ok(()) }
/// let path = Path::new("notes.txt");
/// build_memo_mode_tui(&path)?;
/// ```
fn build_memo_mode_tui(file_path: &Path) -> io::Result<()> {
    // Pre-allocated buffer for reading file tail
    const TAIL_BUFFER_SIZE: usize = 512;
    let mut tail_buffer = [0u8; TAIL_BUFFER_SIZE];

    // Clear screen
    print!("\x1B[2J\x1B[1;1H");

    // Display header
    println!("lines text editor: Type 'q' to (q)uit");
    println!("file path -> {}", file_path.display());
    println!(); // Blank line after header

    // Open file (read-only)
    let mut file = File::open(file_path)?;

    // Get file size
    let file_size = file.metadata()?.len();

    // Handle empty file
    if file_size == 0 {
        println!("> ");
        io::stdout().flush()?;
        return Ok(());
    }

    // Calculate how many bytes to read (512 or less if file is smaller)
    let bytes_to_read = if file_size < TAIL_BUFFER_SIZE as u64 {
        file_size as usize
    } else {
        TAIL_BUFFER_SIZE
    };

    // Seek to position: file_size - bytes_to_read
    let seek_position = file_size - bytes_to_read as u64;
    file.seek(SeekFrom::Start(seek_position))?;

    // Clear buffer (defensive)
    for i in 0..TAIL_BUFFER_SIZE {
        tail_buffer[i] = 0;
    }

    // Read the tail portion
    let bytes_read = file.read(&mut tail_buffer[..bytes_to_read])?;

    // Defensive assertion
    assert_eq!(
        bytes_read, bytes_to_read,
        "File read incomplete: expected {}, got {}",
        bytes_to_read, bytes_read
    );

    // Scan forward and record newline positions
    const MAX_NEWLINES: usize = 100; // Upper bound for line counting
    let mut newline_positions = [0usize; MAX_NEWLINES];
    let mut newline_count = 0;

    for i in 0..bytes_read {
        if tail_buffer[i] == b'\n' {
            if newline_count < MAX_NEWLINES {
                newline_positions[newline_count] = i;
                newline_count += 1;
            }
        }
    }

    // Determine display start position
    let display_start = if newline_count >= 10 {
        // Find the position after the (newline_count - 10)th newline
        // This gives us the last 10 lines
        let target_newline_index = newline_count - 10;
        newline_positions[target_newline_index] + 1 // Start after that newline
    } else {
        // Less than 10 lines, show entire buffer
        0
    };

    // Convert buffer slice to string (lossy conversion for invalid UTF-8)
    let display_text = String::from_utf8_lossy(&tail_buffer[display_start..bytes_read]);

    // Display the content
    print!("{}", display_text);

    // Ensure there's a newline before prompt if content doesn't end with one
    if !tail_buffer[..bytes_read].ends_with(&[b'\n']) {
        println!();
    }

    // Display prompt
    print!("> ");
    io::stdout().flush()?;

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
pub fn get_default_filepath(custom_name: Option<&str>) -> io::Result<PathBuf> {
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
    // artifact of mod calling mod in flat-file
    use crate::lines_editor_module::limits;

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
/// - Horizontal scrolling is controlled by state.horizontal_utf8txt_line_char_offset
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
/// - `state.utf8_txt_display_buffers` - Filled with line numbers and visible text
/// - `state.display_utf8txt_buffer_lengths` - Set to bytes used per row
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
/// WindowMapStruct will map each character to its file byte position.
pub fn build_windowmap_nowrap(state: &mut EditorState, readcopy_file_path: &Path) -> Result<usize> {
    // Defensive: Validate inputs
    if !readcopy_file_path.is_absolute() {
        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "File path must be absolute",
        )));
    }

    if !readcopy_file_path.exists() {
        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File not found: {:?}", readcopy_file_path),
        )));
    }

    // Assertion: State should have valid dimensions
    debug_assert!(state.effective_rows > 0, "Effective rows must be positive");
    debug_assert!(state.effective_cols > 0, "Effective cols must be positive");

    // Clear existing buffers and map before building
    state.clear_utf8_displaybuffers();
    state.window_map.clear();

    // Clear EOF tracking - will be rediscovered if EOF appears in this window
    // Note: file state changes with every edit, must refresh
    state.eof_fileline_tuirow_tuple = None;

    // Open file for reading
    let mut file = File::open(readcopy_file_path)?;

    // Calculate byte position for the target line
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

    // Clear EOF tracking at start of rebuild
    state.eof_fileline_tuirow_tuple = None;

    // Process lines until display is full or file ends
    while current_display_row < state.effective_rows && iteration_count < limits::WINDOW_BUILD_LINES
    {
        iteration_count += 1;

        // Assertion: We should not exceed our display buffer count
        debug_assert!(current_display_row < 45, "Display row exceeds maximum");

        // Read one line from file (up to newline or MAX_LINE_BYTES)
        let (line_bytes, line_length, found_newline) =
            read_single_line(&mut file, &mut line_buffer)?;

        // // Check for end of file
        // if line_length == 0 && !found_newline {
        //     break; // End of file reached
        // }

        // Check for end of file
        if line_length == 0 && !found_newline {
            // EOF detected - record position for cursor movement boundaries

            if lines_processed > 0 {
                // We processed at least one line before hitting EOF
                // The last valid line is one before current position
                let last_valid_file_line = current_line_number.saturating_sub(1);
                let last_valid_display_row = current_display_row.saturating_sub(1);

                state.eof_fileline_tuirow_tuple = Some((
                    last_valid_file_line,   // File line number (0-indexed)
                    last_valid_display_row, // TUI display row (0-indexed)
                ));
            } else {
                // No lines processed - positioned at or past EOF from start
                // This happens with empty files or seeking past end
                state.eof_fileline_tuirow_tuple = Some((
                    current_line_number, // Current file line position (0-indexed)
                    current_display_row, // Current display row (0-indexed)
                ));
            }

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
            state.horizontal_utf8txt_line_char_offset,
            remaining_cols,
            file_byte_position,
        )?;

        // Update total buffer length for this row
        state.display_utf8txt_buffer_lengths[current_display_row] =
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
        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::Other,
            "Maximum iterations exceeded in build_windowmap_nowrap",
        )));
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
    let mut bytes_read = 0usize;
    let mut found_newline = false;
    let mut single_byte = [0u8; 1];

    // Defensive: Limit iterations
    let mut iterations = 0;

    while bytes_read < buffer.len() && iterations < limits::LINE_READ_BYTES {
        iterations += 1;

        // // Diagnostics: print bytes read so far
        // if iterations % 10 == 0 {
        //     println!("Iterations: {}, Bytes read: {}", iterations, bytes_read);
        // }

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
    // // Diagnostics: print
    // println!(
    //     "Final bytes read: {}, Found newline: {}",
    //     bytes_read, found_newline
    // );

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
/// the visible portion to the display buffer while updating WindowMapStruct.
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
///
/// note: "end of TUI" is not "end of line"
/// byte_offset: file_line_start + line_bytes.len() as u64, // ❌ Wrong
/// rustCopybyte_offset: file_line_start + byte_index as u64, // ✅ Correct
///
fn process_line_with_offset(
    state: &mut EditorState,
    row: usize,
    col_start: usize,
    line_bytes: &[u8],
    horizontal_offset: usize,
    max_cols: usize,
    file_line_start: u64,
) -> Result<usize> {
    // Defensive: Validate row index
    if row >= 45 {
        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Row {} exceeds maximum display rows", row),
        )));
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
                state.utf8_txt_display_buffers[row][col_start + bytes_written + i] = char_bytes[i];
            }

            // Update WindowMapStruct for this character position
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
        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::Other,
            "Maximum iterations exceeded in line processing",
        )));
    }

    // Assertion 9: Verify we stayed within display buffer bounds
    //
    //    =================================================
    // // Debug-Assert, Test-Asset, Production-Catch-Handle
    //    =================================================
    // // This is not included in production builds
    // assert: only when running in a debug-build: will panic
    debug_assert!(
        bytes_written <= 182,
        "Wrote {} bytes but buffer is only 182 bytes",
        bytes_written
    );
    // This is not included in production builds
    // assert: only when running cargo test: will panic
    #[cfg(test)]
    assert!(
        bytes_written <= 182,
        "Wrote {} bytes but buffer is only 182 bytes",
        bytes_written
    );
    // Catch & Handle without panic in production
    // This IS included in production to safe-catch
    if !bytes_written <= 182 {
        // state.set_info_bar_message("Config error");
        return Err(LinesError::GeneralAssertionCatchViolation(
            "!bytes_written <= 182".into(),
        ));
    }

    // Assertion 10: Verify display column stayed within bounds
    //
    //    =================================================
    // // Debug-Assert, Test-Asset, Production-Catch-Handle
    //    =================================================
    // // This is not included in production builds
    // assert: only when running in a debug-build: will panic
    debug_assert!(
        display_col <= col_start + max_cols,
        "Display column {} exceeds limit {}",
        display_col,
        col_start + max_cols
    );
    // This is not included in production builds
    // assert: only when running cargo test: will panic
    #[cfg(test)]
    assert!(
        display_col <= col_start + max_cols,
        "Display column {} exceeds limit {}",
        display_col,
        col_start + max_cols
    );
    // Catch & Handle without panic in production
    // This IS included in production to safe-catch
    if !display_col <= col_start + max_cols {
        // state.set_info_bar_message("Config error");
        return Err(LinesError::GeneralAssertionCatchViolation(
            "!dsplycol<=colstrt+mxcol".into(),
        ));
    }

    // Map one additional "virtual" position after the last character
    // This allows cursor to be positioned at "end of line"
    let eol_display_col = display_col; // The column right after last char

    // Only add if we have room in the display
    if eol_display_col < col_start + max_cols {
        let eol_file_pos = FilePosition {
            byte_offset: file_line_start + byte_index as u64, // end of TUI is not EOLine
            line_number: state.line_count_at_top_of_window + row,
            byte_in_line: byte_index, // end of TUI is not EOLine
        };

        state
            .window_map
            .set_file_position(row, eol_display_col, Some(eol_file_pos))?;
    }
    // ===== END NEW CODE =====

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
pub fn is_in_home_directory() -> io::Result<bool> {
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

            // Defensive: Verify it's a directory
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

// Stretech goal TODO: try to make non-heap stdin read...
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
pub fn prompt_for_filename() -> io::Result<String> {
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

    /// Move word forward (count times)
    /// Vim/Helix 'w' command
    MoveWordForward(usize),

    /// Move to word end (count times)
    /// Vim/Helix 'e' command
    MoveWordEnd(usize),

    /// Move word backward (count times)
    /// Vim/Helix 'b' command
    MoveWordBack(usize),

    /// Jump to absolute line number (1-indexed, as displayed)
    ///
    /// # Examples
    /// - `g1` - Go to line 1 (file start)
    /// - `g45` - Go to line 45
    /// - `g999` - Go to line 999 (or last line if file shorter)
    GotoLine(usize),

    GotoFileStart,
    GotoFileLastLine,
    GotoLineStart,
    GotoLineEnd,

    // Mode changes
    EnterInsertMode, // i
    EnterVisualMode, // v
    EnterNormalMode, // n or Esc or ??? -> Ctrl-[

    EnterPastyClipboardMode, // pasty: clipboard et al
    EnterHexEditMode,        // Hex Edith

    // Text editing
    InsertNewline(char), // Insert single \n at cursor's file-position
    // DeleteChar,          // Delete character at cursor // legacy?
    /// Delete entire line at cursor (normal mode)
    DeleteLine,

    /// Backspace-style delete (visual/insert modes)
    DeleteBackspace,

    // Select? up down left right byte count? or... to position?

    // File operations
    Save,        // s
    Quit,        // q
    SaveAndQuit, // w (write-quit)

    // Display
    // ToggleWrap, // w (in normal mode) // not in scope?

    // Cosplay for Variables
    Copyank, // c,y (in a normal mood)

    // No operation
    None,
}

/// Cleans up session directory and all its contents
///
/// # Purpose
/// Removes the session directory created for this editing session.
/// Called on normal exit (quit/save-quit) to cleanup temporary files.
///
/// # Arguments
/// * `state` - Editor state containing session directory path
///
/// # Returns
/// * `Ok(())` - Cleanup successful or no session directory to clean
/// * `Err(io::Error)` - Cleanup failed (non-fatal, logged)
///
/// # Safety
/// - Only removes directories under lines_data/tmp/sessions/
/// - Defensive checks prevent removing wrong directories
/// - Errors are logged but don't prevent exit
fn cleanup_session_directory(state: &EditorState) -> io::Result<()> {
    // Get session directory path
    let session_dir = match &state.session_directory_path {
        Some(path) => path,
        None => {
            // No session directory - nothing to clean
            return Ok(());
        }
    };

    // Defensive: Verify this is a session directory
    let path_str = session_dir.to_string_lossy();
    if !path_str.contains("lines_data") || !path_str.contains("sessions") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Refusing to delete directory that doesn't look like a session dir: {}",
                path_str
            ),
        ));
    }

    // Check if directory exists
    if !session_dir.exists() {
        // Already gone - that's fine
        return Ok(());
    }

    // Defensive: Verify it's a directory
    if !session_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Session path exists but is not a directory: {}",
                session_dir.display()
            ),
        ));
    }

    // Remove the directory and all contents
    fs::remove_dir_all(session_dir).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to remove session directory: {}", e),
        )
    })?;

    println!("Session directory cleaned up: {}", session_dir.display());

    Ok(())
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
pub fn execute_command(lines_editor_state: &mut EditorState, command: Command) -> Result<bool> {
    // Get read-copy path
    let base_edit_filepath: PathBuf = lines_editor_state
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
        // Command::MoveLeft(count) => {
        //     let read_copy = lines_editor_state
        //         .read_copy_path
        //         .clone()
        //         .ok_or_else(|| LinesError::StateError("No read-copy path".into()))?;

        //     let mut remaining_moves = count;
        //     let mut needs_rebuild = false;
        //     let mut iterations = 0;

        //     while remaining_moves > 0 && iterations < limits::CURSOR_MOVEMENT_STEPS {
        //         iterations += 1;

        //         // ===================================================================
        //         // AT COLUMN 0: Check if we should move to previous line
        //         // ===================================================================

        //         if lines_editor_state.cursor.col == 0
        //             && lines_editor_state.horizontal_utf8txt_line_char_offset == 0
        //         {
        //             // At absolute left edge of visible line
        //             // Move to end of previous line

        //             if lines_editor_state.cursor.row > 0 {
        //                 // Can move up within visible window
        //                 execute_command(lines_editor_state, Command::MoveUp(1))?;
        //                 execute_command(lines_editor_state, Command::GotoLineEnd)?;
        //                 remaining_moves -= 1;
        //                 needs_rebuild = true;
        //                 continue;
        //             } else if lines_editor_state.line_count_at_top_of_window > 0 {
        //                 // At top of window but not top of file - scroll up
        //                 execute_command(lines_editor_state, Command::MoveUp(1))?;
        //                 execute_command(lines_editor_state, Command::GotoLineEnd)?;
        //                 remaining_moves -= 1;
        //                 needs_rebuild = true;
        //                 continue;
        //             } else {
        //                 // At absolute top-left of file - can't move further
        //                 break;
        //             }
        //         }

        //         // ===================================================================
        //         // NORMAL LEFT MOVEMENT
        //         // ===================================================================

        //         if lines_editor_state.cursor.col > 0 {
        //             let cursor_moves = remaining_moves.min(lines_editor_state.cursor.col);
        //             lines_editor_state.cursor.col -= cursor_moves;
        //             remaining_moves -= cursor_moves;
        //         } else if lines_editor_state.horizontal_utf8txt_line_char_offset > 0 {
        //             let scroll_amount =
        //                 remaining_moves.min(lines_editor_state.horizontal_utf8txt_line_char_offset);
        //             lines_editor_state.horizontal_utf8txt_line_char_offset -= scroll_amount;
        //             remaining_moves -= scroll_amount;
        //             needs_rebuild = true;
        //         } else {
        //             // Shouldn't reach here due to check above, but defensive
        //             break;
        //         }
        //     }

        //     if iterations >= limits::CURSOR_MOVEMENT_STEPS {
        //         return Err(LinesError::Io(io::Error::new(
        //             io::ErrorKind::Other,
        //             "Maximum iterations exceeded in MoveLeft",
        //         )));
        //     }

        //     if needs_rebuild {
        //         build_windowmap_nowrap(lines_editor_state, &read_copy)?;
        //     }

        //     Ok(true)
        // }

        // v2
        Command::MoveLeft(count) => {
            // let read_copy = lines_editor_state
            //     .read_copy_path
            //     .clone()
            //     .ok_or_else(|| LinesError::StateError("No read-copy path".into()))?;

            // Vim-like behavior: move cursor left, scroll window if at edge
            let mut remaining_moves = count;
            let mut needs_rebuild = false;

            // Defensive: Limit iterations to prevent infinite loops
            let mut iterations = 0;

            while remaining_moves > 0 && iterations < limits::CURSOR_MOVEMENT_STEPS {
                iterations += 1;

                // ===================================================================
                // CHECK IF AT START OF LINE DATA (SAFER THAN PEEKING AT BYTES)
                // ===================================================================
                // Strategy: Look at position to the left in window map
                // If it's None, we're at the start of line data → move to previous line

                if lines_editor_state.cursor.col > 0 {
                    // Check if position to our left has file data
                    let left_col = lines_editor_state.cursor.col - 1;

                    match lines_editor_state
                        .window_map
                        .get_file_position(lines_editor_state.cursor.row, left_col)
                    {
                        Ok(None) => {
                            // Position to left is empty (no file data)
                            // We're at the start of line content
                            // Move to end of previous line

                            if lines_editor_state.cursor.row > 0
                                || lines_editor_state.line_count_at_top_of_window > 0
                            {
                                // Not at first line of file - can move up
                                execute_command(lines_editor_state, Command::MoveUp(1))?;
                                execute_command(lines_editor_state, Command::GotoLineEnd)?;
                                remaining_moves -= 1;
                                needs_rebuild = true;
                                continue;
                            } else {
                                // At first line of file - can't go up
                                break;
                            }
                        }
                        Ok(Some(_)) => {
                            // Position to left has file data - normal left movement
                            // Fall through to existing movement logic below
                        }
                        Err(_) => {
                            // Window map lookup failed - stop here
                            break;
                        }
                    }
                }

                // Kind of works but not stable yet...
                // // ===================================================================
                // // CHECK IF PREVIOUS CHARACTER IS A NEWLINE (BEFORE MOVING)
                // // ===================================================================

                // let current_file_pos = match lines_editor_state
                //     .window_map
                //     .get_file_position(lines_editor_state.cursor.row, lines_editor_state.cursor.col)
                // {
                //     Ok(Some(pos)) => pos.byte_offset,
                //     Ok(None) => {
                //         // Can't get position, just try moving left normally
                //         break;
                //     }
                //     Err(_) => break,
                // };

                // // Peek backward to see if previous byte is newline
                // let is_prev_newline = if current_file_pos > 0 {
                //     let prev_byte_pos = current_file_pos.saturating_sub(1);

                //     let mut peek_buffer = [0u8; 1];
                //     let mut peek_file = File::open(&read_copy)?;

                //     peek_file.seek(io::SeekFrom::Start(prev_byte_pos)).ok();
                //     match peek_file.read(&mut peek_buffer) {
                //         Ok(1) if peek_buffer[0] == b'\n' => true,
                //         _ => false,
                //     }
                // } else {
                //     false // At start of file, no previous byte
                // };

                // // ===================================================================
                // // IF NEWLINE BEHIND: SWITCH TO LINE NAVIGATION
                // // ===================================================================

                // if is_prev_newline {
                //     // Move up one line
                //     execute_command(lines_editor_state, Command::MoveUp(1))?;

                //     // Move to end of that line
                //     execute_command(lines_editor_state, Command::GotoLineEnd)?;

                //     remaining_moves -= 1;
                //     needs_rebuild = true;
                //     continue;
                // }

                // ===================================================================
                // NORMAL LEFT MOVEMENT (EXISTING LOGIC)
                // ===================================================================

                if lines_editor_state.cursor.col > 0 {
                    // Cursor can move left within visible window
                    let cursor_moves = remaining_moves.min(lines_editor_state.cursor.col);
                    lines_editor_state.cursor.col -= cursor_moves;
                    remaining_moves -= cursor_moves;
                } else if lines_editor_state.horizontal_utf8txt_line_char_offset > 0 {
                    // Cursor at left edge, scroll window left
                    let scroll_amount =
                        remaining_moves.min(lines_editor_state.horizontal_utf8txt_line_char_offset);
                    lines_editor_state.horizontal_utf8txt_line_char_offset -= scroll_amount;
                    remaining_moves -= scroll_amount;
                    needs_rebuild = true;
                } else {
                    // At absolute left edge - can't move further
                    break;
                }
            }

            // Defensive: Check iteration limit
            if iterations >= limits::CURSOR_MOVEMENT_STEPS {
                return Err(LinesError::Io(io::Error::new(
                    io::ErrorKind::Other,
                    "Maximum iterations exceeded in MoveLeft",
                )));
            }

            // Only rebuild if we scrolled the window
            if needs_rebuild {
                // Rebuild window to show the change from read-copy file
                build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            }

            Ok(true)
        }

        // // v2
        // Command::MoveLeft(count) => {
        //     let read_copy = lines_editor_state
        //         .read_copy_path
        //         .clone()
        //         .ok_or_else(|| LinesError::StateError("No read-copy path".into()))?;

        //     let file_size = fs::metadata(&read_copy).ok().map(|m| m.len()).unwrap_or(0);

        //     let mut remaining_moves = count;
        //     let mut needs_rebuild = false;
        //     let mut iterations = 0;

        //     while remaining_moves > 0 && iterations < limits::CURSOR_MOVEMENT_STEPS {
        //         iterations += 1;

        //         // ===================================================================
        //         // CHECK IF PREVIOUS CHARACTER IS A NEWLINE (BEFORE MOVING)
        //         // ===================================================================

        //         let current_file_pos = match lines_editor_state
        //             .window_map
        //             .get_file_position(lines_editor_state.cursor.row, lines_editor_state.cursor.col)
        //         {
        //             Ok(Some(pos)) => pos.byte_offset,
        //             Ok(None) => {
        //                 // Can't get position, just try moving left normally
        //                 break;
        //             }
        //             Err(_) => break,
        //         };

        //         // Peek backward to see if previous byte is newline
        //         let is_prev_newline = if current_file_pos > 0 {
        //             let prev_byte_pos = current_file_pos.saturating_sub(1);

        //             let mut peek_buffer = [0u8; 1];
        //             let mut peek_file = File::open(&read_copy)?;

        //             peek_file.seek(io::SeekFrom::Start(prev_byte_pos)).ok();
        //             match peek_file.read(&mut peek_buffer) {
        //                 Ok(1) if peek_buffer[0] == b'\n' => true,
        //                 _ => false,
        //             }
        //         } else {
        //             false // At start of file, no previous byte
        //         };

        //         // ===================================================================
        //         // IF NEWLINE BEHIND: SWITCH TO LINE NAVIGATION
        //         // ===================================================================

        //         if is_prev_newline {
        //             // Move up one line
        //             execute_command(lines_editor_state, Command::MoveUp(1))?;

        //             // Move to end of that line
        //             execute_command(lines_editor_state, Command::GotoLineEnd)?;

        //             remaining_moves -= 1;
        //             needs_rebuild = true;
        //             continue;
        //         }

        //         // ===================================================================
        //         // NORMAL LEFT MOVEMENT (EXISTING LOGIC)
        //         // ===================================================================

        //         if lines_editor_state.cursor.col > 0 {
        //             // Cursor can move left within visible window
        //             let cursor_moves = remaining_moves.min(lines_editor_state.cursor.col);
        //             lines_editor_state.cursor.col -= cursor_moves;
        //             remaining_moves -= cursor_moves;
        //         } else if lines_editor_state.horizontal_utf8txt_line_char_offset > 0 {
        //             // Cursor at left edge, scroll window left
        //             let scroll_amount =
        //                 remaining_moves.min(lines_editor_state.horizontal_utf8txt_line_char_offset);
        //             lines_editor_state.horizontal_utf8txt_line_char_offset -= scroll_amount;
        //             remaining_moves -= scroll_amount;
        //             needs_rebuild = true;
        //         } else {
        //             // At absolute left edge - can't move further
        //             break;
        //         }
        //     }

        //     // Defensive: Check iteration limit
        //     if iterations >= limits::CURSOR_MOVEMENT_STEPS {
        //         return Err(LinesError::Io(io::Error::new(
        //             io::ErrorKind::Other,
        //             "Maximum iterations exceeded in MoveLeft",
        //         )));
        //     }

        //     // Only rebuild if we scrolled the window
        //     if needs_rebuild {
        //         build_windowmap_nowrap(lines_editor_state, &read_copy)?;
        //     }

        //     Ok(true)
        // }

        // v2 - next line
        Command::MoveRight(count) => {
            let read_copy = lines_editor_state
                .read_copy_path
                .clone()
                .ok_or_else(|| LinesError::StateError("No read-copy path".into()))?;

            let file_size = fs::metadata(&read_copy).ok().map(|m| m.len()).unwrap_or(0);

            let mut remaining_moves = count;
            let mut needs_rebuild = false;
            let mut iterations = 0;

            while remaining_moves > 0 && iterations < limits::CURSOR_MOVEMENT_STEPS {
                iterations += 1;

                // ===================================================================
                // CHECK IF NEXT CHARACTER IS A NEWLINE (BEFORE MOVING)
                // ===================================================================

                let current_file_pos = match lines_editor_state
                    .window_map
                    .get_file_position(lines_editor_state.cursor.row, lines_editor_state.cursor.col)
                {
                    Ok(Some(pos)) => pos.byte_offset,
                    Ok(None) => {
                        // Can't get position, just try moving right normally
                        break;
                    }
                    Err(_) => break,
                };

                // Peek ahead to see if next byte is newline
                let mut peek_buffer = [0u8; 1];
                let mut peek_file = File::open(&read_copy)?;
                let next_byte_pos = current_file_pos.saturating_add(1);

                let is_next_newline = if next_byte_pos < file_size {
                    peek_file.seek(io::SeekFrom::Start(next_byte_pos)).ok();
                    match peek_file.read(&mut peek_buffer) {
                        Ok(1) if peek_buffer[0] == b'\n' => true,
                        _ => false,
                    }
                } else {
                    false
                };

                // ===================================================================
                // IF NEWLINE AHEAD: SWITCH TO LINE NAVIGATION
                // ===================================================================

                if is_next_newline {
                    // Move to start of current line
                    execute_command(lines_editor_state, Command::GotoLineStart)?;

                    // Move down one line
                    execute_command(lines_editor_state, Command::MoveDown(1))?;

                    remaining_moves -= 1;
                    needs_rebuild = true;
                    continue;
                }

                // ===================================================================
                // NORMAL RIGHT MOVEMENT (EXISTING LOGIC)
                // ===================================================================

                let right_edge = lines_editor_state.effective_cols.saturating_sub(1);

                if lines_editor_state.cursor.col < right_edge {
                    // Cursor can move right within visible window
                    let space_available = right_edge - lines_editor_state.cursor.col;
                    let cursor_moves = remaining_moves.min(space_available);

                    lines_editor_state.cursor.col += cursor_moves;
                    remaining_moves -= cursor_moves;
                } else {
                    // Cursor at right edge, scroll window right
                    if lines_editor_state.horizontal_utf8txt_line_char_offset
                        < limits::CURSOR_MOVEMENT_STEPS
                    {
                        let max_scroll = limits::CURSOR_MOVEMENT_STEPS
                            - lines_editor_state.horizontal_utf8txt_line_char_offset;
                        let scroll_amount = remaining_moves.min(max_scroll);
                        lines_editor_state.horizontal_utf8txt_line_char_offset += scroll_amount;
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
                return Err(LinesError::Io(io::Error::new(
                    io::ErrorKind::Other,
                    "Maximum iterations exceeded in MoveRight",
                )));
            }

            // Rebuild if needed
            if needs_rebuild {
                build_windowmap_nowrap(lines_editor_state, &read_copy)?;
            }

            Ok(true)
        }

        // What is 'count'?
        Command::MoveDown(count) => {
            // Vim-like behavior: move cursor down, scroll window if at bottom edge
            // Handle downward cursor movement with EOF boundary enforcement
            //
            // # Behavior
            // - Moves cursor down within visible window when possible
            // - Scrolls window down when cursor at bottom edge
            // - Stops at EOF: cursor cannot move past last line
            // - Gracefully handles all boundary conditions
            //
            // # EOF Handling
            // - If EOF visible in window: cursor stops at EOF display row
            // - If EOF visible and cursor at bottom: no scrolling occurs
            // - If EOF not visible: normal movement and scrolling

            let mut remaining_moves = count;
            let mut needs_rebuild = false;

            // Defensive: Limit iterations
            let mut iterations = 0;

            while remaining_moves > 0 && iterations < limits::CURSOR_MOVEMENT_STEPS {
                iterations += 1;

                // Calculate space available before bottom edge
                let bottom_edge = lines_editor_state.effective_rows.saturating_sub(1);

                if lines_editor_state.cursor.row < bottom_edge {
                    // Cursor can move down within visible window
                    // Check if EOF limits movement
                    let space_available = if let Some((_eof_line, eof_row)) =
                        lines_editor_state.eof_fileline_tuirow_tuple
                    {
                        if lines_editor_state.cursor.row < eof_row {
                            // Can move toward EOF
                            (eof_row - lines_editor_state.cursor.row)
                                .min(bottom_edge - lines_editor_state.cursor.row)
                        } else {
                            // At or past EOF, cannot move
                            0
                        }
                    } else {
                        // No EOF visible, normal movement
                        bottom_edge - lines_editor_state.cursor.row
                    };

                    if space_available == 0 {
                        break;
                    }

                    let cursor_moves = remaining_moves.min(space_available);
                    lines_editor_state.cursor.row += cursor_moves;
                    remaining_moves -= cursor_moves;
                } else {
                    // Cursor at bottom edge, try: scroll window down
                    // Check if EOF is visible (prevents scrolling past end)
                    if lines_editor_state.eof_fileline_tuirow_tuple.is_some() {
                        // EOF visible, cannot scroll further
                        break;
                    }

                    lines_editor_state.line_count_at_top_of_window += remaining_moves;
                    remaining_moves = 0;
                    needs_rebuild = true;
                }
            }

            // Defensive: Check iteration limit
            if iterations >= limits::CURSOR_MOVEMENT_STEPS {
                return Err(LinesError::Io(io::Error::new(
                    io::ErrorKind::Other,
                    "Maximum iterations exceeded in MoveDown",
                )));
            }

            // Rebuild window if we scrolled
            if needs_rebuild {
                // Rebuild window to show new content from file
                build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;

                // Defensive: After scrolling, verify cursor didn't scroll past EOF
                match lines_editor_state.eof_fileline_tuirow_tuple {
                    Some((_file_line_of_eof, eof_tui_display_row)) => {
                        // EOF is visible in rebuilt window
                        // file_line_of_eof = file line number where EOF occurs
                        // eof_tui_display_row = display row showing EOF

                        if lines_editor_state.cursor.row > eof_tui_display_row {
                            // Cursor past EOF, clamp to EOF position
                            lines_editor_state.cursor.row = eof_tui_display_row;
                        }
                    }
                    None => {
                        // EOF not visible in window, no clamping needed
                    }
                }
            }

            // TODO maybe a section to see if col is out of range?
            // // Position cursor AFTER line number (same as bootstrap)
            // // number of digits in line number + 1 is first character
            // let line_num_width =
            //     calculate_line_number_width(lines_editor_state.cursor.row);
            // lines_editor_state.cursor.col = line_num_width; // Skip over line number displayfull_lines_editor

            Ok(true)
        }

        /*'w' rules: Helix-type, move ON to next 'space or symbol'
        1. Move cursor forward 1 position
        2. Loop:
           - Look at char UNDER/AT cursor (at current byte-set)
           - If syntax char or EOF → STOP
           - If not-syntax → iterate and repeat

         */
        // Moves cursor forward to next syntax character (Helix-style 'w' command)
        //
        // # Purpose
        // Implements 'w' command for word navigation. Moves cursor forward one position,
        // then repeatedly checks if on syntax character. Stops when landing ON a syntax
        // character (space, tab, newline, or punctuation) or EOF.
        //
        // # Algorithm
        // For each count iteration:
        // 1. Move cursor forward 1 position (call MoveRight(1))
        // 2. Loop:
        //    - Get byte at current cursor position from file
        //    - Check if byte is syntax character or EOF
        //    - If syntax or EOF → STOP (cursor positioned on it)
        //    - If not syntax → Move forward 1 position and loop back
        //
        // # Arguments
        // * `count` - Number of syntax characters to move to (usually 1)
        //
        // # How It Works
        // - Uses existing MoveRight command for each forward step
        // - MoveRight handles all scrolling (horizontal and vertical)
        // - MoveRight handles newline crossing via existing logic
        // - This function just provides the "stop at syntax" logic
        //
        // # Return Value
        // * `Ok(true)` - Movement completed, editor loop continues
        // * `Err(LinesError)` - File read or cursor lookup failed
        //
        // # Edge Cases
        // - Cursor already on syntax: moves past it to next syntax
        // - Reaches EOF: stops at EOF position
        // - Long line requiring horizontal scroll: MoveRight handles it
        // - Line crossing: MoveRight's newline detection handles it
        //
        // # Example
        // File: "hello world"
        // Cursor at 'h' (position 0)
        // 1. MoveRight(1) → cursor on 'e'
        // 2. Not syntax, MoveRight(1) → cursor on 'l'
        // 3. Not syntax, MoveRight(1) → cursor on 'l'
        // 4. Not syntax, MoveRight(1) → cursor on 'o'
        // 5. Not syntax, MoveRight(1) → cursor on space
        // 6. IS syntax → STOP
        Command::MoveWordForward(count) => {
            let read_copy = lines_editor_state
                .read_copy_path
                .clone()
                .ok_or_else(|| LinesError::StateError("No read-copy path".into()))?;

            for _ in 0..count {
                // Step 1: Move forward 1 position
                execute_command(lines_editor_state, Command::MoveRight(1))?;

                // check each ~word-length move ahead if over 64 (or 32)
                let mut iteration = 0;

                // Step 2: Loop - check and stop at syntax
                loop {
                    // Defensive: Check iteration limit
                    if iteration >= WORD_MOVE_MAX_ITERATIONS {
                        // Hit limit - stop here even if no syntax found
                        let _ = lines_editor_state.set_info_bar_message("long word limit");
                        break;
                    }
                    iteration += 1;

                    // Get byte at current cursor position
                    let current_byte = match lines_editor_state.window_map.get_file_position(
                        lines_editor_state.cursor.row,
                        lines_editor_state.cursor.col,
                    ) {
                        Ok(Some(pos)) => {
                            let mut byte_buf = [0u8; 1];
                            let mut f = File::open(&read_copy)?;
                            f.seek(io::SeekFrom::Start(pos.byte_offset))?;
                            match f.read(&mut byte_buf) {
                                Ok(1) => byte_buf[0],
                                _ => 0, // EOF
                            }
                        }
                        _ => 0,
                    };

                    // Check if syntax or EOF
                    match is_syntax_char(current_byte) {
                        Ok(true) => break,               // STOP - on syntax
                        _ if current_byte == 0 => break, // STOP - at EOF
                        _ => {
                            // Not syntax - move forward and check again
                            execute_command(lines_editor_state, Command::MoveRight(1))?;
                        }
                    }
                }
            }

            Ok(true)
        }

        Command::MoveWordEnd(count) => {
            let read_copy = lines_editor_state
                .read_copy_path
                .clone()
                .ok_or_else(|| LinesError::StateError("No read-copy path".into()))?;

            for _ in 0..count {
                // ===================================================================
                // STEP 1: Initial forward movement (2 positions)
                // ===================================================================
                // Assumption: current position might be syntax, skip past it
                // and position to start searching for next syntax

                execute_command(lines_editor_state, Command::MoveRight(1))?;
                execute_command(lines_editor_state, Command::MoveRight(1))?;

                // ===================================================================
                // STEP 2: Loop - peek ahead until next char is syntax
                // ===================================================================

                // check each ~word-length move ahead if over 64 (or 32)
                let mut iteration = 0;

                // Step 2: Loop - check and stop at syntax
                loop {
                    // Defensive: Check iteration limit
                    if iteration >= WORD_MOVE_MAX_ITERATIONS {
                        // Hit limit - stop here even if no syntax found
                        let _ = lines_editor_state.set_info_bar_message("long word limit");
                        break;
                    }
                    iteration += 1;
                    // Get current cursor position in file
                    let current_pos = match lines_editor_state.window_map.get_file_position(
                        lines_editor_state.cursor.row,
                        lines_editor_state.cursor.col,
                    ) {
                        Ok(Some(pos)) => pos.byte_offset,
                        Ok(None) => break, // Invalid position, stop here
                        Err(_) => break,   // Lookup failed, stop here
                    };

                    // ===================================================================
                    // PEEK AHEAD: Look at NEXT byte (after current position)
                    // ===================================================================

                    let next_byte_pos = current_pos.saturating_add(1);

                    // Open file for peek operation
                    let mut f = File::open(&read_copy)?;

                    // Seek to next byte position
                    if let Err(_) = f.seek(io::SeekFrom::Start(next_byte_pos)) {
                        break; // Seek failed, probably at EOF
                    }

                    // Read next byte
                    let mut byte_buf = [0u8; 1];
                    let next_byte = match f.read(&mut byte_buf) {
                        Ok(1) => byte_buf[0],
                        Ok(0) => {
                            // EOF - next position has no byte
                            break; // STOP - we're at the position before EOF
                        }
                        _ => break, // Read error, stop here
                    };

                    // ===================================================================
                    // CHECK: Is next byte syntax?
                    // ===================================================================

                    match is_syntax_char(next_byte) {
                        Ok(true) => {
                            // Next byte IS syntax → STOP HERE
                            // Cursor is positioned BEFORE the syntax character
                            break;
                        }
                        Ok(false) => {
                            // Next byte is NOT syntax → continue moving forward
                            execute_command(lines_editor_state, Command::MoveRight(1))?;
                            // Loop will check the byte after this new position
                        }
                        Err(_) => {
                            // Error checking syntax - stop here
                            break;
                        }
                    }
                }
            }

            Ok(true)
        }
        Command::MoveWordBack(count) => {
            let read_copy = lines_editor_state
                .read_copy_path
                .clone()
                .ok_or_else(|| LinesError::StateError("No read-copy path".into()))?;

            for _ in 0..count {
                // ===================================================================
                // STEP 1: Initial backward movement (2 positions)
                // ===================================================================
                // Assumption: current position might be syntax, skip past it
                // Move back twice to position for search

                execute_command(lines_editor_state, Command::MoveLeft(1))?;
                execute_command(lines_editor_state, Command::MoveLeft(1))?;

                // ===================================================================
                // STEP 2: Loop - peek backward until previous char is syntax
                // ===================================================================

                let mut iteration = 0;

                loop {
                    // Defensive: Check iteration limit
                    if iteration >= WORD_MOVE_MAX_ITERATIONS {
                        // Hit limit - stop here even if no syntax found
                        break;
                    }
                    iteration += 1;

                    // Get current cursor position in file
                    let current_pos = match lines_editor_state.window_map.get_file_position(
                        lines_editor_state.cursor.row,
                        lines_editor_state.cursor.col,
                    ) {
                        Ok(Some(pos)) => pos.byte_offset,
                        Ok(None) => break, // Invalid position, stop here
                        Err(_) => break,   // Lookup failed, stop here
                    };

                    // Check if we're at start of file
                    if current_pos == 0 {
                        break; // Can't go back further
                    }

                    // ===================================================================
                    // PEEK BACKWARD: Look at PREVIOUS byte (before current position)
                    // ===================================================================

                    let prev_byte_pos = current_pos.saturating_sub(1);

                    // Open file for peek operation
                    let mut f = File::open(&read_copy)?;

                    // Seek to previous byte position
                    if let Err(_) = f.seek(io::SeekFrom::Start(prev_byte_pos)) {
                        break; // Seek failed, probably at start of file
                    }

                    // Read previous byte
                    let mut byte_buf = [0u8; 1];
                    let prev_byte = match f.read(&mut byte_buf) {
                        Ok(1) => byte_buf[0],
                        Ok(0) => {
                            // Unexpected EOF
                            break;
                        }
                        _ => break, // Read error, stop here
                    };

                    // ===================================================================
                    // CHECK: Is previous byte syntax?
                    // ===================================================================

                    match is_syntax_char(prev_byte) {
                        Ok(true) => {
                            // Previous byte IS syntax → STOP HERE
                            // Cursor is positioned AFTER the syntax character
                            break;
                        }
                        Ok(false) => {
                            // Previous byte is NOT syntax → continue moving backward
                            execute_command(lines_editor_state, Command::MoveLeft(1))?;
                            // Loop will check the byte before this new position
                        }
                        Err(_) => {
                            // Error checking syntax - stop here
                            break;
                        }
                    }
                }
            }

            Ok(true)
        }
        Command::GotoLine(line_number) => {
            // Convert 1-indexed (user display) to 0-indexed (file storage)
            let target_line = line_number.saturating_sub(1);

            // // Get file path
            // Get the read_copy path BEFORE the mutable borrow
            let read_copy = lines_editor_state
                .read_copy_path
                .clone()
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No read copy path"))?;

            // Seek to target line and update window position
            match seek_to_line_number(&mut File::open(&read_copy)?, target_line) {
                Ok(byte_pos) => {
                    lines_editor_state.line_count_at_top_of_window = target_line;
                    lines_editor_state.file_position_of_topline_start = byte_pos;
                    lines_editor_state.cursor.row = 0;
                    lines_editor_state.cursor.col = 0;

                    // Position cursor AFTER line number (same as bootstrap)
                    // number of digits in line number + 1 is first character
                    let line_num_width = calculate_line_number_width(line_number);
                    lines_editor_state.cursor.col = line_num_width; // Skip over line number displayfull_lines_editor

                    // Rebuild window to show the new position
                    build_windowmap_nowrap(lines_editor_state, &read_copy)?;

                    let _ = lines_editor_state
                        .set_info_bar_message(&format!("Jumped to line {}", line_number));
                    Ok(true)
                }
                Err(_) => {
                    let _ = lines_editor_state.set_info_bar_message("Line not found");
                    Ok(true)
                }
            }
        }

        Command::GotoFileStart => {
            // same as go-to-line-1
            let line_number: usize = 0;
            // Convert 1-indexed (user display) to 0-indexed (file storage)
            let target_line = line_number.saturating_sub(1);

            // // Get file path
            // Get the read_copy path BEFORE the mutable borrow
            let read_copy = lines_editor_state
                .read_copy_path
                .clone()
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No read copy path"))?;

            // Seek to target line and update window position
            match seek_to_line_number(&mut File::open(&read_copy)?, target_line) {
                Ok(byte_pos) => {
                    lines_editor_state.line_count_at_top_of_window = target_line;
                    lines_editor_state.file_position_of_topline_start = byte_pos;
                    lines_editor_state.cursor.row = 0;
                    lines_editor_state.cursor.col = 3; // Skip over line number displayfull_lines_editor

                    // Rebuild window to show the new position
                    build_windowmap_nowrap(lines_editor_state, &read_copy)?;

                    let _ = lines_editor_state
                        .set_info_bar_message(&format!("Jumped to line {}", line_number));
                    Ok(true)
                }
                Err(_) => {
                    let _ = lines_editor_state.set_info_bar_message("Line not found");
                    Ok(true)
                }
            }
        }

        Command::GotoFileLastLine => {
            // Get read-copy path
            let read_copy = lines_editor_state
                .read_copy_path
                .clone()
                .ok_or_else(|| LinesError::StateError("No read-copy path available".into()))?;

            // Count lines in file
            let (total_lines, _) = count_lines_in_file(&read_copy)?;

            // If file is empty, stay at current position
            if total_lines == 0 {
                let _ = lines_editor_state.set_info_bar_message("File is empty");
                return Ok(true);
            }

            // Jump to last line
            execute_command(lines_editor_state, Command::GotoLine(total_lines))?;

            Ok(true)
        }

        Command::GotoLineStart => {
            let read_copy = lines_editor_state
                .read_copy_path
                .clone()
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No read copy path"))?;

            goto_line_start(lines_editor_state, &read_copy)?;
            Ok(true)
        }

        Command::GotoLineEnd => {
            let read_copy = lines_editor_state
                .read_copy_path
                .clone()
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No read copy path"))?;

            goto_line_end(lines_editor_state, &read_copy)?;
            Ok(true)
        }
        Command::DeleteLine => {
            delete_current_line_noload(lines_editor_state, &edit_file_path)?;
            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            Ok(true)
        }

        Command::DeleteBackspace => {
            backspace_style_delete_noload(lines_editor_state, &edit_file_path)?;
            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
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

                if lines_editor_state.cursor.row > 0 {
                    // Cursor can move up within visible window
                    let cursor_moves = remaining_moves.min(lines_editor_state.cursor.row);
                    lines_editor_state.cursor.row -= cursor_moves;
                    remaining_moves -= cursor_moves;
                } else if lines_editor_state.line_count_at_top_of_window > 0 {
                    // Cursor at top edge, scroll window up
                    let scroll_amount =
                        remaining_moves.min(lines_editor_state.line_count_at_top_of_window);
                    lines_editor_state.line_count_at_top_of_window -= scroll_amount;
                    remaining_moves -= scroll_amount;
                    needs_rebuild = true;
                } else {
                    // At absolute top of file - can't move further
                    break;
                }
            }

            // Defensive: Check iteration limit
            if iterations >= limits::CURSOR_MOVEMENT_STEPS {
                return Err(LinesError::Io(io::Error::new(
                    io::ErrorKind::Other,
                    "Maximum iterations exceeded in MoveUp",
                )));
            }

            // Rebuild window if we scrolled
            if needs_rebuild {
                // Rebuild window to show the change from read-copy file
                build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            }

            Ok(true)
        }

        Command::InsertNewline(_) => {
            insert_newline_at_cursor_chunked(lines_editor_state, edit_file_path)?;

            // Rebuild window to show the change
            build_windowmap_nowrap(lines_editor_state, edit_file_path)?;

            Ok(true)
        }

        // TODO: this legacy item may not be needed...
        // Command::DeleteChar => {
        //     // This command is not used in normal flow
        //     eprintln!("Warning: DeleteChar command called directly (unexpected)");
        //     Ok(true)
        // }
        Command::EnterInsertMode => {
            // Without rebuild here, hexedit changes do not appear until
            // after a next change. Keep in Sync.
            // Rebuild window to show the change from read-copy file
            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            lines_editor_state.mode = EditorMode::Insert;
            Ok(true)
        }

        Command::EnterNormalMode => {
            // Without rebuild here, hexedit changes do not appear until
            // after a next change. Keep in Sync.
            // Rebuild window to show the change from read-copy file
            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            lines_editor_state.mode = EditorMode::Normal;
            Ok(true)
        }

        Command::EnterVisualMode => {
            // Without rebuild here, hexedit changes do not appear until
            // after a next change. Keep in Sync.

            // Set cursor position to file_position_of_vis_select_start
            // Get current cursor position in FILE
            if let Ok(Some(file_pos)) = lines_editor_state
                .window_map
                .get_file_position(lines_editor_state.cursor.row, lines_editor_state.cursor.col)
            {
                // Set BOTH start and end to same position initially
                lines_editor_state.file_position_of_vis_select_start = file_pos.byte_offset;
                lines_editor_state.file_position_of_vis_select_end = file_pos.byte_offset;
            }

            // Rebuild window to show the change from read-copy file
            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            lines_editor_state.mode = EditorMode::VisualSelectMode;
            // Set selection start at current cursor position
            if let Ok(Some(file_pos)) = lines_editor_state
                .window_map
                .get_file_position(lines_editor_state.cursor.row, lines_editor_state.cursor.col)
            {
                lines_editor_state.selection_start = Some(file_pos);
            }
            Ok(true)
        }

        Command::EnterPastyClipboardMode => {
            // rebuild may not be needed here, but just in case
            // Rebuild window to show the change from read-copy file
            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            lines_editor_state.mode = EditorMode::PastyMode;
            Ok(true)
        }

        Command::EnterHexEditMode => {
            // rebuild may not be needed here, but just in case
            // Rebuild window to show the change from read-copy file
            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            lines_editor_state.mode = EditorMode::HexMode;

            // Convert current window position to file byte offset
            if let Ok(Some(file_pos)) = lines_editor_state
                .window_map
                .get_file_position(lines_editor_state.cursor.row, lines_editor_state.cursor.col)
            {
                // Start hex cursor at same file position
                lines_editor_state.hex_cursor.byte_offset = file_pos.byte_offset as usize;
            } else {
                // Fallback to file start if cursor position invalid
                lines_editor_state.hex_cursor.byte_offset = 0;
            }

            Ok(true)
        }

        Command::Save => {
            save_file(lines_editor_state)?;
            Ok(true)
            // Save doesn't need rebuild (no content change in display)
        }

        Command::Quit => {
            // // Must-Save Mode (optional)
            // if state.is_modified {
            //     // Todo, maybe have a press enter to proceed thing...
            //     println!("Warning: Unsaved changes! Use 'w' to save.");
            //     Ok(true)
            // } else {
            //     Ok(false) // Signal to exit loop
            // }
            // Clean up session directory before the exiting
            // Wash your teeth and brush your face!

            if let Err(e) = cleanup_session_directory(lines_editor_state) {
                eprintln!("Warning: Session cleanup failed: {}", e);
                log_error(
                    &format!("Session cleanup failed: {}", e),
                    Some("Command::Quit"),
                );
                // Continue with exit anyway
            }

            // Default behavior: Let User Decide
            Ok(false) // Signal to exit loop
        }

        Command::SaveAndQuit => {
            save_file(lines_editor_state)?; // save file

            // Clean up session directory after save
            if let Err(e) = cleanup_session_directory(lines_editor_state) {
                eprintln!("Warning: Session cleanup failed: {}", e);
                log_error(
                    &format!("Session cleanup failed: {}", e),
                    Some("Command::SaveAndQuit"),
                );
                // Continue with exit anyway
            }
            Ok(false) // Signal to exit after save
        }
        // TODO Maybe not in scope
        // Command::ToggleWrap => {
        //     lines_editor_state.wrap_mode = match lines_editor_state.wrap_mode {
        //         WrapMode::Wrap => WrapMode::NoWrap,
        //         WrapMode::NoWrap => WrapMode::Wrap,
        //     };

        //     // Rebuild window with new wrap mode
        //     build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
        //     Ok(true)
        // }
        Command::Copyank => {
            let read_copy = lines_editor_state
                .read_copy_path
                .clone()
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No read copy path"))?;

            // Copy the Selection To The Pasty Clipboard (as a file)
            copy_selection_to_clipboardfile(lines_editor_state, &read_copy)?;

            Ok(true)
        }

        Command::None => Ok(true),
    }
}

/// Moves cursor to start of current displayed line
///
/// # Purpose
/// Positions cursor at the beginning of the line in the file.
/// Scrolls horizontally back to show line start (undoes any rightward scroll).
///
/// # Memory Safety
/// - Pre-allocated 4096-byte buffer (stack-only)
/// - Reads ONE line at a time
/// - No file loading
/// - No dynamic allocation
///
/// # Defensive Programming
/// - All errors logged and handled gracefully
/// - Returns with info bar message on any issue
/// - Never crashes, never panics in production
///
/// # Arguments
/// * `state` - Editor state with cursor position
/// * `file_path` - Path to read-copy file
///
/// # Returns
/// * `Ok(())` - Always succeeds or logs error and continues
fn goto_line_start(state: &mut EditorState, file_path: &Path) -> Result<()> {
    // ========================================================================
    // STEP 1: Get file position from cursor (defensive)
    // ========================================================================

    let current_file_pos = match state
        .window_map
        .get_file_position(state.cursor.row, state.cursor.col)
    {
        Ok(Some(pos)) => pos,
        Ok(None) => {
            let _ = state.set_info_bar_message("cursor position unavailable");
            return Ok(());
        }
        Err(e) => {
            let _ = state.set_info_bar_message("cannot get cursor position");
            log_error(
                &format!("goto_line_start window_map error: {}", e),
                Some("goto_line_start"),
            );
            return Ok(());
        }
    };

    let line_number_for_display = current_file_pos.line_number + 1; // Convert to 1-indexed

    // ========================================================================
    // STEP 2: Calculate line number width for cursor positioning
    // ========================================================================

    let line_num_width = calculate_line_number_width(line_number_for_display);

    // ========================================================================
    // STEP 3: Reset horizontal scroll to 0
    // ========================================================================

    // This is the key difference from gl
    // gh always goes back to the left edge (character 0 of the line)
    let previous_offset = state.horizontal_utf8txt_line_char_offset;
    state.horizontal_utf8txt_line_char_offset = 0;

    // ========================================================================
    // STEP 4: Position cursor at line start (after line number)
    // ========================================================================

    state.cursor.col = line_num_width;

    // ========================================================================
    // STEP 5: Rebuild window if horizontal offset changed
    // ========================================================================

    let needs_rebuild = previous_offset != 0;

    if needs_rebuild {
        if let Err(e) = build_windowmap_nowrap(state, file_path) {
            let _ = state.set_info_bar_message("display update failed");
            log_error(
                &format!("goto_line_start rebuild error: {}", e),
                Some("goto_line_start"),
            );
            // Continue anyway - cursor was updated
        }
    }

    let _ = state.set_info_bar_message("start of line");
    Ok(())
}

/// Moves cursor to end of current displayed line
///
/// # Purpose
/// Positions cursor at last character of line displayed at cursor.row.
/// If line longer than terminal width, scrolls horizontally to show end.
///
/// # Memory Safety
/// - Pre-allocated 4096-byte buffer (stack-only)
/// - Reads ONE line at a time
/// - No file loading
/// - No dynamic allocation
///
/// # Defensive Programming
/// - All errors logged and handled gracefully
/// - Returns with info bar message on any issue
/// - Never crashes, never panics in production
///
/// # Arguments
/// * `state` - Editor state with cursor position
/// * `file_path` - Path to read-copy file
///
/// # Returns
/// * `Ok(())` - Always succeeds or logs error and continues
fn goto_line_end(state: &mut EditorState, file_path: &Path) -> Result<()> {
    // ========================================================================
    // STEP 1: Get file position from cursor (defensive)
    // ========================================================================

    let current_file_pos = match state
        .window_map
        .get_file_position(state.cursor.row, state.cursor.col)
    {
        Ok(Some(pos)) => pos,
        Ok(None) => {
            let _ = state.set_info_bar_message("cursor position unavailable");
            return Ok(());
        }
        Err(e) => {
            let _ = state.set_info_bar_message("cannot get cursor position");
            log_error(
                &format!("goto_line_end window_map error: {}", e),
                Some("goto_line_end"),
            );
            return Ok(());
        }
    };

    let line_number_for_display = current_file_pos.line_number + 1; // Convert to 1-indexed
    let line_start_byte = current_file_pos.byte_offset - (current_file_pos.byte_in_line as u64);

    // ========================================================================
    // STEP 2: Read the line from file
    // ========================================================================

    let mut line_buffer = [0u8; 4096];

    let mut file = match File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            let _ = state.set_info_bar_message("cannot open file");
            log_error(
                &format!("goto_line_end open error: {}", e),
                Some("goto_line_end"),
            );
            return Ok(());
        }
    };

    if let Err(e) = file.seek(SeekFrom::Start(line_start_byte)) {
        let _ = state.set_info_bar_message("cannot seek to line");
        log_error(
            &format!("goto_line_end seek error: {}", e),
            Some("goto_line_end"),
        );
        return Ok(());
    }

    let (_, line_length, _) = match read_single_line(&mut file, &mut line_buffer) {
        Ok(result) => result,
        Err(e) => {
            let _ = state.set_info_bar_message("cannot read line");
            log_error(
                &format!("goto_line_end read error: {}", e),
                Some("goto_line_end"),
            );
            return Ok(());
        }
    };

    // ========================================================================
    // STEP 3: Convert bytes to characters
    // ========================================================================

    let line_bytes = &line_buffer[..line_length];
    let line_str = std::str::from_utf8(line_bytes).unwrap_or("");

    let char_count = line_str.chars().count();
    let char_position_in_line = if char_count > 0 { char_count - 1 } else { 0 };

    // ========================================================================
    // STEP 4: Calculate display column
    // ========================================================================

    let line_num_width = calculate_line_number_width(line_number_for_display);
    let display_col_for_line_end = line_num_width + char_position_in_line;

    let right_edge = state.effective_cols.saturating_sub(1);
    let mut needs_rebuild = false;

    // ========================================================================
    // STEP 5: Handle horizontal scrolling
    // ========================================================================

    if display_col_for_line_end > right_edge {
        // Line is longer than terminal width
        let overflow = display_col_for_line_end - right_edge;

        state.horizontal_utf8txt_line_char_offset = state
            .horizontal_utf8txt_line_char_offset
            .saturating_add(overflow);

        state.cursor.col = right_edge;
        // println!("right_edge {right_edge}, display_col_for_line_end {display_col_for_line_end}");
        needs_rebuild = true;
    } else {
        // Line fits within terminal
        // TODO: why is this odd?
        state.cursor.col = display_col_for_line_end;
    }

    // ========================================================================
    // STEP 6: Rebuild if needed
    // ========================================================================

    if needs_rebuild {
        if let Err(e) = build_windowmap_nowrap(state, file_path) {
            let _ = state.set_info_bar_message("display update failed");
            log_error(
                &format!("goto_line_end rebuild error: {}", e),
                Some("goto_line_end"),
            );
            // Continue anyway - cursor was updated
        }
    }

    let _ = state.set_info_bar_message(&format!("end of line ({} chars)", char_count));
    Ok(())
}

/// Deletes the character before cursor WITHOUT loading whole file
///
/// # Algorithm
/// 1. Get cursor file position
/// 2. Find previous UTF-8 character boundary (walk back max 4 bytes)
/// 3. Use chunked delete: copy [0..prev_char) + copy [cursor..EOF)
/// 4. Update cursor position
///
/// # Memory
/// - 8KB pre-allocated buffer for chunking
/// - No whole-file load
/// - Bounded iterations
fn backspace_style_delete_noload(state: &mut EditorState, file_path: &Path) -> io::Result<()> {
    // Step 1: Get current file position
    let file_pos = state
        .window_map
        .get_file_position(state.cursor.row, state.cursor.col)?
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "Cursor not on valid position")
        })?;

    let cursor_byte = file_pos.byte_offset;

    // Step 2: Can't delete before start of file
    if cursor_byte == 0 {
        return Ok(()); // Nothing to delete
    }

    // Step 3: Find start of previous UTF-8 character
    // Read up to 4 bytes back to find character boundary
    let prev_char_start = find_previous_utf8_boundary(file_path, cursor_byte)?;

    // Step 4: Delete byte range [prev_char_start..cursor_byte)
    delete_byte_range_chunked(file_path, prev_char_start, cursor_byte)?;

    // Step 5: Update state
    state.is_modified = true;

    // Step 6: Log edit
    let bytes_deleted = cursor_byte - prev_char_start;
    state.log_edit(&format!(
        "BACKSPACE line:{} byte:{} deleted:{} bytes",
        file_pos.line_number, prev_char_start, bytes_deleted
    ))?;

    // Step 7: Move cursor back one position
    if state.cursor.col > 0 {
        state.cursor.col -= 1;
    } else if state.cursor.row > 0 {
        // Deleted at line start - move to end of previous line
        state.cursor.row -= 1;
        // Will be repositioned correctly after window rebuild
    }

    Ok(())
}

/// Scans backward from position to find start of current line
/// Returns byte position right after previous \n (or 0 if at BOF)
fn find_line_start(file_path: &Path, from_byte: u64) -> io::Result<u64> {
    if from_byte == 0 {
        return Ok(0);
    }

    let mut file = File::open(file_path)?;
    let mut pos = from_byte.saturating_sub(1);
    let mut buffer = [0u8; 1];
    let mut iterations = 0;

    loop {
        if iterations >= limits::FILE_SEEK_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Max iterations finding line start",
            ));
        }
        iterations += 1;

        file.seek(SeekFrom::Start(pos))?;
        let n = file.read(&mut buffer)?;

        if n == 0 || buffer[0] == b'\n' {
            return Ok(pos + 1); // Start of line is after \n
        }

        if pos == 0 {
            return Ok(0); // Reached start of file
        }
        pos -= 1;
    }
}

/// Finds the byte position of the character before cursor
///
/// # Algorithm
/// - Seek to cursor_byte - 1
/// - Walk back up to 3 more bytes checking for UTF-8 start byte
/// - UTF-8 start bytes: 0b0xxxxxxx or 0b11xxxxxx
/// - Continuation bytes: 0b10xxxxxx
fn find_previous_utf8_boundary(file_path: &Path, cursor_byte: u64) -> io::Result<u64> {
    if cursor_byte == 0 {
        return Ok(0);
    }

    let mut file = File::open(file_path)?;

    // Start 1 byte back
    let mut pos = cursor_byte - 1;
    let mut buffer = [0u8; 1];

    // Defensive: limit iterations (UTF-8 chars max 4 bytes)
    for _ in 0..limits::MAX_UTF8_BOUNDARY_SCAN {
        file.seek(SeekFrom::Start(pos))?;
        file.read_exact(&mut buffer)?;

        let byte = buffer[0];

        // Check if this is a UTF-8 start byte
        if (byte & 0b1100_0000) != 0b1000_0000 {
            // Found start of character
            return Ok(pos);
        }

        // This is a continuation byte, keep going back
        if pos == 0 {
            return Ok(0); // Hit start of file
        }
        pos -= 1;
    }

    // Shouldn't happen with valid UTF-8
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "Could not find UTF-8 character boundary",
    ))
}

/// Scans forward from position to find end of current line
/// Returns byte position of \n character (or EOF position)
///
/// # Arguments
/// * `file_path` - Path to file to scan
/// * `from_byte` - Starting byte position (anywhere in the line)
///
/// # Returns
/// * `Ok(byte_pos)` - Position of \n or EOF
/// * `Err(io::Error)` - If scan fails or exceeds limits
fn find_line_end(file_path: &Path, from_byte: u64) -> io::Result<u64> {
    let mut file = File::open(file_path)?;

    // Get file size for EOF detection
    let file_size = file.metadata()?.len();

    if from_byte >= file_size {
        return Ok(file_size); // Already at/past EOF
    }

    // Seek to starting position
    file.seek(SeekFrom::Start(from_byte))?;

    let mut pos = from_byte;
    let mut buffer = [0u8; 1];
    let mut iterations = 0;

    loop {
        // Defensive: Check iteration limit
        if iterations >= limits::FILE_SEEK_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Max iterations exceeded finding line end",
            ));
        }
        iterations += 1;

        // Read one byte
        let n = file.read(&mut buffer)?;

        if n == 0 {
            // Reached EOF
            return Ok(pos);
        }

        if buffer[0] == b'\n' {
            // Found newline - return its position
            return Ok(pos);
        }

        pos += 1;
    }
}

/// Checks if there's a newline character at the given position
///
/// # Arguments
/// * `file_path` - Path to file to check
/// * `byte_pos` - Position to check for newline
///
/// # Returns
/// * `Ok(true)` - There is a \n at this position
/// * `Ok(false)` - No \n at this position (different char or EOF)
/// * `Err(io::Error)` - If read fails
fn line_end_has_newline(file_path: &Path, byte_pos: u64) -> io::Result<bool> {
    /*
    // Case 1: Normal line with newline
    // File: "Line1\nLine2\nLine3\n"
    // Cursor on Line2
    // line_start = 6, line_end = 11 (the \n), delete_end = 12
    // Result: "Line1\nLine3\n"

    // Case 2: Last line without newline
    // File: "Line1\nLine2"
    // Cursor on Line2
    // line_start = 6, line_end = 11 (EOF), delete_end = 11
    // Result: "Line1\n"

    // Case 3: Single line file
    // File: "OnlyLine\n"
    // line_start = 0, line_end = 8, delete_end = 9
    // Result: "" (empty file)
     */

    let mut file = File::open(file_path)?;

    // Get file size
    let file_size = file.metadata()?.len();

    // If position is at or past EOF, there's no newline
    if byte_pos >= file_size {
        return Ok(false);
    }

    // Seek to position and read one byte
    file.seek(SeekFrom::Start(byte_pos))?;

    let mut buffer = [0u8; 1];
    let n = file.read(&mut buffer)?;

    if n == 0 {
        // EOF reached (shouldn't happen after size check, but defensive)
        return Ok(false);
    }

    // Check if it's a newline
    Ok(buffer[0] == b'\n')
}

/// Deletes entire line at cursor WITHOUT loading whole file
///
/// # Algorithm
/// 1. Find start of current line (scan back to previous \n or BOF)
/// 2. Find end of current line (scan forward to next \n or EOF)
/// 3. Delete byte range [line_start..line_end+1] (include newline)
/// 4. Cursor stays at same row (now showing next line)
///
/// # Edge Cases
/// - Last line with no trailing \n: delete to EOF
/// - Single line file: leaves empty file
/// - First line: deletes from BOF
fn delete_current_line_noload(state: &mut EditorState, file_path: &Path) -> io::Result<()> {
    // Step 1: Get current line's file position
    let file_pos = state
        .window_map
        .get_file_position(state.cursor.row, state.cursor.col)?
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "Cursor not on valid position")
        })?;

    // Step 2: Find line boundaries
    let line_start = find_line_start(file_path, file_pos.byte_offset)?;
    let line_end = find_line_end(file_path, file_pos.byte_offset)?;

    // Step 3: Include the newline character if present
    let delete_end = if line_end_has_newline(file_path, line_end)? {
        line_end + 1
    } else {
        line_end
    };

    // Step 4: Delete the line
    delete_byte_range_chunked(file_path, line_start, delete_end)?;

    // Step 5: Update state
    state.is_modified = true;
    state.log_edit(&format!(
        "DELETE_LINE line:{} bytes:{}-{}",
        file_pos.line_number, line_start, delete_end
    ))?;

    // Step 6: Cursor stays at current row
    // After rebuild, this row will show the next line
    state.cursor.col = 0; // Move to start of (new) line

    Ok(())
}

// TODO why is this re-allocating the same chunk-buffer size?
/// Deletes a byte range from file using chunked operations
///
/// # Algorithm
/// 1. Create temporary file
/// 2. Copy bytes [0..start) from source to temp
/// 3. Skip bytes [start..end) (the deletion)
/// 4. Copy bytes [end..EOF) from source to temp
/// 5. Replace source with temp
///
/// # Memory
/// - Uses 8KB buffer (pre-allocated)
/// - Never loads full file
/// - Bounded iteration with MAX_FILE_SIZE check
fn delete_byte_range_chunked(file_path: &Path, start_byte: u64, end_byte: u64) -> io::Result<()> {
    // Defensive: Validate range
    if start_byte >= end_byte {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid deletion range",
        ));
    }

    // Create temp file in same directory
    let temp_path = file_path.with_extension("tmp_delete");

    // Pre-allocated 8KB buffer
    const CHUNK_SIZE: usize = 8192;
    let mut buffer = [0u8; CHUNK_SIZE];

    let mut source = File::open(file_path)?;
    let mut dest = File::create(&temp_path)?;

    // Phase 1: Copy bytes before deletion point
    let mut bytes_copied = 0u64;
    let mut iterations = 0;

    while bytes_copied < start_byte && iterations < limits::FILE_SEEK_BYTES {
        iterations += 1;

        let to_read = ((start_byte - bytes_copied) as usize).min(CHUNK_SIZE);
        let n = source.read(&mut buffer[..to_read])?;

        if n == 0 {
            break;
        } // EOF before start_byte

        dest.write_all(&buffer[..n])?;
        bytes_copied += n as u64;
    }

    // Phase 2: Skip deletion range
    source.seek(SeekFrom::Start(end_byte))?;

    // Phase 3: Copy remaining bytes
    iterations = 0;
    loop {
        if iterations >= limits::FILE_SEEK_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Max iterations exceeded",
            ));
        }
        iterations += 1;

        let n = source.read(&mut buffer)?;
        if n == 0 {
            break;
        }

        dest.write_all(&buffer[..n])?;
    }

    dest.flush()?;
    drop(dest);
    drop(source);

    // Replace original with modified
    fs::rename(&temp_path, file_path)?;

    Ok(())
}

/// Calculates how many columns the line number display uses
///
/// # Arguments
/// * `line_number` - The 1-indexed line number (for display)
///
/// # Returns
/// * Number of columns used (digits + 1 space)
///
/// # Examples
/// * Line 1-9: "1 " = 2 columns
/// * Line 10-99: "10 " = 3 columns
/// * Line 100-999: "100 " = 4 columns
fn calculate_line_number_width(line_number: usize) -> usize {
    if line_number == 0 {
        return 2; // Edge case: treat as single digit
    }

    // Count digits
    let digits = if line_number < 10 {
        1
    } else if line_number < 100 {
        2
    } else if line_number < 1000 {
        3
    } else if line_number < 10000 {
        4
    } else if line_number < 100000 {
        5
    } else {
        6 // Cap at 6 digits (999,999 lines max)
    };

    digits + 1 // Add 1 for the space after the number
}

// TODO: this should use general_use_256_buffer
/// Inserts a newline character at cursor position WITHOUT loading whole file
///
/// # Purpose
/// Chunked implementation of newline insertion following NASA Power of 10 rules.
/// Uses pre-allocated buffers and bounded iterations.
///
/// # Algorithm
/// 1. Get cursor byte position
/// 2. Create temporary file
/// 3. Copy bytes [0..cursor) from source to temp (chunked)
/// 4. Write '\n' to temp
/// 5. Copy bytes [cursor..EOF) from source to temp (chunked)
/// 6. Replace source with temp
///
/// # Arguments
/// * `state` - Editor state with cursor position
/// * `file_path` - Path to the file being edited (read-copy)
///
/// # Returns
/// * `Ok(())` - Newline inserted successfully
/// * `Err(io::Error)` - File operations failed
///
/// # Memory
/// - Uses 8KB pre-allocated buffer
/// - Never loads whole file
/// - Bounded iteration counts
fn insert_newline_at_cursor_chunked(state: &mut EditorState, file_path: &Path) -> io::Result<()> {
    // Step 1: Get file position from cursor (with graceful error handling)
    let file_pos = match state
        .window_map
        .get_file_position(state.cursor.row, state.cursor.col)
    {
        Ok(Some(pos)) => pos,
        Ok(None) => {
            eprintln!("Warning: Cannot insert - cursor not on valid file position");
            log_error(
                "Insert newline failed: cursor not on valid file position",
                Some("insert_newline_at_cursor_chunked"),
            );
            return Ok(());
        }
        Err(e) => {
            eprintln!("Warning: Cannot get cursor position: {}", e);
            log_error(
                &format!("Insert newline failed: {}", e),
                Some("insert_newline_at_cursor_chunked"),
            );
            return Ok(());
        }
    };

    let insert_position = file_pos.byte_offset;

    // Step 2: Create temporary file
    let temp_path = file_path.with_extension("tmp_insert");

    // Step 3: Open source and destination files
    let mut source = File::open(file_path)?;
    let mut dest = File::create(&temp_path)?;

    // TODO this should not be be allocating MORE memory
    // this should use a standard modular buffer
    // Pre-allocated 8KB buffer
    const CHUNK_SIZE: usize = 8192;
    let mut buffer = [0u8; CHUNK_SIZE];

    // ...general_use_256_buffer
    //
    // state.clear_general_256_buffer;

    // Step 4: Copy bytes before insertion point
    let mut bytes_copied = 0u64;
    let mut iterations = 0;

    while bytes_copied < insert_position && iterations < limits::FILE_SEEK_BYTES {
        iterations += 1;

        let to_read = ((insert_position - bytes_copied) as usize).min(CHUNK_SIZE);

        // TODO use state buffer
        // let n = source.read(state.general_use_256_buffer[..to_read])?;
        let n = source.read(&mut buffer[..to_read])?;

        if n == 0 {
            // EOF before insert position - this is an error
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Insert position {} exceeds file length {}",
                    insert_position, bytes_copied
                ),
            ));
        }

        dest.write_all(&buffer[..n])?;
        bytes_copied += n as u64;
    }

    // Defensive: Check iteration limit
    if iterations >= limits::FILE_SEEK_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Max iterations exceeded copying before insert point",
        ));
    }

    // Step 5: Write the newline character
    dest.write_all(b"\n")?;

    // Step 6: Copy remaining bytes (from insert position to EOF)
    // Source is already positioned at insert_position from previous reads
    iterations = 0;

    loop {
        if iterations >= limits::FILE_SEEK_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Max iterations exceeded copying after insert point",
            ));
        }
        iterations += 1;

        let n = source.read(&mut buffer)?;
        if n == 0 {
            break; // EOF reached
        }

        dest.write_all(&buffer[..n])?;
    }

    // Step 7: Flush and close files
    dest.flush()?;
    drop(dest);
    drop(source);

    // Step 8: Replace original with modified temp file
    fs::rename(&temp_path, file_path)?;

    // Step 9: Mark file as modified
    state.is_modified = true;

    // Step 10: Log the edit
    state.log_edit(&format!(
        "INSERT_NEWLINE line:{} byte:{}",
        file_pos.line_number, file_pos.byte_offset
    ))?;

    // // Step 11: Update cursor - move to start of new line
    // state.cursor.row += 1;
    // state.cursor.col = 0;

    // Step 11: Update cursor - move to start of new line
    state.cursor.row += 1;

    // Calculate where the text starts after the line number
    let new_line_number = state.line_count_at_top_of_window + state.cursor.row;
    let line_num_width = calculate_line_number_width(new_line_number + 1); // +1 for 1-indexed display

    state.cursor.col = line_num_width; // Position cursor after line number

    // Note: We don't update line_count_at_top_of_window here
    // The window rebuild will handle proper positioning

    Ok(())
}

// ============================================================================
// FILE INSERTION AT CURSOR
// ============================================================================

/// Inserts entire source file at cursor position, then removes final byte
///
/// # Overview
///
/// This function reads a source file chunk-by-chunk and inserts it at the current
/// cursor position in the target file. After all chunks are inserted, it removes
/// the final byte (typically a trailing newline per POSIX convention).
///
/// # Design Philosophy: Byte Offset Math, Not Cursor Tracking
///
/// **Problem with cursor tracking:**
/// During multi-line insertion, cursor position becomes ambiguous. After inserting
/// "hello\nworld", where is the cursor? Line 2, column 5? But what if windowmap
/// hasn't rebuilt yet? What if horizontal scrolling occurred? Cursor state becomes
/// unreliable mid-operation.
///
/// **Solution: Pure byte offset arithmetic:**
/// - Read cursor position ONCE at start → get starting byte offset
/// - Calculate each chunk's position: `start_offset + bytes_already_written`
/// - Track total bytes written as simple integer counter
/// - Delete final byte at known position: `start_offset + total_bytes - 1`
///
/// This eliminates state synchronization issues. No cursor updates during insertion.
/// Windowmap rebuilt once at end when all data is in place.
///
/// # Memory Safety - Stack Allocation Only
///
/// **Heap allocations in this function (unavoidable):**
/// - `PathBuf` for file paths (Rust stdlib requirement)
/// - Error message strings via `format!()` (logging only)
///
/// **Critical buffers are stack-allocated:**
/// - Source file read buffer: `[0u8; 256]` - 256 bytes on stack
/// - Shift buffer in helper functions: `[0u8; 8192]` - 8KB on stack
/// - No Vec, no String for data processing
/// - No dynamic allocation during bucket brigade
///
/// **Per NASA Rule 3 (pre-allocate memory):**
/// All working buffers are fixed-size arrays allocated at function scope.
/// No runtime memory allocation for data processing occurs.
///
/// # Bucket Brigade Pattern
///
/// Named after firefighting bucket brigades where buckets pass hand-to-hand:
/// 1. Read 256-byte chunk from source file
/// 2. Calculate insertion position for this chunk
/// 3. Insert chunk at calculated position
/// 4. Update total bytes written counter
/// 5. Repeat until EOF (bytes_read == 0)
///
/// **Iteration safety:** Limited to MAX_CHUNKS (16,777,216) to prevent infinite
/// loops from filesystem corruption or cosmic ray bit flips.
///
/// # File Operations
///
/// **Source file:**
/// - Opened read-only
/// - Read sequentially chunk-by-chunk
/// - Never loaded entirely into memory
/// - Automatically closed when function exits (RAII)
///
/// **Target file (read_copy):**
/// - Modified via position-based insertion
/// - Each chunk insertion shifts subsequent bytes right
/// - Final byte deletion shifts bytes left by 1
/// - File operations are atomic per-chunk (but not transactional overall)
///
/// # Why Remove Final Byte?
///
/// Most text files end with `\n` per POSIX convention. When inserting file contents
/// at cursor position (middle of existing content), that trailing newline would
/// create an unwanted blank line. Solution: remove it after insertion completes.
///
/// **Examples:**
/// - Inserting "hello\nworld\n" → We want "hello\nworld" (no trailing blank line)
/// - Inserting "hello" → We remove 'o', resulting in "hell" (edge case, but consistent)
/// - Inserting empty file → Nothing inserted, nothing deleted
///
/// # Workflow
///
/// ```text
/// 1. Validate source file path (absolute path, exists, is file not directory)
/// 2. Get target file path from editor state
/// 3. Get starting byte position from cursor (only cursor access in entire function)
/// 4. Open source file read-only
/// 5. Initialize counters and safety limits
/// 6. Bucket brigade loop:
///    a. Read up to 256 bytes into stack buffer
///    b. If EOF (bytes_read == 0): exit loop
///    c. Calculate insertion position: start + total_written
///    d. Call insert_bytes_at_position() to insert chunk
///    e. Increment total_bytes_written counter
///    f. Increment chunk counter, check MAX_CHUNKS limit
///    g. Repeat
/// 7. If any bytes were written:
///    a. Calculate last byte position: start + total - 1
///    b. Call delete_byte_at_position() to remove it
/// 8. Mark editor state as modified
/// 9. Rebuild windowmap once to reflect all changes
/// 10. Set success message in info bar
/// 11. Return Ok(())
/// ```
///
/// # Arguments
///
/// * `state` - Editor state
///   - Used to read: cursor position, read_copy_path, security_mode
///   - Used to modify: is_modified flag, info bar message
/// * `source_file_path` - Absolute or relative path to source file
///   - Converted to absolute path if relative
///   - Must exist, must be a file (not directory)
///
/// # Returns
///
/// * `Ok(())` - Entire file inserted successfully, final byte removed, windowmap rebuilt
/// * `Err(io::Error)` - Operation failed at some stage, partial insert may remain
///
/// # Error Conditions
///
/// Sets info bar message and returns Err if:
/// - Cannot get current working directory → "cannot get cwd"
/// - Source file doesn't exist → "file not found"
/// - Source path is directory, not file → "not a file"
/// - read_copy_path not set in state → "no target file"
/// - Cannot get byte position from cursor → "invalid cursor position"
/// - Source file can't be opened → "cannot read file"
/// - Read fails mid-file → "read error chunk N"
/// - Insert operation fails → propagates error from insert_bytes_at_position()
/// - Delete operation fails → propagates error from delete_byte_at_position()
/// - Iteration limit exceeded → "file too large"
/// - Windowmap rebuild fails → propagates error from build_windowmap_nowrap()
///
/// # Safety Limits
///
/// **Maximum chunks:** 16,777,216 (allows ~4GB at 256-byte chunks)
/// - Per NASA Rule 2: upper bound on all loops
/// - Prevents infinite loops from:
///   - Filesystem corruption returning garbage data
///   - Cosmic ray bit flips in file size metadata
///   - Malicious or malformed files
///
/// **Chunk size:** 256 bytes
/// - Balance between I/O efficiency and memory usage
/// - Small enough for stack allocation safety
/// - Large enough to minimize syscall overhead
///
/// # Edge Cases
///
/// **Empty source file:**
/// - First read returns 0 bytes
/// - Loop exits immediately
/// - total_bytes_written == 0
/// - No deletion attempted (if-guard protects)
/// - Info bar shows "inserted 0 bytes"
/// - Returns Ok(()) - valid operation
///
/// **Single-byte file:**
/// - Inserts 1 byte
/// - Deletes that byte
/// - Result: nothing inserted
/// - Edge case but consistent with "remove final byte" policy
///
/// **File with no trailing newline:**
/// - Inserts entire file content
/// - Deletes last character (whatever it is)
/// - User loses one character
/// - Documented behavior - "removes final byte", not "final newline"
///
/// **Very large file (triggers MAX_CHUNKS):**
/// - Insertion stops at chunk limit
/// - Partial file inserted
/// - Error returned with "file too large" message
/// - No automatic rollback
///
/// **Binary file:**
/// - Works correctly - byte-level operations
/// - No UTF-8 assumptions
/// - No text processing
/// - Final byte still removed (might corrupt binary format)
///
/// **Source same as target:**
/// - Not checked - caller's responsibility
/// - Would likely cause undefined behavior
/// - File modified while being read
/// - Defensive programming note: should be checked at caller level
///
/// **Multi-byte UTF-8 character at chunk boundary:**
/// - Not handled specially
/// - Chunk-based insertion preserves byte sequence
/// - UTF-8 sequences stay intact (inserted as-is)
/// - Final byte deletion might split UTF-8 character if file ends mid-character
///
/// **Cursor at EOF:**
/// - Valid insertion point (appends to file)
/// - Works correctly - start_byte_position points past last byte
/// - Subsequent bytes shifted from that position (none exist)
/// - Final byte deletion removes last byte of inserted content
///
/// # Defensive Programming
///
/// - **Path validation:** Converts relative to absolute, checks existence, checks is_file
/// - **Buffer clearing:** In security_mode, manually zeros buffers before use
/// - **Assertion:** bytes_read never exceeds buffer size (detects memory corruption)
/// - **Bounded loops:** MAX_CHUNKS prevents infinite loops
/// - **Fail-fast:** Returns error immediately on first failure
/// - **No unwrap:** All Result types explicitly handled
/// - **No panic:** Assertion is only check that would panic (memory corruption case)
/// - **No unsafe:** Pure safe Rust
/// - **Logging:** All errors logged with context before returning
/// - **User feedback:** Info bar updated with success/error messages
///
/// # Performance Characteristics
///
/// **Time complexity:**
/// - O(N * M) where N = file size, M = average bytes after insertion point
/// - Each chunk insertion shifts M bytes
/// - Worst case: inserting at start of large file
/// - Not optimized for performance - correctness prioritized
///
/// **Space complexity:**
/// - O(1) - fixed-size stack buffers only
/// - No growth with file size
/// - 256-byte read buffer + 8KB shift buffer = ~8.3KB max stack usage
///
/// **I/O operations:**
/// - Read: N/256 sequential reads from source (where N = file size)
/// - Write: N/256 * 2 writes to target (insert + shift for each chunk)
/// - Seek: N/256 * 2 seeks (position for read + position for write)
/// - Final deletion: 1 read, 1 write, 1 seek, 1 truncate
/// - Total: ~(N/256) * 5 + 4 I/O operations
///
/// # Policy Notes
///
/// - **No rollback on error:** Follows Lines policy - user controls undo, not automatic
/// - **No progress bar:** Follows Lines policy - simplicity over features
/// - **Disk space not optimized:** In-place shifting is inefficient but simple
/// - **Absolute paths preferred:** Defensive programming policy
/// - **Immediate windowmap rebuild:** Happens once at end, not per-chunk
/// - **Position-based insertion:** Avoids cursor state management complexity
///
/// # Example Usage
///
/// ```ignore
/// // Insert another file at current cursor position
/// let source = Path::new("/home/user/snippet.txt");
/// match insert_file_at_cursor(&mut state, source) {
///     Ok(()) => {
///         // File inserted, final byte removed
///         // Windowmap updated, ready for next operation
///         println!("File inserted successfully");
///     }
///     Err(e) => {
///         // Error logged, info bar shows message
///         // Partial insert may remain (no rollback)
///         eprintln!("Insert failed: {}", e);
///     }
/// }
/// ```
///
/// # Comparison to Other Insertion Methods
///
/// **vs. insert_text_chunk_at_cursor_position():**
/// - That function updates cursor after each insert
/// - This function bypasses cursor entirely
/// - That function for single chunks, this for entire files
///
/// **vs. handle_utf8txt_insert_mode_input():**
/// - That function processes stdin with delimiter detection
/// - This function reads files with no delimiter ambiguity
/// - That function has complex newline handling logic
/// - This function uses simple "remove final byte" strategy
///
/// # See Also
///
/// * `insert_bytes_at_position()` - Helper function for chunk insertion
/// * `delete_byte_at_position()` - Helper function for final byte removal
/// * `build_windowmap_nowrap()` - Called once at end to update display
/// * `handle_utf8txt_insert_mode_input()` - Parallel implementation for stdin (more complex)
///
/// # Testing Considerations
///
/// Test with files containing:
/// - Empty file (0 bytes)
/// - Single byte ('a')
/// - Single line with newline ("hello\n")
/// - Single line without newline ("hello")
/// - Multiple lines ("hello\nworld\n")
/// - Only newlines ("\n\n\n")
/// - Binary data (null bytes, non-UTF-8)
/// - File size exactly 256 bytes (one chunk)
/// - File size 257 bytes (two chunks, second has 1 byte)
/// - Large file (multiple chunks, test performance)
/// - Very large file (trigger MAX_CHUNKS limit)
pub fn insert_file_at_cursor(state: &mut EditorState, source_file_path: &Path) -> Result<()> {
    // ============================================
    // Phase 1: Path Validation and Normalization
    // ============================================
    // Defensive: Convert relative paths to absolute
    // Relative paths depend on cwd which can change during execution

    let source_path = if source_file_path.is_absolute() {
        source_file_path.to_path_buf()
    } else {
        // Convert relative path to absolute path
        match std::env::current_dir() {
            Ok(cwd) => cwd.join(source_file_path),
            Err(e) => {
                let _ = state.set_info_bar_message("cannot get cwd");
                log_error(
                    &format!("Cannot get current directory: {}", e),
                    Some("insert_file_at_cursor"),
                );
                return Err(LinesError::Io(e));
            }
        }
    };

    // Defensive: Check source file exists before attempting to open
    // Fail fast with clear error message
    if !source_path.exists() {
        let _ = state.set_info_bar_message("file not found");
        log_error(
            &format!("Source file does not exist: {}", source_path.display()),
            Some("insert_file_at_cursor"),
        );
        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File not found: {}", source_path.display()),
        )));
    }

    // Defensive: Check source path is a file (not directory)
    // Attempting to read a directory would cause confusing errors later
    if !source_path.is_file() {
        let _ = state.set_info_bar_message("not a file");
        log_error(
            &format!("Source path is not a file: {}", source_path.display()),
            Some("insert_file_at_cursor"),
        );
        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Not a file: {}", source_path.display()),
        )));
    }

    // ============================================
    // Phase 2: Get Target File and Starting Position
    // ============================================
    // This is the ONLY place we read cursor position
    // After this, all operations use byte offset arithmetic

    let target_file_path = state.read_copy_path.clone().ok_or_else(|| {
        let _ = state.set_info_bar_message("no target file");
        log_error(
            "read_copy_path not set in editor state",
            Some("insert_file_at_cursor"),
        );
        io::Error::new(io::ErrorKind::Other, "No read copy path")
    })?;

    // Get starting byte position from cursor
    // This is the insertion point for the first chunk
    // Subsequent chunks insert at: start_position + bytes_already_written
    let start_byte_position = match state
        .window_map
        .get_file_position(state.cursor.row, state.cursor.col)
    {
        Ok(Some(pos)) => pos.byte_offset,
        Ok(None) => {
            let _ = state.set_info_bar_message("invalid cursor position");
            log_error(
                "Cannot get byte position from cursor",
                Some("insert_file_at_cursor"),
            );
            return Err(LinesError::Io(io::Error::new(
                io::ErrorKind::Other,
                "Invalid cursor position",
            )));
        }
        Err(e) => {
            let _ = state.set_info_bar_message("cursor position error");
            log_error(
                &format!("Error getting cursor position: {}", e),
                Some("insert_file_at_cursor"),
            );
            return Err(LinesError::Io(e));
        }
    };

    // ============================================
    // Phase 3: Open Source File
    // ============================================
    // File opened read-only
    // Automatically closed when function exits (RAII pattern)

    let mut source_file = match File::open(&source_path) {
        Ok(file) => file,
        Err(e) => {
            let _ = state.set_info_bar_message("cannot read file");
            log_error(
                &format!("Cannot open source file: {} - {}", source_path.display(), e),
                Some("insert_file_at_cursor"),
            );
            return Err(LinesError::Io(e));
        }
    };

    // ============================================
    // Phase 4: Initialize Bucket Brigade
    // ============================================
    // Counters and constants for the insertion loop

    const CHUNK_SIZE: usize = 256;
    const MAX_CHUNKS: usize = 16_777_216; // Allows ~4GB at 256-byte chunks

    let mut chunk_counter: usize = 0;
    let mut total_bytes_written: u64 = 0;

    // ============================================
    // Phase 5: Bucket Brigade Loop
    // ============================================
    // Read chunks from source, insert at calculated positions
    // Loop bounded by MAX_CHUNKS for safety (NASA Rule 2)

    loop {
        // Defensive: Prevent infinite loop from filesystem corruption
        // Cosmic ray bit flips in file metadata could cause endless reads
        if chunk_counter >= MAX_CHUNKS {
            let _ = state.set_info_bar_message("file too large");
            log_error(
                &format!("Maximum chunk limit reached: {}", MAX_CHUNKS),
                Some("insert_file_at_cursor"),
            );
            return Err(LinesError::Io(io::Error::new(
                io::ErrorKind::Other,
                "File too large",
            )));
        }

        // Pre-allocated buffer on stack (NASA Rule 3: no dynamic allocation)
        // This buffer is reused for each chunk - no per-iteration allocation
        let mut buffer = [0u8; CHUNK_SIZE];

        // Security mode: manually clear buffer before use
        // Prevents data leakage between chunks if read fails mid-buffer
        if state.security_mode {
            for i in 0..CHUNK_SIZE {
                buffer[i] = 0;
            }
        }

        // Read next chunk from source file
        // Returns Ok(n) where n = bytes read (0 = EOF)
        let bytes_read = match source_file.read(&mut buffer) {
            Ok(n) => n,
            Err(e) => {
                let _ = state.set_info_bar_message(&format!("read error chunk {}", chunk_counter));
                log_error(
                    &format!("Read error at chunk {}: {}", chunk_counter, e),
                    Some("insert_file_at_cursor"),
                );
                return Err(LinesError::Io(e));
            }
        };

        // Defensive assertion: bytes_read should never exceed buffer size
        //
        //    =================================================
        // // Debug-Assert, Test-Asset, Production-Catch-Handle
        //    =================================================
        // This is not included in production builds
        // assert: only when running in a debug-build: will panic
        debug_assert!(
            bytes_read <= CHUNK_SIZE,
            "bytes_read ({}) exceeded buffer size ({})",
            bytes_read,
            CHUNK_SIZE
        );
        // Defensive assertion: bytes_read should never exceed buffer size
        // If it does, indicates memory corruption or cosmic ray bit flip
        // This is the only panic point - for catastrophic failure only
        #[cfg(test)]
        assert!(
            bytes_read <= CHUNK_SIZE,
            "bytes_read ({}) exceeded buffer size ({})",
            bytes_read,
            CHUNK_SIZE
        );
        // Catch & Handle without panic in production
        // This IS included in production to safe-catch
        if !bytes_read <= CHUNK_SIZE {
            // state.set_info_bar_message("Config error");
            return Err(LinesError::GeneralAssertionCatchViolation(
                "zero buffer size error".into(),
            ));
        }

        // EOF detection: bytes_read == 0 reliably signals end of file
        // Unlike stdin, file EOF is deterministic and unambiguous
        if bytes_read == 0 {
            // Success - entire file read, exit loop normally
            break;
        }

        chunk_counter += 1;

        // Calculate insertion position for this chunk
        // Math: start_offset + sum_of_previous_chunks
        // This is why we don't need cursor - pure arithmetic
        let insert_position = start_byte_position + total_bytes_written;

        // Insert this chunk at calculated position
        // Helper function handles: read-after-point, seek, write, shift, flush
        insert_bytes_at_position(&target_file_path, insert_position, &buffer[..bytes_read])?;

        // Update counter for next iteration's calculation
        total_bytes_written += bytes_read as u64;

        // Continue to next chunk
        // Loop will exit when bytes_read == 0 (EOF) or chunk_counter >= MAX_CHUNKS
    }

    // ============================================
    // Phase 7: Update Editor State
    // ============================================
    // Mark file as modified and rebuild display

    state.is_modified = true;

    // Rebuild windowmap to reflect all insertions
    // This updates line numbering, cursor constraints, display mapping
    // Done once at end, not per-chunk (efficiency and simplicity)
    build_windowmap_nowrap(state, &target_file_path)?;

    // Set success message in info bar
    // Shows total bytes (after final byte deletion)
    // state.set_info_bar_message(&format!(
    //     "inserted {} bytes",
    //     total_bytes_written.saturating_sub(1) // -1 because we deleted final byte
    // ));

    // Set success message in info bar
    // If it fails, continue operation (message display is non-critical)
    let _ = state
        .set_info_bar_message(&format!(
            "inserted {} bytes",
            total_bytes_written.saturating_sub(1)
        ))
        .or_else(|e| {
            // Log error but don't propagate (message is cosmetic)
            eprintln!("Warning: Failed to set info bar message: {}", e);
            Ok::<(), LinesError>(()) // Convert to Ok to discard error
        });

    Ok(())
}

/// Parse single hex digit (0-9, A-F, a-f) into nibble value (0-15)
fn parse_hex_digit(byte: u8) -> io::Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid hex digit",
        )),
    }
}

/// Replaces a single byte at specified position (in-place, no shifting)
///
/// # Purpose
/// Overwrites one byte in file without changing file size.
/// Simplest possible file edit operation.
///
/// # Arguments
/// * `file_path` - Path to file to edit
/// * `position` - Byte offset to replace (0-indexed)
/// * `new_byte` - New byte value to write
///
/// # Returns
/// * `Ok(())` - Byte successfully replaced
/// * `Err(e)` - File operation failed
///
/// # File Operations
/// 1. Open file in write mode (preserves existing content)
/// 2. Seek to position
/// 3. Write 1 byte
/// 4. Flush to disk
/// 5. Close (automatic via RAII)
///
/// # Safety
/// - Bounded operation: writes exactly 1 byte
/// - No buffer allocation
/// - No read-modify-write
/// - Atomic at OS level (single-byte write)
///
/// # Edge Cases
/// - Position past EOF: write will extend file (OS behavior)
/// - Position at EOF: write will append 1 byte
/// - Read-only file: returns permission error
fn replace_byte_in_place(file_path: &Path, position: usize, new_byte: u8) -> io::Result<()> {
    // Open file for writing (preserves existing content)
    let mut file = OpenOptions::new().write(true).open(file_path)?;

    // Seek to target position
    file.seek(SeekFrom::Start(position as u64))?;

    // Write single byte (stack-allocated array)
    let byte_buffer = [new_byte];
    file.write_all(&byte_buffer)?;

    // Ensure write completes before function returns
    file.flush()?;

    Ok(())
    // File automatically closed here (RAII)
}

/// Inserts bytes at specific file position without cursor tracking
///
/// # Overview
///
/// This helper function inserts a byte slice at an arbitrary position in a file
/// by reading the bytes after that position, writing the new bytes, then writing
/// back the shifted bytes.
///
/// **Operation:**
/// ```text
/// Before: [A B C D E F]
///         Insert "XY" at position 3
/// After:  [A B C X Y D E F]
///                 ↑ insertion point (position 3)
/// ```
///
/// # Memory Safety - Stack Allocated Buffer
///
/// Uses 8KB stack buffer for shifting bytes after insertion point.
/// - No heap allocation for data processing
/// - Fixed-size buffer regardless of file size
/// - If file has > 8KB after insertion point, shifts occur in 8KB chunks
///
/// # Arguments
///
/// * `file_path` - Path to target file (read+write access required)
/// * `position` - Byte offset where to insert (0 = start, file_size = append)
/// * `bytes` - Slice of bytes to insert (any length, copied in one write)
///
/// # Returns
///
/// * `Ok(())` - Bytes inserted successfully, file modified
/// * `Err(io::Error)` - File operation failed (open, seek, read, write, flush)
///
/// # Algorithm
///
/// 1. Open file in read+write mode
/// 2. Seek to insertion position
/// 3. Read bytes after insertion point into buffer (up to 8KB)
/// 4. Seek back to insertion position
/// 5. Write new bytes
/// 6. Write back the shifted bytes (from buffer)
/// 7. Flush to ensure data written to disk
///
/// # Edge Cases
///
/// **Insert at EOF (position == file size):**
/// - Read after position returns 0 bytes
/// - Writes new bytes
/// - Nothing to shift back
/// - Equivalent to append operation
///
/// **Insert at start (position == 0):**
/// - Reads entire file into buffer (up to 8KB)
/// - Writes new bytes at position 0
/// - Writes back original content (shifted right)
/// - Most expensive case (maximum data movement)
///
/// **Insert with > 8KB after insertion point:**
/// - Only first 8KB shifted in this call
/// - Remaining bytes stay in original positions
/// - **BUG:** This corrupts the file if bytes_to_insert.len() + bytes_after > 8KB
/// - Should be fixed to loop-shift in chunks
/// - Current implementation assumes insert size + remaining < 8KB
///
/// **Empty insertion (bytes.len() == 0):**
/// - Valid operation (no-op)
/// - Still performs read/seek/write/flush
/// - File unchanged but timestamp updated
///
/// # Defensive Programming
///
/// - No unwrap calls
/// - All I/O operations explicitly error-checked
/// - Flush called to ensure disk write
/// - No assumptions about file permissions (error if not writable)
///
/// # Performance
///
/// - **Time:** O(M) where M = bytes after insertion point (up to 8KB)
/// - **Space:** O(1) - fixed 8KB stack buffer
/// - **I/O:** 1 read, 2 seeks, 2 writes, 1 flush = 6 operations
/// - Not optimized for repeated insertions (each call shifts independently)
///
/// # Known Limitations
///
/// **8KB shift buffer limit:**
/// If inserting N bytes at position P, and (file_size - P) > 8KB:
/// - Only first 8KB shifted correctly
/// - Data beyond 8KB may be overwritten
/// - Should loop to shift all remaining bytes
///
/// **No atomic operation:**
/// If write fails mid-operation, file left in inconsistent state.
/// No rollback, no transaction, no recovery.
///
/// # See Also
///
/// * `delete_byte_at_position()` - Inverse operation (removes byte)
/// * `insert_file_at_cursor()` - Caller that uses this function repeatedly
fn insert_bytes_at_position(file_path: &Path, position: u64, bytes: &[u8]) -> io::Result<()> {
    // Open file for read+write
    // Requires file already exists (won't create new file)
    let mut file = OpenOptions::new().read(true).write(true).open(file_path)?;

    // Pre-allocated buffer for bytes after insertion point
    // 8KB chosen as balance between stack usage and shift efficiency
    const BUFFER_SIZE: usize = 8192;
    let mut after_buffer = [0u8; BUFFER_SIZE];

    // Seek to insertion position and read bytes that will be shifted
    file.seek(SeekFrom::Start(position))?;
    let bytes_after = file.read(&mut after_buffer)?;

    // Seek back to insertion position to write new bytes
    file.seek(SeekFrom::Start(position))?;
    file.write_all(bytes)?;

    // Write the shifted bytes (what was at position, now at position+insert_size)
    file.write_all(&after_buffer[..bytes_after])?;

    // Flush to ensure data written to disk
    // Without flush, data might sit in OS buffer cache
    file.flush()?;

    Ok(())
}

/// Deletes single byte at specific file position
///
/// # Overview
///
/// This helper function removes one byte from a file by shifting all subsequent
/// bytes left by one position, then truncating the file to new length.
///
/// **Operation:**
/// ```text
/// Before: [A B C D E F]
///         Delete byte at position 3
/// After:  [A B C E F]
///                 ↑ D removed, E shifted left
/// ```
///
/// # Memory Safety - Stack Allocated Buffer
///
/// Uses 8KB stack buffer for shifting bytes after deletion point.
/// - No heap allocation for data processing
/// - Fixed-size buffer regardless of file size
/// - If file has > 8KB after deletion point, shifts occur in 8KB chunks
///
/// # Arguments
///
/// * `file_path` - Path to target file (read+write access required)
/// * `position` - Byte offset to delete (0 = first byte, file_size-1 = last byte)
///
/// # Returns
///
/// * `Ok(())` - Byte deleted successfully, file shortened by 1 byte
/// * `Err(io::Error)` - File operation failed (open, seek, read, write, truncate, flush)
///
/// # Algorithm
///
/// 1. Open file in read+write mode
/// 2. Seek to position+1 (first byte to keep)
/// 3. Read bytes after deletion point into buffer (up to 8KB)
/// 4. Seek back to deletion position
/// 5. Write shifted bytes (from buffer)
/// 6. Truncate file to new length (original - 1 byte)
/// 7. Flush to ensure data written to disk
///
/// # Edge Cases
///
/// **Delete last byte (position == file_size - 1):**
/// - Read after position+1 returns 0 bytes
/// - Nothing to shift
/// - File truncated by 1 byte
/// - Most efficient case
///
/// **Delete first byte (position == 0):**
/// - Reads entire file into buffer (up to 8KB)
/// - Writes at position 0 (original position 1 bytes)
/// - All bytes shifted left
/// - Most expensive case
///
/// **Delete with > 8KB after deletion point:**
/// - Only first 8KB shifted
/// - **BUG:** Bytes beyond 8KB not shifted, file corrupted
/// - Should loop-shift in chunks
/// - Current implementation assumes remaining bytes < 8KB
///
/// **Delete beyond EOF (position >= file_size):**
/// - Read returns 0 bytes
/// - Write does nothing
/// - Truncate sets file size to position (might grow file!)
/// - Unexpected behavior - should validate position < file_size
///
/// **Empty file (file_size == 0):**
/// - Any position is invalid
/// - Read returns 0 bytes
/// - Truncate sets size to position (creates zero-byte file)
/// - Should error if file empty
///
/// # Defensive Programming
///
/// - No unwrap calls
/// - All I/O operations explicitly error-checked
/// - Truncate ensures file size reflects deletion
/// - Flush called to ensure disk write
///
/// # Performance
///
/// - **Time:** O(M) where M = bytes after deletion point (up to 8KB)
/// - **Space:** O(1) - fixed 8KB stack buffer
/// - **I/O:** 1 read, 2 seeks, 1 write, 1 truncate, 1 flush = 6 operations
/// - Not optimized for repeated deletions (each call shifts independently)
///
/// # Known Limitations
///
/// **8KB shift buffer limit:**
/// If file has > 8KB bytes after deletion point:
/// - Only first 8KB shifted correctly
/// - Data beyond 8KB lost
/// - Should loop to shift all remaining bytes
///
/// **No validation:**
/// Doesn't check if position is valid (< file_size)
/// Invalid position causes undefined behavior
///
/// **No atomic operation:**
/// If write or truncate fails mid-operation, file left inconsistent.
/// No rollback mechanism.
///
/// # See Also
///
/// * `insert_bytes_at_position()` - Inverse operation (adds bytes)
/// * `insert_file_at_cursor()` - Caller that uses this for final byte removal
fn delete_byte_at_position(file_path: &Path, position: u64) -> io::Result<()> {
    // Open file for read+write
    // Requires file already exists
    let mut file = OpenOptions::new().read(true).write(true).open(file_path)?;

    // Pre-allocated buffer for bytes after deletion point
    // 8KB chosen as balance between stack usage and shift efficiency
    const BUFFER_SIZE: usize = 8192;
    let mut after_buffer = [0u8; BUFFER_SIZE];

    // Seek to position+1 (skip the byte being deleted)
    // Read bytes that need to be shifted left
    file.seek(SeekFrom::Start(position + 1))?;
    let bytes_after = file.read(&mut after_buffer)?;

    // Seek back to deletion position
    // Write the shifted bytes starting at deletion position
    file.seek(SeekFrom::Start(position))?;
    file.write_all(&after_buffer[..bytes_after])?;

    // Truncate file to new length (original size - 1 byte)
    // This removes the duplicate byte at end that resulted from shift-left
    let new_length = position + bytes_after as u64;
    file.set_len(new_length)?;

    // Flush to ensure data written to disk
    file.flush()?;

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
pub fn insert_text_chunk_at_cursor_position(
    state: &mut EditorState,
    file_path: &Path,
    text_bytes: &[u8],
) -> Result<()> {
    let file_pos = match state
        .window_map
        .get_file_position(state.cursor.row, state.cursor.col)
    {
        Ok(Some(pos)) => pos,
        Ok(None) => {
            // Cursor not on valid position - log and return without crashing
            eprintln!("Warning: Cannot insert - cursor not on valid file position");
            log_error(
                "Insert newline failed: cursor not on valid file position",
                Some("insert_newline_at_cursor"),
            );
            return Ok(()); // Return success but do nothing
        }
        Err(e) => {
            // Error getting position - log and return
            eprintln!("Warning: Cannot get cursor position: {}", e);
            log_error(
                &format!("Insert newline failed: {}", e),
                Some("insert_newline_at_cursor"),
            );
            return Ok(()); // Return success but do nothing
        }
    };

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

    // ==========================================
    //  Check if cursor exceeded right edge
    // ==========================================
    let right_edge = state.effective_cols.saturating_sub(1);

    if state.cursor.col > right_edge {
        // Calculate how far past edge we went
        let overflow = state.cursor.col - right_edge;

        // Scroll window right to accommodate
        state.horizontal_utf8txt_line_char_offset += overflow;

        // Move cursor back to right edge
        state.cursor.col = right_edge;

        // Rebuild window to show new viewport
        build_windowmap_nowrap(state, file_path)?;
    }

    Ok(())
}

// ===============
//  Have a Pasty!!
// ===============

/// Copies visual selection from source file to clipboard file with UTF-8 safety
///
/// # Purpose
/// Extracts bytes from a visual selection in the source document and saves them
/// as a new clipboard file. Handles multi-byte UTF-8 characters correctly by
/// ensuring character boundaries are not split. Generates human-readable filenames
/// from selection content (alphanumeric extraction).
///
/// # High-Level Workflow
/// ```text
/// 1. Normalize selection range (handle forward/backward selection)
/// 2. Adjust end position to include complete UTF-8 character
///    - If end points to start of multi-byte char, find its last byte
///    - Example: 花 (3 bytes) → ensures all bytes included
/// 3. Ensure clipboard directory exists (create if needed)
/// 4. Generate unique filename from selection content
///    - Extract alphanumeric chars for readable name
///    - Handle collisions with _2, _3, etc.
/// 5. Copy byte range to clipboard file (one byte at a time)
/// 6. Return Ok(()) on success
/// ```
///
/// # UTF-8 Character Boundary Safety
///
/// **Critical:** Selection end positions are byte offsets, not character offsets.
/// If user selects text ending with multi-byte character (e.g., Kanji, emoji),
/// the end position might point to the **start byte** of that character.
///
/// **Example without adjustment:**
/// ```text
/// Text: "hello 花"
/// 花 = 0xE8 0x8A 0xB1 (3 bytes at positions 6,7,8)
/// User selects to position 6 (start of 花)
/// Copy bytes 0-6 → gets "hello \xE8" ❌ CORRUPTED
/// ```
///
/// **Example with adjustment:**
/// ```text
/// Text: "hello 花"
/// User selects to position 6 (start of 花)
/// find_utf8_char_end(6) → returns 8 (last byte of 花)
/// Copy bytes 0-8 → gets "hello 花" ✓ COMPLETE
/// ```
///
/// This adjustment is performed by `find_utf8_char_end()`, which:
/// - Reads first byte at end position
/// - Determines character length from UTF-8 encoding pattern
/// - Calculates position of last byte in character
/// - Returns adjusted end position
///
/// # Arguments
///
/// * `state` - Editor state containing:
///   - `file_position_of_vis_select_start` - Selection start byte offset (inclusive)
///   - `file_position_of_vis_select_end` - Selection end byte offset (inclusive)
///   - `session_directory_path` - Root directory for session data
///   - Used to modify: (none - state not changed by this function)
///
/// * `source_file_path` - Absolute path to document being copied from
///   - Must exist and be readable
///   - Selection byte positions are relative to this file
///
/// # Returns
///
/// * `Ok(())` - Selection copied successfully to clipboard file
/// * `Err(LinesError)` - Operation failed at some stage
///
/// # Error Conditions
///
/// Returns `Err` with detailed context if:
/// - Selection range invalid (start > end after normalization)
/// - Session directory path not initialized in state
/// - Cannot create clipboard directory (permissions, disk space)
/// - Cannot read source file for filename generation (permissions, hardware)
/// - Cannot determine UTF-8 character boundary (corrupted file, invalid UTF-8)
/// - All 1000 filename variants already exist (hash collision)
/// - Cannot copy bytes to clipboard file (permissions, disk full, hardware)
///
/// # Memory Safety
///
/// **Stack allocations only:**
/// - No heap allocation for data processing
/// - Filename generation: 16-byte buffer for alphanumeric extraction
/// - Byte copying: 1-byte buffer for sequential read/write
///
/// **Never loads entire selection:**
/// - Selection may be gigabytes - never loaded into memory
/// - All operations byte-by-byte or small fixed buffers
/// - Per NASA Rule 3: pre-allocate all memory
///
/// # Clipboard Organization
///
/// **Directory structure:**
/// ```text
/// <session_dir>/
///   clipboard/
///     HelloWorld       ← alphanumeric from "Hello, World!"
///     test123          ← alphanumeric from "test 123 !!!"
///     item             ← fallback when no alphanumeric found
///     item_2           ← collision resolution
///     README_3         ← collision resolution for "README"
/// ```
///
/// **File naming policy:**
/// - Extract first 16 alphanumeric characters (a-z, A-Z, 0-9)
/// - Skip punctuation, whitespace, special characters
/// - Use "item" if no alphanumeric characters found
/// - Append _2, _3, ... _1000 to resolve name collisions
/// - No file extensions - clipboard files are raw byte copies
///
/// **Filename generation algorithm:**
/// ```text
/// 1. Read up to 16 bytes from selection start
/// 2. Extract ASCII alphanumeric only
/// 3. Convert to string (e.g., "Hello123")
/// 4. Check if clipboard/Hello123 exists
/// 5. If exists, try Hello123_2, Hello123_3, ..., Hello123_1000
/// 6. If all 1000 slots taken, return error
/// 7. Return unique filename (no path, no extension)
/// ```
///
/// # Selection Direction Handling
///
/// Visual selection can be forward or backward:
/// ```text
/// Forward:  start=10, end=20 → copy bytes 10-20
/// Backward: start=20, end=10 → normalize to 10-20, copy bytes 10-20
/// ```
///
/// Normalization by `normalize_pasty_selection_range()`:
/// - Compares start and end positions
/// - Returns `(min, max)` tuple ensuring start ≤ end
/// - Both positions remain inclusive after normalization
///
/// # Byte Position Semantics
///
/// **All positions are 0-indexed byte offsets:**
/// - Position 0 = first byte of file
/// - Position N = (N+1)th byte of file
/// - Both start and end are **inclusive**
///
/// **Inclusive range examples:**
/// ```text
/// start=0, end=0   → Copy 1 byte (byte 0)
/// start=0, end=3   → Copy 4 bytes (bytes 0,1,2,3)
/// start=5, end=5   → Copy 1 byte (byte 5)
/// ```
///
/// **Range calculation:**
/// ```text
/// bytes_to_copy = (end - start) + 1
/// Example: (3 - 0) + 1 = 4 bytes ✓
/// ```
///
/// # Edge Cases
///
/// **Empty selection (0 bytes):**
/// - Not possible: start and end are always equal or different
/// - Minimum selection is 1 byte (start == end)
/// - Single byte selection is valid
///
/// **Selection ends mid-character:**
/// - Handled by `find_utf8_char_end()` adjustment
/// - Ensures complete character copied
/// - Example: Select up to 2nd byte of 花 → adjusted to include all 3 bytes
///
/// **Selection contains only non-alphanumeric:**
/// - Example: "!@#$%^&*()"
/// - Filename generation uses fallback: "item"
/// - File content still copied correctly (raw bytes preserved)
///
/// **Selection starts mid-character:**
/// - Not adjusted - start position used as-is
/// - May result in partial character at start (corrupted)
/// - Current design: only adjust end, not start (room for improvement)
///
/// **Selection spans multi-byte characters:**
/// - Example: "hello 花 world 🌟"
/// - All bytes copied correctly (byte-by-byte copy)
/// - End adjustment ensures last character complete
/// - Filename: "helloworld" (alphanumeric only)
///
/// **Very large selection (gigabytes):**
/// - Memory safe: never loads entire selection
/// - Time: slow (one byte at a time)
/// - Storage: creates file of equal size
/// - No size limit enforced (disk space is limit)
///
/// **Filename collision cascade:**
/// - "test" exists → try "test_2"
/// - "test_2" exists → try "test_3"
/// - ... continues to "test_1000"
/// - If all 1000 exist → return error
///
/// **Session directory not initialized:**
/// - Returns error immediately
/// - No clipboard operation attempted
/// - Error message: "Session directory path is not initialized"
///
/// **Source file modified during copy:**
/// - Not detected or handled
/// - Byte positions may become invalid mid-operation
/// - May copy garbage data or fail with I/O error
/// - Defensive note: caller should ensure file stable
///
/// # Integration with Editor Modes
///
/// **Called by:**
/// - Visual mode: 'y' (yank) command
/// - Visual mode: 'c' (change/copy) command
/// - Both commands select text, then call this function
///
/// **Preconditions:**
/// - Visual selection active (start and end positions set)
/// - Source file exists and readable
/// - Session directory initialized
///
/// **Postconditions:**
/// - New file created in clipboard directory
/// - File contains exact byte copy of selection (UTF-8 safe)
/// - Editor state unchanged (selection still active)
/// - Can paste from clipboard using Pasty mode
///
/// # Performance Characteristics
///
/// **Time complexity:**
/// - O(N) where N = selection size in bytes
/// - One byte at a time (no buffering for correctness)
/// - Sequential I/O (no random seeks during copy)
///
/// **Space complexity:**
/// - O(1) - fixed-size stack buffers only
/// - 16-byte filename buffer + 1-byte copy buffer = 17 bytes
/// - No growth with selection size
///
/// **I/O operations:**
/// - Filename generation: Up to 16 sequential reads from source
/// - Filename collision check: Up to 1000 directory lookups
/// - Byte copy: N sequential reads + N sequential writes (where N = selection size)
/// - Total: O(N) I/O operations
///
/// # Defensive Programming
///
/// **Guards against:**
/// - Cosmic ray bit flips: Validates all calculations, checks all returns
/// - Hardware failures: All I/O operations return Result, explicitly handled
/// - Filesystem corruption: Bounded loops, validates file existence
/// - Invalid UTF-8: find_utf8_char_end handles gracefully, returns error
/// - Disk full: File write errors caught and returned
/// - Permission errors: Directory creation and file operations checked
///
/// **Bounded operations:**
/// - Filename generation: Max 1024 bytes read (safety limit)
/// - Collision resolution: Max 1000 attempts
/// - Byte copy: Bounded by selection size (validated)
///
/// **No unwrap, no panic in production:**
/// - All Results explicitly handled with `?` or match
/// - Error context logged before returning
/// - Uses defensive arithmetic (saturating_sub, saturating_add)
///
/// # Example Usage
///
/// ```no_run
/// # use std::path::Path;
/// # fn example(state: &mut EditorState) -> Result<()> {
/// // User selects "Hello, 世界!" in visual mode and presses 'y'
/// // Selection: bytes 100-120 (includes multi-byte characters)
/// // state.file_position_of_vis_select_start = 100
/// // state.file_position_of_vis_select_end = 120
///
/// let source = Path::new("/home/user/document.txt");
///
/// // Copy selection to clipboard
/// copy_selection_to_clipboardfile(state, source)?;
///
/// // Result:
/// // - File created: <session_dir>/clipboard/Hello
/// // - Contains UTF-8 bytes: "Hello, 世界!"
/// // - Multi-byte characters complete and uncorrupted
/// // - Can paste via Pasty mode
/// # Ok(())
/// # }
/// ```
///
/// # Policy Notes
///
/// **No automatic clipboard management:**
/// - Old clipboard items not auto-deleted
/// - User must manually clear via Pasty mode
/// - All clipboard items preserved across sessions
///
/// **No clipboard size limits:**
/// - Selection size unlimited (disk space is limit)
/// - Number of clipboard items unlimited (up to filesystem limits)
/// - No auto-cleanup of old items
///
/// **Filename conflicts resolved, not prevented:**
/// - No attempt to predict or prevent collisions
/// - Simple numbered suffix strategy (_2, _3, etc.)
/// - Limit of 1000 variants per base name
///
/// **UTF-8 safety philosophy:**
/// - End position adjusted to preserve complete characters
/// - Start position not adjusted (may begin mid-character)
/// - Byte-level operations preserve all data as-is
/// - No character encoding conversion
///
/// # See Also
///
/// * `normalize_pasty_selection_range()` - Handles forward/backward selection
/// * `find_utf8_char_end()` - UTF-8 character boundary detection
/// * `generate_clipboard_filename()` - Alphanumeric extraction for names
/// * `append_bytes_from_file_to_file()` - Low-level byte copying
/// * `pasty_mode()` - Clipboard browsing and paste interface
/// * `insert_file_at_cursor()` - Used by paste to insert clipboard files
///
/// # Testing Considerations
///
/// Test with selections containing:
/// - Pure ASCII text
/// - Multi-byte UTF-8 (Kanji, emoji, accented characters)
/// - Selection ending exactly on multi-byte character start
/// - Selection ending mid-multi-byte character
/// - Only punctuation (tests fallback filename)
/// - Very long alphanumeric string (tests 16-char limit)
/// - Duplicate selections (tests collision resolution)
/// - 1-byte selection
/// - Large selection (megabytes)
/// - Forward and backward selections
/// - Selection at start of file (byte 0)
/// - Selection at end of file
pub fn copy_selection_to_clipboardfile(
    state: &mut EditorState,
    source_file_path: &Path,
) -> Result<()> {
    // Step 1: Normalize selection
    let (start, end) = normalize_pasty_selection_range(
        state.file_position_of_vis_select_start,
        state.file_position_of_vis_select_end,
    )?;

    // Step 1.5: Adjust end position to include complete UTF-8 character
    // If end points to start of multi-byte char (like 花), find its last byte
    // Example: end=7 for 花 at bytes [7,8,9] → adjusted_end=9
    let adjusted_end = find_utf8_char_end(source_file_path, end)?;

    // Step 2: Get clipboard directory
    let clipboard_dir = state
        .session_directory_path
        .as_ref()
        .ok_or_else(|| {
            log_error(
                "Session directory path is not set",
                Some("copy_selection_to_clipboardfile"),
            );
            LinesError::StateError("Session directory path is not initialized".into())
        })?
        .join("clipboard");

    // Create clipboard directory if it doesn't exist
    if !clipboard_dir.exists() {
        fs::create_dir_all(&clipboard_dir)?;
    }

    // Step 3: Generate filename
    let filename =
        generate_clipboard_filename(start, adjusted_end, source_file_path, &clipboard_dir)?;

    // Step 4: Copy selection to clipboard file using adjusted end
    let clipboard_path = clipboard_dir.join(&filename);
    append_bytes_from_file_to_file(source_file_path, start, adjusted_end, &clipboard_path)?;

    Ok(())
}

/// Checks if a file byte position is within the current visual selection
///
/// # Purpose
/// Determines if a given byte offset falls within the selected range.
/// Handles both forward and backward selections.
///
/// # Arguments
/// * `file_pos` - Byte offset in file to check
/// * `sel_start` - Selection start byte (may be > sel_end if backward select)
/// * `sel_end` - Selection end byte (may be < sel_start if backward select)
///
/// # Returns
/// * `true` if file_pos is within selection range (inclusive)
/// * `false` otherwise
///
/// # Examples
/// ```ignore
/// // Forward selection: bytes 10-20
/// is_in_selection(15, 10, 20) → true
/// is_in_selection(5, 10, 20) → false
///
/// // Backward selection: bytes 20-10
/// is_in_selection(15, 20, 10) → true
/// is_in_selection(5, 20, 10) → false
/// ```
fn is_in_selection(file_pos: u64, sel_start: u64, sel_end: u64) -> Result<bool> {
    // Normalize: ensure start ≤ end
    let (start, end) = if sel_start <= sel_end {
        (sel_start, sel_end)
    } else {
        (sel_end, sel_start)
    };

    // Check if position falls within normalized range (inclusive on both ends)
    Ok(file_pos >= start && file_pos <= end)
}

/// If: Backwards, Then: Makes Not Backwards
fn normalize_pasty_selection_range(start: u64, end: u64) -> Result<(u64, u64)> {
    if start <= end {
        Ok((start, end))
    } else {
        Ok((end, start))
    }
}

/// Finds the last byte position of a UTF-8 character starting at given position
///
/// # Purpose
/// Given a byte position pointing to the START of a UTF-8 character,
/// returns the position of the LAST byte of that character.
///
/// # Arguments
/// * `file_path` - Path to the UTF-8 encoded file
/// * `char_start_byte` - Byte offset pointing to start of UTF-8 character
///
/// # Returns
/// * `Ok(u64)` - Position of the last byte of the character
/// * `Err(LinesError)` - If file operations fail
///
/// # UTF-8 Character Length Detection
/// UTF-8 first byte patterns indicate character byte length:
/// - `0xxxxxxx` (0x00-0x7F): 1-byte character (ASCII) → returns same position
/// - `110xxxxx` (0xC0-0xDF): 2-byte character → returns position + 1
/// - `1110xxxx` (0xE0-0xEF): 3-byte character → returns position + 2
/// - `11110xxx` (0xF0-0xF7): 4-byte character → returns position + 3
///
/// # Example
/// ```ignore
/// // 花 (U+82B1) = E8 8A B1 (3 bytes) at position 7
/// find_utf8_char_end(path, 7) → Ok(9)  // Last byte at position 9
///
/// // ASCII 'a' = 0x61 (1 byte) at position 5
/// find_utf8_char_end(path, 5) → Ok(5)  // Last byte at position 5
/// ```
pub fn find_utf8_char_end(file_path: &Path, char_start_byte: u64) -> Result<u64> {
    // Open file for reading
    let mut file = File::open(file_path).map_err(|e| {
        log_error(
            &format!("Cannot open file for UTF-8 character end check: {}", e),
            Some("find_utf8_char_end"),
        );
        LinesError::Io(e)
    })?;

    // Seek to character start position
    file.seek(SeekFrom::Start(char_start_byte)).map_err(|e| {
        log_error(
            &format!("Cannot seek to byte {}: {}", char_start_byte, e),
            Some("find_utf8_char_end"),
        );
        LinesError::Io(e)
    })?;

    // Read first byte to determine character length
    let mut byte_buffer: [u8; 1] = [0; 1];

    match file.read(&mut byte_buffer) {
        Ok(0) => {
            // EOF reached - return start position
            Ok(char_start_byte)
        }
        Ok(_) => {
            let first_byte = byte_buffer[0];

            // Determine character byte length from first byte bit pattern
            let char_byte_length: u64 = if first_byte < 0x80 {
                // 0xxxxxxx: 1-byte character (ASCII)
                1
            } else if (first_byte & 0b1110_0000) == 0b1100_0000 {
                // 110xxxxx: 2-byte character
                2
            } else if (first_byte & 0b1111_0000) == 0b1110_0000 {
                // 1110xxxx: 3-byte character (like 花)
                3
            } else if (first_byte & 0b1111_1000) == 0b1111_0000 {
                // 11110xxx: 4-byte character
                4
            } else {
                // Invalid UTF-8 or continuation byte - treat as 1 byte
                log_error(
                    &format!(
                        "Invalid UTF-8 start byte 0x{:02X} at position {}",
                        first_byte, char_start_byte
                    ),
                    Some("find_utf8_char_end"),
                );
                1
            };

            // Calculate last byte position of this character
            // For 1-byte char at position N: last byte is at N (0 additional bytes)
            // For 2-byte char at position N: last byte is at N+1 (1 additional byte)
            // For 3-byte char at position N: last byte is at N+2 (2 additional bytes)
            // For 4-byte char at position N: last byte is at N+3 (3 additional bytes)
            let last_byte_position = char_start_byte.saturating_add(char_byte_length - 1);

            Ok(last_byte_position)
        }
        Err(e) => {
            log_error(
                &format!("Error reading byte for UTF-8 character length: {}", e),
                Some("find_utf8_char_end"),
            );
            Err(LinesError::Io(e))
        }
    }
}

/// Creates a readable clipboard filename from selected text
///
/// # Purpose
/// Generates a unique filename based on alphanumeric characters extracted from
/// a byte range in a source file. Used for saving clipboard content with
/// human-readable names.
///
/// # Algorithm
/// 1. Reads up to 16 bytes from source file starting at `start_byte`
/// 2. Extracts ASCII alphanumeric characters only (a-z, A-Z, 0-9)
/// 3. Falls back to "item" if no valid characters found
/// 4. Checks for filename conflicts in clipboard directory
/// 5. Appends _2, _3, ... _1000 to resolve conflicts
/// 6. Returns unique filename string (no path, no extension)
///
/// # Arguments
/// * `start_byte` - Starting byte position in source file
/// * `end_byte` - Ending byte position in source file
/// * `source_file_path` - Path to file being read from
/// * `clipboard_path` - Session directory where clipboard files are stored
///
/// # Returns
/// * `Ok(String)` - Unique filename (just the name, no path or extension)
/// * `Err(LinesError)` - If file operations fail or all 1000 name variants exist
///
/// # Memory Safety
/// Uses only pre-allocated 16-byte buffer. Never loads entire files.
/// Reads source file incrementally, one byte at a time.
///
/// # Error Handling
/// - Invalid byte range (start > end)
/// - Source file open/seek/read failures
/// - Clipboard directory access failures
/// - All 1000 filename slots taken
///
/// # Example Filenames
/// - Source text "Hello World!" → "HelloWorld"
/// - Source text "123 test" → "123test"
/// - Source text "!@#$" → "item" (fallback)
/// - Conflict resolution → "item_2", "item_3", etc.
pub fn generate_clipboard_filename(
    start_byte: u64,
    end_byte: u64,
    source_file_path: &Path,
    clipboard_path: &Path,
) -> Result<String> {
    // =========================================================================
    // VALIDATION: Check byte range validity
    // =========================================================================

    // Debug-Assert: Validate byte range in debug builds
    //
    //    =================================================
    // // Debug-Assert, Test-Asset, Production-Catch-Handle
    //    =================================================
    // This is not included in production builds
    // assert: only when running in a debug-build: will panic
    debug_assert!(start_byte <= end_byte, "start_byte must be <= end_byte");
    // This is not included in production builds
    // assert: only when running cargo test: will panic
    #[cfg(test)]
    assert!(start_byte <= end_byte, "start_byte must be <= end_byte");
    // Catch & Handle without panic in production
    // This IS included in production to safe-catch
    if !start_byte <= end_byte {
        // state.set_info_bar_message("Config error");
        return Err(LinesError::GeneralAssertionCatchViolation(
            "start_byte must be <= end_byte".into(),
        ));
    }

    // Production-Catch-Handle: Invalid byte range
    if start_byte > end_byte {
        log_error(
            &format!(
                "Invalid byte range: start={} > end={}",
                start_byte, end_byte
            ),
            Some("generate_clipboard_filename"),
        );
        return Err(LinesError::InvalidInput(
            "start_byte must be less than or equal to end_byte".into(),
        ));
    }

    // =========================================================================
    // STEP 1: Extract alphanumeric characters from source file
    // =========================================================================

    // Pre-allocated buffer for extracted name (max 16 ASCII chars)
    let mut name_buffer: [u8; 16] = [0; 16];
    let mut name_length: usize = 0;

    // Open source file for reading
    let mut file = File::open(source_file_path).map_err(|e| {
        log_error(
            &format!("Cannot open source file: {}", e),
            Some("generate_clipboard_filename"),
        );
        LinesError::Io(e)
    })?;

    // Seek to start position
    file.seek(SeekFrom::Start(start_byte)).map_err(|e| {
        log_error(
            &format!("Cannot seek to byte {}: {}", start_byte, e),
            Some("generate_clipboard_filename"),
        );
        LinesError::Io(e)
    })?;

    // Read bytes one at a time, extracting alphanumeric characters
    // Loop bounded by: selection size and buffer capacity
    let bytes_to_read = end_byte.saturating_sub(start_byte) + 1; // +1 for inclusive range
    let max_iterations = bytes_to_read.min(1024); // Safety limit: read max 1KB

    for iteration in 0..max_iterations {
        // Stop if buffer is full
        if name_length >= 16 {
            break;
        }

        // Stop if we've reached end of selection
        if iteration >= bytes_to_read {
            break;
        }

        // Read one byte
        let mut byte_buffer: [u8; 1] = [0; 1];
        match file.read(&mut byte_buffer) {
            Ok(0) => {
                // End of file reached
                break;
            }
            Ok(_) => {
                let byte = byte_buffer[0];

                // Check if byte is ASCII alphanumeric
                // a-z: 97-122, A-Z: 65-90, 0-9: 48-57
                let is_alphanumeric = (byte >= 48 && byte <= 57)  // 0-9
                    || (byte >= 65 && byte <= 90)  // A-Z
                    || (byte >= 97 && byte <= 122); // a-z

                if is_alphanumeric {
                    name_buffer[name_length] = byte;
                    name_length += 1;
                }
                // Skip non-alphanumeric bytes (punctuation, whitespace, etc.)
            }
            Err(e) => {
                // Read error - log and stop reading
                log_error(
                    &format!("Error reading source file: {}", e),
                    Some("generate_clipboard_filename"),
                );
                break;
            }
        }
    }

    // =========================================================================
    // STEP 2: Create base filename (or use fallback)
    // =========================================================================

    let base_name = if name_length == 0 {
        // No alphanumeric characters found - use fallback
        String::from("item")
    } else {
        // Convert extracted bytes to string
        // We know these are valid ASCII alphanumeric, so UTF-8 conversion is safe
        match std::str::from_utf8(&name_buffer[..name_length]) {
            Ok(s) => String::from(s),
            Err(e) => {
                // This should never happen with ASCII alphanumeric, but handle defensively
                log_error(
                    &format!("UTF-8 conversion error (using fallback): {}", e),
                    Some("generate_clipboard_filename"),
                );
                String::from("item")
            }
        }
    };

    // =========================================================================
    // STEP 3: Find unique filename (handle conflicts)
    // =========================================================================

    // Check if base name is available
    let candidate_path = clipboard_path.join(&base_name);

    if !candidate_path.exists() {
        // Base name is unique - return it
        return Ok(base_name);
    }

    // Base name exists - try numbered variants
    // Loop bounded: max 1000 attempts
    const MAX_ATTEMPTS: u32 = 1000;

    for suffix in 2..=MAX_ATTEMPTS {
        // Build candidate name with suffix
        // Pre-allocate string capacity to avoid heap reallocation
        let mut candidate_name = String::with_capacity(base_name.len() + 10);
        candidate_name.push_str(&base_name);
        candidate_name.push('_');
        candidate_name.push_str(&suffix.to_string());

        // Check if this candidate exists
        let candidate_path = clipboard_path.join(&candidate_name);

        if !candidate_path.exists() {
            // Found unique name
            return Ok(candidate_name);
        }
    }

    // =========================================================================
    // ERROR: All 1000 filename slots are taken
    // =========================================================================

    log_error(
        &format!(
            "All {} filename variants exist for base name: {}",
            MAX_ATTEMPTS, base_name
        ),
        Some("generate_clipboard_filename"),
    );

    Err(LinesError::StateError(format!(
        "Cannot generate unique filename - all {} variants of '{}' already exist",
        MAX_ATTEMPTS, base_name
    )))
}

/// Appends a range of bytes from one file to another, one byte at a time
///
/// # Purpose
/// Copies bytes from a specific byte range in a source file and appends them
/// to the end of a target file. This operation is performed ONE BYTE AT A TIME
/// to minimize memory usage and avoid loading entire files or sections into memory.
///
/// # Policy and Scope
/// This function has a deliberately minimal scope:
/// - Reads exactly 1 byte from source
/// - Writes exactly 1 byte to target
/// - Repeats for each byte in range
/// - No buffering beyond a single byte
/// - No file loading or pre-scanning
/// - No file size checks or metadata queries
/// - Creates target file if it doesn't exist
/// - Stops gracefully when bytes are unavailable
///
/// # Arguments
/// * `source_file_path` - Absolute path to the file to read bytes from
/// * `start_byte_position` - Zero-indexed position of first byte to copy (inclusive)
/// * `end_byte_position` - Zero-indexed position of last byte to copy (inclusive)
/// * `append_to_this_file_path` - Absolute path to the file to append bytes to
///
/// # Returns
/// * `Ok(())` - Operation completed successfully (or gracefully stopped)
/// * `Err(LinesError)` - Operation failed due to file system error
///
/// # Behavior Details
/// - **Memory usage:** Exactly 1 byte (`u8`) at a time - no buffer
/// - **Target file:** Created if doesn't exist, appended if exists
/// - **Source file missing:** Returns `Ok(())` with no action
/// - **Byte not found:** Stops immediately and returns `Ok(())`
/// - **Write failure:** Returns `Err()` immediately
/// - **Byte positions:** Both start and end are inclusive (0-indexed)
/// - **Loop bound:** `(end - start + 1)` iterations maximum
///
/// # Graceful Stop Conditions (returns Ok with no error)
/// - Source file does not exist
/// - Start position has no byte available
/// - Any position in range has no byte available (stops at that point)
/// - End of file reached before end_byte_position
///
/// # Error Conditions (returns Err)
/// - Invalid byte range: start position > end position
/// - Cannot create target file (permissions, disk space)
/// - Cannot open source file (permissions, hardware failure)
/// - Cannot open target file (permissions, hardware failure)
/// - Cannot seek to position (hardware failure)
/// - Cannot read byte (hardware failure, cosmic ray bit flip)
/// - Cannot write byte (disk full, hardware failure, cosmic ray bit flip)
/// - Cannot flush target file (hardware failure)
///
/// # Safety and Reliability
/// - No unsafe code
/// - No recursion
/// - Loop has strict upper bound
/// - All errors handled without panic in production
/// - Uses debug_assert for debug builds
/// - Uses #[cfg(test)] assert for testing release builds
/// - Production code catches violations and returns error
/// - Never unwrap() - all Results handled explicitly
///
/// # Edge Cases
/// - `start_byte_position == end_byte_position`: Copies exactly 1 byte
/// - Empty source file: Returns Ok() immediately when first byte not found
/// - Start position at EOF: Returns Ok() immediately
/// - End position beyond EOF: Copies until last available byte, then returns Ok()
/// - Target file doesn't exist: Created automatically
/// - Large byte ranges: Handled safely with loop upper bound
///
/// # Example
/// ```no_run
/// # use std::path::Path;
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Copy bytes 10 through 20 (inclusive) from source.txt
/// // and append them to the end of target.txt
/// append_bytes_from_file_to_file(
///     Path::new("/absolute/path/to/source.txt"),
///     10,
///     20,
///     Path::new("/absolute/path/to/target.txt"),
/// )?;
/// # Ok(())
/// # }
/// ```
///
/// # Use Case Example
/// When building a file from fragments without loading entire files:
/// ```no_run
/// # use std::path::Path;
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let source = Path::new("/data/large_file.dat");
/// let output = Path::new("/data/output.dat");
///
/// // Append header (first 512 bytes)
/// append_bytes_from_file_to_file(source, 0, 511, output)?;
///
/// // Append specific data section (bytes 1024-2047)
/// append_bytes_from_file_to_file(source, 1024, 2047, output)?;
///
/// // Append footer (last 256 bytes, assuming we know the positions)
/// append_bytes_from_file_to_file(source, 999744, 999999, output)?;
/// # Ok(())
/// # }
/// ```
pub fn append_bytes_from_file_to_file(
    source_file_path: &Path,
    start_byte_position: u64,
    end_byte_position: u64,
    append_to_this_file_path: &Path,
) -> Result<()> {
    // ========================================================================
    // INPUT VALIDATION
    // ========================================================================

    // Validate byte positions: start must not be greater than end
    // This is a logic error in the caller's arguments
    if start_byte_position > end_byte_position {
        let error_msg = format!(
            "Invalid byte range: start position ({}) is greater than end position ({})",
            start_byte_position, end_byte_position
        );
        log_error(&error_msg, Some("append_bytes_from_file_to_file"));
        return Err(LinesError::InvalidInput(error_msg));
    }

    // ========================================================================
    // SOURCE FILE EXISTENCE CHECK
    // ========================================================================

    // Check if source file exists
    // If source doesn't exist, there's nothing to copy - return gracefully
    // This is not an error - it's a no-op situation
    if !source_file_path.exists() {
        return Ok(());
    }

    // ========================================================================
    // OPEN SOURCE FILE FOR READING
    // ========================================================================

    // Open source file for reading
    // If we can't open it (permissions, hardware failure), this is an error
    let mut source_file = match File::open(source_file_path) {
        Ok(file) => file,
        Err(e) => {
            let error_msg = format!("Cannot open source file: {}", e);
            log_error(&error_msg, Some("append_bytes_from_file_to_file"));
            return Err(LinesError::Io(e));
        }
    };

    // ========================================================================
    // OPEN OR CREATE TARGET FILE FOR APPENDING
    // ========================================================================

    // Open (or create) target file for appending
    // OpenOptions::create(true) - create file if it doesn't exist
    // OpenOptions::append(true) - append to end of file (don't overwrite)
    let mut target_file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(append_to_this_file_path)
    {
        Ok(file) => file,
        Err(e) => {
            let error_msg = format!("Cannot open or create target file: {}", e);
            log_error(&error_msg, Some("append_bytes_from_file_to_file"));
            return Err(LinesError::Io(e));
        }
    };

    // ========================================================================
    // CALCULATE LOOP UPPER BOUND
    // ========================================================================

    // Calculate total number of bytes to copy (for loop upper bound)
    // Formula: (end - start + 1) because both positions are inclusive
    // Example: bytes 5 to 7 inclusive = positions [5,6,7] = 3 bytes = (7-5+1)
    // Use saturating arithmetic to prevent overflow (cosmic ray protection)
    let total_bytes_to_copy = end_byte_position
        .saturating_sub(start_byte_position)
        .saturating_add(1);

    // =================================================
    // Debug-Assert, Test-Asset, Production-Catch-Handle
    // =================================================
    // Defensive assertion: total_bytes_to_copy should never be zero
    // Given our validation above (start <= end), result should always be >= 1
    // If this triggers, indicates memory corruption or cosmic ray bit flip

    // Debug builds only: will panic to help catch bugs during development
    debug_assert!(
        total_bytes_to_copy > 0,
        "total_bytes_to_copy should be at least 1, got: {}",
        total_bytes_to_copy
    );

    // Test builds (including release testing): will panic during cargo test
    #[cfg(test)]
    assert!(
        total_bytes_to_copy > 0,
        "total_bytes_to_copy should be at least 1, got: {}",
        total_bytes_to_copy
    );

    // Production builds: catch and handle without panic
    if total_bytes_to_copy == 0 {
        let error_msg = "Invalid byte range calculation resulted in zero bytes to copy";
        log_error(error_msg, Some("append_bytes_from_file_to_file"));
        return Err(LinesError::GeneralAssertionCatchViolation(error_msg.into()));
    }

    // ========================================================================
    // ALLOCATE SINGLE BYTE BUFFER
    // ========================================================================

    // Single byte buffer - we read exactly one byte at a time
    // This is our only memory allocation - exactly 1 byte
    // No buffering, no loading files or sections into memory
    let mut single_byte_buffer: [u8; 1] = [0];

    // ========================================================================
    // SEEK TO START POSITION
    // ========================================================================

    // Seek to start position in source file
    // SeekFrom::Start is absolute positioning from beginning of file
    // If we can't seek (hardware failure, invalid position), return error
    if let Err(e) = source_file.seek(SeekFrom::Start(start_byte_position)) {
        let error_msg = format!(
            "Cannot seek to start position {} in source file: {}",
            start_byte_position, e
        );
        log_error(&error_msg, Some("append_bytes_from_file_to_file"));
        return Err(LinesError::Io(e));
    }

    // ========================================================================
    // MAIN LOOP: COPY BYTES ONE AT A TIME
    // ========================================================================

    // Loop through each byte position from start to end (inclusive)
    // Upper bound: total_bytes_to_copy ensures loop terminates
    // No recursion - simple for-loop with known upper bound
    for byte_index in 0..total_bytes_to_copy {
        // Calculate current absolute position for error messages
        // Using saturating_add to protect against overflow
        let current_position = start_byte_position.saturating_add(byte_index);

        // ====================================================================
        // READ ONE BYTE FROM SOURCE
        // ====================================================================

        // Try to read exactly 1 byte from source file at current position
        // read_exact() will:
        // - Read exactly 1 byte if available
        // - Return UnexpectedEof if no byte at this position
        // - Return other errors for hardware failures
        match source_file.read_exact(&mut single_byte_buffer) {
            Ok(()) => {
                // Successfully read 1 byte into single_byte_buffer
                // Continue to write it to target
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                // Reached end of file - no more bytes available at this position
                // This is a GRACEFUL STOP condition, not an error
                // We copied all available bytes up to EOF
                return Ok(());
            }
            Err(e) => {
                // Other read error (hardware failure, permissions, cosmic ray bit flip)
                // This IS an error - log it and return
                let error_msg = format!(
                    "Cannot read byte at position {} in source file: {}",
                    current_position, e
                );
                log_error(&error_msg, Some("append_bytes_from_file_to_file"));
                return Err(LinesError::Io(e));
            }
        }

        // ====================================================================
        // WRITE ONE BYTE TO TARGET
        // ====================================================================

        // Try to write the single byte to target file
        // write_all() ensures the entire buffer (1 byte) is written
        // If write fails: disk full, hardware failure, permissions, cosmic ray bit flip
        if let Err(e) = target_file.write_all(&single_byte_buffer) {
            let error_msg = format!(
                "Cannot write byte from position {} to target file: {}",
                current_position, e
            );
            log_error(&error_msg, Some("append_bytes_from_file_to_file"));
            return Err(LinesError::Io(e));
        }

        // Successfully copied one byte from source to target
        // Continue to next byte in loop
    }

    // ========================================================================
    // FLUSH TARGET FILE
    // ========================================================================

    // All bytes copied successfully
    // Flush target file to ensure data is written to physical disk
    // This protects against data loss from power failure or system crash
    if let Err(e) = target_file.flush() {
        let error_msg = format!("Cannot flush target file to disk: {}", e);
        log_error(&error_msg, Some("append_bytes_from_file_to_file"));
        return Err(LinesError::Io(e));
    }

    // ========================================================================
    // SUCCESS
    // ========================================================================

    // All bytes successfully copied and flushed
    Ok(())
}

/// Reads clipboard directory and returns files sorted by modified time (newest first)
pub fn read_and_sort_pasty_clipboard(clipboard_dir: &PathBuf) -> io::Result<Vec<PathBuf>> {
    if !clipboard_dir.exists() {
        return Ok(Vec::new());
    }

    let mut files_with_time: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();

    // Read directory entries
    for entry in fs::read_dir(clipboard_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Only include files (not directories)
        if path.is_file() {
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    files_with_time.push((path, modified));
                }
            }
        }
    }

    // Sort by modified time (newest first)
    files_with_time.sort_by(|a, b| b.1.cmp(&a.1));

    // Extract just the paths
    Ok(files_with_time.into_iter().map(|(path, _)| path).collect())
}

/// Formats the Pasty legend with color-coded commands
fn format_pasty_tui_legend() -> io::Result<String> {
    Ok(format!(
        "{}Have a Pasty!! {}b{}ack paste{}N{} {}str{}{}(any file) {}clear{}all|{}clear{}N {}Empty{}(Add Freshest!){}",
        YELLOW,
        RED,
        YELLOW,
        RED,
        YELLOW,
        RED,
        YELLOW,
        YELLOW,
        RED,
        YELLOW,
        RED,
        RESET,
        RED,
        YELLOW,
        RESET
    ))
}

/// Formats the Pasty info bar with count, pagination, and error messages
fn format_pasty_info_bar(
    total_count: usize,
    first_count_visible: usize,
    last_count_visible: usize,
    info_bar_message: &str,
) -> io::Result<String> {
    let infobar_message_display = if !info_bar_message.is_empty() {
        format!(" {}", info_bar_message)
    } else {
        String::new()
    };

    Ok(format!(
        // "{}{}{}Total, {}Showing{} {}{}-{}{}{} (Page up/down k/j) {}{} >{} ",  // minimal
        "{}{}{} Clipboard Items, {}Showing{} {}{}-{}{}{} (Page up/down k/j) {}{}\nEnter clipboard item # to paste, or a file-path to paste file text {}> ",
        RED,
        total_count,
        YELLOW,
        YELLOW,
        RED,
        first_count_visible,
        YELLOW,
        RED,
        last_count_visible,
        YELLOW,
        infobar_message_display,
        YELLOW,
        RESET
    ))
}

/// Clears all files from clipboard directory
fn clear_pasty_file_clipboard(clipboard_dir: &PathBuf) -> io::Result<()> {
    if !clipboard_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(clipboard_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            fs::remove_file(path)?;
        }
    }

    Ok(())
}

/// Resolves and prepares the target file path for editing
///
/// # Purpose
/// Handles all file path resolution logic, converting user input into
/// an absolute, validated file path ready for editing. Manages:
/// - Relative to absolute path conversion
/// - Directory vs file discrimination
/// - User prompting for missing filenames
/// - Parent directory creation
/// - Final path validation
///
/// # Arguments
/// * `original_file_path` - Optional path provided by user (file or directory)
///
/// # Returns
/// * `Ok(PathBuf)` - Absolute path to target file, ready for editing
/// * `Err(io::Error)` - Path resolution, validation, or directory creation failed
///
/// # Behavior by Input Type
/// * `None` - Returns `InvalidInput` error (full editor requires path)
/// * `Some(existing_file)` - Returns absolute path to existing file
/// * `Some(existing_dir)` - Prompts user for filename, returns `dir/filename`
/// * `Some(new_path/)` - Creates directory, prompts for filename, returns path
/// * `Some(new_path)` - Creates parent directories if needed, returns absolute path
///
/// # Edge Cases
/// - Empty path strings: Returns `InvalidInput` error
/// - Trailing path separators: Interpreted as directory request
/// - Missing parent directories: Created automatically with notification
/// - Relative paths: Converted to absolute based on current working directory
///
/// # Side Effects
/// - Creates directories on filesystem (with user notification)
/// - Prompts user for input via `prompt_for_filename()` when needed
/// - Prints status messages to stdout for transparency
///
/// # Error Conditions
/// - No path provided (None input)
/// - Empty resolved path
/// - Directory creation failure (permissions, disk space, etc.)
/// - User filename prompt failure or cancellation
/// - Current directory access failure (for relative path conversion)
fn resolve_target_file_path(original_file_path: Option<PathBuf>) -> io::Result<PathBuf> {
    // Require path in full editor mode (not optional like memo mode)
    let path = match original_file_path {
        None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "File path required in full editor mode. Usage: lines <filename>",
            ));
        }
        Some(p) => p,
    };

    // Convert to absolute path for consistency and safety
    let absolute_path = if path.is_absolute() {
        path.clone()
    } else {
        // Resolve relative to current working directory
        env::current_dir()?.join(&path)
    };

    // Route based on whether path exists and what type it is
    let target_path = if absolute_path.exists() {
        resolve_existing_path(absolute_path)?
    } else {
        resolve_new_path(path, absolute_path)?
    };

    // Defensive: Final validation before returning
    if target_path.to_string_lossy().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid file path: resolved to empty path",
        ));
    }

    Ok(target_path)
}

/// Handles resolution of paths that already exist on filesystem
///
/// # Purpose
/// Determines if existing path is a file (use as-is) or directory
/// (prompt for filename). Part of path resolution workflow.
///
/// # Arguments
/// * `absolute_path` - Existing absolute path to resolve
///
/// # Returns
/// * `Ok(PathBuf)` - Resolved file path (either original file or dir + prompted filename)
/// * `Err(io::Error)` - Filename prompting failed
///
/// # Behavior
/// - If path is file: returns path unchanged
/// - If path is directory: prompts user for filename, returns `dir/filename`
///
/// # Assertions
/// - Path must exist (caller's responsibility)
fn resolve_existing_path(absolute_path: PathBuf) -> io::Result<PathBuf> {
    // Defensive: Verify precondition
    debug_assert!(
        absolute_path.exists(),
        "resolve_existing_path called with non-existent path"
    );

    if absolute_path.is_dir() {
        // Directory: prompt user for filename to create within it
        println!("Directory specified: {}", absolute_path.display());
        let filename = prompt_for_filename()?;
        Ok(absolute_path.join(filename))
    } else {
        // Existing file: use as-is
        Ok(absolute_path)
    }
}

/// Handles resolution of paths that don't exist yet
///
/// # Purpose
/// Distinguishes between new file requests and new directory requests
/// based on trailing separators. Creates directories as needed.
/// Part of path resolution workflow.
///
/// # Arguments
/// * `original_path` - Original path as provided by user (may be relative)
/// * `absolute_path` - Absolute version of original path
///
/// # Returns
/// * `Ok(PathBuf)` - Resolved file path ready for creation
/// * `Err(io::Error)` - Directory creation or filename prompting failed
///
/// # Behavior
/// - Path ends with `/` or `\`: Creates directory, prompts for filename
/// - Path without separator: Creates parent dirs if needed, returns path
///
/// # Side Effects
/// - Creates directories on filesystem when needed
/// - Prompts user for filename when directory specified
/// - Prints status messages about directory creation
///
/// # Assertions
/// - Path must NOT exist (caller's responsibility)
fn resolve_new_path(original_path: PathBuf, absolute_path: PathBuf) -> io::Result<PathBuf> {
    // Defensive: Verify precondition
    debug_assert!(
        !absolute_path.exists(),
        "resolve_new_path called with existing path"
    );

    // Check if user specified a directory (trailing separator)
    let path_str = original_path.to_string_lossy();
    if path_str.ends_with('/') || path_str.ends_with('\\') {
        // Treat as directory that needs creating
        fs::create_dir_all(&absolute_path)?;
        println!("Created directory: {}", absolute_path.display());

        // Prompt for filename within new directory
        let filename = prompt_for_filename()?;
        Ok(absolute_path.join(filename))
    } else {
        // Treat as new file path - create parent directories if needed
        if let Some(parent) = absolute_path.parent() {
            if !parent.exists() {
                println!("Creating parent directories: {}", parent.display());
                fs::create_dir_all(parent)?;
            }
        }
        Ok(absolute_path)
    }
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
/// Session dir: `{executable_dir}/lines_data/sessions/2025_01_15_14_30_45/`
/// Read-copy: `{session_dir}/2025_01_15_14_30_45_file.txt`
///
/// # Design Notes
/// - NO hidden files (no leading dot) - files should be visible to user
/// - Stored in session directory for crash recovery
/// - Timestamp prefix ensures uniqueness
/// - Session directory persists after exit for recovery
pub fn create_a_readcopy_of_file(
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

    // // Diagnostic Log success for user visibility
    // println!("Read-copy created: {}", read_copy_path.display());

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
pub fn print_help() {
    println!("About Lines Editor:");
    println!("USAGE:");
    println!("    lines [FILE]");
    println!("    lines FILE:LINE          # Open at : specific line");
    println!("OPTIONS:");
    println!("    --help, -h      Show this help message");
    println!("    --version, -v   Show version information");
    println!("MODES:");
    println!("    Memo Mode:      Run from home directory, Append-only quickie");
    println!("                    Creates dated files in ~/Documents/lines_editor/");
    println!("    Full Editor:    Run from any other directory");
    println!("DELETE: d");
    println!("    Normal Mode: 'd' delete a line");
    println!("    Visual Mode  'd'/ Insert Mode '-d': Backspace-Style Delete");
    println!("NAVIGATION:");
    println!("    hjkl            Move cursor");
    println!("    5j, 10l         Move with repeat count");
    println!("    [Empty Enter]   Repeat last command (Normal/Visual/ ...?)");
    println!("-n -v -wq -q -s -d  Insert Mode: Flag style commands");
    println!("Examples in terminal/shell:");
    println!("  lines                Memo mode (if in home)");
    println!("  lines notes.txt      Create/open notes.txt");
    println!("  lines notes.txt:42    # Open to line 42");
    println!("  lines mydir/ Create new file in directory");
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
fn format_info_bar_cafe_normal_visualselect(state: &EditorState) -> Result<String> {
    /*
    Calculation note:
    The column number should be - the number of digits +1

    */
    // Get mode string
    let mode_str = match state.mode {
        EditorMode::Normal => "NORMAL",
        EditorMode::Insert => "INSERT",
        EditorMode::VisualSelectMode => "VISUAL",
        EditorMode::PastyMode => "PASTY",
        EditorMode::MultiCursor => "MULTI",
        EditorMode::HexMode => "HEX",
    };

    // Get current line and column
    // Line is 1-indexed for display (humans count from 1)
    let line_display = state.line_count_at_top_of_window + state.cursor.row + 1;

    // Get line number to calculate line number display width
    let line_num = state.line_count_at_top_of_window + state.cursor.row + 1;
    let line_num_width = calculate_line_number_width(line_num);

    // Add horizontal offset to get character position in line
    // Subtract line number width from displayed column
    let true_char_position = state.cursor.col + state.horizontal_utf8txt_line_char_offset;
    let col_display = true_char_position.saturating_sub(line_num_width) + 1;

    // Get filename (or "unnamed" if none)
    let filename = state
        .original_file_path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unmanned file");

    // Extract message from buffer (find null terminator or use full buffer)
    let message_len = state
        .info_bar_message_buffer
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(state.info_bar_message_buffer.len());

    let message_for_infobar =
        std::str::from_utf8(&state.info_bar_message_buffer[..message_len]).unwrap_or(""); // Empty string if invalid UTF-8

    // Build the info bar
    let info = format!(
        // "{}{}{} line{}{} {}col{}{}{} {}{} >{}",
        "{}{} {}{}{}:{}{}{} {}{} {}{} > ",
        YELLOW,
        mode_str,
        // YELLOW,
        RED,
        line_display,
        YELLOW,
        YELLOW,
        RED,
        col_display,
        YELLOW,
        filename,
        message_for_infobar,
        RESET,
    );

    Ok(info)
}

/// Hex editor display state
///
/// # Purpose
/// Tracks position within file for hex viewing/editing.
/// Separate from UTF-8 cursor position to avoid conflating byte-offset
/// with character-offset semantics.
///
/// # Fields
/// * `byte_offset` - Absolute position in file (0-indexed)
/// * `bytes_per_row` - Display width constant (26 for 80-char TUI)
pub struct HexCursor {
    /// Absolute byte position in file (0-indexed)
    /// Range: 0 to file_size
    pub byte_offset: usize,

    /// Number of bytes shown per display row
    /// Constant: 26 (fits in 80-char terminal width)
    pub bytes_per_row: usize,
}

impl HexCursor {
    /// Creates new hex cursor at file start
    ///
    /// # Returns
    /// Cursor positioned at byte 0, displaying 26 bytes per row
    pub fn new() -> Self {
        HexCursor {
            byte_offset: 0,
            bytes_per_row: 26,
        }
    }

    /// Calculates which display row this byte offset is on
    ///
    /// # Returns
    /// Row number (0-indexed)
    pub fn current_row(&self) -> usize {
        self.byte_offset / self.bytes_per_row
    }

    /// Calculates column within current row
    ///
    /// # Returns
    /// Column position (0-25 for 26 bytes per row)
    pub fn current_col(&self) -> usize {
        self.byte_offset % self.bytes_per_row
    }
}

/// Renders the complete TUI in hex mode
///
/// # Purpose
/// Displays hex editor view with:
/// 1. Top: Command legend (1 line, same as UTF-8 mode)
/// 2. Middle: Hex bytes + UTF-8 interpretation (2 lines)
/// 3. Bottom: Info bar (1 line, shows byte offset)
///
/// # Layout
/// ```text
/// quit ins vis save undo hjkl wb /search       <- Legend
/// 48 65 6C 6C 6F 20 57 6F 72 6C 64 0A 41 42   <- Hex bytes
/// H  e  l  l  o     W  o  r  l  d  ␊  A  B    <- UTF-8 chars
/// HEX byte 156 of 1024 doc.txt > cmd_         <- Info bar
/// ```
///
/// # Arguments
/// * `state` - Current editor state with hex_cursor position
///
/// # Returns
/// * `Ok(())` - Successfully rendered
/// * `Err(LinesError)` - Display or file read failed
///
/// # Design
/// - Shows exactly ONE row of file data (26 bytes)
/// - Cursor highlights current byte position
/// - Unprintable bytes shown as · in UTF-8 line
/// - Control characters shown with symbols (␊ for newline)
///
/// # File Reading
/// Reads only 26 bytes starting at `hex_cursor.byte_offset`
/// Does NOT load entire file into memory
pub fn render_tui_hex(state: &EditorState) -> Result<()> {
    // Clear screen
    print!("\x1B[2J\x1B[H");
    io::stdout()
        .flush()
        .map_err(|e| LinesError::DisplayError(format!("Failed to flush stdout: {}", e)))?;

    // === TOP LINE: LEGEND (same as UTF-8 mode) ===
    let legend = format_navigation_legend()?;
    println!("{}", legend);

    // padding
    for _ in 0..5 {
        println!();
    }

    // === MIDDLE: HEX + UTF-8 DISPLAY (2 lines) ===
    let hex_display = render_hex_row(state)?;
    print!("{}", hex_display);

    // padding
    for _ in 0..14 {
        println!();
    }

    // === BOTTOM LINE: INFO BAR ===
    let info_bar = format_hex_info_bar(state)?;
    print!("{}", info_bar);

    io::stdout()
        .flush()
        .map_err(|e| LinesError::DisplayError(format!("Failed to flush stdout: {}", e)))?;

    Ok(())
}

/// Renders one row of hex data with UTF-8 interpretation
///
/// # Purpose
/// Displays 26 bytes in two formats:
/// 1. Hex representation (with cursor highlighting)
/// 2. UTF-8 character representation
///
/// # Arguments
/// * `state` - Editor state with file path and hex cursor
///
/// # Returns
/// * `Ok(String)` - Two-line display string
/// * `Err(LinesError)` - File read failed
///
/// # Format
/// ```text
/// 48 65 6C 6C 6F 20 57 6F 72 6C 64 0A 41 42
/// H  e  l  l  o     W  o  r  l  d  ␊  A  B
/// ```
///
/// # IMPORTANT: Display Logic
/// The display shows the ENTIRE ROW containing the cursor, not starting from cursor.
///
/// Example: If cursor is at byte 28 (row 1, column 2):
/// - Row 1 starts at byte 26 (row * bytes_per_row = 1 * 26 = 26)
/// - Display bytes 26-51
/// - Highlight byte 28 (column 2 within that row)
///
/// This keeps the row stable as cursor moves within it.
///
/// # Cursor Highlighting
/// Current byte shown with: BOLD + RED + WHITE_BG
/// Example: `48` becomes `[1m[31m[47m48[0m`
///
/// # UTF-8 Handling
/// - Valid UTF-8 bytes shown as characters
/// - Invalid/unprintable shown as ·
/// - Control chars shown with Unicode symbols:
///   - 0x0A (newline) → ␊
///   - 0x09 (tab) → ␉
///   - 0x20 (space) → ⎕ (visible space)
///
/// # Memory Safety
/// - Pre-allocates 26-byte buffer
/// - Reads exactly 26 bytes (or less at EOF)
/// - No heap allocation during render
fn render_hex_row(state: &EditorState) -> Result<String> {
    const BYTES_TO_DISPLAY: usize = 26;
    const BOLD: &str = "\x1b[1m";
    const RED: &str = "\x1b[31m";
    const BG_WHITE: &str = "\x1b[47m";
    const RESET: &str = "\x1b[0m";

    // Pre-allocate display buffers
    // 26 bytes × 3 chars per byte ("48 ") = 78 chars + safety margin
    let mut hex_line = String::with_capacity(80);
    // 26 bytes × 3 chars per UTF-8 display ("H  ") = 78 chars + safety margin
    let mut utf8_line = String::with_capacity(80);

    // Pre-allocate byte buffer for file reading
    let mut byte_buffer = [0u8; BYTES_TO_DISPLAY];

    // Get file path from state
    let file_path = state
        .read_copy_path
        .as_ref()
        .ok_or_else(|| LinesError::StateError("No file path in hex mode".to_string()))?;

    // Open file
    let mut file = File::open(file_path).map_err(|e| LinesError::Io(e))?;

    // ===================================================================
    // KEY FIX: Calculate ROW START, not cursor position
    // ===================================================================
    // If cursor is at byte 28:
    //   - current_row() = 28 / 26 = 1 (integer division)
    //   - row_start_offset = 1 * 26 = 26
    //   - We display bytes 26-51 (the entire second row)
    //   - Cursor highlights byte 28 (column 2 of that row)
    // ===================================================================
    let current_row = state.hex_cursor.current_row();
    let row_start_offset = current_row * state.hex_cursor.bytes_per_row;

    // Seek to START OF ROW, not cursor position
    file.seek(io::SeekFrom::Start(row_start_offset as u64))
        .map_err(|e| LinesError::Io(e))?;

    // Read up to 26 bytes (may be less at EOF)
    let bytes_read = file.read(&mut byte_buffer).map_err(|e| LinesError::Io(e))?;

    // Calculate which byte position in this row is under cursor
    let cursor_col = state.hex_cursor.current_col();

    // Build hex line and UTF-8 line simultaneously
    for i in 0..BYTES_TO_DISPLAY {
        if i < bytes_read {
            let byte = byte_buffer[i];

            // === HEX LINE ===
            // Highlight if this is cursor position
            if i == cursor_col {
                hex_line.push_str(&format!(
                    "{}{}{}{:02X}{} ",
                    BOLD, RED, BG_WHITE, byte, RESET
                ));
            } else {
                hex_line.push_str(&format!("{:02X} ", byte));
            }

            // === UTF-8 LINE ===
            // Convert byte to displayable character
            let display_char = byte_to_display_char(byte);

            // Highlight if this is cursor position
            if i == cursor_col {
                utf8_line.push_str(&format!(
                    "{}{}{}{}{}  ",
                    BOLD, RED, BG_WHITE, display_char, RESET
                ));
            } else {
                utf8_line.push_str(&format!("{}  ", display_char));
            }
        } else {
            // Past EOF - show empty space
            hex_line.push_str("   "); // 3 spaces (matches "48 " width)
            utf8_line.push_str("   "); // 3 spaces (matches "H  " width)
        }
    }

    // Combine into two-line output
    let result = format!("{}\n{}\n", hex_line.trim_end(), utf8_line.trim_end());

    Ok(result)
}

/// Finds the next newline byte position after current cursor
///
/// # Purpose
/// Searches forward from current position to find next 0x0A byte.
/// Used for "next line" navigation in hex mode.
///
/// # Arguments
/// * `file_path` - Path to file to search
/// * `start_offset` - Byte position to start searching from (exclusive)
/// * `file_size` - Total file size for bounds checking
///
/// # Returns
/// * `Ok(Some(position))` - Found newline at this byte offset
/// * `Ok(None)` - No newline found before EOF
/// * `Err(e)` - File read error
///
/// # Search Strategy
/// Reads file in 256-byte chunks to avoid loading entire file.
/// Bounded by file size to prevent infinite loops.
///
/// # Memory Safety
/// - Pre-allocated 256-byte buffer (no dynamic allocation)
/// - Bounded iteration (stops at EOF)
/// - Returns position, not reference (no lifetime issues)
fn find_next_newline(
    file_path: &PathBuf,
    start_offset: usize,
    file_size: usize,
) -> io::Result<Option<usize>> {
    const SEARCH_CHUNK_SIZE: usize = 256;
    let mut buffer = [0u8; SEARCH_CHUNK_SIZE];

    let mut file = File::open(file_path)?;

    // Start search from byte AFTER current position
    let mut current_offset = start_offset + 1;

    // Defensive: don't start past EOF
    if current_offset >= file_size {
        return Ok(None);
    }

    // Bounded search: iterate through file in chunks
    let max_iterations = (file_size / SEARCH_CHUNK_SIZE) + 2; // +2 for safety
    let mut iteration = 0;

    while current_offset < file_size && iteration < max_iterations {
        iteration += 1;

        // Seek to current position
        file.seek(io::SeekFrom::Start(current_offset as u64))?;

        // Read chunk
        let bytes_read = file.read(&mut buffer)?;

        if bytes_read == 0 {
            break; // EOF
        }

        // Search for newline in this chunk
        for i in 0..bytes_read {
            if buffer[i] == 0x0A {
                return Ok(Some(current_offset + i));
            }
        }

        // Move to next chunk
        current_offset += bytes_read;
    }

    Ok(None) // No newline found
}

/// Finds the previous newline byte position before current cursor
///
/// # Purpose
/// Searches backward from current position to find previous 0x0A byte.
/// Used for "previous line" navigation in hex mode.
///
/// # Arguments
/// * `file_path` - Path to file to search
/// * `start_offset` - Byte position to start searching from (exclusive)
///
/// # Returns
/// * `Ok(Some(position))` - Found newline at this byte offset
/// * `Ok(None)` - No newline found before file start
/// * `Err(e)` - File read error
///
/// # Search Strategy
/// Reads file in 256-byte chunks backward from cursor position.
/// Stops at byte 0 (file start).
///
/// # Memory Safety
/// - Pre-allocated 256-byte buffer
/// - Bounded iteration (stops at offset 0)
/// - Underflow protection (checked subtraction)
fn find_previous_newline(file_path: &PathBuf, start_offset: usize) -> io::Result<Option<usize>> {
    const SEARCH_CHUNK_SIZE: usize = 256;
    let mut buffer = [0u8; SEARCH_CHUNK_SIZE];

    if start_offset == 0 {
        return Ok(None); // Already at start
    }

    let mut file = File::open(file_path)?;

    // Start search from byte BEFORE current position
    let mut current_offset = start_offset.saturating_sub(1);

    // Bounded search: maximum iterations
    let max_iterations = (start_offset / SEARCH_CHUNK_SIZE) + 2;
    let mut iteration = 0;

    loop {
        iteration += 1;

        if iteration > max_iterations {
            break; // Safety bound reached
        }

        // Calculate chunk start (search backward)
        let chunk_start = current_offset.saturating_sub(SEARCH_CHUNK_SIZE - 1);
        let chunk_size = current_offset - chunk_start + 1;

        // Seek to chunk start
        file.seek(io::SeekFrom::Start(chunk_start as u64))?;

        // Read chunk
        let bytes_read = file.read(&mut buffer[..chunk_size])?;

        if bytes_read == 0 {
            break; // Unexpected EOF
        }

        // Search backward through chunk
        for i in (0..bytes_read).rev() {
            if buffer[i] == 0x0A {
                return Ok(Some(chunk_start + i));
            }
        }

        // Move to previous chunk
        if chunk_start == 0 {
            break; // Reached file start
        }

        current_offset = chunk_start.saturating_sub(1);
    }

    Ok(None) // No newline found
}

/// Converts a byte to a displayable character for hex editor UTF-8 line
///
/// # Purpose
/// Maps bytes to visible characters for the UTF-8 interpretation line.
/// Makes control characters and unprintable bytes visible.
///
/// # Arguments
/// * `byte` - The byte value to convert (0x00 - 0xFF)
///
/// # Returns
/// A single character representing the byte
///
/// # Mapping Rules
/// 1. **Printable ASCII (0x20-0x7E)**: Display as-is
/// 2. **Space (0x20)**: Show as '·' (middle dot) for visibility
/// 3. **Common control characters**: Show with Unicode symbols
///    - 0x09 (tab) → '␉'
///    - 0x0A (line feed) → '␊'
///    - 0x0D (carriage return) → '␍'
/// 4. **Other control/unprintable**: Show as '·'
///
/// # Design Notes
/// - Always returns exactly one char (important for alignment)
/// - Non-panicking: all 256 byte values handled
/// - Unicode symbols from "Control Pictures" block (U+2400-U+2426)
pub fn byte_to_display_char(byte: u8) -> char {
    match byte {
        // Tab
        0x09 => '␉',
        // Line feed (newline)
        0x0A => '␊',
        // Carriage return
        0x0D => '␍',
        // Space - show as visible character
        0x20 => '⎕',
        // Printable ASCII range (excluding space, already handled)
        0x21..=0x7E => byte as char,
        // Everything else (control chars, high bytes)
        _ => '▚',
    }
}

/// Formats the info bar for hex mode
///
/// # Purpose
/// Shows hex-specific status information at bottom of TUI
///
/// # Arguments
/// * `state` - Editor state with hex cursor and file info
///
/// # Returns
/// * `Ok(String)` - Formatted info bar
/// * `Err(LinesError)` - Failed to get file size
///
/// # Format
/// ```text
/// HEX byte 156 of 1024 doc.txt > cmd_
/// ```
///
/// # Information Displayed
/// - Mode indicator: "HEX"
/// - Current byte offset (0-indexed, shown as 1-indexed for users)
/// - Total file size in bytes
/// - Filename (basename only, not full path)
/// - Command input indicator
fn format_hex_info_bar(state: &EditorState) -> Result<String> {
    // Get file size
    let file_size = match &state.read_copy_path {
        Some(path) => match fs::metadata(path) {
            Ok(metadata) => metadata.len() as usize,
            Err(_) => 0,
        },
        None => 0,
    };

    // Get filename (or "unnamed" if none)
    let filename = state
        .original_file_path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unmanned phile");

    // Build info bar
    // Show byte position as 1-indexed for human readability
    let info_bar = format!(
        "{}HEX byte {}{}{} of {}{}{} {} (Enter Hex to Edit Cursor) {}> ",
        YELLOW,
        RED,
        state.hex_cursor.byte_offset + 1, // Human-friendly: 1-indexed
        YELLOW,
        RED,
        file_size,
        YELLOW,
        filename,
        RESET,
    );

    Ok(info_bar)
}

/// Renders the complete UTF8-text TUI to terminal: legend + content + info bar
///
/// # Purpose
/// Displays the minimal 3-section TUI:
/// 1. Top: Command legend (1 line)
/// 2. Middle: File content (effective_rows lines)
/// 3. Bottom: Info bar with command input (1 line)
///
/// # Arguments
/// * `state` - Current editor state with display buffers
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
pub fn render_tui_utf8txt(state: &EditorState) -> Result<()> {
    // Clear screen
    print!("\x1B[2J\x1B[H");
    io::stdout()
        .flush()
        .map_err(|e| LinesError::DisplayError(format!("Failed to flush stdout: {}", e)))?;

    // === TOP LINE: LEGEND ===
    let legend = format_navigation_legend()?;
    println!("{}", legend);

    // === MIDDLE: FILE CONTENT WITH CURSOR ===
    // // Render each content row
    for row in 0..state.effective_rows {
        if state.display_utf8txt_buffer_lengths[row] > 0 {
            let row_content =
                &state.utf8_txt_display_buffers[row][..state.display_utf8txt_buffer_lengths[row]];

            match std::str::from_utf8(row_content) {
                Ok(row_str) => {
                    // ADD CURSOR HIGHLIGHTING HERE (was missing!)
                    let display_str = render_utf8txt_row_with_cursor(state, row, row_str)?;
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
    let info_bar = format_info_bar_cafe_normal_visualselect(state)?;
    print!("{}", info_bar);

    io::stdout()
        .flush()
        .map_err(|e| LinesError::DisplayError(format!("Failed to flush stdout: {}", e)))?;

    Ok(())
}
/// Renders one row of display with both cursor and visual selection highlighting
///
/// # Purpose
/// Takes a display row and adds:
/// 1. Cursor highlighting (RED + WHITE_BG) if cursor on this row - PRIORITY 1
/// 2. Visual selection highlighting (YELLOW + CYAN_BG) if in visual mode - PRIORITY 2
/// 3. Character-by-character highlighting via window_map
///
/// # Arguments
/// * `state` - Editor state (mode, cursor position, window_map)
/// * `row_index` - Display row being rendered (0-indexed)
/// * `row_content` - The text content for this row
///
/// # Returns
/// * `Ok(String)` - Row with highlighting applied
/// * `Err(LinesError)` - If window_map lookup fails or selection check fails
///
/// # Error Handling
/// All failures are propagated - no silent failures.
/// Window_map errors, selection calculation errors, all returned as Err.
///
/// # Design Notes
/// - Window_map provides byte_offset for each display position
/// - Cursor takes priority over selection highlighting
/// - All operations can fail and must be handled by caller
fn render_utf8txt_row_with_cursor(
    state: &EditorState,
    row_index: usize,
    row_content: &str,
) -> Result<String> {
    const BOLD: &str = "\x1b[1m";
    const RED: &str = "\x1b[31m";
    const YELLOW: &str = "\x1b[33m";
    const BG_WHITE: &str = "\x1b[47m";
    const BG_CYAN: &str = "\x1b[46m";
    const RESET: &str = "\x1b[0m";

    let chars: Vec<char> = row_content.chars().collect();
    let mut result = String::with_capacity(row_content.len() + 100);

    // Defensive: prevent cursor beyond line length
    let cursor_col = state.cursor.col.min(chars.len());
    let cursor_on_this_row = row_index == state.cursor.row;

    // Process each character in the row
    for col in 0..chars.len() {
        let ch = chars[col];

        // PRIORITY 1: Cursor highlighting (takes precedence)
        if cursor_on_this_row && col == cursor_col {
            result.push_str(&format!("{}{}{}{}{}", BOLD, RED, BG_WHITE, ch, RESET));
            continue;
        }

        // PRIORITY 2: Visual selection highlighting
        if state.mode == EditorMode::VisualSelectMode {
            // Get file position - propagate error if lookup fails
            let file_pos_option = state.window_map.get_file_position(row_index, col)?;

            if let Some(file_pos) = file_pos_option {
                // Check if in selection - propagate error if check fails
                let in_selection = is_in_selection(
                    file_pos.byte_offset,
                    state.file_position_of_vis_select_start,
                    state.file_position_of_vis_select_end,
                )?;

                if in_selection {
                    result.push_str(&format!("{}{}{}{}{}", BOLD, YELLOW, BG_CYAN, ch, RESET));
                    continue;
                }
            }
        }

        // PRIORITY 3: Normal character (no highlighting)
        result.push(ch);
    }

    // Handle cursor at/past end of line
    if cursor_on_this_row && cursor_col >= chars.len() {
        result.push_str(&format!("{}{}{}█{}", BOLD, RED, BG_WHITE, RESET));
    }

    Ok(result)
}

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
///     sessions/
///       {timestamp}/          <- This session's directory
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
pub fn initialize_session_directory(
    state: &mut EditorState,
    session_time_stamp: FixedSize32Timestamp,
) -> io::Result<()> {
    // Defensive: Verify state is in clean initial state
    debug_assert!(
        state.session_directory_path.is_none(),
        "Session directory should not be initialized twice"
    );

    // Step 1: Ensure base directory structure exists
    // Creates: {executable_dir}/lines_data/sessions/
    let base_sessions_path = "lines_data/sessions";

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

    // Step 2: Get timestamp for this session: synced at source, from parameter now

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

    // // Diagnostic Log success for user visibility
    // println!("Session directory created: {}", session_path.display());
    // println!("(Session files persist for crash recovery)");

    // Assertion: Verify state was updated
    debug_assert!(
        state.session_directory_path.is_some(),
        "Session directory path should be set in state"
    );

    Ok(())
}

/*
for main
/// Parses "filename:line" format and returns (filename, optional_line)
fn parse_file_with_line(input: &str) -> (String, Option<usize>) {
    // Split on last colon (to handle paths like /path/to:file.txt)
    match input.rfind(':') {
        Some(pos) => {
            let (file_part, line_part) = input.split_at(pos);
            let line_str = &line_part[1..]; // Skip the ':'

            // Try to parse as line number
            match line_str.parse::<usize>() {
                Ok(line_num) if line_num > 0 => {
                    // Valid: "file.txt:42"
                    (file_part.to_string(), Some(line_num))
                }
                _ => {
                    // Invalid line number or special flag
                    // Treat whole thing as filename (e.g., "my:file.txt")
                    (input.to_string(), None)
                }
            }
        }
        None => {
            // No colon: just a filename
            (input.to_string(), None)
        }
    }
}
*/

/// Line-Editor, Full-Mode for editing files
///
/// # Purpose
/// Main entry point for full editor functionality (not memo mode).
/// Handles file creation, opening, and launching the editor loop.
/// Insert Text uses the 'Bucket Brigade' method of processing
/// an input of unknown length in known-length modular chunks.
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
pub fn full_lines_editor(
    original_file_path: Option<PathBuf>,
    starting_line: Option<usize>,
) -> Result<()> {
    //  ///////////////////////////////////////
    //  Initialization & Bootstrap Lines Editor
    //  ///////////////////////////////////////

    // Resolve target file path (all path handling logic extracted)
    let target_path = resolve_target_file_path(original_file_path)?;
    // // Diagnostic
    // println!("\n=== Opening Lines Editor ==="); // TODO remove/commentout debug print
    // println!("File: {}", target_path.display());

    // Create file if it doesn't exist
    if !target_path.exists() {
        // // Diagnostic
        // println!("Creating new file...");

        // new file header = timestamp
        let timestamp = get_timestamp()?;
        let header = format!("# {}", timestamp);

        // Create with header
        let mut file = File::create(&target_path)?;
        writeln!(file, "{}", header)?;
        writeln!(file)?; // Empty line after header
        file.flush()?;

        // // Diagnostic
        // println!("Created new file with header");
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

    //  ////////////////////////
    //  Set Up & Build The State
    //  ////////////////////////

    let mut lines_editor_state = EditorState::new();
    lines_editor_state.original_file_path = Some(target_path.clone());

    // Initialize session directory FIRST
    initialize_session_directory(&mut lines_editor_state, session_time_stamp1)?;

    // Get session directory path (we just initialized it)
    let session_dir = lines_editor_state
        .session_directory_path
        .as_ref()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Session directory not initialized"))?;

    // Create read-copy for safety
    let read_copy_path =
        create_a_readcopy_of_file(&target_path, session_dir, session_time_stamp2.to_string())?;

    // // Diagnostic
    // println!("Read-copy: {}", read_copy_path.display());

    // Initialize window position
    lines_editor_state.line_count_at_top_of_window = 0;
    lines_editor_state.file_position_of_topline_start = 0;
    lines_editor_state.horizontal_utf8txt_line_char_offset = 0;

    // Bootstrap initial cursor position, start of file, after "l "
    lines_editor_state.cursor.row = 0;
    lines_editor_state.cursor.col = 2; // Bootstrap Bumb: start after line nunber (zero-index 2)

    // IF userizer input line: Jump to starting line if provided
    if let Some(line_num) = starting_line {
        let target_line = line_num.saturating_sub(1); // Convert 1-indexed to 0-indexed

        match seek_to_line_number(&mut File::open(&read_copy_path)?, target_line) {
            Ok(byte_pos) => {
                // Position cursor AFTER line number (same as bootstrap)
                let line_num_width = calculate_line_number_width(target_line);
                // println!("{line_num_width}{target_line}");
                lines_editor_state.cursor.col = line_num_width; // Skip over line number display
                lines_editor_state.line_count_at_top_of_window = target_line;
                lines_editor_state.file_position_of_topline_start = byte_pos;
            }
            Err(_) => {
                eprintln!("Warning: Line {} not found, starting at line 1", line_num);
                // Keep default (line 0)
            }
        }
    }
    // Initialize editor lines_editor_state
    lines_editor_state.read_copy_path = Some(read_copy_path);

    // Build initial window content
    // Get the read_copy path BEFORE the mutable borrow
    let read_copy = lines_editor_state
        .read_copy_path
        .clone()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No read copy path"))?;

    // // diagnostic
    // // Now we can mutably borrow lines_editor_state
    // let lines_processed = build_windowmap_nowrap(&mut lines_editor_state, &read_copy)?;
    // println!("Loaded {} lines", lines_processed); // TODO remove/commentout debug line

    // Now we can mutably borrow lines_editor_state
    let _ = build_windowmap_nowrap(&mut lines_editor_state, &read_copy)?;

    // Main editor loop
    let mut keep_editor_loop_running = true;

    //  ////////////////
    //  Set Up Main Loop
    //  ////////////////

    // set up pre-allocated input buffere, short for commands
    // and Bucket Brigade! for text input:
    let mut command_buffer = [0u8; WHOLE_COMMAND_BUFFER_SIZE];

    // TODO: use/reuse general 256 buffer
    let mut text_buffer = [0u8; TEXT_BUCKET_BRIGADE_CHUNKING_BUFFER_SIZE];
    let stdin = io::stdin();
    let mut stdin_handle = stdin.lock(); // Lock stdin once for entire session

    // Defensive: Limit loop iterations to prevent infinite loops
    let mut iteration_count = 0;

    // // boot strap Render TUI (convert LinesError to io::Error)
    // render_tui_utf8txt(&lines_editor_state)
    //     .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Display error: {}", e)))?;

    //  ///////////////////////////////
    //  Main Loop for Full Lines Editor
    //  ///////////////////////////////
    while keep_editor_loop_running && iteration_count < limits::MAIN_EDITOR_LOOP_COMMANDS {
        iteration_count += 1;

        //  ==================
        //  Render a Flesh TUI
        //  ==================
        if lines_editor_state.mode == EditorMode::HexMode {
            render_tui_hex(&lines_editor_state).map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("Display error: {}", e))
            })?;
        } else {
            // Render TUI (convert LinesError to io::Error)
            render_tui_utf8txt(&lines_editor_state).map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("Display error: {}", e))
            })?;
        }

        //  ====
        //  Iput
        //  ====
        if lines_editor_state.mode == EditorMode::Insert {
            //  ===========
            //  Insert Mode
            //  ===========
            keep_editor_loop_running = lines_editor_state
                .handle_utf8txt_insert_mode_input(&mut stdin_handle, &mut text_buffer)?;
        } else if lines_editor_state.mode == EditorMode::PastyMode {
            //  ==========
            //  Pasty Mode
            //  ==========
            keep_editor_loop_running =
                lines_editor_state.pasty_mode(&mut stdin_handle, &mut text_buffer)?;
        } else if lines_editor_state.mode == EditorMode::HexMode {
            //  ===============
            //  Hex Editor Mode
            //  ===============
            keep_editor_loop_running = lines_editor_state
                .handle_parse_hex_mode_input_and_commands(&mut stdin_handle, &mut command_buffer)?;
        } else if lines_editor_state.mode == EditorMode::VisualSelectMode {
            //  ==================
            //  Visual Select Mode
            //  ==================
            // Set cursor position to file_position_of_vis_select_end
            // After movement, update END position to new cursor location
            if let Ok(Some(file_pos)) = lines_editor_state
                .window_map
                .get_file_position(lines_editor_state.cursor.row, lines_editor_state.cursor.col)
            {
                lines_editor_state.file_position_of_vis_select_end = file_pos.byte_offset;
            }
            keep_editor_loop_running = lines_editor_state
                .handle_normalmode_and_visualmode_input(&mut stdin_handle, &mut command_buffer)?;
        } else {
            //  ===================================
            //  IF in Normal mode: parse as command
            //  ===================================
            keep_editor_loop_running = lines_editor_state
                .handle_normalmode_and_visualmode_input(&mut stdin_handle, &mut command_buffer)?;
        }
    }

    // Defensive: Check if we hit iteration limit
    if iteration_count >= limits::MAIN_EDITOR_LOOP_COMMANDS {
        eprintln!("Warning: Editor loop exceeded maximum iterations");
    }

    // Clean exit
    println!("\nExciting Lines Editor!");

    // Clean up read-copy file if it exists
    if let Some(read_copy) = lines_editor_state.read_copy_path {
        if read_copy.exists() {
            fs::remove_file(read_copy).ok(); // Ignore errors on cleanup
        }
    }

    Ok(())
}

// Keep This
//  /// This is a template for calling the lines module
//  /// Main entry point - routes between memo mode and full editor mode
//  ///
//  /// # Purpose
//  /// Determines which mode to use based on current directory and arguments.
//  ///
//  /// # Command Line Usage
//  /// - `lines` - Memo mode (if in home) or error (if elsewhere)
//  /// - `lines file.txt` - Full editor mode with file
//  /// - `lines /path/to/dir/` - Full editor mode, prompts for filename
//  ///
//  /// # Mode Selection Logic
//  /// 1. If CWD is home directory -> memo mode available
//  /// 2. Otherwise -> full editor mode (requires file argument)
//  ///
//  /// # Exit Codes
//  /// - 0: Success
//  /// - 1: General error
//  /// - 2: Invalid arguments
// fn main() -> io::Result<()> {
//     let args: Vec<String> = env::args().collect();

//     // Check if we're in home directory
//     let in_home = is_in_home_directory()?;

//     // // Diagnostics
//     // println!("=== Lines Text Editor ===");
//     // println!("Current directory: {}", env::current_dir()?.display());
//     // if in_home {
//     //     println!("Mode: Memo mode available (in home directory)");
//     // } else {
//     //     println!("Mode: Full editor (not in home directory)");
//     // }
//     // println!();

//     // Parse command line arguments
//     match args.len() {
//         1 => {
//             // No arguments provided
//             if in_home {
//                 // Memo mode: create today's file
//                 println!("Starting memo mode...");
//                 let original_file_path = get_default_filepath(None)?;
//                 memo_mode_mini_editor_loop(&original_file_path)
//             } else {
//                 // Full editor mode - prompt for filename in current directory
//                 println!("No file specified. Creating new file in current directory.");
//                 let filename = prompt_for_filename()?;
//                 let current_dir = env::current_dir()?;
//                 let original_file_path = current_dir.join(filename);
//                 full_lines_editor(Some(original_file_path))
//             }
//         }
//         2 => {
//             // One argument provided
//             let arg = &args[1];

//             // Check for special commands
//             match arg.as_str() {
//                 "--help" | "-h" | "help" => {
//                     print_help();
//                     Ok(())
//                 }
//                 "--version" | "-v" | "version" => {
//                     println!("lines editor v0.2.0");
//                     Ok(())
//                 }
//                 _ => {
//                     /*
//                     TODO:
//                     only open an existing file in memo-mode(append)
//                     if it has the -a flag
//                     if not existing path.. then memo mode...
//                     */
//                     // Treat as file/directory path
//                     if in_home && !arg.contains('/') && !arg.contains('\\') {
//                         // In home + simple filename = memo mode with custom name
//                         println!("Starting memo mode with custom file: {}", arg);
//                         let original_file_path = get_default_filepath(Some(arg))?;
//                         memo_mode_mini_editor_loop(&original_file_path)
//                     } else {
//                         // Full editor mode with specified path
//                         let path = PathBuf::from(arg);
//                         full_lines_editor(Some(path))
//                     }
//                 }
//             }
//         }
//         3 => {
//             // Two arguments provided
//             let flag = &args[1];
//             let filepath_arg = &args[2];

//             // Check if first arg is append flag
//             match flag.as_str() {
//                 "-a" | "--append" => {
//                     // Memo mode (append-only) with specified file path
//                     let file_path = PathBuf::from(filepath_arg);
//                     println!(
//                         "Starting memo mode (append-only) with file: {}",
//                         file_path.display()
//                     );
//                     memo_mode_mini_editor_loop(&file_path)
//                 }
//                 _ => {
//                     // Unknown flag combination
//                     eprintln!("Error: Invalid arguments");
//                     eprintln!("Usage: lines [filename | -a <filepath> | --help]");
//                     eprintln!("Examples:");
//                     eprintln!("  lines notes.txt          # Full editor mode");
//                     eprintln!("  lines -a notes.txt       # Append-only mode");
//                     eprintln!("  lines --append /tmp/log  # Append-only mode");
//                     std::process::exit(2);
//                 }
//             }
//         }
//         _ => {
//             // Multiple arguments - currently not supported
//             eprintln!("Error: It's The Too many arguments!");
//             eprintln!("Try Usage: lines [filename | -a <filepath> | --help]");
//             std::process::exit(2);
//         }
//     }
// }

/*
Build Notes:
.

All clone() heap use, or read_lie() that can be re-done in a stack based should/must be:
- Replace Clone() with Copy Trait
- Use References and Borrowing // Instead of cloning command, pass references
e.g.     original_file_path: &Path,


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


make a check and add tests and power-of-10 items
any more heap allocations that can be pre-allocated?



Todo:
check for redundant standard libraries

...
error handling reorganization...
no heap strings for production... if terse errors.

probably skip file manager...
put lines in to ff

...

1. try to completely eliminate all heap...
- heap in verbose error messages?

2. add --insert_from_file feature
- how to call... tricky, file path longer than command..
if insermode starts with '--insert-file '?
safe to check for? (possible edge cases collisions)

4. struct for multi-select?
edit in reverse order?
make a stack of operations?
a. get file positions
b. do operation at each file position starting with the last
c. limit to... 32? 16?

*/
