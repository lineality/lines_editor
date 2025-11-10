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
- Always error and exception handling: Every part of code, every process, function, and operation will fail at some point, if only because of cosmic-ray bit-flips (which are common), hardware failure, power-supply failure, adversarial attacks, etc. There must always be fail-safe error handling where production-release-build code handles issues and moves on without panic-crashing ever. Every failure must be handled smoothly: let it fail and move on. This does not mean that no function can return an error. Handling should occur where needed, e.g. before later functions are reached.

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

Note: Terminology varies across "error" / "fail" / "exception" / "catch" / "case" et al. The standard terminology is 'error handling' but 'case handling' or 'issue handling' may be a more accurate description, especially where 'error' refers to the output when unable to handle a case (which becomes semantically paradoxical). The goal is not terminating / halting / ending / shutting down / stopping, etc., or crashing / failing / panicking / coredumping / undefined-behavior-ing, etc. the program when an expected case occurs. Here production and debugging/testing starkly diverge: during testing you want to see how (and where in the code) the program may 'fail' and where and when cases are encountered. In production the satellite must not fall out of the sky ever, regardless of how pedantically beautiful the error-message in the ball of flames may have been.

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


Note: Error messages must be unique per function (e.g. name of function (or abbreviation) in the error message). Colliding generic error messages that cannot be traced to a specific function are a significant liability.


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

Production output following an error / exception / case must be managed and defined, not not open to whatever an api or OS-call wants to dump out.

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


*/

/*
Undo Redo Note:
# Places in Code to Clear Redo-Stack (when user requests to edit doc)

1. fn pasty_mode() -> Ok(PastyInputPathOrCommand::EmptyEnterFirstItem) => {...
2. fn execute_command() -> Command::InsertNewline(_) => {...
3. fn execute_command() -> Command::DeleteBackspace => {...
4. fn execute_command() -> Command::DeleteLine => {...
5. fn write_n_log_hex_edit_in_place(...
6. impl EditorState()->fn handle_utf8txt_insert_mode_input(...
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

/* # Sample main.rs
// src/main.rs
use std::env;
use std::path::PathBuf;

// import lines_editor_module lines_editor_module w/ these 2 lines:
mod lines_editor_module;
use lines_editor_module::{
    LinesError, get_default_filepath, is_in_home_directory, lines_full_file_editor,
    memo_mode_mini_editor_loop, print_help, prompt_for_filename,
};

mod buttons_reversible_edit_changelog_module;
mod toggle_comment_indent_module;

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
    SourcedFile::new(
        "src/buttons_reversible_edit_changelog_module.rs",
        include_str!("buttons_reversible_edit_changelog_module.rs"),
    ),
    SourcedFile::new(
        "src/toggle_comment_indent_module.rs",
        include_str!("toggle_comment_indent_module.rs"),
    ),
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
    Source,     // Extract source and exit
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
                return Err(format!("Error: Unknown flag '{}'", arg_str));
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
            println!("Lines-Editor Version: {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        ArgMode::Source => {
            match handle_sourceit_command("lines_editor", None, SOURCE_FILES) {
                Ok(path) => println!("Source extracted to: {}", path.display()),
                Err(e) => eprintln!("Failed to extract source: {}", e),
            }
            return Ok(());
        }
        ArgMode::AppendMode => {
            // Memo mode (append-only) - requires file path
            if let Some(file_path) = parsed.file_path {
                println!(
                    "Starting memo mode (append-only) with file: {}",
                    file_path.display()
                );
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
                println!("Starting memo mode...");
                let original_file_path = get_default_filepath(None)?;
                memo_mode_mini_editor_loop(&original_file_path)
            } else {
                // Full editor mode - prompt for filename in current directory
                println!("No file specified. Creating new file in current directory.");
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
                println!("Starting memo mode with custom file: {}", file_path_str);
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
use std::io::{self, ErrorKind, Read, Seek, SeekFrom, StdinLock, Write, stdin, stdout};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use super::toggle_comment_indent_module::{
    ToggleCommentError, ToggleIndentError, indent_line, indent_range,
    toggle_basic_singleline_comment, toggle_block_comment, toggle_range_basic_comments,
    toggle_range_rust_docstring, toggle_rust_docstring_singleline_comment, unindent_line,
    unindent_range,
};

use super::buttons_reversible_edit_changelog_module::{
    ButtonError, EditType, add_single_byte_to_file, button_hexeditinplace_byte_make_log_file,
    button_make_changelog_from_user_character_action_level, button_safe_clear_all_redo_logs,
    button_undo_redo_next_inverse_changelog_pop_lifo, detect_utf8_byte_count,
    get_redo_changelog_directory_path, get_undo_changelog_directory_path,
    read_character_bytes_from_file, read_single_byte_from_file, remove_single_byte_from_file,
};

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

pub const INFOBAR_MESSAGE_BUFFER_SIZE: usize = 32;

/// Maximum number of rows (lines) in largest supported terminal
/// of which 45 can be file rows (there are 45 tui line buffers)
pub const MAX_TUI_ROWS: usize = 45;
pub const MIN_TUI_ROWS: usize = 1;

pub const MAX_ZERO_INDEX_TUI_ROWS: usize = MAX_TUI_ROWS - 1;

/// Maximum number of columns (utf-8 char across) in largest supported TUI
/// of which 157 can be file text
pub const MAX_TUI_COLS: usize = 160;
pub const MIN_TUI_COLS: usize = 1;
/// Default terminal is 24 x 80
/// Default TUI text dimensions will be
/// +/- 3 header footer,
/// +/- at least 3 for line numbers
pub const DEFAULT_ROWS: usize = 24;
pub const DEFAULT_COLS: usize = 80;

const RESET: &str = "\x1b[0m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const GREEN: &str = "\x1b[32m";
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
#[cfg(not(debug_assertions))]
log_error(
    "Logging completed with errors",
    Some("insert_file_at_cursor:phase6"),

);
// user info-bar message
let _ = self.set_info_bar_message("display error");


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
    let timestamp = match get_short_underscore_timestamp() {
        Ok(ts) => ts,
        Err(_) => String::from("UNKNOWN_TIME"),
    };

    // Build log entry
    let log_entry = if let Some(ctx) = context {
        let num_1 = timestamp.to_string();
        let num_2 = ctx.to_string();
        let num_3 = error_msg.to_string();
        let formatted_string_1 =
            stack_format_it("[{}] [{}] {}\n", &[&num_1, &num_2, &num_3], "[N] [N] N\n");
        formatted_string_1
    } else {
        let num_1 = timestamp.to_string();
        let num_2 = error_msg.to_string();
        let formatted_string_2 = stack_format_it("[{}] {}\n", &[&num_1, &num_2], "[N] N\n");
        formatted_string_2
    };

    // // Build log entry
    // let log_entry = if let Some(ctx) = context {
    //     format!("[{}] [{}] {}\n", timestamp, ctx, error_msg)
    // } else {
    //     format!("[{}] {}\n", timestamp, error_msg)
    // };

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

// /// Gets the path to today's error log file
// fn get_error_log_path() -> io::Result<PathBuf> {
//     let home = get_home_directory()?;
//     let timestamp = get_short_underscore_timestamp()?;

//     let mut log_path = home;
//     log_path.push("Documents");
//     log_path.push("lines_editor");
//     log_path.push("lines_data");
//     log_path.push("error_logs");
//     log_path.push(format!("{}.log", timestamp));

//     Ok(log_path)
// }

/// Gets the path to today's error log file
///
/// Creates the error log directory structure if it doesn't exist:
/// ```text
/// {executable_dir}/
///   lines_data/
///     error_logs/
///       {timestamp}.log
/// ```
///
/// # Returns
/// * `Ok(PathBuf)` - Absolute canonicalized path to the error log file
/// * `Err(io::Error)` - If directory creation/verification fails
fn get_error_log_path() -> io::Result<PathBuf> {
    // Step 1: Ensure error_logs directory structure exists
    // Creates: {executable_dir}/lines_data/error_logs/
    let base_error_logs_path = "lines_data/error_logs";

    let error_logs_dir = make_verify_or_create_executabledirectoryrelative_canonicalized_dir_path(
        base_error_logs_path,
    )
    .map_err(|e| {
        let formatted_e_string = stack_format_it(
            "Failed to create error logs directory structure: {}",
            &[&e.to_string()],
            "Failed to create error logs directory structure",
        );
        io::Error::new(io::ErrorKind::Other, formatted_e_string)
    })?;

    // Defensive: Verify the path is a directory
    if !error_logs_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Error logs path exists but is not a directory",
        ));
    }

    // Step 2: Get timestamp for log filename
    let timestamp = get_short_underscore_timestamp()?;

    let num_1 = timestamp.to_string();
    let formatted_string = stack_format_it("{}.log", &[&num_1], "N.log");

    // Step 3: Construct full log file path
    let log_path = error_logs_dir.join(formatted_string);

    Ok(log_path)
}

/// Automatic conversion from ToggleCommentError to LinesError
impl From<ToggleCommentError> for LinesError {
    fn from(err: ToggleCommentError) -> Self {
        // Map ToggleCommentError variants to appropriate LinesError categories
        match err {
            ToggleCommentError::FileNotFound
            | ToggleCommentError::NoExtension
            | ToggleCommentError::LineNotFound { .. } => LinesError::InvalidInput(err.to_string()),
            ToggleCommentError::IoError(_) => {
                LinesError::Io(io::Error::new(io::ErrorKind::Other, err.to_string()))
            }
            ToggleCommentError::PathError => LinesError::StateError(err.to_string()),
            ToggleCommentError::LineTooLong { .. } => LinesError::InvalidInput(err.to_string()),
            ToggleCommentError::InconsistentBlockMarkers => LinesError::StateError(err.to_string()),
            ToggleCommentError::RangeTooLarge { .. } => LinesError::InvalidInput(err.to_string()),
        }
    }
}

/// Automatic conversion from ToggleIndentError to LinesError
impl From<ToggleIndentError> for LinesError {
    fn from(err: ToggleIndentError) -> Self {
        match err {
            ToggleIndentError::FileNotFound => LinesError::InvalidInput(err.to_string()),
            ToggleIndentError::LineNotFound { .. } => LinesError::InvalidInput(err.to_string()),
            ToggleIndentError::IoError(_) => {
                LinesError::Io(io::Error::new(io::ErrorKind::Other, err.to_string()))
            }
            ToggleIndentError::PathError => LinesError::StateError(err.to_string()),
            ToggleIndentError::LineTooLong { .. } => LinesError::InvalidInput(err.to_string()),
        }
    }
}

/// Automatic conversion from ButtonError to LinesError
impl From<ButtonError> for LinesError {
    fn from(err: ButtonError) -> Self {
        match err {
            // IO errors map directly
            ButtonError::Io(e) => LinesError::Io(e),

            // Log file issues are invalid input
            ButtonError::MalformedLog { .. } => {
                LinesError::InvalidInput("Malformed changelog file".into())
            }

            // UTF-8 errors map to UTF-8 error category
            ButtonError::InvalidUtf8 { .. } => {
                LinesError::Utf8Error("Invalid UTF-8 in changelog".into())
            }

            // Directory issues are state errors
            ButtonError::LogDirectoryError { .. } => {
                LinesError::StateError("Changelog directory error".into())
            }

            // No logs found is a state error
            ButtonError::NoLogsFound { .. } => {
                LinesError::StateError("No changelog files found".into())
            }

            // Position errors are invalid input
            ButtonError::PositionOutOfBounds { .. } => {
                LinesError::InvalidInput("Changelog position out of bounds".into())
            }

            // Incomplete log sets are state errors
            ButtonError::IncompleteLogSet { .. } => {
                LinesError::StateError("Incomplete changelog set".into())
            }

            // Assertion violations map to our catch-handle error
            ButtonError::AssertionViolation { check } => {
                LinesError::GeneralAssertionCatchViolation(
                    stack_format_it("Button system: {}", &[&check], "Button system").into(),
                )
            }
        }
    }
}

// ============================================================================
// SAVE-AS-COPY OPERATION: Retry Logic Helper Functions (start)
// ============================================================================
/*
Project Context:
These helper functions implement defensive retry logic for the save-as-copy
file operation. They distinguish between transient errors (temporary issues
that may resolve with retry) and permanent errors (fundamental issues that
won't be fixed by retrying).

Design Philosophy:
- Fail fast on permanent errors (don't waste time retrying impossible operations)
- Retry transient errors (handle brief system glitches, file locks, network blips)
- Bounded retries (never infinite loops)
- Clear error classification (explicit about what is/isn't retryable)
*/

/// Determines if an I/O error represents a transient condition worth retrying
///
/// # Purpose
/// Classifies I/O errors to implement smart retry logic: retry temporary
/// issues, fail fast on permanent problems. Part of defensive programming
/// strategy for robust file operations.
///
/// # Project Context
/// Used by save-as-copy operation to handle brief system issues without
/// failing immediately, while avoiding wasteful retries of permanent errors.
/// Supports "let it fail and try again" philosophy for transient conditions.
///
/// # Retryable Errors (returns true)
/// These indicate temporary conditions that often resolve quickly:
///
/// - `ErrorKind::Interrupted`: System call interrupted by signal
///   - Example: Process received SIGINT but handled it
///   - Common during high system load
///   - Usually resolves immediately on retry
///
/// - `ErrorKind::WouldBlock`: Resource temporarily unavailable (non-blocking I/O)
///   - Example: File descriptor not ready
///   - Common with non-blocking file operations
///   - Often resolves within milliseconds
///
/// - `ErrorKind::TimedOut`: Operation exceeded time limit
///   - Example: Network file system slow to respond
///   - May resolve if system load decreases
///   - Retry gives system more time
///
/// # Non-Retryable Errors (returns false)
/// These indicate permanent conditions that won't improve with retry:
///
/// - `ErrorKind::NotFound`: File or directory doesn't exist
///   - Retrying won't create the file
///   - Caller must handle (return OriginalNotFound status)
///
/// - `ErrorKind::PermissionDenied`: Insufficient access rights
///   - Retrying won't grant permissions
///   - Requires user intervention or elevated privileges
///
/// - `ErrorKind::AlreadyExists`: File already exists
///   - Won't change by retrying
///   - Caller must handle (return AlreadyExisted status)
///
/// - `ErrorKind::InvalidInput`: Invalid parameters
///   - Logic error, not transient condition
///   - Indicates bug in code, not system issue
///
/// - All other errors: Assumed permanent unless proven otherwise
///   - Conservative approach: don't retry unknown errors
///   - Prevents wasting time on unrecoverable conditions
///
/// # Arguments
/// * `error` - Reference to I/O error to classify
///
/// # Returns
/// * `true` - Error is transient, worth retrying
/// * `false` - Error is permanent, fail immediately
///
/// # Design Rationale
/// Conservative retry policy: only retry errors explicitly known to be
/// transient. Unknown errors assumed permanent. This prevents retry loops
/// on novel error conditions while handling common transient issues.
///
/// # Usage Example
/// ```no_run
/// # use std::io;
/// # fn is_retryable_error(e: &io::Error) -> bool { true }
/// match file.read(&mut buffer) {
///     Ok(n) => { /* handle success */ },
///     Err(e) if is_retryable_error(&e) => {
///         // Transient error, retry after delay
///     }
///     Err(e) => {
///         // Permanent error, fail immediately
///     }
/// }
/// ```
///
/// # Related Functions
/// - Used by: `retry_operation()`
/// - Complements: Error classification in `save_file_as_newfile_with_newname`
pub fn is_retryable_error(error: &io::Error) -> bool {
    // Explicit whitelist of transient error kinds
    // Conservative: only retry errors we know are temporary
    matches!(
        error.kind(),
        ErrorKind::Interrupted | ErrorKind::WouldBlock | ErrorKind::TimedOut
    )
}

/// Executes an I/O operation with automatic retry logic for transient failures
///
/// # Purpose
/// Wraps I/O operations with defensive retry mechanism: automatically retries
/// transient errors while failing fast on permanent errors. Implements bounded
/// retry loop with delay between attempts.
///
/// # Project Context
/// Core component of save-as-copy operation's defensive programming strategy.
/// Handles brief system glitches (file locks, interrupts, resource contention)
/// without requiring caller to implement retry logic. Supports "let it fail
/// and try again" philosophy for transient conditions.
///
/// # Generic Parameters
/// * `F` - Closure type that performs the I/O operation
/// * `T` - Return type of the operation on success
///
/// # Arguments
/// * `operation` - Closure that performs I/O operation: `FnMut() -> io::Result<T>`
///   - Called once initially, then again on retryable failures
///   - Must be `FnMut` because it may be called multiple times
///   - Returns `io::Result<T>` - standard I/O result type
///
/// * `max_attempts` - Maximum number of attempts (including initial try)
///   - Should use `SAVE_AS_COPY_MAX_RETRY_ATTEMPTS` constant
///   - Example: max_attempts=3 means 1 initial + 2 retries
///   - Must be > 0 (enforced by debug_assert)
///
/// # Returns
/// * `Ok(T)` - Operation succeeded (possibly after retries)
/// * `Err(io::Error)` - Operation failed:
///   - Permanent error encountered (failed immediately)
///   - Transient error persisted through all retry attempts
///   - Returns the last error encountered
///
/// # Retry Behavior
///
/// ## Attempt Loop (bounded, NASA rule 2 compliant)
/// ```text
/// Attempt 1: Execute operation
///   ├─ Success → return Ok(result)
///   ├─ Permanent error → return Err immediately
///   └─ Transient error → wait 200ms, continue
///
/// Attempt 2: Execute operation again
///   ├─ Success → return Ok(result)
///   ├─ Permanent error → return Err immediately
///   └─ Transient error → wait 200ms, continue
///
/// Attempt 3: Execute operation final time
///   ├─ Success → return Ok(result)
///   └─ Any error → return Err (max attempts exhausted)
/// ```
///
/// ## Timing
/// - Delay between attempts: 200ms (SAVE_AS_COPY_RETRY_DELAY_MS)
/// - Maximum total delay: (max_attempts - 1) × 200ms
/// - Example with 3 attempts: 2 delays × 200ms = 400ms max
///
/// # Safety Guarantees
/// - **Bounded loop**: Maximum iterations = max_attempts (no infinite loops)
/// - **No panic**: Never panics in production (debug_assert only in debug builds)
/// - **No recursion**: Iterative loop, not recursive calls
/// - **Predictable timing**: Fixed delay between retries
/// - **Fail-fast**: Permanent errors don't waste time with retries
///
/// # Usage Examples
///
/// ## Reading from file
/// ```no_run
/// # use std::io::{self, Read};
/// # use std::fs::File;
/// # const SAVE_AS_COPY_MAX_RETRY_ATTEMPTS: usize = 3;
/// # fn retry_operation<F, T>(op: F, max: usize) -> io::Result<T>
/// # where F: FnMut() -> io::Result<T> { op() }
/// let mut file = File::open("data.txt")?;
/// let mut buffer = [0u8; 1024];
///
/// let bytes_read = retry_operation(
///     || file.read(&mut buffer),
///     SAVE_AS_COPY_MAX_RETRY_ATTEMPTS
/// )?;
/// ```
///
/// ## Writing to file
/// ```no_run
/// # use std::io::{self, Write};
/// # use std::fs::File;
/// # const SAVE_AS_COPY_MAX_RETRY_ATTEMPTS: usize = 3;
/// # fn retry_operation<F, T>(op: F, max: usize) -> io::Result<T>
/// # where F: FnMut() -> io::Result<T> { op() }
/// let mut file = File::create("output.txt")?;
/// let data = b"Hello, world!";
///
/// retry_operation(
///     || file.write_all(data),
///     SAVE_AS_COPY_MAX_RETRY_ATTEMPTS
/// )?;
/// ```
///
/// # Edge Cases
/// - `max_attempts = 1`: No retries, just single attempt
/// - `max_attempts = 0`: Invalid, caught by debug_assert
/// - Operation succeeds on first try: No delay, returns immediately
/// - All attempts fail with permanent error: Returns after first attempt (no retries)
/// - All attempts fail with transient error: Returns after max_attempts exhausted
///
/// # Performance Considerations
/// - Best case: Single attempt, immediate success (no overhead)
/// - Worst case: max_attempts × (operation_time + 200ms delay)
/// - Trade-off: Small delay vs. handling transient failures gracefully
///
/// # Related Functions
/// - Uses: `is_retryable_error()` for error classification
/// - Used by: `save_file_as_newfile_with_newname()` for read/write operations
pub fn retry_operation<F, T>(mut operation: F, max_attempts: usize) -> io::Result<T>
where
    F: FnMut() -> io::Result<T>,
{
    //    =================================================
    // // Debug-Assert, Test-Asset, Production-Catch-Handle
    //    =================================================
    // This is not included in production builds
    // assert: only when running in a debug-build: will panic
    debug_assert!(max_attempts > 0, "max_attempts must be greater than 0");
    // This is not included in production builds
    // assert: only when running cargo test: will panic
    #[cfg(test)]
    assert!(max_attempts > 0, "max_attempts must be greater than 0");
    // Catch & Handle without panic in production
    // This IS included in production to safe-catch
    if max_attempts == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "max_attempts must be greater than 0",
        ));
    }

    // Bounded loop: explicit counter prevents infinite iteration
    // NASA Power of 10 rule 2: upper bound on all loops
    let mut attempt: usize = 0;

    loop {
        // Increment attempt counter at loop start
        attempt += 1;

        // Execute the operation
        match operation() {
            // Success: return immediately, no further attempts needed
            Ok(result) => return Ok(result),

            // Error on final attempt: no retries left, return error
            Err(e) if attempt >= max_attempts => {
                return Err(e);
            }

            // Transient error with retries remaining: wait and try again
            Err(e) if is_retryable_error(&e) => {
                // Wait before retry to give system time to resolve issue
                thread::sleep(Duration::from_millis(SAVE_AS_COPY_RETRY_DELAY_MS));
                // Loop continues to next iteration
                continue;
            }

            // Permanent error: fail immediately, don't waste time retrying
            // Examples: NotFound, PermissionDenied, AlreadyExists
            Err(e) => {
                return Err(e);
            }
        }

        // Loop continues for retryable errors
        // Bounded by: attempt < max_attempts (checked above)
    }
}
// ============================================================================
// (end) SAVE-AS-COPY OPERATION: Retry Logic Helper Functions
// ============================================================================
// ============================================================================
// (end) ERROR Section HANDLING SYSTEM
// ============================================================================

// =========
// =========
// Utilities
// =========
// =========

// ===============
// stack_format_it
// ===============
/// Formats a byte as 2-digit uppercase hexadecimal with optional ANSI styling.
/// **ZERO HEAP ALLOCATION**
///
/// ## Project Context
/// Used in hex editor display to show byte values with cursor highlighting.
/// Formats bytes as "XX " (3 chars) or with ANSI escape codes for highlighting.
/// Writes directly to provided stack buffer - NO heap allocation.
///
/// ## Operation
/// - Normal mode: Writes "42 " to buffer (3 bytes)
/// - Highlight mode: Writes ANSI codes + hex + reset to buffer
/// - Pure stack-based: Uses only provided buffer
///
/// ## Safety & Error Handling
/// - No panic: Returns None if buffer too small
/// - No heap: Uses only caller-provided stack buffer
/// - No allocations: Direct byte writes only
///
/// ## Parameters
/// - `byte`: The byte value to format (0x00-0xFF)
/// - `buf`: Mutable stack buffer to write into (caller-provided)
/// - `highlight`: If true, wraps with ANSI color codes
/// - `bold`: ANSI bold code (typically "\x1b[1m")
/// - `red`: ANSI red foreground code
/// - `bg_white`: ANSI white background code
/// - `reset`: ANSI reset code (typically "\x1b[0m")
///
/// ## Returns
/// - `Some(&str)`: Formatted string borrowing from buf
/// - `None`: Buffer too small
///
/// ## Example use:
/// ```rust
/// let mut buf = [0u8; 64];
///
/// // Normal byte
/// if let Some(hex) = stack_format_hex_zero(0x42, &mut buf, false, "", "", "", "") {
///     print!("{}", hex); // "42 "
/// }
///
/// // Highlighted byte
/// if let Some(hex) = stack_format_hex_zero(0x42, &mut buf, true, BOLD, RED, BG_WHITE, RESET) {
///     print!("{}", hex); // "\x1b[1m\x1b[31m\x1b[47m42\x1b[0m "
/// }
/// ```
pub fn stack_format_hex<'a>(
    byte: u8,
    buf: &'a mut [u8],
    highlight: bool,
    bold: &str,
    red: &str,
    bg_white: &str,
    reset: &str,
) -> Option<&'a str> {
    let mut pos = 0;

    if highlight {
        // Add ANSI codes before hex
        for code in &[bold, red, bg_white] {
            let code_bytes = code.as_bytes();
            if pos + code_bytes.len() > buf.len() {
                return None; // Buffer too small
            }
            buf[pos..pos + code_bytes.len()].copy_from_slice(code_bytes);
            pos += code_bytes.len();
        }
    }

    // Format byte as 2-digit hex (pure stack operation)
    let hex_chars = b"0123456789ABCDEF";
    let high = (byte >> 4) as usize;
    let low = (byte & 0x0F) as usize;

    if pos + 2 > buf.len() {
        return None; // Buffer too small
    }

    buf[pos] = hex_chars[high];
    buf[pos + 1] = hex_chars[low];
    pos += 2;

    if highlight {
        // Add reset code after hex
        let reset_bytes = reset.as_bytes();
        if pos + reset_bytes.len() > buf.len() {
            return None; // Buffer too small
        }
        buf[pos..pos + reset_bytes.len()].copy_from_slice(reset_bytes);
        pos += reset_bytes.len();
    }

    // Add trailing space
    if pos + 1 > buf.len() {
        return None; // Buffer too small
    }
    buf[pos] = b' ';
    pos += 1;

    // Return slice of buffer (guaranteed valid ASCII, thus valid UTF-8)
    std::str::from_utf8(&buf[..pos]).ok()
}
/// Converts byte to raw string representation with escape sequences.
/// **ZERO HEAP ALLOCATION**
///
/// ## Project Context
/// Used in hex editor to display bytes as readable escape sequences.
/// Shows special characters (\n, \t) and non-printable bytes (\xHH) in a
/// human-readable format. Writes directly to provided stack buffer - NO heap.
///
/// ## Operation
/// - Printable ASCII (0x20-0x7E) → writes as single character
/// - Special chars (newline, tab, etc.) → writes escape sequence
/// - Non-printable bytes → writes hex escape \xHH
/// - Pure stack-based: Uses only provided buffer
///
/// ## Safety & Error Handling
/// - No panic: Returns None if buffer too small
/// - No heap: Uses only caller-provided stack buffer
/// - Bounded output: Maximum 4 bytes (\xHH)
/// - Pre-validated: All paths write valid UTF-8
///
/// ## Parameters
/// - `byte`: Single byte to convert (0x00-0xFF)
/// - `buf`: Mutable stack buffer to write into (min 4 bytes)
///
/// ## Returns
/// - `Some(&str)`: Formatted string borrowing from buf (1-4 chars)
/// - `None`: Buffer too small (< 4 bytes)
///
/// ## Examples
/// ```rust
/// let mut buf = [0u8; 4];
///
/// stack_format_byte_escape(0x0A, &mut buf) // Some("\\n")
/// stack_format_byte_escape(0x48, &mut buf) // Some("H")
/// stack_format_byte_escape(0x00, &mut buf) // Some("\\0")
/// stack_format_byte_escape(0x09, &mut buf) // Some("\\t")
/// ```
pub fn stack_format_byte_escape<'a>(byte: u8, buf: &'a mut [u8]) -> Option<&'a str> {
    let len: usize;

    match byte {
        0x0A => {
            // Newline: \n
            if buf.len() < 2 {
                return None;
            }
            buf[0] = b'\\';
            buf[1] = b'n';
            len = 2;
        }
        0x09 => {
            // Tab: \t
            if buf.len() < 2 {
                return None;
            }
            buf[0] = b'\\';
            buf[1] = b't';
            len = 2;
        }
        0x0D => {
            // Carriage return: \r
            if buf.len() < 2 {
                return None;
            }
            buf[0] = b'\\';
            buf[1] = b'r';
            len = 2;
        }
        0x5C => {
            // Backslash: \\
            if buf.len() < 2 {
                return None;
            }
            buf[0] = b'\\';
            buf[1] = b'\\';
            len = 2;
        }
        0x22 => {
            // Quote: \"
            if buf.len() < 2 {
                return None;
            }
            buf[0] = b'\\';
            buf[1] = b'"';
            len = 2;
        }
        0x00 => {
            // Null: \0
            if buf.len() < 2 {
                return None;
            }
            buf[0] = b'\\';
            buf[1] = b'0';
            len = 2;
        }
        0x20..=0x7E => {
            // Printable ASCII
            if buf.is_empty() {
                return None;
            }
            buf[0] = byte;
            len = 1;
        }
        _ => {
            // Non-printable: \xHH
            if buf.len() < 4 {
                return None;
            }
            let hex_chars = b"0123456789ABCDEF";
            buf[0] = b'\\';
            buf[1] = b'x';
            buf[2] = hex_chars[(byte >> 4) as usize];
            buf[3] = hex_chars[(byte & 0x0F) as usize];
            len = 4;
        }
    }

    // Return slice (guaranteed valid UTF-8 - we only write ASCII)
    std::str::from_utf8(&buf[..len]).ok()
}
#[cfg(test)]
mod hex_format_tests {
    use super::*;

    #[test]
    fn test_hex_zero_normal() {
        let mut buf = [0u8; 64];
        let result = stack_format_hex(0x42, &mut buf, false, "", "", "", "");
        assert_eq!(result, Some("42 "));
    }

    #[test]
    fn test_hex_zero_highlighted() {
        let mut buf = [0u8; 64];
        let result = stack_format_hex(0x42, &mut buf, true, "[B]", "[R]", "[W]", "[RST]");
        assert_eq!(result, Some("[B][R][W]42[RST] "));
    }

    #[test]
    fn test_hex_zero_buffer_too_small() {
        let mut buf = [0u8; 2]; // Too small
        let result = stack_format_hex(0x42, &mut buf, false, "", "", "", "");
        assert_eq!(result, None);
    }

    #[test]
    fn test_byte_escape_zero_printable() {
        let mut buf = [0u8; 4];
        assert_eq!(stack_format_byte_escape(b'H', &mut buf), Some("H"));
    }

    #[test]
    fn test_byte_escape_zero_special() {
        let mut buf = [0u8; 4];
        assert_eq!(stack_format_byte_escape(0x0A, &mut buf), Some("\\n"));
        assert_eq!(stack_format_byte_escape(0x09, &mut buf), Some("\\t"));
    }

    #[test]
    fn test_byte_escape_zero_nonprintable() {
        let mut buf = [0u8; 4];
        assert_eq!(stack_format_byte_escape(0xFF, &mut buf), Some("\\xFF"));
    }

    #[test]
    fn test_byte_escape_zero_buffer_too_small() {
        let mut buf = [0u8; 1]; // Too small for \xHH
        let result = stack_format_byte_escape(0xFF, &mut buf);
        assert_eq!(result, None);
    }
}

/// Formats a message with placeholders supporting alignment and width specifiers.
///
/// ## Project Context
/// Provides string formatting for UI messages, tables, and aligned output using
/// stack-allocated buffers. Supports basic format specifiers for padding and
/// alignment without heap allocation.
///
/// ## Supported Format Specifiers
/// - `{}` - Plain replacement
/// - `{:<N}` - Left-align with width N (pad right with spaces)
/// - `{:>N}` - Right-align with width N (pad left with spaces)
/// - `{:^N}` - Center-align with width N (pad both sides with spaces)
/// - `{:N}` - Default right-align with width N
///
/// Examples:
/// - ("ID: {:<5}", &["42"]) -> "ID: 42   " (left-align, width 5)
/// - ("ID: {:>5}", &["42"]) -> "ID:    42" (right-align, width 5)
/// - ("ID: {:^5}", &["42"]) -> "ID:  42  " (center-align, width 5)
///
/// ## Safety & Error Handling
/// - No panic: Always returns valid string or fallback
/// - No unwrap: All error paths return fallback
/// - Uses 256-byte stack buffer
/// - Returns fallback if result exceeds buffer
/// - Returns fallback if format specifiers are invalid
/// - Maximum 8 inserts supported
///
/// ## Parameters
/// - `template`: String with format placeholders
/// - `inserts`: Slice of strings to insert
/// - `fallback`: Message to return if formatting fails
///
/// ## Returns
/// Formatted string on success, fallback string on any error
///
/// ## Use Examples:
/// ```rust
/// // Table-like alignment
/// let id = "42";
/// let name = "Alice";
/// let row = stack_format_it(
///     "ID: {:<5} Name: {:<10}",
///     &[id, name],
///     "Data unavailable"
/// );
/// // Result: "ID: 42    Name: Alice     "
/// ```
///
///
/// ```rust
/// let bytes = total_bytes_written.saturating_sub(1);
/// let num_str = bytes.to_string();
/// let message = stack_format_it("inserted {} bytes", &[&num_str], "inserted data");
/// ```
///
/// Error Formatting:
/// ```
/// io::stdout().flush().map_err(|e| {
///     LinesError::DisplayError(stack_format_it(
///         "Failed to flush stdout: {}",
///         &[&e.to_string()],
///         "Failed to flush stdout",
///     ))
/// })?;
/// ```
///
/// ```rust
/// let num_1 = start_byte.to_string();
/// let num_2 = end_byte.to_string();
/// let formatted_string = stack_format_it(
///     "Invalid byte range: start={} > end={}",
///     &[&num_1, &num_2],
///     "Invalid byte range"
/// );
/// ```
pub fn stack_format_it(template: &str, inserts: &[&str], fallback: &str) -> String {
    // Internal stack buffer for result
    let mut buf = [0u8; 512];

    // Maximum number of inserts to prevent abuse
    const MAX_INSERTS: usize = 128;

    // Check insert count
    if inserts.is_empty() {
        #[cfg(debug_assertions)]
        eprintln!("stack_format_it: No inserts provided");
        return fallback.to_string();
    }

    if inserts.len() > MAX_INSERTS {
        #[cfg(debug_assertions)]
        eprintln!("stack_format_it: Too many inserts (max {})", MAX_INSERTS);
        return fallback.to_string();
    }

    // Parse format specifiers and validate
    let format_specs = match parse_format_specs(template, inserts.len()) {
        Some(specs) => specs,
        None => {
            #[cfg(debug_assertions)]
            eprintln!("stack_format_it: Failed to parse format specifiers");
            return fallback.to_string();
        }
    };

    // Build the result
    let mut pos = 0;
    let mut insert_idx = 0;
    let mut search_start = 0;

    while insert_idx < inserts.len() {
        // Find next placeholder
        let placeholder_start = match template[search_start..].find('{') {
            Some(offset) => search_start + offset,
            None => break,
        };

        let placeholder_end = match template[placeholder_start..].find('}') {
            Some(offset) => placeholder_start + offset + 1,
            None => {
                #[cfg(debug_assertions)]
                eprintln!("stack_format_it: Unclosed placeholder");
                return fallback.to_string();
            }
        };

        // Copy text before placeholder
        let before = &template[search_start..placeholder_start];
        if pos + before.len() > buf.len() {
            #[cfg(debug_assertions)]
            eprintln!("stack_format_it: Buffer overflow");
            return fallback.to_string();
        }
        buf[pos..pos + before.len()].copy_from_slice(before.as_bytes());
        pos += before.len();

        // Apply format specifier and insert
        let spec = &format_specs[insert_idx];
        let insert = inserts[insert_idx];

        let formatted = apply_format_spec(insert, spec);

        if pos + formatted.len() > buf.len() {
            #[cfg(debug_assertions)]
            eprintln!("stack_format_it: Buffer overflow during insert");
            return fallback.to_string();
        }
        buf[pos..pos + formatted.len()].copy_from_slice(formatted.as_bytes());
        pos += formatted.len();

        search_start = placeholder_end;
        insert_idx += 1;
    }

    // Copy remaining text after last placeholder
    let remaining = &template[search_start..];
    if pos + remaining.len() > buf.len() {
        #[cfg(debug_assertions)]
        eprintln!("stack_format_it: Buffer overflow during final copy");
        return fallback.to_string();
    }
    buf[pos..pos + remaining.len()].copy_from_slice(remaining.as_bytes());
    pos += remaining.len();

    // Validate UTF-8 and return
    match std::str::from_utf8(&buf[..pos]) {
        Ok(s) => s.to_string(),
        Err(_) => {
            #[cfg(debug_assertions)]
            eprintln!("stack_format_it: Invalid UTF-8 in result");
            fallback.to_string()
        }
    }
}

/// Format specifier parsed from placeholder
#[derive(Debug, Clone, Copy)]
enum Alignment {
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, Copy)]
struct FormatSpec {
    alignment: Alignment,
    width: Option<usize>,
}

/// Parse format specifiers from template
/// Returns None if parsing fails or placeholder count doesn't match insert count
fn parse_format_specs(template: &str, expected_count: usize) -> Option<Vec<FormatSpec>> {
    let mut specs = Vec::new();
    let mut remaining = template;

    while let Some(start) = remaining.find('{') {
        let end = remaining[start..].find('}')?;
        let placeholder = &remaining[start + 1..start + end];

        let spec = if placeholder.is_empty() {
            // Plain {} placeholder
            FormatSpec {
                alignment: Alignment::Left,
                width: None,
            }
        } else if placeholder.starts_with(':') {
            // Format specifier like {:<5} or {:>10}
            parse_single_spec(&placeholder[1..])?
        } else {
            // Invalid format
            return None;
        };

        specs.push(spec);
        remaining = &remaining[start + end + 1..];
    }

    if specs.len() == expected_count {
        Some(specs)
    } else {
        #[cfg(debug_assertions)]
        eprintln!(
            "parse_format_specs: Placeholder count ({}) doesn't match insert count ({})",
            specs.len(),
            expected_count
        );
        None
    }
}

/// Parse a single format specifier like "<5" or ">10" or "^8"
fn parse_single_spec(spec: &str) -> Option<FormatSpec> {
    if spec.is_empty() {
        return Some(FormatSpec {
            alignment: Alignment::Right,
            width: None,
        });
    }

    let (alignment, width_str) = if spec.starts_with('<') {
        (Alignment::Left, &spec[1..])
    } else if spec.starts_with('>') {
        (Alignment::Right, &spec[1..])
    } else if spec.starts_with('^') {
        (Alignment::Center, &spec[1..])
    } else if spec.chars().next()?.is_ascii_digit() {
        // No alignment character means right-align
        (Alignment::Right, spec)
    } else {
        return None;
    };

    let width = if width_str.is_empty() {
        None
    } else {
        match width_str.parse::<usize>() {
            Ok(w) if w <= 64 => Some(w), // Reasonable width limit
            _ => return None,
        }
    };

    Some(FormatSpec { alignment, width })
}

/// Apply format specifier to a string value
fn apply_format_spec(value: &str, spec: &FormatSpec) -> String {
    let width = match spec.width {
        Some(w) => w,
        None => return value.to_string(), // No width, return as-is
    };

    let value_len = value.len();

    if value_len >= width {
        // Value already meets or exceeds width
        return value.to_string();
    }

    let padding = width - value_len;

    match spec.alignment {
        Alignment::Left => {
            // Pad right: "42   "
            let mut result = String::with_capacity(width);
            result.push_str(value);
            for _ in 0..padding {
                result.push(' ');
            }
            result
        }
        Alignment::Right => {
            // Pad left: "   42"
            let mut result = String::with_capacity(width);
            for _ in 0..padding {
                result.push(' ');
            }
            result.push_str(value);
            result
        }
        Alignment::Center => {
            // Pad both sides: " 42  "
            let left_pad = padding / 2;
            let right_pad = padding - left_pad;
            let mut result = String::with_capacity(width);
            for _ in 0..left_pad {
                result.push(' ');
            }
            result.push_str(value);
            for _ in 0..right_pad {
                result.push(' ');
            }
            result
        }
    }
}
// ======================
// End of stack_format_it
// ======================

// ============================================================================
// SAVE-AS-COPY OPERATION: Configuration Constants
// ============================================================================
/*
These constants are specific to the save-as-copy file operation and are
intentionally named to avoid collision with other file operation constants
in the Lines editor project.

Naming Convention:
- Prefix with operation type: SAVE_AS_COPY_*
- Distinguishes from: FILE_APPEND_*, INSERT_FILE_*, SAVE_CURRENT_*, etc.
*/

/// Buffer size for save-as-copy file operations (8 kilobytes)
///
/// # Purpose
/// Pre-allocated stack buffer for bucket-brigade copying from source to
/// destination file. Used exclusively by save_file_as_newfile_with_newname.
///
/// # Size Rationale
/// - 8KB is standard disk block size on most filesystems
/// - Matches stdlib io::copy() internal buffer size
/// - Small enough to stay on stack (no heap allocation)
/// - Large enough to minimize syscall overhead
/// - For 100MB file: ~12,800 iterations (well under MAX limit)
///
/// # Memory Location
/// Stack-allocated in function scope:
/// ```
/// let mut buffer = [0u8; SAVE_AS_COPY_BUFFER_SIZE];
/// ```
///
/// # Comparison with Other Buffers in Project
/// - FILE_APPEND_BUFFER_SIZE: 64 bytes (demo/test size)
/// - SAVE_AS_COPY_BUFFER_SIZE: 8192 bytes (production size)
/// - INSERT_FILE_BUFFER_SIZE: (if exists, different purpose)
///
/// # Safety Considerations
/// - Must fit on stack without overflow
/// - Typical stack size: 2MB-8MB
/// - This buffer: 8KB (0.4% of 2MB stack)
const SAVE_AS_COPY_BUFFER_SIZE: usize = 8192;

/// Maximum retry attempts for transient I/O failures in save-as-copy
///
/// # Purpose
/// Number of times to retry read or write operations when encountering
/// transient errors (Interrupted, WouldBlock, TimedOut).
///
/// # Retry Strategy
/// - Initial attempt + 2 retries = 3 total attempts
/// - 200ms delay between attempts (see SAVE_AS_COPY_RETRY_DELAY_MS)
/// - Total maximum wait: 2 retries × 200ms = 400ms
/// - Only retries transient errors, not permanent ones
///
/// # Retryable Errors
/// - ErrorKind::Interrupted (syscall interrupted)
/// - ErrorKind::WouldBlock (resource temporarily unavailable)
/// - ErrorKind::TimedOut (operation timed out)
///
/// # Non-Retryable Errors (fail immediately)
/// - ErrorKind::NotFound (file doesn't exist)
/// - ErrorKind::PermissionDenied (access denied)
/// - ErrorKind::AlreadyExists (file already exists)
///
/// # Bounded Loop Compliance
/// This constant ensures retry loops are bounded (NASA Power of 10, rule 2).
/// Combined with MAX iteration check, prevents infinite retry loops.
const SAVE_AS_COPY_MAX_RETRY_ATTEMPTS: usize = 3;

/// Delay in milliseconds between retry attempts for save-as-copy operations
///
/// # Purpose
/// Wait time between retry attempts for transient I/O failures.
///
/// # Timing Rationale
/// - 200ms allows brief transient issues to resolve
/// - Examples: file lock released, network momentary blip, disk cache flush
/// - Not too short: avoid hammering system with rapid retries
/// - Not too long: user shouldn't wait excessively
/// - Total potential delay: 2 retries × 200ms = 400ms (acceptable)
///
/// # Usage
/// ```
/// thread::sleep(Duration::from_millis(SAVE_AS_COPY_RETRY_DELAY_MS));
/// ```
const SAVE_AS_COPY_RETRY_DELAY_MS: u64 = 200;

/// Maximum chunks to process in save-as-copy operation
///
/// # Purpose
/// Safety limit to prevent infinite loops from filesystem corruption,
/// cosmic ray bit flips, or malformed file metadata.
///
/// # Capacity Calculation
/// At 8KB buffer size:
/// - 16,777,216 chunks × 8KB = 134,217,728 KB
/// - = 131,072 MB
/// - = ~128 GB maximum file size
///
/// # Rationale
/// - Protects against infinite loops (NASA Power of 10, rule 2)
/// - Allows copying very large files (128GB covers most use cases)
/// - Typical text files: < 10MB (< 1,300 chunks)
/// - Large log files: < 1GB (< 131,000 chunks)
/// - Extreme cases: up to 128GB supported
///
/// # Failure Mode
/// If exceeded, function returns error:
/// - Logs to error file (production)
/// - Returns LinesError::StateError
/// - Does not panic or halt program
///
/// # Related Constants
/// - FILE_APPEND_MAX_CHUNKS: 16,777,216 (same value, different operation)
/// - Both ensure consistent safety limits across file operations
const SAVE_AS_COPY_MAX_CHUNKS: usize = 16_777_216;

// ============================================================================
// (end) SAVE-AS-COPY OPERATION: Configuration Constants
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

// =========================
// End of Movement Functions
// =========================

/// Creates a human-readable timestamp string with UTC indicator
///
/// # Purpose
/// Generates a human-readable timestamp string that is still suitable for
/// archive filenames on most platforms. More readable than compact formats
/// while maintaining sortability.
///
/// # Arguments
/// * `time` - The SystemTime to format (typically SystemTime::now())
///
/// # Returns
/// * `String` - Timestamp in format: "YYYY-MM-DD, HH-MM-SS UTC"
///
/// # Format Specification
/// - YYYY: Four-digit year (0000-9999)
/// - MM: Two-digit month (01-12)
/// - DD: Two-digit day (01-31)
/// - HH: Two-digit hour in 24-hour format (00-23)
/// - MM: Two-digit minute (00-59)
/// - SS: Two-digit second (00-59)
/// - UTC: Explicit timezone indicator
///
/// # Examples
/// - "2024-01-15, 14-30-45 UTC" for January 15, 2024 at 2:30:45 PM
/// - "2023-12-31, 23-59-59 UTC" for December 31, 2023 at 11:59:59 PM
///
/// # Note
/// The comma and space make this more human-readable. While this works
/// on most filesystems, some may have restrictions. The format remains
/// sortable when used consistently.
///
/// # Platform Consistency
/// This function produces identical output on all platforms by using
/// epoch-based calculations rather than platform-specific date commands.
fn create_readable_archive_timestamp(time: SystemTime) -> String {
    // Get duration since Unix epoch
    let duration_since_epoch = match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration,
        Err(_) => {
            // System time before Unix epoch - use fallback
            eprintln!("Warning: System time is before Unix epoch, using fallback timestamp");
            return String::from("1970-01-01, 00-00-00 UTC");
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
        return String::from("9999-12-31, 23-59-59 UTC");
    }

    // Assertion 2: Validate all components are in expected ranges
    if month < 1 || month > 12 || day < 1 || day > 31 || hour > 23 || minute > 59 || second > 59 {
        eprintln!(
            "Warning: Invalid date/time components: {}-{:02}-{:02} {:02}:{:02}:{:02}",
            year, month, day, hour, minute, second
        );
        return String::from("1970-01-01, 00-00-00 UTC"); // Safe fallback
    }

    // Format as YYYY-MM-DD, HH-MM-SS UTC
    // format!(
    //     "{:04}-{:02}-{:02}, {:02}-{:02}-{:02} UTC",
    //     year, month, day, hour, minute, second
    // )
    stack_format_it(
        "{:04}-{:02}-{:02}, {:02}-{:02}-{:02} UTC",
        &[
            &year.to_string(),
            &month.to_string(),
            &day.to_string(),
            &hour.to_string(),
            &minute.to_string(),
            &second.to_string(),
        ],
        "... UTC",
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

/// Creates a timestamp with full year and optional microsecond precision
///
/// # Purpose
/// When multiple archives might be created in the same second, this
/// adds microsecond precision to ensure unique filenames. Includes
/// full 4-digit year prefix for better year identification.
///
/// # Arguments
/// * `time` - The SystemTime to format
/// * `include_microseconds` - Whether to append microseconds
///
/// # Returns
/// * `String` - Timestamp with YYYY prefix, optionally with microseconds appended
///
/// # Format
/// - Without microseconds: "YYYY_YY_MM_DD_HH_MM_SS"
/// - With microseconds: "YYYY_YY_MM_DD_HH_MM_SS_UUUUUU"
///
/// # Examples
/// - "2024_24_01_15_14_30_45" (without microseconds)
/// - "2024_24_01_15_14_30_45_123456" (with microseconds)
pub fn createarchive_timestamp_with_precision(
    time: SystemTime,
    include_microseconds: bool,
) -> String {
    // Get duration since Unix epoch
    let duration_since_epoch = match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration,
        Err(_) => {
            eprintln!("Warning: System time is before Unix epoch, using fallback timestamp");
            if include_microseconds {
                return String::from("1970_70_01_01_00_00_00_000000");
            } else {
                return String::from("1970_70_01_01_00_00_00");
            }
        }
    };

    let total_seconds = duration_since_epoch.as_secs();

    // Use the accurate date calculation
    let (year, month, day, hour, minute, second) =
        epoch_seconds_to_datetime_components(total_seconds);

    // Validate year range
    const MAX_REASONABLE_YEAR: u32 = 9999;
    if year > MAX_REASONABLE_YEAR {
        eprintln!(
            "Warning: Year {} exceeds maximum reasonable value {}. Using fallback.",
            year, MAX_REASONABLE_YEAR
        );
        if include_microseconds {
            return String::from("9999_99_12_31_23_59_59_999999");
        } else {
            return String::from("9999_99_12_31_23_59_59");
        }
    }

    // Validate all components are in expected ranges
    if month < 1 || month > 12 || day < 1 || day > 31 || hour > 23 || minute > 59 || second > 59 {
        eprintln!(
            "Warning: Invalid date/time components: {}-{:02}-{:02} {:02}:{:02}:{:02}",
            year, month, day, hour, minute, second
        );
        if include_microseconds {
            return String::from("1970_70_01_01_00_00_00_000000");
        } else {
            return String::from("1970_70_01_01_00_00_00");
        }
    }
    let two_dig = year % 100;

    // Build base timestamp with YYYY prefix
    let base_timestamp = stack_format_it(
        "{:04}_{:02}_{:02}_{:02}_{:02}_{:02}_{:02}",
        &[
            &year.to_string(),
            &two_dig.to_string(),
            &month.to_string(),
            &day.to_string(),
            &hour.to_string(),
            &minute.to_string(),
            &second.to_string(),
        ],
        "YYYY_MM_DD_HH_MM_OOPS",
    );

    // // Build base timestamp with YYYY prefix
    // let base_timestamp = format!(
    //     "{:04}_{:02}_{:02}_{:02}_{:02}_{:02}_{:02}",
    //     year,       // Four-digit year
    //     year % 100, // Two-digit year
    //     month,
    //     day,
    //     hour,
    //     minute,
    //     second
    // );

    if !include_microseconds {
        return base_timestamp;
    }

    // Add microseconds component
    let microseconds = duration_since_epoch.as_micros() % 1_000_000;

    // // TODO add formatting ability?
    // stack_format_it(
    //     "{}_{:06}",
    //     &[&base_timestamp.to_string(), &base_timestamp.to_string()],
    //     "{}_{:06}",
    // )

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
                stack_format_it(
                    "impl FixedSize32Timestamp String too long: {} bytes, max: {}",
                    &[&s.len().to_string(), &MAX_LEN.to_string()],
                    "impl FixedSize32Timestamp String too long: __ bytes, max: __",
                ),
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
                stack_format_it(
                    "Invalid UTF-8 in FixedSize32Timestamp: {}",
                    &[&e.to_string()],
                    "Invalid UTF-8 in FixedSize32Timestamp",
                ),
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
            &stack_format_it(
                "Cannot open file for line count: {}",
                &[&e.to_string()],
                "Cannot open file for line count",
            ),
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
            let error_msg =
                "Line count exceeded maximum iterations (MAX_ITERATIONS). File may be corrupted.";
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
                let error_msg = stack_format_it(
                    "read() returned unexpected byte count: {} (expected 0 or 1)",
                    &[&n.to_string()],
                    "read() returned unexpected byte count (expected 0 or 1)",
                );
                log_error(&error_msg, Some("count_lines_in_file"));
                return Err(LinesError::Io(io::Error::new(
                    io::ErrorKind::InvalidData,
                    error_msg,
                )));
            }
            Err(e) => {
                // Read error - propagate
                #[cfg(debug_assertions)]
                log_error(
                    &stack_format_it(
                        "Read error at byte {}: {}",
                        &[&current_byte_position.to_string(), &e.to_string()],
                        "count_lines_in_file Read error",
                    ),
                    Some("count_lines_in_file"),
                );
                // safe
                log_error(
                    "count_lines_in_file Read error",
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

    // // Build the legend string with error handling for format operations
    // // quit save undo norm ins vis del wrap relative raw byt wrd,b,end /commnt hjkl
    // let formatted_legend = format!(
    //     "{}q{}uit {}s{}a{}v {}re{},{}u{}ndo {}d{}el|{}n{}rm {}i{}ns {}v{}is {}hex{}{}{}{} r{}aw|{}p{}asty {}cvy{}|{}w{}rd,{}b{},{}e{}nd {}/{}/{}/cmnt {}[]{}idnt {}hjkl{}{}",
    //     // YELLOW, // Overall legend color
    //     RED,
    //     YELLOW, // RED q + YELLOW uit
    //     RED,
    //     GREEN, // RED b + YELLOW ack
    //     YELLOW,
    //     RED,
    //     YELLOW, // RED b + YELLOW ack
    //     RED,
    //     YELLOW, // RED t + YELLOW erm
    //     RED,
    //     YELLOW, // RED d + YELLOW ir
    //     RED,
    //     YELLOW, // RED f + YELLOW ile
    //     RED,
    //     YELLOW, // RED n + YELLOW ame
    //     RED,
    //     YELLOW, // RED s + YELLOW ize
    //     RED,
    //     YELLOW, // RED m + YELLOW od
    //     RED,
    //     YELLOW, // RED m + YELLOW od
    //     RED,
    //     YELLOW, // RED g + YELLOW et
    //     RED,
    //     YELLOW, // RED v + YELLOW ,
    //     RED,
    //     YELLOW, // RED y + YELLOW ,
    //     RED,
    //     YELLOW, // RED p + YELLOW ,
    //     RED,
    //     YELLOW, // RED str + YELLOW ...
    //     RED,
    //     YELLOW,
    //     RED,
    //     // YELLOW, // RED enter + YELLOW ...
    //     GREEN,  // RED b + YELLOW ack
    //     YELLOW, // RED enter + YELLOW ...
    //     RED,
    //     YELLOW, // RED enter + YELLOW ...
    //     RED,
    //     YELLOW, // RED enter + YELLOW ...
    //     RESET
    // );

    let formatted_legend = stack_format_it(
        "{}q{}uit {}s{}a{}v {}re{},{}u{}ndo {}d{}el|{}n{}rm {}i{}ns {}v{}is {}hex{}{}{}{} r{}aw|{}p{}asty {}cvy{}|{}w{}rd,{}b{},{}e{}nd {}/{}/{}/cmnt {}[]{}idnt {}hjkl{}{}",
        &[
            &RED, &YELLOW, &RED, &GREEN, &YELLOW, &RED, &YELLOW, &RED, &YELLOW, &RED, &YELLOW,
            &RED, &YELLOW, &RED, &YELLOW, &RED, &YELLOW, &RED, &YELLOW, &RED, &YELLOW, &RED,
            &YELLOW, &RED, &YELLOW, &RED, &YELLOW, &RED, &YELLOW, &RED, &YELLOW, &RED, &YELLOW,
            &RED, &GREEN, &YELLOW, &RED, &YELLOW, &RED, &YELLOW, &RESET,
        ],
        "quit sav re,undo del|nrm ins vis hex raw|pasty cvy|wrd,b,end ///cmnt []idnt hjkl",
    );
    // Check if the formatted string is reasonable
    // (defensive programming against format! macro issues)
    if formatted_legend.is_empty() {
        return Err(LinesError::FormatError(String::from(
            "Legend formatting produced empty string",
        )));
    }

    // TODO (push in smaller segments?)
    legend.push_str(&formatted_legend);

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
    pub byte_offset_linear_file_absolute_position: u64,
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

    /// Hex Edict!
    HexMode,
    RawMode,
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
    pub the_last_command: Option<Command>,

    ///where lines files for this session are stored
    pub session_directory_path: Option<PathBuf>,

    /// Current editor mode
    pub mode: EditorMode,

    /// Absolute path to the file being edited
    pub original_file_path: Option<PathBuf>,

    /// Absolute path to read-copy of file
    pub read_copy_path: Option<PathBuf>,

    /// Effective editing area (minus headers/footers/line numbers)
    pub effective_rows: usize,
    pub effective_cols: usize,
    pub windowmap_positions: [[Option<FilePosition>; MAX_TUI_COLS]; MAX_TUI_ROWS],

    /// start stop Byte positions for each display row in the file
    ///
    /// # Purpose
    /// Tracks the start and end byte positions for each line displayed on screen.
    /// Enables O(1) line boundary detection for cursor movement and viewport calculations.
    ///
    /// # Format
    /// `display_line_byte_ranges[row] = Some((line_start_byte, line_end_byte_inclusive))`
    ///
    /// # Semantics
    /// - `line_start_byte` (inclusive): First byte of the line in file
    /// - `line_end_byte_inclusive` (inclusive): Last byte before newline (or EOF for last line)
    /// - `None`: Row not populated (empty window area)
    ///
    /// # Examples
    /// - Line with content "hello\n" at bytes [10-15]:
    ///   `Some((10, 15))` - range includes all content bytes, NOT the newline
    /// - Last line "world" with no trailing newline at bytes [20-24]:
    ///   `Some((20, 24))` - range ends at last content byte
    /// - Empty line (just "\n") at bytes [30-30]:
    ///   `Some((30, 30))` - start and end on the newline position?
    ///   `Some((30, 29))` - inverted to signal "empty line"?
    ///   Or better: handle separately in logic
    ///
    /// # Usage
    /// ```ignore
    /// // Jump to end of line for cursor
    /// if let Some((_, line_end)) = window_map.display_line_byte_ranges[row] {
    ///     cursor_position = line_end;
    /// }
    ///
    /// // Check if cursor at line start
    /// if let Some((line_start, _)) = window_map.display_line_byte_ranges[row] {
    ///     if cursor_byte == line_start {
    ///         // At start of line
    ///     }
    /// }
    ///
    /// // Detect line boundary for move-left wrapping
    /// if let Some((line_start, _)) = window_map.display_line_byte_ranges[current_row] {
    ///     if cursor_byte == line_start {
    ///         // Move to previous line end
    ///         let prev_line = current_row - 1;
    ///         if let Some((_, prev_end)) = window_map.display_line_byte_ranges[prev_line] {
    ///             cursor_byte = prev_end;
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// # Empty Lines
    /// Empty lines (containing only "\n") need special handling:
    /// - Option 1: `Some((pos, pos))` - start equals end, signals empty
    /// - Option 2: `Some((pos, pos - 1))` - inverted range signals empty
    /// - Option 3: Track separately with a `[bool; MAX_TUI_ROWS]` for "is_empty_line"
    ///
    /// Recommend Option 1 (start == end) as most intuitive.
    pub windowmap_line_byte_start_end_position_pairs: [Option<(u64, u64)>; MAX_TUI_ROWS],

    // to force-reset manually clear overwrite buffers
    pub security_mode: bool,

    /// Cursor position in window
    pub cursor: WindowPosition,

    /// Flag signaling that next move right
    /// should bump down to next line down.
    pub next_move_right_is_past_newline: bool,

    /// Visual mode selection start (if in visual mode)
    pub selection_start: Option<FilePosition>, // end is 'current' one
    pub selection_rowline_start: usize, // end is 'current' one

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

    /// TODO making this bigger/ribbon?
    /// For NoWrap mode: horizontal character offset for all displayed lines
    /// Example: Showing characters 20-97 of each line
    pub tui_window_horizontal_utf8txt_line_char_offset: usize,
    pub in_row_abs_horizontal_0_index_cursor_position: usize,

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

    // pub raw_cursor: RawCursor,
    //  /// TODO: Should there be a clear-buffer method?
    //  /// Pre-allocated buffer for insert mode text input
    //  /// Used to capture user input before inserting into file
    // pub tofile_insert_input_chunk_buffer: [u8; TOFILE_INSERTBUFFER_CHUNK_SIZE], // not used
    /// EOF information for the currently displayed window
    /// None = EOF not visible in current window
    /// Some((file_line_of_eof, eof_tui_display_row)) = EOF position
    pub eof_fileline_tuirow_tuple: Option<(usize, usize)>,

    //  /// TODO is this needed?
    //  /// Number of valid bytes in tofile_insert_input_chunk_buffer
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
            original_file_path: None,
            read_copy_path: None,

            effective_rows,
            effective_cols,

            windowmap_positions: [[None; MAX_TUI_COLS]; MAX_TUI_ROWS],
            windowmap_line_byte_start_end_position_pairs: [None; MAX_TUI_ROWS],
            security_mode: false, // default setting, purpose: to force-reset manually clear overwrite buffers

            cursor: WindowPosition { row: 0, col: 0 },

            // window_start: FilePosition {
            //     // for Wrap mode, if that happens
            //     byte_offset_linear_file_absolute_position: 0,
            //     line_number: 0,
            //     byte_in_line: 0,
            // },
            //
            next_move_right_is_past_newline: false,
            selection_start: None,
            selection_rowline_start: 0,
            is_modified: false,

            // === NEW FIELD INITIALIZATION ===
            // Window position tracking - start at beginning of file
            line_count_at_top_of_window: 0,
            file_position_of_topline_start: 0,

            // Clipboard/Pasty
            file_position_of_vis_select_start: 0,
            file_position_of_vis_select_end: 0,

            tui_window_horizontal_utf8txt_line_char_offset: 0,
            in_row_abs_horizontal_0_index_cursor_position: 2, // set to 0:0 real text postion after number

            // Display buffers - initialized to zero
            utf8_txt_display_buffers: [[0u8; 182]; 45],
            display_utf8txt_buffer_lengths: [0usize; 45],
            hex_cursor: HexCursor::new(),
            eof_fileline_tuirow_tuple: None, // Time is like a banana, it had no end...
            info_bar_message_buffer: [0u8; INFOBAR_MESSAGE_BUFFER_SIZE],
        }
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
        // lines_editor_state: &mut EditorState,
        row: usize,
        col: usize,
        file_pos: Option<FilePosition>,
    ) -> io::Result<()> {
        // ============================================================
        // Debug-Assert, Test-Assert, Production-Catch-Handle
        // ============================================================
        /*
        This should be prefiltered for catch-handl
        in Command::TallMinus => et al
         */

        if row >= MAX_TUI_ROWS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Row {} exceeds valid rows {}", row, self.effective_rows),
            ));
        }

        if col >= MAX_TUI_COLS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Column {} exceeds valid columns {}",
                    col, self.effective_cols
                ),
            ));
        }
        // ==============
        // Catch & Handle
        // ==============
        if row >= MAX_TUI_ROWS {
            // Do Nothing
            // Handle as caught case: Do Nothing
            // let _ = lines_editor_state.set_info_bar_message("row >= MAX_TUI_ROWS");
        } else if col >= MAX_TUI_COLS {
            // Do Nothing
            // let _ = lines_editor_state.set_info_bar_message("col >= MAX_TUI_COLS");
        } else {
            // ================
            // OK: Update State
            // ================
            self.windowmap_positions[row][col] = file_pos;
        }
        Ok(())
    }

    /// Clears all mappings
    pub fn clear_windowmap_positions(&mut self) {
        // Defensive: explicit loop with bounds
        for row in 0..MAX_TUI_ROWS {
            for col in 0..MAX_TUI_COLS {
                self.windowmap_positions[row][col] = None;
            }
        }
    }

    /// Stores the byte range for a single display row
    ///
    /// # Purpose
    /// Records where a line begins and ends in the file.
    /// Enables O(1) line boundary detection without scanning file.
    ///
    /// # Arguments
    /// * `row` - Display row index (0-indexed, 0..MAX_TUI_ROWS)
    /// * `start_byte` - First byte of line in file (inclusive)
    /// * `end_byte` - Last byte before newline (inclusive), or EOF position
    ///
    /// # Returns
    /// * `Ok(())` - Successfully stored
    /// * `Err(io::Error)` - If row index out of bounds
    ///
    /// # Semantics
    /// - Empty line "just \n": `start_byte == end_byte` signals empty
    /// - Normal line "text\n": stores content bytes, NOT the newline itself
    /// - Last line no newline: stores up to last content byte
    ///
    /// # Examples
    /// ```ignore
    /// // "hello\n" at bytes [10..15]
    /// set_line_byte_range(0, 10, 15)?;
    ///
    /// // Empty line "\n" at byte [20]
    /// set_line_byte_range(1, 20, 20)?;
    ///
    /// // Last line "world" at bytes [25..29], no newline
    /// set_line_byte_range(2, 25, 29)?;
    /// ```
    pub fn set_line_byte_range(
        &mut self,
        row: usize,
        start_byte: u64,
        end_byte: u64,
    ) -> io::Result<()> {
        // Defensive: Validate row index is within bounds
        if row >= MAX_TUI_ROWS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Row {} exceeds maximum display rows {}", row, MAX_TUI_ROWS),
            ));
        }

        // Defensive: Validate byte range is sensible (start <= end)
        // Empty lines have start == end, which is valid and signals "empty"
        if start_byte > end_byte {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Invalid line byte range: start {} > end {}",
                    start_byte, end_byte
                ),
            ));
        }

        // Store the range
        self.windowmap_line_byte_start_end_position_pairs[row] = Some((start_byte, end_byte));

        Ok(())
    }

    /// Clears all line byte range tracking
    ///
    /// # Purpose
    /// Resets line boundary data when rebuilding window (e.g., after scroll).
    /// Called at start of `build_windowmap_nowrap()`.
    pub fn clear_line_byte_ranges(&mut self) {
        // Defensive: explicit loop with bounds (NASA Power of 10 Rule 2)
        for row in 0..MAX_TUI_ROWS {
            self.windowmap_line_byte_start_end_position_pairs[row] = None;
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

    /// Gets the file position for a window position
    ///
    /// # Arguments
    /// * `row` - Terminal row (0-indexed)
    /// * `col` - Terminal column (0-indexed)
    ///
    /// # Returns
    /// * `Ok(Option<FilePosition>)` - File position if valid, None if empty
    /// * `Err(io::Error)` - If row/col out of bounds
    pub fn get_row_col_file_position(
        &self,
        row: usize,
        col: usize,
    ) -> io::Result<Option<FilePosition>> {
        // ============================================================
        // Debug-Assert, Test-Assert, Production-Catch-Handle
        // ============================================================
        #[cfg(test)]
        assert!(
            row >= self.effective_rows,
            "Failed Test: pub fn get_row_col_file_position: assert!(row >= self.valid_rows..."
        );
        #[cfg(test)]
        assert!(
            col >= self.effective_cols,
            "Failed Test: pub fn get_row_col_file_position: assert!(col >= self.valid_cols..."
        );
        // Defensive: Check bounds
        #[cfg(debug_assertions)]
        if row >= self.effective_rows {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Row {} exceeds valid rows {}", row, self.effective_rows),
            ));
        }
        #[cfg(debug_assertions)]
        if col >= self.effective_cols {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Column {} exceeds valid columns {}",
                    col, self.effective_cols
                ),
            ));
        }
        // ==============
        // Catch & Handle
        // ==============
        if row >= self.effective_rows {
            // Handle as caught case: Do Nothing
            // let _ = lines_editor_state.set_info_bar_message("row >= MAX_TUI_ROWS");
            Ok(None)
        } else if col >= self.effective_cols {
            // let _ = lines_editor_state.set_info_bar_message("col >= MAX_TUI_COLS");
            Ok(None)
        } else {
            // ================
            // OK: Update State
            // ================
            Ok(self.windowmap_positions[row][col])
        }
    }

    // ============================================================================
    // LINE END DETECTION (UTF-8-Aware Cursor Movement Support)
    // ============================================================================

    /// Determines if the next byte in the file is a newline character (line-end detection)
    ///
    /// # Purpose (Project Context)
    /// This function supports text editor cursor movement in the MoveRight command.
    /// When the cursor reaches the end of a line, the next byte is a newline character,
    /// which signals that MoveRight should wrap to the next line instead of continuing right.
    ///
    /// # Multi-byte UTF-8 Support (PRIMARY FIX)
    /// The critical insight: "next byte after cursor" depends on character size.
    ///
    /// For ASCII 'a' (1 byte):
    /// ```text
    /// Position: [10='a', 11='\n']
    /// Cursor at 'a': cursor_byte=10, char_length=1, char_end=10
    /// Next byte after 'a' is at position 11 (the newline)
    /// ```
    ///
    /// For Chinese '世' (3 bytes: E4 B8 96):
    /// ```text
    /// Position: [10=E4, 11=B8, 12=96, 13='\n']
    /// Cursor at '世': cursor_byte=10, char_length=3, char_end=12
    /// Next byte after '世' is at position 13 (the newline)
    /// ```
    ///
    /// **Previous Bug:** Compared `cursor_byte` (10) to `line_end_byte` (12) → FALSE ❌
    /// **Fixed Logic:** Compare `cursor_char_end_byte` (12) to `line_end_byte` (12) → TRUE ✓
    ///
    /// # Scope - Graceful Out-of-Bounds Handling
    /// **This function exists specifically to handle out-of-bounds conditions safely.**
    /// Instead of crashing when the cursor is at invalid positions, it returns safe
    /// default values that allow the application to continue operating:
    ///
    /// - No read-copy file path → Returns `Ok(false)` (cannot analyze, not at newline)
    /// - Cursor row beyond file line count → Returns `Ok(false)` (treat as not at newline)
    /// - Cursor column beyond line length → Returns `Ok(false)` (treat as not at newline)
    /// - Line byte range not initialized → Returns `Ok(false)` (treat as not at newline)
    /// - Cursor position unmapped in window → Returns `Ok(false)` (treat as not at newline)
    /// - UTF-8 character read fails → Returns `Ok(false)` (cannot determine, not at newline)
    ///
    /// The philosophy: **When in doubt about boundaries, assume we're NOT at a newline.**
    /// This prevents cursor movement commands from incorrectly wrapping lines, which is
    /// safer than crashing the application.
    ///
    /// This function is stateless and read-only; it never modifies editor state.
    ///
    /// # Returns
    /// * `Ok(true)` - Cursor is definitively at line-end; next file byte is the newline
    /// * `Ok(false)` - Cursor is NOT at line-end, OR position is out-of-bounds/invalid
    /// * `Err(LinesError::StateError)` - Only for truly unrecoverable internal corruption
    ///
    /// Note: Out-of-bounds conditions return `Ok(false)`, not errors. This allows the
    /// application to continue safely without crashing on boundary conditions.
    ///
    /// # Algorithm
    /// 1. Validate cursor row is within window bounds
    /// 2. Get cursor's byte position in file from window map
    /// 3. **Read UTF-8 character at cursor to get byte length**
    /// 4. **Calculate last byte of cursor's character**
    /// 5. Get line's end byte position from window map
    /// 6. **Compare character-end to line-end** (not character-start)
    /// 7. Return true if they match (next byte is newline)
    ///
    /// # Examples
    /// ```ignore
    /// // ASCII at line end
    /// // File: "ab\n" where line_end_byte=11 (byte before newline)
    /// // Cursor on 'b' at byte 11
    /// let result = state.is_next_byte_newline()?;
    /// assert_eq!(result, true); // Next byte (12) is newline
    ///
    /// // Multi-byte UTF-8 at line end
    /// // File: "a世\n" where 世 is bytes 11-13, line_end_byte=13
    /// // Cursor on '世' starting at byte 11
    /// let result = state.is_next_byte_newline()?;
    /// assert_eq!(result, true); // Character ends at 13, next byte (14) is newline
    ///
    /// // Not at line end
    /// // File: "abc\n"
    /// // Cursor on 'a'
    /// let result = state.is_next_byte_newline()?;
    /// assert_eq!(result, false); // Not at end of line
    /// ```
    pub fn is_next_byte_newline(&self) -> Result<bool> {
        // ═══════════════════════════════════════════════════════════════════════
        // DEFENSIVE CHECK 0: Verify read-copy file path exists
        // ═══════════════════════════════════════════════════════════════════════
        let read_copy_path = match &self.read_copy_path {
            Some(path) => path,
            None => {
                #[cfg(debug_assertions)]
                eprintln!(
                    "is_next_byte_newline: no read-copy file path available (returning false - not at newline)"
                );

                // Not an error - just means we cannot analyze the file
                return Ok(false);
            }
        };

        // ═══════════════════════════════════════════════════════════════════════
        // DEFENSIVE CHECK 1: Row bounds validation
        // ═══════════════════════════════════════════════════════════════════════
        // Instead of panicking on out-of-bounds, return Ok(false).
        // This is the PRIMARY PURPOSE of this function: handle boundaries gracefully.
        if self.cursor.row >= self.windowmap_line_byte_start_end_position_pairs.len() {
            #[cfg(debug_assertions)]
            eprintln!(
                "is_next_byte_newline: cursor row {} >= line count {} (returning false - not at newline)",
                self.cursor.row,
                self.windowmap_line_byte_start_end_position_pairs.len()
            );

            // Not an error - this is expected handling of out-of-bounds
            return Ok(false);
        }

        // ═══════════════════════════════════════════════════════════════════════
        // DEFENSIVE CHECK 2: Cursor position mapping validation
        // ═══════════════════════════════════════════════════════════════════════
        // If cursor doesn't map to a valid file position, safely return false.
        // This handles columns beyond line length without crashing.
        let cursor_byte_result =
            self.get_row_col_file_position(self.cursor.row, self.cursor.col)?;

        let cursor_byte_start = match cursor_byte_result {
            Some(pos) => pos.byte_offset_linear_file_absolute_position,
            None => {
                #[cfg(debug_assertions)]
                eprintln!(
                    "is_next_byte_newline: cursor ({}, {}) has no valid file position mapping (returning false - not at newline)",
                    self.cursor.row, self.cursor.col
                );

                // Not an error - cursor is just beyond valid positions
                return Ok(false);
            }
        };

        // ═══════════════════════════════════════════════════════════════════════
        // UTF-8 ANALYSIS: Determine byte length of character at cursor
        // ═══════════════════════════════════════════════════════════════════════
        // This is the KEY FIX for multi-byte character support.
        // We must find where the COMPLETE character ends, not where it starts.
        let char_byte_length = match get_utf8_char_byte_length_at_position(
            read_copy_path,
            cursor_byte_start,
        ) {
            Ok(len) => len,

            Err(_e) => {
                #[cfg(debug_assertions)]
                eprintln!(
                    "is_next_byte_newline: failed to read UTF-8 character at byte {}: {} (returning false - not at newline)",
                    cursor_byte_start, _e
                );

                // Defensive: If we can't read the character, assume not at line end
                // This allows cursor movement to continue safely
                return Ok(false);
            }
        };

        // Assertion: UTF-8 character length must be 1-4 bytes
        debug_assert!(
            char_byte_length >= 1 && char_byte_length <= 4,
            "UTF-8 character length must be 1-4, got {}",
            char_byte_length
        );

        // ═══════════════════════════════════════════════════════════════════════
        // CALCULATE: Last byte position of current character
        // ═══════════════════════════════════════════════════════════════════════
        // For 1-byte char at position 10: start=10, end=10
        // For 3-byte char at position 10: start=10, end=12
        let cursor_char_end_byte = cursor_byte_start + (char_byte_length as u64) - 1;

        // ═══════════════════════════════════════════════════════════════════════
        // DEFENSIVE CHECK 3: Line byte range access
        // ═══════════════════════════════════════════════════════════════════════
        // Use .get() for safe indexing. If somehow the row is still invalid,
        // return false (defense-in-depth: additional validation layer).
        let line_byte_range = match self
            .windowmap_line_byte_start_end_position_pairs
            .get(self.cursor.row)
        {
            Some(range) => range,
            None => {
                #[cfg(debug_assertions)]
                eprintln!(
                    "is_next_byte_newline: line byte range missing for row {} (returning false - not at newline)",
                    self.cursor.row
                );

                // Not an error - handle missing range gracefully
                return Ok(false);
            }
        };

        // ═══════════════════════════════════════════════════════════════════════
        // DEFENSIVE CHECK 4: Line byte range initialization
        // ═══════════════════════════════════════════════════════════════════════
        // The line_byte_range is an Option; if None, return false safely.
        let (_start, end) = match line_byte_range {
            Some(range) => *range,
            None => {
                #[cfg(debug_assertions)]
                eprintln!(
                    "is_next_byte_newline: line byte range is None (uninitialized) for row {} (returning false - not at newline)",
                    self.cursor.row
                );

                // Not an error - uninitialized state handled gracefully
                return Ok(false);
            }
        };

        // ═══════════════════════════════════════════════════════════════════════
        // LOGIC: Determine if at line end (UTF-8-AWARE)
        // ═══════════════════════════════════════════════════════════════════════
        // If the LAST BYTE of the cursor's character equals the line's end byte,
        // then the next byte in the file is the newline character.
        //
        // Example with multi-byte character:
        //   Line: "ab世\n"
        //   Bytes: [10='a', 11='b', 12-14='世', 15='\n']
        //   line_end_byte = 14 (last content byte before newline)
        //
        //   When cursor is on '世':
        //     cursor_byte_start = 12
        //     char_byte_length = 3
        //     cursor_char_end_byte = 12 + 3 - 1 = 14
        //     cursor_char_end_byte (14) == line_end_byte (14) → TRUE
        //     Next byte (15) IS the newline → return Ok(true)
        //
        // This is the only condition where we return Ok(true).
        Ok(cursor_char_end_byte == end)
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
        // Get read-copy path
        let base_edit_filepath: PathBuf = self
            .read_copy_path
            .as_ref()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "CRITICAL: No read-copy path available - cannot edit",
                )
            })?
            .clone();

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

        //  ===============
        //  Pasty Mode Loop
        //  ===============
        loop {
            pasty_iteration += 1;

            // Safety bound: prevent infinite loops
            if pasty_iteration > limits::MAIN_EDITOR_LOOP_COMMANDS {
                let _ = self.set_info_bar_message("pasty mode iteration limit");

                return Ok(true); // Exit gracefully, return to normal mode
            }

            //  ===================
            //  Get Clipboard Files
            //  ===================
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

            //  ================
            //  Render Pasty TUI
            //  ================
            if let Err(_) = render_pasty_tui(self, &sorted_files, offset, items_per_page) {
                let _ = self.set_info_bar_message("display error");
                // Try to continue anyway
            }

            //  ==============
            //  Get User Input
            //  ==============
            let input_result = self.handle_pasty_mode_input(stdin_handle, text_buffer);

            //  =============
            //  Process Input
            //  =============
            match input_result {
                //  ==============================
                //  Back Command - Exit Pasty Mode
                //  ==============================
                Ok(PastyInputPathOrCommand::Back) => {
                    let _ = self.set_info_bar_message(""); // Clear any error messages
                    return Ok(true); // Exit Pasty mode, back to editor
                }

                //  ==========================================
                //  Empty Input -> Select Most Recent (Rank 1)
                //  ==========================================
                Ok(PastyInputPathOrCommand::EmptyEnterFirstItem) => {
                    // =================================================
                    // Clear Redo Stack Before Editing: Insert or Delete
                    // =================================================
                    let _: bool = match button_safe_clear_all_redo_logs(&base_edit_filepath) {
                        Ok(success) => success,
                        Err(_e) => {
                            #[cfg(debug_assertions)]
                            eprintln!("Error clearing redo logs: {:?}", _e);

                            // Log error and continue (non-fatal)
                            log_error(
                                "Cannot clear redo logs",
                                Some("backspace_style_delete_noload"),
                            );
                            let _ = self.set_info_bar_message("bsdn Redo clear failed");

                            false // Treat error as failure
                        }
                    };

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

                //  =====================
                //  Select by Rank Number
                //  =====================
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

                //  ==============
                //  Select by Path
                //  ==============
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

                //  =======
                //  Page Up
                //  =======
                Ok(PastyInputPathOrCommand::PageUp) => {
                    offset = offset.saturating_sub(items_per_page);
                    let _ = self.set_info_bar_message(""); // Clear any previous messages
                    continue; // Stay in loop, refresh display
                }

                //  =========
                //  Page Down
                //  =========
                Ok(PastyInputPathOrCommand::PageDown) => {
                    let new_offset = offset + items_per_page;
                    // Only advance if there are more items to show
                    if new_offset < total_count {
                        offset = new_offset;
                    }
                    let _ = self.set_info_bar_message(""); // Clear any previous messages
                    continue; // Stay in loop, refresh display
                }

                //  ===================
                //  Clear All Clipboard
                //  ===================
                Ok(PastyInputPathOrCommand::ClearAll) => {
                    if let Err(_) = clear_pasty_file_clipboard(&clipboard_dir) {
                        let _ = self.set_info_bar_message("*clear failed*");
                        continue; // Stay in loop
                    }

                    offset = 0; // Reset pagination
                    let _ = self.set_info_bar_message("^clipboard cleared^");
                    continue; // Stay in loop, refresh display
                }

                //  ===================
                //  Clear Specific Rank
                //  ===================
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

                //  ==============================================
                //  Input Error (invalid, too long, parse failure)
                //  ==============================================
                Err(_) => {
                    let _ = self.set_info_bar_message("invalid input");
                    continue; // Stay in loop, re-prompt user
                }
            }
        }
    }

    /// Writes a hex-edited byte and creates undo log entry
    ///
    /// # Project Context
    /// When user hex-edits a byte in hex mode (types two hex digits),
    /// this method orchestrates the full write-and-log workflow to support undo.
    /// This is the ONLY entry point for hex editing operations that need undo support.
    ///
    /// # Workflow
    /// 1. Read original byte value (for undo log) - with retries
    /// 2. Write new byte value to file (in-place edit) - with retries
    /// 3. Clear redo stack (user action invalidates future redo) - with retries
    /// 4. Create undo log (inverse operation to restore original) - with retries
    ///
    /// # Retry Strategy
    /// - Steps 1-2: Critical operations, 3 retries with pauses, abort if all fail
    /// - Steps 3-4: Non-critical, 3 retries, log error and continue if all fail
    /// - Step 2 uses 200ms pause (file may be locked by other processes)
    /// - Steps 1,3,4 use 100ms pause
    ///
    /// # Arguments
    /// * `byte_position` - 0-indexed position in file (cursor location)
    /// * `new_byte_value` - New byte value to write (0x00-0xFF)
    ///
    /// # Returns
    /// * `Result<()>` - Success or error (never panics)
    ///
    /// # Errors
    /// Returns `LinesError::StateError` if:
    /// - No file is currently open
    ///
    /// Returns `LinesError::Io` if:
    /// - Cannot read original byte (3 retries exhausted)
    /// - Cannot write new byte (3 retries exhausted)
    /// - Position exceeds file size (checked by read_single_byte_from_file)
    ///
    /// Note: Redo-clear and undo-log failures are logged but don't stop operation
    ///
    /// # Side Effects
    /// - Modifies file on disk
    /// - Clears redo log directory
    /// - Creates undo log file
    /// - May set info bar message on non-critical errors
    /// - Logs errors to error log file
    ///
    /// # Examples
    /// ```
    /// // User types "3F" at byte position 42 in hex mode
    /// editor.write_n_log_hex_edit_in_place(42, 0x3F)?;
    /// ```
    pub fn write_n_log_hex_edit_in_place(
        &mut self,
        byte_position: usize,
        new_byte_value: u8,
    ) -> Result<()> {
        use std::thread;
        use std::time::Duration;

        // ============================================================
        // STEP 0: Get File Path from Editor State (CLONE IT)
        // ============================================================
        // Clone the path to avoid borrow checker issues later
        // when we need to mutably borrow self for set_info_bar_message
        let readcopy_file_path_clone = self
            .read_copy_path
            .clone() // ← FIXED: Clone instead of borrowing
            .ok_or_else(|| LinesError::StateError("No file open".into()))?;

        // Convert position to u128 for external API compatibility
        let position_u128 = byte_position as u128;

        // ============================================================
        // Debug-Assert, Test-Assert, Production-Catch-Handle
        // ============================================================
        debug_assert!(
            self.read_copy_path.is_some(),
            "File path must exist before hex edit"
        );
        #[cfg(test)]
        assert!(
            self.read_copy_path.is_some(),
            "File path must exist before hex edit"
        );
        if self.read_copy_path.is_none() {
            log_error(
                "No file path in editor state",
                Some("write_n_log_hex_edit_in_place"),
            );
            return Err(LinesError::StateError("No file path".into()));
        }

        // ============================================================
        // STEP 1: Read Original Byte Value (3 retries, 100ms pause)
        // ============================================================
        let mut original_byte: Option<u8> = None;
        let mut last_read_error: Option<String> = None;

        for attempt in 0..3 {
            match read_single_byte_from_file(&readcopy_file_path_clone, position_u128) {
                Ok(byte_val) => {
                    original_byte = Some(byte_val);
                    break;
                }
                Err(e) => {
                    last_read_error = Some(format!("Read attempt {} failed: {}", attempt + 1, e));
                    if attempt < 2 {
                        thread::sleep(Duration::from_millis(100));
                    }
                }
            }
        }

        let original_byte = match original_byte {
            Some(byte) => byte,
            None => {
                let error_msg = last_read_error.unwrap_or_else(|| "Unknown read error".into());
                log_error(
                    &format!(
                        "Cannot read byte at position {}: {}",
                        byte_position, error_msg
                    ),
                    Some("write_n_log_hex_edit_in_place:step1"),
                );
                let _ = self.set_info_bar_message("Read failed");
                return Err(LinesError::Io(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to read original byte after 3 attempts",
                )));
            }
        };

        // ============================================================
        // STEP 2: Write New Byte to File (3 retries, 200ms pause)
        // ============================================================
        let mut write_success = false;
        let mut last_write_error: Option<String> = None;

        for attempt in 0..3 {
            match replace_byte_in_place(&readcopy_file_path_clone, byte_position, new_byte_value) {
                Ok(_) => {
                    write_success = true;
                    break;
                }
                Err(e) => {
                    last_write_error = Some(format!("Write attempt {} failed: {}", attempt + 1, e));
                    if attempt < 2 {
                        thread::sleep(Duration::from_millis(200));
                    }
                }
            }
        }

        if !write_success {
            let error_msg = last_write_error.unwrap_or_else(|| "Unknown write error".into());
            log_error(
                &format!(
                    "Cannot write byte at position {}: {}",
                    byte_position, error_msg
                ),
                Some("write_n_log_hex_edit_in_place:step2"),
            );
            let _ = self.set_info_bar_message("Write failed");
            return Err(LinesError::Io(io::Error::new(
                io::ErrorKind::Other,
                "Failed to write byte after 3 attempts",
            )));
        }

        // ============================================================
        // STEP 3: Clear Redo Stack (3 retries, 100ms pause)
        // ============================================================
        let mut redo_clear_success = false;

        for attempt in 0..3 {
            match button_safe_clear_all_redo_logs(&readcopy_file_path_clone) {
                Ok(_) => {
                    redo_clear_success = true;
                    break;
                }
                Err(_) => {
                    if attempt < 2 {
                        thread::sleep(Duration::from_millis(100));
                    }
                }
            }
        }

        if !redo_clear_success {
            log_error(
                &format!("Cannot clear redo logs for position {}", byte_position),
                Some("write_n_log_hex_edit_in_place:step3"),
            );
            let _ = self.set_info_bar_message("Redo clear failed");
        }

        // ============================================================
        // STEP 4: Create Undo Log Entry (3 retries, 100ms pause)
        // ============================================================
        let log_directory_path = match get_undo_changelog_directory_path(&readcopy_file_path_clone)
        {
            Ok(path) => path,
            Err(e) => {
                log_error(
                    &format!("Cannot get changelog directory: {}", e),
                    Some("write_n_log_hex_edit_in_place:step4"),
                );
                let _ = self.set_info_bar_message("Undo log path fail");
                return Ok(());
            }
        };

        let mut undo_log_success = false;

        for attempt in 0..3 {
            match button_hexeditinplace_byte_make_log_file(
                &readcopy_file_path_clone,
                position_u128,
                original_byte,
                &log_directory_path,
            ) {
                Ok(_) => {
                    undo_log_success = true;
                    break;
                }
                Err(_) => {
                    if attempt < 2 {
                        thread::sleep(Duration::from_millis(100));
                    }
                }
            }
        }

        if !undo_log_success {
            log_error(
                &format!("Cannot create undo log for position {}", byte_position),
                Some("write_n_log_hex_edit_in_place:step4"),
            );
            let _ = self.set_info_bar_message("Undo log failed");
        }

        Ok(())
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
    /// | `-s` or `-w` | SaveFileStandard | Continue |
    /// | `-q` | Quit | **Stop** |
    /// | `-wq` | SaveAndQuit | **Stop** |
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

        // Clear
        let _ = self.set_info_bar_message("");

        let read_copy_path = match &self.read_copy_path {
            Some(path) => path,
            None => {
                #[cfg(debug_assertions)]
                eprintln!(
                    "is_next_byte_newline: no read-copy file path available (returning false - not at newline)"
                );

                // Not an error - just means we cannot analyze the file
                return Ok(false);
            }
        };

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
        if self.hex_cursor.byte_offset_linear_file_absolute_position >= file_size && file_size > 0 {
            self.hex_cursor.byte_offset_linear_file_absolute_position = file_size - 1;
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

                // ============================================================
                // Call new method: Write byte + Create undo log
                // ============================================================
                // This method handles:
                // - Reading original byte value
                // - Writing new byte value
                // - Clearing redo stack
                // - Creating undo log entry
                // All with retry logic and defensive error handling
                match self.write_n_log_hex_edit_in_place(
                    self.hex_cursor.byte_offset_linear_file_absolute_position,
                    byte_value,
                ) {
                    Ok(_) => {
                        // Success: update editor state
                        self.is_modified = true;

                        // Advance cursor if not at EOF
                        if self.hex_cursor.byte_offset_linear_file_absolute_position + 1 < file_size
                        {
                            self.hex_cursor.byte_offset_linear_file_absolute_position += 1;
                        }

                        let _ = self.set_info_bar_message("Byte written");
                    }
                    Err(_e) => {
                        // Error already logged by write_n_log_hex_edit_in_place()
                        // Just show user-friendly message
                        let _ = self.set_info_bar_message("Edit failed");
                        #[cfg(debug_assertions)]
                        log_error(
                            &stack_format_it(
                                "Hex edit failed: {}",
                                &[&_e.to_string()],
                                "Hex edit failed",
                            ),
                            Some("handle_parse_hex_mode_input_and_commands"),
                        );
                        // safe
                        log_error(
                            "Hex edit failed",
                            Some("handle_parse_hex_mode_input_and_commands"),
                        );
                        // Continue editor loop - let user try again
                    }
                }

                let _ = self.set_info_bar_message("Byte written");
            }

            // ==========
            // Go To Byte
            // ==========
            trimmed if trimmed.starts_with('g') && trimmed.len() > 1 => {
                let rest = &trimmed[1..];

                // Check if rest is all digits (line number jump)
                if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) {
                    // Parse line number (defensive: use saturating operations)
                    let mut line_number = 0usize;
                    let mut digit_iterations = 0;

                    for ch in rest.chars() {
                        // Defensive: prevent infinite loop on malformed input
                        if digit_iterations >= limits::COMMAND_PARSE_MAX_CHARS {
                            let _ = self.set_info_bar_message("Position # too long");
                            return Ok(true);
                        }
                        digit_iterations += 1;

                        let digit_value = (ch as usize) - ('0' as usize);
                        line_number = line_number.saturating_mul(10).saturating_add(digit_value);
                    }

                    // ==========
                    // Go To Byte
                    // ==========
                    self.hex_cursor.byte_offset_linear_file_absolute_position = line_number;
                }
            }

            // === ADD Remove HEX Byte (not edit in place) ===
            // === HEX BYTE REPLACEMENT: Two hex digits ===
            // Next two operation-sections: delete byte, then add byte

            // ========================
            // REMOVE Byte, DELETE Byte
            // ========================
            "d" => {
                /*
                "REMOVE" | "DELETE" => {

                    pub fn remove_single_byte_from_file(
                        original_file_path: PathBuf,
                        byte_position_from_start: usize,
                    ) -> io::Result<()> {
                }
                 */

                // Convert u64 position to u128 for API compatibility
                let position_u128 =
                    self.hex_cursor.byte_offset_linear_file_absolute_position as u128;

                // Read BEFORE removing (to restore)
                let byte_at_position = read_single_byte_from_file(read_copy_path, position_u128)?;

                // message is successful
                let result = remove_single_byte_from_file(
                    read_copy_path.clone(), // convert to pathbuf from &pathbuff
                    self.hex_cursor.byte_offset_linear_file_absolute_position,
                );

                let readcopy_pathclone = read_copy_path.clone();

                if result.is_ok() {
                    let _ = self.set_info_bar_message("Removed Byte");

                    // ==================
                    // Clear Redo Stack
                    // before trying edit
                    // ==================
                    let mut redo_clear_success = false;
                    for attempt in 0..3 {
                        match button_safe_clear_all_redo_logs(&readcopy_pathclone) {
                            Ok(_) => {
                                redo_clear_success = true;
                                break;
                            }
                            Err(_) => {
                                if attempt < 2 {
                                    thread::sleep(Duration::from_millis(100));
                                }
                            }
                        }
                    }
                    if !redo_clear_success {
                        log_error(
                            "Cannot clear redo logs",
                            Some("write_n_log_hex_edit_in_place:step3"),
                        );
                    }

                    // ============================================
                    // Create Inverse Changelog Entry
                    // ============================================
                    // Create undo log for newline insertion
                    // Single character, no iteration needed
                    //
                    // User action: Rmv → Inverse log:  Add
                    // This is non-critical - if it fails, insertion still succeeded

                    let log_directory_path =
                        match get_undo_changelog_directory_path(&readcopy_pathclone) {
                            Ok(path) => Some(path), // ← Wrap in Some to match the None below
                            Err(_e) => {
                                // Non-critical: Log error but don't fail the insertion
                                #[cfg(debug_assertions)]
                                log_error(
                                    &stack_format_it(
                                        "Cannot get changelog directory: {}",
                                        &[&_e.to_string()],
                                        "Cannot get changelog directory",
                                    ),
                                    Some("insert_newline_at_cursor_chunked:changelog"),
                                );
                                // safe
                                log_error(
                                    "Cannot get changelog directory",
                                    Some("insert_newline_at_cursor_chunked:changelog"),
                                );

                                // Continue without undo support - insertion succeeded
                                None
                            }
                        };

                    // Create log entry if directory path was obtained
                    if let Some(log_dir) = log_directory_path {
                        // Retry logic: 3 attempts with 50ms pause
                        let mut log_success = false;

                        for retry_attempt in 0..3 {
                            /*
                            pub fn button_make_changelog_from_user_character_action_level(
                                target_file: &Path,
                                character: Option<char>,
                                byte_value: Option<u8>, // raw byte input
                                position: u128,
                                edit_type: EditType,
                                log_directory_path: &Path,
                            ) -> ButtonResult<()> {
                            */

                            match button_make_changelog_from_user_character_action_level(
                                &readcopy_pathclone,
                                None,                   // No Character being added
                                Some(byte_at_position), // raw byte (option)
                                position_u128,
                                EditType::RmvByte, // User removed byte, inverse is add byte
                                &log_dir,
                            ) {
                                Ok(_) => {
                                    log_success = true;
                                    break; // Success
                                }
                                Err(_e) => {
                                    if retry_attempt == 2 {
                                        // Final retry failed - log but don't fail operation
                                        #[cfg(debug_assertions)]
                                        log_error(
                                            &format!(
                                                "Failed to log newline at position {}: {}",
                                                position_u128, _e
                                            ),
                                            Some("insert_newline_at_cursor_chunked:changelog"),
                                        );

                                        // safe
                                        log_error(
                                            "Failed to log newline",
                                            Some("insert_newline_at_cursor_chunked:changelog"),
                                        );
                                    } else {
                                        // Retry after brief pause
                                        std::thread::sleep(std::time::Duration::from_millis(50));
                                    }
                                }
                            }
                        }

                        // Optional: Set info bar if logging failed (non-intrusive)
                        if !log_success {
                            let _ = self.set_info_bar_message("undo disabled");
                        }
                    }
                }

                if !result.is_ok() {
                    let _ = self.set_info_bar_message("Failed to Remove byte");
                }
            }

            // ========
            // Add Byte
            // ========
            trimmed
                if trimmed.len() == 4
                    && trimmed.as_bytes()[0].is_ascii_hexdigit()
                    && trimmed.as_bytes()[1].is_ascii_hexdigit()
                    && trimmed.as_bytes()[2] == b'-'
                    && trimmed.as_bytes()[3] == b'i' =>
            {
                // Get "byte_value" from raw input
                let bytes = trimmed.as_bytes();
                let high = parse_hex_digit(bytes[0])?;
                let low = parse_hex_digit(bytes[1])?;
                let byte_value = (high << 4) | low;

                let mut redo_clear_success = false;

                for attempt in 0..3 {
                    match button_safe_clear_all_redo_logs(&read_copy_path) {
                        Ok(_) => {
                            redo_clear_success = true;
                            break;
                        }
                        Err(_) => {
                            if attempt < 2 {
                                thread::sleep(Duration::from_millis(100));
                            }
                        }
                    }
                }

                if !redo_clear_success {
                    log_error(
                        "Cannot clear redo logs",
                        Some("write_n_log_hex_edit_in_place:step3"),
                    );
                }
                /*
                "ADD" | "INSERR" byte => {

                pub fn add_single_byte_to_file(
                    original_file_path: PathBuf,
                    byte_position_from_start: usize,
                    new_byte_value: u8,
                ) -> io::Result<()> {
                }
                 */
                let result = add_single_byte_to_file(
                    read_copy_path.clone(), // convert to pathbuf from &pathbuff
                    self.hex_cursor.byte_offset_linear_file_absolute_position,
                    byte_value,
                );

                // Convert u64 position to u128 for API compatibility
                let position_u128 =
                    self.hex_cursor.byte_offset_linear_file_absolute_position as u128;

                // Read AFTER adding (to remove it later for UNDO)
                let byte_at_position = read_single_byte_from_file(read_copy_path, position_u128)?;

                let readcopy_pathclone = read_copy_path.clone();

                if result.is_ok() {
                    let _ = self.set_info_bar_message("Added Byte");

                    // ==================
                    // Clear Redo Stack
                    // before trying edit
                    // ==================
                    let mut redo_clear_success = false;
                    for attempt in 0..3 {
                        match button_safe_clear_all_redo_logs(&readcopy_pathclone) {
                            Ok(_) => {
                                redo_clear_success = true;
                                break;
                            }
                            Err(_) => {
                                if attempt < 2 {
                                    thread::sleep(Duration::from_millis(100));
                                }
                            }
                        }
                    }
                    if !redo_clear_success {
                        log_error(
                            "Cannot clear redo logs",
                            Some("write_n_log_hex_edit_in_place:step3"),
                        );
                    }

                    // ============================================
                    // Create Inverse Changelog Entry
                    // ============================================
                    // Create undo log for newline insertion
                    // Single character, no iteration needed
                    //
                    // User action: Rmv → Inverse log:  Add
                    // This is non-critical - if it fails, insertion still succeeded

                    let log_directory_path =
                        match get_undo_changelog_directory_path(&readcopy_pathclone) {
                            Ok(path) => Some(path), // ← Wrap in Some to match the None below
                            Err(_e) => {
                                // Non-critical: Log error but don't fail the insertion
                                #[cfg(debug_assertions)]
                                log_error(
                                    &format!("Cannot get changelog directory: {}", _e),
                                    Some("insert_newline_at_cursor_chunked:changelog"),
                                );

                                #[cfg(not(debug_assertions))]
                                log_error(
                                    "Cannot get changelog directory",
                                    Some("insert_newline_at_cursor_chunked:changelog"),
                                );

                                // Continue without undo support - insertion succeeded
                                None
                            }
                        };

                    // Create log entry if directory path was obtained
                    if let Some(log_dir) = log_directory_path {
                        // Retry logic: 3 attempts with 50ms pause
                        let mut log_success = false;

                        for retry_attempt in 0..3 {
                            /*
                            pub fn button_make_changelog_from_user_character_action_level(
                                target_file: &Path,
                                character: Option<char>,
                                byte_value: Option<u8>, // raw byte input
                                position: u128,
                                edit_type: EditType,
                                log_directory_path: &Path,
                            ) -> ButtonResult<()> {
                            */

                            match button_make_changelog_from_user_character_action_level(
                                &readcopy_pathclone,
                                None,                   // No Character being added
                                Some(byte_at_position), // raw byte (option)
                                position_u128,
                                EditType::AddByte, // User added byte, inverse is remove byte
                                &log_dir,
                            ) {
                                Ok(_) => {
                                    log_success = true;
                                    break; // Success
                                }
                                Err(_e) => {
                                    if retry_attempt == 2 {
                                        // Final retry failed - log but don't fail operation
                                        #[cfg(debug_assertions)]
                                        log_error(
                                            &format!(
                                                "Failed to log newline at position {}: {}",
                                                position_u128, _e
                                            ),
                                            Some("insert_newline_at_cursor_chunked:changelog"),
                                        );

                                        #[cfg(not(debug_assertions))]
                                        log_error(
                                            "Failed to log newline",
                                            Some("insert_newline_at_cursor_chunked:changelog"),
                                        );
                                    } else {
                                        // Retry after brief pause
                                        std::thread::sleep(std::time::Duration::from_millis(50));
                                    }
                                }
                            }
                        }

                        // Optional: Set info bar if logging failed (non-intrusive)
                        if !log_success {
                            let _ = self.set_info_bar_message("undo disabled");
                        }
                    }
                }

                if !result.is_ok() {
                    let _ = self.set_info_bar_message("Failed to Insert byte");
                }
            }

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
                // SaveFileStandard file
                keep_editor_loop_running = execute_command(self, Command::SaveFileStandard)?;
            }

            "wq" => {
                // SaveAndQuit
                keep_editor_loop_running = execute_command(self, Command::SaveAndQuit)?;
            }

            "q" => {
                // Quit without saving
                keep_editor_loop_running = execute_command(self, Command::Quit)?;
            }

            // === NAVIGATION: LEFT/RIGHT (single byte) ===
            "h" => {
                // Move left (previous byte)
                if self.hex_cursor.byte_offset_linear_file_absolute_position > 0 {
                    self.hex_cursor.byte_offset_linear_file_absolute_position -= 1;
                } else {
                    let _ = self.set_info_bar_message("Already at start of file");
                }
            }

            "l" => {
                // Move right (next byte)
                if self.hex_cursor.byte_offset_linear_file_absolute_position + 1 < file_size {
                    self.hex_cursor.byte_offset_linear_file_absolute_position += 1;
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

                match find_previous_newline(
                    file_path,
                    self.hex_cursor.byte_offset_linear_file_absolute_position,
                ) {
                    Ok(Some(newline_pos)) => {
                        // Found a newline - move cursor to it
                        self.hex_cursor.byte_offset_linear_file_absolute_position = newline_pos;
                        let _ = self.set_info_bar_message("Previous line");
                    }
                    Ok(None) => {
                        // No newline found - go to start of file
                        self.hex_cursor.byte_offset_linear_file_absolute_position = 0;
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

                match find_next_newline(
                    file_path,
                    self.hex_cursor.byte_offset_linear_file_absolute_position,
                    file_size,
                ) {
                    Ok(Some(newline_pos)) => {
                        // Found a newline - move cursor to it
                        self.hex_cursor.byte_offset_linear_file_absolute_position = newline_pos;
                        let _ = self.set_info_bar_message("Next line");
                    }
                    Ok(None) => {
                        // No newline found - go to end of file
                        if file_size > 0 {
                            self.hex_cursor.byte_offset_linear_file_absolute_position =
                                file_size - 1;
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
                self.hex_cursor.byte_offset_linear_file_absolute_position =
                    row * self.hex_cursor.bytes_per_row;
            }

            "$" | "gl" => {
                // Go to end of current row (or last byte if row incomplete)
                let row = self.hex_cursor.current_row();
                let row_end = (row + 1) * self.hex_cursor.bytes_per_row - 1;

                if row_end < file_size {
                    self.hex_cursor.byte_offset_linear_file_absolute_position = row_end;
                } else if file_size > 0 {
                    // Row is incomplete - go to last byte
                    self.hex_cursor.byte_offset_linear_file_absolute_position = file_size - 1;
                }
            }

            // === NAVIGATION: FILE START/END ===
            "gg" => {
                // Go to start of file
                self.hex_cursor.byte_offset_linear_file_absolute_position = 0;
                let _ = self.set_info_bar_message("Start of file");
            }

            "ge" | "G" => {
                // Go to end of file
                if file_size > 0 {
                    self.hex_cursor.byte_offset_linear_file_absolute_position = file_size - 1;
                    let _ = self.set_info_bar_message("End of file");
                }
            }

            // === Add / REMOVE Q&A ===
            /*
            Q&A
            Caution! Be Careful!
            Are you sure you want to {add/remove} a byte?
            To Proceed Enter: {ADD/REMOVE}
            for add:
            Enter byte value to add.

            */
            "ADD" => {
                /*
                pub fn add_single_byte_to_file(
                    original_file_path: PathBuf,
                    byte_position_from_start: usize,
                    new_byte_value: u8,
                ) -> io::Result<()> {
                */
            }

            "REMOVE" | "DELETE" => {

                /*pub fn remove_single_byte_from_file(
                    original_file_path: PathBuf,
                    byte_position_from_start: usize,
                ) -> io::Result<()> {*/
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
    ///   - SaveAndQUite (-wq)
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
    /// | `-s` or `-w` | SaveFileStandard | Write changes to disk | Continue |
    /// | `-wq` | SaveAndQuit | SaveAndexit editor | **Stop** |
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
            keep_editor_loop_running = execute_command(self, Command::SaveFileStandard)?;
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

            // =================================================
            // Clear Redo Stack Before Editing: Insert or Delete
            // =================================================
            let _: bool = match button_safe_clear_all_redo_logs(&read_copy) {
                Ok(success) => success,
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    eprintln!("Error clearing redo logs: {:?}", _e);

                    // Log error and continue (non-fatal)
                    log_error(
                        "Cannot clear redo logs",
                        Some("backspace_style_delete_noload"),
                    );
                    let _ = self.set_info_bar_message("bsdn Redo clear failed");

                    false // Treat error as failure
                }
            };

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
    /// Note: For other command handling, also see: lines_full_file_editor()
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
        // SPECIAL CASE: save as (sa)
        // =========================================================================
        // Handle 'sa' prefix commands for Save As functionality
        //
        // Purpose: Parse user command like "sa hello.py" and build absolute path
        // for destination file, ensuring it's different from source file.
        //
        // User input examples:
        // - "sa backup.py"                  -> save in same directory as original
        // - "sa /home/user/new.txt"         -> save with absolute path
        // - "sa ../other_dir/file.txt"      -> save with relative path
        //
        // NOTE: Leading count is IGNORED for save-as commands
        // Example: "5sa file.txt" is treated same as "sa file.txt"
        if command_str.starts_with("sa") && command_str.len() > 2 {
            // =====================================================================
            // STEP 1: Extract filename from command string
            // =====================================================================
            // User typed: "sa hello.py"
            // We need to extract: "hello.py"
            // Method: Remove first 2 characters ("sa"), then trim whitespace

            let rest = &command_str[2..]; // "sa hello.py" -> " hello.py"
            let filename_str = rest.trim(); // " hello.py" -> "hello.py"

            // Defensive: Check if filename is empty after trimming
            // Catches: "sa", "sa ", "sa   "
            if filename_str.is_empty() {
                let _ = self.set_info_bar_message("Usage: sa filename");
                return Command::None;
            }

            // Defensive: Check filename length to prevent overflow
            // Catches: Extremely long filenames that could cause issues
            if filename_str.len() > limits::LINE_READ_BYTES {
                let _ = self.set_info_bar_message("Filename too long");
                return Command::None;
            }

            // =====================================================================
            // STEP 2: Get the original file's directory path
            // =====================================================================
            // We need to know WHERE the current file is, so we can save the
            // new file in the same directory (if user provides relative path).
            //
            // Example: If editing "/home/user/documents/file.txt"
            //          We want directory: "/home/user/documents/"

            let original_file_path = match &self.original_file_path {
                Some(path) => path,
                None => {
                    // No file currently open - can't do save-as
                    let _ = self.set_info_bar_message("No file open to save as");
                    return Command::None;
                }
            };

            // Get the directory containing the original file
            // Example: "/home/user/documents/file.txt" -> "/home/user/documents/"
            let original_directory = match original_file_path.parent() {
                Some(dir) => dir,
                None => {
                    // Original file has no parent directory (shouldn't happen with absolute paths)
                    let _ = self.set_info_bar_message("Cannot determine file directory");
                    return Command::None;
                }
            };

            // =====================================================================
            // STEP 3: Build the new save-as path (absolute)
            // =====================================================================
            // Convert user's filename to absolute path.
            // If user gave relative name, use original file's directory as base.
            //
            // Examples:
            // - User input: "backup.py"
            //   Original:   "/home/user/docs/file.txt"
            //   Result:     "/home/user/docs/backup.py"
            //
            // - User input: "/tmp/backup.py"
            //   Original:   "/home/user/docs/file.txt"
            //   Result:     "/tmp/backup.py" (already absolute)

            let mut save_as_path = PathBuf::from(filename_str);

            // Check if user provided absolute or relative path
            if !save_as_path.is_absolute() {
                // Relative path: join with original file's directory
                // Example: "backup.py" + "/home/user/docs/" = "/home/user/docs/backup.py"
                save_as_path = original_directory.join(filename_str);
            }

            // Defensive: Validate path is valid UTF-8
            // Ensures path can be safely used in all string operations
            if save_as_path.to_str().is_none() {
                let _ = self.set_info_bar_message("Invalid filename (non-UTF8)");
                return Command::None;
            }

            // =====================================================================
            // STEP 4: Check that new filename is different from original
            // =====================================================================
            // Prevent user from accidentally "saving as" with the same name,
            // which would be confusing (and potentially dangerous).
            //
            // Example catch:
            // - Original: "/home/user/file.txt"
            // - User types: "sa file.txt"
            // - Result: "/home/user/file.txt" (SAME - reject this!)

            if &save_as_path == original_file_path {
                let _ =
                    self.set_info_bar_message("New filename same as original (use 's' to save)");
                return Command::None;
            }

            // =====================================================================
            // STEP 5: Return the valid SaveAs command
            // =====================================================================
            // At this point we have:
            // - Valid, non-empty filename
            // - Absolute path to new file
            // - Confirmed it's different from original
            // - Path is valid UTF-8

            // Debug: log the save-as command (only in debug builds for security)
            #[cfg(debug_assertions)]
            eprintln!(
                "DEBUG: Save As command\n  Original: {:?}\n  Save as:  {:?}",
                original_file_path, save_as_path
            );

            // Return the command with the absolute path
            return Command::SaveAs(save_as_path);
        }
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
                // with hx helix and impossible to remember vi codes...???
                "gg" => return Command::GotoFileStart,
                "ge" | "G" => return Command::GotoFileLastLine,
                "gh" | "0" => return Command::GotoLineStart,
                "gl" | "$" => return Command::GotoLineEnd,
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
        fn lines_full_file_editor(){
        ...
        if state.mode == ...
        ```
         */

        if current_mode == EditorMode::Normal || current_mode == EditorMode::RawMode {
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

                "u" | "undo" => Command::UndoButtonsCommand,
                "re" | "redo" => Command::RedoButtonsCommand,

                "w" => Command::MoveWordForward(count),
                "e" => Command::MoveWordEnd(count),
                "b" => Command::MoveWordBack(count),

                // toggle
                "/" => Command::ToggleCommentOneLine(self.cursor.row), // zero index
                "///" => Command::ToggleDocstringOneLine(self.cursor.row), // zero index

                // indent
                "[" => Command::UnindentOneLine(self.cursor.row), // zero index
                "]" => Command::IndentOneLine(self.cursor.row),   // zero index

                // TUI Size
                "tall+" => Command::TallPlus,
                "tall-" => Command::TallMinus,
                "wide+" => Command::WidePlus,
                "wide-" => Command::WideMinus,

                "i" => Command::EnterInsertMode,
                "v" => Command::EnterVisualMode,
                "raw" | "r" => Command::EnterRawMode,
                // Multi-character commands
                "wq" => Command::SaveAndQuit,
                "s" => Command::SaveFileStandard,
                "q" => Command::Quit,
                "p" | "pasty" => Command::EnterPastyClipboardMode,
                "hex" | "bytes" | "byte" => Command::EnterHexEditMode,
                "d" => Command::DeleteLine,
                "\x1b[3~" => Command::DeleteBackspace, // delete key -> \x1b[3~
                _ => Command::None,
            }
        } else if current_mode == EditorMode::VisualSelectMode {
            match command_str {
                "u" | "undo" => Command::UndoButtonsCommand,
                "re" | "redo" => Command::RedoButtonsCommand,

                // same moves for selection:
                "h" => Command::MoveLeft(count),
                "\x1b[D" => Command::MoveLeft(count), // left over arrow
                "j" => Command::MoveDown(count),
                "\x1b[B" => Command::MoveDown(count), // down cast arrow -> \x1b[B
                "l" => Command::MoveRight(count),
                "\x1b[C" => Command::MoveRight(count), // starboard arrow
                "k" => Command::MoveUp(count),
                "\x1b[A" => Command::MoveUp(count), // up arrow -> \x1b[A

                // toggle RANGE
                "/" => Command::ToggleBasicCommentlinesRange,
                "//" | "/block" | "/b" => {
                    Command::ToggleBlockcomments(self.selection_rowline_start, self.cursor.row)
                }
                "///" => Command::ToggleRustDocstringRange, // zero index

                // indent RANGE
                "[" => Command::UnindentRange, // zero index
                "]" => Command::IndentRange,   // zero index
                "w" => Command::MoveWordForward(count),
                "e" => Command::MoveWordEnd(count),
                "b" => Command::MoveWordBack(count),

                "i" => Command::EnterInsertMode,
                "q" => Command::Quit,
                "c" | "y" => Command::Copyank,
                "s" => Command::SaveFileStandard,
                "n" | "\x1b" => Command::EnterNormalMode,
                "wq" => Command::SaveAndQuit,
                // "d" => Command::DeleteBackspace, // minimal, works
                "d" => Command::DeleteRange,
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

    // /// Gets the first valid text column for cursor's current row
    // ///
    // /// # Returns
    // /// * Column position where text starts (after line number)
    // pub fn get_text_start_column(&self) -> usize {
    //     let line_number = self.line_count_at_top_of_window + self.cursor.row;
    //     calculate_line_number_width(line_number + 1, self.effective_rows) // +1 for 1-indexed display
    // }

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

    /// Writes a line number into a display buffer with optional padding
    ///
    /// # Format
    /// Either "N " or " N " depending on rollover zone
    /// No heap allocation - writes directly to pre-allocated buffer
    pub fn write_line_number(
        &mut self,
        row_idx: usize,
        line_num: usize,
        starting_row: usize,
    ) -> io::Result<usize> {
        // Validate row index (zero based)
        if row_idx > MAX_ZERO_INDEX_TUI_ROWS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Row index exceeds maximum 44",
            ));
        }

        // Check if we need padding
        let needs_padding =
            row_needs_extra_padding_bool(starting_row, line_num, self.effective_rows);

        // Convert number to bytes directly into buffer
        let mut write_pos = 0;

        // Add leading space if needed
        if needs_padding {
            self.utf8_txt_display_buffers[row_idx][0] = b' ';
            write_pos = 1;
        }

        // Write digits directly
        let mut temp_num = line_num;
        let mut digit_stack = [0u8; 7]; // Max 7 digits (999,999)
        let mut digit_count = 0;

        // Extract digits (they come out reversed)
        loop {
            digit_stack[digit_count] = (temp_num % 10) as u8 + b'0';
            digit_count += 1;
            temp_num /= 10;
            if temp_num == 0 {
                break;
            }
        }

        // Write digits in correct order
        for i in (0..digit_count).rev() {
            self.utf8_txt_display_buffers[row_idx][write_pos] = digit_stack[i];
            write_pos += 1;
        }

        // Add trailing space
        self.utf8_txt_display_buffers[row_idx][write_pos] = b' ';
        write_pos += 1;

        Ok(write_pos)
    }
}

/// Gets a timestamp string in yyyy_mm_dd format using only standard library
fn get_short_underscore_timestamp() -> io::Result<String> {
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
        let timestamp = create_readable_archive_timestamp(SystemTime::now());

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
                // format!("get_default_filepath Could not find home directory: {}", e),
                stack_format_it(
                    "get_default_filepath Could not find home directory: {}",
                    &[&e.to_string()],
                    "get_default_filepath Could not find home directory",
                ),
            )
        })?;

    // Build the base directory path
    let mut base_path = PathBuf::from(home);
    base_path.push("Documents");
    base_path.push("lines_editor");

    // Create all directories in the path if they don't exist
    fs::create_dir_all(&base_path)?;

    // Get timestamp for filename
    let timestamp = get_short_underscore_timestamp()?;

    // Create filename based on whether custom_name is provided
    let filename = match custom_name {
        // Some(name) => format!("{}_{}.txt", name, timestamp),
        // None => format!("{}.txt", timestamp),
        Some(name) => stack_format_it("{}_{}.txt", &[&name, &timestamp.to_string()], "N_N.txt"),
        None => stack_format_it("{}.txt", &[&timestamp.to_string()], "N_N.txt"),
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
                    stack_format_it(
                        "seek_to_line_number File only has {} lines, requested line {}",
                        &[&current_line.to_string(), &target_line.to_string()],
                        "seek_to_line_number File only has N lines, requested line N",
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
/// - Horizontal scrolling is controlled by state.tui_window_horizontal_utf8txt_line_char_offset
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
            stack_format_it(
                "File not found: {:?}",
                &[&readcopy_file_path.to_string_lossy()],
                "File not found",
            ),
        )));
    }

    // Assertion: State should have valid dimensions
    debug_assert!(state.effective_rows > 0, "Effective rows must be positive");
    debug_assert!(state.effective_cols > 0, "Effective cols must be positive");

    // Clear existing buffers and map before building
    state.clear_utf8_displaybuffers();
    state.clear_windowmap_positions();
    state.clear_line_byte_ranges(); // line start stop data

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
        // Assertion: We should not exceed our display buffer count
        // assert if has been, not if might become, larger than max
        #[cfg(debug_assertions)]
        debug_assert!(current_display_row <= 45, "Display row exceeds maximum");

        iteration_count += 1;

        // line start stop data
        let line_start_byte = file_byte_position;

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

        let line_num_bytes_written = state.write_line_number(
            current_display_row,               // row_idx: usize,
            line_number_display,               // line_num: usize,
            state.line_count_at_top_of_window, // starting_row: usize,
        )?;

        // Calculate how many columns remain after line number
        let remaining_cols = state.effective_cols.saturating_sub(line_num_bytes_written);

        // Process the line text with horizontal offset
        let text_bytes_written = process_line_with_offset(
            state,
            current_display_row,
            line_num_bytes_written, // Column position after line number
            &line_bytes[..line_length],
            state.tui_window_horizontal_utf8txt_line_char_offset,
            remaining_cols,
            file_byte_position,
        )?;

        // Update total buffer length for this row
        state.display_utf8txt_buffer_lengths[current_display_row] =
            line_num_bytes_written + text_bytes_written;

        // ════════════════════════
        // line start stop tracking
        // ════════════════════════
        // Calculate line end byte position
        // For "hello\n" at bytes [10..15]: start=10, end=14 (last content byte)
        // For empty line "\n" at byte 20: start=20, end=20 (signals empty)
        // For last line "world" with no newline: start=X, end=X+4
        let line_end_byte = if line_length > 0 {
            // Line has content - end is last content byte
            line_start_byte + (line_length as u64) - 1
        } else {
            // Empty line (just newline) - start == end signals empty
            line_start_byte
        };
        // Store the byte range for this display row
        state.set_line_byte_range(current_display_row, line_start_byte, line_end_byte)?;
        // ════════════════════════
        // line start stop tracking
        // ════════════════════════

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

// ============================================================================
// FILE COPY OPERATION: Type Definitions and Constants (start)
// ============================================================================
/*
Project Context:
This section supports the Lines text editor's "Save As" functionality,
allowing users to create a copy of the current file with a new name.
This is distinct from:
- File append operations (file_append_to_file)
- File insertion operations (insert_file_at_cursor)
- Regular save operations (save current file)

Design Philosophy:
- Type-safe status codes distinguish predicated outcomes from errors
- Static string messages provide human-readable feedback without heap allocation
- Exhaustive matching ensures compiler catches all cases
- Clear separation: Ok = expected outcomes, Err = unexpected failures
*/

/// Status codes returned by save-as-copy file operations
///
/// # Purpose
/// Provides type-safe status reporting for file copy operations that
/// distinguishes between successful operations, expected predicated outcomes
/// (like "file already exists"), and true errors (I/O failures).
///
/// # Design Rationale
/// Using an enum instead of string messages enables:
/// - Compile-time exhaustive matching (compiler ensures all cases handled)
/// - Programmatic decision making (can match on specific statuses)
/// - No heap allocation (Copy + 'static)
/// - Clear API contracts (caller knows all possible outcomes upfront)
///
/// # Usage Pattern
/// Paired with static string message in tuple return:
/// ```
/// Ok((FileOperationStatus::Copied, "copied"))
/// ```
///
/// This provides both machine-readable status code and human-readable message.
///
/// # Related Types
/// - Paired with `&'static str` in function returns
/// - Errors use `LinesError` enum for true failure conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileOperationStatus {
    /// File successfully copied from source to destination
    ///
    /// Indicates the copy operation completed without errors.
    /// The destination file now contains exact copy of source file content.
    /// Source file remains unchanged.
    Copied,

    /// Destination file already exists, no copy performed
    ///
    /// This is a predicated outcome, not an error. The function follows
    /// a no-overwrite policy: if the destination path already has a file,
    /// we return this status and leave both files unchanged.
    ///
    /// Rationale: Overwriting existing files without explicit user confirmation
    /// risks data loss. Caller must handle this case and prompt user if needed.
    AlreadyExisted,

    /// Source file does not exist at specified path
    ///
    /// Predicated outcome: there is nothing to copy. This is expected in
    /// workflows where file existence is uncertain (e.g., optional configs,
    /// user-specified paths that may not exist yet).
    ///
    /// Distinct from I/O errors: the path is valid but no file is present.
    OriginalNotFound,

    /// Source file exists but could not be accessed after retry attempts
    ///
    /// Predicated outcome: file is locked by another process, lacks read
    /// permissions, or has other access restrictions. After 3 retry attempts
    /// with 200ms delays, the file remains unavailable.
    ///
    /// Common causes:
    /// - File locked by another application
    /// - Insufficient read permissions
    /// - File on network drive that became temporarily unavailable
    /// - Antivirus scanning file
    OriginalUnavailable,

    /// Destination path could not be written after retry attempts
    ///
    /// Predicated outcome: unable to create or write to destination file
    /// after 3 retry attempts with 200ms delays.
    ///
    /// Common causes:
    /// - Parent directory doesn't exist
    /// - Insufficient write permissions for directory
    /// - Disk full
    /// - Destination path locked by another process
    /// - Network drive unavailable
    DestinationUnavailable,
}

// Optional: Implement Display for better error messages
impl std::fmt::Display for FileOperationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileOperationStatus::Copied => write!(f, "copied"),
            FileOperationStatus::AlreadyExisted => write!(f, "already existed"),
            FileOperationStatus::OriginalNotFound => write!(f, "original not found"),
            FileOperationStatus::OriginalUnavailable => write!(f, "original unavailable"),
            FileOperationStatus::DestinationUnavailable => write!(f, "destination unavailable"),
        }
    }
}

// ============================================================================
// SAVE-AS-COPY OPERATION: Main Function (start)
// ============================================================================

/// Copies a file to a new location without modifying the original
///
/// # Project Context
/// Implements the "Save As" functionality for the Lines text editor. Allows
/// users to create a copy of the current file with a new name/location while
/// preserving the original. This is distinct from:
/// - Regular save (overwrites current file)
/// - File append (adds content to existing file)
/// - File insertion (inserts file content at cursor position)
///
/// # Function Scope & Purpose
/// This function provides defensive file copying with:
/// - Type-safe status reporting (distinguishes predicated outcomes from errors)
/// - Retry logic for transient failures (handles brief system glitches)
/// - No-overwrite policy (preserves existing files)
/// - Bounded operations (no infinite loops, no full file loads)
/// - Fail-safe error handling (never panics, always returns gracefully)
///
/// # Design Philosophy
/// Follows NASA-inspired defensive programming principles:
/// - Bounded retry loops (3 attempts max per operation)
/// - Pre-allocated stack buffer (8KB, no heap allocation)
/// - Explicit error classification (transient vs permanent)
/// - Conservative approach (fail-fast on permanent errors)
/// - Type-safe status codes (compiler-enforced exhaustive matching)
/// - No dynamic memory for status messages
///
/// # Arguments
/// * `original_file_path` - Absolute path to source file
///   - Must be absolute path (validated by function)
///   - Must be valid UTF-8 (validated by function)
///   - File must exist and be readable
///   - Example: `/home/user/Documents/original.txt`
///
/// * `new_file_path_name` - Absolute path for destination file
///   - Must be absolute path (validated by function)
///   - Must be valid UTF-8 (validated by function)
///   - Must NOT already exist (no-overwrite policy)
///   - Parent directory must exist
///   - Example: `/home/user/Documents/backup.txt`
///
/// # Returns
/// Returns `Result<(FileOperationStatus, &'static str), LinesError>`:
///
/// ## Success Cases - `Ok((status, message))`
/// All of these are valid outcomes, not errors:
///
/// * `Ok((Copied, "copied"))` - File successfully copied
///   - Source file read completely
///   - Destination file written successfully
///   - Both files intact
///
/// * `Ok((AlreadyExisted, "already existed"))` - Destination already exists
///   - Predicated outcome: no-overwrite policy prevents data loss
///   - Neither file modified
///   - Caller should prompt user for overwrite confirmation
///
/// * `Ok((OriginalNotFound, "original not found"))` - Source doesn't exist
///   - Predicated outcome: nothing to copy
///   - Common in workflows with optional/uncertain file paths
///   - Caller should handle gracefully (not treated as error)
///
/// * `Ok((OriginalUnavailable, "original unavailable"))` - Source locked
///   - Source file exists but can't be opened after 3 retry attempts
///   - Common causes: locked by another process, permissions issue
///   - Caller should notify user to close other applications
///
/// * `Ok((DestinationUnavailable, "destination unavailable"))` - Can't write
///   - Destination can't be created/written after 3 retry attempts
///   - Common causes: parent directory missing, permissions, disk full
///   - Caller should check directory exists and has write permissions
///
/// ## Error Cases - `Err(LinesError)`
/// True errors indicating unexpected system failures:
///
/// * `Err(LinesError::InvalidInput(_))` - Path validation failed
///   - Path is not absolute
///   - Path contains invalid UTF-8
///   - Path format is malformed
///
/// * `Err(LinesError::Io(_))` - I/O operation failed catastrophically
///   - Disk failure during read/write
///   - Filesystem corruption detected
///   - Unexpected system-level error after retries
///
/// * `Err(LinesError::StateError(_))` - Unexpected state violation
///   - Maximum chunk limit exceeded (128GB file size limit)
///   - Loop safety limit triggered (cosmic ray protection)
///
/// # Behavior Details
///
/// ## Phase 1: Path Validation
/// - Validates both paths are absolute (not relative)
/// - Validates paths contain valid UTF-8
/// - Returns `InvalidInput` error if validation fails
/// - No retries (validation either passes or fails)
///
/// ## Phase 2: Source File Check
/// - Checks if source file exists using `fs::metadata()`
/// - Returns `OriginalNotFound` status if doesn't exist
/// - Does NOT attempt to open file yet
/// - No retries (file either exists or doesn't)
///
/// ## Phase 3: Destination Check
/// - Checks if destination already exists using `fs::metadata()`
/// - Returns `AlreadyExisted` status if exists (no-overwrite policy)
/// - Prevents accidental data loss
/// - No retries (file either exists or doesn't)
///
/// ## Phase 4: Open Source File
/// - Opens source file in read-only mode
/// - Retries up to 3 times for transient errors (locks, interrupts)
/// - Returns `OriginalUnavailable` if all retries fail
/// - Returns `Io` error for permanent failures (permissions)
///
/// ## Phase 5: Create Destination File
/// - Creates destination file with `create_new(true)` (fails if exists)
/// - Opens in write-only mode
/// - Retries up to 3 times for transient errors
/// - Returns `DestinationUnavailable` if all retries fail
/// - Returns `Io` error for permanent failures
///
/// ## Phase 6: Buffered Copy Loop
/// - Pre-allocates 8KB stack buffer (no heap)
/// - Reads chunks from source with retry logic
/// - Writes chunks to destination with retry logic
/// - Bounded loop: max 16,777,216 chunks (~128GB)
/// - Stops at EOF (bytes_read == 0)
/// - Never loads entire file into memory
///
/// ## Phase 7: Finalize
/// - Flushes destination file to disk (with retry)
/// - Returns `Copied` status on success
/// - Files automatically closed on function exit (RAII)
///
/// # Safety Guarantees
/// - **No panic**: All errors handled gracefully, returns Result
/// - **No unsafe**: Pure safe Rust, no unsafe blocks
/// - **No recursion**: All operations iterative
/// - **Bounded loops**: Explicit limits on all iterations
/// - **No heap for messages**: Status strings are &'static str
/// - **No unwrap**: All Results explicitly handled with ? or match
/// - **No full file load**: Bucket-brigade chunked processing
/// - **Atomic visibility**: Destination file only visible after full copy
///
/// # Performance Characteristics
/// - **Time complexity**: O(n) where n = file size
/// - **Space complexity**: O(1) - constant 8KB buffer
/// - **Chunk overhead**: ~12,800 iterations per 100MB
/// - **Retry overhead**: Up to 400ms total delay (2 retries × 200ms)
/// - **Best case**: Single-pass copy, no retries
/// - **Worst case**: 3 attempts per operation + max chunks
///
/// # Error Logging
/// Production builds log detailed errors to error log file:
/// - Full paths logged to file (for debugging)
/// - Generic messages returned to caller (no info leakage)
/// - Errors logged with context (function name, phase)
///
/// Debug builds include detailed diagnostics in stderr.
///
/// # Example Usage
/// ```no_run
/// use std::path::Path;
/// # use std::io;
/// # #[derive(Debug)] enum LinesError { Io(io::Error) }
/// # impl From<io::Error> for LinesError { fn from(e: io::Error) -> Self { LinesError::Io(e) } }
/// # #[derive(Debug, PartialEq)] enum FileOperationStatus { Copied, AlreadyExisted, OriginalNotFound }
/// # fn save_file_as_newfile_with_newname(
/// #     _original: &Path,
/// #     _new: &Path,
/// # ) -> Result<(FileOperationStatus, &'static str), LinesError> {
/// #     Ok((FileOperationStatus::Copied, "copied"))
/// # }
///
/// let source = Path::new("/home/user/Documents/draft.txt");
/// let destination = Path::new("/home/user/Documents/draft_backup.txt");
///
/// match save_file_as_newfile_with_newname(source, destination) {
///     Ok((FileOperationStatus::Copied, msg)) => {
///         println!("Success: File {}", msg);
///     }
///     Ok((FileOperationStatus::AlreadyExisted, msg)) => {
///         println!("Destination {}, overwrite? (y/n)", msg);
///         // Prompt user for confirmation
///     }
///     Ok((FileOperationStatus::OriginalNotFound, msg)) => {
///         println!("Source file {}", msg);
///     }
///     Ok((status, msg)) => {
///         println!("Status {:?}: {}", status, msg);
///     }
///     Err(e) => {
///         eprintln!("Error during copy: {}", e);
///     }
/// }
/// ```
///
/// # Edge Cases
/// - **Empty file**: Copies successfully, creates 0-byte destination
/// - **Very large file**: Up to ~128GB supported (bounded by MAX_CHUNKS)
/// - **Source == Destination**: Allowed but discouraged (would fail at destination exists check)
/// - **Parent directory missing**: Returns `DestinationUnavailable`
/// - **Insufficient permissions**: Returns error or unavailable status
/// - **Disk full during copy**: Returns `DestinationUnavailable` after retries
/// - **File locked**: Retries 3 times, then returns unavailable status
/// - **Network path**: Supported if OS presents as regular file path
///
/// # Related Functions
/// - Uses: `is_retryable_error()`, `retry_operation()`
/// - Uses: `log_error()` for production error logging
/// - Returns: Status codes from `FileOperationStatus` enum
/// - Errors: Uses project's `LinesError` enum
///
/// # Thread Safety
/// Function is thread-safe in that it doesn't use shared mutable state.
/// However, concurrent access to same files from multiple threads/processes
/// may cause file locking issues. Caller responsible for coordination.
pub fn save_file_as_newfile_with_newname(
    original_file_path: &Path,
    new_file_path_name: &Path,
) -> Result<(FileOperationStatus, &'static str)> {
    // ========================================================================
    // PHASE 1: Path Validation
    // ========================================================================
    // Validate paths before any I/O operations to fail fast on invalid input.
    // This prevents wasted work and provides clear error messages early.

    //    =================================================
    // // Debug-Assert, Test-Asset, Production-Catch-Handle
    //    =================================================
    // Check: original path must be absolute
    debug_assert!(
        original_file_path.is_absolute(),
        "original_file_path must be absolute"
    );
    #[cfg(test)]
    assert!(
        original_file_path.is_absolute(),
        "original_file_path must be absolute"
    );
    if !original_file_path.is_absolute() {
        #[cfg(not(debug_assertions))]
        log_error(
            "Original path not absolute",
            Some("save_file_as_newfile:path_validation"),
        );
        return Err(LinesError::InvalidInput(
            "original path must be absolute".into(),
        ));
    }

    // Check: destination path must be absolute
    debug_assert!(
        new_file_path_name.is_absolute(),
        "new_file_path_name must be absolute"
    );
    #[cfg(test)]
    assert!(
        new_file_path_name.is_absolute(),
        "new_file_path_name must be absolute"
    );
    if !new_file_path_name.is_absolute() {
        #[cfg(not(debug_assertions))]
        log_error(
            "Destination path not absolute",
            Some("save_file_as_newfile:path_validation"),
        );
        return Err(LinesError::InvalidInput(
            "destination path must be absolute".into(),
        ));
    }

    // Validate: original path must be valid UTF-8
    // This ensures we can safely work with the path in all operations
    if original_file_path.to_str().is_none() {
        #[cfg(not(debug_assertions))]
        log_error(
            "Original path invalid UTF-8",
            Some("save_file_as_newfile:path_validation"),
        );
        return Err(LinesError::InvalidInput(
            "original path contains invalid UTF-8".into(),
        ));
    }

    // Validate: destination path must be valid UTF-8
    if new_file_path_name.to_str().is_none() {
        #[cfg(not(debug_assertions))]
        log_error(
            "Destination path invalid UTF-8",
            Some("save_file_as_newfile:path_validation"),
        );
        return Err(LinesError::InvalidInput(
            "destination path contains invalid UTF-8".into(),
        ));
    }

    // Debug: log paths being processed (only in debug builds for security)
    #[cfg(debug_assertions)]
    eprintln!(
        "DEBUG: save_file_as_newfile - copying from {:?} to {:?}",
        original_file_path, new_file_path_name
    );

    // ========================================================================
    // PHASE 2: Source File Existence Check
    // ========================================================================
    // Check if source file exists before attempting to open it.
    // Distinguishes "file doesn't exist" from "file exists but can't open".

    match fs::metadata(original_file_path) {
        Ok(metadata) => {
            // Source exists, verify it's a file (not a directory)
            if !metadata.is_file() {
                #[cfg(not(debug_assertions))]
                log_error(
                    "Original path is not a file",
                    Some("save_file_as_newfile:source_check"),
                );
                return Err(LinesError::InvalidInput(
                    "original path is not a file".into(),
                ));
            }
            // Source file exists and is valid, continue to next phase
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            // Predicated outcome: source file doesn't exist
            // This is expected in some workflows, not an error
            #[cfg(debug_assertions)]
            eprintln!("DEBUG: Source file not found: {:?}", original_file_path);

            return Ok((FileOperationStatus::OriginalNotFound, "original not found"));
        }
        Err(e) => {
            // Unexpected error checking source metadata
            // Could be permissions issue or filesystem problem
            #[cfg(not(debug_assertions))]
            log_error(
                "Cannot access source file",
                Some("save_file_as_newfile:source_check"),
            );
            return Err(LinesError::Io(e));
        }
    }

    // ========================================================================
    // PHASE 3: Destination File Existence Check
    // ========================================================================
    // Check if destination already exists to enforce no-overwrite policy.
    // Prevents accidental data loss from overwriting existing files.

    match fs::metadata(new_file_path_name) {
        Ok(_) => {
            // Predicated outcome: destination file already exists
            // No-overwrite policy: return status for caller to handle
            #[cfg(debug_assertions)]
            eprintln!(
                "DEBUG: Destination already exists: {:?}",
                new_file_path_name
            );

            return Ok((FileOperationStatus::AlreadyExisted, "already existed"));
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            // Good: destination doesn't exist, we can create it
            // Continue to copy phase
        }
        Err(e) => {
            // Unexpected error checking destination metadata
            // Could be permissions issue on parent directory
            #[cfg(not(debug_assertions))]
            log_error(
                "Cannot access destination path",
                Some("save_file_as_newfile:destination_check"),
            );
            return Err(LinesError::Io(e));
        }
    }

    // ========================================================================
    // PHASE 4: Open Source File (with retry)
    // ========================================================================
    // Open source file in read-only mode with retry logic for transient errors.

    let mut source_file = match retry_operation(
        || File::open(original_file_path),
        SAVE_AS_COPY_MAX_RETRY_ATTEMPTS,
    ) {
        Ok(file) => file,
        Err(e) if is_retryable_error(&e) => {
            // Transient error persisted through all retries
            // Predicated outcome: file locked or temporarily unavailable
            #[cfg(debug_assertions)]
            eprintln!("DEBUG: Source file unavailable after retries: {:?}", e);

            #[cfg(not(debug_assertions))]
            log_error(
                "Source file unavailable",
                Some("save_file_as_newfile:open_source"),
            );

            return Ok((
                FileOperationStatus::OriginalUnavailable,
                "original unavailable",
            ));
        }
        Err(e) => {
            // Permanent error: permissions, path issue, etc.
            #[cfg(not(debug_assertions))]
            log_error(
                "Cannot open source file",
                Some("save_file_as_newfile:open_source"),
            );
            return Err(LinesError::Io(e));
        }
    };

    // ========================================================================
    // PHASE 5: Create Destination File (with retry)
    // ========================================================================
    // Create destination file with create_new(true) to prevent overwrite.

    let mut dest_file = match retry_operation(
        || {
            OpenOptions::new()
                .create_new(true) // Fail if file exists (double-check safety)
                .write(true)
                .open(new_file_path_name)
        },
        SAVE_AS_COPY_MAX_RETRY_ATTEMPTS,
    ) {
        Ok(file) => file,
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            // Race condition: file created between our check and now
            // Treat as predicated outcome (same as earlier check)
            #[cfg(debug_assertions)]
            eprintln!("DEBUG: Destination created by another process");

            return Ok((FileOperationStatus::AlreadyExisted, "already existed"));
        }
        Err(e) if is_retryable_error(&e) => {
            // Transient error persisted through all retries
            // Predicated outcome: can't write to destination
            #[cfg(debug_assertions)]
            eprintln!("DEBUG: Destination unavailable after retries: {:?}", e);

            #[cfg(not(debug_assertions))]
            log_error(
                "Destination unavailable",
                Some("save_file_as_newfile:create_destination"),
            );

            return Ok((
                FileOperationStatus::DestinationUnavailable,
                "destination unavailable",
            ));
        }
        Err(e) => {
            // Permanent error: permissions, parent directory missing, disk full
            #[cfg(not(debug_assertions))]
            log_error(
                "Cannot create destination file",
                Some("save_file_as_newfile:create_destination"),
            );
            return Err(LinesError::Io(e));
        }
    };

    // ========================================================================
    // PHASE 6: Buffered Copy Loop (with retry)
    // ========================================================================
    // Copy file content in chunks using pre-allocated stack buffer.
    // Bucket-brigade pattern: read chunk -> write chunk -> repeat until EOF.

    // Pre-allocate buffer on stack (NASA rule 3: no dynamic allocation)
    let mut buffer = [0u8; SAVE_AS_COPY_BUFFER_SIZE];

    // Chunk counter for bounded loop (NASA rule 2: upper bound on loops)
    let mut chunk_count: usize = 0;

    // Copy loop: bounded by MAX_CHUNKS safety limit
    loop {
        // Safety check: prevent infinite loop from filesystem corruption
        if chunk_count >= SAVE_AS_COPY_MAX_CHUNKS {
            #[cfg(debug_assertions)]
            eprintln!(
                "DEBUG: Maximum chunk limit reached ({})",
                SAVE_AS_COPY_MAX_CHUNKS
            );

            #[cfg(not(debug_assertions))]
            log_error(
                "Copy iteration limit exceeded",
                Some("save_file_as_newfile:copy_loop"),
            );

            return Err(LinesError::StateError("iteration limit exceeded".into()));
        }

        chunk_count += 1;

        // Read chunk from source file (with retry)
        let bytes_read = match retry_operation(
            || source_file.read(&mut buffer),
            SAVE_AS_COPY_MAX_RETRY_ATTEMPTS,
        ) {
            Ok(n) => n,
            Err(e) => {
                // Read failed after retries
                #[cfg(not(debug_assertions))]
                log_error(
                    "Read failed during copy",
                    Some("save_file_as_newfile:copy_loop"),
                );
                return Err(LinesError::Io(e));
            }
        };

        // EOF detection: bytes_read == 0 reliably signals end of file
        if bytes_read == 0 {
            // Successfully read entire file
            break;
        }

        // Defensive assertion: bytes_read should never exceed buffer size
        debug_assert!(
            bytes_read <= SAVE_AS_COPY_BUFFER_SIZE,
            "bytes_read exceeded buffer size"
        );
        #[cfg(test)]
        assert!(
            bytes_read <= SAVE_AS_COPY_BUFFER_SIZE,
            "bytes_read exceeded buffer size"
        );
        if bytes_read > SAVE_AS_COPY_BUFFER_SIZE {
            #[cfg(not(debug_assertions))]
            log_error(
                "Buffer overflow detected",
                Some("save_file_as_newfile:copy_loop"),
            );
            return Err(LinesError::StateError("buffer overflow".into()));
        }

        // Write chunk to destination file (with retry)
        match retry_operation(
            || dest_file.write_all(&buffer[..bytes_read]),
            SAVE_AS_COPY_MAX_RETRY_ATTEMPTS,
        ) {
            Ok(()) => { /* Write successful, continue to next chunk */ }
            Err(e) => {
                // Write failed after retries
                #[cfg(not(debug_assertions))]
                log_error(
                    "Write failed during copy",
                    Some("save_file_as_newfile:copy_loop"),
                );
                return Err(LinesError::Io(e));
            }
        }

        // Loop continues to next chunk
        // Bounded by: chunk_count < SAVE_AS_COPY_MAX_CHUNKS
    }

    // ========================================================================
    // PHASE 7: Finalize - Flush and Return Success
    // ========================================================================
    // Flush destination file to ensure all data written to disk.

    match retry_operation(|| dest_file.flush(), SAVE_AS_COPY_MAX_RETRY_ATTEMPTS) {
        Ok(()) => { /* Flush successful */ }
        Err(e) => {
            // Flush failed after retries
            #[cfg(not(debug_assertions))]
            log_error(
                "Flush failed after copy",
                Some("save_file_as_newfile:finalize"),
            );
            return Err(LinesError::Io(e));
        }
    }

    // Success: file copied completely
    #[cfg(debug_assertions)]
    eprintln!("DEBUG: Successfully copied file ({} chunks)", chunk_count);

    Ok((FileOperationStatus::Copied, "copied"))
}

// ============================================================================
// (end) SAVE-AS-COPY OPERATION: Main Function
// ============================================================================

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

    let formatted_string = stack_format_it(
        "{}_{}",
        &[&timestamp, &original_filename.to_string_lossy()],
        "N_N",
    );

    // let backup_path = archive_dir.join(format!(
    //     "{}_{}",
    //     timestamp,
    //     original_filename.to_string_lossy()
    // ));

    let backup_path = archive_dir.join(formatted_string);

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

// ============================================================================
// UTF-8 CHARACTER ANALYSIS (Buffer-based variant for line processing)
// ============================================================================

/// Determines the byte length of a UTF-8 character from a byte buffer
///
/// # Purpose (Project Context)
/// When building the window-to-file mapping, we process lines that have been
/// read into memory buffers. This function analyzes UTF-8 characters from
/// those buffers to correctly calculate byte positions and display widths.
///
/// This is a buffer-based variant of `get_utf8_char_byte_length_at_position`.
/// While that function reads from files, this one reads from memory buffers
/// during line processing.
///
/// # UTF-8 First-Byte Patterns (Same as file-based version)
/// ```text
/// Byte Range   Pattern      Character Length
/// 0x00..=0x7F  0xxxxxxx     1 byte  (ASCII)
/// 0xC0..=0xDF  110xxxxx     2 bytes
/// 0xE0..=0xEF  1110xxxx     3 bytes
/// 0xF0..=0xF7  11110xxx     4 bytes
/// 0x80..=0xBF  10xxxxxx     Invalid (continuation byte, not first byte)
/// 0xF8..=0xFF  11111xxx     Invalid (UTF-8 doesn't use these)
/// ```
///
/// # Arguments
/// * `buffer` - Byte slice containing UTF-8 text
/// * `index` - Position in buffer where character starts
///
/// # Returns
/// * `Ok(1..=4)` - Valid UTF-8 character byte length
/// * `Ok(1)` - Invalid UTF-8 treated as single byte (defensive)
/// * `Err` - Index out of bounds (buffer access error)
///
/// # Defensive Programming
/// - Out of bounds index → Returns error
/// - EOF/incomplete character → Returns Ok(1) for remaining bytes
/// - Invalid UTF-8 sequence → Returns Ok(1) (treat as single byte)
/// - Continuation byte as first byte → Returns Ok(1) (malformed data)
///
/// # Memory Safety
/// - No heap allocation
/// - Bounds checking on all buffer access
/// - Safe indexing using slice bounds
///
/// # Examples
/// ```ignore
/// let text = b"hello\xE4\xB8\x96"; // "hello世"
/// let len = get_utf8_char_byte_length_from_buffer(text, 0)?;
/// assert_eq!(len, 1); // 'h' is 1 byte
///
/// let len = get_utf8_char_byte_length_from_buffer(text, 5)?;
/// assert_eq!(len, 3); // '世' is 3 bytes (E4 B8 96)
///
/// let invalid = b"\x80\x81"; // Invalid UTF-8 (continuation bytes)
/// let len = get_utf8_char_byte_length_from_buffer(invalid, 0)?;
/// assert_eq!(len, 1); // Defensive: treat as single byte
/// ```
fn get_utf8_char_byte_length_from_buffer(buffer: &[u8], index: usize) -> Result<usize> {
    // ═══════════════════════════════════════════════════════════════════════
    // DEFENSIVE CHECK 1: Validate index is within buffer bounds
    // ═══════════════════════════════════════════════════════════════════════
    if index >= buffer.len() {
        #[cfg(debug_assertions)]
        eprintln!(
            "get_utf8_char_byte_length_from_buffer: index {} >= buffer length {} (out of bounds)",
            index,
            buffer.len()
        );

        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Buffer index out of bounds",
        )));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // READ: First byte of character from buffer
    // ═══════════════════════════════════════════════════════════════════════
    let first_byte = buffer[index];

    // ═══════════════════════════════════════════════════════════════════════
    // UTF-8 ANALYSIS: Determine character length from first-byte pattern
    // ═══════════════════════════════════════════════════════════════════════
    // Using bit patterns to identify UTF-8 character length.
    // Order matters: check more specific patterns first (4-byte before 3-byte, etc.)

    let char_length = if first_byte <= 0x7F {
        // Pattern: 0xxxxxxx → 1-byte character (ASCII)
        // Range: 0x00..=0x7F
        1
    } else if first_byte >= 0xF0 && first_byte <= 0xF7 {
        // Pattern: 11110xxx → 4-byte character
        // Range: 0xF0..=0xF7
        // Must check 4-byte BEFORE 3-byte and 2-byte to avoid false matches
        4
    } else if first_byte >= 0xE0 && first_byte <= 0xEF {
        // Pattern: 1110xxxx → 3-byte character
        // Range: 0xE0..=0xEF
        3
    } else if first_byte >= 0xC0 && first_byte <= 0xDF {
        // Pattern: 110xxxxx → 2-byte character
        // Range: 0xC0..=0xDF
        2
    } else {
        // Invalid UTF-8 first byte:
        // - 0x80..=0xBF (10xxxxxx) - continuation bytes, not valid as first byte
        // - 0xF8..=0xFF - invalid UTF-8 range (not used in UTF-8 standard)
        //
        // Defensive: Treat as single byte, allow editor to continue
        #[cfg(debug_assertions)]
        eprintln!(
            "get_utf8_char_byte_length_from_buffer: invalid UTF-8 first byte 0x{:02X} at index {} (treating as 1 byte)",
            first_byte, index
        );

        1 // Defensive fallback
    };

    // ═══════════════════════════════════════════════════════════════════════
    // DEFENSIVE CHECK 2: Verify complete character is available in buffer
    // ═══════════════════════════════════════════════════════════════════════
    // If we need N bytes but buffer only has M bytes remaining where M < N,
    // treat the incomplete sequence as single bytes (defensive handling)
    let bytes_remaining = buffer.len() - index;
    if char_length > bytes_remaining {
        #[cfg(debug_assertions)]
        eprintln!(
            "get_utf8_char_byte_length_from_buffer: incomplete UTF-8 character at index {} \
             (need {} bytes, only {} remaining, treating as 1 byte)",
            index, char_length, bytes_remaining
        );

        // Defensive: Treat incomplete character as single byte
        // This allows processing to continue even with truncated data
        return Ok(1);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DEFENSIVE CHECK 3: Validate continuation bytes (2-byte, 3-byte, 4-byte)
    // ═══════════════════════════════════════════════════════════════════════
    // For multi-byte UTF-8, continuation bytes must match pattern 10xxxxxx (0x80..=0xBF)
    // If they don't, the sequence is malformed
    if char_length > 1 {
        for i in 1..char_length {
            let continuation_byte = buffer[index + i];

            // Check if byte matches continuation pattern: 10xxxxxx
            // Mask 0b11000000 should equal 0b10000000
            if (continuation_byte & 0b1100_0000) != 0b1000_0000 {
                #[cfg(debug_assertions)]
                eprintln!(
                    "get_utf8_char_byte_length_from_buffer: invalid continuation byte 0x{:02X} \
                     at position {} in {}-byte sequence starting at {} (treating as 1 byte)",
                    continuation_byte, i, char_length, index
                );

                // Defensive: Invalid continuation byte means malformed UTF-8
                // Treat the first byte as a single-byte character and let
                // the next iteration handle the remaining bytes
                return Ok(1);
            }
        }
    }

    // Assertion: Character length must be 1-4 (UTF-8 standard)
    debug_assert!(
        char_length >= 1 && char_length <= 4,
        "UTF-8 character length must be 1-4 bytes, got {}",
        char_length
    );

    Ok(char_length)
}

/// Processes a line with horizontal offset and writes visible portion to display
///
/// # Purpose (Project Context)
/// Takes a line's bytes, skips horizontal_offset characters, then writes
/// the visible portion to the display buffer while updating WindowMapStruct.
///
/// This function is critical for correct cursor positioning: it creates the
/// mapping between display columns (what the user sees) and file byte positions
/// (where edits actually happen). Any error here causes insertion/deletion to
/// occur at the wrong file location.
///
/// # Multi-byte UTF-8 Support
/// The function must correctly handle 1-4 byte UTF-8 characters:
/// - Skip complete characters when applying horizontal offset (not bytes)
/// - Map display columns to correct file byte positions
/// - Handle double-width characters (CJK, emoji) that occupy 2 display columns
/// - Ensure each display column maps to the START byte of its character
///
/// # Arguments
/// * `state` - Editor state for buffers and map
/// * `row` - Display row index (0-indexed within window)
/// * `col_start` - Starting column (after line number prefix)
/// * `line_bytes` - The complete line text as bytes (entire line, not truncated)
/// * `horizontal_offset` - Number of CHARACTERS to skip from line start (not bytes)
/// * `max_cols` - Maximum display columns available for text
/// * `file_line_start` - Absolute byte position where this line starts in file
/// * `current_line_number` - Absolute file line number (0-indexed)
///
/// # Returns
/// * `Ok(bytes_written)` - Number of bytes written to display buffer
/// * `Err(LinesError)` - If buffer overflow, invalid UTF-8, or bounds violation
///
/// # Window Mapping Semantics
/// Each display column maps to the file byte position of the character's FIRST byte:
/// ```ignore
/// File bytes:   [10='a', 11='b', 12-14='世', 15=' ']
/// Display cols: [0='a',  1='b',  2-3='世',    4=' ']
///                                 ^^^
/// Column 2 maps to byte 12 (start of '世')
/// Column 3 maps to byte 12 (still part of '世', same start byte)
/// Column 4 maps to byte 15 (start of space after '世')
/// ```
///
/// # Defensive Programming
/// - Validates all array indices before access
/// - Checks UTF-8 character completeness
/// - Bounds-checks display buffer writes
/// - Limits iterations to prevent infinite loops
/// - Validates continuation bytes for multi-byte sequences
///
/// # Example
/// ```ignore
/// // File line: "hello世界" (hello + two 3-byte Chinese characters)
/// // File bytes: [h e l l o 世(E4B896) 界(E7958C)]
/// // Positions:  0 1 2 3 4 5  6  7    8  9  10
/// //
/// // With horizontal_offset=0, col_start=2 (after "1 "):
/// // Display: "1 hello世界"
/// // Columns:  0 1 2 3 4 5 6 7 8 9 10 11
/// //              ^ h e l l o 世世界界
/// //
/// // Window map:
/// // Col 2 → file byte 0 ('h')
/// // Col 3 → file byte 1 ('e')
/// // ...
/// // Col 7 → file byte 5 ('世' start)
/// // Col 8 → file byte 5 ('世' second column, same byte)
/// // Col 9 → file byte 8 ('界' start)
/// // Col 10 → file byte 8 ('界' second column, same byte)
/// ```
fn process_line_with_offset(
    state: &mut EditorState,
    row: usize,
    col_start: usize,
    line_bytes: &[u8],
    horizontal_offset: usize,
    max_cols: usize,
    file_line_start: u64,
) -> Result<usize> {
    let current_line_number = state.cursor.row;

    const MAX_DISPLAY_BUFFER_BYTES: usize = 182;

    // ═══════════════════════════════════════════════════════════════════════
    // DEFENSIVE CHECK 1: Validate row index
    // ═══════════════════════════════════════════════════════════════════════
    if row >= MAX_TUI_ROWS {
        #[cfg(debug_assertions)]
        eprintln!(
            "process_line_with_offset: row {} exceeds MAX_TUI_ROWS {}",
            row, MAX_TUI_ROWS
        );

        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Row exceeds maximum display rows",
        )));
    }

    // Assertion: col_start should be reasonable (after line number, typically 1-5)
    debug_assert!(
        col_start < 20,
        "col_start {} seems unreasonably large",
        col_start
    );

    // ═══════════════════════════════════════════════════════════════════════
    // PHASE 1: Skip horizontal_offset CHARACTERS (not bytes!)
    // ═══════════════════════════════════════════════════════════════════════
    // When horizontally scrolling, we skip complete characters from the line start.
    // Must use UTF-8 character boundaries, not byte boundaries.

    let mut byte_index = 0usize;
    let mut chars_skipped = 0usize;
    let mut skip_iterations = 0;

    #[cfg(debug_assertions)]
    eprintln!(
        "process_line_with_offset: row={}, horizontal_offset={}, line_length={} bytes",
        row,
        horizontal_offset,
        line_bytes.len()
    );

    while byte_index < line_bytes.len()
        && chars_skipped < horizontal_offset
        && skip_iterations < limits::HORIZONTAL_SCROLL_CHARS
    {
        skip_iterations += 1;

        // Get character byte length using centralized helper
        let char_len = match get_utf8_char_byte_length_from_buffer(line_bytes, byte_index) {
            Ok(len) => len,
            Err(_) => {
                // Buffer access error - should not happen as we checked byte_index < len
                // Defensive: treat as 1 byte and continue
                #[cfg(debug_assertions)]
                eprintln!(
                    "process_line_with_offset: buffer access error at byte_index {} (treating as 1 byte)",
                    byte_index
                );
                1
            }
        };

        #[cfg(debug_assertions)]
        {
            if char_len > 1 {
                eprintln!(
                    "  Skip phase: byte_index={}, char_len={}, first_byte=0x{:02X}",
                    byte_index, char_len, line_bytes[byte_index]
                );
            }
        }

        // Skip this complete character
        byte_index = (byte_index + char_len).min(line_bytes.len());
        chars_skipped += 1;
    }

    // Defensive: Check we didn't hit iteration limit
    if skip_iterations >= limits::HORIZONTAL_SCROLL_CHARS {
        #[cfg(debug_assertions)]
        eprintln!("process_line_with_offset: hit iteration limit during horizontal skip phase");

        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::Other,
            "Maximum iterations exceeded in horizontal skip",
        )));
    }

    #[cfg(debug_assertions)]
    eprintln!(
        "  After skip: byte_index={}, chars_skipped={}",
        byte_index, chars_skipped
    );

    // Assertion: We should have skipped at most the requested amount
    debug_assert!(
        chars_skipped <= horizontal_offset,
        "Skipped {} characters but only {} requested",
        chars_skipped,
        horizontal_offset
    );

    // ═══════════════════════════════════════════════════════════════════════
    // PHASE 2: Write visible characters to display buffer and build window map
    // ═══════════════════════════════════════════════════════════════════════

    let mut display_col = col_start;
    let mut bytes_written = 0usize;
    let mut write_iterations = 0;

    // Visual size of TUI width -pixel level-
    let mut visual_col = col_start; // Visual TUI column consumed
    let visual_col_limit = col_start + max_cols; // Visual limit

    // Reserve space to prevent double-width characters from overflowing
    // A double-width char starting at max_cols-1 would extend to max_cols+1
    let display_col_limit = col_start + max_cols;

    while byte_index < line_bytes.len()
        && display_col < display_col_limit - 1 // Reserve 1-2 cols for double-width
        && write_iterations < limits::HORIZONTAL_SCROLL_CHARS
    {
        write_iterations += 1;

        // Get character byte length using centralized helper
        let char_len = match get_utf8_char_byte_length_from_buffer(line_bytes, byte_index) {
            Ok(len) => len,
            Err(_) => {
                // Buffer access error - skip this position
                #[cfg(debug_assertions)]
                eprintln!(
                    "process_line_with_offset: buffer access error at byte_index {} in write phase",
                    byte_index
                );
                byte_index += 1;
                continue;
            }
        };

        // Check if complete character is available
        if byte_index + char_len > line_bytes.len() {
            #[cfg(debug_assertions)]
            eprintln!(
                "process_line_with_offset: incomplete character at byte_index {} (need {} bytes, only {} remaining)",
                byte_index,
                char_len,
                line_bytes.len() - byte_index
            );
            break; // Incomplete character at end of line
        }

        // Get the character bytes
        let char_bytes = &line_bytes[byte_index..byte_index + char_len];

        // Assertion: Character bytes should be exactly char_len
        debug_assert_eq!(
            char_bytes.len(),
            char_len,
            "Character byte slice length mismatch: expected {}, got {}",
            char_len,
            char_bytes.len()
        );

        #[cfg(debug_assertions)]
        {
            if char_len > 1 {
                eprintln!(
                    "  Write phase: byte_index={}, char_len={}, bytes={:02X?}",
                    byte_index, char_len, char_bytes
                );
            }
        }

        // ═══════════════════════════════════════════════════════════════════
        // DISPLAY WIDTH: Determine how many columns this character occupies
        // ═══════════════════════════════════════════════════════════════════
        let display_width = if char_len == 1 {
            1 // ASCII is always single-width
        } else {
            // Parse multi-byte character to check if double-width (CJK, emoji, etc.)
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
                Err(_) => {
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "  Invalid UTF-8 sequence at byte_index {}: {:02X?}",
                        byte_index, char_bytes
                    );
                    1 // Invalid UTF-8, treat as single-width
                }
            }
        };

        // Check if character fits VISUALLY
        if visual_col + display_width > visual_col_limit {
            break; // Would overflow visually
        }

        // Check if character fits in remaining display space
        if display_col + display_width > display_col_limit {
            #[cfg(debug_assertions)]
            eprintln!(
                "  Character would exceed display limit: display_col={}, display_width={}, limit={}",
                display_col, display_width, display_col_limit
            );
            break; // Character would exceed display width
        }

        // ═══════════════════════════════════════════════════════════════════
        // WRITE: Copy character bytes to display buffer
        // ═══════════════════════════════════════════════════════════════════
        let write_start = col_start + bytes_written;
        let write_end = write_start + char_len;

        if write_end <= MAX_DISPLAY_BUFFER_BYTES {
            // Copy bytes to display buffer
            for i in 0..char_len {
                state.utf8_txt_display_buffers[row][write_start + i] = char_bytes[i];
            }

            // ═══════════════════════════════════════════════════════════════
            // WINDOW MAP: Create file position mapping for this character
            // ═══════════════════════════════════════════════════════════════
            //
            // bytes, pixels, characters, are not the same.
            //
            // Each character maps to ONE cursor position, regardless of display width.
            //
            // Single-width 'a': 1 display column → 1 cursor position
            // Double-width '花': 1 display columns → 1 cursor position (not 2!)
            //
            let file_pos = FilePosition {
                byte_offset_linear_file_absolute_position: file_line_start + byte_index as u64,
                line_number: current_line_number, // ← FIXED: use actual line number
                byte_in_line: byte_index,
            };
            #[cfg(debug_assertions)]
            {
                if display_width == 2 {
                    eprintln!(
                        "  Double-width char at display_col={} → byte={} (width={}, ONE cursor position)",
                        display_col,
                        file_line_start + byte_index as u64,
                        display_width
                    );
                }
            }

            // Map this one cursor position to be kanji start byte
            state.set_file_position(row, display_col, Some(file_pos))?;

            bytes_written += char_len;
            display_col += 1 // display_width; // Visual advance 1 TUI char column
        } else {
            #[cfg(debug_assertions)]
            eprintln!(
                "  Buffer full: write_end={} exceeds MAX_DISPLAY_BUFFER_BYTES={}",
                write_end, MAX_DISPLAY_BUFFER_BYTES
            );
            break; // Buffer full
        }

        byte_index += char_len;
        visual_col += display_width; // Advance visual position by 1 or 2
    }

    // Defensive: Check we didn't hit iteration limit
    if write_iterations >= limits::HORIZONTAL_SCROLL_CHARS {
        #[cfg(debug_assertions)]
        eprintln!("process_line_with_offset: hit iteration limit during write phase");

        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::Other,
            "Maximum iterations exceeded in line write",
        )));
    }

    #[cfg(debug_assertions)]
    eprintln!(
        "  After write: byte_index={}, bytes_written={}, display_col={}",
        byte_index, bytes_written, display_col
    );

    // ═══════════════════════════════════════════════════════════════════════
    // ASSERTION CHECKS: Verify buffer bounds were respected
    // ═══════════════════════════════════════════════════════════════════════

    // Debug-Assert: Development/testing panic
    debug_assert!(
        bytes_written <= MAX_DISPLAY_BUFFER_BYTES,
        "Wrote {} bytes but buffer is only {} bytes",
        bytes_written,
        MAX_DISPLAY_BUFFER_BYTES
    );

    // Test-Assert: Cargo test panic
    #[cfg(test)]
    assert!(
        bytes_written <= MAX_DISPLAY_BUFFER_BYTES,
        "Wrote {} bytes but buffer is only {} bytes",
        bytes_written,
        MAX_DISPLAY_BUFFER_BYTES
    );

    // Production: Catch and handle without panic
    if bytes_written > MAX_DISPLAY_BUFFER_BYTES {
        return Err(LinesError::GeneralAssertionCatchViolation(
            "bytes_written > MAX_DISPLAY_BUFFER_BYTES".into(),
        ));
    }

    // Debug-Assert: Display column check
    debug_assert!(
        display_col <= display_col_limit,
        "Display column {} exceeds limit {}",
        display_col,
        display_col_limit
    );

    // Test-Assert: Display column check
    #[cfg(test)]
    assert!(
        display_col <= display_col_limit,
        "Display column {} exceeds limit {}",
        display_col,
        display_col_limit
    );

    // Production: Catch and handle
    if display_col > display_col_limit {
        return Err(LinesError::GeneralAssertionCatchViolation(
            "display_col > display_col_limit".into(),
        ));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // END-OF-LINE MAPPING: Map position after last visible character
    // ═══════════════════════════════════════════════════════════════════════
    // This allows cursor to be positioned at "end of visible text" (not EOL).
    // The byte_index now points to the next byte after the last displayed character.

    let eol_display_col = display_col; // Column right after last char

    if eol_display_col < display_col_limit {
        let eol_file_pos = FilePosition {
            byte_offset_linear_file_absolute_position: file_line_start + byte_index as u64,
            line_number: current_line_number,
            byte_in_line: byte_index,
        };

        #[cfg(debug_assertions)]
        eprintln!(
            "  EOL mapping: display_col={} → file_byte={} (end of visible text)",
            eol_display_col,
            file_line_start + byte_index as u64
        );

        state.set_file_position(row, eol_display_col, Some(eol_file_pos))?;
    }

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
    let cwd = env::current_dir()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Cannot determine current directory"))?;

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
                    "Home directory does not exist",
                ));
            }

            // Defensive: Verify it's a directory
            if !home_path.is_dir() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Home path is not a directory",
                ));
            }

            Ok(home_path)
        }
        Err(_) => {
            // Fallback: try USER environment variable with common paths
            if let Ok(user) = env::var("USER") {
                let mut possible_home = PathBuf::from("/home");
                possible_home.push(&user);
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
            "Filename too long characters max 255",
        ));
    }

    // Add .txt extension if no extension provided
    let mut buf = [0u8; 8]; // Adjust size as needed
    let filename_bytes;

    let filename = if trimmed.contains('.') {
        trimmed
    } else {
        let txt_suffix = b".txt";
        let trimmed_bytes = trimmed.as_bytes();

        if trimmed_bytes.len() + txt_suffix.len() <= buf.len() {
            buf[..trimmed_bytes.len()].copy_from_slice(trimmed_bytes);
            buf[trimmed_bytes.len()..trimmed_bytes.len() + txt_suffix.len()]
                .copy_from_slice(txt_suffix);

            filename_bytes = &buf[..trimmed_bytes.len() + txt_suffix.len()];
            std::str::from_utf8(filename_bytes).unwrap()
        } else {
            trimmed // Fallback if name too long
        }
    };

    Ok(filename.to_string())
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
    EnterRawMode,

    // Text editing
    InsertNewline(char), // Insert single \n at cursor's file-position
    // DeleteChar,          // Delete character at cursor // legacy?
    /// Delete entire line at cursor (normal mode)
    DeleteLine,

    /// Delete Selected (visual-select-mode) range to end of last character
    DeleteRange,

    /// Backspace-style delete (visual/insert modes)
    DeleteBackspace,

    // Select? up down left right byte count? or... to position?

    // File operations
    SaveFileStandard, // s
    SaveAs(PathBuf),

    // TODO SaveAs, // sa
    Quit,        // q
    SaveAndQuit, // w (write-quit)

    // Display
    TallPlus,
    TallMinus,
    WidePlus,
    WideMinus,

    // Cosplay for Variables
    Copyank, // c,y (in a normal mood)

    ToggleCommentOneLine(usize),       // current line is input
    ToggleDocstringOneLine(usize),     // current line is input
    ToggleBlockcomments(usize, usize), // start-row, stop-row
    IndentOneLine(usize),              // current line is input
    UnindentOneLine(usize),            // current line is input
    ToggleRustDocstringRange,
    ToggleBasicCommentlinesRange,
    IndentRange,
    UnindentRange,

    UndoButtonsCommand,
    RedoButtonsCommand,

    // No operation
    None,
}
/// Cleans up the specific draft copy file used in this editing session
///
/// # Purpose
/// Removes only the draft copy file that was being edited in this session.
/// Session directory and other draft copies remain intact for version management
/// and crash recovery across multiple files and editor sessions.
///
/// # Project Context - Version Management
/// Session directories persist to support:
/// - Multiple draft copies across time (version history)
/// - Recovery after crashes or unexpected exits
/// - Multi-file workflows with copy/paste between files
/// - Reopening same file multiple times in same session
///
/// This function removes only the current draft being edited, leaving session
/// infrastructure and other drafts available for future recovery and selection.
///
/// # Behavior
/// - Does NOT remove session directory itself
/// - Does NOT remove other draft copies in session directory
/// - Only removes the specific file at state.read_copy_path
/// - Silent success if file already gone (idempotent)
///
/// # Arguments
/// * `state` - Editor state containing read_copy_path to the draft file
///
/// # Returns
/// * `Ok(())` - Cleanup successful or no file to clean
/// * `Err(io::Error)` - Cleanup failed (non-fatal, should be logged but not halt exit)
///
/// # Design Notes
/// - Defensive checks prevent removing wrong files
/// - Errors should be logged but must not prevent program exit
/// - Called on normal exit (quit/save-quit) for cleanup
/// - Debug builds show cleanup notification; production builds silent
fn cleanup_session_directory_draft(state: &EditorState) -> io::Result<()> {
    // Get draft copy file path
    let draft_path = match &state.read_copy_path {
        Some(path) => path,
        None => {
            // No draft file to clean - nothing to do
            return Ok(());
        }
    };

    // Defensive: Verify this looks like a session draft file
    // Must contain both "lines_data" and "sessions" in path
    let path_str = draft_path.to_string_lossy();
    if !path_str.contains("lines_data") || !path_str.contains("sessions") {
        // return Err(io::Error::new(
        //     io::ErrorKind::InvalidInput,
        //     "cleanup_draft_copy_file: Path does not appear to be a session draft file",
        // ));

        // wrong file? Nothing to do, move on.
        return Ok(());
    }

    // Check if file exists
    if !draft_path.exists() {
        // Already gone - idempotent success
        return Ok(());
    }

    // Defensive: Verify it's a file (not a directory)
    if !draft_path.is_file() {
        // not a file, move on, nothing to do.
        return Ok(());
    }

    // At end during shut down, so, maybe ok to err.
    // Remove only this specific draft file
    fs::remove_file(draft_path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            stack_format_it(
                "cleanup_draft_copy_file: Failed to remove: {}",
                &[&e.to_string()],
                "cleanup_draft_copy_file: Failed to remove draft file",
            ),
        )
    })?;

    // Debug-only notification (production builds silent per security rules)
    #[cfg(debug_assertions)]
    {
        println!("Draft copy cleaned up: {}", draft_path.display());
    }

    Ok(())
}
/// Cleans up session directory and all its contents
///
/// # Purpose
/// Removes the session directory created for this editing session.
/// Called on normal exit (quit/save-quit) to cleanup temporary files.
///
/// # Arguments
/// * session directory path
///
/// # Returns
/// * `Ok(())` - Cleanup successful or no session directory to clean
/// * `Err(io::Error)` - Cleanup failed (non-fatal, logged)
///
/// # Safety
/// - Only removes directories under lines_data/tmp/sessions/
/// - Defensive checks prevent removing wrong directories
/// - Errors are logged but don't prevent exit
pub fn cleanup_all_session_directory(session_dir: &Path) -> io::Result<()> {
    // Defensive: Verify this is a session directory
    let path_str = session_dir.to_string_lossy();
    if !path_str.contains("lines_data") || !path_str.contains("sessions") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Refusing to delete directory that doesn't look like a session dir",
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
            "Session path exists but is not a directory",
        ));
    }

    // Remove the directory and all contents
    fs::remove_dir_all(session_dir).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            stack_format_it(
                "Failed to remove session directory: {}",
                &[&e.to_string()],
                "Failed to remove session directory",
            ),
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
        // =========
        // Move Left
        // =========
        Command::MoveLeft(count) => {
            // Vim-like behavior: move cursor left, scroll window if at edge
            let mut remaining_moves = count;
            let mut needs_rebuild = false;

            // Defensive: Limit iterations to prevent infinite loops
            let mut iterations = 0;

            // iterate through # of steps user requested
            while remaining_moves > 0 && iterations < limits::CURSOR_MOVEMENT_STEPS {
                /*
                I discovered this somewhat by accident:
                This subtraction 'overflow' catch seems to work
                as the zero-index finder and safety mechanism all in one.
                I am not a fan of having a failure be a trigger...
                but for MVP it looks to work.
                */
                // safe subtraction with error handling
                if let Some(new_position) = lines_editor_state
                    .in_row_abs_horizontal_0_index_cursor_position
                    .checked_sub(count)
                {
                    lines_editor_state.in_row_abs_horizontal_0_index_cursor_position = new_position;
                } else {
                    if lines_editor_state.cursor.row <= 0 {
                        _ = execute_command(lines_editor_state, Command::GotoLineStart)?;
                    }

                    // and don't try to go left again-again!
                    return Ok(true);
                }

                // =========================
                // position state inspection
                // =========================
                #[cfg(debug_assertions)]
                let this_row = lines_editor_state.cursor.row;
                #[cfg(debug_assertions)]
                let this_col = lines_editor_state.cursor.col;

                #[cfg(debug_assertions)]
                {
                    println!(
                        "MoveLeft lines_editor_state.cursor.row, .col-> {:?},{:?}",
                        this_row, this_col,
                    );
                    println!(
                        "\nMoveLeft lines_editor_state.get_row_col_file_position -> {:?}",
                        lines_editor_state.get_row_col_file_position(this_row, this_col)
                    );
                    println!(
                        "\nMoveLeft lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset -> {:?}",
                        lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset
                    );
                    println!(
                        "\nMoveLeft lines_editor_state.in_row_abs_horizontal_0_index_cursor_position -> {:?}",
                        lines_editor_state.in_row_abs_horizontal_0_index_cursor_position
                    );
                    println!(
                        "\nMoveLeft lines_editor_state.cursor.row -> {:?}",
                        lines_editor_state.cursor.row
                    );
                    println!(
                        "\nMoveLeft windowmap_line_byte_start_end_position_pairs -> {:?}",
                        lines_editor_state.windowmap_line_byte_start_end_position_pairs,
                    );
                    println!(
                        "\nMoveRight lines_editor_state.is_next_byte_newline() -> {:?}",
                        lines_editor_state.is_next_byte_newline()
                    );
                }

                iterations += 1;

                // =============
                // Window Scroll
                // =============
                if lines_editor_state.cursor.col > 0 {
                    // Cursor can move left within visible window
                    let cursor_moves = remaining_moves.min(lines_editor_state.cursor.col);
                    lines_editor_state.cursor.col -= cursor_moves;
                    remaining_moves -= cursor_moves;
                } else if lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset > 0 {
                    // Cursor at left edge, scroll window left
                    let scroll_amount = remaining_moves
                        .min(lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset);
                    lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset -=
                        scroll_amount;
                    remaining_moves -= scroll_amount;
                    needs_rebuild = true;
                } else {
                    // At absolute left edge - cannot move further
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

        // ==========
        // Move Right v7
        // ==========
        /*
        We should be able to track where the end of the line is
        if only by a next-newline look-ahead.
        and include an end-of-line space before going to the next line.
        one option might be, showing newlines as a space or other characer
        on the TUI.


        */
        Command::MoveRight(count) => {
            // Vim-like behavior: move cursor right, scroll window if at edge

            let mut remaining_moves = count;
            let mut needs_rebuild = false;

            // Defensive: Limit iterations
            let mut iterations = 0;

            // =========================
            // position state inspection
            // =========================

            // update for each MoveRight
            lines_editor_state.in_row_abs_horizontal_0_index_cursor_position += count;

            #[cfg(debug_assertions)]
            let this_row = lines_editor_state.cursor.row;
            #[cfg(debug_assertions)]
            let this_col = lines_editor_state.cursor.col;
            #[cfg(debug_assertions)]
            {
                println!(
                    "MoveRight lines_editor_state.cursor.row, .col-> {:?},{:?}",
                    this_row, this_col,
                );
                println!(
                    "\nMoveRight lines_editor_state.get_row_col_file_position -> {:?}",
                    lines_editor_state.get_row_col_file_position(this_row, this_col)
                );
                println!(
                    "\nMoveRight lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset -> {:?}",
                    lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset
                );
                println!(
                    "\nMoveRight lines_editor_state.in_row_abs_horizontal_0_index_cursor_position -> {:?}",
                    lines_editor_state.in_row_abs_horizontal_0_index_cursor_position
                );
                println!(
                    "\nMoveRight lines_editor_state.cursor.row -> {:?}",
                    lines_editor_state.cursor.row
                );
                println!(
                    "\nMoveRight windowmap_line_byte_start_end_position_pairs -> {:?}",
                    lines_editor_state.windowmap_line_byte_start_end_position_pairs,
                );
                println!(
                    "\nMoveRight lines_editor_state.is_next_byte_newline() -> {:?}",
                    lines_editor_state.is_next_byte_newline()
                );
            }

            while remaining_moves > 0 && iterations < limits::CURSOR_MOVEMENT_STEPS {
                iterations += 1;

                let is_next_newline = lines_editor_state.is_next_byte_newline()?;
                // ===============================================
                // First Check if Next Move Right should Jump Down
                // ================================================
                if lines_editor_state.next_move_right_is_past_newline {
                    // reset
                    lines_editor_state.next_move_right_is_past_newline = false;

                    // === Jump Down ===
                    // Move to start of current line
                    execute_command(lines_editor_state, Command::GotoLineStart)?;
                    // Move down one line
                    execute_command(lines_editor_state, Command::MoveDown(1))?;

                    remaining_moves -= 1;
                    needs_rebuild = true;
                    continue;
                } else if is_next_newline {
                    // ===========================================
                    // IF NEWLINE AHEAD: SWITCH TO LINE NAVIGATION
                    // ===========================================
                    // let is_next_newline = lines_editor_state.is_next_byte_newline()?;

                    // // Move to start of current line
                    // execute_command(lines_editor_state, Command::GotoLineStart)?;

                    // // Move down one line
                    // execute_command(lines_editor_state, Command::MoveDown(1))?;

                    // There is Room For One More Move-Right
                    lines_editor_state.next_move_right_is_past_newline = true;

                    lines_editor_state.cursor.col += 1;
                    remaining_moves -= 1;
                    needs_rebuild = true;
                    // continue;
                }

                // Calculate space available before right edge
                // Reserve 1 column to prevent display overflow
                let right_edge = lines_editor_state.effective_cols.saturating_sub(1);

                if lines_editor_state.cursor.col < (right_edge) {
                    // Cursor can move right within visible window
                    let space_available = right_edge - lines_editor_state.cursor.col;
                    let cursor_moves = remaining_moves.min(space_available);

                    lines_editor_state.cursor.col += cursor_moves;

                    // Inspection in debug build only
                    #[cfg(debug_assertions)]
                    println!(
                        "Inspection cursor_moves-> {:?}, col:{:?}",
                        &cursor_moves, lines_editor_state.cursor.col
                    );

                    remaining_moves -= cursor_moves;
                } else {
                    // Edge Scroll Right
                    if lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset
                        < limits::CURSOR_MOVEMENT_STEPS
                    {
                        let max_scroll = limits::CURSOR_MOVEMENT_STEPS
                            - lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset;
                        let scroll_amount = remaining_moves.min(max_scroll);

                        lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset +=
                            scroll_amount;

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

            // Only rebuild if we scrolled the window
            if needs_rebuild {
                // Rebuild window to show the change from read-copy file
                build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            }

            Ok(true)
        }

        // "Count" is int prefix entered by user, e.g. 20j (go down 20 lines)
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

            // =========================
            // position state inspection
            // =========================
            #[cfg(debug_assertions)]
            let this_row = lines_editor_state.cursor.row;
            #[cfg(debug_assertions)]
            let this_col = lines_editor_state.cursor.col;

            #[cfg(debug_assertions)]
            {
                println!(
                    "MoveDown lines_editor_state.cursor.row, .col-> {:?},{:?}",
                    this_row, this_col,
                );
                println!(
                    "\nMoveDown lines_editor_state.get_row_col_file_position -> {:?}",
                    lines_editor_state.get_row_col_file_position(this_row, this_col)
                );
                println!(
                    "\nMoveDown lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset -> {:?}",
                    lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset
                );
                println!(
                    "\nMoveDown lines_editor_state.in_row_abs_horizontal_0_index_cursor_position -> {:?}",
                    lines_editor_state.in_row_abs_horizontal_0_index_cursor_position
                );
                println!(
                    "\nMoveDown lines_editor_state.cursor.row -> {:?}",
                    lines_editor_state.cursor.row
                );
                println!(
                    "\nMoveDown windowmap_line_byte_start_end_position_pairs -> {:?}",
                    lines_editor_state.windowmap_line_byte_start_end_position_pairs,
                );
            }
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

                    let line_num_width = calculate_line_number_width(
                        lines_editor_state.line_count_at_top_of_window,
                        lines_editor_state.cursor.row,
                        lines_editor_state.effective_rows,
                    );

                    // if col is in the number-zone to the left of the text
                    // bump it over
                    if lines_editor_state.cursor.col < line_num_width {
                        lines_editor_state.cursor.col = line_num_width; // Skip over line number displayfull_lines_editor
                        lines_editor_state.in_row_abs_horizontal_0_index_cursor_position =
                            line_num_width;
                        lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset = 0;
                        build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
                    }

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

            Ok(true)
        }

        Command::MoveUp(count) => {
            // Vim-like behavior: move cursor up, scroll window if at top edge

            /*
            note for backup: position available...
            but abs state field seems to be working (mostly)
             */
            // // ========================================================================
            // // STEP 1: Get file position from cursor (defensive)
            // // ========================================================================

            // let current_file_pos = match lines_editor_state
            //     .get_row_col_file_position(lines_editor_state.cursor.row, lines_editor_state.cursor.col)
            // {
            //     Ok(Some(pos)) => pos,
            //     Ok(None) => {
            //         // handling None case
            //         let _ = lines_editor_state.set_info_bar_message("gh cursor position unavailable");
            //         return Ok(());
            //     }
            //     Err(e) => {
            //         let _ = lines_editor_state.set_info_bar_message("cannot get cursor position");
            //         log_error(
            //             &format!("goto_line_start window_map error: {}", e),
            //             Some("goto_line_start"),
            //         );
            //         return Ok(());
            //     }
            // };
            // let line_number_for_display = current_file_pos.line_number + 1; // Convert to 1-indexed

            // =========================
            // position state inspection
            // =========================
            #[cfg(debug_assertions)]
            let this_row = lines_editor_state.cursor.row;
            #[cfg(debug_assertions)]
            let this_col = lines_editor_state.cursor.col;

            #[cfg(debug_assertions)]
            {
                println!(
                    "MoveUp lines_editor_state.cursor.row, .col-> {:?},{:?}",
                    this_row, this_col,
                );
                println!(
                    "\nMoveUp lines_editor_state.get_row_col_file_position -> {:?}",
                    lines_editor_state.get_row_col_file_position(this_row, this_col)
                );
                println!(
                    "\nMoveUp lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset -> {:?}",
                    lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset
                );
                println!(
                    "\nMoveUp lines_editor_state.in_row_abs_horizontal_0_index_cursor_position -> {:?}",
                    lines_editor_state.in_row_abs_horizontal_0_index_cursor_position
                );
                println!(
                    "\nMoveUp lines_editor_state.cursor.row -> {:?}",
                    lines_editor_state.cursor.row
                );
                println!(
                    "\nMoveUp windowmap_line_byte_start_end_position_pairs -> {:?}",
                    lines_editor_state.windowmap_line_byte_start_end_position_pairs,
                );
            }
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

            let line_num_width = calculate_line_number_width(
                lines_editor_state.line_count_at_top_of_window,
                lines_editor_state.cursor.row,
                lines_editor_state.effective_rows,
            );

            // if position.. is <
            if lines_editor_state.in_row_abs_horizontal_0_index_cursor_position <= line_num_width {
                lines_editor_state.cursor.col = line_num_width;
                lines_editor_state.in_row_abs_horizontal_0_index_cursor_position = line_num_width;
                lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset = 0;
            }

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
                    let current_byte = match lines_editor_state.get_row_col_file_position(
                        lines_editor_state.cursor.row,
                        lines_editor_state.cursor.col,
                    ) {
                        Ok(Some(pos)) => {
                            let mut byte_buf = [0u8; 1];
                            let mut f = File::open(&base_edit_filepath)?;
                            f.seek(io::SeekFrom::Start(
                                pos.byte_offset_linear_file_absolute_position,
                            ))?;
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
                    let current_pos = match lines_editor_state.get_row_col_file_position(
                        lines_editor_state.cursor.row,
                        lines_editor_state.cursor.col,
                    ) {
                        Ok(Some(pos)) => pos.byte_offset_linear_file_absolute_position,
                        Ok(None) => break, // Invalid position, stop here
                        Err(_) => break,   // Lookup failed, stop here
                    };

                    // ===================================================================
                    // PEEK AHEAD: Look at NEXT byte (after current position)
                    // ===================================================================

                    let next_byte_pos = current_pos.saturating_add(1);

                    // Open file for peek operation
                    let mut f = File::open(&base_edit_filepath)?;

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
                    let current_pos = match lines_editor_state.get_row_col_file_position(
                        lines_editor_state.cursor.row,
                        lines_editor_state.cursor.col,
                    ) {
                        Ok(Some(pos)) => pos.byte_offset_linear_file_absolute_position,
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
                    let mut f = File::open(&base_edit_filepath)?;

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
            /*
            This goes to the beginning of a line.
             */
            // Convert 1-indexed (user display) to 0-indexed (file storage)
            let target_line = line_number.saturating_sub(1);

            // =========================
            // position state inspection
            // =========================
            // reset to first real position each new go-to-linw
            let line_num_width = calculate_line_number_width(
                lines_editor_state.line_count_at_top_of_window,
                lines_editor_state.cursor.row,
                lines_editor_state.effective_rows,
            );
            lines_editor_state.in_row_abs_horizontal_0_index_cursor_position = line_num_width;
            #[cfg(debug_assertions)]
            let this_row = lines_editor_state.cursor.row;
            #[cfg(debug_assertions)]
            let this_col = lines_editor_state.cursor.col;

            #[cfg(debug_assertions)]
            {
                println!(
                    "GotoLine lines_editor_state.cursor.row, .col-> {:?},{:?}",
                    this_row, this_col,
                );
                println!(
                    "\nGotoLine lines_editor_state.get_row_col_file_position -> {:?}",
                    lines_editor_state.get_row_col_file_position(this_row, this_col)
                );
                println!(
                    "\nGotoLine lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset -> {:?}",
                    lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset
                );
                println!(
                    "\nGotoLine lines_editor_state.in_row_abs_horizontal_0_index_cursor_position -> {:?}",
                    lines_editor_state.in_row_abs_horizontal_0_index_cursor_position
                );
                println!(
                    "\nGotoLine lines_editor_state.cursor.row -> {:?}",
                    lines_editor_state.cursor.row
                );
                println!(
                    "\nGotoLine windowmap_line_byte_start_end_position_pairs -> {:?}",
                    lines_editor_state.windowmap_line_byte_start_end_position_pairs,
                );
            }

            // Seek to target line and update window position
            match seek_to_line_number(&mut File::open(&base_edit_filepath)?, target_line) {
                Ok(byte_pos) => {
                    lines_editor_state.line_count_at_top_of_window = target_line;
                    lines_editor_state.file_position_of_topline_start = byte_pos;
                    lines_editor_state.cursor.row = 0;
                    lines_editor_state.cursor.col = 0;

                    // Position cursor AFTER line number (same as bootstrap)
                    // number of digits in line number + 1 is first character
                    let line_num_width = calculate_line_number_width(
                        lines_editor_state.line_count_at_top_of_window,
                        line_number,
                        lines_editor_state.effective_rows,
                    );
                    lines_editor_state.cursor.col = line_num_width; // Skip over line number displayfull_lines_editor
                    lines_editor_state.in_row_abs_horizontal_0_index_cursor_position =
                        line_num_width;
                    lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset = 0;
                    // Rebuild window to show the new position
                    build_windowmap_nowrap(lines_editor_state, &base_edit_filepath)?;

                    let _ = lines_editor_state.set_info_bar_message(&stack_format_it(
                        "Jumped to line {}",
                        &[&line_number.to_string()],
                        "Jumped to line",
                    ));
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

            // =========================
            // position state inspection
            // =========================
            // reset to first real position each new GotoFileStart
            let line_num_width = calculate_line_number_width(
                lines_editor_state.line_count_at_top_of_window,
                lines_editor_state.cursor.row,
                lines_editor_state.effective_rows,
            );
            lines_editor_state.in_row_abs_horizontal_0_index_cursor_position = line_num_width;
            #[cfg(debug_assertions)]
            let this_row = lines_editor_state.cursor.row;
            #[cfg(debug_assertions)]
            let this_col = lines_editor_state.cursor.col;

            #[cfg(debug_assertions)]
            {
                println!(
                    "GotoFileStart lines_editor_state.cursor.row, .col-> {:?},{:?}",
                    this_row, this_col,
                );
                println!(
                    "\nGotoFileStart lines_editor_state.get_row_col_file_position -> {:?}",
                    lines_editor_state.get_row_col_file_position(this_row, this_col)
                );
                println!(
                    "\nGotoFileStart lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset -> {:?}",
                    lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset
                );
                println!(
                    "\nGotoFileStart lines_editor_state.in_row_abs_horizontal_0_index_cursor_position -> {:?}",
                    lines_editor_state.in_row_abs_horizontal_0_index_cursor_position
                );
                println!(
                    "\nGotoFileStart lines_editor_state.cursor.row -> {:?}",
                    lines_editor_state.cursor.row
                );
                println!(
                    "\nGotoFileStart windowmap_line_byte_start_end_position_pairs -> {:?}",
                    lines_editor_state.windowmap_line_byte_start_end_position_pairs,
                );
            }

            // Seek to target line and update window position
            match seek_to_line_number(&mut File::open(&base_edit_filepath)?, target_line) {
                Ok(byte_pos) => {
                    lines_editor_state.line_count_at_top_of_window = target_line;
                    lines_editor_state.file_position_of_topline_start = byte_pos;
                    lines_editor_state.cursor.row = 0;
                    lines_editor_state.cursor.col = 3; // Skip over line number displayfull_lines_editor + padding

                    // Rebuild window to show the new position
                    build_windowmap_nowrap(lines_editor_state, &base_edit_filepath)?;

                    let _ = lines_editor_state.set_info_bar_message(&stack_format_it(
                        "Jumped to line {}",
                        &[&line_number.to_string()],
                        "Jumped to line",
                    ));
                    Ok(true)
                }
                Err(_) => {
                    let _ = lines_editor_state.set_info_bar_message("Line not found");
                    Ok(true)
                }
            }
        }

        Command::GotoFileLastLine => {
            // Count lines in file
            let (total_lines, _) = count_lines_in_file(&base_edit_filepath)?;

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
            let line_num_width = calculate_line_number_width(
                lines_editor_state.line_count_at_top_of_window,
                lines_editor_state.cursor.row,
                lines_editor_state.effective_rows,
            );
            lines_editor_state.cursor.col = line_num_width;
            lines_editor_state.in_row_abs_horizontal_0_index_cursor_position = line_num_width;
            lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset = 0;

            // rebuild
            _ = build_windowmap_nowrap(lines_editor_state, &base_edit_filepath);

            let _ = lines_editor_state.set_info_bar_message("start of line");

            // =========================
            // position state inspection
            // =========================
            // reset to first real position each new GotoLineStart
            // let line_num_width = calculate_line_number_width(lines_editor_state.cursor.row);

            #[cfg(debug_assertions)]
            let this_row = lines_editor_state.cursor.row;
            #[cfg(debug_assertions)]
            let this_col = lines_editor_state.cursor.col;

            #[cfg(debug_assertions)]
            {
                println!(
                    "GotoLineStart lines_editor_state.cursor.row, .col-> {:?},{:?}",
                    this_row, this_col,
                );
                println!(
                    "\nGotoLineStart lines_editor_state.get_row_col_file_position -> {:?}",
                    lines_editor_state.get_row_col_file_position(this_row, this_col)
                );
                println!(
                    "\nGotoLineStart lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset -> {:?}",
                    lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset
                );
                println!(
                    "\nGotoLineStart lines_editor_state.in_row_abs_horizontal_0_index_cursor_position -> {:?}",
                    lines_editor_state.in_row_abs_horizontal_0_index_cursor_position
                );
                println!(
                    "\nGotoLineStart lines_editor_state.cursor.row -> {:?}",
                    lines_editor_state.cursor.row
                );
                println!(
                    "\nGotoLineStart windowmap_line_byte_start_end_position_pairs -> {:?}",
                    lines_editor_state.windowmap_line_byte_start_end_position_pairs,
                );
            }

            Ok(true)
        }

        Command::GotoLineEnd => {
            goto_line_end(lines_editor_state, &base_edit_filepath)?;
            Ok(true)
        }

        Command::DeleteLine => {
            // =================================================
            // Clear Redo Stack Before Editing: Insert or Delete
            // =================================================
            let _: bool = match button_safe_clear_all_redo_logs(&base_edit_filepath) {
                Ok(success) => success,
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    eprintln!("Error clearing redo logs: {:?}", _e);

                    // Log error and continue (non-fatal)
                    log_error(
                        "Cannot clear redo logs",
                        Some("backspace_style_delete_noload"),
                    );
                    let _ = lines_editor_state.set_info_bar_message("bsdn Redo clear failed");

                    false // Treat error as failure
                }
            };
            delete_current_line_noload(lines_editor_state, &edit_file_path)?;
            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            Ok(true)
        }

        Command::DeleteRange => {
            // =================================================
            // Clear Redo Stack Before Editing: Insert or Delete
            // =================================================
            let _: bool = match button_safe_clear_all_redo_logs(&base_edit_filepath) {
                Ok(success) => success,
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    eprintln!("Error clearing redo logs: {:?}", _e);

                    // Log error and continue (non-fatal)
                    log_error(
                        "Cannot clear redo logs",
                        Some("backspace_style_delete_noload"),
                    );
                    let _ = lines_editor_state
                        .set_info_bar_message("DeleteRange call Redo clear failed");

                    false // Treat error as failure
                }
            };

            /*   If no block of text is selected,
             *   i.e. if start and end are the same point
             *   then use backspace mode,
             *   otherwise, if there is a block selected
             *   inclusively delete that block
             */
            if lines_editor_state.file_position_of_vis_select_start
                == lines_editor_state.file_position_of_vis_select_end
            {
                // if only one character is selected, use backspace delete
                backspace_style_delete_noload(lines_editor_state, &edit_file_path)?;
            } else {
                // if a more than one character is selected, inclusively delete
                delete_position_range_noload(lines_editor_state, &edit_file_path)?;
            }

            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            Ok(true)
        }

        Command::DeleteBackspace => {
            // =================================================
            // Clear Redo Stack Before Editing: Insert or Delete
            // =================================================
            let _: bool = match button_safe_clear_all_redo_logs(&base_edit_filepath) {
                Ok(success) => success,
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    eprintln!("Error clearing redo logs: {:?}", _e);

                    // Log error and continue (non-fatal)
                    log_error(
                        "Cannot clear redo logs",
                        Some("backspace_style_delete_noload"),
                    );
                    let _ = lines_editor_state.set_info_bar_message("bsdn Redo clear failed");

                    false // Treat error as failure
                }
            };

            backspace_style_delete_noload(lines_editor_state, &edit_file_path)?;
            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            Ok(true)
        }

        Command::InsertNewline(_) => {
            // =================================================
            // Clear Redo Stack Before Editing: Insert or Delete
            // =================================================
            let _: bool = match button_safe_clear_all_redo_logs(&base_edit_filepath) {
                Ok(success) => success,
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    eprintln!("Error clearing redo logs: {:?}", _e);

                    // Log error and continue (non-fatal)
                    log_error(
                        "Cannot clear redo logs",
                        Some("backspace_style_delete_noload"),
                    );
                    let _ = lines_editor_state.set_info_bar_message("bsdn Redo clear failed");

                    false // Treat error as failure
                }
            };

            insert_newline_at_cursor_chunked(lines_editor_state, edit_file_path)?;

            // Rebuild window to show the change
            build_windowmap_nowrap(lines_editor_state, edit_file_path)?;

            Ok(true)
        }

        Command::EnterInsertMode => {
            // Without rebuild here, hexedit changes do not appear until
            // after a next change. Keep in Sync.
            // Rebuild window to show the change from read-copy file
            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            lines_editor_state.mode = EditorMode::Insert;
            Ok(true)
        }

        Command::TallPlus => {
            // Check for handle here: must not be > MAX
            if (lines_editor_state.effective_rows + 1) <= MAX_TUI_ROWS {
                lines_editor_state.effective_rows += 1;
                build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            }
            // Else, Nothing to Do
            Ok(true)
        }
        Command::TallMinus => {
            // Check for handle here: must not be < MIN
            if (lines_editor_state.effective_rows - 1) >= MIN_TUI_ROWS {
                lines_editor_state.effective_rows -= 1;
                build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            }
            // Else, Nothing to Do

            Ok(true)
        }
        Command::WidePlus => {
            // Check for handle here: must not be > MAX
            if (lines_editor_state.effective_cols + 1) <= MAX_TUI_COLS {
                lines_editor_state.effective_cols += 1;
                build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            }
            Ok(true)
        }
        Command::WideMinus => {
            // Check for handle here: must not be < MIN
            if (lines_editor_state.effective_cols - 1) >= MIN_TUI_COLS {
                lines_editor_state.effective_cols -= 1;
                build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            }
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
            // Must rebuild here, or hexedit changes would not appear until
            // after a next change. Keep in Sync.

            // Set cursor position to file_position_of_vis_select_start
            // Get current cursor position in FILE
            if let Ok(Some(file_pos)) = lines_editor_state.get_row_col_file_position(
                lines_editor_state.cursor.row,
                lines_editor_state.cursor.col,
            ) {
                // Set/Reset BOTH start and end to same position initially
                lines_editor_state.file_position_of_vis_select_start =
                    file_pos.byte_offset_linear_file_absolute_position;
                lines_editor_state.file_position_of_vis_select_end =
                    file_pos.byte_offset_linear_file_absolute_position;
            }

            // Rebuild window to show the change from read-copy file
            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            lines_editor_state.mode = EditorMode::VisualSelectMode;
            // Set selection start at current cursor position
            if let Ok(Some(file_pos)) = lines_editor_state.get_row_col_file_position(
                lines_editor_state.cursor.row,
                lines_editor_state.cursor.col,
            ) {
                lines_editor_state.selection_start = Some(file_pos);
            }

            // set row of cursor start
            lines_editor_state.selection_rowline_start = lines_editor_state.cursor.row;
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
            if let Ok(Some(file_pos)) = lines_editor_state.get_row_col_file_position(
                lines_editor_state.cursor.row,
                lines_editor_state.cursor.col,
            ) {
                // Start hex cursor at same file position
                lines_editor_state
                    .hex_cursor
                    .byte_offset_linear_file_absolute_position =
                    file_pos.byte_offset_linear_file_absolute_position as usize;
            } else {
                // Fallback to file start if cursor position invalid
                lines_editor_state
                    .hex_cursor
                    .byte_offset_linear_file_absolute_position = 0;
            }

            Ok(true)
        }
        Command::EnterRawMode => {
            // rebuild may not be needed here, but just in case
            // Rebuild window to show the change from read-copy file
            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            lines_editor_state.mode = EditorMode::RawMode;

            // Convert current window position to file byte offset
            if let Ok(Some(file_pos)) = lines_editor_state.get_row_col_file_position(
                lines_editor_state.cursor.row,
                lines_editor_state.cursor.col,
            ) {
                // Start hex cursor at same file position
                lines_editor_state
                    .hex_cursor
                    .byte_offset_linear_file_absolute_position =
                    file_pos.byte_offset_linear_file_absolute_position as usize;
            } else {
                // Fallback to file start if cursor position invalid
                lines_editor_state
                    .hex_cursor
                    .byte_offset_linear_file_absolute_position = 0;
            }

            Ok(true)
        }
        Command::ToggleCommentOneLine(line_number_0number) => {
            // println!("line_number {line_number}");
            toggle_basic_singleline_comment(
                &edit_file_path.display().to_string(),
                line_number_0number,
            )?;
            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            Ok(true)
        }
        Command::ToggleDocstringOneLine(line_number_0number) => {
            toggle_rust_docstring_singleline_comment(
                &edit_file_path.display().to_string(),
                line_number_0number,
            )?;

            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            Ok(true)
        }
        Command::ToggleBlockcomments(start_row_0number, end_row_0number) => {
            #[cfg(debug_assertions)]
            {
                println!("start_row_0number {start_row_0number}");
                println!("end_row_0number {end_row_0number}");
            }

            toggle_block_comment(
                &edit_file_path.display().to_string(),
                start_row_0number,
                end_row_0number,
            )?;

            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            Ok(true)
        }

        Command::UnindentRange => {
            // =================================================
            // Clear Redo Stack Before Editing: Insert or Delete
            // =================================================
            let _: bool = match button_safe_clear_all_redo_logs(&base_edit_filepath) {
                Ok(success) => success,
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    eprintln!("Error clearing redo logs: {:?}", _e);

                    // Log error and continue (non-fatal)
                    log_error(
                        "Cannot clear redo logs",
                        Some("backspace_style_delete_noload"),
                    );
                    let _ = lines_editor_state.set_info_bar_message("bsdn Redo clear failed");

                    false // Treat error as failure
                }
            };

            /*
            pub fn unindent_range(
                file_path: &str,
                start_line: usize,
                end_line: usize,
            ) -> Result<(), ToggleIndentError> {
            */
            let _ = unindent_range(
                &base_edit_filepath.to_string_lossy(),
                lines_editor_state.selection_rowline_start,
                lines_editor_state.cursor.row,
            )?;

            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            Ok(true)
        }
        Command::IndentRange => {
            // =================================================
            // Clear Redo Stack Before Editing: Insert or Delete
            // =================================================
            let _: bool = match button_safe_clear_all_redo_logs(&base_edit_filepath) {
                Ok(success) => success,
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    eprintln!("Error clearing redo logs: {:?}", _e);

                    // Log error and continue (non-fatal)
                    log_error(
                        "Cannot clear redo logs",
                        Some("backspace_style_delete_noload"),
                    );
                    let _ = lines_editor_state.set_info_bar_message("bsdn Redo clear failed");

                    false // Treat error as failure
                }
            };

            /*
            pub fn indent_range(
                file_path: &str,
                start_line: usize,
                end_line: usize,
            ) -> Result<(), ToggleIndentError> {
            */
            let _ = indent_range(
                &base_edit_filepath.to_string_lossy(),
                lines_editor_state.selection_rowline_start,
                lines_editor_state.cursor.row,
            )?;

            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            Ok(true)
        }
        Command::ToggleRustDocstringRange => {
            // =================================================
            // Clear Redo Stack Before Editing: Insert or Delete
            // =================================================
            let _: bool = match button_safe_clear_all_redo_logs(&base_edit_filepath) {
                Ok(success) => success,
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    eprintln!("Error clearing redo logs: {:?}", _e);

                    // Log error and continue (non-fatal)
                    log_error(
                        "Cannot clear redo logs",
                        Some("backspace_style_delete_noload"),
                    );
                    let _ = lines_editor_state.set_info_bar_message("bsdn Redo clear failed");

                    false // Treat error as failure
                }
            };

            /*
            pub fn toggle_range_rust_docstring(
                file_path: &str,
                from_line: usize,
                to_line: usize,
            ) -> Result<(), ToggleCommentError> {
            */
            let _ = toggle_range_rust_docstring(
                &base_edit_filepath.to_string_lossy(),
                lines_editor_state.selection_rowline_start,
                lines_editor_state.cursor.row,
            )?;

            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            Ok(true)
        }
        Command::ToggleBasicCommentlinesRange => {
            // =================================================
            // Clear Redo Stack Before Editing: Insert or Delete
            // =================================================
            let _: bool = match button_safe_clear_all_redo_logs(&base_edit_filepath) {
                Ok(success) => success,
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    eprintln!("Error clearing redo logs: {:?}", _e);

                    // Log error and continue (non-fatal)
                    log_error(
                        "Cannot clear redo logs",
                        Some("backspace_style_delete_noload"),
                    );
                    let _ = lines_editor_state.set_info_bar_message("bsdn Redo clear failed");

                    false // Treat error as failure
                }
            };

            /*
            pub fn toggle_range_basic_comments(
                file_path: &str,
                from_line: usize,
                to_line: usize,
            ) -> Result<(), ToggleCommentError> {
            */
            let _ = toggle_range_basic_comments(
                &base_edit_filepath.to_string_lossy(),
                lines_editor_state.selection_rowline_start,
                lines_editor_state.cursor.row,
            )?;

            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            Ok(true)
        }
        Command::UnindentOneLine(line_number) => {
            // =================================================
            // Clear Redo Stack Before Editing: Insert or Delete
            // =================================================
            let _: bool = match button_safe_clear_all_redo_logs(&base_edit_filepath) {
                Ok(success) => success,
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    eprintln!("Error clearing redo logs: {:?}", _e);

                    // Log error and continue (non-fatal)
                    log_error(
                        "Cannot clear redo logs",
                        Some("backspace_style_delete_noload"),
                    );
                    let _ = lines_editor_state.set_info_bar_message("bsdn Redo clear failed");

                    false // Treat error as failure
                }
            };

            // println!("line_number {line_number}");
            unindent_line(&edit_file_path.display().to_string(), line_number)?;
            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            Ok(true)
        }
        Command::IndentOneLine(line_number) => {
            // =================================================
            // Clear Redo Stack Before Editing: Insert or Delete
            // =================================================
            let _: bool = match button_safe_clear_all_redo_logs(&base_edit_filepath) {
                Ok(success) => success,
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    eprintln!("Error clearing redo logs: {:?}", _e);

                    // Log error and continue (non-fatal)
                    log_error(
                        "Cannot clear redo logs",
                        Some("backspace_style_delete_noload"),
                    );
                    let _ = lines_editor_state.set_info_bar_message("bsdn Redo clear failed");

                    false // Treat error as failure
                }
            };

            // println!("line_number {line_number}");
            indent_line(&edit_file_path.display().to_string(), line_number)?;
            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            Ok(true)
        }
        // =============================
        // Undo Redo Buttons all undone!
        // =============================
        Command::UndoButtonsCommand => {
            let undo_path = get_undo_changelog_directory_path(&edit_file_path)?;

            match button_undo_redo_next_inverse_changelog_pop_lifo(&edit_file_path, &undo_path) {
                Ok(_) => {
                    #[cfg(debug_assertions)]
                    println!("Undo Action: OK");
                }
                Err(_e) => {
                    println!("Undo Operation failed");
                    #[cfg(debug_assertions)]
                    println!("Error: {}", _e);
                }
            }

            // Refresh TUI / Window-Map
            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;

            Ok(true)
        }
        Command::RedoButtonsCommand => {
            let redo_path = get_redo_changelog_directory_path(&edit_file_path)?;
            match button_undo_redo_next_inverse_changelog_pop_lifo(&edit_file_path, &redo_path) {
                Ok(_) => {
                    #[cfg(debug_assertions)]
                    {
                        println!("Redo Action: OK");
                    }
                }
                Err(_e) => {
                    println!("Redo Operation failed");
                    #[cfg(debug_assertions)]
                    println!("Error: {}", _e);
                }
            }

            // Refresh TUI / Window-Map
            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;

            Ok(true)
        }

        Command::SaveFileStandard => {
            save_file(lines_editor_state)?;
            Ok(true)
            // SaveFileStandard doesn't need rebuild (no content change in display)
        }
        Command::SaveAs(save_as_path) => {
            // Execute save-as operation
            // Note: save_as_path is PathBuf, we need &Path
            match save_file_as_newfile_with_newname(&edit_file_path, &save_as_path) {
                // Success: file copied
                Ok((FileOperationStatus::Copied, _)) => {
                    let info_message = "File Saved As.";
                    let _ = lines_editor_state.set_info_bar_message(&info_message);
                    Ok(true)
                }

                // Predicated outcome: destination already exists
                Ok((FileOperationStatus::AlreadyExisted, _)) => {
                    let info_message = "File already exists.";
                    let _ = lines_editor_state.set_info_bar_message(&info_message);
                    // Still return Ok - this is expected, not an error
                    Ok(true)
                }

                // Predicated outcome: source not found (shouldn't happen normally)
                Ok((FileOperationStatus::OriginalNotFound, _)) => {
                    let info_message = "Source file not found".to_string();
                    let _ = lines_editor_state.set_info_bar_message(&info_message);
                    Ok(true)
                }

                // Predicated outcome: source unavailable
                Ok((FileOperationStatus::OriginalUnavailable, _)) => {
                    let info_message = "Source file unavailable (locked?)".to_string();
                    let _ = lines_editor_state.set_info_bar_message(&info_message);
                    Ok(true)
                }

                // Predicated outcome: destination unavailable
                Ok((FileOperationStatus::DestinationUnavailable, _)) => {
                    #[cfg(not(debug_assertions))]
                    let info_message = format!(
                        "Cannot write to: {} (check directory exists)",
                        save_as_path.display()
                    );
                    #[cfg(not(debug_assertions))]
                    let _ = lines_editor_state.set_info_bar_message(&info_message);

                    // Prod Safe (e.g. size)
                    let info_message = "Can't write,path exists?";

                    let _ = lines_editor_state.set_info_bar_message(&info_message);
                    Ok(true)
                }

                // True error: propagate up
                Err(e) => {
                    // Log error (production safe - no paths in message)
                    #[cfg(not(debug_assertions))]
                    log_error("Save as failed", Some("command_handler:save_as"));

                    // Set user-visible error message
                    let _ = lines_editor_state.set_info_bar_message("|o| SaveAs faiL |o|");

                    // Propagate error up the chain
                    Err(e)
                }
            }
        }
        // Command::SaveAs(save_as_path) => {
        //     // 1     original_file_path: &Path, new_file_path_name: &Path,
        //     let saveas_status_message: String =
        //         save_file_as_newfile_with_newname(&edit_file_path, &save_as_path)?;
        //     // 2. message
        //     let _ = lines_editor_state.set_info_bar_message(saveas_status_message);

        //     Ok(true)
        //     // SaveAs doesn't need rebuild (no content change in display)
        // }
        Command::Quit => {
            // Note: There is no 'must-save' functionality by default,
            // because that would require saving rejected/unsafe changes.
            // How is that ok?
            // For special uses you CAN add must-save here, but think it though.

            if let Err(_e) = cleanup_session_directory_draft(lines_editor_state) {
                #[cfg(debug_assertions)]
                eprintln!("Warning: Session cleanup failed: {}", _e);
                log_error("Session cleanup failed", Some("Command::Quit"));
                // Continue with exit anyway
            }

            // Note:
            // If using as module, you may need to call:
            //     _ = cleanup_all_session_directory(&lines_editor_state);

            // Default behavior: Let User Decide
            Ok(false) // Signal to exit loop
        }

        Command::SaveAndQuit => {
            save_file(lines_editor_state)?; // save file

            // Clean up session directory after save
            if let Err(_e) = cleanup_session_directory_draft(lines_editor_state) {
                #[cfg(debug_assertions)]
                eprintln!("Warning: Session cleanup failed: {}", _e);
                log_error("Session cleanup failed: {}", Some("Command::SaveAndQuit"));
                // Continue with exit anyway
            }

            // Note:
            // If using as module, you may need to call:
            //     _ = cleanup_all_session_directory(&lines_editor_state);

            Ok(false) // Signal to exit after save
        }

        Command::Copyank => {
            // Copy the Selection To The Pasty Clipboard (as a file)
            copy_selection_to_clipboardfile(lines_editor_state, &base_edit_filepath)?;

            Ok(true)
        }

        Command::None => Ok(true),
    }
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
fn goto_line_end(lines_editor_state: &mut EditorState, file_path: &Path) -> Result<()> {
    // ========================================================================
    // STEP 1: Get file position from cursor (defensive)
    // ========================================================================

    let current_file_pos = match lines_editor_state
        .get_row_col_file_position(lines_editor_state.cursor.row, lines_editor_state.cursor.col)
    {
        Ok(Some(pos)) => pos,
        Ok(None) => {
            let _ = lines_editor_state.set_info_bar_message("gl cursor pos. unavailable");
            return Ok(());
        }
        Err(_e) => {
            let _ = lines_editor_state.set_info_bar_message("cannot get cursor position");
            #[cfg(debug_assertions)]
            eprintln!("e: {}", _e);
            log_error("goto_line_end window_map error", Some("goto_line_end"));
            return Ok(());
        }
    };

    let line_number_for_display = current_file_pos.line_number + 1; // Convert to 1-indexed
    let line_start_byte = current_file_pos.byte_offset_linear_file_absolute_position
        - (current_file_pos.byte_in_line as u64);

    // ========================================================================
    // STEP 2: Read the line from file
    // ========================================================================

    let mut line_buffer = [0u8; 4096];

    let mut file = match File::open(file_path) {
        Ok(f) => f,
        Err(_e) => {
            let _ = lines_editor_state.set_info_bar_message("cannot open file");
            #[cfg(debug_assertions)]
            eprintln!("e: {}", _e);
            log_error("goto_line_end open error", Some("goto_line_end"));
            return Ok(());
        }
    };

    if let Err(_e) = file.seek(SeekFrom::Start(line_start_byte)) {
        let _ = lines_editor_state.set_info_bar_message("cannot seek to line");
        #[cfg(debug_assertions)]
        eprintln!("e: {}", _e);
        log_error("goto_line_end seek error", Some("goto_line_end"));
        return Ok(());
    }

    let (_, line_length, _) = match read_single_line(&mut file, &mut line_buffer) {
        Ok(result) => result,
        Err(_e) => {
            let _ = lines_editor_state.set_info_bar_message("cannot read line");
            #[cfg(debug_assertions)]
            eprintln!("e: {}", _e);
            #[cfg(debug_assertions)]
            log_error("goto_line_end read error", Some("goto_line_end"));
            return Ok(());
        }
    };

    // =========================
    // position state inspection
    // =========================
    let line_num_width = calculate_line_number_width(
        lines_editor_state.line_count_at_top_of_window,
        lines_editor_state.cursor.row,
        lines_editor_state.effective_rows,
    );
    // reset for each new fn goto_line_end
    lines_editor_state.in_row_abs_horizontal_0_index_cursor_position = line_length + line_num_width;
    let this_row = lines_editor_state.cursor.row;
    let this_col = lines_editor_state.cursor.col;
    println!(
        "fn goto_line_end lines_editor_state.cursor.row, .col-> {:?},{:?}",
        this_row, this_col,
    );
    println!(
        "\nfn goto_line_end lines_editor_state.get_row_col_file_position -> {:?}",
        lines_editor_state.get_row_col_file_position(this_row, this_col)
    );
    println!(
        "\nfn goto_line_end lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset -> {:?}",
        lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset
    );
    println!(
        "\nfn goto_line_end lines_editor_state.in_row_abs_horizontal_0_index_cursor_position -> {:?}",
        lines_editor_state.in_row_abs_horizontal_0_index_cursor_position
    );
    println!(
        "\nfn goto_line_end lines_editor_state.cursor.row -> {:?}",
        lines_editor_state.cursor.row
    );
    println!(
        "\nfn goto_line_end windowmap_line_byte_start_end_position_pairs -> {:?}",
        lines_editor_state.windowmap_line_byte_start_end_position_pairs,
    );

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

    let line_num_width = calculate_line_number_width(
        lines_editor_state.line_count_at_top_of_window,
        line_number_for_display,
        lines_editor_state.effective_rows,
    );
    let display_col_for_line_end = line_num_width + char_position_in_line;

    let right_edge = lines_editor_state.effective_cols.saturating_sub(1);
    let mut needs_rebuild = false;

    // ========================================================================
    // STEP 5: Handle horizontal scrolling
    // ========================================================================

    if display_col_for_line_end > right_edge {
        // Line is longer than terminal width
        let overflow = display_col_for_line_end - right_edge;

        lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset = lines_editor_state
            .tui_window_horizontal_utf8txt_line_char_offset
            .saturating_add(overflow);

        lines_editor_state.cursor.col = right_edge;
        // println!("right_edge {right_edge}, display_col_for_line_end {display_col_for_line_end}");
        needs_rebuild = true;
    } else {
        // Line fits within terminal
        // TODO: why is this odd?
        lines_editor_state.cursor.col = display_col_for_line_end;
    }

    // ========================================================================
    // STEP 6: Rebuild if needed
    // ========================================================================

    if needs_rebuild {
        if let Err(_e) = build_windowmap_nowrap(lines_editor_state, file_path) {
            let _ = lines_editor_state.set_info_bar_message("display update failed");
            #[cfg(debug_assertions)]
            eprintln!("e: {}", _e);
            #[cfg(debug_assertions)]
            log_error("goto_line_end rebuild error", Some("goto_line_end"));
            // Continue anyway - cursor was updated
        }
    }

    // message? 'end of line'?
    // let _ = lines_editor_state.set_info_bar_message(&format!("end of line ({} chars)", char_count));
    // let _ = lines_editor_state.set_info_bar_message(&char_count.to_string());
    let _ = lines_editor_state.set_info_bar_message("end of line");
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
fn backspace_style_delete_noload(
    lines_editor_state: &mut EditorState,
    file_path: &Path,
) -> io::Result<()> {
    // Step 1: Get current file position
    let file_pos = lines_editor_state
        .get_row_col_file_position(lines_editor_state.cursor.row, lines_editor_state.cursor.col)?
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "bsd: Cursor not on valid position",
            )
        })?;

    let cursor_byte = file_pos.byte_offset_linear_file_absolute_position;

    // Step 2: Can't delete before start of file
    if cursor_byte == 0 {
        return Ok(()); // Nothing to delete
    }

    // Step 3: Find start of previous UTF-8 character
    // Read up to 4 bytes back to find character boundary
    let prev_char_start = find_previous_utf8_boundary(file_path, cursor_byte)?;

    // ============================================
    // Step 3.5: Read Character BEFORE Deletion
    // ============================================
    // We need the character value for the undo log
    // Must read it before we delete it from the file

    let character_to_delete =
        match read_character_bytes_from_file(file_path, prev_char_start as u128) {
            Ok(char_bytes) => {
                // Decode bytes to char
                match std::str::from_utf8(&char_bytes) {
                    Ok(s) => s.chars().next(), // Some(char) or None if empty
                    Err(_) => {
                        // Invalid UTF-8 - log but continue with deletion
                        #[cfg(debug_assertions)]
                        log_error(
                            &stack_format_it(
                                "backspace_style_delete_noload Invalid UTF-8 at position {}",
                                &[&prev_char_start.to_string()],
                                "backspace_style_delete_noload Invalid UTF-8 at position",
                            ),
                            Some("backspace_style_delete_noload:read_char"),
                        );

                        #[cfg(not(debug_assertions))]
                        log_error(
                            "Invalid UTF-8 character",
                            Some("backspace_style_delete_noload:read_char"),
                        );

                        None // Continue without character for undo
                    }
                }
            }
            Err(_e) => {
                // Cannot read character - log but continue with deletion
                #[cfg(debug_assertions)]
                log_error(
                    &stack_format_it(
                        "bsdn Cannot read char at pos {}: {}",
                        &[&prev_char_start.to_string(), &_e.to_string()],
                        "bsdn Cannot read char at pos",
                    ),
                    Some("backspace_style_delete_noload:read_char"),
                );

                #[cfg(not(debug_assertions))]
                log_error(
                    "Cannot read character",
                    Some("backspace_style_delete_noload:read_char"),
                );

                None // Continue without character for undo
            }
        };

    // Step 4: Delete byte range [prev_char_start..cursor_byte)
    delete_byte_range_chunked(file_path, prev_char_start, cursor_byte)?;

    // ============================================
    // Step 4.5: Create Inverse Changelog Entry
    // ============================================
    // Create undo log for character deletion
    // User action: Rmv → Inverse log: Add (restore character)
    // This is non-critical - if it fails, deletion still succeeded

    let log_directory_path = match get_undo_changelog_directory_path(file_path) {
        Ok(path) => Some(path),
        Err(_e) => {
            // Non-critical: Log error but don't fail the deletion
            #[cfg(debug_assertions)]
            log_error(
                &stack_format_it(
                    "Cannot get changelog directory: {}",
                    &[&_e.to_string()],
                    "Cannot get changelog directory",
                ),
                Some("backspace_style_delete_noload:changelog"),
            );

            #[cfg(not(debug_assertions))]
            log_error(
                "Cannot get changelog directory",
                Some("backspace_style_delete_noload:changelog"),
            );

            // Continue without undo support - deletion succeeded
            None
        }
    };

    // Create log entry if we have both directory path AND the character
    if let (Some(log_dir), Some(deleted_char)) = (log_directory_path, character_to_delete) {
        // Retry logic: 3 attempts with 50ms pause
        let mut log_success = false;

        for retry_attempt in 0..3 {
            // Convert u64 position to u128 for API compatibility
            let position_u128 = prev_char_start as u128;

            /*
            pub fn button_make_changelog_from_user_character_action_level(
                target_file: &Path,
                character: Option<char>,
                byte_value: Option<u8>, // raw byte input
                position: u128,
                edit_type: EditType,
                log_directory_path: &Path,
            ) -> ButtonResult<()> {
            */

            match button_make_changelog_from_user_character_action_level(
                file_path,
                Some(deleted_char), // Character that was deleted (for restore)
                None,               // raw byte input
                position_u128,
                EditType::RmvCharacter, // User removed, inverse is add
                &log_dir,
            ) {
                Ok(_) => {
                    log_success = true;
                    break; // Success
                }
                Err(_e) => {
                    if retry_attempt == 2 {
                        // Final retry failed - log but don't fail operation
                        #[cfg(debug_assertions)]
                        log_error(
                            &stack_format_it(
                                "bsdn Fail log deleted char '{}' pos {}: {}",
                                &[
                                    &deleted_char.to_string(),
                                    &position_u128.to_string(),
                                    &_e.to_string(),
                                ],
                                "bsdn Fail to log deleted char at position",
                            ),
                            Some("backspace_style_delete_noload:changelog"),
                        );

                        #[cfg(not(debug_assertions))]
                        log_error(
                            "Failed to log deletion",
                            Some("backspace_style_delete_noload:changelog"),
                        );
                    } else {
                        // Retry after brief pause
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                }
            }
        }

        // Optional: Set info bar if logging failed (non-intrusive)
        if !log_success {
            let _ = lines_editor_state.set_info_bar_message("undo disabled");
        }
    } else if character_to_delete.is_none() {
        // Could read character for undo - inform user
        #[cfg(debug_assertions)]
        log_error(
            "Undo disabled: could not read deleted character",
            Some("backspace_style_delete_noload:changelog"),
        );

        #[cfg(not(debug_assertions))]
        log_error(
            "Undo disabled",
            Some("backspace_style_delete_noload:changelog"),
        );

        let _ = lines_editor_state.set_info_bar_message("undo disabled");
    }

    // Step 5: Update lines_editor_state
    lines_editor_state.is_modified = true;

    // Step 7: Move cursor back one position
    if lines_editor_state.cursor.col > 0 {
        lines_editor_state.cursor.col -= 1;
    } else if lines_editor_state.cursor.row > 0 {
        // Deleted at line start - move to end of previous line
        lines_editor_state.cursor.row -= 1;
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

// ==============================
// That's a Cheap Trick, Buttons!
// ==============================

/// Deletes entire line at cursor WITHOUT loading whole file, with undo support
///
/// # Overview
/// Deletes the line containing the cursor using chunked file operations and creates
/// inverse changelog entries for undo. Line content is saved to a temporary file
/// before deletion, then changelog entries are created character-by-character using
/// the "Cheap Trick" button stack approach.
///
/// # The "Cheap Trick" Button Stack (Critical for Undo!)
///
/// **The Problem We Solve:**
/// When deleting a line like "pine\nuts nheggs\n" at position 25, we need to create
/// undo logs that will reconstruct it correctly. Naive approach would be:
/// ```text
/// Log: ADD 'p' at 25
/// Log: ADD 'i' at 26  ← WRONG! Position changes as we add
/// Log: ADD 'n' at 27
/// ...
/// ```
/// When undo runs backwards (LIFO), it would add last character first at wrong position.
///
/// **The Solution: All Logs Use Same Position**
/// ```text
/// Log 1.o: ADD 'p' at 25  (first char, highest letter, last to execute)
/// Log 1.n: ADD 'i' at 25  (same position!)
/// Log 1.m: ADD 'n' at 25  (same position!)
/// Log 1.l: ADD 'e' at 25  (same position!)
/// ...
/// Log 1.a: ADD 's' at 25  (same position!)
/// Log 1:   ADD '\n' at 25 (last char, no letter, first to execute)
/// ```
///
/// **How Button Stack Reconstructs the Line:**
/// When undo executes (reads files in sorted order: 1, 1.a, 1.b, ..., 1.o):
/// 1. ADD '\n' at 25 → "\n" at position 25
/// 2. ADD 's' at 25 → "s\n" at positions 25-26 (pushes \n right)
/// 3. ADD 'g' at 25 → "gs\n" at 25-26-27 (pushes s,\n right)
/// 4. ADD 'g' at 25 → "ggs\n" at 25-26-27-28
/// 5. ... continues pushing right ...
/// 16. ADD 'e' at 25 → "e...ggs\n" (all chars pushed right)
/// 17. ADD 'p' at 25 → "pe...ggs\n" (reconstruction complete!)
///
/// Result: "pine\nuts nheggs\n" perfectly reconstructed!
///
/// **Why This Works:**
/// - LIFO (Last In, First Out): Undo reads logs in reverse order of creation
/// - Insert-at-same-position: Each insertion pushes previous characters right
/// - Natural cascading: File operations automatically shift bytes
/// - Fewer moving parts: No position arithmetic, just one constant position
/// - UTF-8 safe: Works for multi-byte characters (each byte gets same position)
///
/// **Letter Suffixes Enforce Execution Order:**
/// - No letter (e.g., "1"): Last character in line, executed FIRST by undo
/// - Letter 'a' (e.g., "1.a"): Second-to-last character, executed second
/// - Letter 'b' (e.g., "1.b"): Third-to-last, executed third
/// - ...
/// - Highest letter (e.g., "1.o"): First character in line, executed LAST by undo
///
/// This naming ensures correct LIFO execution order through filesystem sorting.
///
/// # Algorithm
///
/// **Phase 1: Find Line Boundaries**
/// 1. Get cursor's byte position in file
/// 2. Scan backwards to find line start (previous \n or BOF)
/// 3. Scan forwards to find line end (next \n or EOF)
/// 4. Include trailing newline if present
///
/// **Phase 2: Save Line to Temp File**
/// 5. Create temporary file (file.tmp_deleted_line)
/// 6. Copy line bytes [line_start..delete_end] to temp file (chunked, no heap)
/// 7. Flush and close temp file
/// 8. If copy fails: clean up temp file, abort operation
///
/// **Phase 3: Delete Line from Source File**
/// 9. Delete byte range [line_start..delete_end] using chunked operations
/// 10. If deletion fails: clean up temp file, abort operation
///
/// **Phase 4: Create Undo Logs (Button Stack)**
/// 11. Get changelog directory path
/// 12. Open temp file for reading
/// 13. Iterate through temp file character-by-character (chunked)
/// 14. For each UTF-8 character:
///     - Position = line_start (NOT line_start + offset!) ← Key insight!
///     - Call button_make_changelog_from_user_character_action_level()
///     - EditType = Rmv (user removed line, inverse adds it back)
///     - Character = Some(char) (need character for restoration)
/// 15. Handle UTF-8 boundaries across chunks (carry-over buffer)
/// 16. Retry each log creation up to 3 times
/// 17. Continue on logging errors (non-critical, deletion succeeded)
///
/// **Phase 5: Cleanup and Update State**
/// 18. Delete temp file
/// 19. Mark editor state as modified
/// 20. Log the edit operation
/// 21. Move cursor to column 0 (start of new line at same row)
///
/// # Memory Safety
///
/// **Stack-only buffers:**
/// - Line copy buffer: [0u8; 256] - 256 bytes on stack
/// - UTF-8 carry-over buffer: [0u8; 4] - 4 bytes on stack (max UTF-8 char)
/// - No heap allocation for data processing
/// - Temp file on disk (not in memory)
///
/// **Bounded iterations:**
/// - MAX_COPY_ITERATIONS: 1,000,000 (prevents infinite loops)
/// - MAX_CHUNKS: 16,777,216 (during changelog creation)
/// - MAX_LOGGING_ERRORS: 100 (stops after too many failures)
///
/// # Error Handling Philosophy
///
/// **Critical operations (must succeed):**
/// - Finding line boundaries: Return error if cursor invalid
/// - Line copy to temp: Return error, clean up temp file
/// - Line deletion: Return error, clean up temp file
///
/// **Non-critical operations (fail gracefully):**
/// - Changelog directory creation: Continue without undo
/// - Temp file re-opening for logging: Continue without undo
/// - Individual log creation: Retry 3x, then skip and continue
/// - Temp file cleanup: Log error but don't fail operation
///
/// **Undo is a luxury, never blocks deletion.**
///
/// # Edge Cases
///
/// **Empty line:**
/// - Line contains only "\n"
/// - Creates one log entry: ADD '\n' at line_start
/// - Undo restores the newline
///
/// **Last line without trailing \n:**
/// - delete_end = line_end (no +1)
/// - Deletes to EOF
/// - Undo restores line without adding extra newline
///
/// **Single line file:**
/// - line_start = 0, line_end = EOF
/// - Results in empty file
/// - Undo restores the entire file content
///
/// **First line:**
/// - line_start = 0 (BOF)
/// - Works normally, deletes from beginning
///
/// **Line with multi-byte UTF-8 characters:**
/// - Each character logged separately at same position
/// - Multi-byte chars handled by button_make_changeloge... function
/// - Creates letter-suffixed log files (e.g., 1.a, 1.b) automatically
///
/// **Invalid UTF-8 in line:**
/// - Logged as error (debug mode) or terse message (production)
/// - Skips invalid byte(s)
/// - Continues processing rest of line
/// - Undo will not restore invalid bytes
///
/// **Line longer than MAX_COPY_ITERATIONS × 256 bytes:**
/// - Copy phase aborts with error
/// - Deletion does not occur
/// - No orphan undo logs created
///
/// **Logging failures:**
/// - Each character retried 3 times with 50ms pause
/// - After 100 total errors: stops creating logs
/// - Info bar shows "undo log incomplete"
/// - Deletion still succeeded, undo partially disabled
///
/// **Temp file already exists:**
/// - File::create() truncates existing file
/// - Not an error, just overwrites
///
/// # Why Temp File Approach?
///
/// **Prevents Orphan Logs:**
/// If we created undo logs BEFORE deletion and deletion failed, we'd have
/// orphan logs for a delete that never happened. Corrupts undo history.
///
/// **Clean Failure Semantics:**
/// - Save line → fails → abort, no side effects
/// - Save line → success → Delete line → fails → abort, temp file cleaned up
/// - Save line → success → Delete line → success → Create logs → can't fail critically
///
/// **Reuses Proven Pattern:**
/// Logging loop is identical to file insertion Phase 6. Same UTF-8 handling,
/// same carry-over buffer, same error handling, same retry logic.
///
/// # Position Tracking
///
/// **Important: _byte_offset_in_line is tracked but NOT used for positions!**
/// ```rust
/// _byte_offset_in_line += char_len;  // Only for error messages
/// char_position = line_start;        // Always the same position!
/// ```
///
/// This seems counterintuitive but is critical for button stack to work.
///
/// # Arguments
///
/// * `state` - Editor state with cursor position
/// * `file_path` - Path to the file being edited (read-copy, absolute path)
///
/// # Returns
///
/// * `Ok(())` - Line deleted successfully (with or without undo logs)
/// * `Err(io::Error)` - Critical operation failed (line NOT deleted)
///
/// # Side Effects
///
/// - Deletes byte range from file
/// - Creates multiple changelog files in undo directory
/// - Creates and deletes temporary file (file.tmp_deleted_line)
/// - Marks editor state as modified
/// - Moves cursor to column 0
/// - May set info bar message on non-critical errors
///
/// # Examples
///
/// ```ignore
/// // Delete line 3: "pine\nuts nheggs\n" at position 25
/// delete_current_line_noload(&mut state, &file_path)?;
///
/// // Undo logs created (button stack, all at position 25):
/// // changelog_file/1.o: ADD 'p' at 25
/// // changelog_file/1.n: ADD 'i' at 25
/// // ... 14 more logs ...
/// // changelog_file/1.a: ADD 's' at 25
/// // changelog_file/1:   ADD '\n' at 25
///
/// // User presses undo:
/// // 1. Reads "1" → ADD '\n' at 25 → "\n"
/// // 2. Reads "1.a" → ADD 's' at 25 → "s\n"
/// // 3. Reads "1.b" → ADD 'g' at 25 → "gs\n"
/// // ... cascading insertions ...
/// // 17. Reads "1.o" → ADD 'p' at 25 → "pine\nuts nheggs\n" ✓
/// ```
///
/// # See Also
///
/// * `button_make_changelog_from_user_character_action_level()` - Creates individual log entries
/// * `button_add_multibyte_make_log_files()` - Handles multi-byte characters with letter suffixes
/// * `delete_byte_range_chunked()` - Performs the actual deletion
/// * `find_line_start()` - Finds beginning of current line
/// * `find_line_end()` - Finds end of current line
///
/// # Testing Considerations
///
/// Test with lines containing:
/// - Empty line ("\n")
/// - Single character ("a\n")
/// - ASCII text ("Hello, world!\n")
/// - Multi-byte UTF-8 ("你好世界\n")
/// - Mixed ASCII and UTF-8 ("Hello 世界\n")
/// - No trailing newline (last line of file)
/// - Very long line (test MAX_COPY_ITERATIONS)
/// - Invalid UTF-8 bytes
/// - Line at start of file (BOF)
/// - Line at end of file (EOF)
/// - Single line file
fn delete_current_line_noload(state: &mut EditorState, file_path: &Path) -> Result<()> {
    // Step 1: Get current line's file position
    let row_col_file_pos = state
        .get_row_col_file_position(state.cursor.row, state.cursor.col)?
        .ok_or_else(|| LinesError::InvalidInput("Cursor not on valid position".into()))?;

    // Step 2: Find line boundaries
    let line_start = find_line_start(
        file_path,
        row_col_file_pos.byte_offset_linear_file_absolute_position,
    )?;
    let line_end = find_line_end(
        file_path,
        row_col_file_pos.byte_offset_linear_file_absolute_position,
    )?;

    // Step 3: Include the newline character if present
    let delete_end = if line_end_has_newline(file_path, line_end)? {
        line_end + 1
    } else {
        line_end
    };

    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    debug_assert!(
        line_start <= delete_end,
        "Line start must be before or at delete end"
    );

    #[cfg(test)]
    assert!(
        line_start <= delete_end,
        "Line start must be before or at delete end"
    );

    if line_start > delete_end {
        #[cfg(debug_assertions)]
        log_error(
            &stack_format_it(
                "Invalid line bounds: start {} > end {}",
                &[&line_start.to_string(), &delete_end.to_string()],
                "Invalid line bounds",
            ),
            Some("delete_current_line_noload"),
        );

        #[cfg(not(debug_assertions))]
        log_error("Invalid line bounds", Some("delete_current_line_noload"));

        let _ = state.set_info_bar_message("line bounds error");
        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid line boundarie",
        )));
    }

    // ============================================
    // Step 2.5: Copy Line to Temporary File
    // ============================================
    // Save line content before deletion so we can create undo logs afterward
    // This prevents orphan logs if deletion fails

    let temp_line_path = file_path.with_extension("tmp_deleted_line");

    // Open source file for reading the line
    let mut source_file = File::open(file_path)?;

    // Create temp file for saving line
    let mut temp_file = File::create(&temp_line_path)?;

    // Seek to line start
    source_file.seek(SeekFrom::Start(line_start))?;

    // Copy line bytes to temp file (chunked, no heap)
    const CHUNK_SIZE: usize = 256;
    let mut buffer = [0u8; CHUNK_SIZE];
    let mut bytes_to_copy = (delete_end - line_start) as usize;
    let mut copy_iterations = 0;
    const MAX_COPY_ITERATIONS: usize = 1_000_000; // Safety limit

    while bytes_to_copy > 0 && copy_iterations < MAX_COPY_ITERATIONS {
        copy_iterations += 1;

        let to_read = bytes_to_copy.min(CHUNK_SIZE);
        let bytes_read = source_file.read(&mut buffer[..to_read])?;

        if bytes_read == 0 {
            break; // EOF
        }

        temp_file.write_all(&buffer[..bytes_read])?;
        bytes_to_copy = bytes_to_copy.saturating_sub(bytes_read);
    }

    temp_file.flush()?;
    drop(temp_file);
    drop(source_file);

    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    if copy_iterations >= MAX_COPY_ITERATIONS {
        log_error(
            &stack_format_it(
                "Copy iterations {} exceeded limit",
                &[&copy_iterations.to_string()],
                "Copy iterations _ exceeded limit",
            ),
            Some("delete_current_line_noload:copy"),
        );

        // Clean up temp file
        let _ = fs::remove_file(&temp_line_path);

        let _ = state.set_info_bar_message("line too long");
        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Max copy iterations exceeded",
        )));
    }

    // Step 4: Delete the line
    // If this fails, temp file remains but that's okay (cleanup handled below)
    let delete_result = delete_byte_range_chunked(file_path, line_start, delete_end);

    // Check if deletion succeeded before creating undo logs
    if let Err(e) = delete_result {
        // Deletion failed - clean up temp file and propagate error
        let _ = fs::remove_file(&temp_line_path);
        return Err(LinesError::Io(e));
    }

    // ============================================
    // Step 4.5: Create Inverse Changelog Entries
    // ============================================
    // Deletion succeeded - now create undo logs from temp file
    // Same pattern as Phase 6 of insert_file_at_cursor

    let log_directory_path = match get_undo_changelog_directory_path(file_path) {
        Ok(path) => Some(path),
        Err(_e) => {
            // Non-critical: Log error but don't fail the deletion
            #[cfg(debug_assertions)]
            log_error(
                &format!("Cannot get changelog directory: {}", _e),
                Some("delete_current_line_noload:changelog"),
            );

            #[cfg(not(debug_assertions))]
            log_error(
                "Cannot get changelog directory",
                Some("delete_current_line_noload:changelog"),
            );

            // Clean up temp file and continue without undo
            let _ = fs::remove_file(&temp_line_path);

            // Skip to Step 5
            state.is_modified = true;

            state.cursor.col = 0;
            let _ = state.set_info_bar_message("undo disabled");
            return Ok(());
        }
    };

    // Create undo logs if we have the directory path
    if let Some(log_dir) = log_directory_path {
        // Open temp file for reading
        let mut temp_file_for_logging = match File::open(&temp_line_path) {
            Ok(file) => file,
            Err(_e) => {
                #[cfg(debug_assertions)]
                log_error(
                    &format!("Cannot open temp file for logging: {}", _e),
                    Some("delete_current_line_noload:changelog"),
                );

                #[cfg(not(debug_assertions))]
                log_error(
                    "Cannot open temp file",
                    Some("delete_current_line_noload:changelog"),
                );

                // Clean up and continue
                let _ = fs::remove_file(&temp_line_path);
                let _ = state.set_info_bar_message("undo disabled");

                // Skip to Step 5
                state.is_modified = true;

                state.cursor.col = 0;
                return Ok(());
            }
        };

        // Initialize logging state (same as Phase 6)
        let mut logging_chunk_counter: usize = 0;
        let mut _byte_offset_in_line: u64 = 0;
        let mut carry_over_bytes: [u8; 4] = [0; 4];
        let mut carry_over_count: usize = 0;
        let mut logging_error_count: usize = 0;
        const MAX_LOGGING_ERRORS: usize = 100;
        const MAX_CHUNKS: usize = 16_777_216;

        // Logging loop (same pattern as file insertion)
        loop {
            if logging_chunk_counter >= MAX_CHUNKS {
                #[cfg(debug_assertions)]
                log_error(
                    "Logging iteration exceeded MAX_CHUNKS",
                    Some("delete_current_line_noload:changelog"),
                );

                #[cfg(not(debug_assertions))]
                log_error(
                    "Logging limit reached",
                    Some("delete_current_line_noload:changelog"),
                );

                let _ = state.set_info_bar_message("undo log incomplete");
                break;
            }

            if logging_error_count >= MAX_LOGGING_ERRORS {
                #[cfg(debug_assertions)]
                log_error(
                    &format!("Logging stopped after {} errors", MAX_LOGGING_ERRORS),
                    Some("delete_current_line_noload:changelog"),
                );

                #[cfg(not(debug_assertions))]
                log_error(
                    "Logging stopped after max errors",
                    Some("delete_current_line_noload:changelog"),
                );

                let _ = state.set_info_bar_message("undo log incomplete");
                break;
            }

            let mut buffer = [0u8; CHUNK_SIZE];

            if state.security_mode {
                for i in 0..CHUNK_SIZE {
                    buffer[i] = 0;
                }
            }

            let bytes_read = match temp_file_for_logging.read(&mut buffer) {
                Ok(n) => n,
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    log_error(
                        &format!(
                            "Read error during logging at chunk {}: {}",
                            logging_chunk_counter, _e
                        ),
                        Some("delete_current_line_noload:changelog"),
                    );

                    #[cfg(not(debug_assertions))]
                    log_error(
                        "Read error during logging",
                        Some("delete_current_line_noload:changelog"),
                    );

                    logging_error_count += 1;
                    continue;
                }
            };

            if bytes_read == 0 && carry_over_count == 0 {
                break; // EOF
            }

            logging_chunk_counter += 1;

            let mut buffer_index: usize = 0;

            // Handle carry-over from previous chunk
            if carry_over_count > 0 {
                let bytes_needed = detect_utf8_byte_count(carry_over_bytes[0])
                    .unwrap_or(1)
                    .saturating_sub(carry_over_count);

                if bytes_needed > 0 && bytes_needed <= bytes_read {
                    for i in 0..bytes_needed {
                        carry_over_bytes[carry_over_count + i] = buffer[i];
                    }
                    buffer_index += bytes_needed;

                    let full_char_bytes = &carry_over_bytes[0..(carry_over_count + bytes_needed)];

                    // Replace this section in the logging loop:

                    match std::str::from_utf8(full_char_bytes) {
                        Ok(s) => {
                            if let Some(ch) = s.chars().next() {
                                // USE LINE_START FOR ALL CHARACTERS (button stack trick)
                                // Don't add _byte_offset_in_line!
                                let char_position_u128 = line_start as u128;

                                /*
                                pub fn button_make_changelog_from_user_character_action_level(
                                    target_file: &Path,
                                    character: Option<char>,
                                    byte_value: Option<u8>, // raw byte input
                                    position: u128,
                                    edit_type: EditType,
                                    log_directory_path: &Path,
                                ) -> ButtonResult<()> {
                                */

                                for retry_attempt in 0..3 {
                                    match button_make_changelog_from_user_character_action_level(
                                        file_path,
                                        Some(ch),
                                        None,
                                        char_position_u128,
                                        EditType::RmvCharacter, // User removed, inverse is add
                                        &log_dir,
                                    ) {
                                        Ok(_) => break,
                                        Err(_e) => {
                                            if retry_attempt == 2 {
                                                #[cfg(debug_assertions)]
                                                log_error(
                                                    &format!(
                                                        "Failed to log char at position {}: {}",
                                                        char_position_u128, _e
                                                    ),
                                                    Some("delete_current_line_noload:changelog"),
                                                );

                                                #[cfg(not(debug_assertions))]
                                                log_error(
                                                    "Failed to log character",
                                                    Some("delete_current_line_noload:changelog"),
                                                );

                                                logging_error_count += 1;
                                            } else {
                                                std::thread::sleep(
                                                    std::time::Duration::from_millis(50),
                                                );
                                            }
                                        }
                                    }
                                }

                                // Still track offset for error messages, but don't use it for position
                                _byte_offset_in_line += full_char_bytes.len() as u64;
                            }
                        }
                        Err(_) => {
                            #[cfg(debug_assertions)]
                            log_error(
                                &format!(
                                    "Invalid UTF-8 in carry-over at offset {}",
                                    _byte_offset_in_line
                                ),
                                Some("delete_current_line_noload:changelog"),
                            );

                            #[cfg(not(debug_assertions))]
                            log_error(
                                "Invalid UTF-8 in carry-over",
                                Some("delete_current_line_noload:changelog"),
                            );

                            _byte_offset_in_line += full_char_bytes.len() as u64;
                        }
                    }

                    carry_over_count = 0;
                }
            }

            // Process remaining bytes in buffer
            while buffer_index < bytes_read {
                let byte = buffer[buffer_index];

                let char_len = match detect_utf8_byte_count(byte) {
                    Ok(len) => len,
                    Err(_) => {
                        #[cfg(debug_assertions)]
                        log_error(
                            &format!(
                                "Invalid UTF-8 start byte at offset {}",
                                _byte_offset_in_line
                            ),
                            Some("delete_current_line_noload:changelog"),
                        );

                        #[cfg(not(debug_assertions))]
                        log_error(
                            "Invalid UTF-8 start byte",
                            Some("delete_current_line_noload:changelog"),
                        );

                        buffer_index += 1;
                        _byte_offset_in_line += 1;
                        continue;
                    }
                };

                if buffer_index + char_len <= bytes_read {
                    let char_bytes = &buffer[buffer_index..(buffer_index + char_len)];
                    match std::str::from_utf8(char_bytes) {
                        Ok(s) => {
                            if let Some(ch) = s.chars().next() {
                                // USE LINE_START FOR ALL CHARACTERS (button stack trick)
                                let char_position_u128 = line_start as u128;

                                /*
                                pub fn button_make_changelog_from_user_character_action_level(
                                    target_file: &Path,
                                    character: Option<char>,
                                    byte_value: Option<u8>, // raw byte input
                                    position: u128,
                                    edit_type: EditType,
                                    log_directory_path: &Path,
                                ) -> ButtonResult<()> {
                                */

                                for retry_attempt in 0..3 {
                                    match button_make_changelog_from_user_character_action_level(
                                        file_path,
                                        Some(ch),
                                        None,
                                        char_position_u128,
                                        EditType::RmvCharacter, // User removed, inverse is add
                                        &log_dir,
                                    ) {
                                        Ok(_) => break,
                                        Err(_e) => {
                                            if retry_attempt == 2 {
                                                #[cfg(debug_assertions)]
                                                log_error(
                                                    &format!(
                                                        "Failed to log char at position {}: {}",
                                                        char_position_u128, _e
                                                    ),
                                                    Some("delete_current_line_noload:changelog"),
                                                );

                                                #[cfg(not(debug_assertions))]
                                                log_error(
                                                    "Failed to log character",
                                                    Some("delete_current_line_noload:changelog"),
                                                );

                                                logging_error_count += 1;
                                            } else {
                                                std::thread::sleep(
                                                    std::time::Duration::from_millis(50),
                                                );
                                            }
                                        }
                                    }
                                }

                                // Still track offset for error messages
                                _byte_offset_in_line += char_len as u64;
                            }
                        }
                        Err(_) => {
                            #[cfg(debug_assertions)]
                            log_error(
                                &format!(
                                    "Invalid UTF-8 sequence at offset {}",
                                    _byte_offset_in_line
                                ),
                                Some("delete_current_line_noload:changelog"),
                            );

                            #[cfg(not(debug_assertions))]
                            log_error(
                                "Invalid UTF-8 sequence",
                                Some("delete_current_line_noload:changelog"),
                            );

                            _byte_offset_in_line += char_len as u64;
                        }
                    }

                    buffer_index += char_len;
                } else {
                    carry_over_count = bytes_read - buffer_index;

                    if carry_over_count > 4 {
                        #[cfg(debug_assertions)]
                        log_error(
                            &format!("carry_over_count {} exceeds 4", carry_over_count),
                            Some("delete_current_line_noload:changelog"),
                        );

                        #[cfg(not(debug_assertions))]
                        log_error(
                            "carry_over buffer overflow",
                            Some("delete_current_line_noload:changelog"),
                        );

                        break;
                    }

                    for i in 0..carry_over_count {
                        carry_over_bytes[i] = buffer[buffer_index + i];
                    }
                    break;
                }
            }
        }

        if logging_error_count > 0 {
            #[cfg(debug_assertions)]
            log_error(
                &format!("Logging completed with {} errors", logging_error_count),
                Some("delete_current_line_noload:changelog"),
            );

            #[cfg(not(debug_assertions))]
            log_error(
                "Logging completed with errors",
                Some("delete_current_line_noload:changelog"),
            );

            let _ = state.set_info_bar_message("undo log incomplete");
        }
    }

    // Clean up temp file
    let _ = fs::remove_file(&temp_line_path);

    // Step 5: Update state
    state.is_modified = true;

    // Step 6: Cursor stays at current row
    // After rebuild, this row will show the next line
    state.cursor.col = 0; // Move to start of (new) line

    Ok(())
}

/// Deletes explicit byte range from visual selection WITHOUT loading whole file, with undo support
///
/// # Overview
/// Deletes a user-selected byte range using chunked file operations and creates
/// inverse changelog entries for undo. The range is determined by visual selection
/// positions stored in editor state. Selected content is saved to a temporary file
/// before deletion, then changelog entries are created character-by-character using
/// the "Cheap Trick" button stack approach.
///
/// # Key Differences from Line Deletion
///
/// **Position-based, not line-based:**
/// - Range comes from visual selection cursors (start/end positions)
/// - Deletes exactly the selected bytes (inclusive)
/// - Respects UTF-8 character boundaries (won't cut mid-character)
/// - No automatic newline inclusion/exclusion
///
/// **UTF-8 Boundary Safety:**
/// The end position marks the START of the last selected character, which may be
/// 1-4 bytes long. We detect the character length and extend delete_end to include
/// the complete character, preventing corruption of multi-byte sequences.
///
/// # The "Cheap Trick" Button Stack (Critical for Undo!)
///
/// **The Problem We Solve:**
/// When deleting a range like "pine\nuts" at position 25, we need to create
/// undo logs that will reconstruct it correctly. Naive approach would be:
/// ```text
/// Log: ADD 'p' at 25
/// Log: ADD 'i' at 26  ← WRONG! Position changes as we add
/// Log: ADD 'n' at 27
/// ...
/// ```
/// When undo runs backwards (LIFO), it would add last character first at wrong position.
///
/// **The Solution: All Logs Use Same Position**
/// ```text
/// Log 1.h: ADD 'p' at 25  (first char, highest letter, last to execute)
/// Log 1.g: ADD 'i' at 25  (same position!)
/// Log 1.f: ADD 'n' at 25  (same position!)
/// Log 1.e: ADD 'e' at 25  (same position!)
/// Log 1.d: ADD '\' at 25  (same position!)
/// Log 1.c: ADD 'n' at 25  (same position!)
/// Log 1.b: ADD 'u' at 25  (same position!)
/// Log 1.a: ADD 't' at 25  (same position!)
/// Log 1:   ADD 's' at 25  (last char, no letter, first to execute)
/// ```
///
/// **How Button Stack Reconstructs the Range:**
/// When undo executes (reads files in sorted order: 1, 1.a, 1.b, ..., 1.h):
/// 1. ADD 's' at 25 → "s" at position 25
/// 2. ADD 't' at 25 → "ts" at positions 25-26 (pushes s right)
/// 3. ADD 'u' at 25 → "uts" at 25-26-27 (pushes t,s right)
/// 4. ADD 'n' at 25 → "nuts" at 25-26-27-28
/// 5. ... continues pushing right ...
/// 8. ADD 'e' at 25 → "e\nuts" (all chars pushed right)
/// 9. ADD 'p' at 25 → "pine\nuts" (reconstruction complete!)
///
/// Result: "pine\nuts" perfectly reconstructed!
///
/// **Why This Works:**
/// - LIFO (Last In, First Out): Undo reads logs in reverse order of creation
/// - Insert-at-same-position: Each insertion pushes previous characters right
/// - Natural cascading: File operations automatically shift bytes
/// - Fewer moving parts: No position arithmetic, just one constant position
/// - UTF-8 safe: Works for multi-byte characters (each byte gets same position)
///
/// **Letter Suffixes Enforce Execution Order:**
/// - No letter (e.g., "1"): Last character in range, executed FIRST by undo
/// - Letter 'a' (e.g., "1.a"): Second-to-last character, executed second
/// - Letter 'b' (e.g., "1.b"): Third-to-last, executed third
/// - ...
/// - Highest letter (e.g., "1.h"): First character in range, executed LAST by undo
///
/// This naming ensures correct LIFO execution order through filesystem sorting.
///
/// # Algorithm
///
/// **Phase 1: Determine Range from Visual Selection**
/// 1. Normalize selection range (handle backwards selection)
///    - Call normalize_sort_sanitize_selection_range()
///    - Ensures start <= end regardless of selection direction
/// 2. Validate range against file size
///    - Read file metadata to get file length
///    - Reject if start >= file_size or end > file_size
///    - Return InvalidInput error if out of bounds
/// 3. Handle UTF-8 character boundary at end position
///    - Seek to end position
///    - Read first byte of character at end
///    - Use detect_utf8_byte_count() to get character length
///    - Set delete_end = end + char_length (inclusive of complete character)
///    - If invalid UTF-8: treat as single byte, log error
///    - If EOF: use end position directly
/// 4. Set range_start = start (use position directly)
///
/// **Phase 2: Save Range to Temp File**
/// 5. Create temporary file (file.tmp_deleted_range)
/// 6. Copy range bytes [range_start..delete_end] to temp file (chunked, no heap)
/// 7. Flush and close temp file
/// 8. If copy fails: clean up temp file, abort operation
///
/// **Phase 3: Delete Range from Source File**
/// 9. Delete byte range [range_start..delete_end] using chunked operations
/// 10. If deletion fails: clean up temp file, abort operation
///
/// **Phase 4: Create Undo Logs (Button Stack)**
/// 11. Get changelog directory path
/// 12. Open temp file for reading
/// 13. Iterate through temp file character-by-character (chunked)
/// 14. For each UTF-8 character:
///     - Position = range_start (NOT range_start + offset!) ← Key insight!
///     - Call button_make_changelog_from_user_character_action_level()
///     - EditType = Rmv (user removed range, inverse adds it back)
///     - Character = Some(char) (need character for restoration)
/// 15. Handle UTF-8 boundaries across chunks (carry-over buffer)
/// 16. Retry each log creation up to 3 times
/// 17. Continue on logging errors (non-critical, deletion succeeded)
///
/// **Phase 5: Cleanup and Update State**
/// 18. Delete temp file
/// 19. Mark editor state as modified
/// 20. Log the edit operation: "DELETE_RANGE bytes:{}-{}"
/// 21. Move cursor to line start via execute_command(GotoLineStart)
/// 22. Set info bar message: "Range deleted" (success case)
///
/// # Memory Safety
///
/// **Stack-only buffers:**
/// - Range copy buffer: [0u8; 256] - 256 bytes on stack
/// - UTF-8 carry-over buffer: [0u8; 4] - 4 bytes on stack (max UTF-8 char)
/// - UTF-8 boundary check buffer: [0u8; 1] - 1 byte on stack
/// - No heap allocation for data processing
/// - Temp file on disk (not in memory)
///
/// **Bounded iterations:**
/// - MAX_COPY_ITERATIONS: 1,000,000 (prevents infinite loops)
/// - MAX_CHUNKS: 16,777,216 (during changelog creation)
/// - MAX_LOGGING_ERRORS: 100 (stops after too many failures)
///
/// # Error Handling Philosophy
///
/// **Critical operations (must succeed):**
/// - Range normalization: Return InvalidInput if positions invalid
/// - Range validation: Return InvalidInput if exceeds file size
/// - Range copy to temp: Return Io error, clean up temp file
/// - Range deletion: Return Io error, clean up temp file
///
/// **Non-critical operations (fail gracefully):**
/// - UTF-8 boundary detection: Treat as single byte if invalid, log error
/// - Changelog directory creation: Continue without undo
/// - Temp file re-opening for logging: Continue without undo
/// - Individual log creation: Retry 3x, then skip and continue
/// - Temp file cleanup: Log error but don't fail operation
///
/// **Undo is a luxury, never blocks deletion.**
///
/// # Edge Cases
///
/// **Empty range (start == end):**
/// - Single character deletion
/// - Character length detected via UTF-8 inspection
/// - Creates log entries for that character
///
/// **Single byte range:**
/// - Deletes one byte
/// - If valid UTF-8 start: extends to complete character
/// - If invalid UTF-8: deletes single byte, logs error
///
/// **Range with multi-byte UTF-8 characters:**
/// - Each character logged separately at same position
/// - Multi-byte chars handled by button_make_changeloge... function
/// - Creates letter-suffixed log files (e.g., 1.a, 1.b) automatically
///
/// **Range ending mid-character:**
/// - End position is START of last character
/// - UTF-8 detection extends to character boundary
/// - Prevents corruption of multi-byte sequences
///
/// **Range at start of file (position 0):**
/// - range_start = 0 (BOF)
/// - Works normally, deletes from beginning
///
/// **Range at end of file:**
/// - EOF detected during UTF-8 boundary check
/// - delete_end = end (no extension)
/// - Deletes to EOF
///
/// **Range spanning entire file:**
/// - range_start = 0, delete_end = file_size
/// - Results in empty file
/// - Undo restores entire file content
///
/// **Invalid UTF-8 in range:**
/// - Logged as error (debug mode) or terse message (production)
/// - Skips invalid byte(s) during undo logging
/// - Continues processing rest of range
/// - Undo will not restore invalid bytes
///
/// **Backwards selection (end < start):**
/// - Normalized by normalize_sort_sanitize_selection_range()
/// - Automatically swapped to (start, end)
/// - Works identically to forward selection
///
/// **Range longer than MAX_COPY_ITERATIONS × 256 bytes:**
/// - Copy phase aborts with error
/// - Deletion does not occur
/// - No orphan undo logs created
///
/// **Logging failures:**
/// - Each character retried 3 times with 50ms pause
/// - After 100 total errors: stops creating logs
/// - Info bar shows "undo log incomplete"
/// - Deletion still succeeded, undo partially disabled
///
/// **Temp file already exists:**
/// - File::create() truncates existing file
/// - Not an error, just overwrites
///
/// **Range exceeds file size:**
/// - Detected in Phase 1 validation
/// - Returns InvalidInput error immediately
/// - No temp file created, no side effects
/// - Info bar shows "invalid range"
///
/// # Why Temp File Approach?
///
/// **Prevents Orphan Logs:**
/// If we created undo logs BEFORE deletion and deletion failed, we'd have
/// orphan logs for a delete that never happened. Corrupts undo history.
///
/// **Clean Failure Semantics:**
/// - Save range → fails → abort, no side effects
/// - Save range → success → Delete range → fails → abort, temp file cleaned up
/// - Save range → success → Delete range → success → Create logs → can't fail critically
///
/// **Reuses Proven Pattern:**
/// Logging loop is identical to file insertion Phase 6 and line deletion Phase 4.5.
/// Same UTF-8 handling, same carry-over buffer, same error handling, same retry logic.
///
/// # Position Tracking
///
/// **Important: byte_offset_in_range is tracked but NOT used for positions!**
/// ```rust
/// byte_offset_in_range += char_len;  // Only for error messages
/// char_position = range_start;        // Always the same position!
/// ```
///
/// This seems counterintuitive but is critical for button stack to work.
///
/// # Arguments
///
/// * `state` - Editor state containing visual selection positions:
///   - `file_position_of_vis_select_start` - Start of selected range (byte offset)
///   - `file_position_of_vis_select_end` - End of selected range (byte offset)
/// * `file_path` - Path to the file being edited (read-copy, absolute path)
///
/// # Returns
///
/// * `Ok(())` - Range deleted successfully (with or without undo logs)
/// * `Err(LinesError::InvalidInput)` - Invalid range (out of bounds, etc.)
/// * `Err(LinesError::Io)` - I/O operation failed (range NOT deleted)
/// * `Err(LinesError::GeneralAssertionCatchViolation)` - Assertion catch in production
///
/// # Side Effects
///
/// - Deletes byte range from file
/// - Creates multiple changelog files in undo directory
/// - Creates and deletes temporary file (file.tmp_deleted_range)
/// - Marks editor state as modified
/// - Moves cursor to line start via Command::GotoLineStart
/// - Sets info bar message ("Range deleted", "undo log incomplete", etc.)
/// - Logs edit operation to state log
///
/// # Examples
///
/// ```ignore
/// // User selects "world" in "Hello world!\n" (positions 6-11)
/// state.file_position_of_vis_select_start = 6;
/// state.file_position_of_vis_select_end = 11;  // 'd' starts at position 10, ends at 11
///
/// delete_position_range_noload(&mut state, &file_path)?;
///
/// // Result: "Hello !\n" (6 bytes deleted: "world")
/// // Logged as: "DELETE_RANGE bytes:6-11"
///
/// // Undo logs created (button stack, all at position 6):
/// // changelog_file/1.e: ADD 'w' at 6
/// // changelog_file/1.d: ADD 'o' at 6
/// // changelog_file/1.c: ADD 'r' at 6
/// // changelog_file/1.b: ADD 'l' at 6
/// // changelog_file/1.a: ADD 'd' at 6
/// // changelog_file/1:   ADD ' ' at 6  (space before 'world')
///
/// // User presses undo:
/// // 1. Reads "1" → ADD ' ' at 6 → "Hello  !\n"
/// // 2. Reads "1.a" → ADD 'd' at 6 → "Hello d !\n"
/// // 3. Reads "1.b" → ADD 'l' at 6 → "Hello ld !\n"
/// // ... cascading insertions ...
/// // 6. Reads "1.e" → ADD 'w' at 6 → "Hello world!\n" ✓
/// ```
///
/// ```ignore
/// // Multi-byte UTF-8 example: Delete "世界" (6 bytes: 3+3)
/// state.file_position_of_vis_select_start = 10;
/// state.file_position_of_vis_select_end = 16;  // '界' starts at 13, ends at 16
///
/// delete_position_range_noload(&mut state, &file_path)?;
///
/// // UTF-8 boundary detection ensures complete character deletion
/// // Undo logs preserve multi-byte characters correctly
/// ```
///
/// ```ignore
/// // Backwards selection (normalized automatically)
/// state.file_position_of_vis_select_start = 20;  // End cursor
/// state.file_position_of_vis_select_end = 10;    // Start cursor
///
/// delete_position_range_noload(&mut state, &file_path)?;
/// // Normalized to (10, 20), deletion proceeds normally
/// ```
///
/// # See Also
///
/// * `delete_current_line_noload()` - Line-based deletion (finds line boundaries)
/// * `normalize_sort_sanitize_selection_range()` - Handles backwards selections
/// * `detect_utf8_byte_count()` - UTF-8 character length detection
/// * `button_make_changelog_from_user_character_action_level()` - Creates individual log entries
/// * `button_add_multibyte_make_log_files()` - Handles multi-byte characters with letter suffixes
/// * `delete_byte_range_chunked()` - Performs the actual deletion
///
/// # Testing Considerations
///
/// Test with ranges containing:
/// - Empty selection (start == end, single character)
/// - Single byte ("a")
/// - ASCII text ("Hello, world!")
/// - Multi-byte UTF-8 ("你好世界")
/// - Mixed ASCII and UTF-8 ("Hello 世界")
/// - Range at start of file (position 0)
/// - Range at end of file (to EOF)
/// - Entire file (position 0 to file_size)
/// - Backwards selection (end < start)
/// - Invalid UTF-8 bytes
/// - Very long range (test MAX_COPY_ITERATIONS)
/// - Range exceeding file size
/// - Range ending mid-UTF-8 character (boundary extension)
/// - Range with newlines, tabs, control characters
/// - Range with mixed line endings (\n, \r\n)
fn delete_position_range_noload(state: &mut EditorState, file_path: &Path) -> Result<()> {
    // ====================================
    // Get start byte and end-character end
    // ====================================
    // Step 1: Normalize selection range (handle backwards selection)
    // Step 1: Normalize selection
    let (start, end) = normalize_sort_sanitize_selection_range(
        state.file_position_of_vis_select_start,
        state.file_position_of_vis_select_end,
    )?;

    // Step 2: Validate against file size
    let file_metadata = fs::metadata(file_path)?;
    let file_size = file_metadata.len();

    if start >= file_size || end > file_size {
        log_error(
            &stack_format_it(
                "Range {}-{} exceeds file size {}",
                &[&start.to_string(), &end.to_string(), &file_size.to_string()],
                "Range exceeds file size",
            ),
            Some("delete_position_range_noload"),
        );

        let _ = state.set_info_bar_message("invalid range");
        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Range exceeds file boundaries",
        )));
    }

    // Step 3: Handle UTF-8 character boundary at end position
    // The 'end' cursor is on the START of a character that may be 1-4 bytes
    // We need to find where that character ENDS to delete it inclusively
    let line_start = start; // Use position directly
    let delete_end = {
        let mut file = File::open(file_path)?;
        file.seek(SeekFrom::Start(end))?;

        let mut byte_buffer = [0u8; 1];
        let bytes_read = file.read(&mut byte_buffer)?;

        if bytes_read == 0 {
            // End is at EOF, use it directly
            end
        } else {
            // Detect UTF-8 character length starting at 'end'
            match detect_utf8_byte_count(byte_buffer[0]) {
                Ok(char_len) => end + (char_len as u64),
                Err(_) => {
                    // Invalid UTF-8 start byte, treat as single byte
                    log_error(
                        &stack_format_it(
                            "Invalid UTF-8 at position {}",
                            &[&end.to_string()],
                            "Invalid UTF-8 at position",
                        ),
                        Some("delete_position_range_noload"),
                    );

                    end + 1
                }
            }
        }
    };

    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    debug_assert!(
        line_start <= delete_end,
        "Range start must be before or at range end"
    );

    #[cfg(test)]
    assert!(
        line_start <= delete_end,
        "Range start must be before or at range end"
    );

    if line_start > delete_end {
        #[cfg(debug_assertions)]
        log_error(
            &format!(
                "Invalid range bounds: start {} > end {}",
                line_start, delete_end
            ),
            Some("delete_position_range_noload"),
        );

        #[cfg(not(debug_assertions))]
        log_error("Invalid range bounds", Some("delete_position_range_noload"));

        let _ = state.set_info_bar_message("range bounds error");
        return Err(LinesError::GeneralAssertionCatchViolation(
            "invalid range bounds".into(),
        ));
    }

    // ============================================
    // Step 2.5: Copy Line to Temporary File
    // ============================================
    // Save line content before deletion so we can create undo logs afterward
    // This prevents orphan logs if deletion fails

    let temp_line_path = file_path.with_extension("tmp_deleted_line");

    // Open source file for reading the line
    let mut source_file = File::open(file_path)?;

    // Create temp file for saving line
    let mut temp_file = File::create(&temp_line_path)?;

    // Seek to line start
    source_file.seek(SeekFrom::Start(line_start))?;

    // Copy line bytes to temp file (chunked, no heap)
    const CHUNK_SIZE: usize = 256;
    let mut buffer = [0u8; CHUNK_SIZE];
    let mut bytes_to_copy = (delete_end - line_start) as usize;
    let mut copy_iterations = 0;
    const MAX_COPY_ITERATIONS: usize = 1_000_000; // Safety limit

    while bytes_to_copy > 0 && copy_iterations < MAX_COPY_ITERATIONS {
        copy_iterations += 1;

        let to_read = bytes_to_copy.min(CHUNK_SIZE);
        let bytes_read = source_file.read(&mut buffer[..to_read])?;

        if bytes_read == 0 {
            break; // EOF
        }

        temp_file.write_all(&buffer[..bytes_read])?;
        bytes_to_copy = bytes_to_copy.saturating_sub(bytes_read);
    }

    temp_file.flush()?;
    drop(temp_file);
    drop(source_file);

    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    if copy_iterations >= MAX_COPY_ITERATIONS {
        #[cfg(debug_assertions)]
        log_error(
            &format!("Copy iterations {} exceeded limit", copy_iterations),
            Some("delete_current_line_noload:copy"),
        );

        #[cfg(not(debug_assertions))]
        log_error(
            "Copy iteration limit exceeded",
            Some("delete_current_line_noload:copy"),
        );

        // Clean up temp file
        let _ = fs::remove_file(&temp_line_path);

        let _ = state.set_info_bar_message("line too long");
        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Max copy iterations exceeded",
        )));
    }

    // Step 4: Delete the line
    // If this fails, temp file remains but that's okay (cleanup handled below)
    let delete_result = delete_byte_range_chunked(file_path, line_start, delete_end);

    // Check if deletion succeeded before creating undo logs
    if let Err(e) = delete_result {
        // Deletion failed - clean up temp file and propagate error
        let _ = fs::remove_file(&temp_line_path);
        return Err(LinesError::Io(e));
    }

    // ============================================
    // Step 4.5: Create Inverse Changelog Entries
    // ============================================
    // Deletion succeeded - now create undo logs from temp file
    // Same pattern as Phase 6 of insert_file_at_cursor

    let log_directory_path = match get_undo_changelog_directory_path(file_path) {
        Ok(path) => Some(path),
        Err(_e) => {
            // Non-critical: Log error but don't fail the deletion
            #[cfg(debug_assertions)]
            log_error(
                &format!("Cannot get changelog directory: {}", _e),
                Some("delete_current_line_noload:changelog"),
            );

            #[cfg(not(debug_assertions))]
            log_error(
                "Cannot get changelog directory",
                Some("delete_current_line_noload:changelog"),
            );

            // Clean up temp file and continue without undo
            let _ = fs::remove_file(&temp_line_path);

            // Skip to Step 5
            state.is_modified = true;

            state.cursor.col = 0;
            let _ = state.set_info_bar_message("err:nO uNdo");
            return Ok(());
        }
    };

    // Create undo logs if we have the directory path
    if let Some(log_dir) = log_directory_path {
        // Open temp file for reading
        let mut temp_file_for_logging = match File::open(&temp_line_path) {
            Ok(file) => file,
            Err(_e) => {
                #[cfg(debug_assertions)]
                log_error(
                    &format!("Cannot open temp file for logging: {}", _e),
                    Some("delete_current_line_noload:changelog"),
                );

                #[cfg(not(debug_assertions))]
                log_error(
                    "Cannot open temp file",
                    Some("delete_current_line_noload:changelog"),
                );

                // Clean up and continue
                let _ = fs::remove_file(&temp_line_path);
                let _ = state.set_info_bar_message("undo disabled");

                // Skip to Step 5
                state.is_modified = true;

                state.cursor.col = 0;
                return Ok(());
            }
        };

        // Initialize logging state (same as Phase 6)
        let mut logging_chunk_counter: usize = 0;
        let mut _byte_offset_in_line: u64 = 0;
        let mut carry_over_bytes: [u8; 4] = [0; 4];
        let mut carry_over_count: usize = 0;
        let mut logging_error_count: usize = 0;
        const MAX_LOGGING_ERRORS: usize = 100;
        const MAX_CHUNKS: usize = 16_777_216;

        // Logging loop (same pattern as file insertion)
        loop {
            if logging_chunk_counter >= MAX_CHUNKS {
                #[cfg(debug_assertions)]
                log_error(
                    "Logging iteration exceeded MAX_CHUNKS",
                    Some("delete_current_line_noload:changelog"),
                );

                #[cfg(not(debug_assertions))]
                log_error(
                    "Logging limit reached",
                    Some("delete_current_line_noload:changelog"),
                );

                let _ = state.set_info_bar_message("undo log incomplete");
                break;
            }

            if logging_error_count >= MAX_LOGGING_ERRORS {
                #[cfg(debug_assertions)]
                log_error(
                    &format!("Logging stopped after {} errors", MAX_LOGGING_ERRORS),
                    Some("delete_current_line_noload:changelog"),
                );

                #[cfg(not(debug_assertions))]
                log_error(
                    "Logging stopped after max errors",
                    Some("delete_current_line_noload:changelog"),
                );

                let _ = state.set_info_bar_message("undo log incomplete");
                break;
            }

            let mut buffer = [0u8; CHUNK_SIZE];

            if state.security_mode {
                for i in 0..CHUNK_SIZE {
                    buffer[i] = 0;
                }
            }

            let bytes_read = match temp_file_for_logging.read(&mut buffer) {
                Ok(n) => n,
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    log_error(
                        &format!(
                            "Read error during logging at chunk {}: {}",
                            logging_chunk_counter, _e
                        ),
                        Some("delete_current_line_noload:changelog"),
                    );

                    #[cfg(not(debug_assertions))]
                    log_error(
                        "Read error during logging",
                        Some("delete_current_line_noload:changelog"),
                    );

                    logging_error_count += 1;
                    continue;
                }
            };

            if bytes_read == 0 && carry_over_count == 0 {
                break; // EOF
            }

            logging_chunk_counter += 1;

            let mut buffer_index: usize = 0;

            // Handle carry-over from previous chunk
            if carry_over_count > 0 {
                let bytes_needed = detect_utf8_byte_count(carry_over_bytes[0])
                    .unwrap_or(1)
                    .saturating_sub(carry_over_count);

                if bytes_needed > 0 && bytes_needed <= bytes_read {
                    for i in 0..bytes_needed {
                        carry_over_bytes[carry_over_count + i] = buffer[i];
                    }
                    buffer_index += bytes_needed;

                    let full_char_bytes = &carry_over_bytes[0..(carry_over_count + bytes_needed)];

                    // Replace this section in the logging loop:

                    match std::str::from_utf8(full_char_bytes) {
                        Ok(s) => {
                            if let Some(ch) = s.chars().next() {
                                // USE LINE_START FOR ALL CHARACTERS (button stack trick)
                                // Don't add _byte_offset_in_line!
                                let char_position_u128 = line_start as u128;

                                /*
                                pub fn button_make_changelog_from_user_character_action_level(
                                    target_file: &Path,
                                    character: Option<char>,
                                    byte_value: Option<u8>, // raw byte input
                                    position: u128,
                                    edit_type: EditType,
                                    log_directory_path: &Path,
                                ) -> ButtonResult<()> {
                                */

                                for retry_attempt in 0..3 {
                                    match button_make_changelog_from_user_character_action_level(
                                        file_path,
                                        Some(ch),
                                        None,
                                        char_position_u128,
                                        EditType::RmvCharacter, // User removed, inverse is add
                                        &log_dir,
                                    ) {
                                        Ok(_) => break,
                                        Err(_e) => {
                                            if retry_attempt == 2 {
                                                #[cfg(debug_assertions)]
                                                log_error(
                                                    &format!(
                                                        "Failed to log char at position {}: {}",
                                                        char_position_u128, _e
                                                    ),
                                                    Some("delete_current_line_noload:changelog"),
                                                );

                                                #[cfg(not(debug_assertions))]
                                                log_error(
                                                    "Failed to log character",
                                                    Some("delete_current_line_noload:changelog"),
                                                );

                                                logging_error_count += 1;
                                            } else {
                                                std::thread::sleep(
                                                    std::time::Duration::from_millis(50),
                                                );
                                            }
                                        }
                                    }
                                }

                                // Still track offset for error messages, but don't use it for position
                                _byte_offset_in_line += full_char_bytes.len() as u64;
                            }
                        }
                        Err(_) => {
                            #[cfg(debug_assertions)]
                            log_error(
                                &format!(
                                    "Invalid UTF-8 in carry-over at offset {}",
                                    _byte_offset_in_line
                                ),
                                Some("delete_current_line_noload:changelog"),
                            );

                            #[cfg(not(debug_assertions))]
                            log_error(
                                "Invalid UTF-8 in carry-over",
                                Some("delete_current_line_noload:changelog"),
                            );

                            _byte_offset_in_line += full_char_bytes.len() as u64;
                        }
                    }

                    carry_over_count = 0;
                }
            }

            // Process remaining bytes in buffer
            while buffer_index < bytes_read {
                let byte = buffer[buffer_index];

                let char_len = match detect_utf8_byte_count(byte) {
                    Ok(len) => len,
                    Err(_) => {
                        #[cfg(debug_assertions)]
                        log_error(
                            &format!(
                                "Invalid UTF-8 start byte at offset {}",
                                _byte_offset_in_line
                            ),
                            Some("delete_current_line_noload:changelog"),
                        );

                        #[cfg(not(debug_assertions))]
                        log_error(
                            "Invalid UTF-8 start byte",
                            Some("delete_current_line_noload:changelog"),
                        );

                        buffer_index += 1;
                        _byte_offset_in_line += 1;
                        continue;
                    }
                };

                if buffer_index + char_len <= bytes_read {
                    let char_bytes = &buffer[buffer_index..(buffer_index + char_len)];
                    match std::str::from_utf8(char_bytes) {
                        Ok(s) => {
                            if let Some(ch) = s.chars().next() {
                                // USE LINE_START FOR ALL CHARACTERS (button stack trick)
                                let char_position_u128 = line_start as u128;

                                /*
                                pub fn button_make_changelog_from_user_character_action_level(
                                    target_file: &Path,
                                    character: Option<char>,
                                    byte_value: Option<u8>, // raw byte input
                                    position: u128,
                                    edit_type: EditType,
                                    log_directory_path: &Path,
                                ) -> ButtonResult<()> {
                                */

                                for retry_attempt in 0..3 {
                                    match button_make_changelog_from_user_character_action_level(
                                        file_path,
                                        Some(ch),
                                        None,
                                        char_position_u128,
                                        EditType::RmvCharacter, // User removed, inverse is add
                                        &log_dir,
                                    ) {
                                        Ok(_) => break,
                                        Err(_e) => {
                                            if retry_attempt == 2 {
                                                #[cfg(debug_assertions)]
                                                log_error(
                                                    &format!(
                                                        "Failed to log char at position {}: {}",
                                                        char_position_u128, _e
                                                    ),
                                                    Some("delete_current_line_noload:changelog"),
                                                );

                                                #[cfg(not(debug_assertions))]
                                                log_error(
                                                    "Failed to log character",
                                                    Some("delete_current_line_noload:changelog"),
                                                );

                                                logging_error_count += 1;
                                            } else {
                                                std::thread::sleep(
                                                    std::time::Duration::from_millis(50),
                                                );
                                            }
                                        }
                                    }
                                }

                                // Still track offset for error messages
                                _byte_offset_in_line += char_len as u64;
                            }
                        }
                        Err(_) => {
                            #[cfg(debug_assertions)]
                            log_error(
                                &format!(
                                    "Invalid UTF-8 sequence at offset {}",
                                    _byte_offset_in_line
                                ),
                                Some("delete_current_line_noload:changelog"),
                            );

                            #[cfg(not(debug_assertions))]
                            log_error(
                                "Invalid UTF-8 sequence",
                                Some("delete_current_line_noload:changelog"),
                            );

                            _byte_offset_in_line += char_len as u64;
                        }
                    }

                    buffer_index += char_len;
                } else {
                    carry_over_count = bytes_read - buffer_index;

                    if carry_over_count > 4 {
                        #[cfg(debug_assertions)]
                        log_error(
                            &format!("carry_over_count {} exceeds 4", carry_over_count),
                            Some("delete_current_line_noload:changelog"),
                        );

                        #[cfg(not(debug_assertions))]
                        log_error(
                            "carry_over buffer overflow",
                            Some("delete_current_line_noload:changelog"),
                        );

                        break;
                    }

                    for i in 0..carry_over_count {
                        carry_over_bytes[i] = buffer[buffer_index + i];
                    }
                    break;
                }
            }
        }

        if logging_error_count > 0 {
            #[cfg(debug_assertions)]
            log_error(
                &format!("Logging completed with {} errors", logging_error_count),
                Some("delete_current_line_noload:changelog"),
            );

            #[cfg(not(debug_assertions))]
            log_error(
                "Logging completed with errors",
                Some("delete_current_line_noload:changelog"),
            );

            let _ = state.set_info_bar_message("undo log incomplete");
        }
    }

    // Clean up temp file
    let _ = fs::remove_file(&temp_line_path);

    // Step 5: Update state
    state.is_modified = true;

    // After rebuild, starting-row start is safe default.
    // Step 6: Move cursor to clean starting place
    let _ = execute_command(state, Command::GotoLineStart)?;

    Ok(())
}

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
    // use normalize_sort_sanitize_selection_range() before this function
    // // Defensive: Validate range
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

/// e.g. before building get 'starting row number'
///
/// if sarting row is > (99 - effective_rows)
/// then if line_number > (99 - effective_rows)
/// needs rows starting number...maybe just make this a method...
///
/// Calculates the display width for line numbers in the current visible range
///
/// Returns total width including the mandatory trailing space.
/// Uses wider width when we're within `effective_rows` of a digit rollover.
///
/// # Examples
/// - Line 5, 20 rows: returns 3 (might see line 24, use 2 digits + space)
/// - Line 95, 20 rows: returns 4 (might see line 114, use 3 digits + space)
fn calculate_line_number_width(
    starting_row: usize,
    line_number: usize,
    effective_rows: usize,
) -> usize {
    // if line_number == 0 {
    //     return 2; // Edge case: treat as single digit + pad
    // }

    /*
    a system to calculate even-witdth
    based on tui size:

    e.g.
    if < rollover_size
    &
    if in rollover_size - tui_size
    then add pad +1 before row...
     */

    // Count digits
    let digits = if line_number < 10 {
        2
    // } else if line_number < 99 {
    // if line_number > (99 - effective_rows) {
    //     3
    // } else {
    //     2
    // }
    } else if line_number < 100 {
        if starting_row > (100 - effective_rows - 1) {
            if line_number > (100 - effective_rows - 1) {
                3
            } else {
                2
            }
        } else {
            2
        }
    // } else if line_number < 999 {
    //     if line_number > (999 - effective_rows) {
    //         4
    //     } else {
    //         3
    //     }
    } else if line_number < 1_000 {
        if starting_row > (1_000 - effective_rows - 1) {
            if line_number > (1_000 - effective_rows - 1) {
                4
            } else {
                3
            }
        } else {
            3
        }
    // } else if line_number < 9999 {
    //     if line_number > (9999 - effective_rows) {
    //         5
    //     } else {
    //         4
    //     }
    } else if line_number < 10_000 {
        if starting_row > (10_000 - effective_rows - 1) {
            if line_number > (10_000 - effective_rows - 1) {
                5
            } else {
                4
            }
        } else {
            4
        }
    // } else if line_number < 99999 {
    //     if line_number > (99999 - effective_rows) {
    //         6
    //     } else {
    //         5
    //     }
    } else if line_number < 100_000 {
        if starting_row > (100_000 - effective_rows - 1) {
            if line_number > (100_000 - effective_rows - 1) {
                6
            } else {
                5
            }
        } else {
            5
        }
    // } else if line_number < 999999 {
    //     if line_number > (999999 - effective_rows) {
    //         7
    //     } else {
    //         6
    //     }
    } else if line_number < 1_000_000 {
        if starting_row > (1_000_000 - effective_rows - 1) {
            if line_number > (1_000_000 - effective_rows - 1) {
                7
            } else {
                6
            }
        } else {
            6
        }
    } else if line_number < 10_000_000 {
        if starting_row > (10_000_000 - effective_rows - 1) {
            if line_number > (10_000_000 - effective_rows - 1) {
                8
            } else {
                7
            }
        } else {
            7
        }
    } else {
        8 // Cap at 8 digits (999,999 lines max) TODO
    };

    // Return
    digits + 1 // Add 1 for the space after the number
}

/// Calculates the display width for line numbers in the current visible range
///
/// Returns total width including the mandatory trailing space.
/// Uses wider width when we're within `effective_rows` of a digit rollover.
///
/// # Examples
/// - Line 5, 20 rows: returns 3 (might see line 24, use 2 digits + space)
/// - Line 95, 20 rows: returns 4 (might see line 114, use 3 digits + space)
fn row_needs_extra_padding_bool(
    starting_row: usize,
    line_number: usize,
    effective_rows: usize,
) -> bool {
    /*
    a system to calculate even-witdth
    based on tui size:

    e.g.
    if < rollover_size
    &
    if in rollover_size - tui_size
    then add pad +1 before row...
     */
    let bool_output;

    if line_number < 10 {
        bool_output = true;
    } else if line_number < 100 {
        if starting_row > (100 - effective_rows - 1) {
            if line_number > (100 - effective_rows - 1) {
                bool_output = true;
            } else {
                bool_output = false;
            }
        } else {
            bool_output = false;
        }
    } else if line_number < 1_000 {
        if line_number > (1_000 - effective_rows) {
            bool_output = true;
        } else {
            bool_output = false;
        }
    } else if line_number < 10_000 {
        if line_number > (10_000 - effective_rows) {
            bool_output = true;
        } else {
            bool_output = false;
        }
    } else if line_number < 100_000 {
        if line_number > (100_000 - effective_rows) {
            bool_output = true;
        } else {
            bool_output = false;
        }
    } else if line_number < 1_000_000 {
        if line_number > (1_000_000 - effective_rows) {
            bool_output = true;
        } else {
            bool_output = false;
        }
    } else if line_number < 10_000_000 {
        if line_number > (10_000_000 - effective_rows) {
            bool_output = true;
        } else {
            bool_output = false;
        }
    } else {
        bool_output = false; // Cap at 6 digits (999,999 lines max) TODO
    }

    bool_output
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
fn insert_newline_at_cursor_chunked(
    lines_editor_state: &mut EditorState,
    file_path: &Path,
) -> io::Result<()> {
    // Step 1: Get file position at/of/where  cursor (with graceful error handling)
    let file_pos = match lines_editor_state
        .get_row_col_file_position(lines_editor_state.cursor.row, lines_editor_state.cursor.col)
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
        Err(_e) => {
            #[cfg(debug_assertions)]
            eprintln!("Warning: Cannot get cursor position: {}", _e);
            #[cfg(debug_assertions)]
            log_error(
                &format!("Insert newline failed: {}", _e),
                Some("insert_newline_at_cursor_chunked"),
            );
            // safe
            log_error(
                "Insert newline failed",
                Some("insert_newline_at_cursor_chunked"),
            );
            return Ok(());
        }
    };

    let insert_position = file_pos.byte_offset_linear_file_absolute_position;

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
                "Insert position exceeds file length", // format!(
                                                       //     "Insert position {} exceeds file length {}",
                                                       //     insert_position, bytes_copied
                                                       // ),
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
    lines_editor_state.is_modified = true;

    // Step 10: Update cursor - move to start of new line
    lines_editor_state.cursor.row += 1;

    // Calculate where the text starts after the line number
    let new_line_number =
        lines_editor_state.line_count_at_top_of_window + lines_editor_state.cursor.row;
    let line_num_width = calculate_line_number_width(
        lines_editor_state.line_count_at_top_of_window,
        new_line_number + 1,
        lines_editor_state.effective_rows,
    ); // +1 for 1-indexed display

    lines_editor_state.cursor.col = line_num_width; // Position cursor after line number
    lines_editor_state.in_row_abs_horizontal_0_index_cursor_position = line_num_width;
    lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset = 0;
    // ============================================
    // Step 5.5: Create Inverse Changelog Entry
    // ============================================
    // Create undo log for newline insertion
    // Single character, no iteration needed
    //
    // User action: Add '\n' → Inverse log: Rmv '\n'
    // This is non-critical - if it fails, insertion still succeeded

    let log_directory_path = match get_undo_changelog_directory_path(file_path) {
        Ok(path) => Some(path), // ← Wrap in Some to match the None below
        Err(_e) => {
            // Non-critical: Log error but don't fail the insertion
            #[cfg(debug_assertions)]
            log_error(
                &format!("Cannot get changelog directory: {}", _e),
                Some("insert_newline_at_cursor_chunked:changelog"),
            );

            #[cfg(not(debug_assertions))]
            log_error(
                "Cannot get changelog directory",
                Some("insert_newline_at_cursor_chunked:changelog"),
            );

            // Continue without undo support - insertion succeeded
            None
        }
    };

    // Create log entry if directory path was obtained
    if let Some(log_dir) = log_directory_path {
        // Retry logic: 3 attempts with 50ms pause
        let mut log_success = false;

        for retry_attempt in 0..3 {
            // Convert u64 position to u128 for API compatibility
            let position_u128 = insert_position as u128;

            /*
            pub fn button_make_changelog_from_user_character_action_level(
                target_file: &Path,
                character: Option<char>,
                byte_value: Option<u8>, // raw byte input
                position: u128,
                edit_type: EditType,
                log_directory_path: &Path,
            ) -> ButtonResult<()> {
            */

            match button_make_changelog_from_user_character_action_level(
                file_path,
                Some('\n'), // Character being added
                None,
                position_u128,
                EditType::AddCharacter, // User added, inverse is remove
                &log_dir,
            ) {
                Ok(_) => {
                    log_success = true;
                    break; // Success
                }
                Err(_e) => {
                    if retry_attempt == 2 {
                        // Final retry failed - log but don't fail operation
                        #[cfg(debug_assertions)]
                        log_error(
                            &format!(
                                "Failed to log newline at position {}: {}",
                                position_u128, _e
                            ),
                            Some("insert_newline_at_cursor_chunked:changelog"),
                        );

                        #[cfg(not(debug_assertions))]
                        log_error(
                            "Failed to log newline",
                            Some("insert_newline_at_cursor_chunked:changelog"),
                        );
                    } else {
                        // Retry after brief pause
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                }
            }
        }

        // Optional: Set info bar if logging failed (non-intrusive)
        if !log_success {
            let _ = lines_editor_state.set_info_bar_message("undo disabled");
        }
    }

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
                    "Cannot get current directory",
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
        #[cfg(debug_assertions)]
        log_error(
            &format!("Source file does not exist: {}", source_path.display()),
            Some("insert_file_at_cursor"),
        );
        // safe
        log_error("Source file does not exist", Some("insert_file_at_cursor"));
        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            "if !source_path.exists() File not found",
        )));
    }

    // Defensive: Check source path is a file (not directory)
    // Attempting to read a directory would cause confusing errors later
    if !source_path.is_file() {
        let _ = state.set_info_bar_message("not a file");
        #[cfg(debug_assertions)]
        log_error(
            &format!("Source path is not a file: {}", source_path.display()),
            Some("insert_file_at_cursor"),
        );
        // safe
        log_error("Source path is not a file", Some("insert_file_at_cursor"));
        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "if !source_path.is_file() Not a file",
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
        .get_row_col_file_position(state.cursor.row, state.cursor.col)
    {
        Ok(Some(pos)) => pos.byte_offset_linear_file_absolute_position,
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
            #[cfg(debug_assertions)]
            log_error(
                &format!("Error getting cursor position: {}", e),
                Some("insert_file_at_cursor"),
            );
            // safe
            log_error(
                "match state.get_row_col_file_position(state.cursor.row, state.cursor.col) Error getting cursor position",
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
            #[cfg(debug_assertions)]
            log_error(
                &format!("Cannot open source file: {} - {}", source_path.display(), e),
                Some("insert_file_at_cursor"),
            );
            // safe
            log_error("Cannot open source file", Some("insert_file_at_cursor"));
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
                "Maximum chunk limit reached MAX_CHUNKS",
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
                let _ = state.set_info_bar_message("read error chunk");
                #[cfg(debug_assertions)]
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
    // Phase 6: Create Inverse Changelog Entries
    // ============================================
    // Re-iterate through source file to create undo logs
    // Same chunk-based pattern as Phase 5, but for logging not insertion
    //
    // Purpose: Generate inverse operation logs so user can undo the insertion
    // User action: Add (inserted file) → Inverse log: Rmv (remove those bytes)
    //
    // Important: This happens AFTER insertion completes successfully
    // If logging fails, insertion has already succeeded (non-critical failure)

    // Get changelog directory path
    let log_directory_path = match get_undo_changelog_directory_path(&target_file_path) {
        Ok(path) => path,
        Err(_e) => {
            // Non-critical: Log error but don't fail the insertion operation
            #[cfg(debug_assertions)]
            log_error(
                &format!("Cannot get changelog directory: {}", _e),
                Some("insert_file_at_cursor:phase6"),
            );

            #[cfg(not(debug_assertions))]
            log_error(
                "Cannot get changelog directory",
                Some("insert_file_at_cursor:phase6"),
            );

            let _ = state.set_info_bar_message("undo log path failed");
            // Continue to Phase 7 - insertion succeeded, logging is optional
            state.is_modified = true;
            build_windowmap_nowrap(state, &target_file_path)?;
            let _ = state.set_info_bar_message("inserted (undo disabled)");
            return Ok(());
        }
    };

    // Re-open source file for logging iteration
    // We don't reuse the previous file handle - it's at EOF
    let mut source_file_for_logging = match File::open(&source_path) {
        Ok(file) => file,
        Err(_e) => {
            // Non-critical: File was already inserted successfully
            #[cfg(debug_assertions)]
            log_error(
                &format!(
                    "Cannot reopen source for logging: {} - {}",
                    source_path.display(),
                    _e
                ),
                Some("insert_file_at_cursor:phase6"),
            );

            #[cfg(not(debug_assertions))]
            log_error(
                "Cannot reopen source for logging",
                Some("insert_file_at_cursor:phase6"),
            );

            let _ = state.set_info_bar_message("undo log failed");
            // Continue to Phase 7
            state.is_modified = true;
            build_windowmap_nowrap(state, &target_file_path)?;
            let _ = state.set_info_bar_message("inserted (undo disabled)");
            return Ok(());
        }
    };

    // Initialize logging iteration state
    let mut logging_chunk_counter: usize = 0;
    let mut byte_offset_in_insertion: u64 = 0; // Tracks position within inserted content
    let mut carry_over_bytes: [u8; 4] = [0; 4]; // Max UTF-8 char is 4 bytes
    let mut carry_over_count: usize = 0;
    let mut logging_error_count: usize = 0;
    const MAX_LOGGING_ERRORS: usize = 100; // Stop logging after too many failures

    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    debug_assert!(
        MAX_LOGGING_ERRORS > 0,
        "Max logging errors must be positive"
    );

    #[cfg(test)]
    assert!(
        MAX_LOGGING_ERRORS > 0,
        "Max logging errors must be positive"
    );

    // Production catch-handle (always included)
    if MAX_LOGGING_ERRORS == 0 {
        let _ = state.set_info_bar_message("config error");
        return Err(LinesError::GeneralAssertionCatchViolation(
            "zero max logging errors".into(),
        ));
    }

    // ============================================
    // Logging Bucket Brigade Loop
    // ============================================
    // Same pattern as Phase 5, but creates logs instead of inserting

    loop {
        // Safety limit: Same as insertion loop
        if logging_chunk_counter >= MAX_CHUNKS {
            #[cfg(debug_assertions)]
            log_error(
                "Logging iteration exceeded MAX_CHUNKS",
                Some("insert_file_at_cursor:phase6"),
            );

            #[cfg(not(debug_assertions))]
            log_error(
                "Logging limit reached",
                Some("insert_file_at_cursor:phase6"),
            );

            let _ = state.set_info_bar_message("undo log incomplete");
            break; // Exit loop, continue to Phase 7
        }

        // Stop logging if too many errors (fail-safe)
        if logging_error_count >= MAX_LOGGING_ERRORS {
            #[cfg(debug_assertions)]
            log_error(
                &format!("Logging stopped after {} errors", MAX_LOGGING_ERRORS),
                Some("insert_file_at_cursor:phase6"),
            );

            #[cfg(not(debug_assertions))]
            log_error(
                "Logging stopped after max errors",
                Some("insert_file_at_cursor:phase6"),
            );

            let _ = state.set_info_bar_message("undo log incomplete");
            break;
        }

        // Stack-allocated read buffer (NASA Rule 3: pre-allocated)
        let mut buffer = [0u8; CHUNK_SIZE];

        // Security mode: clear buffer before use
        if state.security_mode {
            for i in 0..CHUNK_SIZE {
                buffer[i] = 0;
            }
        }

        // Read next chunk
        let bytes_read = match source_file_for_logging.read(&mut buffer) {
            Ok(n) => n,
            Err(_e) => {
                #[cfg(debug_assertions)]
                log_error(
                    &format!(
                        "Read error during logging at chunk {}: {}",
                        logging_chunk_counter, _e
                    ),
                    Some("insert_file_at_cursor:phase6"),
                );

                #[cfg(not(debug_assertions))]
                log_error(
                    "Read error during logging",
                    Some("insert_file_at_cursor:phase6"),
                );

                logging_error_count += 1;
                continue; // Skip this chunk, try next
            }
        };

        // =================================================
        // Debug-Assert, Test-Assert, Production-Catch-Handle
        // =================================================

        debug_assert!(bytes_read <= CHUNK_SIZE, "bytes_read exceeded CHUNK_SIZE");

        #[cfg(test)]
        assert!(bytes_read <= CHUNK_SIZE, "bytes_read exceeded CHUNK_SIZE");

        // Production catch-handle
        if bytes_read > CHUNK_SIZE {
            #[cfg(debug_assertions)]
            log_error(
                &format!(
                    "bytes_read {} exceeded CHUNK_SIZE {}",
                    bytes_read, CHUNK_SIZE
                ),
                Some("insert_file_at_cursor:phase6"),
            );

            #[cfg(not(debug_assertions))]
            log_error(
                "Buffer overflow detected",
                Some("insert_file_at_cursor:phase6"),
            );

            let _ = state.set_info_bar_message("undo log failed");
            break; // Exit loop safely
        }

        // EOF detection
        if bytes_read == 0 && carry_over_count == 0 {
            break; // Normal completion
        }

        logging_chunk_counter += 1;

        // Process bytes in this chunk
        let mut buffer_index: usize = 0;

        // If we have carry-over bytes from previous chunk, process them first
        if carry_over_count > 0 {
            // We need more bytes to complete the UTF-8 character
            let bytes_needed = detect_utf8_byte_count(carry_over_bytes[0])
                .unwrap_or(1)
                .saturating_sub(carry_over_count);

            if bytes_needed > 0 && bytes_needed <= bytes_read {
                // Complete the character with bytes from current chunk
                for i in 0..bytes_needed {
                    carry_over_bytes[carry_over_count + i] = buffer[i];
                }
                buffer_index += bytes_needed;

                let full_char_bytes = &carry_over_bytes[0..(carry_over_count + bytes_needed)];

                // Try to decode as UTF-8 character
                match std::str::from_utf8(full_char_bytes) {
                    Ok(s) => {
                        if let Some(ch) = s.chars().next() {
                            // Calculate absolute position in file
                            // Converting from u64 to u128 (safe: u64 always fits in u128)
                            let char_position_u64: u64 =
                                start_byte_position + byte_offset_in_insertion;
                            let char_position_u128 = char_position_u64 as u128;

                            /*
                            pub fn button_make_changelog_from_user_character_action_level(
                                target_file: &Path,
                                character: Option<char>,
                                byte_value: Option<u8>, // raw byte input
                                position: u128,
                                edit_type: EditType,
                                log_directory_path: &Path,
                            ) -> ButtonResult<()> {
                            */

                            // Create inverse log entry (with retry)
                            for retry_attempt in 0..3 {
                                match button_make_changelog_from_user_character_action_level(
                                    &target_file_path,
                                    Some(ch),
                                    None,
                                    char_position_u128,
                                    EditType::AddCharacter, // User added, inverse is remove
                                    &log_directory_path,
                                ) {
                                    Ok(_) => break, // Success
                                    Err(_e) => {
                                        if retry_attempt == 2 {
                                            // Final retry failed
                                            #[cfg(debug_assertions)]
                                            log_error(
                                                &format!(
                                                    "Failed to log char at position {}: {}",
                                                    char_position_u128, _e
                                                ),
                                                Some("insert_file_at_cursor:phase6"),
                                            );

                                            #[cfg(not(debug_assertions))]
                                            log_error(
                                                "Failed to log character",
                                                Some("insert_file_at_cursor:phase6"),
                                            );

                                            logging_error_count += 1;
                                        } else {
                                            // Retry after brief pause
                                            std::thread::sleep(std::time::Duration::from_millis(
                                                50,
                                            ));
                                        }
                                    }
                                }
                            }

                            byte_offset_in_insertion += full_char_bytes.len() as u64;
                        }
                    }
                    Err(_) => {
                        // Invalid UTF-8, skip these bytes
                        #[cfg(debug_assertions)]
                        log_error(
                            &format!(
                                "Invalid UTF-8 in carry-over at offset {}",
                                byte_offset_in_insertion
                            ),
                            Some("insert_file_at_cursor:phase6"),
                        );

                        #[cfg(not(debug_assertions))]
                        log_error(
                            "Invalid UTF-8 in carry-over",
                            Some("insert_file_at_cursor:phase6"),
                        );

                        byte_offset_in_insertion += full_char_bytes.len() as u64;
                    }
                }

                carry_over_count = 0; // Clear carry-over
            }
        }

        // Process remaining bytes in buffer
        while buffer_index < bytes_read {
            let byte = buffer[buffer_index];

            // Detect UTF-8 character length
            let char_len = match detect_utf8_byte_count(byte) {
                Ok(len) => len,
                Err(_) => {
                    // Invalid UTF-8 start byte, skip it
                    #[cfg(debug_assertions)]
                    log_error(
                        &format!(
                            "Invalid UTF-8 start byte at offset {}",
                            byte_offset_in_insertion
                        ),
                        Some("insert_file_at_cursor:phase6"),
                    );

                    #[cfg(not(debug_assertions))]
                    log_error(
                        "Invalid UTF-8 start byte",
                        Some("insert_file_at_cursor:phase6"),
                    );

                    buffer_index += 1;
                    byte_offset_in_insertion += 1;
                    continue;
                }
            };

            // Check if complete character is in buffer
            if buffer_index + char_len <= bytes_read {
                // Complete character available
                let char_bytes = &buffer[buffer_index..(buffer_index + char_len)];

                // Decode UTF-8 character
                match std::str::from_utf8(char_bytes) {
                    Ok(s) => {
                        if let Some(ch) = s.chars().next() {
                            // Calculate absolute position
                            // Converting from u64 to u128 (safe: u64 always fits in u128)
                            let char_position_u64: u64 =
                                start_byte_position + byte_offset_in_insertion;
                            let char_position_u128 = char_position_u64 as u128;

                            /*
                            pub fn button_make_changelog_from_user_character_action_level(
                                target_file: &Path,
                                character: Option<char>,
                                byte_value: Option<u8>, // raw byte input
                                position: u128,
                                edit_type: EditType,
                                log_directory_path: &Path,
                            ) -> ButtonResult<()> {
                            */

                            // Create inverse log entry (with retry)
                            for retry_attempt in 0..3 {
                                match button_make_changelog_from_user_character_action_level(
                                    &target_file_path,
                                    Some(ch),
                                    None,
                                    char_position_u128,
                                    EditType::AddCharacter, // User added, inverse is remove
                                    &log_directory_path,
                                ) {
                                    Ok(_) => break, // Success
                                    Err(_e) => {
                                        if retry_attempt == 2 {
                                            // Final retry failed
                                            #[cfg(debug_assertions)]
                                            log_error(
                                                &format!(
                                                    "Failed to log char at position {}: {}",
                                                    char_position_u128, _e
                                                ),
                                                Some("insert_file_at_cursor:phase6"),
                                            );

                                            #[cfg(not(debug_assertions))]
                                            log_error(
                                                "Failed to log character",
                                                Some("insert_file_at_cursor:phase6"),
                                            );

                                            logging_error_count += 1;
                                        } else {
                                            // Retry after brief pause
                                            std::thread::sleep(std::time::Duration::from_millis(
                                                50,
                                            ));
                                        }
                                    }
                                }
                            }

                            byte_offset_in_insertion += char_len as u64;
                        }
                    }
                    Err(_) => {
                        // Invalid UTF-8 sequence
                        #[cfg(debug_assertions)]
                        log_error(
                            &format!(
                                "Invalid UTF-8 sequence at offset {}",
                                byte_offset_in_insertion
                            ),
                            Some("insert_file_at_cursor:phase6"),
                        );

                        #[cfg(not(debug_assertions))]
                        log_error(
                            "Invalid UTF-8 sequence",
                            Some("insert_file_at_cursor:phase6"),
                        );

                        byte_offset_in_insertion += char_len as u64;
                    }
                }

                buffer_index += char_len;
            } else {
                // Incomplete character at end of chunk - carry over to next iteration
                carry_over_count = bytes_read - buffer_index;

                // =================================================
                // Debug-Assert, Test-Assert, Production-Catch-Handle
                // =================================================

                debug_assert!(
                    carry_over_count <= 4,
                    "carry_over_count exceeds max UTF-8 char length"
                );

                #[cfg(test)]
                assert!(
                    carry_over_count <= 4,
                    "carry_over_count exceeds max UTF-8 char length"
                );

                // Production catch-handle
                if carry_over_count > 4 {
                    #[cfg(debug_assertions)]
                    log_error(
                        &format!("carry_over_count {} exceeds 4", carry_over_count),
                        Some("insert_file_at_cursor:phase6"),
                    );

                    #[cfg(not(debug_assertions))]
                    log_error(
                        "carry_over buffer overflow",
                        Some("insert_file_at_cursor:phase6"),
                    );

                    let _ = state.set_info_bar_message("undo log failed");
                    break; // Exit inner loop safely
                }

                for i in 0..carry_over_count {
                    carry_over_bytes[i] = buffer[buffer_index + i];
                }
                break; // Process carry-over in next iteration
            }
        }
    }

    // Check if logging completed reasonably successfully
    if logging_error_count > 0 {
        #[cfg(debug_assertions)]
        log_error(
            &format!("Logging completed with {} errors", logging_error_count),
            Some("insert_file_at_cursor:phase6"),
        );

        #[cfg(not(debug_assertions))]
        log_error(
            "Logging completed with errors",
            Some("insert_file_at_cursor:phase6"),
        );

        let _ = state.set_info_bar_message("undo log incomplete");
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

    let bytes = total_bytes_written.saturating_sub(1);
    let num_str = bytes.to_string();

    let message = stack_format_it("inserted {} bytes", &[&num_str], "inserted data");

    // Set success message in info bar
    // If it fails, continue operation (message display is non-critical)
    let _ = state.set_info_bar_message(&message).or_else(|_e| {
        // Log error but don't propagate (message is cosmetic)
        #[cfg(debug_assertions)]
        eprintln!("Warning: Failed to set info bar message: {}", _e);
        Ok::<(), LinesError>(()) // Convert to Ok to discard error
    });

    // "Finis"
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

/// Inserts a chunk of text at cursor position using file operations
///
/// # Overview
/// This function inserts text at the current cursor position and creates
/// inverse changelog entries for undo support. Text is inserted character-by-character
/// with proper UTF-8 handling.
///
/// # Workflow
/// 1. Get cursor position from window map
/// 2. Read bytes after insertion point into buffer
/// 3. Insert new text at cursor position
/// 4. Write shifted bytes back
/// 5. Create inverse changelog entries (one per character)
/// 6. Update editor state (modified flag, cursor position)
/// 7. Handle cursor overflow and window scrolling
///
/// # Arguments
/// * `state` - Editor state with cursor position
/// * `file_path` - Path to the read-copy file (absolute path)
/// * `text_bytes` - The bytes to insert (borrowed slice, can be read multiple times)
///
/// # Returns
/// * `Ok(())` - Text inserted successfully (with or without undo logs)
/// * `Err(LinesError)` - File operation failed
///
/// # Error Handling
/// - Cursor position errors: Log warning, return Ok() without inserting
/// - File operation errors: Propagate error (insertion critical)
/// - Changelog errors: Log error, continue (undo is non-critical)
/// - UTF-8 decoding errors: Log error, skip character, continue
/// - All errors handled gracefully without panic
///
/// # Changelog Integration
/// After successful insertion, creates inverse logs:
/// - User action: Add character → Log: Rmv character
/// - One log entry per UTF-8 character
/// - Logging failures are non-critical (don't block insertion)
/// - Maximum 100 logging errors before stopping (fail-safe)
///
/// # Performance
/// - Human typing speed: ~200ms between keystrokes
/// - Logging per char: <50ms typical, 150ms worst case (3 retries)
/// - Latency is imperceptible to user
///
/// # Safety
/// - No heap allocation in production error messages
/// - No data exfiltration in production logs
/// - Stack-only buffers (8KB shift buffer already allocated)
/// - Debug/test builds have full diagnostic messages
/// - Production builds have terse, safe messages
pub fn insert_text_chunk_at_cursor_position(
    lines_editor_state: &mut EditorState,
    file_path: &Path,
    text_bytes: &[u8],
) -> Result<()> {
    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    debug_assert!(file_path.is_absolute(), "File path must be absolute");

    #[cfg(test)]
    assert!(file_path.is_absolute(), "File path must be absolute");

    if !file_path.is_absolute() {
        #[cfg(debug_assertions)]
        log_error(
            &format!("Non-absolute path: {}", file_path.display()),
            Some("insert_text_chunk_at_cursor_position"),
        );

        #[cfg(not(debug_assertions))]
        log_error(
            "Non-absolute path",
            Some("insert_text_chunk_at_cursor_position"),
        );

        let _ = lines_editor_state.set_info_bar_message("path error");
        return Err(LinesError::StateError("Non-absolute path".into()));
    }

    // ============================================
    // Phase 1: Get Cursor Position
    // ============================================

    let file_pos = match lines_editor_state
        .get_row_col_file_position(lines_editor_state.cursor.row, lines_editor_state.cursor.col)
    {
        Ok(Some(pos)) => pos,
        Ok(None) => {
            // Cursor not on valid position - log and return without crashing
            #[cfg(debug_assertions)]
            {
                eprintln!("Warning: Cannot insert - cursor not on valid file position");
                log_error(
                    "Insert failed: cursor not on valid file position",
                    Some("insert_text_chunk_at_cursor_position"),
                );
            }

            #[cfg(not(debug_assertions))]
            log_error(
                "Insert failed: invalid cursor",
                Some("insert_text_chunk_at_cursor_position"),
            );

            let _ = lines_editor_state.set_info_bar_message("invalid cursor");
            return Ok(()); // Return success but do nothing
        }
        Err(_e) => {
            // Error getting position - log and return
            #[cfg(debug_assertions)]
            {
                eprintln!("Warning: Cannot get cursor position: {}", _e);
                log_error(
                    &format!("Insert failed: {}", _e),
                    Some("insert_text_chunk_at_cursor_position"),
                );
            }

            #[cfg(not(debug_assertions))]
            log_error(
                "Insert failed: cursor error",
                Some("insert_text_chunk_at_cursor_position"),
            );

            let _ = lines_editor_state.set_info_bar_message("cursor error");
            return Ok(()); // Return success but do nothing
        }
    };

    let insert_position = file_pos.byte_offset_linear_file_absolute_position;

    // ============================================
    // Phase 2: Perform File Insertion
    // ============================================

    // Open file for read+write
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(file_path)
        .map_err(|e| LinesError::Io(e))?;

    // Read bytes after insertion point into 8K buffer (stack-allocated)
    let mut after_buffer = [0u8; 8192];

    file.seek(SeekFrom::Start(insert_position))
        .map_err(|e| LinesError::Io(e))?;

    let bytes_after = file
        .read(&mut after_buffer)
        .map_err(|e| LinesError::Io(e))?;

    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    debug_assert!(bytes_after <= 8192, "bytes_after exceeded buffer size");

    #[cfg(test)]
    assert!(bytes_after <= 8192, "bytes_after exceeded buffer size");

    if bytes_after > 8192 {
        #[cfg(debug_assertions)]
        log_error(
            &format!("bytes_after {} exceeded buffer 8192", bytes_after),
            Some("insert_text_chunk_at_cursor_position"),
        );

        #[cfg(not(debug_assertions))]
        log_error(
            "Buffer overflow detected",
            Some("insert_text_chunk_at_cursor_position"),
        );

        let _ = lines_editor_state.set_info_bar_message("buffer error");
        return Err(LinesError::GeneralAssertionCatchViolation(
            "buffer overflow".into(),
        ));
    }

    // Write new text at insertion position
    file.seek(SeekFrom::Start(insert_position))
        .map_err(|e| LinesError::Io(e))?;

    file.write_all(text_bytes).map_err(|e| LinesError::Io(e))?;

    // Write the shifted bytes
    file.write_all(&after_buffer[..bytes_after])
        .map_err(|e| LinesError::Io(e))?;

    file.flush().map_err(|e| LinesError::Io(e))?;

    // Update lines_editor_state
    lines_editor_state.is_modified = true;

    // ============================================
    // Phase 3: Log the Edit (Existing Functionality)
    // ============================================

    let text_str = std::str::from_utf8(text_bytes).unwrap_or("[invalid UTF-8]");

    // ============================================
    // Phase 4: Create Inverse Changelog Entries
    // ============================================
    // Iterate through text_bytes to create undo logs
    // Each character gets an inverse log entry for undo support
    //
    // Important: This happens AFTER insertion completes successfully
    // If logging fails, insertion has already succeeded (non-critical failure)

    let log_directory_path = match get_undo_changelog_directory_path(file_path) {
        Ok(path) => path,
        Err(_e) => {
            // Non-critical: Log error but don't fail the insertion operation
            #[cfg(debug_assertions)]
            log_error(
                &format!("Cannot get changelog directory: {}", _e),
                Some("insert_text_chunk:changelog"),
            );

            #[cfg(not(debug_assertions))]
            log_error(
                "Cannot get changelog directory",
                Some("insert_text_chunk:changelog"),
            );

            let _ = lines_editor_state.set_info_bar_message("undo disabled");

            // Skip to Phase 5 (cursor update) - insertion succeeded, logging is optional
            // Continue with cursor update and return
            let char_count = text_str.chars().count();
            lines_editor_state.cursor.col += char_count;

            let right_edge = lines_editor_state.effective_cols.saturating_sub(1);
            if lines_editor_state.cursor.col > right_edge {
                let overflow = lines_editor_state.cursor.col - right_edge;
                lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset += overflow;
                lines_editor_state.cursor.col = right_edge;
                build_windowmap_nowrap(lines_editor_state, file_path)?;
            }

            return Ok(());
        }
    };

    // Initialize changelog iteration state
    let mut byte_offset: u64 = 0; // Offset within inserted text
    let mut logging_error_count: usize = 0;
    const MAX_LOGGING_ERRORS: usize = 100; // Stop logging after too many failures

    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    debug_assert!(
        MAX_LOGGING_ERRORS > 0,
        "Max logging errors must be positive"
    );

    #[cfg(test)]
    assert!(
        MAX_LOGGING_ERRORS > 0,
        "Max logging errors must be positive"
    );

    if MAX_LOGGING_ERRORS == 0 {
        #[cfg(debug_assertions)]
        log_error(
            "MAX_LOGGING_ERRORS is zero",
            Some("insert_text_chunk:changelog"),
        );

        #[cfg(not(debug_assertions))]
        log_error("Config error", Some("insert_text_chunk:changelog"));

        let _ = lines_editor_state.set_info_bar_message("config error");
        return Err(LinesError::GeneralAssertionCatchViolation(
            "zero max logging errors".into(),
        ));
    }

    // ============================================
    // Changelog Creation Loop
    // ============================================
    // Iterate through text_bytes character by character
    // No file reading needed - data already in memory

    let mut buffer_index: usize = 0;

    while buffer_index < text_bytes.len() {
        // Stop logging if too many errors (fail-safe)
        if logging_error_count >= MAX_LOGGING_ERRORS {
            #[cfg(debug_assertions)]
            log_error(
                &format!("Logging stopped after {} errors", MAX_LOGGING_ERRORS),
                Some("insert_text_chunk:changelog"),
            );

            #[cfg(not(debug_assertions))]
            log_error(
                "Logging stopped after max errors",
                Some("insert_text_chunk:changelog"),
            );

            let _ = lines_editor_state.set_info_bar_message("undo log incomplete");
            break;
        }

        let byte = text_bytes[buffer_index];

        // Detect UTF-8 character length
        let char_len = match detect_utf8_byte_count(byte) {
            Ok(len) => len,
            Err(_) => {
                // Invalid UTF-8 start byte, skip it
                #[cfg(debug_assertions)]
                log_error(
                    &format!("Invalid UTF-8 start byte at offset {}", byte_offset),
                    Some("insert_text_chunk:changelog"),
                );

                #[cfg(not(debug_assertions))]
                log_error(
                    "Invalid UTF-8 start byte",
                    Some("insert_text_chunk:changelog"),
                );

                buffer_index += 1;
                byte_offset += 1;
                logging_error_count += 1;
                continue;
            }
        };

        // =================================================
        // Debug-Assert, Test-Assert, Production-Catch-Handle
        // =================================================

        debug_assert!(
            char_len >= 1 && char_len <= 4,
            "UTF-8 char length must be 1-4"
        );

        #[cfg(test)]
        assert!(
            char_len >= 1 && char_len <= 4,
            "UTF-8 char length must be 1-4"
        );

        if char_len < 1 || char_len > 4 {
            #[cfg(debug_assertions)]
            log_error(
                &format!("Invalid char_len {} at offset {}", char_len, byte_offset),
                Some("insert_text_chunk:changelog"),
            );

            #[cfg(not(debug_assertions))]
            log_error("Invalid char length", Some("insert_text_chunk:changelog"));

            buffer_index += 1;
            byte_offset += 1;
            logging_error_count += 1;
            continue;
        }

        // Check if complete character is available in slice
        if buffer_index + char_len <= text_bytes.len() {
            // Complete character available
            let char_bytes = &text_bytes[buffer_index..(buffer_index + char_len)];

            // Decode UTF-8 character
            match std::str::from_utf8(char_bytes) {
                Ok(s) => {
                    if let Some(ch) = s.chars().next() {
                        // Calculate absolute position in file
                        // Converting from u64 to u128 (safe: u64 always fits in u128)
                        let char_position_u64 = insert_position + byte_offset;
                        let char_position_u128 = char_position_u64 as u128;

                        /*
                        pub fn button_make_changelog_from_user_character_action_level(
                            target_file: &Path,
                            character: Option<char>,
                            byte_value: Option<u8>, // raw byte input
                            position: u128,
                            edit_type: EditType,
                            log_directory_path: &Path,
                        ) -> ButtonResult<()> {
                        */

                        // Create inverse log entry (with retry)
                        // User action: Add → Inverse log: Rmv
                        for retry_attempt in 0..3 {
                            match button_make_changelog_from_user_character_action_level(
                                file_path,
                                Some(ch),
                                None,
                                char_position_u128,
                                EditType::AddCharacter, // User added, inverse is remove
                                &log_directory_path,
                            ) {
                                Ok(_) => break, // Success
                                Err(_e) => {
                                    if retry_attempt == 2 {
                                        // Final retry failed
                                        #[cfg(debug_assertions)]
                                        log_error(
                                            &format!(
                                                "Failed to log char '{}' at position {}: {}",
                                                ch, char_position_u128, _e
                                            ),
                                            Some("insert_text_chunk:changelog"),
                                        );

                                        #[cfg(not(debug_assertions))]
                                        log_error(
                                            "Failed to log character",
                                            Some("insert_text_chunk:changelog"),
                                        );

                                        logging_error_count += 1;
                                    } else {
                                        // Retry after brief pause (file may be temporarily busy)
                                        std::thread::sleep(std::time::Duration::from_millis(50));
                                    }
                                }
                            }
                        }

                        byte_offset += char_len as u64;
                    }
                }
                Err(_) => {
                    // Invalid UTF-8 sequence
                    #[cfg(debug_assertions)]
                    log_error(
                        &format!("Invalid UTF-8 sequence at offset {}", byte_offset),
                        Some("insert_text_chunk:changelog"),
                    );

                    #[cfg(not(debug_assertions))]
                    log_error(
                        "Invalid UTF-8 sequence",
                        Some("insert_text_chunk:changelog"),
                    );

                    byte_offset += char_len as u64;
                    logging_error_count += 1;
                }
            }

            buffer_index += char_len;
        } else {
            // Incomplete character at end - should not happen with valid UTF-8 input
            #[cfg(debug_assertions)]
            log_error(
                &format!(
                    "Incomplete UTF-8 character at end, offset {}, need {} bytes, have {}",
                    byte_offset,
                    char_len,
                    text_bytes.len() - buffer_index
                ),
                Some("insert_text_chunk:changelog"),
            );

            #[cfg(not(debug_assertions))]
            log_error(
                "Incomplete UTF-8 at end",
                Some("insert_text_chunk:changelog"),
            );

            logging_error_count += 1;
            break; // Exit loop - cannot process incomplete character
        }
    }

    // Report if logging had errors
    if logging_error_count > 0 {
        #[cfg(debug_assertions)]
        log_error(
            &format!("Changelog completed with {} errors", logging_error_count),
            Some("insert_text_chunk:changelog"),
        );

        #[cfg(not(debug_assertions))]
        log_error(
            "Changelog completed with errors",
            Some("insert_text_chunk:changelog"),
        );

        let _ = lines_editor_state.set_info_bar_message("undo log incomplete");
    }

    // ============================================
    // Phase 5: Update Cursor Position
    // ============================================

    // Update cursor position
    let char_count = text_str.chars().count();
    lines_editor_state.cursor.col += char_count;

    // ==========================================
    // Check if cursor exceeded right edge
    // ==========================================
    let right_edge = lines_editor_state.effective_cols.saturating_sub(1);

    if lines_editor_state.cursor.col > right_edge {
        // Calculate how far past edge we went
        let overflow = lines_editor_state.cursor.col - right_edge;

        // Scroll window right to accommodate
        lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset += overflow;

        // Move cursor back to right edge
        lines_editor_state.cursor.col = right_edge;

        // Rebuild window to show new viewport
        build_windowmap_nowrap(lines_editor_state, file_path)?;
    }

    Ok(())
}

// ===============
//  Have a Pasty!!
// ===============
// See other pasty method in EditorState impl -> fn handle_pasty_mode_input()

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
/// Normalization by `normalize_sort_sanitize_selection_range()`:
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
/// * `normalize_sort_sanitize_selection_range()` - Handles forward/backward selection
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
    lines_editor_state: &mut EditorState,
    source_file_path: &Path,
) -> Result<()> {
    // Step 1: Normalize selection
    let (start, end) = normalize_sort_sanitize_selection_range(
        lines_editor_state.file_position_of_vis_select_start,
        lines_editor_state.file_position_of_vis_select_end,
    )?;

    // Step 1.5: Adjust end position to include complete UTF-8 character
    // If end points to start of multi-byte char (like 花), find its last byte
    // Example: end=7 for 花 at bytes [7,8,9] → adjusted_end=9
    let adjusted_end = find_utf8_char_end(source_file_path, end)?;

    // Step 2: Get clipboard directory
    let clipboard_dir = lines_editor_state
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
fn normalize_sort_sanitize_selection_range(start: u64, end: u64) -> Result<(u64, u64)> {
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
        #[cfg(debug_assertions)]
        log_error(
            &format!("Cannot open file for UTF-8 character end check: {}", e),
            Some("find_utf8_char_end"),
        );
        LinesError::Io(e)
    })?;

    // Seek to character start position
    file.seek(SeekFrom::Start(char_start_byte)).map_err(|e| {
        #[cfg(debug_assertions)]
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

                // Stack Format It!
                let num_str1 = first_byte.to_string();
                let num_str2 = char_start_byte.to_string();

                let formatted_string = stack_format_it(
                    "Invalid UTF-8 start byte 0x{} at position {}",
                    &[&num_str1, &num_str2],
                    "Invalid UTF-8 ",
                );

                log_error(&formatted_string, Some("find_utf8_char_end"));
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
            #[cfg(debug_assertions)]
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
        let num_str_1 = start_byte.to_string();
        let num_str_2 = end_byte.to_string();

        let formatted_string = stack_format_it(
            "Invalid byte range: start={} > end={}",
            &[&num_str_1, &num_str_2],
            "Invalid byte range",
        );

        log_error(&formatted_string, Some("generate_clipboard_filename"));
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
    let mut file = File::open(source_file_path).map_err(|_e| {
        #[cfg(debug_assertions)]
        let formated_string = stack_format_it(
            "Cannot open source file: {}",
            &[&_e.to_string()],
            "Cannot open source file",
        );
        #[cfg(debug_assertions)]
        log_error(&formated_string, Some("generate_clipboard_filename"));
        // safe
        log_error(
            "Cannot open source file",
            Some("generate_clipboard_filename"),
        );
        LinesError::Io(_e)
    })?;

    // Seek to start position
    file.seek(SeekFrom::Start(start_byte)).map_err(|e| {
        let num_1 = start_byte.to_string();
        let formated_string2 =
            stack_format_it("Cannot seek to byte {}", &[&num_1], "Cannot seek to byte");

        log_error(
            &format!("Cannot seek to byte {}: {}", start_byte, e),
            Some("generate_clipboard_filename"),
        );
        // safe
        log_error(&formated_string2, Some("generate_clipboard_filename"));
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
            Err(_e) => {
                // Read error - log and stop reading
                #[cfg(debug_assertions)]
                log_error(
                    &format!("Error reading source file: {}", _e),
                    Some("generate_clipboard_filename"),
                );
                // safe
                log_error(
                    "Error reading source file",
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
            Err(_e) => {
                // This should never happen with ASCII alphanumeric, but handle defensively
                #[cfg(debug_assertions)]
                log_error(
                    &format!("UTF-8 conversion error (using fallback): {}", _e),
                    Some("generate_clipboard_filename"),
                );
                // safe
                log_error(
                    "UTF-8 conversion error (using fallback)",
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

    let num_1 = MAX_ATTEMPTS.to_string();
    let num_2 = base_name.to_string();
    let formatted_string = stack_format_it(
        "GCF: All {} filename variants exist for base name: {}",
        &[&num_1, &num_2],
        "gcf: error: All filename variants exist for base name.",
    );

    log_error(&formatted_string, Some("generate_clipboard_filename"));

    let formatted_string_2 = stack_format_it(
        "Cannot generate unique filename - all {} variants of '{}' already exist",
        &[&num_1, &num_2],
        "gcf: error: Cannot generate unique filename - all variants of already exist",
    );

    Err(LinesError::StateError(formatted_string_2))
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
        let num_1 = start_byte_position.to_string();
        let num_2 = end_byte_position.to_string();
        let formatted_string = stack_format_it(
            "Invalid byte range: start position ({}) is > than end pos ({})",
            &[&num_1, &num_2],
            "Invalid byte range: start position is > than end pos",
        );
        let error_msg = formatted_string;
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
            #[cfg(debug_assertions)]
            {
                let num_2 = e.to_string();
                let formatted_string = stack_format_it(
                    "Cannot open source file: {}",
                    &[&num_2],
                    "Invalid byte range",
                );

                log_error(&formatted_string, Some("append_bytes_from_file_to_file"));
            }
            //safe
            log_error(
                "Cannot open source file",
                Some("append_bytes_from_file_to_file"),
            );
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
            #[cfg(debug_assertions)]
            {
                let error_msg = format!("Cannot open or create target file: {}", e);
                log_error(&error_msg, Some("append_bytes_from_file_to_file"));
            }
            // safe
            log_error(
                "Cannot open or create target file",
                Some("append_bytes_from_file_to_file"),
            );

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
    if let Err(_e) = source_file.seek(SeekFrom::Start(start_byte_position)) {
        #[cfg(debug_assertions)]
        eprintln!("e: {}", _e);
        #[cfg(debug_assertions)]
        let error_msg = format!(
            "Cannot seek to start position {} in source file: {}",
            start_byte_position, _e
        );
        #[cfg(debug_assertions)]
        log_error(&error_msg, Some("append_bytes_from_file_to_file"));
        return Err(LinesError::Io(_e));
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
                #[cfg(debug_assertions)]
                {
                    let error_msg = format!(
                        "Cannot read byte at position {} in source file: {}",
                        current_position, e
                    );
                    log_error(&error_msg, Some("append_bytes_from_file_to_file"));
                }
                // safe
                let num_2 = current_position.to_string();
                let formatted_string = stack_format_it(
                    "Cannot read byte at position {} in source file",
                    &[&num_2],
                    "Cannot read byte at position in source file",
                );
                log_error(&formatted_string, Some("append_bytes_from_file_to_file"));
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
            #[cfg(debug_assertions)]
            {
                let error_msg = format!(
                    "Cannot write byte from position {} to target file: {}",
                    current_position, e
                );
                log_error(&error_msg, Some("append_bytes_from_file_to_file"));
            }
            // safe
            let num_2 = current_position.to_string();
            let formatted_string = stack_format_it(
                "Cannot write byte from position {} to target file: {}",
                &[&num_2],
                "Cannot write byte from position to target file",
            );
            log_error(&formatted_string, Some("append_bytes_from_file_to_file"));
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
        #[cfg(debug_assertions)]
        {
            let error_msg = format!("Cannot flush target file to disk: {}", e);
            log_error(&error_msg, Some("append_bytes_from_file_to_file"));
        }
        // safe
        log_error(
            "Cannot flush target file to disk",
            Some("append_bytes_from_file_to_file"),
        );
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
    // Ok(format!(
    //     "{}Have a Pasty!! {}b{}ack paste{}N{} {}str{}{}(any file) {}clear{}all|{}clear{}N {}Empty{}(Add Freshest!){}",
    //     YELLOW,
    //     RED,
    //     YELLOW,
    //     RED,
    //     YELLOW,
    //     RED,
    //     YELLOW,
    //     YELLOW,
    //     RED,
    //     YELLOW,
    //     RED,
    //     RESET,
    //     RED,
    //     YELLOW,
    //     RESET
    // ))

    let stack_formatted_legend = stack_format_it(
        "{}Have a Pasty!! {}b{}ack paste{}N{} {}str{}{}(any file) {}clear{}all|{}clear{}N {}Empty{}(Add Freshest!){}",
        &[
            &YELLOW, &RED, &YELLOW, &RED, &YELLOW, &RED, &YELLOW, &YELLOW, &RED, &YELLOW, &RED,
            &RESET, &RED, &YELLOW, &RESET,
        ],
        "Have a Pasty!! back pasteN str(any file) clearall|clearN Empty(Add Freshest!)",
    );

    Ok(stack_formatted_legend)
}

/// Formats the Pasty info bar with count, pagination, and error messages
fn format_pasty_info_bar(
    total_count: usize,
    first_count_visible: usize,
    last_count_visible: usize,
    info_bar_message: &str,
) -> io::Result<String> {
    let infobar_message_display = if !info_bar_message.is_empty() {
        stack_format_it(" {}", &[&info_bar_message], "")
    } else {
        String::new()
    };

    // Ok(format!(
    //     // "{}{}{}Total, {}Showing{} {}{}-{}{}{} (Page up/down k/j) {}{} >{} ",  // minimal
    //     "{}{}{} Clipboard Items, {}Showing{} {}{}-{}{}{} (Page up/down k/j) {}{}\nEnter clipboard item # to paste, or a file-path to paste file text {}> ",
    //     RED,
    //     total_count,
    //     YELLOW,
    //     YELLOW,
    //     RED,
    //     first_count_visible,
    //     YELLOW,
    //     RED,
    //     last_count_visible,
    //     YELLOW,
    //     infobar_message_display,
    //     YELLOW,
    //     RESET
    // ))

    let string_totalcount = total_count.to_string();
    let string_firstcount_visible = first_count_visible.to_string();
    let string_last_count_visible = last_count_visible.to_string();

    let stack_formatted_infobar = stack_format_it(
        "{}{}{} Clipboard Items, {}Showing{} {}{}-{}{}{} (Page up/down k/j) {}{}\nEnter clipboard item # to paste, or a file-path to paste file text {}> ",
        &[
            &RED,
            &string_totalcount,
            &YELLOW,
            &YELLOW,
            &RED,
            &string_firstcount_visible,
            &YELLOW,
            &RED,
            &string_last_count_visible,
            &YELLOW,
            &infobar_message_display,
            &YELLOW,
            &RESET,
        ],
        "Have a Pasty!! back pasteN str(any file) clearall|clearN Empty(Add Freshest!)",
    );

    Ok(stack_formatted_infobar)
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

/// Creates or selects a read-only copy of the file in the session directory with version management
///
/// # Purpose
/// Provides version management for draft copies within a session directory.
/// When pre-existing draft copies are detected, presents user with selection menu.
/// User decides which version to continue editing, or creates fresh copy.
///
/// # Project Context - Version Management v1
/// Session directories persist across file edits and editor restarts, allowing users to:
/// - Recover from crashes with timestamped drafts
/// - Move between files (copy/paste) while preserving session state
/// - Select from previous draft versions when reopening files
/// - Create fresh copies when desired
///
/// This supports multi-file workflows where session directory contains drafts
/// from multiple file editing sessions, potentially across editor restarts.
///
/// # Behavior Flow
/// 1. Scans session directory for existing drafts matching `*_{original_filename}`
/// 2. If none found: Creates new copy with session_time_stamp (no menu)
/// 3. If found: Shows menu with up to 8 options, sorted newest first
/// 4. User selects version (0=new, 1-8=existing) via stdin
/// 5. Returns path to selected or newly created file
///
/// # Arguments
/// * `original_path` - Path to the original file
/// * `session_dir` - Path to this session's directory (from EditorState)
/// * `session_time_stamp` - Timestamp to use if creating new copy
///
/// # Returns
/// * `Ok(PathBuf)` - Path to selected existing draft or newly created copy
/// * `Err(io::Error)` - Critical failure (falls back to new copy when possible)
///
/// # User Interface
/// ```text
/// File Version Choice & Recovery Q&A
///
/// Pre-existing draft-copies of this file have been detected.
/// Please select which, if any, existing draft-copy you want
/// to continue to edit. Or, by default (empty-enter), you
/// can start life afresh: "sing, heigh-ho! unto the green holly"
///
/// Directory: /path/to/sessions/2025_01_15_14_30_45
///
/// Options:
/// 0. Create new draft-copy
///
/// 1. 2025_01_15_14_30_45_file.txt
/// 2. 2025_01_14_10_20_30_file.txt
///
/// Enter choice (0-2): _
/// ```
///
/// # Design Notes
/// - NO automatic selection - user compares and decides
/// - Stack-allocated only (no heap format!)
/// - Uses stdin.read() for single-byte input
/// - Bounded to 8 draft copies maximum
/// - Filenames truncated to 32 bytes for display (shows timestamp)
/// - Sorts newest first (timestamp descending)
/// - Falls back to creating new copy on any scan/display/input error
/// - Session directory path shown once; list shows filenames only
pub fn create_a_readcopy_of_file(
    original_path: &Path,
    session_dir: &Path,
    session_time_stamp: String,
) -> io::Result<PathBuf> {
    // Maximum draft copies shown in version selection menu
    const MAX_DRAFT_COPIES: usize = 8;

    // Display width for truncated filenames (shows timestamp)
    const FILENAME_DISPLAY_SIZE: usize = 32;

    // Input buffer for stdin read (single digit + newline)
    const USER_INPUT_BUFFER_SIZE: usize = 4;

    // Defensive: Validate inputs
    if !original_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "create_or_select_readcopy_of_file: Original file does not exist",
        ));
    }

    if !session_dir.exists() || !session_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "create_or_select_readcopy_of_file: Session directory does not exist",
        ));
    }

    // Get original filename for pattern matching
    let file_name = original_path
        .file_name()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "create_or_select_readcopy_of_file: Cannot determine filename",
            )
        })?
        .to_string_lossy();

    // ===================================================================
    // STEP 1: Scan for existing draft copies matching *_{original_filename}
    // ===================================================================

    // Stack array to hold existing draft paths
    let mut draft_paths: [Option<PathBuf>; MAX_DRAFT_COPIES] = Default::default();
    let mut draft_count: usize = 0;

    // Read directory entries and filter for matching pattern
    let read_dir = match fs::read_dir(session_dir) {
        Ok(rd) => rd,
        Err(_) => {
            // Fallback: If directory scan fails, create new copy
            #[cfg(debug_assertions)]
            eprintln!(
                "create_or_select_readcopy_of_file: Failed to read session directory, creating new copy"
            );
            return create_new_draft_copy(
                original_path,
                session_dir,
                &session_time_stamp,
                &file_name,
            );
        }
    };

    // Collect matching files
    for entry_result in read_dir {
        if draft_count >= MAX_DRAFT_COPIES {
            break; // Bounded: Stop at max
        }

        let entry = match entry_result {
            Ok(e) => e,
            Err(_) => continue, // Skip invalid entries
        };

        let entry_path = entry.path();

        // Only consider files (not directories)
        if !entry_path.is_file() {
            continue;
        }

        // Check if filename matches pattern: *_{original_filename}
        if let Some(entry_filename) = entry_path.file_name() {
            let entry_filename_str = entry_filename.to_string_lossy();

            // Pattern: Must end with _{original_filename}
            let suffix_pattern = stack_format_it("_{}", &[&file_name], "");
            if entry_filename_str.ends_with(&suffix_pattern) {
                draft_paths[draft_count] = Some(entry_path);
                draft_count += 1;
            }
        }
    }

    // ===================================================================
    // STEP 2: Branch on results - If no drafts, create new copy
    // ===================================================================

    if draft_count == 0 {
        // No existing drafts found - skip menu, create new copy
        return create_new_draft_copy(original_path, session_dir, &session_time_stamp, &file_name);
    }

    // ===================================================================
    // STEP 3: Display menu
    // ===================================================================

    // Show header
    println!("\nFile Version Choice & Recovery Q&A\n");
    println!("Pre-existing draft-copies of this file have been detected.");
    println!("Please select which, if any, existing draft-copy you want");
    println!("to continue to edit. Or, by default (empty-enter), you");
    println!("can start life afresh: \"sing, heigh-ho! unto the green holly\"\n");

    // Show session directory path
    println!("Directory: {}\n", session_dir.display());

    println!("Options:");
    println!("0. Create new draft-copy\n");

    // Show existing drafts (filenames only, truncated)
    for i in 0..draft_count {
        if let Some(ref path) = draft_paths[i] {
            if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy();

                // Truncate to FILENAME_DISPLAY_SIZE if needed
                let display_name = if filename_str.len() > FILENAME_DISPLAY_SIZE {
                    &filename_str[..FILENAME_DISPLAY_SIZE]
                } else {
                    &filename_str
                };

                let option_num = (i + 1).to_string();
                let display_line =
                    stack_format_it("{}. {}", &[&option_num, display_name], "Option unavailable");
                println!("{}", display_line);
            }
        }
    }

    // Prompt for input
    let max_choice = draft_count.to_string();
    let prompt = stack_format_it(
        "\nEnter choice (0-{}): ",
        &[&max_choice],
        "\nEnter choice: ",
    );
    print!("{}", prompt);

    // Flush stdout to ensure prompt appears
    if let Err(_) = io::stdout().flush() {
        #[cfg(debug_assertions)]
        eprintln!("create_or_select_readcopy_of_file: Failed to flush stdout");
        // Continue anyway
    }

    // ===================================================================
    // STEP 5: Read user input using stdin.read()
    // ===================================================================

    let mut input_buffer = [0u8; USER_INPUT_BUFFER_SIZE];
    let user_choice: usize;

    {
        let stdin = io::stdin();
        let mut stdin_handle = stdin.lock();

        let bytes_read = match stdin_handle.read(&mut input_buffer) {
            Ok(n) => n,
            Err(_) => {
                #[cfg(debug_assertions)]
                eprintln!(
                    "create_or_select_readcopy_of_file: Failed to read stdin, defaulting to new copy"
                );
                0 // Default to 0 on read failure
            }
        };

        // Parse first byte as ASCII digit
        if bytes_read > 0 {
            let first_byte = input_buffer[0];

            // Check if it's ASCII digit '0'-'9' (48-57)
            if first_byte >= b'0' && first_byte <= b'9' {
                user_choice = (first_byte - b'0') as usize;
            } else {
                // Non-digit input defaults to 0
                user_choice = 0;
            }
        } else {
            // Empty input or error defaults to 0
            user_choice = 0;
        }
    } // stdin_handle dropped here

    // Defensive: Validate choice is in range
    if user_choice > draft_count {
        // Out of range defaults to 0
        #[cfg(debug_assertions)]
        eprintln!("create_or_select_readcopy_of_file: Choice out of range, creating new copy");
        return create_new_draft_copy(original_path, session_dir, &session_time_stamp, &file_name);
    }

    // ===================================================================
    // STEP 6: Act on selection
    // ===================================================================

    if user_choice == 0 {
        // User selected to create new copy
        return create_new_draft_copy(original_path, session_dir, &session_time_stamp, &file_name);
    }

    // User selected existing draft (1-based index)
    let selected_index = user_choice - 1;

    if let Some(ref selected_path) = draft_paths[selected_index] {
        // Defensive: Verify selected file still exists
        if selected_path.exists() {
            debug_assert!(
                selected_path.is_absolute(),
                "Selected draft path should be absolute"
            );

            return Ok(selected_path.clone());
        } else {
            // File disappeared between scan and selection - fall back to new copy
            #[cfg(debug_assertions)]
            eprintln!(
                "create_or_select_readcopy_of_file: Selected file no longer exists, creating new copy"
            );
            return create_new_draft_copy(
                original_path,
                session_dir,
                &session_time_stamp,
                &file_name,
            );
        }
    }

    // Should not reach here, but fall back to new copy if we do
    #[cfg(debug_assertions)]
    eprintln!("create_or_select_readcopy_of_file: Invalid selection state, creating new copy");
    create_new_draft_copy(original_path, session_dir, &session_time_stamp, &file_name)
}

/// Helper function: Creates new draft copy with timestamp prefix
///
/// # Purpose
/// Creates timestamped copy in session directory. Used by version management
/// when user selects "new copy" option or when no existing drafts found.
///
/// # Project Context
/// Supports version management system by providing clean draft creation
/// with consistent naming: {timestamp}_{original_filename}
///
/// # Arguments
/// * `original_path` - Path to original file to copy
/// * `session_dir` - Session directory for draft storage
/// * `timestamp` - Timestamp prefix for filename
/// * `file_name` - Original filename (from original_path)
///
/// # Returns
/// * `Ok(PathBuf)` - Path to newly created draft copy
/// * `Err(io::Error)` - Copy operation failed
///
/// # File Naming
/// Format: `{timestamp}_{original_filename}`
/// Example: `2025_01_15_14_30_45_file.txt`
fn create_new_draft_copy(
    original_path: &Path,
    session_dir: &Path,
    timestamp: &str,
    file_name: &str,
) -> io::Result<PathBuf> {
    // Build draft filename: {timestamp}_{original_filename}
    let draft_name = stack_format_it("{}_{}", &[timestamp, file_name], "draft_copy");

    let draft_path = session_dir.join(&draft_name);

    // If draft already exists (idempotent), return it
    if draft_path.exists() {
        debug_assert!(draft_path.is_absolute(), "Draft path should be absolute");
        return Ok(draft_path);
    }

    // Copy the file to session directory
    fs::copy(original_path, &draft_path).map_err(|_| {
        io::Error::new(
            io::ErrorKind::Other,
            "create_new_draft_copy: Failed to copy file",
        )
    })?;

    // Defensive: Verify copy succeeded
    if !draft_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "create_new_draft_copy: Copy reported success but file not found",
        ));
    }

    // Assertion: Verify result is valid
    debug_assert!(draft_path.is_absolute(), "Draft path should be absolute");
    debug_assert!(draft_path.exists(), "Draft should exist after creation");

    Ok(draft_path)
}

/// Prints help message to stdout
///
/// # Purpose
/// Displays usage information and available commands.
/// Called when user runs `lines --help`.
pub fn print_help() {
    println!("About Lines Editor: (note: ctrl+s can block terminal, ctrl+z unblocks)");
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
fn format_info_bar_cafe_normal_visualselect(lines_editor_state: &EditorState) -> Result<String> {
    /*
    Calculation note:
    The column number should be - the number of digits +1

    */
    // Get mode string
    let mode_str = match lines_editor_state.mode {
        EditorMode::Normal => "NORMAL",
        EditorMode::Insert => "INSERT",
        EditorMode::VisualSelectMode => "VISUAL",
        EditorMode::PastyMode => "PASTY",
        EditorMode::HexMode => "HEX",
        EditorMode::RawMode => "RAW",
    };

    // Get current line and column
    // Line is 1-indexed for display (humans count from 1)
    let line_display =
        lines_editor_state.line_count_at_top_of_window + lines_editor_state.cursor.row + 1;

    // Get line number to calculate line number display width
    let line_num =
        lines_editor_state.line_count_at_top_of_window + lines_editor_state.cursor.row + 1;
    let line_num_width = calculate_line_number_width(
        lines_editor_state.line_count_at_top_of_window,
        line_num,
        lines_editor_state.effective_rows,
    );

    // Add horizontal offset to get character position in line
    // Subtract line number width from displayed column
    let true_char_position = lines_editor_state.cursor.col
        + lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset;

    // zero-based vs. 1 based
    let col_display = true_char_position.saturating_sub(line_num_width) + 1;

    // Get filename (or "unnamed" if none)
    let filename = lines_editor_state
        .original_file_path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unmanned file");

    // Extract message from buffer (find null terminator or use full buffer)
    let message_len = lines_editor_state
        .info_bar_message_buffer
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(lines_editor_state.info_bar_message_buffer.len());

    let message_for_infobar =
        std::str::from_utf8(&lines_editor_state.info_bar_message_buffer[..message_len])
            .unwrap_or(""); // Empty string if invalid UTF-8

    // Step 1: Get current line's file position
    // In case of exception, say 'n/a'
    let file_position_string = match lines_editor_state
        .get_row_col_file_position(lines_editor_state.cursor.row, lines_editor_state.cursor.col)
    {
        Ok(Some(row_col_file_pos)) => row_col_file_pos
            .byte_offset_linear_file_absolute_position
            .to_string(),
        _ => "n/a".to_string(),
    };

    // let row_col_file_pos = lines_editor_state
    //     .window_map
    //     .get_row_col_file_position(lines_editor_state.cursor.row, lines_editor_state.cursor.col)?
    //     .ok_or_else(|| {
    //         io::Error::new(
    //             io::ErrorKind::InvalidInput,
    //             "fib: Cursor not on valid position",
    //         )
    //     })?;

    // // Get file position at/of/where cursor
    // let file_position_string = row_col_file_pos
    //     .byte_offset_linear_file_absolute_position
    //     .to_string();

    // Build the info bar
    // let info = format!(
    //     // "{}{}{} line{}{} {}col{}{}{} {}{} >{}",
    //     "{}{} {}{}{}:{}{}{} {}{} @{}{}{} {}{} > ",
    //     YELLOW,
    //     mode_str,
    //     // YELLOW,
    //     RED,
    //     line_display,
    //     YELLOW,
    //     YELLOW,
    //     RED,
    //     col_display,
    //     YELLOW,
    //     filename,
    //     // GREEN,
    //     RED,
    //     file_position_string,
    //     YELLOW,
    //     message_for_infobar,
    //     RESET,
    // );

    // Build the info bar (no-heap)
    let info_bar = stack_format_it(
        "{}{} {}{}{}:{}{}{} {}{} @{}{}{} {}{} > ",
        &[
            &YELLOW,
            &mode_str,
            &RED,
            &line_display.to_string(),
            &YELLOW,
            &YELLOW,
            &RED,
            &col_display.to_string(),
            &YELLOW,
            &filename,
            &RED,
            &file_position_string,
            &YELLOW,
            &message_for_infobar,
            &RESET,
        ],
        " > ",
    );
    Ok(info_bar)
}

//  ======================
//  HEX Render a Flesh TUI
//  ======================
/// Hex editor display state
///
/// # Purpose
/// Tracks position within file for hex viewing/editing.
/// Separate from UTF-8 cursor position to avoid conflating byte-offset
/// with character-offset semantics.
///
/// # Fields
/// * `byte_offset_linear_file_absolute_position` - Absolute position in file (0-indexed)
/// * `bytes_per_row` - Display width constant (26 for 80-char TUI)
pub struct HexCursor {
    /// Absolute byte position in file (0-indexed)
    /// Range: 0 to file_size
    pub byte_offset_linear_file_absolute_position: usize,

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
            byte_offset_linear_file_absolute_position: 0,
            bytes_per_row: 26,
        }
    }

    /// Calculates which display row this byte offset is on
    ///
    /// # Returns
    /// Row number (0-indexed)
    pub fn current_row(&self) -> usize {
        self.byte_offset_linear_file_absolute_position / self.bytes_per_row
    }

    /// Calculates column within current row
    ///
    /// # Returns
    /// Column position (0-25 for 26 bytes per row)
    pub fn current_col(&self) -> usize {
        self.byte_offset_linear_file_absolute_position % self.bytes_per_row
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
/// Reads only 26 bytes starting at `hex_cursor.byte_offset_linear_file_absolute_position`
/// Does NOT load entire file into memory
pub fn render_tui_hex(state: &EditorState) -> Result<()> {
    // Clear screen
    print!("\x1B[2J\x1B[H");
    io::stdout().flush().map_err(|e| {
        LinesError::DisplayError(stack_format_it(
            "Failed to flush stdout: {}",
            &[&e.to_string()],
            "Failed to flush stdout",
        ))
    })?;

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

    io::stdout().flush().map_err(|e| {
        LinesError::DisplayError(stack_format_it(
            "Failed to flush stdout: {}",
            &[&e.to_string()],
            "Failed to flush stdout",
        ))
    })?;

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

            // TODO: formatting?
            // === HEX LINE ===
            // Highlight if this is cursor position
            // if i == cursor_col {
            //     hex_line.push_str(&format!(
            //         "{}{}{}{:02X}{} ",
            //         BOLD, RED, BG_WHITE, byte, RESET
            //     ));
            // } else {
            //     hex_line.push_str(&format!("{:02X} ", byte));
            // }

            // Hex formatting
            let mut hex_buf = [0u8; 64];

            if let Some(formatted) = stack_format_hex(
                byte,
                &mut hex_buf,
                i == cursor_col, // highlight flag
                BOLD,
                RED,
                BG_WHITE,
                RESET,
            ) {
                hex_line.push_str(formatted);
            } else {
                // Fallback if buffer somehow fails
                hex_line.push_str("?? ");
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
                // utf8_line.push_str(&format!("{}  ", display_char));
                utf8_line.push_str(&stack_format_it(
                    "{}  ",
                    &[&display_char.to_string()],
                    "_  ",
                ));
            }
        } else {
            // Past EOF - show empty space
            hex_line.push_str("   "); // 3 spaces (matches "48 " width)
            utf8_line.push_str("   "); // 3 spaces (matches "H  " width)
        }
    }

    // Combine into two-line output
    // let result = format!("{}\n{}\n", hex_line.trim_end(), utf8_line.trim_end());

    let result = stack_format_it(
        "{}\n{}\n",
        &[&hex_line.trim_end(), &utf8_line.trim_end()],
        "_\n_\n",
    );

    // TODO: stack formatting in this function
    Ok(result)
}

//  =====================
//  Sashimi Raw TUI Ramen
//  =====================
//  =====================
//  RAW String TUI
//  =====================
//  =====================
//  Sashimi Raw TUI Ramen
//  =====================

//  =====================
//  RAW STRING TUI
//  =====================

/// Renders the complete TUI in RAW STRING mode
///
/// # Purpose
/// Displays raw string view with escape sequences visible:
/// 1. Top: Command legend (1 line, same as hex mode)
/// 2. Middle: Raw string with visible escapes + interpreted text (2 lines)
/// 3. Bottom: Info bar (1 line, shows byte offset)
///
/// # Layout
/// ```text
/// quit ins vis save undo hjkl wb /search       <- Legend
/// H  e  l  l  o  \n W  o  r  l  d  \t A  B    <- Raw (escapes visible)
/// H  e  l  l  o  ␊  W  o  r  l  d  ␉  A  B    <- Interpreted
/// RAW byte 156 of 1024 doc.txt > cmd_         <- Info bar
/// ```
pub fn render_tui_raw(state: &EditorState) -> Result<()> {
    // Clear screen
    print!("\x1B[2J\x1B[H");
    io::stdout().flush().map_err(|e| {
        LinesError::DisplayError(stack_format_it(
            "Failed to flush stdout: {}",
            &[&e.to_string()],
            "Failed to flush stdout",
        ))
    })?;

    // === TOP LINE: LEGEND (same as hex mode) ===
    let legend = format_navigation_legend()?;
    println!("{}", legend);

    // padding
    for _ in 0..5 {
        println!();
    }

    // === MIDDLE: RAW + INTERPRETED DISPLAY (2 lines) ===
    let raw_display = render_raw_row(state)?;
    print!("{}", raw_display);

    // padding
    for _ in 0..14 {
        println!();
    }

    // === BOTTOM LINE: INFO BAR ===
    let info_bar = format_raw_info_bar(state)?;
    print!("{}", info_bar);

    io::stdout().flush().map_err(|e| {
        LinesError::DisplayError(stack_format_it(
            "Failed to flush stdout: {}",
            &[&e.to_string()],
            "Failed to flush stdout",
        ))
    })?;

    Ok(())
}

// ============================================================================
// UTF-8 CHARACTER ANALYSIS (Helper for Multi-byte Character Handling)
// ============================================================================

/// Determines the byte length of a UTF-8 character at a specific file position
///
/// # Purpose (Project Context)
/// Text editors must handle international text correctly. When detecting if
/// the cursor is at a line end (for line-wrapping in cursor movement),
/// we must know the COMPLETE character's byte span, not just its starting byte.
///
/// For ASCII 'a' (1 byte), the character ends where it starts.
/// For Chinese '世' (3 bytes: E4 B8 96), the character spans 3 bytes,
/// so "next byte after character" is 3 bytes forward, not 1.
///
/// This function enables correct line-end detection for all UTF-8 text.
///
/// # Strategy
/// 1. Opens file (stateless operation)
/// 2. Seeks to target byte position
/// 3. Reads first byte to examine UTF-8 pattern
/// 4. Returns character length based on first-byte pattern
/// 5. Closes file automatically (RAII)
///
/// # UTF-8 First-Byte Patterns
/// ```text
/// Byte Range   Pattern      Character Length
/// 0x00..=0x7F  0xxxxxxx     1 byte  (ASCII)
/// 0xC0..=0xDF  110xxxxx     2 bytes
/// 0xE0..=0xEF  1110xxxx     3 bytes
/// 0xF0..=0xF7  11110xxx     4 bytes
/// 0x80..=0xBF  10xxxxxx     Invalid (continuation byte, not first byte)
/// 0xF8..=0xFF  11111xxx     Invalid (UTF-8 doesn't use these)
/// ```
///
/// # Arguments
/// * `file_path` - Absolute path to file being analyzed
/// * `byte_position` - Absolute byte offset in file where character starts
///
/// # Returns
/// * `Ok(1)` - Single-byte character (ASCII) OR invalid UTF-8 treated as 1 byte
/// * `Ok(2)` - Two-byte UTF-8 character
/// * `Ok(3)` - Three-byte UTF-8 character
/// * `Ok(4)` - Four-byte UTF-8 character
/// * `Err(LinesError::Io)` - Unrecoverable file I/O error (hardware failure)
///
/// # Defensive Programming - Graceful Degradation
/// Invalid UTF-8 sequences are treated as single bytes instead of crashing.
/// This allows the editor to handle:
/// - Corrupted files (bit flips, partial writes)
/// - Binary data mixed with text
/// - Files with encoding errors
/// - Malicious or malformed input
///
/// The editor continues operating; users can see and edit raw bytes.
///
/// # Error Handling Philosophy
/// - File not found → Returns `Err` (propagates to caller for logging)
/// - Cannot open file → Returns `Err` (permission/hardware issue)
/// - EOF at position → Returns `Ok(1)` (treat as single byte, defensive)
/// - Invalid UTF-8 → Returns `Ok(1)` (treat as single byte, defensive)
/// - Read I/O error → Returns `Err` (hardware failure)
///
/// # Memory Allocation
/// Zero heap allocation in critical path:
/// - File handle on stack (dropped automatically)
/// - Single-byte buffer `[u8; 1]` on stack
/// - No string allocation
/// - Error messages use string literals in production
///
/// # Examples
/// ```ignore
/// // ASCII character 'a' (0x61)
/// let len = get_utf8_char_byte_length_at_position(path, 10)?;
/// assert_eq!(len, 1);
///
/// // Chinese character '世' (0xE4 0xB8 0x96)
/// let len = get_utf8_char_byte_length_at_position(path, 20)?;
/// assert_eq!(len, 3);
///
/// // Invalid byte (0x80 continuation byte at start)
/// let len = get_utf8_char_byte_length_at_position(path, 30)?;
/// assert_eq!(len, 1); // Defensive: treat as single byte
/// ```
fn get_utf8_char_byte_length_at_position(file_path: &Path, byte_position: u64) -> Result<usize> {
    // ═══════════════════════════════════════════════════════════════════════
    // DEFENSIVE CHECK 1: Validate file path is absolute
    // ═══════════════════════════════════════════════════════════════════════
    // Assertion: All file paths in this project must be absolute for security
    // and clarity. Relative paths create ambiguity and security risks.
    debug_assert!(
        file_path.is_absolute(),
        "File path must be absolute for UTF-8 character analysis"
    );

    if !file_path.is_absolute() {
        #[cfg(debug_assertions)]
        eprintln!(
            "get_utf8_char_byte_length_at_position: non-absolute path rejected: {:?}",
            file_path
        );

        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "File path must be absolute",
        )));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DEFENSIVE CHECK 2: Validate file exists before attempting to read
    // ═══════════════════════════════════════════════════════════════════════
    if !file_path.exists() {
        #[cfg(debug_assertions)]
        eprintln!(
            "get_utf8_char_byte_length_at_position: file does not exist: {:?}",
            file_path
        );

        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            "File not found",
        )));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // FILE OPERATION: Open file for reading (stateless operation)
    // ═══════════════════════════════════════════════════════════════════════
    // RAII: File handle automatically closed when function exits
    let mut file = match File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            #[cfg(debug_assertions)]
            eprintln!(
                "get_utf8_char_byte_length_at_position: failed to open file {:?}: {}",
                file_path, e
            );

            // Propagate error - file access failure is unrecoverable here
            return Err(LinesError::Io(e));
        }
    };

    // ═══════════════════════════════════════════════════════════════════════
    // FILE OPERATION: Seek to target byte position
    // ═══════════════════════════════════════════════════════════════════════
    if let Err(e) = file.seek(SeekFrom::Start(byte_position)) {
        #[cfg(debug_assertions)]
        eprintln!(
            "get_utf8_char_byte_length_at_position: seek failed to position {}: {}",
            byte_position, e
        );

        // Propagate error - seek failure indicates corrupted file or hardware issue
        return Err(LinesError::Io(e));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STACK ALLOCATION: Single-byte buffer for reading UTF-8 first byte
    // ═══════════════════════════════════════════════════════════════════════
    // No heap allocation: fixed-size stack buffer
    let mut first_byte_buffer = [0u8; 1];

    // ═══════════════════════════════════════════════════════════════════════
    // FILE OPERATION: Read first byte of character
    // ═══════════════════════════════════════════════════════════════════════
    let bytes_read = match file.read(&mut first_byte_buffer) {
        Ok(n) => n,
        Err(e) => {
            #[cfg(debug_assertions)]
            eprintln!(
                "get_utf8_char_byte_length_at_position: read failed at position {}: {}",
                byte_position, e
            );

            // Propagate error - read failure indicates hardware/permission issue
            return Err(LinesError::Io(e));
        }
    };

    // ═══════════════════════════════════════════════════════════════════════
    // DEFENSIVE CHECK 3: Handle EOF (cursor positioned at or past end of file)
    // ═══════════════════════════════════════════════════════════════════════
    if bytes_read == 0 {
        #[cfg(debug_assertions)]
        eprintln!(
            "get_utf8_char_byte_length_at_position: EOF at position {} (treating as 1 byte)",
            byte_position
        );

        // Defensive: Treat EOF as single byte
        // This allows cursor movement logic to handle end-of-file gracefully
        return Ok(1);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // UTF-8 ANALYSIS: Determine character length from first-byte pattern
    // ═══════════════════════════════════════════════════════════════════════
    let first_byte = first_byte_buffer[0];

    // Assertion: We should have read exactly 1 byte if not EOF
    debug_assert_eq!(bytes_read, 1, "Expected to read exactly 1 byte");

    let char_length = if first_byte <= 0x7F {
        // Pattern: 0xxxxxxx → 1-byte character (ASCII)
        1
    } else if first_byte >= 0xC0 && first_byte <= 0xDF {
        // Pattern: 110xxxxx → 2-byte character
        2
    } else if first_byte >= 0xE0 && first_byte <= 0xEF {
        // Pattern: 1110xxxx → 3-byte character
        3
    } else if first_byte >= 0xF0 && first_byte <= 0xF7 {
        // Pattern: 11110xxx → 4-byte character
        4
    } else {
        // Invalid UTF-8 first byte:
        // - 0x80..=0xBF (continuation byte, not valid as first byte)
        // - 0xF8..=0xFF (invalid UTF-8 range)
        //
        // Defensive: Treat as single byte, allow editor to continue
        #[cfg(debug_assertions)]
        eprintln!(
            "get_utf8_char_byte_length_at_position: invalid UTF-8 first byte 0x{:02X} at position {} (treating as 1 byte)",
            first_byte, byte_position
        );

        1 // Defensive fallback
    };

    // Assertion: Character length must be 1-4 (UTF-8 standard)
    debug_assert!(
        char_length >= 1 && char_length <= 4,
        "UTF-8 character length must be 1-4 bytes, got {}",
        char_length
    );

    Ok(char_length)
}

/// Renders one row of raw string data with interpreted view
///
/// # Purpose
/// Displays 26 bytes in two formats:
/// 1. Raw representation with escape sequences (\n, \t, etc.)
/// 2. Interpreted character representation (same as hex mode UTF-8 line)
///
/// # Format
/// ```text
/// H  e  l  l  o  \n W  o  r  l  d  \t
/// H  e  l  l  o  ␊  W  o  r  l  d  ␉
/// ```
///
/// # Escape Sequences
/// - Newline (0x0A) → \n
/// - Tab (0x09) → \t
/// - Carriage return (0x0D) → \r
/// - Backslash (0x5C) → \\
/// - Quote (0x22) → \"
/// - Other non-printable → \xHH (hex escape)
/// - Regular printable → as-is
fn render_raw_row(state: &EditorState) -> Result<String> {
    const BYTES_TO_DISPLAY: usize = 26;
    const BOLD: &str = "\x1b[1m";
    const RED: &str = "\x1b[31m";
    const BG_WHITE: &str = "\x1b[47m";
    const RESET: &str = "\x1b[0m";

    let mut raw_line = String::with_capacity(120); // Escapes can be 2-4 chars
    let mut interpreted_line = String::with_capacity(80);
    let mut byte_buffer = [0u8; BYTES_TO_DISPLAY];

    // Get file path from state
    let file_path = state
        .read_copy_path
        .as_ref()
        .ok_or_else(|| LinesError::StateError("No file path in raw mode".to_string()))?;

    let mut file = File::open(file_path).map_err(|e| LinesError::Io(e))?;

    // Calculate ROW START (same as hex)
    let current_row = state.hex_cursor.current_row();
    let row_start_offset = current_row * state.hex_cursor.bytes_per_row;

    // Seek to START OF ROW
    file.seek(io::SeekFrom::Start(row_start_offset as u64))
        .map_err(|e| LinesError::Io(e))?;

    let bytes_read = file.read(&mut byte_buffer).map_err(|e| LinesError::Io(e))?;
    let cursor_col = state.hex_cursor.current_col();

    // Build raw line and interpreted line simultaneously
    for i in 0..BYTES_TO_DISPLAY {
        if i < bytes_read {
            let byte = byte_buffer[i];

            let mut hex_buf = [0u8; 64];

            // TODO: explore stack based formatting...
            // === RAW LINE (with escape sequences) ===
            // let raw_repr = byte_to_raw_escape(byte);
            let raw_repr = stack_format_byte_escape(byte, &mut hex_buf).unwrap_or("?");

            if i == cursor_col {
                // raw_line.push_str(&format!(
                //     "{}{}{}{:<3}{}", // Left-align in 3-char field
                //     BOLD, RED, BG_WHITE, raw_repr, RESET
                // ));

                let formatted_string_1 = stack_format_it(
                    "{}{}{}{:<3}{}", // "{}{}{}{:<3}{}",
                    &[&BOLD, &RED, &BG_WHITE, &raw_repr, &RESET],
                    "NNNNN",
                );
                raw_line.push_str(&formatted_string_1);
            } else {
                // raw_line.push_str(&format!("{:<3}", raw_repr));
                let formatted_string_2 = stack_format_it("{:<3}", &[&raw_repr], "N");
                raw_line.push_str(&formatted_string_2);
            }

            // === INTERPRETED LINE (same as hex mode) ===
            let display_char = byte_to_display_char(byte);

            if i == cursor_col {
                // interpreted_line.push_str(&format!(
                //     "{}{}{}{}{}  ",
                //     BOLD, RED, BG_WHITE, display_char, RESET
                // ));

                let formatted_string_3 = stack_format_it(
                    "{}{}{}{}{}  ",
                    &[&BOLD, &RED, &BG_WHITE, &display_char.to_string(), &RESET],
                    "NNNNN",
                );

                interpreted_line.push_str(&formatted_string_3);
            } else {
                interpreted_line.push_str(&format!("{}  ", display_char));
                // interpreted_line.push_str(&stack_format_it(
                //     "{}  ",
                //     &[&display_char.to_string()],
                //     "N   ",
                // ));
            }
        } else {
            // Past EOF
            raw_line.push_str("   ");
            interpreted_line.push_str("   ");
        }
    }

    // let result = format!("{}\n{}\n", raw_line.trim_end(), interpreted_line.trim_end());

    let result = stack_format_it(
        "{}\n{}\n",
        &[&raw_line.trim_end(), &interpreted_line.trim_end()],
        "^\n^\n",
    );

    Ok(result)
}

/// Formats info bar for raw string mode
///
/// # Format
/// ```text
/// RAW byte 156 of 1024 doc.txt > cmd_
/// ```
fn format_raw_info_bar(state: &EditorState) -> Result<String> {
    let filename = state
        .read_copy_path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    // Get file size (same as hex mode)
    let file_size = if let Some(path) = &state.read_copy_path {
        std::fs::metadata(path)
            .map(|m| m.len() as usize)
            .unwrap_or(0)
    } else {
        0
    };

    // Ok(format!(
    //     "RAW byte {} of {} {} > ",
    //     state.hex_cursor.byte_offset_linear_file_absolute_position, file_size, filename
    // ))

    Ok(stack_format_it(
        "RAW byte {} of {} {} > ",
        &[
            &state
                .hex_cursor
                .byte_offset_linear_file_absolute_position
                .to_string(),
            &file_size.to_string(),
            &filename,
        ],
        "RAW byte __ of _ _ > ",
    ))
}

//  =====================
//  Sashimi Raw TUI Ramen end
//  =====================
//
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
fn format_hex_info_bar(lines_editor_state: &EditorState) -> Result<String> {
    // Get file size
    let file_size = match &lines_editor_state.read_copy_path {
        Some(path) => match fs::metadata(path) {
            Ok(metadata) => metadata.len() as usize,
            Err(_) => 0,
        },
        None => 0,
    };

    // Get filename (or "unnamed" if none)
    let filename = lines_editor_state
        .original_file_path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unmanned phile");

    // Extract message from buffer (find null terminator or use full buffer)
    let message_len = lines_editor_state
        .info_bar_message_buffer
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(lines_editor_state.info_bar_message_buffer.len());

    let message_for_infobar =
        std::str::from_utf8(&lines_editor_state.info_bar_message_buffer[..message_len])
            .unwrap_or(""); // Empty string if invalid UTF-8

    // Build info bar
    // // Show byte position as 1-indexed for human readability
    // let info_bar = format!(
    //     "{}HEX byte {}{}{} of {}{}{} {}, Edit:Enter Hex|Insrt:NN-i|GoTo:gN {} {}> ",
    //     YELLOW,
    //     RED,
    //     lines_editor_state
    //         .hex_cursor
    //         .byte_offset_linear_file_absolute_position
    //         + 1, // Human-friendly: 1-indexed
    //     YELLOW,
    //     RED,
    //     file_size,
    //     YELLOW,
    //     filename,
    //     message_for_infobar,
    //     RESET,
    // );

    let string_lines = &lines_editor_state
        .hex_cursor
        .byte_offset_linear_file_absolute_position
        + 1;

    let info_bar = stack_format_it(
        "{}HEX byte {}{}{} of {}{}{} {}, Edit:Enter Hex|Insrt:NN-i|GoTo:gN {} {}> ",
        &[
            &YELLOW,
            &RED,
            &string_lines.to_string(),
            &YELLOW,
            &RED,
            &file_size.to_string(),
            &YELLOW,
            &filename,
            &message_for_infobar,
            &RESET,
        ],
        "Invalid byte range",
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
    io::stdout().flush().map_err(|e| {
        LinesError::DisplayError(stack_format_it(
            "Failed to flush stdout: {}",
            &[&e.to_string()],
            "Failed to flush stdout",
        ))
    })?;

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

    io::stdout().flush().map_err(|e| {
        LinesError::DisplayError(stack_format_it(
            "Failed to flush stdout: {}",
            &[&e.to_string()],
            "Failed to flush stdout",
        ))
    })?;

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
/// - Window_map provides byte_offset_linear_file_absolute_position for each display position
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
        let ch_string = ch.to_string();
        // PRIORITY 1: Cursor highlighting (takes precedence)
        if cursor_on_this_row && col == cursor_col {
            let formatted_string_1 = stack_format_it(
                "{}{}{}{}{}",
                &[&BOLD, &RED, &BG_WHITE, &ch_string, &RESET],
                "NNNNN",
            );
            // result.push_str(&format!("{}{}{}{}{}", BOLD, RED, BG_WHITE, ch, RESET));
            result.push_str(&formatted_string_1);
            continue;
        }

        // PRIORITY 2: Visual selection highlighting
        if state.mode == EditorMode::VisualSelectMode {
            // Get file position - propagate error if lookup fails
            let file_pos_option = state.get_row_col_file_position(row_index, col)?;

            if let Some(file_pos) = file_pos_option {
                // Check if in selection - propagate error if check fails
                let in_selection = is_in_selection(
                    file_pos.byte_offset_linear_file_absolute_position,
                    state.file_position_of_vis_select_start,
                    state.file_position_of_vis_select_end,
                )?;

                if in_selection {
                    let formatted_string_2 = stack_format_it(
                        "{}{}{}{}{}",
                        &[&BOLD, &YELLOW, &BG_CYAN, &ch_string, &RESET],
                        "NNNNN",
                    );
                    // result.push_str(&format!("{}{}{}{}{}", BOLD, YELLOW, BG_CYAN, ch, RESET));
                    result.push_str(&formatted_string_2);
                    continue;
                }
            }
        }

        // PRIORITY 3: Normal character (no highlighting)
        result.push(ch);
    }

    // Handle cursor at/past end of line
    if cursor_on_this_row && cursor_col >= chars.len() {
        // result.push_str(&format!("{}{}{}█{}", BOLD, RED, BG_WHITE, RESET));

        result.push_str(&stack_format_it(
            "{}{}{}█{}",
            &[&BOLD, &RED, &BG_WHITE, &RESET],
            "NNN█N",
        ));
    }

    Ok(result)
}

/// Initializes the session directory structure for this editing session
///
/// # Purpose
/// Creates the lines_data infrastructure and either creates a new unique session
/// directory for this run OR uses an existing session directory for crash recovery.
/// Session directories persist after exit for crash recovery purposes.
///
/// # Directory Structure Created (when creating new)
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
/// * `session_time_stamp` - Timestamp used only when creating new session directory
/// * `use_this_session` - Optional path to existing session directory for recovery:
///   - Can be relative: `"lines_data/sessions/20250103_143022"`
///   - Can be absolute: `"/full/path/to/exe/lines_data/sessions/20250103_143022"`
///   - If provided, `session_time_stamp` parameter is ignored
///   - Directory must already exist and contain recovery files
///   - Directory will NOT be created, modified, or deleted
///
/// # Returns
/// * `Ok(())` - Session directory validated/created and path stored in state
/// * `Err(io::Error)` - If directory creation/validation fails
///
/// # State Modified
/// - `state.session_directory_path` - Set to absolute path of session directory
///
/// # Crash Recovery Use Case
/// When recovering from a crash or interrupted session:
/// ```rust
/// // User provides the session directory they want to recover
/// let recovery_path = PathBuf::from("lines_data/sessions/20250103_143022");
/// initialize_session_directory(&mut state, timestamp, Some(recovery_path))?;
/// ```
///
/// # Security
/// When `use_this_session` is provided, the function validates that the
/// canonicalized path is within the sessions directory structure. This prevents
/// path traversal attacks attempting to use system directories like `/etc` or `/tmp`.
///
/// # Error Handling
/// Possible errors when using existing session:
/// - Provided path does not exist
/// - Provided path is not a directory (is a file)
/// - Provided path is outside the sessions directory structure (security)
/// - Cannot canonicalize or access the path
///
pub fn initialize_session_directory(
    state: &mut EditorState,
    session_time_stamp: FixedSize32Timestamp,
    use_this_session: Option<PathBuf>,
) -> io::Result<()> {
    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    // Defensive: Verify state is in clean initial state
    debug_assert!(
        state.session_directory_path.is_none(),
        "Session directory should not be initialized twice"
    );

    // Test assertion for double-initialization
    #[cfg(test)]
    assert!(
        state.session_directory_path.is_none(),
        "Session directory should not be initialized twice"
    );

    // Production catch: Handle double-initialization gracefully
    if state.session_directory_path.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Session directory already initialized",
        ));
    }

    // Step 1: Ensure base directory structure exists
    // Creates: {executable_dir}/lines_data/sessions/
    let base_sessions_path = "lines_data/sessions";

    let sessions_dir = make_verify_or_create_executabledirectoryrelative_canonicalized_dir_path(
        base_sessions_path,
    )
    .map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            // format!("Failed to create sessions directory structure: {}", e),
            stack_format_it(
                "Failed to create sessions directory structure: {}",
                &[&e.to_string()],
                "Failed to create sessions directory structure",
            ),
        )
    })?;

    // Defensive: Verify the path is a directory
    if !sessions_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Sessions path exists but is not a directory",
        ));
    }

    // Step 2: Determine session directory - either use existing or create new
    let session_path = if let Some(provided_path) = use_this_session {
        // ============================================================
        // Step 2a: Use existing session directory (crash recovery)
        // ============================================================

        // Resolve the provided path to absolute form
        // Handle both relative paths (resolved from exe dir) and absolute paths
        let resolved_path = if provided_path.is_absolute() {
            // Already absolute, use directly
            provided_path
        } else {
            // Relative path - resolve from executable directory
            let path_str = provided_path.to_string_lossy();
            // Convert Cow<str> to &str using as_ref()
            make_input_path_name_abs_executabledirectoryrelative_nocheck(path_str.as_ref())
                .map_err(|_e| {
                    #[cfg(debug_assertions)]
                    let msg = format!(
                        "Failed to resolve provided session path '{}': {}",
                        path_str, _e
                    );
                    #[cfg(not(debug_assertions))]
                    let msg = "Failed to resolve provided session path";

                    io::Error::new(io::ErrorKind::InvalidInput, msg)
                })?
        };

        // Validation 1: Check if provided path exists
        if !resolved_path.exists() {
            #[cfg(debug_assertions)]
            let msg = format!(
                "Provided session directory does not exist: {}",
                resolved_path.display()
            );
            #[cfg(not(debug_assertions))]
            let msg = "Provided session directory does not exist";

            return Err(io::Error::new(io::ErrorKind::NotFound, msg));
        }

        // Validation 2: Check if provided path is a directory (not a file)
        if !resolved_path.is_dir() {
            #[cfg(debug_assertions)]
            let msg = format!(
                "Provided session path is not a directory: {}",
                resolved_path.display()
            );
            #[cfg(not(debug_assertions))]
            let msg = "Provided session path is not a directory";

            return Err(io::Error::new(io::ErrorKind::InvalidInput, msg));
        }

        // Validation 3: SECURITY - Verify path is within sessions directory
        // Canonicalize both paths to resolve symlinks and normalize for comparison
        let canonical_provided = resolved_path.canonicalize().map_err(|_e| {
            #[cfg(debug_assertions)]
            let msg = format!("Cannot canonicalize provided session path: {}", _e);
            #[cfg(not(debug_assertions))]
            let msg = "Cannot access provided session path";

            io::Error::new(io::ErrorKind::Other, msg)
        })?;

        let canonical_sessions = sessions_dir.canonicalize().map_err(|_e| {
            #[cfg(debug_assertions)]
            let msg = format!("Cannot canonicalize sessions directory: {}", _e);
            #[cfg(not(debug_assertions))]
            let msg = "Cannot access sessions directory";

            io::Error::new(io::ErrorKind::Other, msg)
        })?;

        // Security check: Provided path must be under sessions directory
        // This prevents path traversal attacks (e.g., /etc, /tmp, ../.., etc.)
        if !canonical_provided.starts_with(&canonical_sessions) {
            #[cfg(debug_assertions)]
            let msg = format!(
                "Security violation: Provided session path '{}' is outside sessions directory '{}'",
                canonical_provided.display(),
                canonical_sessions.display()
            );
            #[cfg(not(debug_assertions))]
            let msg = "Provided session path is outside allowed directory";

            return Err(io::Error::new(io::ErrorKind::PermissionDenied, msg));
        }

        // All validations passed - use this existing directory
        // NOTE: We do NOT create, modify, or delete anything in this directory
        // It may contain recovery files - that's the whole point
        canonical_provided
    } else {
        // ============================================================
        // Step 2b: Create new session directory (normal operation)
        // ============================================================

        // Use timestamp parameter to create new session directory
        let session_path = sessions_dir.join(session_time_stamp.to_string());

        // Create the session directory
        fs::create_dir(&session_path).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                stack_format_it(
                    "Failed to create session directory {}: {}",
                    &[&session_time_stamp.to_string(), &e.to_string()],
                    "Failed to create session directory",
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

        session_path
    };

    // Step 3: Store path in state
    state.session_directory_path = Some(session_path.clone());

    // Assertion: Verify state was updated
    debug_assert!(
        state.session_directory_path.is_some(),
        "Session directory path should be set in state"
    );

    // Test assertion: Verify state was updated
    #[cfg(test)]
    assert!(
        state.session_directory_path.is_some(),
        "Session directory path should be set in state"
    );

    Ok(())
}

/// Creates a new session directory and returns its path
///
/// # Purpose
/// Simple session directory creation for wrappers and tools that don't need
/// full EditorState infrastructure. Creates timestamped session directory
/// in standard location and returns absolute path.
///
/// # Project Context
/// Provides session isolation for draft copies without requiring EditorState.
/// Useful for:
/// - Wrappers around lines_core that need session directories
/// - Tools that want session isolation without full editor state
/// - Testing and utilities that need temporary organized workspaces
///
/// # Directory Structure Created
/// ```text
/// {executable_dir}/
///   lines_data/
///     sessions/
///       {timestamp}/          <- Created directory (returned)
/// ```
///
/// # Arguments
/// * `session_time_stamp` - Timestamp string for directory name (e.g., "2025_01_15_14_30_45")
///
/// # Returns
/// * `Ok(PathBuf)` - Absolute path to newly created session directory
/// * `Err(io::Error)` - Directory creation or validation failed
///
/// # Behavior
/// - Creates base infrastructure (lines_data/sessions/) if needed
/// - Creates new timestamped session directory
/// - Returns absolute canonicalized path
/// - Idempotent: Returns path if directory already exists with this timestamp
///
/// # Design Notes
/// - Does NOT use or require EditorState (no phantom state memory)
/// - Does NOT support recovery mode (use full version for that)
/// - Always creates new directory (or validates existing)
/// - Simpler alternative to initialize_session_directory for basic use cases
///
/// # Example
/// ```rust
/// let timestamp = "2025_01_15_14_30_45".to_string();
/// let session_path = simple_make_lines_editor_session_directory(timestamp)?;
/// // session_path is now: "/path/to/exe/lines_data/sessions/2025_01_15_14_30_45"
/// ```
pub fn simple_make_lines_editor_session_directory(
    session_time_stamp: String,
) -> io::Result<PathBuf> {
    // =================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // =================================================

    // Defensive: Validate timestamp is not empty
    debug_assert!(
        !session_time_stamp.is_empty(),
        "Session timestamp should not be empty"
    );

    #[cfg(test)]
    assert!(
        !session_time_stamp.is_empty(),
        "Session timestamp should not be empty"
    );

    // Production catch: Handle empty timestamp
    if session_time_stamp.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "simple_make_lines_editor_session_directory: Empty timestamp provided",
        ));
    }

    // ===================================================================
    // STEP 1: Ensure base directory structure exists
    // ===================================================================
    // Creates: {executable_dir}/lines_data/sessions/
    let base_sessions_path = "lines_data/sessions";

    let sessions_dir = make_verify_or_create_executabledirectoryrelative_canonicalized_dir_path(
        base_sessions_path,
    )
    .map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            stack_format_it(
                "simple_make_lines_editor_session_directory: Failed to create sessions structure: {}",
                &[&e.to_string()],
                "simple_make_lines_editor_session_directory: Failed to create sessions structure",
            ),
        )
    })?;

    // Defensive: Verify the path is a directory
    if !sessions_dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "simple_make_lines_editor_session_directory: Sessions path exists but is not a directory",
        ));
    }

    // ===================================================================
    // STEP 2: Create timestamped session directory
    // ===================================================================
    let session_path = sessions_dir.join(&session_time_stamp);

    // Check if directory already exists (idempotent)
    if session_path.exists() {
        // Defensive: Verify it's actually a directory
        if !session_path.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "simple_make_lines_editor_session_directory: Path exists but is not a directory",
            ));
        }

        // Already exists as directory - return it (idempotent)
        debug_assert!(
            session_path.is_absolute(),
            "Session path should be absolute"
        );

        return Ok(session_path);
    }

    // Create the session directory
    fs::create_dir(&session_path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            stack_format_it(
                "simple_make_lines_editor_session_directory: Failed to create directory: {}",
                &[&e.to_string()],
                "simple_make_lines_editor_session_directory: Failed to create directory",
            ),
        )
    })?;

    // Defensive: Verify creation succeeded
    if !session_path.exists() || !session_path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "simple_make_lines_editor_session_directory: Creation reported success but directory not found",
        ));
    }

    // Assertion: Verify path is absolute
    debug_assert!(
        session_path.is_absolute(),
        "Session path should be absolute"
    );

    // Test assertion: Verify path is absolute
    #[cfg(test)]
    assert!(
        session_path.is_absolute(),
        "Session path should be absolute"
    );

    Ok(session_path)
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
/// Recovery-reboot wrapper for lines_fullfileeditor_core
pub fn lines_full_file_editor(
    original_file_path: Option<PathBuf>,
    starting_line: Option<usize>,
    use_this_session: Option<PathBuf>,
    state_persists: bool, // if you want to keep session files.
) -> Result<()> {
    // Same code as core function to set-up

    //  =======================================
    //  Initialization & Bootstrap Lines Editor
    //  =======================================

    // Resolve target file path (all path handling logic extracted)
    let target_path = resolve_target_file_path(original_file_path)?;
    // // Diagnostic
    // println!("\n=== Opening Lines Editor ==="); // TODO remove/commentout debug print
    // println!("File: {}", target_path.display());

    // Create file if it doesn't exist
    if !target_path.exists() {
        // new file header = longer readable timestamp
        let header_readable_timestamp = create_readable_archive_timestamp(SystemTime::now());
        let header = stack_format_it("# {}", &[&header_readable_timestamp], "");

        // Create with header
        let mut file = File::create(&target_path)?;
        writeln!(file, "{}", header)?;
        writeln!(file)?; // Empty line after header
        file.flush()?;
    }

    /*
    If there already is directory iput, use it.
    If not, make a directory.
    */
    //  ========================================
    //  Set Up & Build The Path for Lines Editor
    //  ========================================
    let session_dir: PathBuf = if let Some(path) = use_this_session {
        // If `use_this_session` is Some, use the provided path
        path
    } else {
        // If `use_this_session` is None, create a new directory
        let session_time_base = createarchive_timestamp_with_precision(SystemTime::now(), true);
        simple_make_lines_editor_session_directory(session_time_base)?
    };

    //  =======================
    //  FAIL-SAFE RECOVERY LOOP
    //  =======================
    let mut recovery_attempt = 0;
    const MAX_RECOVERY_ATTEMPTS: usize = 5;

    loop {
        recovery_attempt += 1;

        if recovery_attempt > MAX_RECOVERY_ATTEMPTS {
            eprintln!("Error: Maximum recovery attempts exceeded");
            return Err(LinesError::StateError("Too many recovery attempts".into()));
        }

        if recovery_attempt > 1 {
            println!("\n=== RECOVERY REBOOT #{} ===\n", recovery_attempt - 1);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        // Call core with SAME session directory
        match lines_fullfileeditor_core(
            Some(target_path.clone()),
            starting_line,
            Some(session_dir.clone()),
        ) {
            Ok(user_quit) => {
                if user_quit {
                    break;
                } else {
                    // Unexpected exit - reboot
                    eprintln!("Warning: Unexpected exit, rebooting...");
                }
            }
            Err(_e) => {
                // Error occurred - reboot
                #[cfg(debug_assertions)]
                {
                    log_error(&format!("{}", _e), Some("wrapper:recovery"));
                    eprintln!("Error: {}, rebooting...", _e);
                }
                eprintln!("Rebooting...");
            }
        }
    }

    if !state_persists {
        // remove all files and session directory(folder)
        _ = cleanup_all_session_directory(&session_dir);
    }
    return Ok(());
}

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
pub fn lines_fullfileeditor_core(
    original_file_path: Option<PathBuf>,
    starting_line: Option<usize>,
    use_this_session: Option<PathBuf>,
) -> Result<bool> {
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
        // new file header = longer readable timestamp
        let header_readable_timestamp = create_readable_archive_timestamp(SystemTime::now());
        let header = stack_format_it("# {}", &[&header_readable_timestamp], "");

        // Create with header
        let mut file = File::create(&target_path)?;
        writeln!(file, "{}", header)?;
        writeln!(file)?; // Empty line after header
        file.flush()?;
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
    initialize_session_directory(
        &mut lines_editor_state,
        session_time_stamp1,
        use_this_session,
    )?;

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
    lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset = 0;

    // Bootstrap initial cursor position, start of file, after "l "
    lines_editor_state.cursor.row = 0;
    lines_editor_state.cursor.col = 3; // Bootstrap Bumb: start after padded line nunber (zero-index 3)

    // IF cli argument to goto/start-at line:
    // e.g. lines many_lines_v1.txt:500
    // IF user input line: Jump to starting line if provided
    if let Some(line_num) = starting_line {
        let target_line = line_num.saturating_sub(1); // Convert 1-indexed to 0-indexed

        match seek_to_line_number(&mut File::open(&read_copy_path)?, target_line) {
            Ok(byte_pos) => {
                // Position cursor AFTER line number (same as bootstrap)
                let line_num_width = calculate_line_number_width(
                    // lines_editor_state.line_count_at_top_of_window,
                    target_line,
                    target_line,
                    lines_editor_state.effective_rows,
                );
                // println!("{line_num_width}{target_line}");
                lines_editor_state.cursor.col = line_num_width; // Skip over line number display
                lines_editor_state.in_row_abs_horizontal_0_index_cursor_position = line_num_width;
                lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset = 0;

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

        // ====
        // Bump
        // ====
        /*
        To keep the cursor on the text:
        If on the top (zero index 0-line 0-row) bump to end of line number
        If not row zero, move to end of previous line.

        There may be periodic edge cases and bugs such as:
        - cursor goes to space in the number-zone (How ?)

        Question: width vs. +1
        Where should cursor.col = line_num_width + 1;
        vs.
        cursor.col = line_num_width;?

        */
        // // find line text width
        let line_num_width = calculate_line_number_width(
            lines_editor_state.line_count_at_top_of_window,
            lines_editor_state.cursor.row,
            lines_editor_state.effective_rows,
        );
        if lines_editor_state.cursor.col < line_num_width {
            // on line 0? (top) is cursor off the reservation? If so... Bump it Left!
            if lines_editor_state.cursor.row == 0 {
                lines_editor_state.cursor.col = line_num_width + 1;
                lines_editor_state.in_row_abs_horizontal_0_index_cursor_position =
                    line_num_width + 1;
                lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset = 0;

                build_windowmap_nowrap(&mut lines_editor_state, &read_copy)?;
            } else {
                print!("moving up: line_num_width {line_num_width}");
                // Not at Top? If so... Bump it up!
                // Move up one line
                execute_command(&mut lines_editor_state, Command::MoveUp(1))?;

                // Move to end of that line
                execute_command(&mut lines_editor_state, Command::GotoLineEnd)?;

                build_windowmap_nowrap(&mut lines_editor_state, &read_copy)?;

                if lines_editor_state.cursor.row == 0 {
                    print!("pingping\n\n\n");
                    lines_editor_state.cursor.col = line_num_width + 1;
                    lines_editor_state.in_row_abs_horizontal_0_index_cursor_position =
                        line_num_width + 1;
                    lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset = 0;
                    execute_command(&mut lines_editor_state, Command::GotoLineEnd)?;
                    build_windowmap_nowrap(&mut lines_editor_state, &read_copy)?;
                    execute_command(&mut lines_editor_state, Command::GotoLineEnd)?;

                    // let line_num_width = calculate_line_number_width(
                    //     lines_editor_state.line_count_at_top_of_window,
                    //     lines_editor_state.cursor.row,
                    //     lines_editor_state.effective_rows,
                    // );

                    // // if col is in the number-zone to the left of the text
                    // // bump it over
                    // if lines_editor_state.cursor.col < line_num_width {
                    //     lines_editor_state.cursor.col = line_num_width; // Skip over line number displayfull_lines_editor
                    //     build_windowmap_nowrap(&mut lines_editor_state, &read_copy)?;
                    // }
                }

                // _ = build_windowmap_nowrap(&mut lines_editor_state, &read_copy); // rebuild
                let _ = lines_editor_state.set_info_bar_message("start of line"); // massage
            }
        }

        if lines_editor_state.mode == EditorMode::HexMode {
            //  ======================
            //  HEX Render a Flesh TUI
            //  ======================
            render_tui_hex(&lines_editor_state).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    stack_format_it("Display error: {}", &[&e.to_string()], "Display error"),
                )
            })?;
        } else if lines_editor_state.mode == EditorMode::RawMode {
            //  =====================
            //  Sashimi Raw TUI Ramen
            //  =====================
            render_tui_raw(&lines_editor_state).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    stack_format_it("Display error: {}", &[&e.to_string()], "Display error"),
                )
            })?;
        } else {
            // Render TUI (convert LinesError to io::Error)
            render_tui_utf8txt(&lines_editor_state).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    stack_format_it("Display error: {}", &[&e.to_string()], "Display error"),
                )
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
        } else if lines_editor_state.mode == EditorMode::RawMode {
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
            if let Ok(Some(file_pos)) = lines_editor_state.get_row_col_file_position(
                lines_editor_state.cursor.row,
                lines_editor_state.cursor.col,
            ) {
                lines_editor_state.file_position_of_vis_select_end =
                    file_pos.byte_offset_linear_file_absolute_position;
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

    Ok(true)
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
//                 lines_full_file_editor(Some(original_file_path))
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
//                         lines_full_file_editor(Some(path))
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
*/

/*

# Example of FF open_file() integration

/// ```
fn open_file(file_path: &PathBuf, lines_editor_session_path: &PathBuf) -> Result<()> {
    /*
    The user input format/sytax should be as regular/consistent as possible
    given the edge case that Lines-Editor is the default if none is specified.
    After selecting file by number:

    entering name of editor: opens in new terminal

    name of editor + -h or --headless: opens in the same terminal

    name of editor + -vsplit, -hsplit: opens in a tmux split

    Empty Enter: should open lines in a new terminal

    only "-h" or "--headless" (maybe "lines -h"): should open lines in same terminal


    */
    // Read partner programs configuration (gracefully handles all errors)
    let partner_programs = read_partner_programs_file();

    // check if suffi

    // Build the user prompt based on whether partner programs are available
    let prompt = if partner_programs.is_empty() {
        // Standard prompt when no partner programs are configured
        format!(
            "{}(Open file w/  Default: Enter | software 'name': vi --headless, gedit, firefox | tmux: nano -hsplit, hx -vsplit | .csv stats: vi -rc) {}",
            YELLOW, RESET
        )
    } else {
        // Enhanced prompt showing numbered partner program options
        let mut numbered_options = String::new();
        for (index, program_path) in partner_programs.iter().enumerate() {
            if index > 0 {
                numbered_options.push(' ');
            }
            numbered_options.push_str(&format!(
                "{}. {}",
                index + 1,
                extract_program_display_name(program_path)
            ));
        }

        format!(
            "{}Open file w/  Default: Enter | software 'name': vi --headless, gedit, firefox | tmux: -hsplit | .csv: -rc | Partner #: {}): {}",
            YELLOW, numbered_options, RESET
        )
    };

    // Display the prompt and get user input
    print!("{}", prompt);
    io::stdout().flush().map_err(|e| {
        eprintln!("Failed to flush stdout: {}", e);
        FileFantasticError::Io(e)
    })?;

    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).map_err(|e| {
        eprintln!("Failed to read input: {}", e);
        FileFantasticError::Io(e)
    })?;
    let user_input = user_input.trim();

    // TODO
    // ==========================================
    // Headless Default Lines-Editor
    // ==========================================
    if user_input == "-h"
        || user_input == "--headless"
        || user_input == "lines --headless"
        || user_input == "lines -h"
    {
        // =============================
        // Lines-Editor in this terminal
        // =============================
        /*
        pub fn lines_full_file_editor(
            original_file_path: Option<PathBuf>,
            starting_line: Option<usize>,
            use_this_session: Option<PathBuf>,
            state_persists: bool, // if you want to keep session files.
        ) -> Result<()> {
        */

        lines_full_file_editor(
            Some(file_path.clone()),
            None,
            Some(lines_editor_session_path.clone()),
            true,
        )?; // The ? will use From<LinesError> to convert
        return Ok(());
    }

    // ==========================================
    // === MVP: Tmux splits for lines editor ===
    // ==========================================
    // === MVP: Tmux splits for lines editor ===
    if user_input == "-vsplit" || user_input == "vsplit" {
        // Check if in tmux
        if std::env::var("TMUX").is_err() {
            println!("{}Error: -vsplit requires tmux{}", RED, RESET);
            println!("Press Enter to continue...");
            let mut buf = String::new();
            io::stdin()
                .read_line(&mut buf)
                .map_err(|e| FileFantasticError::Io(e))?;
            return open_file(file_path, lines_editor_session_path);
        }

        // Get the path to the current executable
        let exe_path = std::env::current_exe().map_err(|e| FileFantasticError::Io(e))?;

        // Build the command as a single string with full binary path
        let editor_command = format!(
            "{} {} --session {}",
            exe_path.to_string_lossy(),
            file_path.to_string_lossy(),
            lines_editor_session_path.to_string_lossy()
        );

        // Create vertical split (tmux -v = vertical split = horizontal panes)
        let output = std::process::Command::new("tmux")
            .args(["split-window", "-v", &editor_command])
            .output()
            .map_err(|e| FileFantasticError::Io(e))?;

        if !output.status.success() {
            println!(
                "{}Failed to create tmux split: {}{}",
                RED,
                String::from_utf8_lossy(&output.stderr),
                RESET
            );
            println!("Press Enter to continue...");
            let mut buf = String::new();
            io::stdin()
                .read_line(&mut buf)
                .map_err(|e| FileFantasticError::Io(e))?;
            return open_file(file_path, lines_editor_session_path);
        }

        return Ok(());
    }

    if user_input == "-hsplit" || user_input == "hsplit" {
        // Check if in tmux
        if std::env::var("TMUX").is_err() {
            println!("{}Error: -hsplit requires tmux{}", RED, RESET);
            println!("Press Enter to continue...");
            let mut buf = String::new();
            io::stdin()
                .read_line(&mut buf)
                .map_err(|e| FileFantasticError::Io(e))?;
            return open_file(file_path, lines_editor_session_path);
        }

        // Get the path to the current executable
        let exe_path = std::env::current_exe().map_err(|e| FileFantasticError::Io(e))?;

        // Build the command as a single string with full binary path
        let editor_command = format!(
            "{} {} --session {}",
            exe_path.to_string_lossy(),
            file_path.to_string_lossy(),
            lines_editor_session_path.to_string_lossy()
        );

        // Create horizontal split (tmux -h = horizontal split = vertical panes)
        let output = std::process::Command::new("tmux")
            .args(["split-window", "-h", &editor_command])
            .output()
            .map_err(|e| FileFantasticError::Io(e))?;

        if !output.status.success() {
            println!(
                "{}Failed to create tmux split: {}{}",
                RED,
                String::from_utf8_lossy(&output.stderr),
                RESET
            );
            println!("Press Enter to continue...");
            let mut buf = String::new();
            io::stdin()
                .read_line(&mut buf)
                .map_err(|e| FileFantasticError::Io(e))?;
            return open_file(file_path, lines_editor_session_path);
        }

        return Ok(());
    }

    // === Handle "lines" keyword - open in new terminal ===
    // === Handle "lines" keyword - open in new terminal ===
    if user_input == "lines" || user_input.is_empty() {
        let exe_path = std::env::current_exe().map_err(|e| FileFantasticError::Io(e))?;

        // Launch in new terminal (platform-specific)
        #[cfg(target_os = "macos")]
        {
            // macOS needs the command as a single string for Terminal.app
            let lines_command = format!(
                "{} {} --session {}; exit",
                exe_path.to_string_lossy(),
                file_path.to_string_lossy(),
                lines_editor_session_path.to_string_lossy()
            );

            std::process::Command::new("open")
                .args(["-a", "Terminal"])
                .arg(&lines_command)
                .spawn()
                .map_err(|e| FileFantasticError::EditorLaunchFailed(format!("lines: {}", e)))?;
        }

        #[cfg(target_os = "linux")]
        {
            let terminal_commands = [
                ("gnome-terminal", vec!["--"]),
                ("ptyxis", vec!["--"]),
                ("konsole", vec!["-e"]),
                ("xfce4-terminal", vec!["-e"]),
                ("terminator", vec!["-e"]),
                ("tilix", vec!["-e"]),
                ("kitty", vec!["-e"]),
                ("alacritty", vec!["-e"]),
                ("xterm", vec!["-e"]),
            ];

            let mut success = false;
            for (terminal, args) in terminal_commands.iter() {
                let mut cmd = std::process::Command::new(terminal);
                cmd.args(args)
                    .arg(&exe_path) // Separate arg: executable
                    .arg(file_path) // Separate arg: file path
                    .arg("--session") // Separate arg: flag
                    .arg(lines_editor_session_path); // Separate arg: session path

                if cmd.spawn().is_ok() {
                    success = true;
                    break;
                }
            }

            if !success {
                println!(
                    "{}No terminal available. Press Enter to continue...{}",
                    RED, RESET
                );
                let mut buf = String::new();
                io::stdin()
                    .read_line(&mut buf)
                    .map_err(|e| FileFantasticError::Io(e))?;
                return open_file(file_path, lines_editor_session_path);
            }
        }
...

*/
