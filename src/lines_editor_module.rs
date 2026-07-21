//! lines_editor_module.rs
//! lines is minimal text editor
//! test files in: src/tests.rs
//!
//! Originally lines used a byte-to-tui lookup table to interface TUI
//! and user actions and the file bytes. Now that lookup functionality is done
//! using a virtual-lookup-table calculated on the fly to conserve memory.
//!
//! There are multiple types of 'positions' and 'coordinates' and making
//! these clear is important.
//!
//! # Coordinate Spaces —  reference for cursor/position math
//!
//! This editor displays UTF-8 text in a non-wrapping, line-by-line terminal
//! window. UTF-8 forces three things apart that were one and the same in the
//! ASCII era:
//!
//! ```text
//!   bytes        ≠   characters      ≠   visual terminal cells
//!   (1–4 bytes       (one Unicode         (1 cell normal,
//!    per char)        scalar value)        2 cells for double-width: CJK/emoji)
//! ```
//!
//! Conflating these is THE bug class this module exists to prevent. Every
//! function that touches a position MUST state, in its own doc, which of the
//! spaces below each input and output uses. Do not assume; the same integer
//! (e.g. "75") is a different location in each space.
//!
//! ## The six coordinate spaces
//!
//! | # | Name (this doc)            | Unit            | Measured from        | Stored in (canonical fields)                                             |
//! |---|----------------------------|-----------------|----------------------|--------------------------------------------------------------------------|
//! | 1 | **file byte (absolute)**   | bytes           | start of the file    | `FilePosition::byte_offset_linear_file_absolute_position`, `file_position_of_topline_start`, `windowmap_line_byte_start_end_position_pairs`, `file_position_of_vis_select_*` |
//! | 2 | **in-line byte**           | bytes           | start of that line   | `FilePosition::byte_in_line`                                             |
//! | 3 | **line number**            | lines (0-idx)   | start of the file    | `FilePosition::line_number`, `line_count_at_top_of_window`               |
//! | 4 | **in-line char index**     | characters      | start of that line   | `tui_window_horizontal_utf8txt_line_char_offset` (the horizontal scroll) |
//! | 5 | **visual cell column**     | terminal cells  | left edge of the row | `cursor.tui_visual_col`; window width is `effective_cols`                |
//! | 6 | **TUI display row**        | rows (0-idx)    | top of the window    | `cursor.tui_row`; window height is `effective_rows`                      |
//!
//! Definitions in detail:
//!
//! - **#1 file byte (absolute):** linear byte offset into the file. The ground
//!   truth for *where an edit happens*. Example: the last char of a line might
//!   be file byte 8659.
//!
//! - **#2 in-line byte:** byte offset of a position measured from the start of
//!   its own line. `file_byte = line_start_byte + line_byte`. For a multibyte
//!   character this is the offset of its FIRST byte.
//!
//! - **#3 line number:** which line of the file (0-indexed internally; the info
//!   bar shows it +1 for humans). The display row `r` shows file line
//!   `line_count_at_top_of_window + r`.
//!
//! - **#4 in-line char index:** the Nth UTF-8 character of a line, counting one
//!   per character regardless of byte length or cell width. The horizontal
//!   scroll offset lives here: scrolling skips whole CHARACTERS, never cells or
//!   bytes (see `process_line_with_offset`). This is why the offset is in
//!   characters even though the cursor column is in cells.
//!
//! - **#5 visual cell column:** how many terminal CELLS from the row's left edge
//!   a position occupies. ASCII/normal characters take 1 cell; double-width
//!   characters (CJK, emoji) take 2 cells. This column INCLUDES the line-number
//!   prefix: cells `[0, line_num_width)` are the prefix, and the line's first
//!   CONTENT cell is at column `line_num_width`. Under the project's **Option A**
//!   decision, `cursor.tui_visual_col` is a VISUAL column (not a character
//!   count), so it compares directly against `effective_cols` with no
//!   conversion, and the terminal draws the cursor where this column points.
//!
//! - **#6 TUI display row:** the row within the visible window (0 = top content
//!   row). `cursor.tui_row` plus `line_count_at_top_of_window` gives the file
//!   line (#3).
//!
//! ## Sources of truth vs. derived
//!
//! **Sources of truth (the ONLY stored cursor/window state):**
//! - `cursor.tui_row`                                   (#6)
//! - `cursor.tui_visual_col`                            (#5)
//! - `tui_window_horizontal_utf8txt_line_char_offset`   (#4)
//! - `line_count_at_top_of_window`                      (#3, top of window)
//!
//! Plus one file-byte CACHE, rebuilt each window refresh:
//! - `windowmap_line_byte_start_end_position_pairs`     (#1 per display row)
//!
//! **Everything else is DERIVED on demand — never stored as a parallel counter.**
//! Storing redundant per-space counters is forbidden here: a single such counter
//! (`in_row_abs_horizontal_0_index_cursor_position`) drifted out of sync and was
//! removed for exactly that reason. Derive instead:
//! - cursor's file byte (#1), in-line byte (#2), line number (#3):
//!   `EditorState::get_row_col_file_position(tui_row, tui_visual_col)` — the one
//!   canonical converter from (#6, #5) into (#1, #2, #3). It reads the read-copy
//!   file on demand; it does not keep a resident grid.
//! - line-number prefix width `line_num_width` (in cells):
//!   `calculate_line_number_width(line_count_at_top_of_window, tui_row,
//!   effective_rows)`. Cells == characters here because the prefix is ASCII.
//! - content visual column (cells past the prefix):
//!   `content_visual_col = tui_visual_col - line_num_width`.
//! - visual width of the char at / left of the cursor:
//!   `cursor_char_visual_width()` / `char_to_left_visual_width()`.
//!
//! ## How #5 is converted to a file byte (the central derivation)
//!
//! Given a display row and a VISUAL column (#6, #5):
//! 1. `content_visual_col = tui_visual_col - line_num_width`   (strip the prefix)
//! 2. seek to the row's line-start byte (from the windowmap cache),
//! 3. skip `tui_window_horizontal_utf8txt_line_char_offset` CHARACTERS (#4),
//! 4. then walk content, summing VISUAL widths (1 or 2 per char), until the
//!    accumulated width reaches `content_visual_col`; that character's first
//!    byte is the result (#1/#2).
//! Two trailing virtual cells extend the line so the cursor can sit at the
//! newline glyph and one past end-of-line (for appending). `goto_line_end`,
//! `MoveLeft`/`MoveRight`, and the renderer all rely on this being the single
//! conversion path.
//!
//! ## Worked example
//!
//! Line content `ab危ない` shown unscrolled with a 4-cell prefix `"166 "`:
//!
//! ```text
//!   prefix │ a  b  危    な    い
//!   cell:  0123 4  5  6 7  8 9 10 11      ← #5 visual cell column (tui_visual_col)
//!   char:        0  1  2    3    4        ← #4 in-line char index
//!   line%        0  1  2..4 5..7 8..10    ← #2 in-line byte (危/な/い are 3 bytes)
//! ```
//! - Cursor ON `危`: `tui_visual_col = 6`, `content_visual_col = 2`,
//!   char index 2, in-line byte 2. Crossing it rightward advances
//!   `tui_visual_col` by **2** (it is double-width), landing on `な` at cell 8.
//!
//! ## Naming convention (so a name reveals its space)
//!
//! - bytes  → name contains **`byte`**         (`file_byte`, `line_byte`, `byte_in_line`)
//! - line   → **`line_number`** / **`line_count`**
//! - char index → name contains **`char`**     (`..._char_offset`, `line_char_index`)
//! - visual cells → name contains **`visual`** (`tui_visual_col`, `content_visual_col`, `..._visual_width`)
//! - display row → **`tui_row`**
//!
//! When in doubt, prefer a longer name that names the space over a short
//! ambiguous one. The cost of `tui_col` meaning two things was this entire
//! refactor.
/*
See: // TODO: determining ideal default buffer & chunk size


See: "diagnostic" flag for debugging inspection

```
| # | Space | Unit | What it measures | Example (`'` ending line 166) | Lives in (current names) |
|---|-------|------|------------------|-------------------------------|--------------------------|
| 1 | **File byte (absolute)** | bytes | offset from start of file | 8662 | `byte_offset_linear_file_absolute_position`, `file_position_of_topline_start`, `windowmap_line_byte_start_end_position_pairs`, `file_position_of_vis_select_*` |
| 2 | **In-line byte** | bytes | offset from start of the line | 128 | `byte_in_line` |
| 3 | **Line number** | lines | which line in the file (0-idx) | 165 | `line_number`, `line_count_at_top_of_window` |
| 4 | **In-line character index** | characters | Nth UTF-8 char in the line (multibyte = 1) | ~125 | **`tui_window_horizontal_utf8txt_line_char_offset`** (the horizontal scroll lives here) |
| 5 | **Visual cell column** | terminal cells | cells from row left, incl. prefix; double-width = 2 | 75 | **`cursor.tui_visual_col`** (since Option A), compared with `effective_cols` |
| 6 | **TUI display row** | rows | display row within the window | 4 | `cursor.tui_row`, count = `effective_rows` |
```

Derived coordinates vs. source of truth: tui_row, tui_visual_col, & char_offset

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



# 🦀 Rust rules 🦀:
(production-Rust rules)

# 🦀 Rust rules 🦀:
- Always best practice.
- Always extensive doc strings: what the code is doing with project context
- Always clear comments.
- Always cargo tests (where possible).
- Never remove (still-current) documentation.
- Always clear, meaningful, unique names (e.g. variables, functions).
- Always absolute file paths.
- Always error handling.
- Never unsafe code.
- Never use unwrap (in production builds).

Theory and real life are completely different in production code.
Production code must be designed for bitflips, hardware failures, OS errors, etc.  Not pure platonic nirvana.
E.g. According to Linus Torvalds, many or most windows blue screen of death issues in 1990-2010 happened because code did not account for real-world physical hard drive behaviors (including memory errors). According to Steve Gibson (and maybe Designing Data-Intensive Applications: by Martin Kleppmann) many network and database issues are caused by hard-radiation ("cosmic-ray") bitflips.

Power failures happen. Hardware failures happen. Cyberattacks happen. Misbehaving applications happen. Rare edge cases happen. Race conditions happen. Undefined behavior happens. Most code does not have guardrails like either Rust or NASA's 'Power of Ten rules'. Etc.

Much code is only for R&D and internal one-off use, and that is fine. Printing 'hello world' to test should not require elaborate production-hardening. Not all code is or needs to be "production" code. But production code must be smart.

In production: Every line of code will fail eventually. Not 'if': every line of code will fail eventually. Production code is written to handle the failures when, (not 'if,' when) they happen. There is no 'should not fail.' There is no 'can not fail.' Every function will fail. Every call to every function will malfunction. Everything (in production) must be checked and handled so that when (not 'if,' when) these expected errors happen the process does not misbehave, crash, abort, or escalate malfunction, etc.

Empirical processes are more "statistical," less tautological; and "statistical" quickly reaches into the unknown and the undefined.

### Rules of Thumb (there will be exceptions and edge cases):

- Classic ~quote from Sid Meyer's Civilization Game: "The bureaucracy has expanded to meet the needs of the expanding bureaucracy." Bloat and project collapse due to nihilist mismanagement and bad project skills is not new to computer science.

#### Rules Require Context:
- Rules such as 'Don't Repeat Yourself' or 'Separation of Concerns' require a context to be coherent and a compelling reason: Do not repeat yourself IF there is a compelling reason in a clear context. Does aerospace engineering have a blind policy of zero redundancy? No, it does not. Context matters.


#### Flat is better than nested. (Just like in the zen of python.)
- Always consider the flat option first.
- Be wary of ever-more nested structs that claim to infinitely 'separate concerns' for the sake of 'separating concerns.'


#### 'Get [what is] needed, when [it is] needed.':
- Do not load more into state than you need.
- Do not store more information than you need.
- Do not use more storage capacity than you need.
- Do not keep a hold/handle on a file longer than is needed (e.g. forever).


#### Grace Hopper ~"The most damaging phrase in the language is 'we've always done it this way.' The second most damaging is 'storage is cheap.'"
- Be as caring and vigilant about memory-economics as Grace Hopper (who famously walked around with a piece of wire 30 cm long — "a nanosecond" — to make engineers physically feel the cost of waste). Before suggesting the size for a variable (such as apathetically using more memory than is needed) imagine you are suggesting this to Grace Hopper to her face. Only use as much memory as you are absolutely required to use.

- Load what is needed when it is needed: Do not ever load a whole file or line, rarely load a whole anything. Increment and load only what is required pragmatically. Do not fill 'state' with anything that is not both necessary and actually used. Do not insecurity output information broadly in the case of production errors and exceptions (testing and debugging.

- Always use defensive best practice.

- Smoothly handle everything: Every part of every function will eventually fail, if only due to hardware failures or bit-flip noise (both of which are common in reality). As Linus Torvalds has explained, at the root of many 'blue screen of death' incessant window crashes in year's past were hardware irregularities that were not 'handled' by software. Production functions are not pure logic bubbles, they are physical engines that must account for all physically-possible (not just ideally-logically pure) outcomes. If a function gets a result from another function that is (for whatever reason, however logically impossible) malformed and broken, this needs to be handled, e.g. with the classic "let it fail and try again" resiliency model. Every return should be checked for what can be checked, with issues handled (structs and enums can be useful here to define what a healthy return value is allowed to be).

Always error and exception handling: Every part of code, every process, function, and operation will fail at some point, if only because of cosmic-ray bit-flips (which are common), hardware failures, power-supply failures, adversarial attacks, etc. There must always be fail-safe error handling where production-release-build code handles issues and moves on without panic-crashing ever. Every failure must be handled smoothly: let it fail and move on. This does not mean that no function can return an error, nor does this mean that errors cannot be logged or reported. Case by case, a process can be retried or skipped, but the overall program must smoothly continue.

## "Do not stop" in production: Case Handling
Somehow there seems to be no clear vocabulary for 'Do not stop.' In production build code, when you come to something to handle, handle it:
- Handle and move on: Do not halt the program.
- Handle and move on: Do not terminate the program.
- Handle and move on: Do not exit the program.
- Handle and move on: Do not crash the program.
- Handle and move on: Do not panic the program.
- Handle and move on: Do not coredump the program.
- Handle and move on: Do not finish the program.
- Handle and move on: Do not spiral into undefined behavior of the program.
- Handle and move on: Do not stop the program.

## Project-Level Context For Functions, Comments, & Doc-Strings
Comments and docs for functions and groups of functions must include project level information: To paraphrase Jack Welch, "The most dangerous thing in the world is a flawless operation that should never have been done in the first place." For projects, functions are not pure platonic abstractions; the project has a need that the function is or is not meeting. It happens constantly that a function does 'the wrong thing' well and so this 'bug' is never detected when functions are examined in isolation. Project-level (strategic level, architecture level) documentation and logic-level (tactical level) documentation are two different things that must both exist such that discrepancies must be identifiable; Project-level documentation, logic-level documentation, and the code, must align and align with user-needs, real conditions, the results of tests, and future conditions.

Safety, reliability, maintainability, fail-safe, communication-documentation, are the goals: not ideology, aesthetics, popularity, momentum-tradition, bad habits, convenience, nihilism, lazyness, lack of impulse control, cooties, etc.

## No third party libraries (or very very strictly avoid third party libraries where possible).

## Scale: Code should be future-proof and scale well. The Y2K bug was not a wonderful feature, it was a horrendous mistake. Scale and size should be handled in a modular no-load way, not arbitrarily capped so that everything breaks.

## Power-of-10-style Rules of Thumb
We can derive a list of '10 Rust Production Rules' updated for general systems programming in 2026 (derived) from NASA's 2006 'Power of 10' rules that were originally narrowly framed for c for embedded-systems.

These are ideals to be followed where possible and sensible, not absolute pedantic rules:

1. no unsafe stuff:
- no recursion
- no goto
- no pointers
- no preprocessor branching
(Term collision: Technically an 'unsafe code' block in Rust may be required for cases such as naked/assembly code or to interact with a Posix-OS, as in the case of raw-terminals. While use of jargon-'unsafe' blocks should be avoided where possible, the term 'unsafe' does not mean that a specific best-practice rule was violated.)

2. Loops: either firmly bounded or unbounded:
- Upper bound on all normal-loops (to make sure they do **not** keep looping)
- Failsafe for all always-loops to make sure they **do** keep looping (e.g. additional restart layer)

3. Pre-allocate all memory (no dynamic memory allocation)
- Production code should minimize or eliminate use of heap (e.g. very terse error messages that do not leak any user-data)
- Debug and testing often make sense to use heap and this code is not in production-binaries (e.g. detailed error messages)
- Clearly separate lazy-convention from real-need. With tools such as "Buffy'
github.com/lineality/buffy_stack_format_write_module, it is not necessary to use heap for string formatting.

4. Clear Function Scope and Data Ownership:
Part of having a function be 'focused' means knowing if the function is in scope. Functions should be neither swiss-army-knife functions that do too many things, nor scope-less micro-functions that may be doing something that should not be done. Many functions should have a narrow focus and a short length, but definition of actual-project scope functionality must be explicit. Replacing one long clear in-scope function with 50 scope-agnostic generic sub-functions with no clear way of telling if they are in scope or how they interact (e.g. hidden indirect recursion) is dangerous. Rust's ownership and borrowing rules focus on Data ownership and hidden dependencies, making it even less appropriate to scatter borrowing and ownership over a spray of microfunctions purely for the ideology of turning every sub-operation into a microfunction just for the sake of doing so. (See more in rule 9.)

5. 'Case Handling' & Defensive Programming: debug-assert, test-assert, prod safely check & handle, not 'assert!' panic in production

Note: Terminology varies across "error" / "fail" / "exception" / "catch" / "case" et al. The standard terminology is 'error handling' but 'case handling' or 'issue handling' may be a more accurate description, especially where 'error' refers to the output when unable to handle a case (which becomes semantically paradoxical). The goal is that a program will not terminate / halt / end / shut down / stop, etc., or crash / fail / panick / coredump / do undefined-behavior, etc. when 'expected' cases occur. Here production and debugging/testing starkly diverge: during testing you **DO** want/need to see how (and where in the code) the program may 'fail' and where and when cases are encountered. In testing you need to stop with extensive details. In debugging you want to show extensive issue-details. But in production you need to never stop and you need to keep logs memory-terse and privacy-safe.
The proverbial satellite must never fall out of the sky, ever, regardless of how pedantically beautiful the error-message in the ball of flames may have been.

#### Six aspects of case-handlng (Rule 5 of revised 'power of 10' for Rust)
For production-release code:

1 of 6: Check and handle without stop/panic/halt in production

2 of 6: return result (such as Result<T, E>) and smoothly handle "errors" (not halt-panic stopping the application): no assert!() outside of test-only code
Return Result<T, E>, with case/error/exception handling, so long as that is caught somewhere. Only in cases where there is no way (or no where) to handle the error-output should the function always return OK(), failing completely silently (sometimes internal-to-function error logging is best). Allow-to-fail and handle is not the same as no-handling. This is case-by case.

3 of 6: test assert: use #[cfg(test)] assert!() to test production binaries (not in prod builds, not in debug builds)

4 of 6: debug assert: use debug_assert! with  #[cfg(all(debug_assertions, not(test)))] to run tests in debug builds (not in prod, not in test)

5 of 6: Note: #[cfg(debug_assertions)] and debug_assert! ARE active in test builds

6 of 6: Use defensive programming with recovery of all issues at all times
- use cargo tests
- use debug_asserts
- do not leave test-panic assertions in production code
- use no-panic error/case handling in production code
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
# "Assert & Catch-Handle" 3-part System for organizing production behavior, debug behavior, and cargo-test behavior:

A three-part rule of thumb:

1 of 3: For Debug assertions: Only in debug builds, NOT in tests - use: #[cfg(all(debug_assertions, not(test)))]

2 of 3:. For Test assertions: use in test functions themselves, not in the function body (easy to conflict with debug/prod handling)
E.g.
When we run a cargo test:
- The #[cfg(test)] assert compiles and is active
- the cargo-test calls string_concat_list_function()
- an assert! in the abc_function (not in the test) panics immediately inside the abc_function
- abc_function never reaches the production error handling
- so abc_function never returns an Err(...)
- so the cargo-test 'fails' with a panic, not with a cargo-test error result

3 of 3:. Production catches: Always present, return production-safe no-heap terse errors (no panic, no open-ended data exfiltration), with unique error prefixes to identify the function, e.g. 'SCLF error: arg empty' for string_concat_list_function()

Terminology: for consistency, "assert" will be used to mean inducing panic, which is the goal only for cargo-test and debug-builds (not production).

For production, the term "required-condition" will be used (e.g. instead of invariant), to avoid the circular terminology vortex that the original 'Power of 10' is not clear on: ' "assert" handing in production without "asserting" because "asserts" are removed in production... but something is still "asserted" without "assert"ing..." etc.

e.g.
"required-condition" — each rule itself, e.g. len <= 8, Independent of build mode

"requirement-check" — the code that tests a required-condition

"reaction-to-check" — what happens when a check fails. This varies by tier:

Test: panic (assert!) (heap)
Debug: panic (debug_assert!), may also eprintln! (heap)
Production: return terse error (no panic, no print) (no heap)

Note: Buffy may be useful in production error string formatting https://github.com/lineality/buffy_stack_format_write_module

// template/example for check/assert format
//    =================================================
// // Debug-Assert, Test-Asset, Production-Catch-Handle
//    =================================================
// This is not included in production builds
// debug_assert: IS also active during test-builds
// use #[cfg(not(test))] to run in debug-build only: will panic
#[cfg(all(debug_assertions, not(test)))]
debug_assert!(
    INFOBAR_MESSAGE_BUFFER_SIZE > 0,
    "Info bar buffer must have non-zero capacity"
);

// this is included in debug builds AND in test builds
#[cfg(debug_assertions)]
{
xyz
}

// Production safe output example (Buffy is a no-heap alternative)
Err(_e) => {
    #[cfg(debug_assertions)]
    eprintln!("function-acronym: process-name: {}", _e);

    // safe log
    buffy_println!("function-acronym: process-name: failed", &[])?;
}

// Note: This is located only in cargo test functions.
// This is not included in production builds.
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

Depending on the test, you may need a test-assert to be in a cargo-test function and not in the main function.

Warning: Do not collide or mix up test-asserts and debug asserts, or forget that debug code also runs in test builds by default.;
use #[cfg(all(debug_assertions, not(test)))] for debug build only (not test build).
use #[cfg(test)] assert!(  for test build only, not debug).
Give descriptive non-colliding names to cargo-tests and test sets.

Note: production-use characters and strings can be formatted, written, printed using modules such as Buffy
https://github.com/lineality/buffy_stack_format_write_module
instead of using standard Rust macros such as format! print! write! that use heap-memory.

Note: Error messages must be unique per function (e.g. name of function (or abbreviation) in the error message). Colliding generic error messages that cannot be traced to a specific function are a significant liability.


Avoid heap for error messages and for all things:
Is heap used for error messages because that is THE best way, the most secure, the most efficient, proper separation of debug testing vs. secure production code?
Or is heap used because of oversights and apathy: "it's future dev's problem, let's party."

We can use heap in debug/test builds only.

Production software must not insecurely output debug diagnostics.
Debug information must not be included in production builds: "developers accidentally left development code in the software" is a classic error (not a desired design spec) that routinely leads to security and other issues. That is NOT supposed to happen. It is not coherent to insist that open ended heap output 'must' or 'should' be in a production build.

This is central to the question about testing vs. a pedantic ban on conditional compilation; not putting full traceback insecurity into production code is not a different operational process logic tree for process operations.

Just like with the pedantic "all loops being bounded" rule, there is a fundamental exception with conditional compilations: code that must NEVER be in production-builds must ALWAYS be excluded using conditional-compilation flags. This is not an OS or algo-tree conditional compilation, or a hardware conditional compilation; This is an 'unsafe-testing-only' vs. 'safe-production-code' condition. This includes several types of items, such as panic-inducing 'assert' statements (as opposed to proverbial-assert checks that do not panic-halt), and error-message data: Error messages and error outcomes in 'production' 'release' (real-use, not debug/testing) must not ever contain any information that could be a security vulnerability or attack surface. Failing to remove debugging inspection is a major category of security and hygiene problems.

Security: Error messages in production must NOT contain:
- File paths (can reveal system structure)
- File contents
- environment variables
- user, file, state, data
- pii data
- internal implementation details
- etc.

All debug-prints not for production must be tagged with:
```
#[cfg(debug_assertions)]
```

Production output following an error / exception / case must be managed and defined, not not open to whatever an api or OS-call wants to dump out.
The three tiers can be handled with a Fieldless enum type system and exit-codes for behavior such as retry-ing in some cases.

(see more below)






6. Manage ownership and borrowing
- Rust is designed to greatly assist here (vs. c).

7. Manage return values:
- use null-void return values
- check non-void-null returns
- see above for designing and checking return values to handle cases of invalid other return-value cases.
- always have functions return a 'result' so errors and cases can be handled

8. Manage conditional compilation: Navigate debugging and testing on the one hand and not-dangerous conditional-compilation on the other hand:
- Here 'conditional compilation' is interpreted as significant changes to the overall 'tree' of operation depending on build settings/conditions, such as using different modules and basal functions. E.g. "GDPR compliance mode compilation"
- Any LLVM type compilation or build-flag will modify compilation details, but not the target tree logic of what the software does (arguably).
- 2025+ "compilation" and "conditions" cannot be simplistically compared with single-architecture 1970 pdp-11-only C or similar embedded device compilation.

9. Communicate:
- Use doc strings; use comments.
- Document use-cases, edge-cases, and policies (These are project specific and cannot be telepathed from generic micro-function code. When a Mars satellite failed because one team used SI-metric units and another team did not, that problem could not have been detected by looking at, and auditing, any individual function in isolation without documentation. Breaking a process into innumerable undocumented micro-functions can make scope and policy impossible to track. To paraphrase Jack Welch: "The most dangerous thing in the world is a flawless operation that should never have been done in the first place.")
- Rather than using '?' for terse function calling, when possible have detailed error handling.
- Rather than having a result hidden in let _ =, allow that result to be shown in debugging

10. Use state-less operations when possible:
- a seemingly invisibly small increase in state often completely destroys projects
- expanding state destroys projects with unmaintainable over-reach


Also: As per Mara Bos's 'Rust Atomics and Locks' (O'Reilly) note the specific use-case and needs for threads, parallelism, concurrency, atomics, async, etc. Distributed processing varies significantly per project, and implementations of production functions, algorithms, and data structures, are rarely the same as abstract text-book examples.
🦀Vigilance🦀: Properly written code supports users, developers, and the people who depend upon maintainable software. Maintainable software supports the future for us all.

#### Links:
- https://en.wikipedia.org/wiki/The_Power_of_10:_Rules_for_Developing_Safety-Critical_Code
- https://spinroot.com/gerard/pdf/P10.pdf
- https://spinroot.com/static/index.html
- https://web.eecs.umich.edu/~imarkov/10rules.pdf
- https://www.youtube.com/watch?v=JWKadu0ks20
- https://en.wikipedia.org/wiki/Static_program_analysis
- https://www.oreilly.com/library/view/designing-data-intensive-applications/9781491903063/



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

// ========================================================================
// Get file position from cursor (defensive)
// ========================================================================

let current_file_pos = match lines_editor_state
    .get_row_col_file_position(lines_editor_state.cursor.tui_row, lines_editor_state.cursor.tui_visual_col)
{
    Ok(Some(pos)) => pos,
    Ok(None) => {
        // handling None case
        let _ = lines_editor_state.set_info_bar_message("gh cursor position unavailable");
        return Ok(());
    }
    Err(e) => {
        let _ = lines_editor_state.set_info_bar_message("cannot get cursor position");
        log_error(
            &format!("goto_line_start window_map error: {}", e),
            Some("goto_line_start"),
        );
        return Ok(());
    }
};
let line_number_for_display = current_file_pos.line_number + 1; // Convert to 1-indexed

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
use std::io::BufRead;
use std::io::{self, ErrorKind, Read, Seek, SeekFrom, StdinLock, Write, stdin, stdout};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use super::toggle_comment_indent_module::{
    ToggleCommentError, ToggleIndentError, indent_line_bytewise, indent_range_bytewise,
    toggle_basic_singleline_comment_bytewise, toggle_block_comment_bytewise,
    toggle_range_basic_comments_bytewise, toggle_range_rust_docstring_bytewise,
    toggle_rust_docstring_singleline_comment_bytewise, unindent_line_bytewise,
    unindent_range_bytewise,
};

use super::buttons_reversible_edit_changelog_module::{
    ButtonError, EditType, add_single_byte_to_file, button_hexeditinplace_byte_make_log_file,
    button_make_changelog_from_user_character_action_level, button_safe_clear_all_redo_logs,
    button_undo_redo_next_inverse_changelog_pop_lifo, detect_utf8_byte_count,
    get_redo_changelog_directory_path, get_undo_changelog_directory_path,
    read_character_bytes_from_file, read_single_byte_from_file, remove_single_byte_from_file,
};

use super::buffy_format_write_module::{
    BuffyFormatArg, BuffyStyles, SyntaxHighlight, buffy_get_syntax_highlight,
    buffy_is_plain_text_extension, buffy_print, buffy_println,
};

// ============================================================================
// RAW TERMINAL IMPORT (for KeystrokeInputMode only)
// ============================================================================
//
// ## Project Context
//
// This is the FIRST and ONLY use of raw-terminal mode in the entire lines
// editor. Every other editor mode (Normal, Insert, VisualSelect, Pasty, Hex,
// reads from a cooked/canonical StdinLock acquired once in
// lines_fullfile_editor_core.
//
// IMPORTANT NAMING NOTE FOR FUTURE DEVS:
//   - RawTerminal (imported here) IS Linux termios raw mode (no line buffering,
//     no echo, byte-by-byte input). These two unrelated concepts share the
//     word "raw" by historical accident. Do not conflate them.
//
// RawTerminal is owned transiently inside handle_keystroke_input_session().
// It is NEVER stored in EditorState (minimal-state rule) and NEVER created at
// the main-loop level (that would break all the cooked-input modes). Its Drop
// implementation restores the terminal on every exit path, including panic.
// ============================================================================
use crate::raw_terminal_x86_module::RawTerminal;

/// Style for line numbers - green, no bold
const LINE_NUMBER_STYLE: BuffyStyles = BuffyStyles {
    fg_color: Some("\x1b[32m"), // GREEN
    bg_color: None,
    bold: false,
    underline: false,
    italic: true,
    dim: true,
};

/// Style for cursor block
const CURSOR_BLOCK_STYLE: BuffyStyles = BuffyStyles {
    fg_color: Some("\x1b[31m"), // RED
    bg_color: Some("\x1b[47m"), // WHITE background
    bold: true,
    underline: false,
    italic: false,
    dim: false,
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
const FILE_TUI_WINDOW_MAP_BUFFER_SIZE: usize = 64; // 2**13=8192

// for commands such as "n"
const WHOLE_COMMAND_BUFFER_SIZE: usize = 16; //

const MAX_DISPLAY_BUFFER_BYTES: usize = 182;

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
pub const MAX_TUI_VIZ_COLS: usize = 160;
pub const MIN_TUI_VIZ_COLS: usize = 1;
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

// ===========================================================
// ANSI ESCAPE CODES — compile-time constants, zero allocation
// ===========================================================
const BOLD_U8: &[u8] = b"\x1b[1m";
const RED_U8: &[u8] = b"\x1b[31m";
const GREEN_U8: &[u8] = b"\x1b[32m";
const YELLOW_U8: &[u8] = b"\x1b[33m";
// const BLUE_U8: &[u8] = b"\x1b[34m";
const MAGENTA_U8: &[u8] = b"\x1b[35m";
// const CYAN: &[u8] = b"\x1b[36m";
const BG_WHITE_U8: &[u8] = b"\x1b[47m";
const BG_CYAN_U8: &[u8] = b"\x1b[46m";
const RESET_U8: &[u8] = b"\x1b[0m";

// =======================================
// Code & Syntax Formatting / Highlighting
// =======================================
const DEFAULT_TEXT_COLOUR: &[u8] = GREEN_U8;
const DEFINITION_COLOUR: &[u8] = YELLOW_U8;
const SYMBOL_COLOUR: &[u8] = MAGENTA_U8;

/// Blue foreground for tab character highlighting.
/// Tabs mixed with spaces are a common source of indentation bugs.
/// We render tabs visibly (as →) in blue so they are unambiguous.
///
/// ANSI: ESC [ 3 4 m  — blue foreground
pub const TAB_COLOUR: &[u8] = b"\x1b[34m";

/// The visible glyph written in place of a raw tab byte.
/// Using a visible arrow makes tab positions unambiguous.
/// The byte sequence is the UTF-8 encoding of U+2192 RIGHTWARDS ARROW.
pub const TAB_GLYPH: &[u8] = "→".as_bytes();

/*
Foreground Colors (Text Color):
Color -> ANSI Code
Black -> \x1b[30m
Red -> \x1b[31m
Green -> \x1b[32m
Yellow -> \x1b[33m
Blue -> \x1b[34m
Magenta -> \x1b[35m
Cyan -> \x1b[36m
White -> \x1b[37m
Default -> \x1b[39m


Background Colors:
Color -> ANSI Code
Black -> \x1b[40m
Red -> \x1b[41m
Green -> \x1b[42m
Yellow -> \x1b[43m
Blue -> \x1b[44m
Magenta -> \x1b[45m
Cyan -> \x1b[46m
White -> \x1b[47m
Default -> \x1b[49m

text styles:
Style -> ANSI Code
Bold -> \x1b[1m
Dim -> \x1b[2m
Italic -> \x1b[3m
Underline -> \x1b[4m
Blink -> \x1b[5m
Reverse -> \x1b[7m
Hidden -> \x1b[8m
Reset -> \x1b[0m

*/

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
//    Debug-Assert, Test-Asset, Production-Catch-Handle
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

    /// UTF-8 encoding/decoding error
    Utf8Error(String),

    /// Terminal or display rendering error
    DisplayError(String),

    /// Configuration or state error
    StateError(String),

    /// For use with suite of
    /// Debug-Assert, Test-Asset, Production-Catch-Handle
    GeneralAssertionCatchViolation(String),

    /// Lines processed exceeded available display rows.
    /// Indicates potential state corruption or file processing logic error.
    /// This should never occur in normal operation; if it does, the file
    /// or internal state may be malformed.
    LineCountExceeded {
        lines_processed: usize,
        available_rows: usize,
    },
}

impl std::fmt::Display for LinesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinesError::Io(e) => write!(f, "IO error: {}", e),
            LinesError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            // LinesError::FormatError(msg) => write!(f, "Format error: {}", msg),
            LinesError::Utf8Error(msg) => write!(f, "UTF-8 error: {}", msg),
            LinesError::DisplayError(msg) => write!(f, "Display error: {}", msg),
            LinesError::StateError(msg) => write!(f, "State error: {}", msg),
            LinesError::GeneralAssertionCatchViolation(msg) => {
                write!(f, "GeneralAssertionCatchViolation error: {}", msg)
            }
            LinesError::LineCountExceeded {
                lines_processed,
                available_rows,
            } => write!(
                f,
                "LineCountExceeded error: {} {}",
                lines_processed, available_rows
            ),
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
/// (TODO: this is an inflamatory example...
/// loading an entire file into memory
/// and desecrating the entire point of
/// of this project????????????????????)
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
    //    Debug-Assert, Test-Asset, Production-Catch-Handle
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
///  // Normal byte
/// if let Some(hex) = stack_format_hex_zero(0x42, &mut buf, false, "", "", "", "") {
///     print!("{}", hex); // "42 "
/// }
///
///  // Highlighted byte
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
///  // Table-like alignment
/// let id = "42";
/// let name = "Alice";
/// let row = stack_format_it(
///     "ID: {:<5} Name: {:<10}",
///     &[id, name],
///     "Data unavailable"
/// );
///  // Result: "ID: 42    Name: Alice     "
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
    let mut buf = [0u8; 256];

    // Maximum number of inserts to prevent abuse
    // does this need to be usize?
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

// TODO vec< is heap
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
const SAVE_AS_COPY_BUFFER_SIZE: usize = 64;

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

// ============================================================================
// (end) SAVE-AS-COPY OPERATION: Configuration Constants
// ============================================================================

// TODO: Why does this 'mod' exist? Why not use normal constants??
/// Defensive programming limits to prevent infinite loops and resource exhaustion
/// Following NASA Power of 10 rules: all loops must have explicit upper bounds
pub mod limits {
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

    // TODO: determining ideal default buffer & chunk size
    /// Maximum bytes to read when processing a single line
    /// Matches the line buffer size
    pub const LINE_CHUNK_READ_BYTES: usize = 32; // original: 4096

    /// Maximum iterations when skipping characters for horizontal offset
    /// Allows scrolling very far right in losng lines
    pub const HORIZONTAL_SCROLL_CHARS: usize = usize::MAX;

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

    pub const TEXT_INPUT_CHUNKS: usize = usize::MAX;

    pub const MAX_CHUNKS: usize = usize::MAX; // e.g. 16_777_216 allows ~4GB at 256-byte chunks
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
/// - days per month
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
/// and month lengths.
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

    if !include_microseconds {
        return base_timestamp;
    }

    // Add microseconds component
    let microseconds = duration_since_epoch.as_micros() % 1_000_000;

    format!("{}_{:06}", base_timestamp, microseconds)
}

/*
The attempt is to follow NASA's only-preallocated-memory rule.
*/

const FIXED_SIZE_32_TIMESTAMP_CAPACITY: usize = 32;
const FIXED_SIZE_32_TIMESTAMP_MAX_LEN: usize = FIXED_SIZE_32_TIMESTAMP_CAPACITY - 1; // 31, reserves one byte (e.g. for null termination)

/// Fixed-size timestamp type - stack allocated, no heap
#[derive(Copy, Clone)]
pub struct FixedSize32Timestamp {
    data: [u8; FIXED_SIZE_32_TIMESTAMP_CAPACITY],
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
        // Assertion 1: Check length
        if s.len() > FIXED_SIZE_32_TIMESTAMP_MAX_LEN {
            return Err(LinesError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                stack_format_it(
                    "impl FixedSize32Timestamp String too long: {} bytes, max: {}",
                    &[
                        &s.len().to_string(),
                        &FIXED_SIZE_32_TIMESTAMP_MAX_LEN.to_string(),
                    ],
                    "impl FixedSize32Timestamp String too long: __ bytes, max: __",
                ),
            )));
        }

        let mut data = [0u8; FIXED_SIZE_32_TIMESTAMP_CAPACITY];
        let bytes = s.as_bytes();

        // Bounded copy loop
        for i in 0..s.len().min(FIXED_SIZE_32_TIMESTAMP_MAX_LEN) {
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
        //    Debug-Assert, Test-Asset, Production-Catch-Handle
        //    =================================================
        // This is not included in production builds
        // assert: only when running in a debug-build: will panic
        debug_assert!(
            self.len <= FIXED_SIZE_32_TIMESTAMP_CAPACITY,
            "Internal invariant violated: length exceeds buffer size"
        );

        // Catch & Handle without panic in production
        // This IS included in production to safe-catch
        if self.len > FIXED_SIZE_32_TIMESTAMP_CAPACITY {
            // state.set_info_bar_message("Config error");
            return Err(LinesError::GeneralAssertionCatchViolation(
                "is len > 32buf".into(),
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
/// - File ending with newline
/// - File ending without newline  (last line still exists)
///
/// # Example
/// ```ignore
/// let (total_lines, _) = count_lines_in_file(Path::new("/path/to/file.txt"))?;
///  // Now jump to last line
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

// TODO, maybe add to buffy
/// Writes a single hotkey command with color highlighting directly to terminal
///
/// ## Memory: ZERO HEAP
/// Writes hotkey (RED) + description (YELLOW) using buffy_print
///
/// ## Parameters
/// - hotkey: The command character(s) to highlight in RED
/// - description: The rest of the text in YELLOW
///
/// ## Example
/// ```rust
/// write_red_hotkey("q", "uit ")?;  // Outputs: RED"q" + YELLOW"uit "
/// ```
fn write_red_hotkey(hotkey: &str, description: &str) -> io::Result<()> {
    buffy_print(
        "{}{}{}{}",
        &[
            BuffyFormatArg::Str(RED),
            BuffyFormatArg::Str(hotkey),
            BuffyFormatArg::Str(YELLOW),
            BuffyFormatArg::Str(description),
        ],
    )
}

// TODO, maybe add to buffy
/// Writes a two-part hotkey command with color highlighting directly to terminal
///
/// ## Memory: ZERO HEAP
/// Writes hotkey_1 (RED) + hotkey_2 (GREEN) + description (YELLOW) using buffy_print
///
/// ## Parameters
/// - hotkey_1: First part of command to highlight in RED
/// - hotkey_2: Second part of command to highlight in GREEN
/// - description: The rest of the text in YELLOW
///
/// ## Example
/// ```rust
/// write_red_green_hotkey("s", "a", "v ")?;  // Outputs: RED"s" + GREEN"a" + YELLOW"v "
/// write_red_green_hotkey("/", "/", "/cmnt ")?;  // Outputs: RED"/" + GREEN"/" + YELLOW"/cmnt "
/// ```
fn write_red_green_hotkey(hotkey_1: &str, hotkey_2: &str, description: &str) -> io::Result<()> {
    buffy_print(
        "{}{}{}{}{}{}",
        &[
            BuffyFormatArg::Str(RED),
            BuffyFormatArg::Str(hotkey_1),
            BuffyFormatArg::Str(GREEN),
            BuffyFormatArg::Str(hotkey_2),
            BuffyFormatArg::Str(YELLOW),
            BuffyFormatArg::Str(description),
        ],
    )
}

/// Writes the complete navigation legend directly to terminal
///
/// ## Project Context
/// Displays all available keyboard commands for file navigation with
/// color-coded hotkeys. Each command section written independently for
/// maintainability - adding/removing commands requires no argument counting.
///
/// ## Memory: ZERO HEAP
/// All output written directly to terminal using buffy functions.
/// No intermediate String building, no heap allocation.
///
/// ## Operation
/// Writes legend in modular sections:
/// - Each command written separately via write_red_hotkey()
/// - Colors applied per-command (RED hotkey, YELLOW description)
/// - RESET applied at end
/// - Modular: Add/remove commands without affecting others
///
/// ## Safety & Error Handling
/// - Returns io::Result for write failures
/// - Each command write is independent
/// - Failure in one command doesn't affect others structurally
///
/// ## Legend Commands
/// - q: quit application
/// - sav: save current state (red and green and yellow)
/// - re: reload/refresh
/// - undo: undo last operation
/// - del: delete item
/// - nrm: normal mode
/// - ins: insert mode
/// - vis: visual mode
/// - hex: hex editor mode
/// - pasty: paste operation
/// - cvy: copy operation
/// - wrd,b,end: word navigation
/// - ///cmnt: comment operations (red and green and yellow)
/// - []idnt: indent operations
/// - hjkl: vim-style navigation
///
/// ## Example
/// ```rust
///  // In main display loop:
/// write_formatted_navigation_legend_to_tui()?;
/// ```
fn write_formatted_navigation_legend_to_tui() -> Result<()> {
    // File operations group
    write_red_hotkey("q", "uit ")?;
    // Three Colour
    write_red_green_hotkey("s", "a", "v ")?;
    // Red only
    write_red_hotkey("re", ",")?;
    write_red_hotkey("u", "ndo ")?;

    // Mode operations group
    write_red_hotkey("d", "el|")?;
    write_red_hotkey("n", "rm ")?;
    // write_red_hotkey("i", "ns ")?;
    write_red_green_hotkey("k", "i", "ns ")?;
    write_red_hotkey("v", "is ")?;
    write_red_hotkey("hex", "|")?;

    // View operations group
    // write_red_hotkey("r", "aw|")?;
    write_red_hotkey("g", "o ")?;
    write_red_hotkey("p", "asty ")?;
    write_red_hotkey("cvy", "|")?;

    // Navigation group
    write_red_hotkey("w", "rd,")?;
    write_red_hotkey("b", ",")?;
    write_red_hotkey("e", "nd ")?;

    // Comment/indent group
    // Three Colour
    write_red_green_hotkey("/", "/", "/cmnt ")?;
    // Red only
    write_red_hotkey("[]", "idnt ")?;

    // Movement group
    write_red_hotkey("hjkl", "")?;

    // Clear formatting: ANSI color codes are stateful
    // Make sure NEXT prints
    // are not also formatted.
    buffy_print("{}", &[BuffyFormatArg::Str(RESET)])?;

    // Complete the line with newline \n
    buffy_println("", &[])?;

    // Done
    Ok(())
}

/// Creates a unique temporary file in the specified base directory with configurable retry logic.
///
/// # Project Context
/// This function generates temporary file names for intermediate processing
/// in our application where we cannot use third-party dependencies. The file
/// is created atomically to prevent race conditions where multiple processes
/// or threads might generate the same name. This version allows the caller
/// to configure retry behavior based on their specific use case (e.g., high
/// contention environments may need more retries, low-priority operations
/// may want fewer retries to fail fast).
///
/// # Implementation Strategy
/// - Uses process ID, thread ID, and nanosecond timestamp for uniqueness
/// - Attempts atomic file creation with `create_new(true)` flag
/// - Retries up to `number_of_attempts` times with configurable delay
/// - Different timestamps on each retry provide additional uniqueness
/// - Parameterized retry logic allows tuning for different deployment scenarios
///
/// # Arguments
/// * `base_path` - The directory where the temporary file will be created (must exist)
/// * `prefix` - A prefix for the filename to identify the file's purpose (e.g., "cache", "upload")
/// * `number_of_attempts` - Maximum number of creation attempts (recommended: 3-10)
/// * `retry_delay_ms` - Milliseconds to wait between retry attempts (recommended: 1-100)
///
/// # Returns
/// * `Ok(PathBuf)` - Absolute path to the newly created unique temporary file
/// * `Err(io::Error)` - If file creation fails after all retry attempts or other I/O error
///
/// # Error Conditions
/// - `CUTF: system time unavailable` - System clock error (rare, catastrophic)
/// - `CUTF: max retry attempts exceeded` - Could not create unique file after all attempts
/// - `CUTF: unexpected loop exit` - Internal logic error (should never occur)
/// - Standard I/O errors: permission denied, disk full, path not found, etc.
///
/// # Safety & Reliability
/// - No panic: all error cases return Result
/// - No heap allocation in error messages (uses static strings with CUTF prefix)
/// - Bounded retry loop (caller-specified maximum attempts)
/// - Atomic file creation prevents race conditions
/// - Thread-safe: uses thread-local IDs
/// - Handles system clock errors gracefully
///
/// # Configuration Guidelines
/// - **Low contention** (single-threaded, low frequency): `number_of_attempts = 3`, `retry_delay_ms = 1`
/// - **Medium contention** (multi-threaded application): `number_of_attempts = 5`, `retry_delay_ms = 1-5`
/// - **High contention** (distributed system, many processes): `number_of_attempts = 10`, `retry_delay_ms = 10-50`
/// - **Fast-fail scenarios** (can afford to fail): `number_of_attempts = 1`, `retry_delay_ms = 0`
///
/// # Edge Cases Handled
/// - Zero attempts: Function will try once (loop runs 0..0 means 0 iterations, caught by unreachable error)
/// - System time moves backwards: Handled gracefully with retry
/// - Concurrent file creation: Atomic `create_new` prevents race conditions
/// - Disk full or permission errors: Immediate return without retries
///
/// # Example
/// ```
/// use std::path::Path;
///
/// let base = Path::new("/tmp");
///
///  // Standard usage
/// match create_unique_temp_name_and_file_filepathbuf(base, "myapp", 5, 1) {
///     Ok(path) => {
///         println!("Created: {:?}", path);
///         // Use the file...
///         // Remember to delete it when done!
///         let _ = std::fs::remove_file(path);
///     },
///     Err(e) => {
///         eprintln!("Failed to create temp file: {}", e);
///         // Handle error - application continues
///     }
/// }
///
///  // High-contention scenario
/// match create_unique_temp_name_and_file_filepathbuf(base, "distributed", 10, 10) {
///     Ok(path) => { /* use file */ },
///     Err(e) => { /* handle gracefully */ }
/// }
/// ```
///
/// # Security Considerations
/// - File names are predictable (not cryptographically random)
/// - Suitable for temporary storage, not for security-critical scenarios
/// - Consider file permissions on created files (inherits from OpenOptions defaults)
/// - Caller must ensure base_path is in a secure location
///
/// # Performance Considerations
/// - Each retry costs `retry_delay_ms` milliseconds
/// - Maximum possible delay: `number_of_attempts * retry_delay_ms`
/// - Nanosecond timestamp provides ~1 billion unique values per second per thread
/// - Thread ID formatting allocates small string (unavoidable with std::thread API)
pub fn create_unique_temp_name_and_file_filepathbuf(
    base_path: &Path,
    prefix: &str,
    number_of_attempts: u32,
    retry_delay_ms: u64,
) -> io::Result<PathBuf> {
    use std::fs::OpenOptions;
    use std::io;
    use std::process;
    use std::thread;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    // Production catch: validate number_of_attempts is non-zero
    // Zero attempts would make function always fail
    if number_of_attempts == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "CUTF: number_of_attempts must be greater than zero",
        ));
    }

    // Get process ID once (constant for this process)
    let pid = process::id();

    // Get thread ID and format it for filename use
    let thread_id = thread::current().id();
    let thread_id_string = format!("{:?}", thread_id);

    // Clean thread ID: remove "ThreadId(" prefix and ")" suffix
    // This converts "ThreadId(123)" to "123"
    let thread_id_clean = thread_id_string
        .trim_start_matches("ThreadId(")
        .trim_end_matches(')');

    // Attempt to create unique file with retry logic
    for attempt in 0..number_of_attempts {
        // Get current timestamp with nanosecond precision
        // This provides uniqueness across time
        let timestamp_result = SystemTime::now().duration_since(UNIX_EPOCH);

        let timestamp_nanos = match timestamp_result {
            Ok(duration) => duration.as_nanos(),
            Err(_) => {
                // System time error (e.g., clock moved backwards)
                // In production, we handle this gracefully and continue
                if attempt == number_of_attempts - 1 {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "CUTF: system time unavailable",
                    ));
                }
                // Try again with next attempt
                thread::sleep(Duration::from_millis(retry_delay_ms));
                continue;
            }
        };

        // Construct filename: prefix_pid_threadid_timestamp.tmp
        // Example: myapp_12345_67_1234567890123456789.tmp
        let filename = format!(
            "{}_{}_{}_{}.tmp",
            prefix, pid, thread_id_clean, timestamp_nanos
        );

        // Build absolute path
        let file_path = base_path.join(&filename);

        // Attempt to create file atomically
        // create_new(true) ensures the operation fails if file exists
        // This prevents race conditions with other processes/threads
        match OpenOptions::new()
            .write(true)
            .create_new(true) // Critical: fails if file already exists
            .open(&file_path)
        {
            Ok(_file) => {
                // Success: file created exclusively
                // File handle is dropped here, closing the file
                // Caller is responsible for file cleanup
                return Ok(file_path);
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                // File name collision detected
                // This is expected in high-concurrency scenarios

                // Production catch: check if we've exhausted retries
                if attempt == number_of_attempts - 1 {
                    // Final attempt failed - return descriptive error
                    return Err(io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        "CUTF: max retry attempts exceeded",
                    ));
                }

                // Wait briefly before retry
                // This allows timestamp to change and reduces contention
                thread::sleep(Duration::from_millis(retry_delay_ms));

                // Continue to next attempt
                continue;
            }
            Err(e) => {
                // Other error occurred (permissions, disk full, etc.)
                // Return immediately - retrying won't help
                return Err(e);
            }
        }
    }

    // Should be unreachable due to loop logic, but rust requires this
    // Production safety: return error rather than panic
    Err(io::Error::new(
        io::ErrorKind::Other,
        "CUTF: unexpected loop exit",
    ))
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
///  // Ensure the project graph data directory exists relative to the executable
/// let project_graph_directory_result = make_verify_or_create_executabledirectoryrelative_canonicalized_dir_path("project_graph_data");

///  // Handle any errors that might occur during directory creation or verification
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
///  // Get an absolute path for "data/config.json" relative to the executable directory
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
    pub tui_row: usize,
    /// Column in terminal (0-indexed, 0-319 max)
    pub tui_visual_col: usize,
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
    /// Keystroke-input mode: byte-by-byte ASCII input via
    /// Linux termios "raw terminal".
    ///
    /// # Project Context
    /// This mode is the only place in the editor that uses a real raw terminal
    /// (`RawTerminal`). It functions like Insert mode (immediate cursor updates,
    /// per-character undo logging, display refresh after each edit) but reads
    /// individual keystroke bytes instead of cooked/Enter-terminated lines.
    ///
    /// # Entry / Exit
    /// - Entered via the `ki` command from Normal mode.
    /// - Exited by pressing ESC (0x1B), which routes through
    ///   `Command::EnterNormalMode` and flips this back to `Normal`. The session
    ///   loop watches `self.mode == KeystrokeInputMode` and exits when it flips.
    /// - Also exited (defensively, set back to Normal) on terminal EOF or read
    ///   error inside the session loop.
    ///
    /// # Accepted Input
    /// - Printable ASCII (0x20..=0x7E): inserted as a single byte.
    /// - Backspace (0x08) and DEL (0x7F): backspace-style delete.
    /// - LF (0x0A) and CR (0x0D): insert a single '\n'.
    /// - ESC (0x1B): exit to Normal mode.
    /// - Everything else (arrow keys, Tab 0x09, Ctrl/Alt/Fn combos, multibyte
    ///   escape-sequence fragments): silently ignored.
    KeystrokeInputMode,
}

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
    PastyPasteInputMode,
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
    let _ = format_pasty_tui_legend();

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

    // writes to TUI
    display_pasty_info_bar(
        total_count,
        first_count_visible,
        last_count_visible,
        message_for_infobar, // Use info_bar_message from state
    )?;

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
    ///  // Jump to end of line for cursor
    /// if let Some((_, line_end)) = window_map.display_line_byte_ranges[row] {
    ///     cursor_position = line_end;
    /// }
    ///
    ///  // Check if cursor at line start
    /// if let Some((line_start, _)) = window_map.display_line_byte_ranges[row] {
    ///     if cursor_byte == line_start {
    ///         // At start of line
    ///     }
    /// }
    ///
    ///  // Detect line boundary for move-left wrapping
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
    // pub next_move_right_is_past_newline: bool,

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

    // === DISPLAY BUFFERS ===
    /// Pre-allocated buffers for each display row (45 rows × 80 chars)
    /// Each buffer holds one terminal row including line number and text
    pub utf8_txt_display_buffers: [[u8; MAX_DISPLAY_BUFFER_BYTES]; MAX_TUI_ROWS],

    /// Bytes used in each display buffer
    /// Since lines can be shorter than 80 chars, we track usage
    pub display_utf8txt_buffer_lengths: [usize; MAX_TUI_ROWS],

    /// Hex mode cursor (byte position in file)
    /// Only used when mode == EditorMode::HexMode
    pub hex_cursor: HexCursor,

    /// EOF information for the currently displayed window
    /// None = EOF not visible in current window
    /// Some((file_line_of_eof, eof_tui_display_row)) = EOF position
    pub eof_fileline_tuirow_tuple: Option<(usize, usize)>,

    /// short message to display in TUI, bottom bar
    pub info_bar_message_buffer: [u8; INFOBAR_MESSAGE_BUFFER_SIZE],

    /// shared scratch pad buffer for reading line-chunks
    pub line_chunk_scratch: [u8; limits::LINE_CHUNK_READ_BYTES],
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

            windowmap_line_byte_start_end_position_pairs: [None; MAX_TUI_ROWS],
            security_mode: false, // default setting, purpose: to force-reset manually clear overwrite buffers

            cursor: WindowPosition {
                tui_row: 0,
                tui_visual_col: 0,
            },

            // window_start: FilePosition {
            //     // for Wrap mode, if that happens
            //     byte_offset_linear_file_absolute_position: 0,
            //     line_number: 0,
            //     byte_in_line: 0,
            // },
            //
            // next_move_right_is_past_newline: false,
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

            // Display buffers - initialized to zero
            utf8_txt_display_buffers: [[0u8; MAX_DISPLAY_BUFFER_BYTES]; MAX_TUI_ROWS],
            display_utf8txt_buffer_lengths: [0usize; MAX_TUI_ROWS],
            hex_cursor: HexCursor::new(),
            eof_fileline_tuirow_tuple: None, // Time is like a banana, it had no end...
            info_bar_message_buffer: [0u8; INFOBAR_MESSAGE_BUFFER_SIZE],
            line_chunk_scratch: [0u8; limits::LINE_CHUNK_READ_BYTES],
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
    ///  // "hello\n" at bytes [10..15]
    /// set_line_byte_range(0, 10, 15)?;
    ///
    ///  // Empty line "\n" at byte [20]
    /// set_line_byte_range(1, 20, 20)?;
    ///
    ///  // Last line "world" at bytes [25..29], no newline
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
    ///  // In pasty_mode() loop:
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

        //  ===================
        //  Bucket Brigade Loop
        //  ===================

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

        //  =======================
        //  Parse Accumulated Input
        //  =======================

        // Convert bytes to UTF-8 string
        // Only process valid bytes [0..accumulated_bytes], rest is unused
        let input_str = std::str::from_utf8(&file_tui_windowmap_buffer[..accumulated_bytes])
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid UTF-8"))?;
        // Trim whitespace and newline delimiter
        let trimmed = input_str.trim();

        //  ==========================
        //  Parse with Priority Order:
        //  1. Empty
        //  2. Explicit commands
        //  3. Numbers (rank selection)
        //  4. Paths (fallback)
        //  ==========================

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

        if trimmed == "paste" {
            return Ok(PastyInputPathOrCommand::PastyPasteInputMode);
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

    /// Computes the file position for a given display (row, col) by reading
    /// the read-copy file on demand — no resident [ROWS][COLS] mapping array.
    ///
    /// # Purpose (Project Context)
    /// Replaces the resident `state_windowmap_positions` grid. The file is the
    /// single source of truth. We seek to the row's known line-start byte and
    /// walk forward to resolve the requested column into a file byte position.
    ///
    /// # CRITICAL: `col` is a VISUAL column (Option A)
    /// Under Option A, `cursor.tui_visual_col` is a VISUAL column measured in terminal
    /// CELLS, not a character count. A double-width character (CJK, emoji) is
    /// ONE character / ONE cursor stop but consumes TWO visual cells. Therefore
    /// this function must walk the line content summing `is_double_width` widths
    /// (1 per ASCII, 2 per double-width) to convert the incoming visual column
    /// into the character / byte position.
    ///
    /// # CRITICAL: the horizontal scroll offset is in CHARACTERS
    /// `tui_window_horizontal_utf8txt_line_char_offset` skips whole CHARACTERS
    /// from the line start (this matches `process_line_with_offset` PHASE 1).
    /// We therefore resolve a column in two stages, mirroring the builder:
    ///   1. Skip `offset` CHARACTERS from line start (char-based).
    ///   2. From there, advance summing VISUAL widths until the accumulated
    ///      visual width reaches the requested content visual column.
    ///
    /// # Mid-double-width-cell (defensive)
    /// Movement never lands the cursor on the SECOND cell of a
    /// double-width char. If such a `col` arrives anyway, we SNAP to the
    /// character whose visual span contains it (return that char's start byte),
    /// via the span test `content_visual_col < accumulated_visual + width`.
    ///
    /// # CRITICAL: two trailing virtual cells (this is what made the grid work)
    /// After the last character, two virtual cursor cells exist:
    ///   1. NEWLINE GLYPH cell (only if the line has a newline) → the '\n' byte.
    ///      It occupies ONE visual cell at the line's total visible visual width.
    ///   2. END-OF-LINE cell (one past) → byte after content (after the '\n' if
    ///      present; EOF byte on the file's last unterminated line).
    /// These let the cursor sit at / after end-of-line for appending. The walk
    /// MUST reproduce them or the cursor cannot reach the end of a line.
    ///
    /// # Coordinate Mapping
    ///   content_visual_col = col - line_num_width        (strip "42 " prefix)
    ///   then: skip `offset` chars, then walk `content_visual_col` VISUAL cells.
    ///
    /// # Coordinate Spaces (see the module "Coordinate Spaces" reference)
    /// - In  `row`           : #6 TUI display row
    /// - In  `col`           : #5 VISUAL cell column (INCLUDES line-number prefix)
    /// - Reads `tui_window_horizontal_utf8txt_line_char_offset` : #4 in-line char index
    /// - Reads `windowmap_line_byte_start_end_position_pairs[row]` : #1 file-byte cache
    /// - Out `byte_offset_linear_file_absolute_position` : #1 file byte
    /// - Out `byte_in_line`  : #2 in-line byte
    /// - Out `line_number`   : #3 line number
    /// This is THE conversion from (#6,#5) → (#1,#2,#3); no other path may invent one.
    ///
    /// # Source of Truth
    /// - Row → line_start_byte: windowmap_line_byte_start_end_position_pairs[row]
    /// - Row → file line number: line_count_at_top_of_window + row
    /// - Bytes: read from the read-copy file at line_start_byte
    ///
    /// # Arguments
    /// * `row` - Display row (0-indexed within window)
    /// * `col` - FULL VISUAL display column (includes line-number prefix)
    ///
    /// # Returns
    /// * `Ok(Some(FilePosition))` - char, newline glyph cell, or EOL cell
    /// * `Ok(None)`               - out of bounds, in prefix, or past EOL cell
    /// * `Err(io::Error)`         - read-copy I/O failure
    ///
    /// # Error Handling
    /// Returns Ok(None) for all "no such cell" cases. Returns Err only for
    /// I/O failure on the read-copy. Never panics in production.
    pub fn get_row_col_file_position(
        &self,
        row: usize,
        col: usize,
    ) -> io::Result<Option<FilePosition>> {
        // ----- window bounds (caught) -----
        if row >= self.effective_rows || col >= self.effective_cols {
            return Ok(None);
        }

        // ----- row's line-start byte (source of truth) -----
        let (line_start_byte, line_end_byte) =
            match self.windowmap_line_byte_start_end_position_pairs[row] {
                Some(pair) => pair,
                None => return Ok(None),
            };

        let file_line_number = self.line_count_at_top_of_window + row;

        // ----- line-number prefix width (same logic as renderer) -----
        let line_num_width =
            calculate_line_number_width(self.line_count_at_top_of_window, row, self.effective_rows);
        if col < line_num_width {
            return Ok(None);
        }

        // VISUAL column into the visible content area (after the prefix).
        let content_visual_col = col - line_num_width;

        // Horizontal scroll offset is in CHARACTERS (matches the builder).
        let char_offset = self.tui_window_horizontal_utf8txt_line_char_offset;

        // extra-check
        // #[cfg(debug_assertions)]
        // eprintln!(
        //     "GRCFP row={} col={} | line_num_width={} content_visual_col={} char_offset={} | line_start={} line_end={}",
        //     row,
        //     col,
        //     line_num_width,
        //     content_visual_col,
        //     char_offset,
        //     line_start_byte,
        //     line_end_byte,
        // );

        // ----- read-copy byte source -----
        let read_copy = match self.read_copy_path.as_ref() {
            Some(p) => p,
            None => return Ok(None),
        };

        let mut file = File::open(read_copy)?;

        // content_exclusive_end = first byte AFTER content (where '\n' would be).
        // line_end_byte is the inclusive last CONTENT byte. For an "empty line"
        // by convention build stores start == end (the byte is the '\n' itself);
        // that case is handled in the walk below (lead == b'\n' → no content).
        let content_exclusive_end = line_end_byte.saturating_add(1);

        // Detect trailing newline at content_exclusive_end.
        let has_newline = {
            file.seek(SeekFrom::Start(content_exclusive_end))?;
            let mut one = [0u8; 1];
            match file.read(&mut one)? {
                0 => false, // EOF: last line, no newline
                _ => one[0] == b'\n',
            }
        };

        // ───────────────────────────────────────────────────────────────────
        // Local reader: resolve the content character at `pos`.
        // Returns (byte_len, visual_width) for a content char, or None
        // when there is no content char at `pos` (newline lead, EOF, boundary
        // crossing, or short read). Width uses the single shared oracle
        // double_width::is_double_width; invalid UTF-8 is treated as width 1
        // (matching process_line_with_offset).
        // ───────────────────────────────────────────────────────────────────
        fn read_one_content_char(
            file: &mut File,
            pos: u64,
            content_exclusive_end: u64,
        ) -> io::Result<Option<(u64, usize)>> {
            if pos >= content_exclusive_end {
                return Ok(None);
            }

            file.seek(SeekFrom::Start(pos))?;
            let mut buf = [0u8; 4];
            let n = file.read(&mut buf)?;
            if n == 0 {
                return Ok(None); // EOF
            }

            let lead = buf[0];
            if lead == b'\n' {
                return Ok(None); // empty-line convention / no more content
            }

            let byte_len: u64 = if lead < 0x80 {
                1
            } else if lead < 0xE0 {
                2
            } else if lead < 0xF0 {
                3
            } else if lead < 0xF8 {
                4
            } else {
                1
            };

            // Do not cross out of content.
            if pos + byte_len > content_exclusive_end {
                return Ok(None);
            }
            // Must have read enough bytes to decode this char.
            if (n as u64) < byte_len {
                return Ok(None);
            }

            let width = match std::str::from_utf8(&buf[..byte_len as usize]) {
                Ok(s) => match s.chars().next() {
                    Some(ch) => {
                        if double_width::is_double_width(ch) {
                            2
                        } else {
                            1
                        }
                    }
                    None => 1,
                },
                Err(_) => 1, // invalid UTF-8 → single width (matches renderer)
            };

            Ok(Some((byte_len, width)))
        }

        // Running position state.
        let mut current_byte = line_start_byte;
        let mut byte_in_line: usize = 0;

        // Upper bound for the bounded walks (each iteration advances ≥ 1 byte).
        let content_byte_len = content_exclusive_end.saturating_sub(line_start_byte) as usize;

        // ───────────────────────────────────────────────────────────────────
        // PHASE A: skip `char_offset` CHARACTERS from the line start so the
        // walk begins at the first VISIBLE character (matches builder PHASE 1).
        // ───────────────────────────────────────────────────────────────────
        let mut chars_skipped: usize = 0;
        let mut skip_guard: usize = 0;
        while chars_skipped < char_offset
            && current_byte < content_exclusive_end
            && skip_guard < limits::HORIZONTAL_SCROLL_CHARS
        {
            skip_guard += 1;

            match read_one_content_char(&mut file, current_byte, content_exclusive_end)? {
                Some((byte_len, _width)) => {
                    current_byte += byte_len;
                    byte_in_line += byte_len as usize;
                    chars_skipped += 1;
                }
                None => break, // no more content to skip (short line / empty line)
            }
        }

        // ───────────────────────────────────────────────────────────────────
        // PHASE B: walk visible content summing VISUAL width. The target is the
        // character whose visual span [acc, acc + width) contains
        // content_visual_col (snap-to-containing for mid-double-width cells).
        // ───────────────────────────────────────────────────────────────────
        let mut accumulated_visual: usize = 0;
        let mut walk_guard: usize = 0;
        loop {
            if current_byte >= content_exclusive_end {
                break;
            }

            walk_guard += 1;
            if walk_guard > content_byte_len + 2 {
                // TEMPORARY: remove in Step 5
                #[cfg(debug_assertions)]
                eprintln!("GRCFP walk guard tripped");
                return Ok(None);
            }

            let (byte_len, width) =
                match read_one_content_char(&mut file, current_byte, content_exclusive_end)? {
                    Some(pair) => pair,
                    None => break, // newline lead / EOF / boundary → content ends
                };

            // Span of this character: [accumulated_visual, accumulated_visual + width)
            if content_visual_col < accumulated_visual + width {
                // extra check
                // #[cfg(debug_assertions)]
                // eprintln!(
                //     "GRCFP HIT real-char: byte={} byte_in_line={} acc_visual={} width={}",
                //     current_byte, byte_in_line, accumulated_visual, width
                // );
                return Ok(Some(FilePosition {
                    byte_offset_linear_file_absolute_position: current_byte,
                    line_number: file_line_number,
                    byte_in_line,
                }));
            }

            accumulated_visual += width;
            current_byte += byte_len;
            byte_in_line += byte_len as usize;
        }

        // At this point:
        //   accumulated_visual = total VISIBLE visual width of content
        //   current_byte       = content_exclusive_end (the '\n' byte, or EOF)
        //   byte_in_line       = visible content byte length
        //   content_visual_col >= accumulated_visual (target not in content)

        // ───────────────────────────────────────────────────────────────────
        // VIRTUAL CELL 1: NEWLINE GLYPH (1 visual cell, only if line ends in \n)
        // Span: [accumulated_visual, accumulated_visual + 1) → the '\n' byte.
        // ───────────────────────────────────────────────────────────────────
        if has_newline && content_visual_col < accumulated_visual + 1 {
            //  // extra inspect
            // #[cfg(debug_assertions)]
            // eprintln!(
            //     "GRCFP HIT newline-glyph: newline_byte={} byte_in_line={} acc_visual={}",
            //     current_byte, byte_in_line, accumulated_visual
            // );
            return Ok(Some(FilePosition {
                // current_byte == content_exclusive_end == the '\n' byte
                byte_offset_linear_file_absolute_position: current_byte,
                line_number: file_line_number,
                byte_in_line,
            }));
        }

        // ───────────────────────────────────────────────────────────────────
        // VIRTUAL CELL 2: END-OF-LINE (1 visual cell, one past last content).
        //   - With a newline: visual cell accumulated_visual + 1, byte after \n.
        //   - Without a newline (file's last line): visual cell accumulated_visual,
        //     byte == EOF byte (current_byte).
        // ───────────────────────────────────────────────────────────────────
        let eol_visual_col = if has_newline {
            accumulated_visual + 1
        } else {
            accumulated_visual
        };
        let eol_byte = if has_newline {
            current_byte + 1 // skip past the '\n'
        } else {
            current_byte // EOF byte
        };
        let eol_byte_in_line = if has_newline {
            byte_in_line + 1 // include the newline byte
        } else {
            byte_in_line
        };

        if content_visual_col < eol_visual_col + 1 {
            // extra inspect
            // #[cfg(debug_assertions)]
            // eprintln!(
            //     "GRCFP HIT eol-cell: eol_visual_col={} eol_byte={} has_newline={}",
            //     eol_visual_col, eol_byte, has_newline
            // );
            return Ok(Some(FilePosition {
                byte_offset_linear_file_absolute_position: eol_byte,
                line_number: file_line_number,
                byte_in_line: eol_byte_in_line,
            }));
        }

        // // extra inspect
        // #[cfg(debug_assertions)]
        // eprintln!(
        //     "GRCFP MISS: content_visual_col={} past all cells (acc_visual={}, eol_visual_col={}, has_newline={})",
        //     content_visual_col, accumulated_visual, eol_visual_col, has_newline
        // );

        // Past every mapped cell → no position.
        Ok(None)
    }

    /// Returns the VISUAL width (terminal cells) of the character currently
    /// under the cursor: 1 for normal/ASCII, 2 for double-width (CJK/emoji).
    ///
    /// # Purpose (Project Context)
    /// MoveRight advances `cursor.tui_visual_col` by the visual width of the character
    /// it crosses (Option A: `tui_visual_col` is a VISUAL column). This reads the
    /// single character at the cursor's resolved file byte and classifies its
    /// width with the shared oracle `double_width::is_double_width`. It mirrors
    /// the read pattern of `is_current_cursor_on_newline` (resolve cursor →
    /// seek → bounded read at the cursor byte). Stateless and read-only.
    ///
    /// # Defensive / Out-of-Bounds (when in doubt, width 1)
    /// - No read-copy path → Ok(1)
    /// - Cursor position unmapped → Ok(1)
    /// - File open/seek/read fails → Ok(1)
    /// - EOF, ASCII/control, or incomplete sequence → Ok(1)
    /// Width 1 is the safe default: it never over-advances past a char.
    ///
    /// # Coordinate Spaces (see the module "Coordinate Spaces" reference)
    /// - Reads cursor (#6 tui_row, #5 tui_visual_col), derives #1 file byte internally
    /// - Out: the cursor character's width in #5 VISUAL cells (1 or 2)
    ///
    /// # Returns
    /// * `Ok(1)` or `Ok(2)`
    /// * `Err(LinesError)` only if the underlying lookup propagates an
    ///   unrecoverable error.
    pub fn cursor_char_visual_width(&self) -> Result<usize> {
        let read_copy_path = match &self.read_copy_path {
            Some(path) => path,
            None => return Ok(1),
        };

        let cursor_file_pos = match self
            .get_row_col_file_position(self.cursor.tui_row, self.cursor.tui_visual_col)?
        {
            Some(pos) => pos,
            None => return Ok(1),
        };

        let mut file = match File::open(read_copy_path) {
            Ok(f) => f,
            Err(_e) => {
                #[cfg(debug_assertions)]
                eprintln!("cursor_char_visual_width: open failed: {} (width 1)", _e);
                return Ok(1);
            }
        };

        if let Err(_e) = file.seek(SeekFrom::Start(
            cursor_file_pos.byte_offset_linear_file_absolute_position,
        )) {
            #[cfg(debug_assertions)]
            eprintln!("cursor_char_visual_width: seek failed: {} (width 1)", _e);
            return Ok(1);
        }

        // Read up to 4 bytes (max UTF-8 character length).
        let mut buf = [0u8; 4];
        let n = match file.read(&mut buf) {
            Ok(n) => n,
            Err(_e) => {
                #[cfg(debug_assertions)]
                eprintln!("cursor_char_visual_width: read failed: {} (width 1)", _e);
                return Ok(1);
            }
        };
        if n == 0 {
            return Ok(1); // EOF
        }

        let lead = buf[0];
        // ASCII / newline / control are single-width.
        if lead < 0x80 {
            return Ok(1);
        }

        let char_byte_len = if lead < 0xE0 {
            2
        } else if lead < 0xF0 {
            3
        } else if lead < 0xF8 {
            4
        } else {
            return Ok(1); // malformed lead byte
        };

        if n < char_byte_len {
            return Ok(1); // incomplete sequence at the read boundary
        }

        let width = match std::str::from_utf8(&buf[..char_byte_len]) {
            Ok(s) => match s.chars().next() {
                Some(ch) => {
                    if double_width::is_double_width(ch) {
                        2
                    } else {
                        1
                    }
                }
                None => 1,
            },
            Err(_) => 1,
        };

        Ok(width)
    }

    /// Debug-only: print the four cursor SOURCES OF TRUTH plus the key DERIVED
    /// values, each tagged with its coordinate space (see the project
    /// "Coordinate Spaces" reference). Compiled out of release builds entirely.
    ///
    /// # Call syntax — this is a METHOD, not a free function
    /// From an `impl EditorState` method:
    ///     #[cfg(debug_assertions)] self.debug_inspect_position("MoveRight");
    /// From a free function holding an `EditorState` (e.g. goto_line_end):
    ///     #[cfg(debug_assertions)] lines_editor_state.debug_inspect_position("GLE");
    /// Method-call syntax resolves by the receiver's TYPE, so it works in both
    /// places without passing state in by hand. Do NOT call it as
    /// `debug_inspect_position(state, label)` — that form needs a module-level
    /// free fn
    #[cfg(debug_assertions)]
    pub fn debug_inspect_position(&self, label: &str) {
        // ─────────────────────────────────────────────────────────────────
        // POSITION INSPECTION (debug builds only) — STANDARD BLOCK
        //
        // Prints the four SOURCES OF TRUTH for cursor location plus the key
        // DERIVED values, each tagged with its coordinate space. See the
        // project "Coordinate Spaces" reference. Sources of truth are the only
        // stored cursor state; every other coordinate is derived on demand.
        // ─────────────────────────────────────────────────────────────────
        #[cfg(debug_assertions)]
        {
            let dbg_row = self.cursor.tui_row;
            let dbg_visual_col = self.cursor.tui_visual_col;
            let dbg_char_offset = self.tui_window_horizontal_utf8txt_line_char_offset;
            let dbg_top_line = self.line_count_at_top_of_window;
            let dbg_line_num_width =
                calculate_line_number_width(dbg_top_line, dbg_row, self.effective_rows);
            let dbg_content_visual_col = dbg_visual_col.saturating_sub(dbg_line_num_width);

            println!("── {} position inspection ──", label);
            // Sources of truth (the only stored cursor state):
            println!(
                "  [truth] tui_row              (#6 TUI display row)        = {}",
                dbg_row
            );
            println!(
                "  [truth] tui_visual_col       (#5 VISUAL cell column)     = {}",
                dbg_visual_col
            );
            println!(
                "  [truth] line_char_offset     (#4 in-line CHAR index)     = {}",
                dbg_char_offset
            );
            println!(
                "  [truth] line_count_top_window(#3 top-of-window line no.) = {}",
                dbg_top_line
            );
            // Derived (computed from the sources of truth):
            println!(
                "  [deriv] line_num_width       (prefix width, cells)       = {}",
                dbg_line_num_width
            );
            println!(
                "  [deriv] content_visual_col   (cells past the prefix)     = {}",
                dbg_content_visual_col
            );
            println!(
                "  [deriv] file_position        (#1 file byte / #2 line byte / #3 line no.) = {:?}",
                self.get_row_col_file_position(dbg_row, dbg_visual_col)
            );
            // File-byte cache the lookup reads from:
            println!(
                "  [cache] windowmap_line_byte_start_end_position_pairs     = {:?}",
                self.windowmap_line_byte_start_end_position_pairs
            );
        }
    }

    /// Returns the VISUAL width (terminal cells) of the character immediately to
    /// the LEFT of the cursor: 1 for normal/ASCII, 2 for double-width.
    ///
    /// # Purpose (Project Context)
    /// MoveLeft decrements `cursor.tui_visual_col` by the visual width of the character
    /// it crosses (Option A: `tui_visual_col` is a VISUAL column). It resolves the
    /// cursor's file byte, then reads up to 4 bytes ending just before that byte
    /// (never crossing the current line's start) and walks backward over UTF-8
    /// continuation bytes (0x80..=0xBF) to find the previous character's lead
    /// byte. Width is classified by the shared oracle
    /// `double_width::is_double_width`. Stateless and read-only.
    ///
    /// # Defensive / Out-of-Bounds (when in doubt, width 1)
    /// - No read-copy path, unmapped cursor, at line start, or any I/O failure
    ///   → Ok(1). Width 1 never over-decrements past a character.
    ///
    /// # Coordinate Spaces (see the module "Coordinate Spaces" reference)
    /// - Reads cursor (#6 tui_row, #5 tui_visual_col), derives #1 file byte internally,
    ///   then reads backward within the same #3 line to the previous character
    /// - Out: that previous character's width in #5 VISUAL cells (1 or 2)
    ///
    /// # Returns
    /// * `Ok(1)` or `Ok(2)`
    /// * `Err(LinesError)` only if the lookup propagates an unrecoverable error.
    pub fn char_to_left_visual_width(&self) -> Result<usize> {
        let read_copy_path = match &self.read_copy_path {
            Some(path) => path,
            None => return Ok(1),
        };

        let pos = match self
            .get_row_col_file_position(self.cursor.tui_row, self.cursor.tui_visual_col)?
        {
            Some(p) => p,
            None => return Ok(1),
        };

        let cursor_byte = pos.byte_offset_linear_file_absolute_position;
        let line_start = cursor_byte.saturating_sub(pos.byte_in_line as u64);

        if cursor_byte <= line_start {
            return Ok(1); // at line start: no character to the left in this line
        }

        // Up to 4 bytes available before the cursor, but not before line_start.
        let want = (cursor_byte - line_start).min(4) as usize;
        let start = cursor_byte - want as u64;

        let mut file = match File::open(read_copy_path) {
            Ok(f) => f,
            Err(_e) => {
                #[cfg(debug_assertions)]
                eprintln!("char_to_left_visual_width: open failed: {} (width 1)", _e);
                return Ok(1);
            }
        };

        if file.seek(SeekFrom::Start(start)).is_err() {
            return Ok(1);
        }

        let mut buf = [0u8; 4];
        let n = match file.read(&mut buf[..want]) {
            Ok(n) => n,
            Err(_e) => {
                #[cfg(debug_assertions)]
                eprintln!("char_to_left_visual_width: read failed: {} (width 1)", _e);
                return Ok(1);
            }
        };
        if n == 0 {
            return Ok(1);
        }

        // buf[..n] are the bytes [start, cursor_byte). Walk backward to the
        // previous character's lead byte (first non-continuation byte).
        let mut i = n;
        let mut steps = 0;
        let mut lead_index: Option<usize> = None;
        while i > 0 && steps < 4 {
            i -= 1;
            steps += 1;
            if buf[i] & 0xC0 != 0x80 {
                lead_index = Some(i);
                break;
            }
        }

        let li = match lead_index {
            Some(x) => x,
            None => return Ok(1),
        };

        let prev_char_bytes = &buf[li..n];
        let width = match std::str::from_utf8(prev_char_bytes) {
            Ok(s) => match s.chars().next() {
                Some(ch) => {
                    if double_width::is_double_width(ch) {
                        2
                    } else {
                        1
                    }
                }
                None => 1,
            },
            Err(_) => 1,
        };

        Ok(width)
    }

    // ============================================================================
    // LINE END DETECTION (UTF-8-Aware Cursor Movement Support)
    // ============================================================================

    /// Determines if the cursor is currently positioned on a newline character
    ///
    /// # Purpose (Project Context)
    /// This function supports text editor cursor movement in the MoveRight command.
    /// When the cursor is already on a newline character (displayed as ␤), the next
    /// MoveRight should jump to the next line start, not scroll further right.
    ///
    /// # Scope - Graceful Out-of-Bounds Handling
    /// This function exists specifically to handle out-of-bounds conditions safely.
    /// Instead of crashing when the cursor is at invalid positions, it returns safe
    /// default values:
    ///
    /// - No read-copy file path → Returns `Ok(false)` (cannot analyze)
    /// - Cursor position unmapped → Returns `Ok(false)` (not on newline)
    /// - File read fails → Returns `Ok(false)` (cannot determine)
    /// - EOF position → Returns `Ok(false)` (no byte to check)
    ///
    /// The philosophy: **When in doubt, assume NOT on a newline.**
    ///
    /// This function is stateless and read-only; it never modifies editor state.
    ///
    /// # Returns
    /// * `Ok(true)` - Cursor is definitively on a newline byte
    /// * `Ok(false)` - Cursor is NOT on newline, OR position is invalid
    /// * `Err(LinesError)` - Only for truly unrecoverable errors
    ///
    /// # Examples
    /// ```ignore
    ///  // Cursor on visible newline character ␤
    /// let result = state.is_current_cursor_on_newline()?;
    /// assert_eq!(result, true);
    ///
    ///  // Cursor on regular text 'a'
    /// let result = state.is_current_cursor_on_newline()?;
    /// assert_eq!(result, false);
    /// ```
    pub fn is_current_cursor_on_newline(&self) -> Result<bool> {
        // ═══════════════════════════════════════════════════════════════════════
        // DEFENSIVE CHECK 1: Verify read-copy file path exists
        // ═══════════════════════════════════════════════════════════════════════
        let read_copy_path = match &self.read_copy_path {
            Some(path) => path,
            None => {
                #[cfg(debug_assertions)]
                eprintln!(
                    "is_current_cursor_on_newline: no read-copy file path available (returning false)"
                );
                return Ok(false);
            }
        };

        // ═══════════════════════════════════════════════════════════════════════
        // DEFENSIVE CHECK 2: Get cursor's current file position
        // ═══════════════════════════════════════════════════════════════════════
        let cursor_file_pos = match self
            .get_row_col_file_position(self.cursor.tui_row, self.cursor.tui_visual_col)?
        {
            Some(pos) => pos,
            None => {
                #[cfg(debug_assertions)]
                eprintln!(
                    "is_current_cursor_on_newline: cursor ({}, {}) has no valid file position mapping (returning false)",
                    self.cursor.tui_row, self.cursor.tui_visual_col
                );
                return Ok(false);
            }
        };

        // ═══════════════════════════════════════════════════════════════════════
        // FILE READ: Read single byte at cursor position
        // ═══════════════════════════════════════════════════════════════════════
        let mut file = match File::open(read_copy_path) {
            Ok(f) => f,
            Err(_e) => {
                #[cfg(debug_assertions)]
                eprintln!(
                    "is_current_cursor_on_newline: failed to open file: {} (returning false)",
                    _e
                );
                return Ok(false);
            }
        };

        // Seek to cursor's byte position
        if let Err(_e) = file.seek(SeekFrom::Start(
            cursor_file_pos.byte_offset_linear_file_absolute_position,
        )) {
            #[cfg(debug_assertions)]
            eprintln!(
                "is_current_cursor_on_newline: failed to seek to byte {}: {} (returning false)",
                cursor_file_pos.byte_offset_linear_file_absolute_position, _e
            );
            return Ok(false);
        }

        // Read exactly 1 byte
        let mut byte = [0u8; 1];
        match file.read_exact(&mut byte) {
            Ok(_) => {
                // Successfully read byte - check if it's newline
                let is_newline = byte[0] == b'\n';

                #[cfg(debug_assertions)]
                eprintln!(
                    "is_current_cursor_on_newline: byte at {} is 0x{:02X} ({})",
                    cursor_file_pos.byte_offset_linear_file_absolute_position,
                    byte[0],
                    if is_newline { "NEWLINE" } else { "not newline" }
                );

                Ok(is_newline)
            }
            Err(_e) => {
                // Read failed - likely EOF or invalid position
                #[cfg(debug_assertions)]
                eprintln!(
                    "is_current_cursor_on_newline: failed to read byte at {}: {} (returning false)",
                    cursor_file_pos.byte_offset_linear_file_absolute_position, _e
                );
                Ok(false)
            }
        }
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
        let items_per_page = self.effective_rows - 1; // double header

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

                //  ==============================
                //  PastyPasteInputMode
                //  ==============================
                /*
                Idea:
                multi-line paste works in append-mode,
                but not well in full-lines
                so:
                have a 'Paste functionality' in Pasty
                to append into a new file
                then Pasty that file in
                */
                Ok(PastyInputPathOrCommand::PastyPasteInputMode) => {
                    // 1 make new file/path
                    let pasty_paste_path_base: Option<PathBuf> =
                        self.session_directory_path.clone();

                    //  // 1: Get clipboard directory
                    // let pasty_paste_path_base = self
                    //     .session_directory_path
                    //     .as_ref()
                    //     .ok_or_else(|| {
                    //         log_error(
                    //             "Session directory path is not set",
                    //             Some("copy_selection_to_clipboardfile"),
                    //         );
                    //         LinesError::StateError(
                    //             "Session directory path is not initialized".into(),
                    //         )
                    //     })?
                    //     .join("clipboard");

                    let extractedpath_base = match pasty_paste_path_base {
                        Some(p) => p,
                        None => PathBuf::from(""),
                    };

                    let extracted_path = extractedpath_base.join("clipboard");

                    // https://github.com/lineality/unique_temp_pathname_rust
                    let pasteinput_path =
                        create_unique_temp_name_and_file_filepathbuf(&extracted_path, "", 10, 10)?;

                    // Convert to absolute path (defensive)
                    let absolute_path = if pasteinput_path.is_absolute() {
                        pasteinput_path
                    } else {
                        // Make relative to current working directory
                        match std::env::current_dir() {
                            Ok(cwd) => cwd.join(&pasteinput_path),
                            Err(_) => {
                                let _ = self.set_info_bar_message("*path resolution failed*");
                                continue; // Stay in loop
                            }
                        }
                    };

                    print!("\x1B[2J\x1B[1;1H");
                    io::stdout().flush()?;

                    // 2. paste into file-path
                    // Lets users do N multi-line pastes, works like append-mode
                    // plus pasty mode
                    // TODO add to clipboard?
                    // ok to pass in handle...hopefully ^
                    let _ = pasty_paste_mode(&absolute_path, stdin_handle);

                    // 3. Insert

                    // Insert file at cursor
                    if let Err(_) = insert_file_at_cursor(self, &absolute_path) {
                        let _ = self.set_info_bar_message("*insert failed*");
                        continue; // Stay in loop
                    }

                    let _ = self.set_info_bar_message(""); // Clear messages

                    return Ok(true); // Exit Pasty mode
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
    ///  // User types "3F" at byte position 42 in hex mode
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
    ///  escape key for normal mode
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

        //  =======================
        //  Parse Hex Mode Commands
        //  =======================

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
                                    Some("get_undo_changelog_directory_path:changelog"),
                                );
                                // safe
                                log_error(
                                    "Cannot get changelog directory",
                                    Some("get_undo_changelog_directory_path:changelog"),
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
                                            Some(
                                                "button_make_changelog_from_user_character_action_level:changelog",
                                            ),
                                        );

                                        // safe
                                        log_error(
                                            "Failed to log newline",
                                            Some(
                                                "button_make_changelog_from_user_character_action_level:changelog",
                                            ),
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
                    let _ = self.set_info_bar_message("(Added a Byte)");

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
                                    Some("get_undo_changelog_directory_path:changelog"),
                                );

                                #[cfg(not(debug_assertions))]
                                log_error(
                                    "Cannot get changelog directory",
                                    Some("get_undo_changelog_directory_path:changelog"),
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
                                            Some(
                                                "button_make_changelog_from_user_character_action_level:changelog",
                                            ),
                                        );

                                        #[cfg(not(debug_assertions))]
                                        log_error(
                                            "Failed to log newline",
                                            Some(
                                                "button_make_changelog_from_user_character_action_level:changelog",
                                            ),
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
            "n" | "\x1b" | "q" | "b" => {
                // Exit to normal mode
                keep_editor_loop_running = execute_command(self, Command::EnterNormalMode)?;
            }

            "i" => {
                // Exit to insert mode
                keep_editor_loop_running = execute_command(self, Command::EnterInsertMode)?;
            }

            "v" => {
                // Exit to visual mode
                keep_editor_loop_running = execute_command(self, Command::EnterVisualSelectMode)?;
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

            "wq" | "sq" => {
                // SaveAndQuit
                keep_editor_loop_running = execute_command(self, Command::SaveAndQuit)?;
            }

            // safer to have q return to normal mode

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

    /// Runs one keystroke-input session: owns the raw terminal and the read loop.
    ///
    /// # Project Context
    ///
    /// This is the ONLY place in the entire editor that creates and owns a
    /// `RawTerminal` (Linux termios raw mode: no line buffering, no echo,
    /// byte-by-byte input). It is the raw-terminal analogue of the cooked-mode
    /// `handle_utf8txt_insert_mode_input`.
    ///
    /// ## Why RawTerminal Is Owned Here (and nowhere else)
    ///
    /// The main loop (`lines_fullfile_editor_core`) acquires a single
    /// `StdinLock` for the whole session, and every OTHER mode reads cooked,
    /// Enter-terminated, echoed lines from it. If the terminal were put into raw
    /// mode at the main-loop level, all those cooked-input modes would break.
    ///
    /// Owning `RawTerminal` here means:
    ///   - Raw mode is active ONLY for the duration of one keystroke session.
    ///   - `RawTerminal`'s `Drop` restores the original terminal settings on
    ///     EVERY exit path — clean exit, EOF, read error, propagated error, and
    ///     even panic. This is exactly what the `#[must_use]` + `Drop` design is
    ///     built for.
    ///   - No `RawTerminal` is stored in `EditorState` (minimal-state rule). The
    ///     handle is a transient local.
    ///
    /// ## The Cooked-Render / Raw-Read Cycle (staircase fix)
    ///
    /// PROBLEM (the "staircase"):
    /// `render_tui_utf8txt` writes a bare `\n` (line feed) at the end of each
    /// display row. In a COOKED terminal the kernel's `OPOST` output flag is on,
    /// so the driver automatically expands every `\n` into `\r\n` (line feed +
    /// carriage return); the carriage return is what returns the cursor to
    /// column 0 at the start of each new line.
    ///
    /// In RAW mode `OPOST` is OFF (see `RawTerminal`/`make_raw` docs). A bare
    /// `\n` then moves the cursor DOWN one line but does NOT return it to
    /// column 0. Each rendered row therefore starts further to the right than
    /// the last — a diagonal "staircase" — and the whole frame is geometrically
    /// corrupted.
    ///
    /// SOLUTION (this method):
    /// We render INSIDE a brief cooked window. Every loop iteration:
    ///   1. `suspend_raw_mode()` — restore the ORIGINAL (cooked) terminal, so
    ///      `OPOST` is on and `\n` → `\r\n` again.
    ///   2. `render_tui_utf8txt(self)` — runs in the same cooked terminal state
    ///      that Normal/Insert/Visual/Pasty modes render in, so the output is
    ///      byte-for-byte identical to those modes. No staircase.
    ///   3. `activate_raw_mode()` — return to raw mode (byte-by-byte, no echo)
    ///      for the keystroke read. `activate_raw_mode` also re-verifies that
    ///      ICANON/ECHO are cleared and VMIN is non-zero, giving a per-keystroke
    ///      check on the terminal state.
    ///   4. `term.read(one byte)` — blocking single-byte read (VMIN=1, VTIME=0).
    ///   5. dispatch the byte.
    ///
    /// This is the documented suspend/activate pattern the raw-terminal module
    /// was designed for (the module's `run_subprocess_demo` uses the same cycle
    /// to hand a cooked terminal to a subprocess and then reclaim raw mode).
    ///
    /// COST: two extra ioctl syscalls per keystroke (suspend = 1 write;
    /// activate = 1 write + 1 read-back verify). At human typing speed this is
    /// imperceptible. We accept it in exchange for reusing the UNCHANGED shared
    /// renderer instead of forking a `\r\n` variant.
    ///
    /// WHY NOT change `render_tui_utf8txt` to write `\r\n`:
    /// That function is shared by all the cooked modes. In cooked mode `OPOST`
    /// would expand a literal `\r\n` into `\r\r\n` (double carriage return), and
    /// we would be altering a working, shared function. Keeping the renderer
    /// untouched and toggling the terminal here is the smaller, lower-risk change.
    ///
    /// ## Session Loop Shape
    ///
    /// ```text
    /// create RawTerminal (on failure: log, set Normal, return Ok(true))
    /// loop while self.mode == KeystrokeInputMode:
    ///     term.suspend_raw_mode()             // -> cooked terminal
    ///     render_tui_utf8txt(self)            // renders like every other mode
    ///     term.activate_raw_mode()            // -> raw terminal (verified)
    ///     n = term.read(&mut [0u8; 3])         // VMIN=1: returns 1..=3 bytes
    ///     match n:
    ///         Ok(0)  -> EOF: break, set Normal
    ///         Ok(k)  -> if classify_arrow_bytes(&buf[0..k]) == Some(dir):
    ///                       handle_arrow_key_input_mode(self, dir)   // checked
    ///                   else:
    ///                       for byte in &buf[0..k]:                  // A2, no drop
    ///                           handle_single_byte_keystroke_input_mode(self, byte, &read_copy)  // checked
    ///         Err(_) -> break, set Normal
    /// (RawTerminal drops here -> terminal restored)
    /// return Ok(true)
    /// ```
    ///
    /// ## Termination / Recovery (the satellite must not fall out of the sky)
    ///
    /// The inner loop ends, and this method returns to the main loop, on any of:
    ///
    ///   1. ESC byte: the dispatcher routes it through `EnterNormalMode`, which
    ///      sets `self.mode = Normal`. The `while self.mode == KeystrokeInputMode`
    ///      condition then fails -> clean exit.
    ///
    ///   2. `term.read` returns `Ok(0)` (EOF): the input source vanished. We
    ///      MUST break — otherwise the loop would spin forever calling read on a
    ///      dead terminal, getting Ok(0) every time, never advancing, never
    ///      exiting. (RawTerminal's docs warn explicitly: "A return of Ok(0)
    ///      means EOF... Callers MUST check the returned count.") After breaking
    ///      we explicitly set `self.mode = Normal` so the editor lands in a
    ///      known-good state.
    ///
    ///   3. `term.read` returns `Err`: same treatment — break, set Normal.
    ///
    ///   4. `RawTerminal::new()` fails at entry: log terse, set `self.mode =
    ///      Normal`, set info-bar message, return `Ok(true)`. We do NOT crash and
    ///      do NOT propagate as the only exit; the editor recovers to Normal mode
    ///      and the main loop continues.
    ///
    /// In every exit path this method returns `Ok(true)` (keep the main editor
    /// loop running). It never returns `Ok(false)`: keystroke-input mode has no
    /// quit command. ESC goes to Normal; quitting is done from Normal mode.
    ///
    /// ## Suspend / Activate Failure Handling
    ///
    /// `suspend_raw_mode` and `activate_raw_mode` can fail (kernel/driver error,
    /// a transient termios glitch, bit-flip). We handle each failure rather than
    /// crash:
    ///   - If `suspend_raw_mode` fails, we still attempt the render. Worst case
    ///     this single frame staircases; the next iteration retries the suspend.
    ///   - If `activate_raw_mode` fails, raw mode may not be (re)established. A
    ///     read in cooked mode would block for a whole line instead of one byte,
    ///     which would silently break byte-by-byte input. Rather than risk that
    ///     confusing failure mode, we recover to Normal mode and exit the session
    ///     cleanly. (RawTerminal::Drop still restores the terminal on the way out.)
    /// Both failures are logged terse (no PII) in production; debug builds get
    /// detail.
    ///
    /// ## Render Inside the Loop (not just once)
    ///
    /// The main loop renders once per outer iteration. But this session stays
    /// inside its own inner loop across MANY keystrokes without returning to the
    /// main loop. Therefore it must render itself, once per keystroke, at the top
    /// of the loop. We use `render_tui_utf8txt` — the same renderer the main loop
    /// uses for normal text modes — because keystroke-input mode displays normal
    /// UTF-8 text. The edit functions own their own `build_windowmap_nowrap`
    /// rebuilds; this method only renders.
    ///
    /// ## Bounded vs Always-Loop (Power of 10, Rule 2)
    ///
    /// The inner loop is unbounded by design (it waits on external user input),
    /// but it has clear, multiple exit conditions (ESC, EOF, read error,
    /// activate failure, mode flip) and never busy-spins: every iteration blocks
    /// on `term.read` (VMIN=1, VTIME=0), and EOF/Err break immediately rather
    /// than looping. The failsafe layer is `RawTerminal::Drop`, which restores
    /// the terminal even if this loop exits abnormally.
    ///
    /// # Arguments
    ///
    /// * `read_copy_path` - borrow of the read-copy file path. The CALLER (the
    ///   main loop) owns the clone and passes a borrow. This method passes that
    ///   same borrow down to `handle_single_byte_keystroke_input_mode` for every keystroke,
    ///   so the path is cloned exactly once per session (in the main loop), not
    ///   once per keystroke.
    ///
    /// # Returns
    ///
    /// * `Ok(true)`  - keep the main editor loop running (always, in this mode)
    /// * `Err(LinesError)` - an unrecoverable error propagated from rendering or
    ///   from a keystroke action. On the way out, `RawTerminal::Drop` still
    ///   restores the terminal. The main loop will surface the error.
    ///
    /// # Defensive Notes
    ///
    /// - No `unwrap` / no panic.
    /// - EOF, read errors, and activate-raw failures are handled, not ignored,
    ///   and always leave the editor in Normal mode.
    /// - Read buffer `[0u8; 3]` sized for one 3-byte arrow escape sequence.
    ///   VMIN=1 means read returns 1..=3 bytes (it does NOT wait for 3). An
    ///   exact 3-byte arrow match is dispatched as one arrow; any other 1..=3
    ///   bytes are dispatched per-byte so none is dropped (A2). Handler return
    ///   values are checked, not discarded: Ok(false) is unexpected in this
    ///   mode and recovers to Normal.
    fn handle_keystroke_input_session(&mut self, read_copy_path: &Path) -> Result<bool> {
        // ---------------------------------------------------------------------
        // Step 1: Enter raw terminal mode (RAII).
        // ---------------------------------------------------------------------
        // On failure we recover to Normal mode rather than crashing. The most
        // common failure is "no controlling terminal" (e.g. stdin redirected or
        // running headless), in which case keystroke-input mode simply cannot
        // function and we fall back to Normal mode.
        let mut term = match RawTerminal::new() {
            Ok(t) => t,
            Err(_e) => {
                #[cfg(debug_assertions)]
                eprintln!("hkis: RawTerminal::new failed: {:?}", _e);

                // Terse, no-PII production log.
                log_error(
                    "raw terminal unavailable",
                    Some("handle_keystroke_input_session:new"),
                );

                // Recover to a known-good mode and inform the user.
                self.mode = EditorMode::Normal;
                let _ = self.set_info_bar_message("ki unavailable (no tty)");

                // Keep the editor running; the main loop continues in Normal mode.
                return Ok(true);
            }
        };

        // ---------------------------------------------------------------------
        // Step 2: Keystroke read loop (cooked-render / raw-read cycle).
        // ---------------------------------------------------------------------
        // Read buffer is sized for the largest single keystroke unit we classify
        // atomically: a 3-byte arrow escape sequence (0x1B 0x5B 0x41..=0x44).
        //
        // The terminal is configured VMIN=1, VTIME=0: read returns as soon as
        // AT LEAST one byte is available, delivering up to 3 bytes if that many
        // have already arrived. Therefore:
        //   - A one-keypress arrow whose 3 bytes arrive together returns n == 3
        //     and is classified as a single arrow by classify_arrow_bytes.
        //   - read does NOT wait for 3 bytes. A lone byte still returns n == 1
        //     (e.g. one typed character, or the first byte of a FRAGMENTED arrow
        //     over a slow link — see classify_arrow_bytes' fragmentation note).
        //   - Fast/pasted typing may return n == 2 or n == 3 printable bytes in
        //     one read. We MUST dispatch each of those bytes (the per-byte loop
        //     below), or enlarging this buffer from [0u8; 1] to [0u8; 3] would
        //     silently drop trailing bytes — a regression we explicitly avoid.
        let mut byte_buffer = [0u8; 3];

        // Loop while we remain in keystroke-input mode. ESC flips the mode to
        // Normal (via the dispatcher), which ends this loop.
        while self.mode == EditorMode::KeystrokeInputMode {
            // -----------------------------------------------------------------
            // (a) Suspend raw mode -> cooked terminal for rendering.
            // -----------------------------------------------------------------
            // Restores the ORIGINAL terminal (OPOST on), so render_tui_utf8txt's
            // bare '\n' is expanded to '\r\n' by the driver, exactly as in every
            // other editor mode. This is the staircase fix.
            //
            // If suspend fails, we still render (worst case: one staircased
            // frame); the next iteration retries. We do not abort on suspend
            // failure because a single bad frame is recoverable.
            if let Err(_e) = term.suspend_raw_mode() {
                #[cfg(debug_assertions)]
                eprintln!("hkis: suspend_raw_mode failed: {:?}", _e);

                log_error(
                    "ki suspend failed",
                    Some("handle_keystroke_input_session:suspend"),
                );
                // Continue: attempt the render anyway.
            }

            // -----------------------------------------------------------------
            // (b) Render the TUI for the current model state (cooked terminal).
            // -----------------------------------------------------------------
            // Unconditional, once per keystroke. The edit functions rebuilt the
            // windowmap; here we only paint it. Errors propagate (RawTerminal
            // Drop will still restore the terminal on the way out).
            render_tui_utf8txt(self)?;

            // -----------------------------------------------------------------
            // (c) Re-activate raw mode -> raw terminal for byte-by-byte read.
            // -----------------------------------------------------------------
            // activate_raw_mode re-applies raw settings derived from the saved
            // original AND verifies ICANON/ECHO/VMIN. If it fails, raw mode may
            // not be established; a subsequent read could block for a whole line
            // instead of one byte (a confusing silent failure). To avoid that,
            // we recover to Normal mode and exit the session cleanly. Drop still
            // restores the terminal.
            if let Err(_e) = term.activate_raw_mode() {
                #[cfg(debug_assertions)]
                eprintln!("hkis: activate_raw_mode failed: {:?}; exiting ki mode", _e);

                log_error(
                    "ki activate failed",
                    Some("handle_keystroke_input_session:activate"),
                );

                self.mode = EditorMode::Normal;
                break;
            }

            // -----------------------------------------------------------------
            // (d) Read exactly one byte from the raw terminal.
            // -----------------------------------------------------------------
            match term.read(&mut byte_buffer) {
                // EOF: input source vanished. Break and recover to Normal mode.
                // We MUST NOT continue looping on Ok(0): read would keep
                // returning Ok(0) forever (a dead-tty spin). Break immediately.
                Ok(0) => {
                    #[cfg(debug_assertions)]
                    eprintln!("hkis: read returned Ok(0) (EOF); exiting ki mode");

                    // Recover to a known-good mode.
                    self.mode = EditorMode::Normal;
                    break;
                }

                // Got at least one byte. First try to classify the whole read as
                // a single 3-byte arrow escape sequence. If it is NOT an arrow,
                // dispatch each returned byte individually so no byte is dropped.
                //
                // bytes_read is the count read returned (guaranteed 1..=3 here:
                // Ok(0) is handled by the EOF arm above, and the buffer is 3
                // bytes wide). We slice byte_buffer to exactly the filled region.
                Ok(bytes_read) => {
                    // Defensive: clamp the slice end to the buffer length so a
                    // bogus oversized count (bit-flip / driver bug) cannot index
                    // out of bounds. min() makes this branch-safe with no panic.
                    let filled_end = bytes_read.min(byte_buffer.len());
                    let filled_buffer = &byte_buffer[0..filled_end];

                    // ---- Arrow path: exact 3-byte escape sequence only. ----
                    if let Some(arrow_direction) = classify_arrow_bytes(filled_buffer) {
                        // Map the classified direction to a cursor-move command.
                        // The returned bool is the keep-running flag; loop exit is
                        // driven by self.mode, not by this value, but we check it
                        // defensively rather than discarding it (see below).
                        match handle_arrow_key_input_mode(self, arrow_direction)? {
                            true => {
                                // Expected: keep running. Loop continues; exit is
                                // governed by self.mode (set to Normal by ESC).
                            }
                            false => {
                                // Not expected in this mode: cursor moves never
                                // request termination. Treat an Ok(false) as an
                                // unexpected contract change and recover safely
                                // rather than silently ignoring it.
                                #[cfg(debug_assertions)]
                                eprintln!(
                                    "hkis: arrow handler returned Ok(false) (unexpected); exiting ki mode"
                                );

                                log_error(
                                    "ki arrow unexpected stop",
                                    Some("handle_keystroke_input_session:arrow"),
                                );

                                self.mode = EditorMode::Normal;
                                break;
                            }
                        }
                    } else {
                        // ---- Per-byte path (A2): dispatch every read byte. ----
                        // Not an arrow. This covers a single typed character
                        // (n == 1) AND fast/pasted multi-byte text (n == 2..=3)
                        // AND the individual bytes of non-arrow escape sequences
                        // (which the single-byte dispatcher silently ignores).
                        //
                        // We iterate over the filled slice so NO byte is dropped.
                        // ESC (0x1B) appearing here as a lone byte flips self.mode
                        // to Normal inside the dispatcher; the for-loop then
                        // finishes its remaining (if any) bytes for this read, and
                        // the while-condition ends the session next iteration.
                        for &single_byte in filled_buffer {
                            // The dispatcher returns Ok(true) to keep running, or
                            // Err on an unrecoverable edit I/O failure
                            // (propagated; terminal restored on Drop). We check
                            // the bool explicitly rather than discarding it.
                            match handle_single_byte_keystroke_input_mode(
                                self,
                                single_byte,
                                read_copy_path,
                            )? {
                                true => {
                                    // Expected: keep running.
                                }
                                false => {
                                    // Not expected in this mode (no quit command).
                                    // Recover safely instead of ignoring it.
                                    #[cfg(debug_assertions)]
                                    eprintln!(
                                        "hkis: single-byte handler returned Ok(false) (unexpected); exiting ki mode"
                                    );

                                    log_error(
                                        "ki byte unexpected stop",
                                        Some("handle_keystroke_input_session:byte"),
                                    );

                                    self.mode = EditorMode::Normal;
                                    break;
                                }
                            }
                        }
                    }
                }

                // Read error: the terminal became unavailable. Break and recover.
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    eprintln!("hkis: read error: {:?}; exiting ki mode", _e);

                    // Terse, no-PII production log.
                    log_error("ki read error", Some("handle_keystroke_input_session:read"));

                    // Recover to a known-good mode.
                    self.mode = EditorMode::Normal;
                    break;
                }
            }
        }

        // ---------------------------------------------------------------------
        // Step 3: Leave the session.
        // ---------------------------------------------------------------------
        // `term` (RawTerminal) drops here, restoring the original (cooked)
        // terminal settings. We return Ok(true) so the main editor loop
        // continues; by now self.mode is Normal (set by ESC, EOF, error, or
        // activate-raw failure).
        Ok(true)
    }

    /// Handles all input when the editor is in Insert mode.
    ///
    /// # Overview
    ///
    /// This method is responsible for ALL insert mode input handling, including:
    /// 1. **Command detection** - Recognizing special commands
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
    /// | ESC-key | Enter Normal Mode | Switch to normal mode | Continue |
    /// | `Delete key` | Delete Backspace | Delete character | Continue |
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
    /// is threaded through any sub-methods.
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
        //  ===========
        //  Insert Mode
        //  ===========
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

        //  ========================
        //  Check for Commands First
        //  ========================

        // Check for exit insert mode commands
        // Only escape key to leave insert mode
        // possible to turn off all ascii keys
        if trimmed == "\x1b" {
            keep_editor_loop_running = execute_command(self, Command::EnterNormalMode)?;
        } else if trimmed == "\x1b[3~" {
            // This is delete-key
            // Do nothing if delete key entered...
            keep_editor_loop_running = execute_command(self, Command::DeleteBackspace)?;
        } else if text_input_str == "\n" || text_input_str == "\r\n" {
            // note: empty isn't empty, it contains a newline
            // Empty line = newline insertion
            keep_editor_loop_running = execute_command(self, Command::InsertNewline('\n'))?;
            build_windowmap_nowrap(self, &read_copy)?; // Rebuild immediately after newline
        } else {
            //  ==============
            //  Text to Insert
            //  ==============

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
                // TODO: this equivalence is taken to indicate what?
                !ends_with_newline && bytes_read == TEXT_BUCKET_BRIGADE_CHUNKING_BUFFER_SIZE;

            // Process the chunk, handling multiple newlines
            let mut chunk_start = 0;

            while chunk_start < bytes_read {
                // =================
                // Handling Newlines
                // =================
                /*
                Issues and known strangeness.

                1. Newlines
                Due to newlines '\n' being the content & code signal for one or more things
                relating to stdin, There may be no perfect way to handle them.

                Multi-line cut and past (for lines less than ~200 char) can work
                smoothly if you double-newline where the inner-newlines are.

                e.g.
                ```
                1 fish

                2 f
                i
                s
                h


                3 f

                i

                s

                h

                ```
                1 and 2 appear the same, 3 looks like 2


                2. there is a long-line bug which is triggered by
                single newlines becoming long-lines.
                Bug: If the line is longer than ~200char, something breaks
                sometimes causes an error ("cursor not on valid file position") from here:
                ```rust
                    // Step 1: Get file position at/of/where  cursor (with graceful error handling)
                    let file_pos = match lines_editor_state.get_row_col_file_position(
                        lines_editor_state.cursor.tui_row,
                        lines_editor_state.cursor.tui_visual_col,
                    ) {
                        Ok(Some(pos)) => pos,
                        Ok(None) => {
                            eprintln!("Warning: Cannot insert - cursor not on valid file position");
                            log_error(
                                "Insert newline failed: cursor not on valid file position",
                                Some("insert_newline_at_cursor_chunked"),
                            );
                            return Ok(());
                ```
                "cursor not on valid file position"
                Sometimes not.
                Lines does not panic or exit or restart, it just hangs, which is odd.

                3. The clean alternative, which is best for large texts
                anyway most likely, is to import a .txt doc (not copy-paste with OS heap)

                4. minimal 'append-mode' works just fine (funnily enough)
                So possibly the issue is knowing where to move the cursor to after input...

                if brute force:

                file byte length before and after insert, and move cursor ahead the difference?

                5. There is also the hex-write system, by which characters are hex-edited in
                onto blank spaces, which is odd, unless the file is huge then it makes sense.

                Update:
                See the pasty paste-in section: "paste multi-line cut and paste"
                */

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
            if trimmed == "\x1b" {
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
                let _ = self.set_info_bar_message("Use: sa FILENAME");
                return Command::None;
            }

            // Defensive: Check filename length to prevent overflow
            // Catches: Extremely long filenames that could cause issues
            if filename_str.len() > limits::LINE_CHUNK_READ_BYTES {
                // TODO: this max length is too big?
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

                "u" | "undo" => Command::UndoButtonsCommand,
                "re" | "redo" => Command::RedoButtonsCommand,

                "w" => Command::MoveWordForward(count),
                "e" => Command::MoveWordEnd(count),
                "b" => Command::MoveWordBack(count),

                // toggle
                "/" => Command::ToggleCommentOneLine(self.cursor.tui_row), // zero index
                "///" => Command::ToggleDocstringOneLine(self.cursor.tui_row), // zero index

                // indent
                "[" => Command::UnindentOneLine(self.cursor.tui_row), // zero index
                "]" => Command::IndentOneLine(self.cursor.tui_row),   // zero index

                // TUI Size
                "tall+" => Command::TallPlus,
                "tall-" => Command::TallMinus,
                "wide+" => Command::WidePlus,
                "wide-" => Command::WideMinus,

                "i" => Command::EnterInsertMode,
                // Keystroke-input mode: byte-by-byte ASCII via raw terminal.
                // Distinct from "i" (cooked insert mode). See
                // Command::EnterKeystrokeInputMode and EditorMode::KeystrokeInputMode.
                "ki" => Command::EnterKeystrokeInputMode,
                "v" => Command::EnterVisualSelectMode,
                // Multi-character commands
                "wq" | "sq" => Command::SaveAndQuit,
                "s" | "ww" => Command::SaveFileStandard,
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
                    Command::ToggleBlockcomments(self.selection_rowline_start, self.cursor.tui_row)
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
                "s" | "ww" => Command::SaveFileStandard,
                "n" | "\x1b" => Command::EnterNormalMode,
                "wq" | "sq" => Command::SaveAndQuit,
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
    ///  // OLD: continue; (skip to next iteration)
    ///  // NEW: return Ok(true); (keep loop running → goes to next iteration)
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
        // Ignore invalid UTF-8
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
            if trimmed == "help" {
                display_help_menu_system(stdin_handle)?; // stdin_handle: &mut StdinLock,
            }

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
        //  =================================================
        //  Debug-Assert, Test-Asset, Production-Catch-Handle
        //  =================================================
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
        for row_idx in 0..MAX_TUI_ROWS {
            for col_idx in 0..DEFAULT_COLS {
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
        fileline_number_for_display: usize, // fileline_number_for_display
        starting_row: usize,                // line_count_at_top_of_window
    ) -> io::Result<usize> {
        // Validate row index (zero based)
        if row_idx > MAX_ZERO_INDEX_TUI_ROWS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Row index exceeds maximum 44",
            ));
        }

        // Check if we need padding
        let needs_padding = row_needs_extra_padding_bool(
            starting_row,                // starting_row
            fileline_number_for_display, // fileline_number_for_display
            self.effective_rows,         // effective_rows
        );

        // Convert number to bytes directly into buffer
        let mut write_pos = 0;

        // Add leading space if needed
        if needs_padding {
            self.utf8_txt_display_buffers[row_idx][0] = b' ';
            write_pos = 1;
        }

        // Write digits directly
        let mut temp_num = fileline_number_for_display;
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

        // Write digits in order
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
    const STDIN_CHUNK_SIZE: usize = 4;
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

        // =================================================
        // Debug-Assert, Test-Asset, Production-Catch-Handle
        // =================================================
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

/// Lets users do N multi-line pastes, works like append-mode
pub fn pasty_paste_mode<R: BufRead>(absolute_path: &Path, stdin_handle: &mut R) -> Result<()> {
    // Pre-allocated buffer for bucket brigade stdin reading
    const STDIN_CHUNK_SIZE: usize = 64;

    let mut stdin_chunk_buffer = [0u8; STDIN_CHUNK_SIZE];

    // Open file in append mode once (keeps handle open for session)
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&absolute_path)?;

    // buffy_print("Paste multiline text here. ", &[])?;
    io::stdout().flush()?;

    let mut chunk_counter = 0;

    // Main editor loop
    loop {
        buffy_print("\x1B[2J\x1B[1;1H", &[])?;
        io::stdout().flush()?;
        write_red_hotkey("", "Paste multiline text here. Type '")?;
        write_red_hotkey("b", "' to go")?;
        write_red_hotkey(" back", ". Paste here:")?;
        buffy_print("{} > ", &[BuffyFormatArg::Str(RESET)])?;
        io::stdout().flush()?;

        // Defensive: prevent infinite loop
        chunk_counter += 1;
        if chunk_counter > limits::MAX_CHUNKS {
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

        // =================================================
        // Debug-Assert, Test-Asset, Production-Catch-Handle
        // =================================================
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
            if trimmed == "b" || trimmed == "back" || trimmed == "q" {
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

// ════════════════════════════════════════════════════════════════════════════
// CHUNKED LINE READING  (memory-thrifty line traversal for NoWrap rendering)
// ════════════════════════════════════════════════════════════════════════════
//
// PURPOSE (project context)
// -------------------------
// The editor must never hold a whole file — or even a whole line — in memory.
// This section provides the one primitive used to walk a file line one UTF-8
// character at a time, holding at most `limits::LINE_CHUNK_READ_BYTES` bytes in
// flight. Two consumers use it:
//
//   1. `build_windowmap_nowrap` — fills the on-screen display buffers and the
//      per-row file-byte ranges for one terminal window.
//   2. `goto_line_end`          — measures a line's visual width (and computes a
//      horizontal scroll offset) for the "End" key.
//
// This section REPLACES two earlier whole-line functions:
//   - `read_single_line`         (read a whole line, one byte at a time, into a
//                                 4096-byte buffer)            — DELETED
//   - `process_line_with_offset` (operated on a full `&[u8]` line)  — DELETED;
//      its skip / visible-write / window-map / `␤` / EOL logic now lives inline
//      inside `build_windowmap_nowrap`.
// `get_utf8_char_byte_length_from_buffer` is RETAINED and reused below.
//
// COMPONENTS
// ----------
//   - `enum LineCharStep { Char { bytes, len }, Newline, Eof }`
//        One unit yielded per call: a character (returned BY VALUE), a consumed
//        '\n' terminator, or end-of-file before any newline.
//   - `struct ChunkReaderState { valid_len, cursor, reached_eof }`
//        Borrow-free position state carried across calls (NO borrows held).
//   - `fn next_line_char(file, scratch, rs) -> Result<LineCharStep>`
//        The primitive: refills `scratch` in chunks, hands back one character.
//   - `fn utf8_declared_len_from_first_byte(u8) -> usize`
//        First-byte-only declared length (1..=4), used ONLY for straddle
//        detection (see "UTF-8 straddling" below).
//   - `fn visual_width_of_char(&[u8]) -> usize`
//        Terminal-CELL width (1 or 2) of one character; shared by both consumers.
//
// KEY DESIGN DECISION #1 — Borrow-free reader state (no per-line seek)
// -------------------------------------------------------------------
// The scratch buffer lives on `EditorState::line_chunk_scratch` (req: shared,
// no per-call stack allocation). A reader that *stored* `&mut scratch` could not
// coexist with the `&mut self` methods `write_line_number` / `set_line_byte_range`.
// So `ChunkReaderState` holds ONLY indices, and `scratch` is passed to
// `next_line_char` per call. The borrow of `line_chunk_scratch` is therefore
// released the instant each call returns, leaving the caller free to invoke
// `&mut self` methods between characters.
//
// A consequence (and a feature): a SINGLE `ChunkReaderState` can span an entire
// `build_windowmap_nowrap` run. `next_line_char` consumes a line's '\n' but the
// `scratch`/`rs` pair retains bytes already read past it — those become the first
// bytes of the next line. Reading is strictly SEQUENTIAL through the file with
// NO per-line `seek`. (`goto_line_end` is the exception: it `seek`s to the line
// start once per scan pass and uses a fresh `ChunkReaderState` each pass.)
//
// KEY DESIGN DECISION #2 — UTF-8 characters straddling a chunk boundary
// --------------------------------------------------------------------
// A multi-byte character can span two chunks. `get_utf8_char_byte_length_from_buffer`
// returns `Ok(1)` for an incomplete sequence at a buffer END — correct for true
// truncation, WRONG for a character merely split across a boundary. So before
// validating, `next_line_char` reads the *declared* length from the first byte
// (`utf8_declared_len_from_first_byte`). If the declared extent runs past
// `valid_len` and we are not at EOF, the ≤3 leftover bytes are slid to the front
// of `scratch`, a fresh chunk is read in behind them, and the character is
// re-evaluated. The authoritative length (with continuation-byte validation)
// always comes from `get_utf8_char_byte_length_from_buffer` once the bytes are
// present.
//
// REQUIREMENT: `limits::LINE_CHUNK_READ_BYTES >= 4`, so a 4-byte character always
// fits after one slide+refill. This is debug-asserted in `next_line_char`. The
// configured sizes (e.g. 64 or 4096) satisfy it; do NOT set the chunk below 4.
//
// KEY DESIGN DECISION #3 — Coordinate spaces (do not conflate)
// ------------------------------------------------------------
//   - CHARACTER space: 1 per character (a kanji counts as 1). The horizontal
//     scroll offset `tui_window_horizontal_utf8txt_line_char_offset` and the
//     skip phase live here (whole characters are skipped, never bytes).
//   - VISUAL space:    1 per ASCII / single-width, 2 per double-width (CJK,
//     emoji). `cursor.tui_visual_col`, `effective_cols`, and the terminal live
//     here. `visual_width_of_char` is the single source of truth for the 1-vs-2
//     decision, used identically by both consumers so the cursor round-trip
//     through `get_row_col_file_position` stays consistent.
//   Preserved layout quirk: `build_windowmap_nowrap` advances `display_col` by 1
//   per displayed character (one cursor stop each) while the right-edge checks
//   gate on VISUAL width. This matches the prior behavior intentionally.
//
//
// LINE-END SEMANTICS
// ------------------
//   - `Newline` step: the '\n' is consumed, not counted as content; the caller
//     sets `found_newline = true`. `file_byte_position += content_bytes + 1`.
//   - `Eof` step with zero content and no newline: end of file. In
//     `build_windowmap_nowrap` this drives the EOF marker (`eof_fileline_tuirow_tuple`)
//     and stops the row loop. A content line with NO trailing newline is a valid
//     last line: it is rendered, then the NEXT row's first `next_line_char`
//     returns `Eof` and triggers the marker.
//   - Empty line (just '\n'): `content_bytes == 0`, recorded as
//     `start_byte == end_byte` (the project's "empty line" signal).
//
// DEFENSIVE PROGRAMMING / POWER-OF-TEN
// ------------------------------------
//   - Every loop is bounded: `next_line_char` refill loop by `limits::MAX_CHUNKS`;
//     `build_windowmap_nowrap` row loop by `limits::WINDOW_BUILD_LINES` and its
//     per-line character loop by `limits::MAX_CHUNKS`; skip/write phases also by
//     `limits::HORIZONTAL_SCROLL_CHARS`; `goto_line_end` scans by `limits::MAX_CHUNKS`.
//   - Malformed / truncated UTF-8 degrades to single-byte / single-cell handling
//     (matches the renderer's tolerance) — it never panics.
//   - All display-buffer writes are bounds-checked against `MAX_DISPLAY_BUFFER_BYTES`.
//   - No heap, no recursion, no unsafe. `goto_line_end` absorbs all I/O failures,
//     reports a terse data-free info-bar message, logs detail only under
//     `#[cfg(debug_assertions)]`, and returns `Ok(())` so the editor keeps running.
//
// MAINTENANCE NOTES
// -----------------
//   - If you ever store the scratch borrow in a struct again, Decision #1 breaks
//     and you will be forced back to per-line `seek`s. Keep `ChunkReaderState`
//     borrow-free.
//   - `next_line_char` returns the character BY VALUE specifically so the scratch
//     borrow does not escape; do not change it to return a slice into `scratch`.
//   - `utf8_declared_len_from_first_byte` and `get_utf8_char_byte_length_from_buffer`
//     duplicate the first-byte classification by design: the former exists ONLY
//     to detect chunk-straddle before the latter validates. Keep them in sync.
// ════════════════════════════════════════════════════════════════════════════

/// One step produced by the chunked, per-character line reader.
///
/// # Purpose (Project Context)
/// The editor must never hold a whole line (or whole file) in memory. Display
/// rebuilds and "End"-key handling both walk a line one UTF-8 character at a
/// time, refilling a small fixed buffer (`limits::LINE_CHUNK_READ_BYTES`) as
/// needed. `next_line_char` yields exactly one of these per call.
///
/// # Variants
/// * `Char { bytes, len }` - One complete UTF-8 character. `bytes[..len]` holds
///   its 1..=4 bytes (the trailing slots are zero padding, never read). The
///   character is returned BY VALUE so the small scratch buffer borrow does not
///   escape the call — that is what lets the caller freely call `&mut self`
///   methods (`write_line_number`, `set_line_byte_range`) between characters.
/// * `Newline` - The line was terminated by a `\n` (which is consumed but not
///   reported as content). Caller stops the line here; `found_newline = true`.
/// * `Eof` - The underlying file produced no more bytes before any newline. The
///   line (or file) ends here with no trailing newline.
#[derive(Debug)]
enum LineCharStep {
    Char { bytes: [u8; 4], len: usize },
    Newline,
    Eof,
}

/// Borrow-free position state for the chunked line reader.
///
/// # Purpose (Project Context)
/// This struct deliberately holds NO borrows — only indices into the caller's
/// scratch buffer and an EOF flag. The scratch buffer itself
/// (`EditorState::line_chunk_scratch`) is passed to `next_line_char` per call,
/// so the borrow of that field is released the moment `next_line_char` returns.
///
/// Because of that, a single `ChunkReaderState` can stay alive across an entire
/// `build_windowmap_nowrap` run while the loop body still calls `&mut self`
/// methods on `EditorState`. Reads stay strictly sequential through the file —
/// there is NO per-line `seek`; the bytes already read past one line's newline
/// (the leftover in `scratch[cursor..valid_len]`) become the first bytes of the
/// next line.
///
/// # Fields
/// * `valid_len` - Number of valid bytes currently buffered in `scratch[0..valid_len]`.
/// * `cursor`    - Index of the next unconsumed byte within `scratch[0..valid_len]`.
/// * `reached_eof` - Set once the underlying file returns a 0-byte read; no
///   further reads are attempted after this.
struct ChunkReaderState {
    valid_len: usize,
    cursor: usize,
    reached_eof: bool,
}

impl ChunkReaderState {
    /// Creates a fresh reader state positioned at the start of a fresh read run.
    ///
    /// Use one instance per *sequential* read pass:
    /// - `build_windowmap_nowrap` creates ONE and reuses it for every line.
    /// - `goto_line_end` creates one per scan pass (it `seek`s first, so a fresh
    ///   state with empty buffers is required each pass).
    fn new() -> Self {
        ChunkReaderState {
            valid_len: 0,
            cursor: 0,
            reached_eof: false,
        }
    }
}

/// Returns the *declared* byte length (1..=4) of a UTF-8 character from its
/// first byte alone, used only to decide whether a character straddles the
/// current chunk boundary.
///
/// # Purpose (Project Context)
/// `get_utf8_char_byte_length_from_buffer` validates continuation bytes and, by
/// design, returns `Ok(1)` for an *incomplete* multi-byte sequence at a buffer
/// end. In the chunked reader an "incomplete sequence at buffer end" usually
/// means the character is split across a 64-byte chunk boundary and we simply
/// need to read more — NOT that the data is truncated. So before validating, we
/// must know the *declared* length from the first byte to decide "do I have all
/// the bytes, or must I refill first?". This tiny function answers exactly that
/// and nothing else; the authoritative length (with continuation-byte checks)
/// still comes from `get_utf8_char_byte_length_from_buffer` once the bytes are
/// guaranteed present.
///
/// # First-byte patterns
/// ```text
/// 0x00..=0x7F  0xxxxxxx -> 1
/// 0xC0..=0xDF  110xxxxx -> 2
/// 0xE0..=0xEF  1110xxxx -> 3
/// 0xF0..=0xF7  11110xxx -> 4
/// else (continuation 0x80..=0xBF, or 0xF8..=0xFF) -> 1 (defensive)
/// ```
fn utf8_declared_len_from_first_byte(first_byte: u8) -> usize {
    if first_byte <= 0x7F {
        1
    } else if (0xF0..=0xF7).contains(&first_byte) {
        4
    } else if (0xE0..=0xEF).contains(&first_byte) {
        3
    } else if (0xC0..=0xDF).contains(&first_byte) {
        2
    } else {
        1
    }
}

/// Reads the next UTF-8 character (or line terminator) from a file, refilling a
/// small fixed scratch buffer in chunks as needed.
///
/// # Purpose (Project Context)
/// This is THE memory-thrift primitive for line handling. It replaces the old
/// `read_single_line`, which read one byte at a time into a whole-line 4096-byte
/// buffer. Here a line of any length is walked character by character while only
/// `limits::LINE_CHUNK_READ_BYTES` bytes are ever held at once.
///
/// # Sequential reading, no per-line seek
/// The reader consumes the terminating `\n` of a line (reporting `Newline`) but
/// the `scratch`/`rs` pair retains any bytes already read past it. The caller
/// therefore continues the *next* line from those leftover bytes with no `seek`.
/// `build_windowmap_nowrap` relies on this to read the whole window in one
/// forward pass.
///
/// # Chunk-straddling UTF-8 characters
/// A multi-byte character can span a chunk boundary. When the declared length
/// (from the first byte) would run past `valid_len`, the ≤3 leftover bytes are
/// slid to the front of `scratch` and a fresh chunk is read in behind them, then
/// the character is re-evaluated. This requires `scratch.len() >= 4` so a
/// 4-byte character always fits after one refill (debug-asserted; the configured
/// chunk size of 64/4096 satisfies this).
///
/// # Arguments
/// * `file`    - Open file, positioned where reading should continue.
/// * `scratch` - The shared `EditorState::line_chunk_scratch` buffer.
/// * `rs`      - Borrow-free reader position state (carried across calls).
///
/// # Returns
/// * `Ok(LineCharStep::Char { .. })` - One complete (or EOF-truncated) character.
/// * `Ok(LineCharStep::Newline)`     - Line terminated by `\n` (consumed).
/// * `Ok(LineCharStep::Eof)`         - No more bytes before any newline.
/// * `Err(LinesError)`               - Underlying read error (handled by callers).
///
/// # Defensive Programming
/// - Inner refill loop bounded by `limits::MAX_CHUNKS` (Power-of-Ten rule 2).
/// - Invalid / truncated UTF-8 falls back to single-byte consumption (matches
///   the rest of the renderer's tolerance of malformed bytes).
/// - No heap, no recursion, no unsafe.
fn next_line_char(
    file: &mut File,
    scratch: &mut [u8; limits::LINE_CHUNK_READ_BYTES],
    rs: &mut ChunkReaderState,
) -> Result<LineCharStep> {
    // The straddle handling slides ≤3 leftover bytes and refills behind them,
    // so a 4-byte character can only be guaranteed to fit if the chunk is >= 4.
    #[cfg(all(debug_assertions, not(test)))]
    debug_assert!(
        scratch.len() >= 4,
        "LINE_CHUNK_READ_BYTES must be >= 4 to hold a 4-byte UTF-8 char across a refill"
    );

    let mut refill_iterations: usize = 0;

    while refill_iterations < limits::MAX_CHUNKS {
        refill_iterations += 1;

        // ─── Ensure at least one buffered byte, else determine EOF ──────────
        if rs.cursor >= rs.valid_len {
            rs.cursor = 0;
            rs.valid_len = 0;

            if rs.reached_eof {
                return Ok(LineCharStep::Eof);
            }

            let n = file.read(&mut scratch[..]).map_err(LinesError::Io)?;
            if n == 0 {
                rs.reached_eof = true;
                return Ok(LineCharStep::Eof);
            }
            rs.valid_len = n;
        }

        let first_byte = scratch[rs.cursor];

        // ─── Line terminator: consume the '\n', report Newline ──────────────
        if first_byte == b'\n' {
            rs.cursor += 1;
            return Ok(LineCharStep::Newline);
        }

        // ─── Decide whether the full character is present in the buffer ─────
        let declared_len = utf8_declared_len_from_first_byte(first_byte);

        if rs.cursor + declared_len > rs.valid_len {
            // Character's declared extent runs past what we have buffered.
            if !rs.reached_eof {
                // Slide leftover bytes to the front and read a fresh chunk in
                // behind them, then re-evaluate this character.
                let leftover = rs.valid_len - rs.cursor;
                scratch.copy_within(rs.cursor..rs.valid_len, 0);
                rs.cursor = 0;
                rs.valid_len = leftover;

                let n = file
                    .read(&mut scratch[leftover..])
                    .map_err(LinesError::Io)?;
                if n == 0 {
                    rs.reached_eof = true;
                } else {
                    rs.valid_len += n;
                }
                continue; // retry with more bytes available
            }
            // EOF with a partial multi-byte sequence: fall through and let the
            // centralized helper return its defensive length (1) for the
            // truncated bytes that remain.
        }

        // ─── Validate and copy the character (authoritative length) ─────────
        // `get_utf8_char_byte_length_from_buffer` checks continuation bytes and
        // returns 1 for malformed/truncated input — matching the renderer's
        // tolerance. We always advance by the length it reports.
        let char_len =
            match get_utf8_char_byte_length_from_buffer(&scratch[..rs.valid_len], rs.cursor) {
                Ok(len) => len,
                Err(_) => 1,
            };

        let mut bytes = [0u8; 4];
        let mut i = 0;
        while i < char_len && rs.cursor + i < rs.valid_len {
            bytes[i] = scratch[rs.cursor + i];
            i += 1;
        }
        rs.cursor += char_len;

        return Ok(LineCharStep::Char {
            bytes,
            len: char_len,
        });
    }

    // Refill ceiling hit (should be unreachable in practice).
    Err(LinesError::Io(io::Error::new(
        io::ErrorKind::Other,
        "Maximum chunk refills exceeded in next_line_char",
    )))
}

/// Returns the VISUAL terminal-cell width (1 or 2) of one UTF-8 character.
///
/// # Purpose (Project Context)
/// Under the project's "Option A" coordinate model, `cursor.tui_visual_col` and
/// `effective_cols` are in terminal CELLS. CJK / emoji characters are one
/// character but two cells. Both the window builder and `goto_line_end` need
/// this width; centralizing it keeps the two paths consistent.
///
/// # Defensive
/// Malformed bytes fall back to width 1 (matches renderer tolerance).
fn visual_width_of_char(char_bytes: &[u8]) -> usize {
    if char_bytes.len() == 1 {
        return 1; // ASCII is always single-width
    }
    match std::str::from_utf8(char_bytes) {
        Ok(s) => match s.chars().next() {
            Some(ch) => {
                if double_width::is_double_width(ch) {
                    2
                } else {
                    1
                }
            }
            None => 1,
        },
        Err(_) => 1,
    }
}

/// Builds the window-to-file mapping for NoWrap mode (chunked, memory-thrifty).
///
/// # Purpose
/// Reads file content and populates display buffers with line numbers and the
/// visible portion of each file line, while recording each display row's
/// file-byte range for cursor math.
///
/// # Memory model (why this version exists)
/// The previous version read each whole line into a 4096-byte buffer (one byte
/// at a time) via `read_single_line`, then handed the full line to
/// `process_line_with_offset`. This version holds at most
/// `limits::LINE_CHUNK_READ_BYTES` bytes at any time: it walks each line one
/// UTF-8 character at a time via `next_line_char`, doing horizontal-offset skip,
/// visible write, and window-map update in a single forward pass. The old
/// `process_line_with_offset` and `read_single_line` are removed; their logic
/// lives inline here.
///
/// # Sequential reading
/// A single `ChunkReaderState` is reused for the whole window. `next_line_char`
/// consumes each line's `\n` but retains bytes already read past it, so the next
/// line continues from there — there is NO per-line `seek`. `file_byte_position`
/// is still tracked explicitly (line content length + 1 for the newline) and is
/// what `set_line_byte_range` records.
///
/// # Long-line scalability (behavior improvement)
/// The old code silently split any line longer than 4096 bytes (the read buffer
/// filled and `read_single_line` returned `found_newline = false`). This version
/// drains the remainder of a too-wide line in chunks to find the real newline,
/// so arbitrarily long lines map to exactly one display row, as intended.
///
/// # NoWrap behavior (unchanged)
/// - One file line per display row; over-wide lines truncate at the display edge.
/// - Horizontal scroll via `tui_window_horizontal_utf8txt_line_char_offset`
///   (skips whole CHARACTERS).
/// - `display_col += 1` per displayed character (visual width still gates the
///   right edge) — preserved exactly from the prior implementation.
/// - Newline shown as `␤` when the full line fits and a cell remains.
///
/// # Arguments / Returns / Coordinate Spaces
/// Unchanged from the prior version (signature identical).
///
/// # Defensive Programming
/// - Outer row loop bounded by `limits::WINDOW_BUILD_LINES`.
/// - Per-line character loop bounded by `limits::MAX_CHUNKS`.
/// - Skip/write phases additionally bounded by `limits::HORIZONTAL_SCROLL_CHARS`.
/// - All buffer writes bounds-checked against `MAX_DISPLAY_BUFFER_BYTES`.
pub fn build_windowmap_nowrap(state: &mut EditorState, readcopy_file_path: &Path) -> Result<usize> {
    // ─── Validate inputs ────────────────────────────────────────────────────
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

    #[cfg(debug_assertions)]
    {
        debug_assert!(state.effective_rows > 0, "Effective rows must be positive");
        debug_assert!(state.effective_cols > 0, "Effective cols must be positive");
    }

    // ─── Reset display + mapping state ──────────────────────────────────────
    state.clear_utf8_displaybuffers();
    state.clear_line_byte_ranges();
    state.eof_fileline_tuirow_tuple = None;

    // ─── Open and seek to the top line of the window ────────────────────────
    let mut file = File::open(readcopy_file_path)?;
    let byte_position = seek_to_line_number(&mut file, state.line_count_at_top_of_window)?;
    state.file_position_of_topline_start = byte_position;

    // ─── Sequential chunk reader: ONE state for the whole window ────────────
    let mut rs = ChunkReaderState::new();

    let mut current_display_row = 0usize;
    let mut current_file_line_number = state.line_count_at_top_of_window;
    let mut lines_processed = 0usize;
    let mut file_byte_position = state.file_position_of_topline_start;

    let mut row_iteration_count = 0usize;

    // ─── Row loop ───────────────────────────────────────────────────────────
    while current_display_row < state.effective_rows
        && row_iteration_count < limits::WINDOW_BUILD_LINES
    {
        #[cfg(debug_assertions)]
        debug_assert!(
            current_display_row <= MAX_TUI_ROWS,
            "Display row exceeds maximum"
        );

        row_iteration_count += 1;

        let line_start_byte = file_byte_position;

        // ── Per-line layout state (was the body of process_line_with_offset) ─
        let horizontal_offset = state.tui_window_horizontal_utf8txt_line_char_offset;

        // Write the line number first (no reader borrow is held here).
        let fileline_number_for_display = current_file_line_number + 1; // 0-idx -> 1-idx
        let line_num_bytes_written = state.write_line_number(
            current_display_row,
            fileline_number_for_display,
            state.line_count_at_top_of_window,
        )?;

        let col_start = line_num_bytes_written;
        let remaining_cols = state.effective_cols.saturating_sub(line_num_bytes_written);
        let visual_col_limit = col_start + remaining_cols;
        let display_col_limit = col_start + remaining_cols;

        // Running per-line counters.
        let mut line_content_bytes: u64 = 0; // total content bytes (excludes '\n')
        let mut found_newline = false;

        let mut chars_skipped = 0usize;
        let mut skip_iterations = 0usize;

        let mut bytes_written = 0usize;
        let mut display_col = col_start;
        let mut visual_col = col_start;
        let mut display_truncated = false; // visible region ran out before line end
        let mut write_iterations = 0usize;

        let mut char_loop_count = 0usize;

        // ── Character loop: skip + write + drain to newline/EOF, one pass ────
        loop {
            if char_loop_count >= limits::MAX_CHUNKS {
                return Err(LinesError::Io(io::Error::new(
                    io::ErrorKind::Other,
                    "Maximum characters exceeded in build_windowmap_nowrap line",
                )));
            }
            char_loop_count += 1;

            let step = next_line_char(&mut file, &mut state.line_chunk_scratch, &mut rs)?;

            let (char_bytes, char_len) = match step {
                LineCharStep::Newline => {
                    found_newline = true;
                    break;
                }
                LineCharStep::Eof => {
                    break; // found_newline stays false
                }
                LineCharStep::Char { bytes, len } => (bytes, len),
            };

            // This character is part of the line content regardless of whether
            // it is skipped, displayed, or beyond the display edge.
            line_content_bytes += char_len as u64;

            // ── Phase 1: horizontal-offset skip (whole characters) ───────────
            if chars_skipped < horizontal_offset {
                if skip_iterations >= limits::HORIZONTAL_SCROLL_CHARS {
                    return Err(LinesError::Io(io::Error::new(
                        io::ErrorKind::Other,
                        "Maximum iterations exceeded in horizontal skip",
                    )));
                }
                skip_iterations += 1;
                chars_skipped += 1;
                continue; // skipped: not written, but already counted as content
            }

            // ── Phase 2: write visible characters (until the edge is reached) ─
            if display_truncated {
                // Display edge already reached; keep draining to find newline,
                // but do not write or map further characters.
                continue;
            }

            if write_iterations >= limits::HORIZONTAL_SCROLL_CHARS {
                return Err(LinesError::Io(io::Error::new(
                    io::ErrorKind::Other,
                    "Maximum iterations exceeded in line write",
                )));
            }
            write_iterations += 1;

            let display_width = visual_width_of_char(&char_bytes[..char_len]);

            // Would this character overflow the visible region (visually or by
            // display column)? If so, stop writing (mark truncated).
            if visual_col + display_width > visual_col_limit
                || display_col + display_width > display_col_limit
            {
                display_truncated = true;
                continue;
            }

            // Copy the character bytes into the display buffer (bounds-checked).
            let write_start = col_start + bytes_written;
            let write_end = write_start + char_len;
            if write_end > MAX_DISPLAY_BUFFER_BYTES {
                display_truncated = true; // buffer full
                continue;
            }

            let mut i = 0;
            while i < char_len {
                state.utf8_txt_display_buffers[current_display_row][write_start + i] =
                    char_bytes[i];
                i += 1;
            }

            bytes_written += char_len;
            // Preserved behavior: one cursor stop per displayed character.
            // (Visual width still gates the right-edge checks above.)
            display_col += 1;
            visual_col += display_width;
        }

        // ── EOF with nothing read: record EOF marker and stop (unchanged) ────
        if line_content_bytes == 0 && !found_newline {
            if lines_processed > 0 {
                let last_valid_file_line = current_file_line_number.saturating_sub(1);
                let last_valid_display_row = current_display_row.saturating_sub(1);
                state.eof_fileline_tuirow_tuple =
                    Some((last_valid_file_line, last_valid_display_row));
            } else {
                state.eof_fileline_tuirow_tuple =
                    Some((current_file_line_number, current_display_row));
            }
            break;
        }

        // ── Newline glyph: only when the full line fit with room to spare ────
        // (Old guard `byte_index >= line_bytes.len()` is equivalent to
        // "not truncated": if the line was clipped we never show the glyph.)
        //
        // Note: `display_col` is intentionally NOT advanced after writing the
        // glyph. In the old `process_line_with_offset` the post-glyph increment
        // fed a later EOL-mapping read; that block produced only a debug log and
        // changed no state, so it was dropped here. `display_col` is dead after
        // this point (re-initialized to `col_start` next row), and the row's
        // recorded length uses `bytes_written`, not `display_col`.
        if found_newline && !display_truncated && display_col < display_col_limit {
            let newline_char = '␤';
            let newline_str = newline_char.to_string();
            let newline_bytes = newline_str.as_bytes();
            let newline_byte_len = newline_bytes.len();

            let write_start = col_start + bytes_written;
            let write_end = write_start + newline_byte_len;
            if write_end <= MAX_DISPLAY_BUFFER_BYTES {
                let mut i = 0;
                while i < newline_byte_len {
                    state.utf8_txt_display_buffers[current_display_row][write_start + i] =
                        newline_bytes[i];
                    i += 1;
                }
                bytes_written += newline_byte_len;
            }
        }

        // ── Record total bytes used in this display row ──────────────────────
        state.display_utf8txt_buffer_lengths[current_display_row] =
            line_num_bytes_written + bytes_written;

        // ── Line byte-range tracking (start == end signals an empty line) ────
        let line_end_byte = if line_content_bytes > 0 {
            line_start_byte + line_content_bytes - 1
        } else {
            line_start_byte
        };
        state.set_line_byte_range(current_display_row, line_start_byte, line_end_byte)?;

        // ── Advance to next line ─────────────────────────────────────────────
        current_display_row += 1;
        current_file_line_number += 1;
        lines_processed += 1;

        file_byte_position += line_content_bytes;
        if found_newline {
            file_byte_position += 1; // account for the consumed '\n'
        }
    }

    if row_iteration_count >= limits::WINDOW_BUILD_LINES {
        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::Other,
            "Maximum iterations exceeded in build_windowmap_nowrap",
        )));
    }

    #[cfg(debug_assertions)]
    debug_assert!(
        lines_processed <= state.effective_rows,
        "Processed more lines than display rows available"
    );

    if row_iteration_count >= limits::WINDOW_BUILD_LINES {
        return Err(LinesError::Io(io::Error::new(
            io::ErrorKind::Other,
            "Maximum iterations exceeded in build_windowmap_nowrap",
        )));
    }

    #[cfg(debug_assertions)]
    debug_assert!(
        lines_processed <= state.effective_rows,
        "Processed more lines than display rows available"
    );

    // Validates that the processed line count does not exceed available display
    // rows. This catch ensures the windowmap construction stayed within bounds
    // and surfaces an impossible state (corruption, off-by-one, etc.) without
    // panicking in production.
    if lines_processed > state.effective_rows {
        return Err(LinesError::LineCountExceeded {
            lines_processed,
            available_rows: state.effective_rows,
        });
    }

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

    // =================================================
    // Debug-Assert, Test-Asset, Production-Catch-Handle
    // =================================================
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
        if chunk_count >= limits::MAX_CHUNKS {
            #[cfg(debug_assertions)]
            eprintln!(
                "DEBUG: Maximum chunk limit reached ({})",
                limits::MAX_CHUNKS
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

// ============================================================================
// UTF-8 CHARACTER ANALYSIS (Buffer-based variant for line processing)
// ============================================================================

/// Determines the byte length of a UTF-8 character from a byte buffer
///
/// # Purpose (Project Context)
/// When building the window-to-file mapping, we process lines that have been
/// read into memory buffers. This function analyzes UTF-8 characters from
/// those buffers to calculate byte positions and display widths.
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
    EnterInsertMode,       // i
    EnterVisualSelectMode, // v
    EnterNormalMode,       // n or Esc or ??? -> Ctrl-[

    EnterPastyClipboardMode, // pasty: clipboard et al
    EnterHexEditMode,        // Hex Edith

    /// Enter keystroke-input mode (the `ki` command).
    ///
    /// # Project Context
    /// Switches the editor into `EditorMode::KeystrokeInputMode`, where input is
    /// read byte-by-byte from a Linux termios raw terminal. This is the only
    /// command that leads to raw-terminal input in the editor.
    ///
    /// # Why a Separate Command from `EnterInsertMode`
    /// Insert mode uses cooked/canonical StdinLock (Enter-terminated lines).
    /// Keystroke-input mode uses a transient `RawTerminal`. They are different
    /// input pipelines, so they get different mode variants and different
    /// commands. The `ki` command is distinct from the `i` command on purpose.
    EnterKeystrokeInputMode,

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
            // Vim-like behavior: move the cursor left one character at a time;
            // scroll the window or wrap to the previous line at the edges.
            //
            // # Coordinate Spaces (see the module "Coordinate Spaces" reference)
            // Edge math is in #5 VISUAL cells: `cursor.tui_visual_col` retreats by
            // the crossed character's #5 visual width (1 or 2 cells). The line's
            // first content cell is at `line_num_width`; cells
            // [0, line_num_width) are the line-number prefix. Horizontal scroll
            // adjusts `tui_window_horizontal_utf8txt_line_char_offset` (#4, in-line
            // characters). The resolved file byte (#1) is always derived via
            // get_row_col_file_position; this command stores no parallel counter.

            let mut remaining_moves = count;

            // Defensive: Limit iterations to prevent infinite loops
            let mut iterations = 0;

            // iterate through the number of steps the user requested
            while remaining_moves > 0 && iterations < limits::CURSOR_MOVEMENT_STEPS {
                iterations += 1;

                // Line-number prefix width in #5 VISUAL cells for THIS row.
                // (ASCII prefix, so cells == characters.) Used by both the
                // defensive recovery guard and the movement cases below. Computed
                // once per iteration because a wrap (Case 3) can change the row.
                let line_num_width = calculate_line_number_width(
                    lines_editor_state.line_count_at_top_of_window,
                    lines_editor_state.cursor.tui_row,
                    lines_editor_state.effective_rows,
                );

                // ─────────────────────────────────────────────────────────────
                // DEFENSIVE RECOVERY — cursor drifted INTO the line-number prefix
                //
                // COORDINATE SPACES IN PLAY (see the module "Coordinate Spaces"
                // reference; confusing these is the bug class this guard belongs
                // to):
                //   • cursor.tui_visual_col — #5 VISUAL cell column of the cursor.
                //       Counts terminal CELLS from the row's left edge, INCLUDING
                //       the line-number prefix; double-width chars (CJK, emoji)
                //       are 2 cells. SOURCE OF TRUTH for horizontal position.
                //   • line_num_width — width, in #5 CELLS, of the line-number
                //       prefix (e.g. "166 " = 4 cells; ASCII, so cells == chars).
                //       The prefix occupies visual columns [0, line_num_width);
                //       the first CONTENT cell is at column line_num_width.
                //   • horizontal scroll (#4) — deliberately NOT consulted here:
                //       the prefix boundary is independent of horizontal scroll.
                //
                // WHAT THIS DOES AND WHY:
                //   The text cursor must never sit INSIDE the prefix (visual
                //   columns [0, line_num_width)). The movement cases below never
                //   produce such a position, so this guard only catches an
                //   out-of-frame glitch present on ENTRY. If tui_visual_col is
                //   STRICTLY LESS than the prefix width, abandon the leftward step
                //   and jump to a known-good position (line start) via
                //   GotoLineStart, then return.
                //
                //   THE STRICT "<" IS INTENTIONAL: at exactly the content-left
                //   edge (tui_visual_col == line_num_width) the cursor is on the
                //   first content cell — a LEGAL position — so this guard must NOT
                //   fire there; Case 2 / Case 3 below own that edge.
                // ─────────────────────────────────────────────────────────────
                if lines_editor_state.cursor.tui_visual_col < line_num_width {
                    execute_command(lines_editor_state, Command::GotoLineStart)?;
                    return Ok(true);
                }

                // position state inspection (debug builds only): prints the four
                // sources of truth + derived values, each tagged by coordinate
                // space (see debug_inspect_position).
                #[cfg(debug_assertions)]
                lines_editor_state.debug_inspect_position("execute_command() Command::MoveLeft");

                // ─────────────────────────────────────────────────────────────
                // MOVE LEFT — cross ONE character (Option A: by #5 VISUAL width)
                //
                // COORDINATE SPACES IN PLAY (see the module "Coordinate Spaces"
                // reference):
                //   • cursor.tui_visual_col (#5 VISUAL cell column) — source of
                //       truth for horizontal position. First content cell is at
                //       `line_num_width`.
                //   • left_width (#5 VISUAL cells) — width (1 or 2) of the
                //       character immediately to the LEFT of the cursor, derived
                //       from the file by char_to_left_visual_width().
                //   • tui_window_horizontal_utf8txt_line_char_offset (#4 in-line
                //       CHARACTER index) — the horizontal scroll; decreasing it by
                //       one reveals one more character on the left.
                //
                // Three cases, evaluated in order:
                //   Case 1 — the character to the left is fully within the visible
                //     content (its start cell >= line_num_width, i.e.
                //     tui_visual_col >= line_num_width + left_width): cross it by
                //     retreating tui_visual_col by its VISUAL width. ONE character
                //     per step (a kanji retreats by 2 cells, not 1). No rebuild:
                //     no scroll and no content change.
                //   Case 2 — the cursor is at the content-left edge but the line
                //     is horizontally scrolled (offset > 0): scroll left by ONE
                //     character. The cursor stays pinned at the content edge
                //     (column line_num_width), which ALWAYS maps to the new
                //     leftmost character — so left scrolling has NO frameshift
                //     problem (unlike right). Rebuild IMMEDIATELY so the next
                //     iteration's lookups read a fresh windowmap cache.
                //   Case 3 — the cursor is at the absolute line start (content
                //     edge AND offset == 0): wrap to the END of the previous line
                //     if one exists (MoveUp + GotoLineEnd, which rebuilds itself);
                //     otherwise stop (start of file).
                // ─────────────────────────────────────────────────────────────
                let left_width = lines_editor_state.char_to_left_visual_width()?;

                if lines_editor_state.cursor.tui_visual_col >= line_num_width + left_width {
                    // Case 1: cross the in-view character to the left.
                    lines_editor_state.cursor.tui_visual_col -= left_width;
                    remaining_moves -= 1;

                    // // extra inspection
                    // #[cfg(debug_assertions)]
                    // println!(
                    //     "MoveLeft cross: left_width={}, new tui_visual_col={}",
                    //     left_width, lines_editor_state.cursor.tui_visual_col
                    // );
                } else if lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset > 0 {
                    // Case 2: at content-left edge, scroll left by one character.
                    lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset -= 1;
                    // Pin the cursor at the content edge (normally already there);
                    // the newly revealed leftmost character snaps to this cell.
                    lines_editor_state.cursor.tui_visual_col = line_num_width;
                    remaining_moves -= 1;

                    // Rebuild NOW so char_to_left_visual_width / the guard read a
                    // windowmap cache that reflects the new horizontal offset.
                    build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
                } else {
                    // Case 3: absolute line start (content edge, no scroll).
                    // Wrap to the end of the previous line, if any.
                    let current_file_line = lines_editor_state.line_count_at_top_of_window
                        + lines_editor_state.cursor.tui_row;
                    if current_file_line > 0 {
                        // MoveUp scrolls/rebuilds as needed; GotoLineEnd positions
                        // at the previous line's end and rebuilds the window.
                        execute_command(lines_editor_state, Command::MoveUp(1))?;
                        execute_command(lines_editor_state, Command::GotoLineEnd)?;
                        remaining_moves -= 1;
                    } else {
                        // At the very start of the file: cannot move further left.
                        break;
                    }
                }
            }

            // Defensive: Check iteration limit
            if iterations >= limits::CURSOR_MOVEMENT_STEPS {
                return Err(LinesError::Io(io::Error::new(
                    io::ErrorKind::Other,
                    "Maximum iterations exceeded in MoveLeft",
                )));
            }

            Ok(true)
        }

        // =============
        // Move Right v8
        // =============
        Command::MoveRight(count) => {
            // Move the cursor right one character at a time (Vim-like), scrolling
            // the line or jumping to the next line at the edges.
            //
            // # Coordinate Spaces (see the module "Coordinate Spaces" reference)
            // Horizontal position is `cursor.tui_visual_col` (#5 VISUAL cell
            // column). Each step crosses ONE character, advancing tui_visual_col
            // by that character's #5 visual width (1 cell normal, 2 for
            // double-width). The right limit is `effective_cols` (#5). Horizontal
            // scroll adjusts `tui_window_horizontal_utf8txt_line_char_offset`
            // (#4, in-line characters). The cursor's file byte (#1) is always
            // derived via get_row_col_file_position — no stored counter.
            //
            // # Three cases per step
            //   1. NEWLINE GLYPH: if the cursor is on the line's newline cell, the
            //      next right jumps to the START of the following line
            //      (GotoLineStart + MoveDown).
            //   2. IN-VIEW ADVANCE: if the crossed character fits before the right
            //      edge (tui_visual_col + width <= right_edge), advance
            //      tui_visual_col by its #5 visual width. No scroll, no rebuild.
            //   3. EDGE SCROLL: otherwise the cursor is riding the right edge.
            //      Scroll the line left by one character (offset += 1) and KEEP
            //      tui_visual_col pinned at the edge. This is free of
            //      "frame-shift" — because both the lookup (#1 derivation) and the
            //      renderer treat tui_visual_col as a VISUAL column, so the pinned
            //      edge cell snaps to whichever character now occupies it. A
            //      double-width character therefore takes TWO edge steps to appear:
            //      one to reach the edge, one to make room — by design, so the
            //      glyph visibly slides into view.

            let mut remaining_moves = count;
            let mut needs_rebuild = false;

            // Defensive: Limit iterations
            let mut iterations = 0;

            // position state inspection (debug builds only): prints the four
            // sources of truth + derived values, each tagged by coordinate space.
            #[cfg(debug_assertions)]
            lines_editor_state.debug_inspect_position("execute_command() Command::MoveRight");

            while remaining_moves > 0 && iterations < limits::CURSOR_MOVEMENT_STEPS {
                iterations += 1;

                // Case 1 — on the newline glyph: jump to the next line's start.
                let cursor_is_on_newline = lines_editor_state.is_current_cursor_on_newline()?;
                if cursor_is_on_newline {
                    execute_command(lines_editor_state, Command::GotoLineStart)?;
                    execute_command(lines_editor_state, Command::MoveDown(1))?;

                    remaining_moves -= 1;
                    needs_rebuild = true;
                    continue;
                }

                // #5 visual width (1 or 2 cells) of the character being crossed.
                let char_width = lines_editor_state.cursor_char_visual_width()?;

                // Reserve one cell at the right so a double-width character cannot
                // overflow the edge. (right_edge is in #5 VISUAL cells.)
                let right_edge = lines_editor_state.effective_cols.saturating_sub(1);

                if lines_editor_state.cursor.tui_visual_col + char_width <= right_edge {
                    // Case 2 — in-view advance by the crossed char's visual width.
                    lines_editor_state.cursor.tui_visual_col += char_width;
                    remaining_moves -= 1;

                    #[cfg(debug_assertions)]
                    println!(
                        "MoveRight advance: char_width={}, new tui_visual_col={}",
                        char_width, lines_editor_state.cursor.tui_visual_col
                    );
                } else {
                    // Case 3 — edge scroll. Scroll the line left one character and
                    // leave tui_visual_col pinned at the edge; the visual lookup /
                    // renderer keep the cursor on the correct character (see the
                    // doc-block above). This is the proven, simple model — NOT a
                    // placeholder.
                    //
                    // NOTE (polish item, not a bug): the horizontal-scroll cap
                    // below reuses limits::CURSOR_MOVEMENT_STEPS, which is a
                    // per-command iteration bound. It happens to bound how far a
                    // line can scroll right; a dedicated limit (or a line-length
                    // bound) would be clearer. Left as-is for now.
                    if lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset
                        < limits::CURSOR_MOVEMENT_STEPS
                    {
                        let max_scroll = limits::CURSOR_MOVEMENT_STEPS
                            - lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset;
                        // For a single right press scroll_amount == 1. For a count
                        // move it scrolls up to that many characters at once; the
                        // edge cell then resolves to the resulting character.
                        let scroll_amount = remaining_moves.min(max_scroll);

                        lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset +=
                            scroll_amount;

                        remaining_moves -= scroll_amount;
                        needs_rebuild = true;
                    } else {
                        // Hit the horizontal-scroll cap — cannot scroll further.
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

            // Rebuild only if we scrolled (Case 3) or jumped lines (Case 1).
            if needs_rebuild {
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

            /*
            # Window-Map System: TUI-Window to File-Bytes-Window

            Lines uses an on-the-fly window-mapping system to sync/build a correlation map between each character on a non-line-wrapping line-by-line display of file lines and each TUI-character's first file-byte.

            ### There are a few pieces of information that anchor this system:

            (Note:  character-spaces were characters and were bytes in ASCII times, but with UTF-8 (which Rust focuses support on, many others 'character encodings also exist) one character may be one or two spaces wide, and may have 1 to 4 bytes. There are many, many, advantages to using ascii for software. There are many, many, costs to having irregular character-byte size and display spaces.

            1. File-line-number at top TUI tui_display_row (both zero-index)
            2. TUI's tui_display_row (zero index)
            3. Horizontal offset.
            4. Number of TUI horizontal character-spaces (window size)
            5. Number of TUI vertical character-spaces (window size)
            6. width of line-number display, or how many character-spaces plus padding the line number takes up in the TUI, e.g. ' 99 ' takes up four single-width characters (not needed for simple functionality)
            */

            // =========================
            // position state inspection
            // =========================

            #[cfg(debug_assertions)]
            lines_editor_state.debug_inspect_position("execute_command() Command::MoveDown");

            let mut remaining_moves = count;
            let mut needs_rebuild = false;

            // Defensive: Limit iterations
            let mut iterations = 0;

            while remaining_moves > 0 && iterations < limits::CURSOR_MOVEMENT_STEPS {
                iterations += 1;

                // Calculate space available before bottom edge
                let bottom_edge = lines_editor_state.effective_rows.saturating_sub(1);

                if lines_editor_state.cursor.tui_row < bottom_edge {
                    // Cursor can move down within visible window
                    // Check if EOF limits movement
                    let space_available = if let Some((_eof_line, eof_row)) =
                        lines_editor_state.eof_fileline_tuirow_tuple
                    {
                        if lines_editor_state.cursor.tui_row < eof_row {
                            // Can move toward EOF
                            (eof_row - lines_editor_state.cursor.tui_row)
                                .min(bottom_edge - lines_editor_state.cursor.tui_row)
                        } else {
                            // At or past EOF, cannot move
                            0
                        }
                    } else {
                        // No EOF visible, normal movement
                        bottom_edge - lines_editor_state.cursor.tui_row
                    };

                    if space_available == 0 {
                        break;
                    }

                    let cursor_moves = remaining_moves.min(space_available);
                    lines_editor_state.cursor.tui_row += cursor_moves;

                    let line_num_width = calculate_line_number_width(
                        lines_editor_state.line_count_at_top_of_window, // starting_row
                        lines_editor_state.cursor.tui_row,              // tui_row
                        lines_editor_state.effective_rows,              // effective_rows
                    );

                    // if col is in the number-zone to the left of the text
                    // bump it over
                    if lines_editor_state.cursor.tui_visual_col < line_num_width {
                        lines_editor_state.cursor.tui_visual_col = line_num_width; // Skip over line number displayfull_lines_editor
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

                    // Scroll Down
                    /*
                    line_count_at_top_of_window is the core of scroll down
                    and scroll up:
                    To scroll down one line we increment (+1) line_count_at_top_of_window
                    so that when the window (re)builds, it does so one line below
                    the past window-scroll frame.
                     */
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

                        if lines_editor_state.cursor.tui_row > eof_tui_display_row {
                            // Cursor past EOF, clamp to EOF position
                            lines_editor_state.cursor.tui_row = eof_tui_display_row;
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
            # Window-Map System: TUI-Window to File-Bytes-Window

            Lines uses an on-the-fly window-mapping system to sync/build a correlation map between each character on a non-line-wrapping line-by-line display of file lines and each TUI-character's first file-byte.

            ### There are a few pieces of information that anchor this system:

            (Note:  character-spaces were characters and were bytes in ASCII times, but with UTF-8 (which Rust focuses support on, many others 'character encodings also exist) one character may be one or two spaces wide, and may have 1 to 4 bytes. There are many, many, advantages to using ascii for software. There are many, many, costs to having irregular character-byte size and display spaces.

            1. File-line-number at top TUI tui_display_row (both zero-index)
            2. TUI's tui_display_row (zero index)
            3. Horizontal offset.
            4. Number of TUI horizontal character-spaces (window size)
            5. Number of TUI vertical character-spaces (window size)
            6. width of line-number display, or how many character-spaces
            plus padding the line number takes up in the TUI,
            e.g. ' 99 ' takes up four single-width characters
            (not needed for simple functionality)
            */

            // =========================
            // position state inspection
            // =========================

            #[cfg(debug_assertions)]
            lines_editor_state.debug_inspect_position("execute_command() Command::MoveUp");

            let mut remaining_moves = count;
            let mut needs_rebuild = false;

            // Defensive: Limit iterations
            let mut iterations = 0;

            while remaining_moves > 0 && iterations < limits::CURSOR_MOVEMENT_STEPS {
                iterations += 1;

                if lines_editor_state.cursor.tui_row > 0 {
                    // Cursor can move up within visible window
                    let cursor_moves = remaining_moves.min(lines_editor_state.cursor.tui_row);
                    lines_editor_state.cursor.tui_row -= cursor_moves;
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
                lines_editor_state.cursor.tui_row,
                lines_editor_state.effective_rows,
            );

            // if position.. is <
            // ─────────────────────────────────────────────────────────────────
            // POST-MOVE COLUMN SNAP — keep the cursor on text after moving up
            //
            // COORDINATE SPACES IN PLAY (see the project "Coordinate Spaces"
            // reference; this editor juggles several distinct location types and
            // they are NOT interchangeable):
            //   • cursor.tui_visual_col — VISUAL cell column of the cursor within
            //       the display row. It counts terminal CELLS from the row's left
            //       edge and INCLUDES the line-number prefix. ASCII/normal chars
            //       occupy 1 cell; double-width characters (CJK, emoji) occupy 2
            //       cells. Under the project's "Option A" decision this is the
            //       SOURCE OF TRUTH for horizontal cursor position — not a derived
            //       or parallel counter.
            //   • line_num_width — the width, in CELLS, of the line-number prefix
            //       (e.g. "166 " = 4 cells). The prefix is ASCII digits + a space,
            //       so its cell width equals its character width. Consequently the
            //       prefix occupies visual columns [0, line_num_width), and the
            //       line's FIRST CONTENT cell is at visual column line_num_width.
            //   • tui_window_horizontal_utf8txt_line_char_offset — the horizontal
            //       scroll, measured in in-line CHARACTERS (a character index),
            //       NOT in cells. It is reset to 0 here so the snapped line-start
            //       is shown unscrolled.
            //
            // WHAT THIS DOES AND WHY:
            //   MoveUp changes cursor.tui_row but leaves tui_visual_col unchanged.
            //   If, just before moving up, the cursor was at or to the LEFT of the
            //   first content cell (tui_visual_col <= line_num_width — i.e. inside
            //   the line-number prefix, or exactly at line start), re-pin it to the
            //   first content cell so the text cursor never lands "inside the line
            //   number". The condition reads the visual column directly.
            // ─────────────────────────────────────────────────────────────────
            if lines_editor_state.cursor.tui_visual_col <= line_num_width {
                lines_editor_state.cursor.tui_visual_col = line_num_width;
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
                        lines_editor_state.cursor.tui_row,
                        lines_editor_state.cursor.tui_visual_col,
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
                        lines_editor_state.cursor.tui_row,
                        lines_editor_state.cursor.tui_visual_col,
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
                        lines_editor_state.cursor.tui_row,
                        lines_editor_state.cursor.tui_visual_col,
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

            #[cfg(debug_assertions)]
            lines_editor_state.debug_inspect_position("execute_command() Command::GotoLine");

            // Seek to target line and update window position
            match seek_to_line_number(&mut File::open(&base_edit_filepath)?, target_line) {
                Ok(byte_pos) => {
                    lines_editor_state.line_count_at_top_of_window = target_line;
                    lines_editor_state.file_position_of_topline_start = byte_pos;
                    lines_editor_state.cursor.tui_row = 0;
                    lines_editor_state.cursor.tui_visual_col = 0;

                    // Position cursor AFTER line number (same as bootstrap)
                    // number of digits in line number + 1 is first character
                    let line_num_width = calculate_line_number_width(
                        lines_editor_state.line_count_at_top_of_window,
                        line_number,
                        lines_editor_state.effective_rows,
                    );
                    lines_editor_state.cursor.tui_visual_col = line_num_width; // Skip over line number displayfull_lines_editor
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

            #[cfg(debug_assertions)]
            lines_editor_state.debug_inspect_position("execute_command() Command::GotoFileStart");

            // Seek to target line and update window position
            match seek_to_line_number(&mut File::open(&base_edit_filepath)?, target_line) {
                Ok(byte_pos) => {
                    lines_editor_state.line_count_at_top_of_window = target_line;
                    lines_editor_state.file_position_of_topline_start = byte_pos;
                    lines_editor_state.cursor.tui_row = 0;
                    lines_editor_state.cursor.tui_visual_col = 3; // Skip over line number displayfull_lines_editor + padding

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
                lines_editor_state.cursor.tui_row,
                lines_editor_state.effective_rows,
            );
            lines_editor_state.cursor.tui_visual_col = line_num_width;
            lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset = 0;

            // rebuild
            _ = build_windowmap_nowrap(lines_editor_state, &base_edit_filepath);

            let _ = lines_editor_state.set_info_bar_message("start of line");

            // =========================
            // position state inspection
            // =========================
            // reset to first position each new GotoLineStart
            // let line_num_width = calculate_line_number_width(lines_editor_state.cursor.tui_row);

            #[cfg(debug_assertions)]
            lines_editor_state.debug_inspect_position("execute_command() Command::GotoLineStart");

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

                    // Safe Error
                    eprintln!("Error clearing redo logs.");

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
                    eprintln!(
                        "button_safe_clear_all_redo_logs Error clearing redo logs: {:?}",
                        _e
                    );

                    // Log error and continue (non-fatal)
                    log_error(
                        "button_safe_clear_all_redo_logs Cannot clear redo logs",
                        Some("DeleteRange"),
                    );
                    let _ = lines_editor_state.set_info_bar_message("Redo-clear failed");

                    false // Treat error as failure
                }
            };

            // v2: delete selection and reset selection-range to current location
            delete_position_range_noload(lines_editor_state, &edit_file_path)?;

            // Set cursor position to file_position_of_vis_select_start
            // Get current cursor position in FILE
            if let Ok(Some(file_pos)) = lines_editor_state.get_row_col_file_position(
                lines_editor_state.cursor.tui_row,
                lines_editor_state.cursor.tui_visual_col,
            ) {
                // Set/Reset BOTH start and end to same position initially
                lines_editor_state.file_position_of_vis_select_start =
                    file_pos.byte_offset_linear_file_absolute_position;
                lines_editor_state.file_position_of_vis_select_end =
                    file_pos.byte_offset_linear_file_absolute_position;
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
                    log_error("Cannot clear redo logs", Some("Command DeleteBackspace"));
                    // Best-effort user notice. The info-bar message is itself
                    // non-critical: if it fails we do NOT abort the insert (the edit
                    // still proceeds). We observe the failure in debug builds rather
                    // than discarding it via `let _ = ...`.
                    match lines_editor_state.set_info_bar_message("redo clear failed") {
                        Ok(_) => {}
                        Err(_e) => {
                            #[cfg(debug_assertions)]
                            eprintln!(
                                "hskim: set_info_bar_message(redo clear failed) failed: {:?}",
                                _e
                            );
                            // No production log here: this is a notice-about-a-notice;
                            // the redo-clear failure itself was already logged above.
                        }
                    }
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
            /*
            Edge case:
            adding a new-line at the bottom of the TUI
            */
            let _: bool = match button_safe_clear_all_redo_logs(&base_edit_filepath) {
                Ok(success) => success,
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    eprintln!("Error clearing redo logs: {:?}", _e);

                    // Log error and continue (non-fatal)
                    log_error(
                        "Cannot clear redo logs",
                        Some("Command::InsertNewline button_safe_clear_all_redo_logs"),
                    );
                    let _ = lines_editor_state.set_info_bar_message("Redo clear failed");

                    false // Treat error as failure
                }
            };

            insert_newline_at_cursor_chunked(lines_editor_state, edit_file_path)?;

            // insert_newline_at_cursor_chunked advances cursor.tui_row by 1
            // but does NOT scroll the window. If the cursor was on the bottom
            // visible row, tui_row now equals effective_rows (off-screen).
            // We must either:
            //   (a) leave tui_row alone if it's still in range, OR
            //   (b) clamp tui_row to bottom_edge and scroll window down by 1
            // ─────────────────────────────────────────────────────────────────
            let bottom_edge = lines_editor_state.effective_rows.saturating_sub(1);
            if lines_editor_state.cursor.tui_row > bottom_edge {
                // Cursor went off the bottom — scroll window down to reveal new line
                let overflow = lines_editor_state.cursor.tui_row - bottom_edge;
                lines_editor_state.line_count_at_top_of_window += overflow;
                lines_editor_state.cursor.tui_row = bottom_edge;
            }

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
            let _ = lines_editor_state.set_info_bar_message("ESC>exit DEL>bckspc ki>key-ins");
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
            if (lines_editor_state.effective_cols + 1) <= MAX_TUI_VIZ_COLS {
                lines_editor_state.effective_cols += 1;
                build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            }
            Ok(true)
        }

        Command::WideMinus => {
            // Check for handle here: must not be < MIN
            if (lines_editor_state.effective_cols - 1) >= MIN_TUI_VIZ_COLS {
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
            let _ = lines_editor_state.set_info_bar_message("");
            Ok(true)
        }

        Command::EnterVisualSelectMode => {
            // Must rebuild here, or hexedit changes would not appear until
            // after a next change. Keep in Sync.

            // Set cursor position to file_position_of_vis_select_start
            // Get current cursor position in FILE
            if let Ok(Some(file_pos)) = lines_editor_state.get_row_col_file_position(
                lines_editor_state.cursor.tui_row,
                lines_editor_state.cursor.tui_visual_col,
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
            let _ = lines_editor_state.set_info_bar_message("");

            // Set selection start at current cursor position
            if let Ok(Some(file_pos)) = lines_editor_state.get_row_col_file_position(
                lines_editor_state.cursor.tui_row,
                lines_editor_state.cursor.tui_visual_col,
            ) {
                lines_editor_state.selection_start = Some(file_pos);
            }

            // set row of cursor start
            lines_editor_state.selection_rowline_start = lines_editor_state.cursor.tui_row;
            Ok(true)
        }

        Command::EnterKeystrokeInputMode => {
            // Rebuild the windowmap before switching modes, for the same reason
            // EnterInsertMode/EnterHexEditMode do: any pending edits (e.g. from a
            // prior hex edit) must be reflected on screen before we hand control
            // to the keystroke-input session. Without this, stale display could
            // persist until the next edit.
            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;

            lines_editor_state.mode = EditorMode::KeystrokeInputMode;

            // Terse hint, in the same style as EnterInsertMode's hint.
            // Non-critical: if setting the message fails, mode switch still
            // succeeded, so we discard the result.
            let _ = lines_editor_state.set_info_bar_message("ki: Esc>normal  type ascii");

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
                lines_editor_state.cursor.tui_row,
                lines_editor_state.cursor.tui_visual_col,
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
            toggle_basic_singleline_comment_bytewise(
                &edit_file_path.display().to_string(),
                line_number_0number,
            )?;
            build_windowmap_nowrap(lines_editor_state, &edit_file_path)?;
            Ok(true)
        }

        Command::ToggleDocstringOneLine(line_number_0number) => {
            toggle_rust_docstring_singleline_comment_bytewise(
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

            toggle_block_comment_bytewise(
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
            let _ = unindent_range_bytewise(
                &base_edit_filepath.to_string_lossy(),
                lines_editor_state.selection_rowline_start,
                lines_editor_state.cursor.tui_row,
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
            let _ = indent_range_bytewise(
                &base_edit_filepath.to_string_lossy(),
                lines_editor_state.selection_rowline_start,
                lines_editor_state.cursor.tui_row,
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
            let _ = toggle_range_rust_docstring_bytewise(
                &base_edit_filepath.to_string_lossy(),
                lines_editor_state.selection_rowline_start,
                lines_editor_state.cursor.tui_row,
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
            let _ = toggle_range_basic_comments_bytewise(
                &base_edit_filepath.to_string_lossy(),
                lines_editor_state.selection_rowline_start,
                lines_editor_state.cursor.tui_row,
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
            unindent_line_bytewise(&edit_file_path.display().to_string(), line_number)?;
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
            indent_line_bytewise(&edit_file_path.display().to_string(), line_number)?;
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
            let _ = lines_editor_state.set_info_bar_message("Saved");
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

/// Moves the cursor to the end of the current displayed line ("End" key),
/// landing ON the last character, scrolling horizontally if needed.
///
/// # Memory model (why this version exists)
/// The previous version read the whole line into a 4096-byte buffer via
/// `read_single_line`, built a `&str` of the entire line, and iterated its
/// `chars()` three times. This version walks the line one UTF-8 character at a
/// time via `next_line_char`, holding at most `limits::LINE_CHUNK_READ_BYTES`
/// bytes and never materializing the whole line.
///
/// # Two scan passes (instead of one whole-line walk)
/// Pass 1 (`seek` to line start, scan to newline/EOF): sum the line's total
/// VISUAL width and remember the LAST character's visual width.
/// Pass 2 (only when the line is wider than the visible area; re-`seek`, scan):
/// drop leading CHARACTERS from the front until the remaining VISUAL width fits,
/// counting the dropped characters (`skip_chars`, the character-space scroll
/// offset). Two short forward scans replace the old three `chars()` iterations;
/// "End" is a single keypress, so the extra scan is inexpensive.
///
/// Both passes reuse `EditorState::line_chunk_scratch` sequentially (each
/// `next_line_char` call releases the borrow), so there is no aliasing concern
/// with the later `build_windowmap_nowrap` rebuild.
///
/// # Coordinate model (unchanged)
/// CHARACTER space holds the scroll offset (`skip_chars`); VISUAL space holds
/// `cursor.tui_visual_col` and `effective_cols`. The line-number prefix width is
/// computed with `cursor.tui_row` so the round-trip through
/// `get_row_col_file_position` resolves to the intended byte. See the original
/// doc for the full rationale (preserved below in intent).
///
/// # Returns
/// * `Ok(())` - Always. Every fallible step (lookup, open, seek, read, rebuild)
///   is handled: a terse, data-free info-bar message is set, detail is logged
///   only under `#[cfg(debug_assertions)]`, and the function returns `Ok(())` so
///   the editor keeps running. The cursor is never left undefined.
///
/// # Defensive Programming
/// - Each scan loop bounded by `limits::MAX_CHUNKS`.
/// - Malformed UTF-8 tolerated (single-cell width via `visual_width_of_char`).
/// - No heap, no recursion, no unsafe.
fn goto_line_end(lines_editor_state: &mut EditorState, file_path: &Path) -> Result<()> {
    // ── STEP 1: resolve current file position to find the line's start byte ──
    let current_file_pos = match lines_editor_state.get_row_col_file_position(
        lines_editor_state.cursor.tui_row,
        lines_editor_state.cursor.tui_visual_col,
    ) {
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

    let line_start_byte = current_file_pos.byte_offset_linear_file_absolute_position
        - (current_file_pos.byte_in_line as u64);

    // ── STEP 2: open the file ────────────────────────────────────────────────
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

    // Prefix width: uses cursor.tui_row to match get_row_col_file_position so the
    // VISUAL column we set below resolves to the intended byte on round-trip.
    let line_num_width = calculate_line_number_width(
        lines_editor_state.line_count_at_top_of_window,
        lines_editor_state.cursor.tui_row,
        lines_editor_state.effective_rows,
    );

    #[cfg(debug_assertions)]
    lines_editor_state.debug_inspect_position("go_to_line()");

    // ── STEP 3 (pass 1): sum total visual width + last char's visual width ───
    if let Err(_e) = file.seek(SeekFrom::Start(line_start_byte)) {
        let _ = lines_editor_state.set_info_bar_message("cannot seek to line");
        #[cfg(debug_assertions)]
        eprintln!("e: {}", _e);
        log_error("goto_line_end seek error", Some("goto_line_end"));
        return Ok(());
    }

    let mut total_visual_width: usize = 0;
    let mut last_char_visual_width: usize = 1; // empty line default (saturates below)
    {
        let mut rs = ChunkReaderState::new();
        let mut scan_count: usize = 0;
        loop {
            if scan_count >= limits::MAX_CHUNKS {
                let _ = lines_editor_state.set_info_bar_message("line scan too long");
                #[cfg(debug_assertions)]
                log_error("goto_line_end pass1 ceiling", Some("goto_line_end"));
                return Ok(());
            }
            scan_count += 1;

            match next_line_char(
                &mut file,
                &mut lines_editor_state.line_chunk_scratch,
                &mut rs,
            ) {
                Ok(LineCharStep::Newline) | Ok(LineCharStep::Eof) => break,
                Ok(LineCharStep::Char { bytes, len }) => {
                    let w = visual_width_of_char(&bytes[..len]);
                    total_visual_width += w;
                    last_char_visual_width = w;
                }
                Err(_e) => {
                    let _ = lines_editor_state.set_info_bar_message("cannot read line");
                    #[cfg(debug_assertions)]
                    eprintln!("e: {}", _e);
                    #[cfg(debug_assertions)]
                    log_error("goto_line_end read error", Some("goto_line_end"));
                    return Ok(());
                }
            }
        }
    }

    #[cfg(debug_assertions)]
    eprintln!(
        "GOTO_END widths: total_visual_width={} last_char_visual_width={}",
        total_visual_width, last_char_visual_width
    );

    // ── STEP 4: visible content width in cells (one cell reserved for edge) ──
    let visible_content_cells = lines_editor_state
        .effective_cols
        .saturating_sub(line_num_width)
        .saturating_sub(1);

    // ── STEP 5: set VISUAL cursor column, scrolling if the line is too wide ──
    if total_visual_width > visible_content_cells {
        // Pass 2: re-seek and drop leading characters until the remaining
        // visual width fits. The offset stays in CHARACTER units.
        if let Err(_e) = file.seek(SeekFrom::Start(line_start_byte)) {
            let _ = lines_editor_state.set_info_bar_message("cannot seek to line");
            #[cfg(debug_assertions)]
            eprintln!("e: {}", _e);
            log_error("goto_line_end seek error (pass2)", Some("goto_line_end"));
            return Ok(());
        }

        let mut skip_chars: usize = 0;
        let mut remaining_visual_width = total_visual_width;
        {
            let mut rs = ChunkReaderState::new();
            let mut scan_count: usize = 0;
            loop {
                if remaining_visual_width <= visible_content_cells {
                    break;
                }
                if scan_count >= limits::MAX_CHUNKS {
                    let _ = lines_editor_state.set_info_bar_message("line scan too long");
                    #[cfg(debug_assertions)]
                    log_error("goto_line_end pass2 ceiling", Some("goto_line_end"));
                    return Ok(());
                }
                scan_count += 1;

                match next_line_char(
                    &mut file,
                    &mut lines_editor_state.line_chunk_scratch,
                    &mut rs,
                ) {
                    Ok(LineCharStep::Newline) | Ok(LineCharStep::Eof) => break,
                    Ok(LineCharStep::Char { bytes, len }) => {
                        remaining_visual_width = remaining_visual_width
                            .saturating_sub(visual_width_of_char(&bytes[..len]));
                        skip_chars += 1;
                    }
                    Err(_e) => {
                        let _ = lines_editor_state.set_info_bar_message("cannot read line");
                        #[cfg(debug_assertions)]
                        eprintln!("e: {}", _e);
                        #[cfg(debug_assertions)]
                        log_error("goto_line_end read error (pass2)", Some("goto_line_end"));
                        return Ok(());
                    }
                }
            }
        }

        let last_char_visual_start = remaining_visual_width.saturating_sub(last_char_visual_width);

        lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset = skip_chars;
        lines_editor_state.cursor.tui_visual_col = line_num_width + last_char_visual_start;
    } else {
        // Fit branch: no scroll. Cursor at the last char's visual start column.
        let last_char_visual_start = total_visual_width.saturating_sub(last_char_visual_width);

        lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset = 0;
        lines_editor_state.cursor.tui_visual_col = line_num_width + last_char_visual_start;
    }

    // ── STEP 6: rebuild the window so the new offset/column are reflected ────
    // A rebuild failure is logged and handled, never panicked: the cursor state
    // is already updated, so we continue.
    if let Err(_e) = build_windowmap_nowrap(lines_editor_state, file_path) {
        let _ = lines_editor_state.set_info_bar_message("display update failed");
        #[cfg(debug_assertions)]
        eprintln!("e: {}", _e);
        #[cfg(debug_assertions)]
        log_error("goto_line_end rebuild error", Some("goto_line_end"));
        // Continue anyway - cursor was already updated.
    }

    let _ = lines_editor_state.set_info_bar_message("end of line");
    Ok(())
}

/// Identifies which arrow key was pressed, after the raw 3-byte escape
/// sequence has been classified by the session loop.
///
/// # Project Context
///
/// In `EditorMode::KeystrokeInputMode`, arrow keys arrive from a raw terminal
/// as a 3-byte escape sequence (`0x1B 0x5B 0x41..=0x44`), NOT as a single byte
/// like printable ASCII. The session loop (`handle_keystroke_input_session`)
/// reads up to 3 bytes per `read()`, classifies an exact arrow match into one
/// of these variants via `classify_arrow_bytes`, and hands the variant to
/// `handle_arrow_key_input_mode`.
///
/// This enum exists so that the byte-pattern match happens exactly ONCE (in the
/// session loop), and the arrow handler receives an already-classified,
/// type-safe direction rather than re-matching raw bytes. This keeps each
/// function's scope narrow: the session loop classifies; the arrow handler maps
/// direction to a cursor-move `Command`.
///
/// # Byte Sequences (raw terminal, decimal / hex)
///
/// | Variant     | Bytes (hex)         | Bytes (decimal) |
/// |-------------|---------------------|-----------------|
/// | `UpArrow`    | `0x1B 0x5B 0x41`    | `27 91 65`      |
/// | `DownArrow`  | `0x1B 0x5B 0x42`    | `27 91 66`      |
/// | `RightArrow` | `0x1B 0x5B 0x43`    | `27 91 67`      |
/// | `LeftArrow`  | `0x1B 0x5B 0x44`    | `27 91 68`      |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArrowKeyDirection {
    UpArrow,
    DownArrow,
    LeftArrow,
    RightArrow,
}

/// Classifies a freshly-read raw-terminal byte buffer as an arrow key, if and
/// only if it is an EXACT 3-byte arrow escape sequence.
///
/// # Project Context
///
/// Called by `handle_keystroke_input_session` immediately after each `read()`
/// into the 3-byte buffer. This function is the single point where the arrow
/// byte-pattern is matched. It returns:
///   - `Some(direction)` ONLY when the buffer is exactly the 3 bytes of a known
///     arrow sequence.
///   - `None` for everything else, in which case the session loop must dispatch
///     the bytes individually through the single-byte path (so that no byte is
///     dropped — see the session loop's per-byte dispatch).
///
/// # Why `n` (the byte count) Matters
///
/// `read()` returns how many bytes it placed in the buffer. We are
/// passed exactly that filled slice (`&buf[0..n]`). An arrow is recognized ONLY
/// when:
///   - the slice length is exactly 3, AND
///   - the slice equals `[0x1B, 0x5B, 0x41..=0x44]`.
///
/// A length of 3 by itself does NOT mean "arrow": three printable bytes (e.g. a
/// fast-typed or pasted "abc") also produce a length-3 slice. Those do not match
/// the pattern (printable bytes are never `0x1B`), so this returns `None` and
/// they go down the per-byte path. There is therefore no collision between
/// "three printable bytes" and "one arrow key."
///
/// # Fragmentation Limitation (documented, accepted for now)
///
/// On a fast local terminal a single arrow keypress arrives as all 3 bytes in
/// one `read()`. Over slow or remote links the kernel MAY split the sequence
/// across multiple reads (e.g. `0x1B` alone, then `0x5B 0x41`). In that case the
/// first read is a length-1 `0x1B`, which the single-byte path treats as ESC
/// (enter Normal mode), and the trailing bytes are then dispatched individually.
/// Handling fragmented sequences robustly requires an ESC-pending state machine
/// with a read timeout; that is a deliberate future step, not implemented here.
///
/// # Arguments
///
/// * `filled_buffer` - the slice of bytes read this iteration
///   (`&byte_buffer[0..bytes_read]`).
///
/// # Returns
///
/// * `Some(ArrowKeyDirection)` if the slice is an exact arrow sequence.
/// * `None` otherwise.
fn classify_arrow_bytes(filled_buffer: &[u8]) -> Option<ArrowKeyDirection> {
    // An arrow sequence is exactly 3 bytes. Anything else cannot be an arrow.
    if filled_buffer.len() != 3 {
        return None;
    }

    // First two bytes of every arrow sequence are ESC ('0x1B') then '[' (0x5B).
    if filled_buffer[0] != 0x1B || filled_buffer[1] != 0x5B {
        return None;
    }

    // The third byte selects the direction.
    match filled_buffer[2] {
        0x41 => Some(ArrowKeyDirection::UpArrow),
        0x42 => Some(ArrowKeyDirection::DownArrow),
        0x43 => Some(ArrowKeyDirection::RightArrow),
        0x44 => Some(ArrowKeyDirection::LeftArrow),
        // 0x1B 0x5B followed by anything else is some other escape sequence
        // (Home/End/Page/F-keys/etc.) — not an arrow. Caller will dispatch the
        // bytes individually (and the single-byte path ignores the unknowns).
        _ => None,
    }
}

/// Maps a classified arrow-key direction to the corresponding cursor-move
/// command, in `EditorMode::KeystrokeInputMode`.
///
/// # Project Context
///
/// This is the arrow-key counterpart to the single-byte dispatcher. The session
/// loop (`handle_keystroke_input_session`) classifies the raw 3-byte arrow
/// escape sequence into an `ArrowKeyDirection` (via `classify_arrow_bytes`) and
/// calls this function. This function does NOT read input, does NOT own the
/// terminal, and does NOT render — it only maps one direction to one cursor-move
/// `Command`.
///
/// Separation of concerns:
/// - `handle_keystroke_input_session` : owns RawTerminal, reads bytes, renders,
///   classifies arrows vs. single bytes, handles EOF / read-error / mode exit.
/// - `classify_arrow_bytes`           : recognizes the exact 3-byte arrow pattern.
/// - `handle_arrow_key_input_mode`    : maps an `ArrowKeyDirection` to a
///   `Command::Move*` (this function).
/// - the single-byte dispatcher        : maps one non-arrow byte to one action.
///
/// # Direction → Command Mapping
///
/// | Direction    | Command            |
/// |--------------|--------------------|
/// | `UpArrow`    | `Command::MoveUp`   |
/// | `DownArrow`  | `Command::MoveDown` |
/// | `LeftArrow`  | `Command::MoveLeft` |
/// | `RightArrow` | `Command::MoveRight`|
///
/// # Rebuild / Render Policy
///
/// Cursor moves route through `execute_command`, exactly like backspace and
/// newline do. The session loop renders unconditionally at the top of its next
/// iteration, so any cursor/window change made by the move command is painted
/// then. This function therefore does NOT call `build_windowmap_nowrap` itself
/// (matching the backspace/newline policy, NOT the printable-byte exception
/// which bypasses `execute_command`). If testing later shows a cursor move needs
/// an explicit rebuild here, it can be added at that point.
///
/// # Arguments
///
/// * `lines_editor_state` - mutable editor state (cursor, window, buffers, etc.).
/// * `arrow_direction`    - the already-classified arrow direction.
///
/// # Returns
///
/// * `Ok(true)` - editor loop should keep running. Cursor moves never request
///   loop termination, so the propagated `bool` from `execute_command` is the
///   running flag (currently always `true` for `Move*` commands; we forward
///   whatever `execute_command` returns rather than hard-coding `true`, so this
///   stays honest if a move command's contract ever changes).
/// * `Err(LinesError)` - propagated from `execute_command` on an
///   unrecoverable failure; the session restores the terminal on the way out
///   (RawTerminal Drop).
///
/// # Defensive Notes
///
/// - No `unwrap` / no panic.
/// - The direction is type-checked (`ArrowKeyDirection`), so there is no
///   "unknown direction" case to handle here; classification already rejected
///   non-arrow sequences upstream.
fn handle_arrow_key_input_mode(
    lines_editor_state: &mut EditorState,
    arrow_direction: ArrowKeyDirection,
) -> Result<bool> {
    match arrow_direction {
        ArrowKeyDirection::UpArrow => execute_command(lines_editor_state, Command::MoveUp(1)),
        ArrowKeyDirection::DownArrow => execute_command(lines_editor_state, Command::MoveDown(1)),
        ArrowKeyDirection::LeftArrow => execute_command(lines_editor_state, Command::MoveLeft(1)),
        ArrowKeyDirection::RightArrow => execute_command(lines_editor_state, Command::MoveRight(1)),
    }
}

/// Dispatches a single keystroke byte to the editor action.
///
/// # Project Context
///
/// This is the per-byte dispatcher for `EditorMode::KeystrokeInputMode`. It is
/// called once per byte by `handle_keystroke_input_session`, which owns the
/// `RawTerminal` and the read loop. This function does NOT read input, does NOT
/// own the terminal, and does NOT render — it only maps one byte to one action.
///
/// Separation of concerns:
/// - `handle_keystroke_input_session` : owns RawTerminal, reads bytes, renders,
///   handles EOF / read-error / mode-flag termination.
/// - `handle_single_byte_keystroke_input_mode`    : maps a single byte to a single action
///   (this function).
///
/// # Byte Dispatch Table
///
/// | Byte (hex)     | Meaning           | Action                                      |
/// |----------------|-------------------|---------------------------------------------|
/// | `0x1B`         | ESC               | `execute_command(.., EnterNormalMode)` — flips mode to Normal; this is the signal the session loop watches to exit |
/// | `0x08`, `0x7F` | Backspace, DEL    | `execute_command(.., DeleteBackspace)` (DEL treated as backspace) |
/// | `0x0A`, `0x0D` | LF, CR            | `execute_command(.., InsertNewline('\n'))` (CR treated as newline) |
/// | `0x20..=0x7E`  | printable ASCII   | clear redo logs, then `insert_text_chunk_at_cursor_position(.., &[byte])` |
/// | everything else| arrows, Tab(0x09), Ctrl/Alt/Fn, multibyte fragments | silently ignored: no edit, no redo-clear, no rebuild |
///
/// # Why the Printable Path Differs from Backspace/Newline (redo-clear)
///
/// In the editor, `button_safe_clear_all_redo_logs` is called by the CALLER of
/// the edit, not by the edit function itself:
///
/// - `Command::DeleteBackspace` and `Command::InsertNewline` arms inside
///   `execute_command` ALREADY call `button_safe_clear_all_redo_logs`
///   internally. So routing backspace and newline through `execute_command`
///   gives redo-clear automatically. We must NOT clear again here, or
///   we would double-clear (harmless but wasteful and misleading).
///
/// - `insert_text_chunk_at_cursor_position` does NOT clear redo logs itself.
///   Insert mode (`handle_utf8txt_insert_mode_input`) wraps it with
///   `button_safe_clear_all_redo_logs` before calling it. We replicate that
///   wrapping here for the printable-byte path. (Deliberate duplication of the
///   3-attempt retry pattern from insert mode — duplication is preferred over
///   abstraction-for-its-own-sake in this codebase.)
///
/// There is intentionally no `Command` variant that inserts a single arbitrary
/// printable byte via the chunk path; arbitrary-text insertion is done by
/// calling `insert_text_chunk_at_cursor_position` directly (as insert mode
/// does). That is why the printable path here does not go through
/// `execute_command`.
///
/// # One ASCII Byte == One Chunk Insert
///
/// A printable-ASCII byte (0x20..=0x7E) is, by definition, a complete and valid
/// single-byte UTF-8 character. Passing `&[byte]` (a one-byte slice) to
/// `insert_text_chunk_at_cursor_position` therefore:
///   - produces exactly ONE `AddCharacter` undo entry,
///   - advances the cursor by exactly one column,
///   - handles right-edge horizontal scroll,
/// matching insert mode precisely. This satisfies both the "make an undo-redo
/// log for that one byte" requirement and the "clear redo logs before each
/// edit" requirement.
///
/// # Rebuild / Render Policy
///
/// This function does NOT call `build_windowmap_nowrap` in the common path.
/// The edit functions own their own rebuilds:
///   - `insert_text_chunk_at_cursor_position` rebuilds on right-edge scroll.
///   - the `execute_command` arms for DeleteBackspace / InsertNewline rebuild
///     after the edit.
/// The session loop renders unconditionally at the top of its next iteration,
/// so whatever the model now holds gets painted. Ignored keys cause no edit and
/// no rebuild: nothing changed.
///
/// # Arguments
///
/// * `lines_editor_state` - mutable editor state (mode, cursor, buffers, etc.)
/// * `keystroke`          - the single raw byte read from the terminal
/// * `read_copy_path`     - borrow of the read-copy file path. The session owns
///                          the clone of `read_copy_path` and passes a borrow
///                          here, so this function never re-clones per keystroke.
///
/// # Returns
///
/// * `Ok(true)`  - editor loop should keep running. In the current command set
///   every handled byte yields `Ok(true)`: ESC routes through
///   `EnterNormalMode` (which returns the keep-running flag and flips the mode),
///   edits return the keep-running flag, and ignored bytes return `Ok(true)`
///   directly. The session loop CHECKS this value rather than assuming it: an
///   `Ok(false)` (no quit command exists in this mode today) is treated by the
///   caller as an unexpected contract violation and triggers a safe recovery to
///   Normal mode — it is not silently ignored.
/// * `Ok(false)` - reserved/unexpected in this mode; see above. This function
///   does not currently produce it, but the type permits it and the caller
///   handles it defensively.
/// * `Err(LinesError)` - a propagated error from an edit or command. Edit
///   functions handle their own non-critical failures internally (logging,
///   info-bar) and return Ok; a returned Err here is an unrecoverable
///   I/O failure and is propagated to the session, which restores the terminal
///   (RawTerminal Drop) on the way out.
///
/// # Defensive Notes
///
/// - No `unwrap` / no panic.
/// - Unknown bytes are silently ignored (handle-and-move-on): no edit, no log,
///   no state change. Goal: for arrow keys, Tab, and
///   stray escape-sequence fragments delivered one byte at a time in raw mode.
fn handle_single_byte_keystroke_input_mode(
    lines_editor_state: &mut EditorState,
    keystroke: u8,
    read_copy_path: &Path,
) -> Result<bool> {
    match keystroke {
        // ---------------------------------------------------------------------
        // ESC (0x1B): exit to Normal mode.
        // ---------------------------------------------------------------------
        // EnterNormalMode sets lines_editor_state.mode = Normal and rebuilds the
        // windowmap. The session loop's `while self.mode == KeystrokeInputMode`
        // condition then fails, so the loop exits cleanly and RawTerminal drops.
        0x1B => {
            // EnterNormalMode returns Ok(true) (keep running). We forward that.
            execute_command(lines_editor_state, Command::EnterNormalMode)
        }

        // ---------------------------------------------------------------------
        // Backspace (0x08) or DEL (0x7F): backspace-style delete.
        // ---------------------------------------------------------------------
        // DEL is treated as backspace per spec. DeleteBackspace's execute_command
        // arm clears redo logs internally and rebuilds the windowmap, so we do
        // NOT clear redo logs here (no double-clear).
        0x08 | 0x7F => execute_command(lines_editor_state, Command::DeleteBackspace),

        // ---------------------------------------------------------------------
        // LF (0x0A) or CR (0x0D): insert a single newline.
        // ---------------------------------------------------------------------
        // CR is treated as newline per spec. InsertNewline's execute_command arm
        // clears redo logs internally and rebuilds the windowmap, so we do NOT
        // clear redo logs here (no double-clear).
        0x0A | 0x0D => execute_command(lines_editor_state, Command::InsertNewline('\n')),

        // ---------------------------------------------------------------------
        // Printable ASCII (0x20 space .. 0x7E tilde): insert one byte.
        // ---------------------------------------------------------------------
        // This path does its OWN redo-clear (matching insert mode), because
        // insert_text_chunk_at_cursor_position does not clear redo logs itself.
        0x20..=0x7E => {
            // =================================================
            // Clear Redo Stack Before Editing (printable path)
            // =================================================
            // Same 3-attempt retry pattern insert mode uses. Redo-clear failure
            // is non-critical: the insert still proceeds, undo/redo may be in a
            // degraded state, and we surface a terse info-bar note. We never
            // abort the keystroke because of a redo-clear failure.
            let mut redo_clear_success = false;
            for attempt in 0..3 {
                match button_safe_clear_all_redo_logs(read_copy_path) {
                    Ok(_) => {
                        redo_clear_success = true;
                        break;
                    }
                    Err(_e) => {
                        #[cfg(debug_assertions)]
                        eprintln!("hkim: redo clear attempt {} failed: {:?}", attempt, _e);

                        if attempt < 2 {
                            thread::sleep(Duration::from_millis(100));
                        }
                    }
                }
            }

            if !redo_clear_success {
                // Terse, no-PII log + info-bar note. Non-fatal.
                log_error(
                    "Cannot clear redo logs",
                    Some("handle_single_byte_keystroke_input_mode:printable"),
                );
                let _ = lines_editor_state.set_info_bar_message("redo clear failed");
            }

            // Insert the single byte as a one-character chunk.
            // One printable-ASCII byte is one valid UTF-8 character, so this
            // produces exactly one AddCharacter undo entry, advances the cursor,
            // and handles right-edge scroll (with its own rebuild) — matching
            // insert mode.
            // Insert the single byte as a one-character chunk.
            // One printable-ASCII byte is one valid UTF-8 character, so this
            // produces exactly one AddCharacter undo entry, advances the cursor,
            // and handles right-edge scroll — matching insert mode.
            let byte_slice = [keystroke];
            insert_text_chunk_at_cursor_position(lines_editor_state, read_copy_path, &byte_slice)?;

            // -----------------------------------------------------------------
            // Rebuild the windowmap after the insert (REQUIRED).
            // -----------------------------------------------------------------
            // insert_text_chunk_at_cursor_position only rebuilds the windowmap
            // CONDITIONALLY — solely when the cursor crosses the right edge and
            // the window must scroll horizontally. In the common case (typing
            // within the visible width), it updates cursor.tui_visual_col and writes the
            // byte to the file, but does NOT rebuild the display model. Without a
            // rebuild here, the display buffers still hold the pre-insert text:
            // the cursor would move but the typed character would be invisible
            // until some OTHER action (newline, backspace) triggered a rebuild.
            //
            // This mirrors EXACTLY what cooked insert mode does: its caller
            // (handle_utf8txt_insert_mode_input) calls build_windowmap_nowrap
            // immediately after each insert_text_chunk_at_cursor_position. We are
            // the caller in ki-mode, so we carry the same responsibility.
            //
            // Backspace (0x08/0x7F) and newline (0x0A/0x0D) do NOT need a rebuild
            // here because they route through execute_command, whose
            // DeleteBackspace / InsertNewline arms already rebuild internally.
            // Adding a rebuild there would double-rebuild. Only this printable
            // path, which calls the chunk function directly, needs this rebuild.
            //
            // If the insert failed gracefully (invalid cursor at end-of-line —
            // a PRE-EXISTING shared bug also present in insert mode), the file
            // is unchanged and this rebuild simply repaints the current model.
            // That is harmless: rebuild is idempotent with respect to an
            // unchanged file.
            build_windowmap_nowrap(lines_editor_state, read_copy_path)?;

            Ok(true)
        }

        // ---------------------------------------------------------------------
        // Everything else: silently ignore.
        // ---------------------------------------------------------------------
        // This includes Tab (0x09), all C0 control codes not handled above,
        // and the individual bytes of multibyte escape sequences (arrow keys,
        // Home/End, Page Up/Down, function keys) which arrive one byte at a time
        // in raw mode. No edit, no redo-clear, no rebuild, no state change.
        // Handle-and-move-on: keep the editor running.
        _ => Ok(true),
    }
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
        .get_row_col_file_position(
            lines_editor_state.cursor.tui_row,
            lines_editor_state.cursor.tui_visual_col,
        )?
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
    if lines_editor_state.cursor.tui_visual_col > 0 {
        lines_editor_state.cursor.tui_visual_col -= 1;
    } else if lines_editor_state.cursor.tui_row > 0 {
        // Deleted at line start - move to end of previous line
        lines_editor_state.cursor.tui_row -= 1;
        // Will be repositioned after window rebuild
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
/// undo logs that will reconstruct it. Naive approach would be:
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
/// This naming ensures LIFO execution order through filesystem sorting.
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
///  // Delete line 3: "pine\nuts nheggs\n" at position 25
/// delete_current_line_noload(&mut state, &file_path)?;
///
///  // Undo logs created (button stack, all at position 25):
///  // changelog_file/1.o: ADD 'p' at 25
///  // changelog_file/1.n: ADD 'i' at 25
///  // ... 14 more logs ...
///  // changelog_file/1.a: ADD 's' at 25
///  // changelog_file/1:   ADD '\n' at 25
///
///  // User presses undo:
///  // 1. Reads "1" → ADD '\n' at 25 → "\n"
///  // 2. Reads "1.a" → ADD 's' at 25 → "s\n"
///  // 3. Reads "1.b" → ADD 'g' at 25 → "gs\n"
///  // ... cascading insertions ...
///  // 17. Reads "1.o" → ADD 'p' at 25 → "pine\nuts nheggs\n" ✓
/// ```
///
/// # See Also
///
/// * `button_make_changelog_from_user_character_action_level()` - Creates individual log entries
/// * `button_add_multibyte_make_log_files()` - Handles multi-byte characters with letter suffixes
/// * `delete_byte_range_chunked()` - Performs the deletion
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
        .get_row_col_file_position(state.cursor.tui_row, state.cursor.tui_visual_col)?
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

    // TODO: determining ideal default buffer & chunk size
    // Copy line bytes to temp file (chunked, no heap)
    const CHUNK_SIZE: usize = 32;
    let mut buffer = [0u8; CHUNK_SIZE];
    let mut bytes_to_copy = (delete_end - line_start) as usize;
    let mut copy_iterations = 0;

    while bytes_to_copy > 0 && copy_iterations < limits::MAX_CHUNKS {
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

    if copy_iterations >= limits::MAX_CHUNKS {
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

            state.cursor.tui_visual_col = 0;
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

                state.cursor.tui_visual_col = 0;
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
    state.cursor.tui_visual_col = 0; // Move to start of (new) line

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
/// undo logs that will reconstruct it. Naive approach would be:
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
/// This naming ensures LIFO execution order through filesystem sorting.
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
/// - MAX_CHUNKS: from standard constant (e.g. size max)
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
///  // User selects "world" in "Hello world!\n" (positions 6-11)
/// state.file_position_of_vis_select_start = 6;
/// state.file_position_of_vis_select_end = 11;  // 'd' starts at position 10, ends at 11
///
/// delete_position_range_noload(&mut state, &file_path)?;
///
///  // Result: "Hello !\n" (6 bytes deleted: "world")
///  // Logged as: "DELETE_RANGE bytes:6-11"
///
///  // Undo logs created (button stack, all at position 6):
///  // changelog_file/1.e: ADD 'w' at 6
///  // changelog_file/1.d: ADD 'o' at 6
///  // changelog_file/1.c: ADD 'r' at 6
///  // changelog_file/1.b: ADD 'l' at 6
///  // changelog_file/1.a: ADD 'd' at 6
///  // changelog_file/1:   ADD ' ' at 6  (space before 'world')
///
///  // User presses undo:
///  // 1. Reads "1" → ADD ' ' at 6 → "Hello  !\n"
///  // 2. Reads "1.a" → ADD 'd' at 6 → "Hello d !\n"
///  // 3. Reads "1.b" → ADD 'l' at 6 → "Hello ld !\n"
///  // ... cascading insertions ...
///  // 6. Reads "1.e" → ADD 'w' at 6 → "Hello world!\n" ✓
/// ```
///
/// ```ignore
///  // Multi-byte UTF-8 example: Delete "世界" (6 bytes: 3+3)
/// state.file_position_of_vis_select_start = 10;
/// state.file_position_of_vis_select_end = 16;  // '界' starts at 13, ends at 16
///
/// delete_position_range_noload(&mut state, &file_path)?;
///
///  // UTF-8 boundary detection ensures complete character deletion
///  // Undo logs preserve multi-byte characters
/// ```
///
/// ```ignore
///  // Backwards selection (normalized automatically)
/// state.file_position_of_vis_select_start = 20;  // End cursor
/// state.file_position_of_vis_select_end = 10;    // Start cursor
///
/// delete_position_range_noload(&mut state, &file_path)?;
///  // Normalized to (10, 20), deletion proceeds normally
/// ```
///
/// # See Also
///
/// * `delete_current_line_noload()` - Line-based deletion (finds line boundaries)
/// * `normalize_sort_sanitize_selection_range()` - Handles backwards selections
/// * `detect_utf8_byte_count()` - UTF-8 character length detection
/// * `button_make_changelog_from_user_character_action_level()` - Creates individual log entries
/// * `button_add_multibyte_make_log_files()` - Handles multi-byte characters with letter suffixes
/// * `delete_byte_range_chunked()` - Performs the deletion
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
    // TODO: determining ideal default buffer & chunk size
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

            state.cursor.tui_visual_col = 0;
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

                state.cursor.tui_visual_col = 0;
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

        // Logging loop (same pattern as file insertion)
        loop {
            if logging_chunk_counter >= limits::MAX_CHUNKS {
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
    // Use normalize_sort_sanitize_selection_range() before this function
    // Defensive: Validate range
    if start_byte >= end_byte {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid deletion range",
        ));
    }

    // Create temp file in same directory
    let temp_path = file_path.with_extension("tmp_delete");

    // TODO: determining ideal default buffer & chunk size
    // Pre-allocated N-bytes buffer
    const DBRC_CHUNK_SIZE: usize = 4;
    let mut buffer = [0u8; DBRC_CHUNK_SIZE];

    let mut source = File::open(file_path)?;
    let mut dest = File::create(&temp_path)?;

    // Phase 1: Copy bytes before deletion point
    let mut bytes_copied = 0u64;
    let mut iterations = 0;

    while bytes_copied < start_byte && iterations < limits::FILE_SEEK_BYTES {
        iterations += 1;

        let to_read = ((start_byte - bytes_copied) as usize).min(DBRC_CHUNK_SIZE);
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
/// # Coordinate Spaces (see the module "Coordinate Spaces" reference)
/// - In  `starting_row` : #3 top-of-window line number
/// - In  `tui_row`      : #6 TUI display row (row + starting_row = this line's #3)
/// - Out: line-number prefix width in #5 VISUAL cells (== chars; prefix is ASCII).
///        The prefix occupies cells [0, return); content begins at cell `return`.
///
/// # Examples
/// - Line 5, 20 rows: returns 3 (might see line 24, use 2 digits + space)
/// - Line 95, 20 rows: returns 4 (might see line 114, use 3 digits + space)
fn calculate_line_number_width(
    starting_row: usize,
    tui_row: usize,
    effective_rows: usize,
) -> usize {
    // if line_number == 0 {
    //     return 2; // Edge case: treat as single digit + pad
    // }
    //

    let line_number = starting_row + tui_row;

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
    line_count_at_top_of_window: usize, // line_count_at_top_of_window
    line_number: usize,                 // fileline_number_for_display
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
        // hard set default for 0-9
        bool_output = true;
    } else if line_number < 100 {
        if line_count_at_top_of_window > (100 - effective_rows - 1) {
            if line_number > (100 - effective_rows - 1) {
                bool_output = true;
            } else {
                bool_output = false;
            }
        } else {
            bool_output = false;
        }
    } else if line_number < 1_000 {
        if line_count_at_top_of_window > (1_000 - effective_rows - 1) {
            if line_number > (1_000 - effective_rows - 1) {
                bool_output = true;
            } else {
                bool_output = false;
            }
        } else {
            bool_output = false;
        }
        // if line_number > (1_000 - effective_rows - 1) {
        //     bool_output = true;
        // } else {
        //     bool_output = false;
        // }
    } else if line_number < 10_000 {
        if line_count_at_top_of_window > (10_000 - effective_rows - 1) {
            if line_number > (10_000 - effective_rows - 1) {
                bool_output = true;
            } else {
                bool_output = false;
            }
        } else {
            bool_output = false;
        }
        // if line_number > (10_000 - effective_rows) {
        //     bool_output = true;
        // } else {
        //     bool_output = false;
        // }
    } else if line_number < 100_000 {
        if line_count_at_top_of_window > (100_000 - effective_rows - 1) {
            if line_number > (100_000 - effective_rows - 1) {
                bool_output = true;
            } else {
                bool_output = false;
            }
        } else {
            bool_output = false;
        }
        // if line_number > (100_000 - effective_rows) {
        //     bool_output = true;
        // } else {
        //     bool_output = false;
        // }
    } else if line_number < 1_000_000 {
        if line_count_at_top_of_window > (1_000_000 - effective_rows - 1) {
            if line_number > (1_000_000 - effective_rows - 1) {
                bool_output = true;
            } else {
                bool_output = false;
            }
        } else {
            bool_output = false;
        }
        // if line_number > (1_000_000 - effective_rows) {
        //     bool_output = true;
        // } else {
        //     bool_output = false;
        // }
    } else if line_number < 10_000_000 {
        if line_count_at_top_of_window > (10_000_000 - effective_rows - 1) {
            if line_number > (10_000_000 - effective_rows - 1) {
                bool_output = true;
            } else {
                bool_output = false;
            }
        } else {
            bool_output = false;
        }
        // if line_number > (10_000_000 - effective_rows) {
        //     bool_output = true;
        // } else {
        //     bool_output = false;
        // }
    } else {
        bool_output = false; // Cap at 6 digits (999,999 lines max) TODO
    }

    bool_output
}

// TODO: determining ideal default buffer & chunk size
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
    let file_pos = match lines_editor_state.get_row_col_file_position(
        lines_editor_state.cursor.tui_row,
        lines_editor_state.cursor.tui_visual_col,
    ) {
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

    // TODO: determining ideal default buffer & chunk size
    // TODO this should not be be allocating MORE memory
    // this should use a standard modular buffer
    // Pre-allocated N-bytes buffer
    // TODO: determining ideal default buffer & chunk size
    const INACC_CHUNK_SIZE: usize = 128;
    let mut buffer = [0u8; INACC_CHUNK_SIZE];

    // Step 4: Copy bytes before insertion point
    let mut bytes_copied = 0u64;
    let mut iterations = 0;

    while bytes_copied < insert_position && iterations < limits::FILE_SEEK_BYTES {
        iterations += 1;

        let to_read = ((insert_position - bytes_copied) as usize).min(INACC_CHUNK_SIZE);

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
    lines_editor_state.cursor.tui_row += 1;

    // Calculate where the text starts after the line number
    let new_line_number =
        lines_editor_state.line_count_at_top_of_window + lines_editor_state.cursor.tui_row;
    let line_num_width = calculate_line_number_width(
        lines_editor_state.line_count_at_top_of_window,
        new_line_number + 1,
        lines_editor_state.effective_rows,
    ); // +1 for 1-indexed display

    lines_editor_state.cursor.tui_visual_col = line_num_width; // Position cursor after line number
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
/// **Iteration safety:** Limited to MAX_CHUNKS
/// (e.g. usize::MAX) to prevent infinite
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
/// - byte-level operations
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
/// - start_byte_position points past last byte
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
/// Insert another file at current cursor position
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
        .get_row_col_file_position(state.cursor.tui_row, state.cursor.tui_visual_col)
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
                "match state.get_row_col_file_position(state.cursor.tui_row, state.cursor.tui_visual_col) Error getting cursor position",
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

    const IFAC_CHUNK_SIZE: usize = 8;

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
        if chunk_counter >= limits::MAX_CHUNKS {
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
        let mut buffer = [0u8; IFAC_CHUNK_SIZE];

        // Security mode: manually clear buffer before use
        // Prevents data leakage between chunks if read fails mid-buffer
        if state.security_mode {
            for i in 0..IFAC_CHUNK_SIZE {
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
        // =================================================
        // Debug-Assert, Test-Asset, Production-Catch-Handle
        // =================================================
        // This is not included in production builds
        // assert: only when running in a debug-build: will panic
        debug_assert!(
            bytes_read <= IFAC_CHUNK_SIZE,
            "bytes_read ({}) exceeded buffer size ({})",
            bytes_read,
            IFAC_CHUNK_SIZE
        );
        // Defensive assertion: bytes_read should never exceed buffer size
        // If it does, indicates memory corruption or cosmic ray bit flip
        // This is the only panic point - for catastrophic failure only
        #[cfg(test)]
        assert!(
            bytes_read <= IFAC_CHUNK_SIZE,
            "bytes_read ({}) exceeded buffer size ({})",
            bytes_read,
            IFAC_CHUNK_SIZE
        );
        // Catch & Handle without panic in production
        // This IS included in production to safe-catch
        if !bytes_read <= IFAC_CHUNK_SIZE {
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
        if logging_chunk_counter >= limits::MAX_CHUNKS {
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
        let mut buffer = [0u8; IFAC_CHUNK_SIZE];

        // Security mode: clear buffer before use
        if state.security_mode {
            for i in 0..IFAC_CHUNK_SIZE {
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

        debug_assert!(
            bytes_read <= IFAC_CHUNK_SIZE,
            "bytes_read exceeded IFAC_CHUNK_SIZE"
        );

        #[cfg(test)]
        assert!(
            bytes_read <= IFAC_CHUNK_SIZE,
            "bytes_read exceeded IFAC_CHUNK_SIZE"
        );

        // Production catch-handle
        if bytes_read > IFAC_CHUNK_SIZE {
            #[cfg(debug_assertions)]
            log_error(
                &format!(
                    "bytes_read {} exceeded IFAC_CHUNK_SIZE {}",
                    bytes_read, IFAC_CHUNK_SIZE
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

/// Inserts bytes at a specific file position using safe chunked temp-file copy.
///
/// # Overview
///
/// This helper inserts a byte slice at an arbitrary byte offset in a file by
/// streaming the file through a temporary file, rather than attempting an
/// in-place shift with a fixed-size buffer. This makes the operation correct
/// for files of *any* size and eliminates the data-truncation bug present in
/// the previous fixed-buffer implementation.
///
/// **Operation:**
/// ```text
/// Before: [A B C D E F]
///         Insert "XY" at position 3
/// After:  [A B C X Y D E F]
///                 ↑ insertion point (position 3)
/// ```
///
/// # Why Temp-File Copy (and not in-place shift)
///
/// A naive in-place shift reads the bytes *after* the insertion point into a
/// stack buffer, writes the new bytes, then writes the buffered tail back.
/// If the tail is larger than the buffer, the remainder of the file is silently
/// lost (truncated). This function avoids that entirely by copying the whole
/// tail through a bounded, *looping* chunked read/write, so no data can be lost
/// regardless of file size or insertion length.
///
/// # Memory Safety - Stack Allocated Bounded Buffer
///
/// - Uses a fixed-size stack buffer for streaming (no per-file heap growth).
/// - The buffer size does NOT limit correctness; large tails are copied in a
///   bounded loop, one chunk at a time.
/// - Iteration counts are bounded by `limits::FILE_SEEK_BYTES` to satisfy
///   NASA-Power-of-10-style bounded-loop requirements.
///
/// # Arguments
///
/// * `file_path` - Path to target file (must already exist; not created here).
/// * `position`  - Byte offset where to insert
///                 (0 = start, file_size = append).
/// * `bytes`     - Slice of bytes to insert (any length; may be empty).
///
/// # Returns
///
/// * `Ok(())`         - Bytes inserted successfully; file replaced atomically
///                      via rename of the temp file.
/// * `Err(io::Error)` - A file operation failed (open, create, seek, read,
///                      write, flush, rename), OR the insertion `position`
///                      exceeds the file length, OR a bounded iteration limit
///                      was exceeded (indicating an unexpectedly large file or
///                      a logic error).
///
/// # Algorithm
///
/// 1. Open source file (read) and create a temp file (write).
/// 2. Copy bytes `[0..position)` from source to temp in bounded chunks.
/// 3. Write the new `bytes` to temp.
/// 4. Copy bytes `[position..EOF)` from source to temp in bounded chunks.
/// 5. Flush and close both files.
/// 6. Atomically replace the original file with the temp file via `fs::rename`.
///
/// # Edge Cases
///
/// **Insert at EOF (position == file size):**
/// - Phase 2 copies the entire file.
/// - Phase 3 writes the new bytes.
/// - Phase 4 copies nothing (already at EOF).
/// - Equivalent to an append.
///
/// **Insert at start (position == 0):**
/// - Phase 2 copies nothing.
/// - Phase 3 writes the new bytes first.
/// - Phase 4 copies the entire original file after them.
///
/// **Empty insertion (bytes.len() == 0):**
/// - Valid no-op in effect: the file is rewritten identically.
/// - Still performs the full copy (file timestamp updates).
///
/// **position > file length:**
/// - Detected in Phase 2 when EOF is reached before reaching `position`.
/// - Returns `io::ErrorKind::InvalidInput`; temp file is left behind but the
///   original file is never modified (rename never occurs).
///
/// # Atomicity
///
/// The original file is only replaced via `fs::rename` after the temp file is
/// fully written and flushed. If any step fails before the rename, the original
/// file is left untouched. (A stray `.tmp_insert` file may remain on failure.)
///
/// # Performance
///
/// - **Time:**  O(N) where N = total file size (full copy per insertion).
/// - **Space:** O(1) stack buffer, independent of file size.
/// - Not optimized for many small repeated insertions (each rewrites the file).
///
/// # Defensive Programming
///
/// - No `unwrap`/`expect`; every I/O operation is explicitly `?`-checked.
/// - Bounded loops guard against runaway iteration.
/// - Both files are explicitly dropped before the rename.
///
/// # See Also
///
/// * `delete_byte_range_chunked()`      - Inverse (removes a byte range).
/// * `insert_newline_at_cursor_chunked()` - Same pattern, specialized for `\n`.
fn insert_bytes_at_position(file_path: &Path, position: u64, bytes: &[u8]) -> io::Result<()> {
    // Create temp file path alongside the original.
    let temp_path = file_path.with_extension("tmp_insert");

    // Open source (read) and destination temp (write).
    let mut source = File::open(file_path)?;
    let mut dest = File::create(&temp_path)?;

    // TODO: determining ideal default buffer & chunk size
    // Bounded, stack-allocated streaming buffer. Size affects performance
    // only, NOT correctness — large tails are copied in a loop.
    const IBAP_CHUNK_SIZE: usize = 256;
    let mut buffer = [0u8; IBAP_CHUNK_SIZE];

    // -----------------------------------------------------------------
    // Phase 1: Copy bytes [0..position) from source to temp (chunked).
    // -----------------------------------------------------------------
    let mut bytes_copied = 0u64;
    let mut iterations = 0;

    while bytes_copied < position && iterations < limits::FILE_SEEK_BYTES {
        iterations += 1;

        // Read only up to the insertion boundary this chunk.
        let to_read = ((position - bytes_copied) as usize).min(IBAP_CHUNK_SIZE);
        let n = source.read(&mut buffer[..to_read])?;

        if n == 0 {
            // Reached EOF before reaching insertion position: invalid.
            // Original file is untouched (no rename has occurred).
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Insert position exceeds file length",
            ));
        }

        dest.write_all(&buffer[..n])?;
        bytes_copied += n as u64;
    }

    // Defensive: bounded-iteration guard for Phase 1.
    if iterations >= limits::FILE_SEEK_BYTES && bytes_copied < position {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Max iterations exceeded copying before insert point",
        ));
    }

    // -----------------------------------------------------------------
    // Phase 2: Write the new bytes at the insertion point.
    // -----------------------------------------------------------------
    // (Safe when bytes.is_empty(): write_all with empty slice is a no-op.)
    dest.write_all(bytes)?;

    // -----------------------------------------------------------------
    // Phase 3: Copy remaining bytes [position..EOF) from source to temp.
    // Source is already positioned at `position` from Phase 1 reads.
    // -----------------------------------------------------------------
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
            break; // EOF reached — tail fully copied.
        }

        dest.write_all(&buffer[..n])?;
    }

    // -----------------------------------------------------------------
    // Phase 4: Flush, close, and atomically replace the original.
    // -----------------------------------------------------------------
    dest.flush()?;
    drop(dest);
    drop(source);

    fs::rename(&temp_path, file_path)?;

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
///
/// # Phase 2 Design: Scale-Agnostic Backward Block-Shift (In-Place Tail Relocation)
///
/// ## Why this design exists (project context for future developers)
///
/// Inserting `N` bytes in the MIDDLE of a file requires relocating every byte
/// AFTER the insertion point forward by `N` bytes, so the new text can occupy
/// the gap. This function performs that relocation **in place**, on the
/// read-copy file, using a **bounded loop of fixed-size chunks**.
///
/// This replaces an earlier transitional approach that relocated the file tail
/// with a single bounded read into a single fixed buffer. That approach could
/// only relocate up to one buffer's worth of tail bytes and therefore corrupted
/// any file where more than `TEXT_BUCKET_BRIGADE_CHUNKING_BUFFER_SIZE` bytes
/// followed the cursor (middle-of-file inserts). The corruption also
/// desynchronized byte offsets from the windowmap, which is a plausible source
/// of downstream "cursor not on valid file position" symptoms on long lines.
///
/// ## The algorithm (why BACKWARD, why chunked)
///
/// To insert `N` bytes at `insert_position` in a file of length `L`:
/// - The tail region is bytes `[insert_position .. L]`, of length `tail_len`.
/// - It must move to `[insert_position + N .. L + N]`.
/// - Source and destination OVERLAP, and destination > source. Copying
///   front-to-back would overwrite tail bytes before they were read. Therefore
///   we copy **back-to-front** (highest addresses first).
///
/// Chunk size is `TEXT_BUCKET_BRIGADE_CHUNKING_BUFFER_SIZE`. **Correctness does
/// not depend on the chunk size** — any positive value yields identical results;
/// only the number of loop iterations changes. This is what makes the design
/// scale-agnostic and consistent with the modular small-chunk stdin brigade.
///
/// ## Bounded-loop guarantees (Power-of-10 rule 2)
///
/// - The shift loop's `bytes_remaining` strictly decreases each iteration and
///   the loop exits at zero: it is intrinsically bounded.
/// - An additional independent iteration cap
///   (`ceil(tail_len / CHUNK) + 1`, plus a hard `limits::TEXT_INPUT_CHUNKS`
///   ceiling) is enforced as a failsafe against a corrupted/short-read stream,
///   so the loop can never spin.
///
/// ## Safety model (why no temp file, why no atomicity)
///
/// - This operates on the **read-copy**, which is disposable/regenerable from
///   the untouched original file (see `create_a_readcopy_of_file()`). The
///   original is never mutated by this function, so the user's real data is
///   never at risk here.
/// - No temporary file is used. A temp file would reintroduce cross-mount
///   non-atomic-rename issues and temp-name collision/cleanup concerns, none of
///   which Rust can portably guarantee away. Same-mount in-place editing avoids
///   all of that.
/// - Power-failure / torn-write atomicity is intentionally **out of scope**: an
///   interrupted shift can only leave the read-copy inconsistent, and the
///   read-copy is reconstructible from the original. We do not attempt journaling
///   or rename-swap here.
///
/// ## Short-read handling
///
/// `Read::read` may legally return fewer bytes than requested. The shift loop
/// therefore loops on each chunk position until the intended chunk length is
/// fully read (bounded by an inner attempt cap), never assuming a single `read`
/// filled the buffer.
///
pub fn insert_text_chunk_at_cursor_position(
    lines_editor_state: &mut EditorState,
    file_path: &Path,
    text_bytes: &[u8],
) -> Result<()> {
    // ==================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // ==================================================

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

    let file_pos = match lines_editor_state.get_row_col_file_position(
        lines_editor_state.cursor.tui_row,
        lines_editor_state.cursor.tui_visual_col,
    ) {
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

    // ============================================
    // Phase 2: Perform File Insertion
    //          (Scale-Agnostic Backward Block-Shift)
    // ============================================
    //
    // See the "Phase 2 Design" section in this function's doc-string for the
    // full rationale. Summary:
    //   - Relocate the file tail [insert_position .. L] forward by N bytes,
    //     where N = text_bytes.len(), using fixed-size chunks.
    //   - Copy BACK-TO-FRONT because source/destination overlap (dst > src).
    //   - Chunk size is TEXT_BUCKET_BRIGADE_CHUNKING_BUFFER_SIZE; correctness
    //     does not depend on its value.
    //   - Operates on the disposable read-copy; original file is untouched.

    let insert_byte_count: u64 = text_bytes.len() as u64;

    // Nothing to insert: succeed without touching the file.
    if insert_byte_count == 0 {
        return Ok(());
    }

    // Open the read-copy for read+write (no truncation).
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(file_path)
        .map_err(|e| LinesError::Io(e))?;

    // Determine current file length (L) to know how much tail must move.
    let file_length: u64 = file.seek(SeekFrom::End(0)).map_err(|e| LinesError::Io(e))?;

    // ==================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // ==================================================
    // Required-condition: insert_position must be within the file [0 .. L].
    // A position past EOF would mean the windowmap and file are desynchronized.
    #[cfg(all(debug_assertions, not(test)))]
    debug_assert!(
        insert_position <= file_length,
        "insert_position beyond end of file"
    );

    #[cfg(test)]
    assert!(
        insert_position <= file_length,
        "insert_position beyond end of file"
    );

    if insert_position > file_length {
        #[cfg(debug_assertions)]
        log_error(
            &format!(
                "itcacp: insert_position {} > file_length {}",
                insert_position, file_length
            ),
            Some("insert_text_chunk_at_cursor_position:phase2"),
        );

        #[cfg(not(debug_assertions))]
        log_error(
            "itcacp: insert pos beyond EOF",
            Some("insert_text_chunk_at_cursor_position:phase2"),
        );

        let _ = lines_editor_state.set_info_bar_message("insert pos error");
        return Err(LinesError::GeneralAssertionCatchViolation(
            "itcacp: insert position beyond EOF".into(),
        ));
    }

    // Length of the tail region that must be relocated forward.
    // Safe: insert_position <= file_length checked above.
    let tail_length: u64 = file_length - insert_position;

    // Fixed-size stack buffer. Chunk size comes from the shared brigade
    // constant; correctness is independent of this value (only iteration
    // count changes).
    let mut shift_buffer = [0u8; TEXT_BUCKET_BRIGADE_CHUNKING_BUFFER_SIZE];
    let chunk_size: u64 = TEXT_BUCKET_BRIGADE_CHUNKING_BUFFER_SIZE as u64;

    // ==================================================
    // Debug-Assert, Test-Assert, Production-Catch-Handle
    // ==================================================
    // Required-condition: chunk size must be positive, else the shift loop
    // could never make progress.
    #[cfg(all(debug_assertions, not(test)))]
    debug_assert!(chunk_size > 0, "chunk_size must be positive");

    #[cfg(test)]
    assert!(chunk_size > 0, "chunk_size must be positive");

    if chunk_size == 0 {
        #[cfg(debug_assertions)]
        log_error(
            "itcacp: chunk_size is zero",
            Some("insert_text_chunk_at_cursor_position:phase2"),
        );

        #[cfg(not(debug_assertions))]
        log_error(
            "itcacp: config error",
            Some("insert_text_chunk_at_cursor_position:phase2"),
        );

        let _ = lines_editor_state.set_info_bar_message("config error");
        return Err(LinesError::GeneralAssertionCatchViolation(
            "itcacp: zero chunk size".into(),
        ));
    }

    // ------------------------------------------------------------------
    // Backward block-shift: move [insert_position .. L] forward by N bytes.
    //
    // We walk from the END of the tail toward insert_position, copying one
    // chunk at a time. Because destination > source and regions overlap,
    // back-to-front ordering guarantees we never overwrite unread bytes.
    // ------------------------------------------------------------------

    // Failsafe iteration cap (independent of the intrinsic bound below).
    // Number of chunks needed is ceil(tail_length / chunk_size). We add a
    // margin and also clamp to a hard project ceiling, so a malformed stream
    // can never cause an unbounded loop.
    let expected_chunk_iterations: u64 = (tail_length / chunk_size) + 1 + 1; // ceil-ish + safety margin
    let max_shift_iterations: u64 = expected_chunk_iterations.min(limits::TEXT_INPUT_CHUNKS as u64);

    let mut bytes_remaining: u64 = tail_length;
    let mut shift_iteration: u64 = 0;

    while bytes_remaining > 0 {
        // Independent failsafe bound (Power-of-10 rule 2).
        shift_iteration += 1;
        if shift_iteration > max_shift_iterations {
            #[cfg(debug_assertions)]
            log_error(
                &format!(
                    "itcacp: shift exceeded max iterations ({})",
                    max_shift_iterations
                ),
                Some("insert_text_chunk_at_cursor_position:phase2"),
            );

            #[cfg(not(debug_assertions))]
            log_error(
                "itcacp: shift iteration overflow",
                Some("insert_text_chunk_at_cursor_position:phase2"),
            );

            let _ = lines_editor_state.set_info_bar_message("shift error");
            return Err(LinesError::GeneralAssertionCatchViolation(
                "itcacp: shift iteration overflow".into(),
            ));
        }

        // Size of the chunk to move this iteration: min(chunk_size, remaining).
        // Safe cast: this_chunk_len <= chunk_size <= buffer length (usize).
        let this_chunk_len: u64 = if bytes_remaining < chunk_size {
            bytes_remaining
        } else {
            chunk_size
        };
        let this_chunk_len_usize: usize = this_chunk_len as usize;

        // Source is the highest not-yet-moved slice of the tail.
        // src = insert_position + (bytes_remaining - this_chunk_len)
        // dst = src + insert_byte_count
        // Safe: bytes_remaining >= this_chunk_len (branch above).
        let source_offset: u64 = insert_position + (bytes_remaining - this_chunk_len);
        let destination_offset: u64 = source_offset + insert_byte_count;

        // --- Read the source chunk (handle short reads defensively) ---
        file.seek(SeekFrom::Start(source_offset))
            .map_err(|e| LinesError::Io(e))?;

        let mut filled: usize = 0;
        let mut read_attempts: u32 = 0;
        // Inner failsafe: bound the short-read retry loop.
        const MAX_READ_ATTEMPTS: u32 = 64;

        while filled < this_chunk_len_usize {
            read_attempts += 1;
            if read_attempts > MAX_READ_ATTEMPTS {
                #[cfg(debug_assertions)]
                log_error(
                    &format!(
                        "itcacp: read stalled at offset {} ({} of {} bytes)",
                        source_offset, filled, this_chunk_len_usize
                    ),
                    Some("insert_text_chunk_at_cursor_position:phase2"),
                );

                #[cfg(not(debug_assertions))]
                log_error(
                    "itcacp: read stalled",
                    Some("insert_text_chunk_at_cursor_position:phase2"),
                );

                let _ = lines_editor_state.set_info_bar_message("read error");
                return Err(LinesError::GeneralAssertionCatchViolation(
                    "itcacp: read stalled during shift".into(),
                ));
            }

            let n = file
                .read(&mut shift_buffer[filled..this_chunk_len_usize])
                .map_err(|e| LinesError::Io(e))?;

            if n == 0 {
                // Unexpected EOF inside a region we already sized from file_length.
                // Treat as a torn/short read-copy: fail cleanly (read-copy is
                // disposable and regenerable from the original).
                #[cfg(debug_assertions)]
                log_error(
                    &format!(
                        "itcacp: unexpected EOF at offset {} ({} of {} bytes)",
                        source_offset, filled, this_chunk_len_usize
                    ),
                    Some("insert_text_chunk_at_cursor_position:phase2"),
                );

                #[cfg(not(debug_assertions))]
                log_error(
                    "itcacp: unexpected EOF",
                    Some("insert_text_chunk_at_cursor_position:phase2"),
                );

                let _ = lines_editor_state.set_info_bar_message("read error");
                return Err(LinesError::GeneralAssertionCatchViolation(
                    "itcacp: unexpected EOF during shift".into(),
                ));
            }

            filled += n;
        }

        // --- Write the chunk to its shifted destination ---
        file.seek(SeekFrom::Start(destination_offset))
            .map_err(|e| LinesError::Io(e))?;

        file.write_all(&shift_buffer[..this_chunk_len_usize])
            .map_err(|e| LinesError::Io(e))?;

        // Progress: strictly decreasing -> intrinsic loop bound.
        bytes_remaining -= this_chunk_len;
    }

    // --- Tail is now relocated; write the new text into the vacated gap ---
    file.seek(SeekFrom::Start(insert_position))
        .map_err(|e| LinesError::Io(e))?;

    file.write_all(text_bytes).map_err(|e| LinesError::Io(e))?;

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
            lines_editor_state.cursor.tui_visual_col += char_count;

            let right_edge = lines_editor_state.effective_cols.saturating_sub(1);
            if lines_editor_state.cursor.tui_visual_col > right_edge {
                let overflow = lines_editor_state.cursor.tui_visual_col - right_edge;
                lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset += overflow;
                lines_editor_state.cursor.tui_visual_col = right_edge;
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
    lines_editor_state.cursor.tui_visual_col += char_count;

    // ==========================================
    // Check if cursor exceeded right edge
    // ==========================================
    let right_edge = lines_editor_state.effective_cols.saturating_sub(1);

    if lines_editor_state.cursor.tui_visual_col > right_edge {
        // Calculate how far past edge we went
        let overflow = lines_editor_state.cursor.tui_visual_col - right_edge;

        // Scroll window right to accommodate
        lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset += overflow;

        // Move cursor back to right edge
        lines_editor_state.cursor.tui_visual_col = right_edge;

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
/// as a new clipboard file. Handles multi-byte UTF-8 characters by
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
/// - File content still copied (raw bytes preserved)
///
/// **Selection starts mid-character:**
/// - Not adjusted - start position used as-is
/// - May result in partial character at start (corrupted)
/// - Current design: only adjust end, not start (room for improvement)
///
/// **Selection spans multi-byte characters:**
/// - Example: "hello 花 world 🌟"
/// - All bytes copied (byte-by-byte copy)
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
/// - One byte at a time (no buffering)
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
///  // User selects "Hello, 世界!" in visual mode and presses 'y'
///  // Selection: bytes 100-120 (includes multi-byte characters)
///  // state.file_position_of_vis_select_start = 100
///  // state.file_position_of_vis_select_end = 120
///
/// let source = Path::new("/home/user/document.txt");
///
///  // Copy selection to clipboard
/// copy_selection_to_clipboardfile(state, source)?;
///
///  // Result:
///  // - File created: <session_dir>/clipboard/Hello
///  // - Contains UTF-8 bytes: "Hello, 世界!"
///  // - Multi-byte characters complete and uncorrupted
///  // - Can paste via Pasty mode
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
///  // Forward selection: bytes 10-20
/// is_in_selection(15, 10, 20) → true
/// is_in_selection(5, 10, 20) → false
///
///  // Backward selection: bytes 20-10
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
///  // 花 (U+82B1) = E8 8A B1 (3 bytes) at position 7
/// find_utf8_char_end(path, 7) → Ok(9)  // Last byte at position 9
///
///  // ASCII 'a' = 0x61 (1 byte) at position 5
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
    // =================================================
    // Debug-Assert, Test-Asset, Production-Catch-Handle
    // =================================================
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
///  Copy bytes 10 through 20 (inclusive) from source.txt
///  and append them to the end of target.txt
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
///  Append header (first 512 bytes)
/// append_bytes_from_file_to_file(source, 0, 511, output)?;
///
///  Append specific data section (bytes 1024-2047)
/// append_bytes_from_file_to_file(source, 1024, 2047, output)?;
///
///  Append footer (last 256 bytes, assuming we know the positions)
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

// TODO vec< is heap
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

/// Writes the complete navigation legend directly to terminal
///
/// ## Project Context
/// Displays all available keyboard commands for file navigation with
/// color-coded hotkeys. Each command section written independently for
/// maintainability - adding/removing commands requires no argument counting.
///
/// ## Memory: ZERO HEAP
/// All output written directly to terminal using buffy functions.
/// No intermediate String building, no heap allocation.
///
/// ## Operation
/// Writes legend in modular sections:
/// - Each command written separately via write_red_hotkey()
/// - Colors applied per-command (RED hotkey, YELLOW description)
/// - RESET applied at end
/// - Modular: Add/remove commands without affecting others
///
/// ## Safety & Error Handling
/// - Returns io::Result for write failures
/// - Each command write is independent
/// - Failure in one command doesn't affect others structurally
///
/// ## Legend Commands
/// - q: quit application
/// - sav: save current state (red and green and yellow)
/// - re: reload/refresh
/// - undo: undo last operation
/// - del: delete item
/// - nrm: normal mode
/// - ins: insert mode
/// - vis: visual mode
/// - hex: hex editor mode
/// - pasty: paste operation
/// - cvy: copy operation
/// - wrd,b,end: word navigation
/// - ///cmnt: comment operations (red and green and yellow)
/// - []idnt: indent operations
/// - hjkl: vim-style navigation
///
/// ## Example
/// ```rust
///  // In main display loop:
/// write_formatted_navigation_legend_to_tui()?;
/// ```
fn format_pasty_tui_legend() -> Result<()> {
    // File operations group
    write_red_hotkey("", "Have a Pasty!! ")?;
    // Three Colour
    // write_red_green_hotkey("s", "a", "v ")?;
    // Red only
    write_red_hotkey("b", "ack paste")?;
    write_red_hotkey("N", " ")?;

    // Mode operations group
    write_red_hotkey("str", "(any file-path) | ")?;
    write_red_hotkey("clear", " all | ")?;
    write_red_green_hotkey("clear", "N", " item ")?;
    // newline \n
    buffy_println("", &[])?;

    write_red_hotkey("Empty Enter", " Add Freshest Clipboard Item | ")?;

    write_red_hotkey("paste", " multi-line cut and paste")?;

    // Clear formatting: ANSI color codes are stateful
    // Make sure NEXT prints
    // are not also formatted.
    buffy_print("{}", &[BuffyFormatArg::Str(RESET)])?;

    // newline \n
    buffy_println("", &[])?;

    // Done
    Ok(())
}

/// Displays the Pasty info bar with count, pagination, and error messages.
/// Writes directly to stdout with zero heap allocation.
///
/// ## Project Context
/// Pasty clipboard manager info bar - shows total items, current view range,
/// navigation hints, and optional error/status messages. Each colored item
/// has its color code with it (not scattered in previous statements).
///
/// ## Memory: ZERO HEAP
/// All output written directly to terminal using stack-based formatting.
///
/// ## Parameters
/// - total_count: Total number of clipboard items
/// - first_count_visible: First item number currently displayed
/// - last_count_visible: Last item number currently displayed
/// - info_bar_message: Optional status/error message (empty string if none)
fn display_pasty_info_bar(
    total_count: usize,
    first_count_visible: usize,
    last_count_visible: usize,
    info_bar_message: &str,
) -> io::Result<()> {
    // =========================================================================
    // SECTION 1: RED total_count
    // =========================================================================
    buffy_print(
        "{}{}",
        &[BuffyFormatArg::Str(RED), BuffyFormatArg::Usize(total_count)],
    )?;

    // =========================================================================
    // SECTION 2: YELLOW " Clipboard Items, "
    // =========================================================================
    buffy_print("{} Clipboard Items, ", &[BuffyFormatArg::Str(YELLOW)])?;

    // =========================================================================
    // SECTION 3: YELLOW "Showing"
    // =========================================================================
    buffy_print("{}Showing ", &[BuffyFormatArg::Str(YELLOW)])?;

    // =========================================================================
    // SECTION 4: RED first_count_visible
    // =========================================================================
    buffy_print(
        "{}{}",
        &[
            BuffyFormatArg::Str(RED),
            BuffyFormatArg::Usize(first_count_visible),
        ],
    )?;

    // =========================================================================
    // SECTION 5: YELLOW "-"
    // =========================================================================
    buffy_print("{}-", &[BuffyFormatArg::Str(YELLOW)])?;

    // =========================================================================
    // SECTION 6: RED last_count_visible
    // =========================================================================
    buffy_print(
        "{}{}",
        &[
            BuffyFormatArg::Str(RED),
            BuffyFormatArg::Usize(last_count_visible),
        ],
    )?;

    // =========================================================================
    // SECTION 7: YELLOW " (Page up/down k/j) "
    // =========================================================================
    buffy_print("{} (Page up/down k/j) ", &[BuffyFormatArg::Str(YELLOW)])?;

    // =========================================================================
    // SECTION 8: YELLOW info_bar_message (if present)
    // =========================================================================
    if !info_bar_message.is_empty() {
        buffy_print(
            "{}{}",
            &[
                BuffyFormatArg::Str(YELLOW),
                BuffyFormatArg::Str(info_bar_message),
            ],
        )?;
    }

    // =========================================================================
    // SECTION 9: Newline + prompt text + RESET
    // =========================================================================
    buffy_print("\nEnter clipboard item #, 'paste', ", &[])?;

    buffy_print("or file-path to paste file text ", &[])?;

    buffy_print("{}> ", &[BuffyFormatArg::Str(RESET)])?;

    // =========================================================================
    // FINAL: Flush to ensure prompt appears immediately
    // =========================================================================
    io::stdout().flush()?;

    Ok(())
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
    println!("HELP MENU:");
    println!("    help            For a help menue with sections.)");
    println!("QUIT & SAVE:");
    println!("                    If you 'quit' without saving, your work is gone.)");
    println!("                    If session ends without 'quit' then a backup exists.");
    println!("    q               quit");
    println!("    wq              save and quit (same as 'write and quit')");
    println!("    s               save / write (same thing), (w alone is 'word' jump)");
    println!("MODES:");
    println!("    Memo Mode:      Run from home directory, Append-only quickie");
    println!("                    Creates dated files in ~/Documents/lines_editor/");
    println!("    Full Editor:    Run from any other directory");
    println!("    n               Normal-Mode (navigation)");
    println!("    i               Insert-Mode (type in text, delete previous)");
    println!("    ki              Keystroke Insert-Mode (type in text, delete previous)");
    println!("    v               Visual/Select-Mode (select and act on selections");
    println!("    hex             Hex Editor Mode");
    println!("    p | pasty       Clipboard / Paste Mode");
    println!("DELETE: d");
    println!("                 All delete operations can be undone/redone at char level");
    println!("    Normal Mode: 'd' deletes a WHOLE file-line");
    println!("    Insert Mode: delete-key for Backspace-Style Delete");
    println!("    Visual Mode  'd' deletes whole selection, not surrounding spaces/items");
    println!("                   then the cursor returns to line start, to re-sync");
    println!("    Visual & Normal: delete-key: deletes a single char backspace-style");

    println!("Resize-Tui: (Works with Enter-Key-to-Repeat");
    println!("    wide+           +1 wider");
    println!("    wide-           -1 wide");
    println!("    tall+           +1 taller");
    println!("    tall-           -1 tall");
    println!("NAVIGATION:");
    println!("    Esc | N         Normal Mode");
    println!("    hjkl            Move cursor");
    println!("    5j, 10l         Move with repeat count");
    println!("    [Empty Enter]   Repeat last command (Normal/Visual/ ...?)");
    println!("MOVE CURSOR: Normal-Mode move, Visual-Mode highlight");
    println!("                    Arrow keys (+ Enter) work too!");
    println!("    j               down");
    println!("    k               up");
    println!("    h               left");
    println!("    l               right");
    println!("    w               jump AHEAD to start of next word/symbol");
    println!("    e               jump AHEAD to end of this word/symbol");
    println!("    b               go BACK to beginning of this/next word/symbol");
    println!("GOTO:");
    println!("    g[int] =>       go to line number");
    println!("                     in Hex-Mode: Go To File Byte");
    println!("    gg     =>       go to start of file");
    println!("    ge | G =>       go to last line of file");
    println!("    gh | 0 =>       go to start of file");
    println!("    gl | $ =>       go to end of this line");
    println!("INDENT/UINDENT :");
    println!("    [               Indent");
    println!("    ]               Unindent");
    println!("COMMENT/UNCOMMENT:");
    println!("    /               Toggle Simple Comment (individual line(s))");
    println!("                     normal-mode or blocks in visual-mode)");
    println!("    //              Comment/Uncomment Block (visual-mode ");
    println!("                     include markers for Uncomment)");
    println!("    ///             Rust Doc-String Comment");
    println!("DELETE:");
    println!("                    Backspace key does not work with input buffer");
    println!("    d               Normal-Mode: like backspace");
    println!("                    Visual-Mode: removes selection");
    println!("    delete(key)     Only like backspace, not remove section");
    println!("UNDO/REDO:");
    println!("    u               undo");
    println!("    r               redo");
    println!("Cut/Past/Clipboard: Pasty!!");
    println!("    c | y           copy, yank (same thing)");
    println!("    v | p | pasty   go to Pasty-Mode (to paste)");
    println!("PASTEY MODE:");
    println!("    Enter           paste last copied/yanked item");
    println!("    [int]           clipboard items are numbered");
    println!("                     that number to past that item)");
    println!("    path            path to any other file to paste in");
    println!("    clear           clear whole clipboard");
    println!("    clear[int]      delete clipboard item by number");
    println!("    paste           to paste multi-line block from outside lines");
    println!("    b               go BACK");
    println!("HEX EDIT: Careful, Edit With The Safety!");
    println!("    hex         Enter hex-edit mode from Normal-Mode");
    println!("    [NN]            Enter two 'digit' hex number to change current byte");
    println!("                     this is standard hex-edit funcationality, in place");
    println!("    [NN]-i          *Insert* New Byte (byte-hex dash i)");
    println!("    d               Delete/Remove current byte");
    println!("    g[int]          Go To File Byte");
    println!("Examples in terminal/shell:");
    println!("  lines                Memo mode (if in home)");
    println!("  lines notes.txt      Create/open notes.txt");
    println!("  lines notes.txt:42   Open to line 42");
    println!("  lines mydir/ Create new file in directory");
}

/// Help section identifiers for menu navigation
///
/// Each variant represents a distinct help section that can be displayed
/// independently to fit within 80x24 terminal constraints.
#[derive(Debug, Clone, Copy, PartialEq)]
enum HelpSections {
    QuickStartBlurb,
    TopbarLegend,
    Navigation,
    HelpSectionGoto,
    HelpSectionCopyPasty,
    HelpSectionIndentComment,
    HelpSectionUndoRedo,
    HelpSectionHexEdit,
    HelpSectionDelete,
    // TerminalManagement,
}

/// Main help menu header text
///
/// Displayed at the top of the help menu selection screen
const HELP_MENU_HEADER: &str = r#"
  ╔═════════════════════════════════════════════════════╗
  ║   Lines  ->  a modal cli/terminal text/hex editor   ║
  ╚══════https://github.com/lineality/lines_editor══════╝
            get source code -> lines --source

   To use lines across multiple files, see File Fantastic
   https://github.com/lineality/file_fantastic
 "#;

/// Quick start and examples help section content
const HELP_SECTION_QUICK_START: &str = r#"
═══ QUICK START & EXAMPLES ═══     Press Enter to return to help menu
 USAGE in terminal:      ff [OPTIONS] [DIRECTORY]
 OPTIONS:   -h, --help            Show this help menu
            --source              Get ff source code, Rust 'crate'

 EXAMPLES for terminal/shell:
   lines                Memo mode (if in home)
   lines notes.txt      Create/open notes.txt
   lines notes.txt:42   Open to line 42
   lines mydir/ Create new file in directory

 BASIC WORKFLOW:
   1. Open or create a file:
    A. Create a new quick-memo file by simply running: lines
       simply type and press enter to append a line; q to quit
    B. Make a specific file by adding path: lines THIS/PATH
   2. Use modes (like vi) and the "+Enter" system to edit files.
   3. Use 'i'(+Enter) for insert mode to enter text
   4. Use 'v'(+Enter) to select and act on selections
   5. copy (c/y), paste & manage clipboard with 'pasty'
   6. Use hex-editor with 'hex' (in place, or insert or delete bytes)
   7. 'q' to quit"#;

const HELP_SECTION_TOPBAR_LEGEND: &str = r#"
"+Enter" Sytem: Press Enter after a command.
 ═══ THE LEGEND OF TOP-BAR ═══
quit sav re,undo del|nrm ins vis hex|go pasty cvy|wrd,b,end ///cmnt []idnt hjkl

 quit............. q for quit
 Save
     s               save / write (same thing), (w alone is 'word' jump)
     wq | sq         save and quit (same as 'write and quit')
     If you 'quit' without saving, your work is gone.)
 Undo/Redo........ u for undo, r for redo
 d................ delete with 'd' (also delete-key variation)
 Modes............ normal (n), insert(i), visual/select(v), hex-editor (hex)
 go...............'g' for go-to commands (see section for those)
 pasty,p.......... paste-content options (see section for that)
                   if already in visual-select mode, 'v' works for paste too
 wrd,b,end........ standard jump-cursor commands (see section for that)
 [,].............. standard indent/unindent keys
 /,//,///......... standard comment/uncomment + blocks (see section for that)
 h,j,k,l.......... standard movements, arrow keys work too

    Press Enter to return to help menu..."#;

/// Navigation commands help section content
const HELP_SECTION_NAVIGATION: &str = r#"
 ═══ NAVIGATION COMMANDS ═══

 NAVIGATION:
     Esc-key | N         Normal Mode
     hjkl            Move cursor
     5j, 10l         Move with repeat count
     [Empty Enter]   Repeat last command (Normal/Visual/ ...?)

MODES:
    Memo Mode:      Run from home directory, Append-only quickie
                    Creates dated files in ~/Documents/lines_editor/
    Full Editor:    Run from any other directory
    n               Normal-Mode (navigation)
    i               Insert-Mode (type in text, delete previous)
    ki              Keystroke Insert-Mode (type in text, del previous)
    v               Visual/Select-Mode (select and act on selections
    hex             Hex Editor Mode
    p | pasty       Clipboard / Paste Mode

  Press Enter to return to help menu..."#;

/// Sorting and filtering help section content
const HELP_SECTION_GOTO: &str = r#"
 ═══ Go To ═══

 NORMAL and Visual-Select Modes:
    g[int] =>       go to line number
                    in Hex-Mode: Go To File Byte
    gg     =>       go to start of file
    ge | G =>       go to last line of file
    gh | 0 =>       go to start of file
    gl | $ =>       go to end of this line

 HEX MODE:
    g[int] =>       in Hex-Mode: Go To File Byte

 OPEN FILE To Line: e.g. Open to line 42
     lines notes.txt:42

  Press Enter to return to help menu..."#;

/// Search options help section content
const HELP_SECTION_COPY_PASTY: &str = r#"
 ═══ COPY PASTE OPTIONS ═══

 Cut/Past/Clipboard: Pasty!!
     c | y           copy, yank (same thing)
     v | p | pasty   go to Pasty-Mode (to paste)
 PASTEY MODE:
     Enter           paste last copied/yanked item
     [int]           clipboard items are numbered
                      that number to past that item)
     path            path to any other file to paste in
     clear           clear whole clipboard
     clear[int]      delete clipboard item by number
     paste           to paste multi-line block from outside lines
     b               go BACK

 Press Enter to return to help menu... "#;

/// File operations help section content
const HELP_SECTION_INDENT_COMMENT: &str = r#"
 ═══ INDENT & COMMENT ═══

 Mode editor/IDE/Notebook systems use standard
   (shift +)   [,],/
 keys for toggle-indent and toggle/comment.
 Lines uses these (with +Enter instead of shift-key)

 Note: block-commenting with /* */ or """ """ is not toggled
 because uncomment must include the ~flag symbols.

 Visual-mode can single-line-comment multiple selected lines.

 INDENT/UINDENT :
     [               Indent
     ]               Unindent
 COMMENT/UNCOMMENT:
     /               Toggle Simple Comment (individual line(s))
                      normal-mode or blocks in visual-mode)
     //              Comment/Uncomment Block (visual-mode
                      include markers for Uncomment)
     ///             Rust Doc-String Comment

    Press  Enter to return to help menu... "#;

/// Get-Send Mode
const HELP_SECTION_UNDO_REDO_DELETE: &str = r#"
 ═══ GET-SEND MODE ═══

 DELETE:
                     Backspace key does not work with input buffer
     d               Normal-Mode: like backspace
                     Visual-Mode: removes selection
     delete(key)     Only like backspace, not remove section

Normal Mode:  'd': deletes a WHOLE file-line
               delete-key: deletes a single char, backspace style

Insert Mode:   delete-key only for Backspace-Style Delete

Visual Mode   'd': deletes a selected-selection inclusive
               delete-key: deletes a single char, backspace style

 UNDO/REDO:
     u               undo
     r               redo

 Press Enter to return to help menu..."#;

/// Get-Send Mode
const HELP_SECTION_HEX_EDIT: &str = r#"
  ═══ HEX EDIT ═══

  HEX EDIT: Careful, Edit With The Safety!
      hex         Enter hex-edit mode from Normal-Mode
      [NN]            Enter two 'digit' hex number to change current byte
                       this is standard hex-edit funcationality, in place
      [NN]-i          *Insert* New Byte (byte-hex dash i)
      d               Delete/Remove current byte
      g[int]          Go To File Byte

 Press Enter to return..."#;

/// Terminal management help section content
const HELP_SECTION_DELETE: &str = r#"
 ═══ DELETE ═══                  ...Press Enter to return
All delete operations can be undone/redone at char level.
'd' character command and 'delete' key commands are options,
there is no 'backspace-key' option. Backspace only operates
within the input-buffer (the characters you type BEFORE
+ Enter-key)

'd' Character Command:
    Normal Mode: 'd' deletes a WHOLE file-line
    Insert Mode: delete-key for Backspace-Style Delete
    Visual Mode  'd' deletes whole selection,
                not surrounding spaces/items
                then the cursor returns to line start, to re-sync

'delete' Key Command:
    To delete-back N spaces sequentially, use 'delete' + Enter
    repeating 'Enter' N times.
    For Visual-Select-Mode & Normal-Mode:
    The delete-key command deletes a single char backspace-style.

The 'backspace' key does not work to modify a file. 'backspace'
does work while you are tying a command, before hitting Enter."#;

//  ═══ PARTNER PROGRAMS CONFIGURATION ═══
//
//  You may want to call your own applications or other applications
//  that are not fully 'installed' on your system. "Partner Programs"
//  allows you to tell File Fantastic where these binary-executible
//  files are, wherever they are. Just list each file-path in this file,
//  which FF will create:
//
//  CONFIGURATION FILE:
//    ~/.ff_data/absolute_paths_to_local_partner_fileopening_executables.txt
//
//  FILE FORMAT:
//    - One program path per line
//    - Use absolute paths
//    - Comments with #, and blank lines, are ignored
//
//  EXAMPLE CONFIGURATION:
//    /usr/bin/emacs
//    # This is a comment
//    /home/user/bin/custom-editor
//
//  Press Enter to return to help menu... "#;

// TODO: is this using heap? improved version probably needed
/// Wait for user to press Enter key
///
/// Simple utility function to pause execution until the user
/// presses the Enter key. Used between help sections.
///
/// # Returns
/// * `Result<()>` - Ok when Enter pressed, Err on I/O error
fn wait_for_enter_keypress(stdin_handle: &mut StdinLock) -> Result<()> {
    let mut buffer = String::new();
    stdin_handle
        .read_line(&mut buffer)
        .map_err(LinesError::Io)?;
    Ok(())
}

/// Display the main help menu and handle section selection
///
/// This function presents the user with a numbered menu of help sections
/// and processes their selection. It returns to the caller when the user
/// chooses to quit.
///
/// # Returns
/// * `Result<()>` - Ok on successful completion, Err on I/O or other errors
///
/// # Errors
/// - I/O errors when reading user input
/// - Terminal display errors
pub fn display_help_menu_system(stdin_handle: &mut StdinLock) -> Result<()> {
    loop {
        // Clear screen for clean display
        clear_terminal_screen()?;

        // Display header with colors
        print!("{}{}", ansi_colors::BOLD, ansi_colors::BRIGHT_WHITE);
        println!("{}", HELP_MENU_HEADER);
        print!("{}", ansi_colors::RESET);

        // Quit instructions (...learning from the vim nightmare...)
        println!(
            "  {}q.{} Type 'q' & hit Enter to quit help menu / File Fantastic",
            ansi_colors::YELLOW,
            ansi_colors::RESET
        );
        println!();

        // Display menu options
        println!(
            "{} Select a help section:{}",
            ansi_colors::CYAN,
            ansi_colors::RESET
        );

        // Menu items with colored numbers
        println!(
            "  {}1.{} Quick Start & Examples",
            ansi_colors::MAGENTA,
            ansi_colors::RESET
        );
        println!(
            "  {}2.{} Top Bar Legend Tips",
            ansi_colors::MAGENTA,
            ansi_colors::RESET
        );
        println!(
            "  {}3.{} Navigation Commands",
            ansi_colors::MAGENTA,
            ansi_colors::RESET
        );
        println!(
            "  {}4.{} Go To (a file-line or start/end of a line)",
            ansi_colors::MAGENTA,
            ansi_colors::RESET
        );
        println!(
            "  {}5.{} Copy Paste & Clipboard",
            ansi_colors::MAGENTA,
            ansi_colors::RESET
        );
        println!(
            "  {}6.{} Indent & Unident Lines, Comment & Uncomment Lines",
            ansi_colors::MAGENTA,
            ansi_colors::RESET
        );
        println!(
            "  {}7.{} Undo / Redo",
            ansi_colors::MAGENTA,
            ansi_colors::RESET
        );
        println!(
            "  {}8.{} Hex-Editor: edit in place, insert, remove raw bytes",
            ansi_colors::MAGENTA,
            ansi_colors::RESET
        );
        println!("  {}9.{} Delete", ansi_colors::MAGENTA, ansi_colors::RESET);
        // println!(
        //     "  {}10.{} 'Partner Programs' Configuration",
        //     ansi_colors::MAGENTA,
        //     ansi_colors::RESET
        // );
        // println!(
        //     "  {}11.{} View help menu doc in editor (vi/nano)",
        //     ansi_colors::GREEN,
        //     ansi_colors::RESET
        // );
        println!();
        print!(
            "{}Enter section number (1-10) or 'q' to quit: {}",
            ansi_colors::BOLD,
            ansi_colors::RESET
        );

        // Flush to ensure prompt appears
        io::stdout().flush().map_err(LinesError::Io)?;

        //  // Read user input
        // let mut input = String::new();
        // io::stdin().read_line(&mut input).map_err(LinesError::Io)?;
        // let input = input.trim().to_lowercase();

        // Read user input using the passed-in lock instead of io::stdin()
        let mut input = String::new();
        stdin_handle.read_line(&mut input).map_err(LinesError::Io)?;
        let input = input.trim().to_lowercase();

        // Process user selection
        match input.as_str() {
            "1" => display_help_section_content(HelpSections::QuickStartBlurb, stdin_handle)?,
            "2" => display_help_section_content(HelpSections::TopbarLegend, stdin_handle)?,
            "3" => display_help_section_content(HelpSections::Navigation, stdin_handle)?,
            "4" => display_help_section_content(HelpSections::HelpSectionGoto, stdin_handle)?,
            "5" => display_help_section_content(HelpSections::HelpSectionCopyPasty, stdin_handle)?,
            "6" => {
                display_help_section_content(HelpSections::HelpSectionIndentComment, stdin_handle)?
            }
            "7" => display_help_section_content(HelpSections::HelpSectionUndoRedo, stdin_handle)?,
            "8" => display_help_section_content(HelpSections::HelpSectionHexEdit, stdin_handle)?,
            "9" => display_help_section_content(HelpSections::HelpSectionDelete, stdin_handle)?,
            // "10" => display_help_section_content(HelpSections::Configuration, stdin_handle)?,
            "q" | "quit" | "exit" => {
                println!(
                    "{}Exiting help system...{}",
                    ansi_colors::GREEN,
                    ansi_colors::RESET
                );
                return Ok(());
            }
            _ => {
                println!(
                    "{}Try again...Please enter 1-10 or 'q'.{}",
                    ansi_colors::YELLOW,
                    ansi_colors::RESET
                );
                wait_for_enter_keypress(stdin_handle)?;
            }
        }
    }
}

/// Clear the terminal screen using ANSI escape codes
///
/// This function uses ANSI escape sequences to clear the terminal
/// and reset the cursor to the top-left position.
///
/// # Returns
/// * `Result<()>` - Ok on success, Err on I/O error
fn clear_terminal_screen() -> Result<()> {
    // ANSI escape codes: clear screen and move cursor to top-left
    print!("\x1b[2J\x1b[1;1H");
    io::stdout().flush().map_err(LinesError::Io)?;
    Ok(())
}

/// ANSI color codes for terminal formatting
///
/// These constants provide color and style formatting for terminal output.
/// Using ANSI escape sequences for maximum compatibility.
mod ansi_colors {
    /// Reset all formatting to default
    pub const RESET: &str = "\x1b[0m";

    /// Bold text for headers
    pub const BOLD: &str = "\x1b[1m";

    /// Cyan color for commands
    pub const CYAN: &str = "\x1b[36m";

    /// Green color for examples
    pub const GREEN: &str = "\x1b[32m";

    /// Yellow color for warnings or important notes
    pub const YELLOW: &str = "\x1b[33m";

    /// Bright white for emphasis
    pub const BRIGHT_WHITE: &str = "\x1b[97m";

    /// Magenta for section numbers
    pub const MAGENTA: &str = "\x1b[35m";
}

/// Display a specific help section with proper formatting
///
/// This function clears the screen and displays the content for the
/// selected help section, waiting for user input before returning.
///
/// # Arguments
/// * `section` - The help section to display
///
/// # Returns
/// * `Result<()>` - Ok on successful display, Err on I/O errors
fn display_help_section_content(section: HelpSections, stdin_handle: &mut StdinLock) -> Result<()> {
    clear_terminal_screen()?;

    // Select and display appropriate section content
    let content = match section {
        HelpSections::QuickStartBlurb => HELP_SECTION_QUICK_START,
        HelpSections::TopbarLegend => HELP_SECTION_TOPBAR_LEGEND,
        HelpSections::Navigation => HELP_SECTION_NAVIGATION,
        HelpSections::HelpSectionGoto => HELP_SECTION_GOTO,
        HelpSections::HelpSectionCopyPasty => HELP_SECTION_COPY_PASTY,
        HelpSections::HelpSectionIndentComment => HELP_SECTION_INDENT_COMMENT,
        HelpSections::HelpSectionUndoRedo => HELP_SECTION_UNDO_REDO_DELETE,
        HelpSections::HelpSectionHexEdit => HELP_SECTION_HEX_EDIT,
        HelpSections::HelpSectionDelete => HELP_SECTION_DELETE,
        // HelpSections::Configuration => HELP_SECTION_CONFIGURATION,
    };

    // Display with color formatting
    print!("{}{}", ansi_colors::BOLD, ansi_colors::CYAN);
    println!("{}", content);
    print!("{}", ansi_colors::RESET);

    // Wait for user to read
    wait_for_enter_keypress(stdin_handle)?;

    Ok(())
}

/// Formats the bottom info bar with current editor state.
///
/// # Purpose
/// Shows critical state on ONE line: mode, position, filename, file byte, and
/// the pending info message.
///
/// # Position Reporting (file-grounded, not TUI/visual)
/// Both numbers come from `get_row_col_file_position`, the single source of
/// truth, NOT from `cursor.tui_visual_col` (which is a VISUAL TUI column under Option A
/// and would mix units with the character-based scroll offset):
///   - "line:N"  → N is the byte offset WITHIN the line (`byte_in_line`); for a
///                 multibyte character this is that character's START byte.
///   - "@M"      → M is the absolute file byte
///                 (`byte_offset_linear_file_absolute_position`).
/// If the cursor is not on a resolvable cell, both show "n/a".
///
/// # Coordinate Spaces (see the module "Coordinate Spaces" reference)
/// Reports FILE-GROUNDED numbers only (never #4/#5 TUI abstractions):
/// - "line N"  : #3 line number (shown +1 for humans)
/// - ":B"      : #2 in-line byte (a multibyte char's START byte)
/// - "@M"      : #1 file byte
/// All three come from one `get_row_col_file_position(#6 tui_row, #5 tui_visual_col)`.
///
/// # Arguments
/// * `lines_editor_state` - Current editor state
///
/// # Returns
/// * `Ok(String)` - Formatted info bar string
/// * `Err(LinesError)` - If formatting fails
fn format_info_bar_cafe_normal_visualselect(lines_editor_state: &EditorState) -> Result<String> {
    // Mode string
    let mode_str = match lines_editor_state.mode {
        EditorMode::Normal => "NORMAL",
        EditorMode::Insert => "INSERT",
        EditorMode::KeystrokeInputMode => "KEY-INSRT",
        EditorMode::VisualSelectMode => "VISUAL",
        EditorMode::PastyMode => "PASTY",
        EditorMode::HexMode => "HEX",
    };

    // Line number (1-indexed for display).
    let line_display =
        lines_editor_state.line_count_at_top_of_window + lines_editor_state.cursor.tui_row + 1;

    // Filename (or a placeholder if none).
    let filename = lines_editor_state
        .original_file_path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unmanned file");

    // Pending info message (up to the NUL terminator, or full buffer).
    let message_len = lines_editor_state
        .info_bar_message_buffer
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(lines_editor_state.info_bar_message_buffer.len());

    let message_for_infobar =
        std::str::from_utf8(&lines_editor_state.info_bar_message_buffer[..message_len])
            .unwrap_or(""); // Empty string if invalid UTF-8

    // Resolve the cursor's file position ONCE. Both reported numbers are
    // file-grounded (see the Position Reporting note in this function's docs):
    //   in_line_byte_string      → byte offset within the line (start byte)
    //   file_position_string     → absolute file byte
    let (in_line_byte_string, file_position_string) = match lines_editor_state
        .get_row_col_file_position(
            lines_editor_state.cursor.tui_row,
            lines_editor_state.cursor.tui_visual_col,
        ) {
        Ok(Some(row_col_file_pos)) => (
            row_col_file_pos.byte_in_line.to_string(),
            row_col_file_pos
                .byte_offset_linear_file_absolute_position
                .to_string(),
        ),
        _ => ("n/a".to_string(), "n/a".to_string()),
    };

    // Build the info bar (no-heap formatter).
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
            &in_line_byte_string,
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
    let _ = write_formatted_navigation_legend_to_tui()?;

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
    let mut hex_line = String::with_capacity(DEFAULT_COLS);
    // 26 bytes × 3 chars per UTF-8 display ("H  ") = 78 chars + safety margin
    let mut utf8_line = String::with_capacity(DEFAULT_COLS);

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

// ============================================================================
// UTF-8 CHARACTER ANALYSIS (Helper for Multi-byte Character Handling)
// ============================================================================

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
/// Reads file in N-byte chunks to avoid loading entire file.
/// Bounded by file size to prevent infinite loops.
///
/// # Memory Safety
/// - Pre-allocated N-byte buffer (no dynamic allocation)
/// - Bounded iteration (stops at EOF)
/// - Returns position, not reference (no lifetime issues)
fn find_next_newline(
    file_path: &PathBuf,
    start_offset: usize,
    file_size: usize,
) -> io::Result<Option<usize>> {
    const SEARCH_CHUNK_SIZE: usize = 32;
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
/// Reads file in N-byte chunks backward from cursor position.
/// Stops at byte 0 (file start).
///
/// # Memory Safety
/// - Pre-allocated N-byte buffer
/// - Bounded iteration (stops at offset 0)
/// - Underflow protection (checked subtraction)
fn find_previous_newline(file_path: &PathBuf, start_offset: usize) -> io::Result<Option<usize>> {
    const SEARCH_CHUNK_SIZE: usize = 32;
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

    let string_lines = &lines_editor_state
        .hex_cursor
        .byte_offset_linear_file_absolute_position
        + 1;

    let info_bar = stack_format_it(
        "{}HEX byte {}{}{} of {}{}{} {}, Edit:Enter Hex|Insrt:NN-i|GoTo:gN|d {} {}> ",
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

/// Renders the complete UTF8-text TUI to terminal: legend + content + info bar.
///
/// # Purpose (Project Context)
/// This is the top-level rendering function for the TUI text editor.
/// It displays the minimal 3-section interface and is called once per
/// screen refresh (after each user action or resize event).
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
/// # Rendering Pipeline
/// This function orchestrates three distinct output phases:
///
/// 1. **Legend** (top line): Static navigation help, written by
///    write_formatted_navigation_legend_to_tui().
///
/// 2. **Content** (middle rows): Each row is rendered in two parts:
///    - Line number prefix: Written by buffy_print() with LINE_NUMBER_STYLE
///      (green). This is the "1 ", "2 ", etc. at the start of each line.
///    - Content portion: Written directly to stdout by
///      render_utf8txt_row_with_cursor(), which applies cursor highlighting
///      (PRIORITY 1), visual selection highlighting (PRIORITY 2), syntax
///      highlighting (PRIORITY 3, if not a plain text file), or no styling
///      (PRIORITY 4). This function writes bytes directly — no intermediate
///      String is built or returned.
///
/// 3. **Info bar** (bottom line): Mode, position, filename, command input.
///    Written by format_info_bar_cafe_normal_visualselect().
///
/// # Syntax Highlighting Decision
/// The file extension is checked ONCE before the row loop using
/// buffy_is_plain_text_extension(). If the file is .txt or .log, syntax
/// highlighting is skipped entirely for all rows. Otherwise, each character
/// in each row is checked for symbol/keyword highlighting during rendering.
///
/// # Cursor Column Adjustment
/// state.cursor.tui_visual_col is in full-row coordinates (including line number
/// prefix characters like "42 "). render_utf8txt_row_with_cursor() receives
/// only the content portion of each row (prefix stripped), so the cursor
/// column must be adjusted by subtracting line_num_width. Saturating
/// subtraction prevents underflow if the cursor is somehow in the prefix area.
///
/// # Memory: Zero Heap in Rendering Path
/// - Line number: Written via buffy_print (stack-only)
/// - Content: Written via stdout.write_all inside render_utf8txt_row_with_cursor
///   (no String, no Vec<char>)
/// - Legend and info bar: Their own rendering functions
/// - is_plain_text: bool computed once, stack
///
/// # Arguments
/// * `state` - Current editor state with display buffers, cursor position,
///             mode, window_map, file path, and all rendering state.
///
/// # Returns
/// * `Ok(())` - Successfully rendered all three sections
/// * `Err(LinesError)` - Display operation failed (write error, window_map
///                        error, or selection calculation error)
///
/// # Coordinate Spaces (see the module "Coordinate Spaces" reference)
/// Computes `content_cursor_col = cursor.tui_visual_col - line_num_width`
/// (#5 full → #5 content-relative) before calling render_utf8txt_row_with_cursor.
///
/// # Error Handling
/// All errors from sub-functions are propagated via `?`. No silent failures.
/// If stdout flush fails, the error is wrapped in LinesError::DisplayError
/// with a unique prefix "render_tui: flush" for tracing.
///
/// # Design Goals
/// - Only 2 non-content lines (legend + info bar)
/// - No wasted space, no filler lines
/// - All essential info visible at all times
/// - Clean, minimal aesthetic
/// - Zero heap allocation in the rendering hot path
pub fn render_tui_utf8txt(state: &EditorState) -> Result<()> {
    // =========================================================================
    // CLEAR SCREEN
    // =========================================================================
    // Move cursor to top-left and clear entire screen.
    // This is a single write of static bytes — no allocation.
    print!("\x1B[2J\x1B[H");
    io::stdout().flush().map_err(|e| {
        LinesError::DisplayError(stack_format_it(
            "render_tui: flush clear: {}",
            &[&e.to_string()],
            "render_tui: flush clear",
        ))
    })?;

    // =========================================================================
    // TOP LINE: NAVIGATION LEGEND
    // =========================================================================
    // Static hotkey reference line. Written once per refresh.
    let _ = write_formatted_navigation_legend_to_tui()?;

    // =========================================================================
    // SYNTAX HIGHLIGHTING: PLAIN TEXT CHECK (computed once for all rows)
    // =========================================================================
    // Check the file extension to decide if syntax highlighting applies.
    // .txt and .log files are plain text: no keyword/symbol colouring.
    // Everything else (including unknown/no extension) gets highlighting.
    //
    // Computed once here rather than per-row or per-character to avoid
    // redundant path inspection on every iteration.
    //
    // state.original_file_path is Option<PathBuf>.
    // .as_deref() converts Option<PathBuf> to Option<&Path> (no allocation).
    let is_plain_text = buffy_is_plain_text_extension(state.original_file_path.as_deref());

    // =========================================================================
    // MIDDLE: FILE CONTENT WITH CURSOR, SELECTION, AND SYNTAX HIGHLIGHTING
    // =========================================================================
    // Each row in the display buffer is rendered in two parts:
    //
    //   1. Line number prefix  →  buffy_print with green styling
    //   2. Content portion     →  render_utf8txt_row_with_cursor (direct write)
    //
    // The line number prefix is computed by calculate_line_number_width()
    // and written BEFORE calling the content renderer. The content renderer
    // receives only the content portion (prefix stripped) and writes it
    // directly to stdout. A newline is written after each row.
    //
    // Empty rows (display_utf8txt_buffer_lengths[row] == 0) get either:
    //   - A cursor block character if the cursor is on this row
    //   - A blank line otherwise
    for row in 0..state.effective_rows {
        if state.display_utf8txt_buffer_lengths[row] > 0 {
            // =================================================================
            // NON-EMPTY ROW: Has content in display buffer
            // =================================================================
            let row_content =
                &state.utf8_txt_display_buffers[row][..state.display_utf8txt_buffer_lengths[row]];

            match std::str::from_utf8(row_content) {
                Ok(row_str) => {
                    // ---------------------------------------------------------
                    // SPLIT: Line number prefix vs content
                    // ---------------------------------------------------------
                    // calculate_line_number_width returns the byte length of
                    // the line number prefix (e.g. "42 " = 3 bytes).
                    // All line numbers are ASCII digits + space, so
                    // byte width == character width for the prefix.
                    let line_num_width = calculate_line_number_width(
                        state.line_count_at_top_of_window,
                        state.cursor.tui_row,
                        state.effective_rows,
                    );

                    // Defensive: ensure line_num_width does not exceed row_str
                    let line_num_width = line_num_width.min(row_str.len());

                    let line_num_part = &row_str[..line_num_width];
                    let content_part = &row_str[line_num_width..];

                    // ---------------------------------------------------------
                    // WRITE LINE NUMBER PREFIX (green)
                    // ---------------------------------------------------------
                    // Written via buffy_print: zero heap, direct to stdout.
                    buffy_print(
                        "{}",
                        &[BuffyFormatArg::StrStyled(line_num_part, LINE_NUMBER_STYLE)],
                    )?;

                    // ---------------------------------------------------------
                    // CURSOR COLUMN ADJUSTMENT
                    // ---------------------------------------------------------
                    // state.cursor.tui_visual_col is in full-row coordinates
                    // (including line number prefix characters).
                    //
                    // render_utf8txt_row_with_cursor receives the content
                    // portion only (prefix stripped), so the cursor column
                    // must be adjusted by subtracting line_num_width.
                    //
                    // saturating_sub prevents underflow if cursor.tui_visual_col
                    // is somehow less than line_num_width (cursor in the
                    // line number prefix area — should not happen in normal
                    // operation, but handled defensively).
                    let content_cursor_col =
                        state.cursor.tui_visual_col.saturating_sub(line_num_width);

                    // ---------------------------------------------------------
                    // WRITE CONTENT WITH HIGHLIGHTING (direct to stdout)
                    // ---------------------------------------------------------
                    // render_utf8txt_row_with_cursor writes each character
                    // directly to stdout with appropriate ANSI styling.
                    // It returns Result<()>, not a String.
                    //
                    // Priority order inside the function:
                    //   1. Cursor (BOLD RED BG_WHITE)
                    //   2. Visual selection (BOLD YELLOW BG_CYAN)
                    //   3. Syntax highlighting (cyan symbols, yellow keywords)
                    //   4. Plain character (no ANSI codes)
                    render_utf8txt_row_with_cursor(
                        state,
                        row,
                        content_part,
                        content_cursor_col,
                        is_plain_text,
                    )?;

                    // ---------------------------------------------------------
                    // NEWLINE AFTER ROW
                    // ---------------------------------------------------------
                    // render_utf8txt_row_with_cursor does NOT write a newline.
                    // The caller (here) is responsible for line termination.
                    // buffy_println with empty template writes just "\n" + flush.
                    buffy_println("", &[])?;
                }
                Err(_) => {
                    // UTF-8 decode failure for this row's display buffer.
                    // Show replacement character and continue rendering
                    // remaining rows. Do not halt for one bad row.
                    buffy_println("�", &[])?;
                }
            }
        } else {
            // =================================================================
            // EMPTY ROW: No content in display buffer
            // =================================================================
            // If the cursor is on this empty row, show a visible cursor block
            // so the user knows where they are. Otherwise, blank line.
            if row == state.cursor.tui_row {
                buffy_println("{}", &[BuffyFormatArg::CharStyled('█', CURSOR_BLOCK_STYLE)])?;
            } else {
                buffy_println("", &[])?;
            }
        }
    }

    // =========================================================================
    // BOTTOM LINE: INFO BAR
    // =========================================================================
    // Shows current mode, cursor position, filename, and command input.
    // Written as the final line with no trailing newline (cursor stays on
    // the info bar for command input visibility).
    let info_bar = format_info_bar_cafe_normal_visualselect(state)?;
    buffy_print(&info_bar, &[])?;

    // =========================================================================
    // FINAL FLUSH
    // =========================================================================
    // Ensure all buffered output reaches the terminal before returning.
    // Without this flush, the screen may appear partially rendered.
    io::stdout().flush().map_err(|e| {
        LinesError::DisplayError(stack_format_it(
            "render_tui: flush final: {}",
            &[&e.to_string()],
            "render_tui: flush final",
        ))
    })?;

    Ok(())
}

/// Renders one row of display directly to stdout with cursor, selection,
/// and syntax highlighting — zero heap allocation.
///
/// # Purpose (Project Context)
/// Character-by-character renderer for the TUI content area. It writes
/// ANSI-styled bytes directly to stdout as it walks the row; no intermediate
/// String is built. It applies, in strict priority:
///   PRIORITY 1: Cursor (BOLD + RED + WHITE_BG)
///   PRIORITY 2: Visual selection (BOLD + YELLOW + CYAN_BG)
///   PRIORITY 3: Syntax highlighting (cyan symbols, yellow keywords)
///   PRIORITY 4: Tab glyph (blue arrow)
///   PRIORITY 5: Plain character (default green)
///
/// # Byte / Visual coordinate tracking (Option A)
/// `cursor.tui_visual_col` is a VISUAL column — a count of terminal CELLS — under the
/// project's Option A decision. A double-width character (CJK/emoji) occupies
/// TWO cells but is ONE character. The caller passes `cursor_col` already
/// adjusted to a VISUAL content column (full visual `tui_visual_col` minus the
/// line-number prefix width). This function therefore maintains:
///
///   - `byte_pos`:   byte offset into `row_content`; advances 1-4 bytes per
///                   character. Used for slicing, syntax prefix matching, and
///                   writing bytes.
///   - `visual_col`: VISUAL column (cells) consumed so far; advances by the
///                   character's display width (1 for ASCII/normal, 2 for
///                   double-width). Compared against `cursor_col` to place the
///                   cursor block, exactly mirroring how
///                   get_row_col_file_position walks visual width.
///
/// The cursor block is drawn on the character whose visual span
/// `[visual_col, visual_col + width)` CONTAINS `cursor_col` (snap-to-containing;
/// the same rule the lookup uses, so block placement and file position agree).
///
/// # Why visual, not character
/// With character counting, a `cursor_col` of (say) 71 on a line whose first 69
/// visible characters span 72 visual cells (three double-width chars) never
/// matches any character index and falls through to the end-of-line block,
/// painting the cursor past the line. Walking visual width fixes this at the
/// source and keeps the block in lockstep with the resolved file byte.
///
/// # Direct-Write Pattern (No Heap)
/// Writes ANSI codes and character bytes via stdout.write_all(). No String
/// accumulation, no Vec<char>, no format!() macro.
///
/// # Coordinate Spaces (see the module "Coordinate Spaces" reference)
/// - In  `row_index`  : #6 TUI display row
/// - In  `cursor_col` : #5 VISUAL cell column, CONTENT-RELATIVE (caller already
///                      subtracted the prefix width). The loop accumulates #5
///                      visual cells and places the cursor where they match.
///
/// # Arguments
/// * `state`          - Editor state (mode, cursor position)
/// * `row_index`      - Display row being rendered (0-indexed within window)
/// * `row_content`    - Content portion of the row (line-number prefix already
///                      excluded by the caller)
/// * `cursor_col`     - VISUAL content column (caller subtracts the prefix
///                      width from the visual `state.cursor.tui_visual_col`)
/// * `is_plain_text`  - If true, skip syntax highlighting entirely
///
/// # Returns
/// * `Ok(())` - Row content written to stdout successfully
/// * `Err(LinesError)` - On lookup, selection, or stdout write failure
///
/// # Error Handling
/// All write and lookup failures are propagated; never panics in production.
fn render_utf8txt_row_with_cursor(
    state: &EditorState,
    row_index: usize,
    row_content: &str,
    cursor_col: usize,
    is_plain_text: bool,
) -> Result<()> {
    let mut stdout = io::stdout();
    let row_bytes = row_content.as_bytes();
    let row_len = row_bytes.len();

    // =========================================================================
    // CURSOR ON THIS ROW?
    // =========================================================================
    let cursor_on_this_row = row_index == state.cursor.tui_row;

    // =========================================================================
    // TOTAL VISUAL WIDTH (for cursor-at/past-end-of-line detection)
    // =========================================================================
    // cursor_col is a VISUAL content column, so end-of-line detection and the
    // clamp below are measured in VISUAL cells (double-width chars count 2).
    let mut total_visual_width: usize = 0;
    for ch in row_content.chars() {
        total_visual_width += if double_width::is_double_width(ch) {
            2
        } else {
            1
        };
    }

    // Defensive clamp: cursor cannot be drawn beyond the row's visual extent.
    let effective_cursor_col = cursor_col.min(total_visual_width);

    // =========================================================================
    // MAIN LOOP: iterate UTF-8 character boundaries, tracking byte_pos and the
    // VISUAL column. (No character-index counter is needed: cursor placement is
    // purely visual under Option A.)
    // =========================================================================
    let mut byte_pos: usize = 0;
    let mut visual_col: usize = 0;

    // Safety bound: never more characters than bytes.
    let max_iterations = row_len + 1;
    let mut iterations: usize = 0;

    while byte_pos < row_len {
        iterations += 1;
        if iterations > max_iterations {
            #[cfg(debug_assertions)]
            eprintln!(
                "render_utf8txt_row_with_cursor: iteration limit reached at byte_pos={}, visual_col={}",
                byte_pos, visual_col
            );
            break;
        }

        // ---- character byte length from the UTF-8 lead byte ----
        let char_byte_len = if byte_pos < row_len {
            let lead = row_bytes[byte_pos];
            if lead < 0x80 {
                1
            } else if lead < 0xE0 {
                2
            } else if lead < 0xF0 {
                3
            } else if lead < 0xF8 {
                4
            } else {
                1 // malformed lead byte; advance 1 to avoid an infinite loop
            }
        } else {
            break;
        };

        // ---- bounds: do not read past the end of the row ----
        let char_end = byte_pos + char_byte_len;
        let char_end = if char_end > row_len {
            #[cfg(debug_assertions)]
            eprintln!(
                "render_utf8txt_row_with_cursor: incomplete UTF-8 at byte_pos={}, need {} bytes, have {}",
                byte_pos,
                char_byte_len,
                row_len - byte_pos
            );
            stdout.write_all("�".as_bytes()).map_err(|e| {
                LinesError::DisplayError(stack_format_it(
                    "rURWC write error: {}",
                    &[&e.to_string()],
                    "rURWC write error",
                ))
            })?;
            break;
        } else {
            char_end
        };

        let char_bytes = &row_bytes[byte_pos..char_end];

        // ---- VISUAL width of THIS character (1 or 2 cells) ----
        let display_width = if char_byte_len == 1 {
            1
        } else {
            match std::str::from_utf8(char_bytes) {
                Ok(s) => match s.chars().next() {
                    Some(ch) => {
                        if double_width::is_double_width(ch) {
                            2
                        } else {
                            1
                        }
                    }
                    None => 1,
                },
                Err(_) => 1,
            }
        };

        // =====================================================================
        // PRIORITY 1: CURSOR — visual span-contains (snap-to-containing)
        // =====================================================================
        if cursor_on_this_row
            && effective_cursor_col >= visual_col
            && effective_cursor_col < visual_col + display_width
        {
            stdout.write_all(BOLD_U8).map_err(|e| {
                LinesError::DisplayError(stack_format_it(
                    "rURWC cursor write: {}",
                    &[&e.to_string()],
                    "rURWC cursor write",
                ))
            })?;
            stdout.write_all(RED_U8).map_err(|e| {
                LinesError::DisplayError(stack_format_it(
                    "rURWC cursor write: {}",
                    &[&e.to_string()],
                    "rURWC cursor write",
                ))
            })?;
            stdout.write_all(BG_WHITE_U8).map_err(|e| {
                LinesError::DisplayError(stack_format_it(
                    "rURWC cursor write: {}",
                    &[&e.to_string()],
                    "rURWC cursor write",
                ))
            })?;
            stdout.write_all(char_bytes).map_err(|e| {
                LinesError::DisplayError(stack_format_it(
                    "rURWC cursor write: {}",
                    &[&e.to_string()],
                    "rURWC cursor write",
                ))
            })?;
            stdout.write_all(RESET_U8).map_err(|e| {
                LinesError::DisplayError(stack_format_it(
                    "rURWC cursor write: {}",
                    &[&e.to_string()],
                    "rURWC cursor write",
                ))
            })?;

            byte_pos = char_end;
            visual_col += display_width;
            continue;
        }

        // =====================================================================
        // PRIORITY 2: VISUAL SELECTION
        // =====================================================================
        if state.mode == EditorMode::VisualSelectMode {
            let line_num_width = calculate_line_number_width(
                state.line_count_at_top_of_window,
                state.cursor.tui_row,
                state.effective_rows,
            );
            // get_row_col_file_position expects a VISUAL column (Option A).
            let map_col = visual_col + line_num_width;

            let file_pos_option = state.get_row_col_file_position(row_index, map_col)?;

            if let Some(file_pos) = file_pos_option {
                let in_selection = is_in_selection(
                    file_pos.byte_offset_linear_file_absolute_position,
                    state.file_position_of_vis_select_start,
                    state.file_position_of_vis_select_end,
                )?;

                if in_selection {
                    stdout.write_all(BOLD_U8).map_err(|e| {
                        LinesError::DisplayError(stack_format_it(
                            "rURWC sel write: {}",
                            &[&e.to_string()],
                            "rURWC sel write",
                        ))
                    })?;
                    stdout.write_all(YELLOW_U8).map_err(|e| {
                        LinesError::DisplayError(stack_format_it(
                            "rURWC sel write: {}",
                            &[&e.to_string()],
                            "rURWC sel write",
                        ))
                    })?;
                    stdout.write_all(BG_CYAN_U8).map_err(|e| {
                        LinesError::DisplayError(stack_format_it(
                            "rURWC sel write: {}",
                            &[&e.to_string()],
                            "rURWC sel write",
                        ))
                    })?;
                    stdout.write_all(char_bytes).map_err(|e| {
                        LinesError::DisplayError(stack_format_it(
                            "rURWC sel write: {}",
                            &[&e.to_string()],
                            "rURWC sel write",
                        ))
                    })?;
                    stdout.write_all(RESET_U8).map_err(|e| {
                        LinesError::DisplayError(stack_format_it(
                            "rURWC sel write: {}",
                            &[&e.to_string()],
                            "rURWC sel write",
                        ))
                    })?;

                    byte_pos = char_end;
                    visual_col += display_width;
                    continue;
                }
            }
        }

        // =====================================================================
        // PRIORITY 3: SYNTAX HIGHLIGHTING
        // =====================================================================
        if !is_plain_text {
            let highlight = buffy_get_syntax_highlight(byte_pos, row_content);

            match highlight {
                SyntaxHighlight::SyntaxSymbol => {
                    // Single symbol character in colour.
                    stdout.write_all(SYMBOL_COLOUR).map_err(|e| {
                        LinesError::DisplayError(stack_format_it(
                            "rURWC syn write: {}",
                            &[&e.to_string()],
                            "rURWC syn write",
                        ))
                    })?;
                    stdout.write_all(char_bytes).map_err(|e| {
                        LinesError::DisplayError(stack_format_it(
                            "rURWC syn write: {}",
                            &[&e.to_string()],
                            "rURWC syn write",
                        ))
                    })?;
                    stdout.write_all(RESET_U8).map_err(|e| {
                        LinesError::DisplayError(stack_format_it(
                            "rURWC syn write: {}",
                            &[&e.to_string()],
                            "rURWC syn write",
                        ))
                    })?;

                    byte_pos = char_end;
                    visual_col += display_width;
                    continue;
                }

                SyntaxHighlight::DefinitionWord { keyword_byte_len } => {
                    // Multi-character keyword in yellow. Computed spans are in
                    // VISUAL cells so the cursor-overlap test agrees with the
                    // visual cursor column.
                    let keyword_end_byte = (byte_pos + keyword_byte_len).min(row_len);
                    let keyword_slice = &row_content[byte_pos..keyword_end_byte];

                    // Visual width of the keyword span (keywords are ASCII, so
                    // this equals the character count, but we sum widths
                    // if that ever changes).
                    let mut keyword_visual_width: usize = 0;
                    for ch in keyword_slice.chars() {
                        keyword_visual_width += if double_width::is_double_width(ch) {
                            2
                        } else {
                            1
                        };
                    }

                    // Does the visual cursor column fall inside this keyword?
                    let cursor_in_keyword = if cursor_on_this_row {
                        let keyword_visual_end = visual_col + keyword_visual_width;
                        effective_cursor_col >= visual_col
                            && effective_cursor_col < keyword_visual_end
                    } else {
                        false
                    };

                    if !cursor_in_keyword {
                        // No cursor conflict: write the whole keyword in yellow.
                        let keyword_bytes = &row_bytes[byte_pos..keyword_end_byte];

                        stdout.write_all(DEFINITION_COLOUR).map_err(|e| {
                            LinesError::DisplayError(stack_format_it(
                                "rURWC kw write: {}",
                                &[&e.to_string()],
                                "rURWC kw write",
                            ))
                        })?;
                        stdout.write_all(keyword_bytes).map_err(|e| {
                            LinesError::DisplayError(stack_format_it(
                                "rURWC kw write: {}",
                                &[&e.to_string()],
                                "rURWC kw write",
                            ))
                        })?;
                        stdout.write_all(RESET_U8).map_err(|e| {
                            LinesError::DisplayError(stack_format_it(
                                "rURWC kw write: {}",
                                &[&e.to_string()],
                                "rURWC kw write",
                            ))
                        })?;

                        byte_pos = keyword_end_byte;
                        visual_col += keyword_visual_width;
                        continue;
                    }

                    // Cursor IS inside the keyword: write only this first
                    // character (in yellow); a later iteration lands the cursor
                    // character on PRIORITY 1.
                    stdout.write_all(YELLOW_U8).map_err(|e| {
                        LinesError::DisplayError(stack_format_it(
                            "rURWC kw partial: {}",
                            &[&e.to_string()],
                            "rURWC kw partial",
                        ))
                    })?;
                    stdout.write_all(char_bytes).map_err(|e| {
                        LinesError::DisplayError(stack_format_it(
                            "rURWC kw partial: {}",
                            &[&e.to_string()],
                            "rURWC kw partial",
                        ))
                    })?;
                    stdout.write_all(RESET_U8).map_err(|e| {
                        LinesError::DisplayError(stack_format_it(
                            "rURWC kw partial: {}",
                            &[&e.to_string()],
                            "rURWC kw partial",
                        ))
                    })?;

                    byte_pos = char_end;
                    visual_col += display_width;
                    continue;
                }

                SyntaxHighlight::None => {
                    // Fall through to PRIORITY 4 / 5 below.
                }
            }
        }

        // =====================================================================
        // PRIORITY 4: TAB CHARACTER — blue visible glyph (single cell)
        // =====================================================================
        // Rendered as a blue → glyph (TAB_GLYPH), which is one visual cell, so
        // visual_col advances by display_width (== 1 for the single-byte tab).
        if char_bytes == b"\t" {
            stdout.write_all(TAB_COLOUR).map_err(|e| {
                LinesError::DisplayError(stack_format_it(
                    "rURWC tab write: {}",
                    &[&e.to_string()],
                    "rURWC tab write",
                ))
            })?;
            stdout.write_all(TAB_GLYPH).map_err(|e| {
                LinesError::DisplayError(stack_format_it(
                    "rURWC tab write: {}",
                    &[&e.to_string()],
                    "rURWC tab write",
                ))
            })?;
            stdout.write_all(RESET_U8).map_err(|e| {
                LinesError::DisplayError(stack_format_it(
                    "rURWC tab write: {}",
                    &[&e.to_string()],
                    "rURWC tab write",
                ))
            })?;

            byte_pos = char_end;
            visual_col += display_width;
            continue;
        }

        // =====================================================================
        // PRIORITY 5: PLAIN CHARACTER — DEFAULT_TEXT_COLOUR (green)
        // =====================================================================
        stdout.write_all(DEFAULT_TEXT_COLOUR).map_err(|e| {
            LinesError::DisplayError(stack_format_it(
                "rURWC plain write: {}",
                &[&e.to_string()],
                "rURWC plain write",
            ))
        })?;
        stdout.write_all(char_bytes).map_err(|e| {
            LinesError::DisplayError(stack_format_it(
                "rURWC plain write: {}",
                &[&e.to_string()],
                "rURWC plain write",
            ))
        })?;
        stdout.write_all(RESET_U8).map_err(|e| {
            LinesError::DisplayError(stack_format_it(
                "rURWC plain write: {}",
                &[&e.to_string()],
                "rURWC plain write",
            ))
        })?;

        byte_pos = char_end;
        visual_col += display_width;
    }

    // =========================================================================
    // CURSOR AT/PAST END OF LINE (visual)
    // =========================================================================
    // When the cursor's visual column is at or beyond the row's total visual
    // width, draw the block at the end so the user can append after the last
    // character. Compared in VISUAL cells (matches Option A).
    if cursor_on_this_row && effective_cursor_col >= total_visual_width {
        stdout.write_all(BOLD_U8).map_err(|e| {
            LinesError::DisplayError(stack_format_it(
                "rURWC eol cursor: {}",
                &[&e.to_string()],
                "rURWC eol cursor",
            ))
        })?;
        stdout.write_all(RED_U8).map_err(|e| {
            LinesError::DisplayError(stack_format_it(
                "rURWC eol cursor: {}",
                &[&e.to_string()],
                "rURWC eol cursor",
            ))
        })?;
        stdout.write_all(BG_WHITE_U8).map_err(|e| {
            LinesError::DisplayError(stack_format_it(
                "rURWC eol cursor: {}",
                &[&e.to_string()],
                "rURWC eol cursor",
            ))
        })?;
        stdout.write_all("█".as_bytes()).map_err(|e| {
            LinesError::DisplayError(stack_format_it(
                "rURWC eol cursor: {}",
                &[&e.to_string()],
                "rURWC eol cursor",
            ))
        })?;
        stdout.write_all(RESET_U8).map_err(|e| {
            LinesError::DisplayError(stack_format_it(
                "rURWC eol cursor: {}",
                &[&e.to_string()],
                "rURWC eol cursor",
            ))
        })?;
    }

    Ok(())
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
///  // User provides the session directory they want to recover
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
///  // session_path is now: "/path/to/exe/lines_data/sessions/2025_01_15_14_30_45"
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
        // Defensive: Verify it is a directory
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
/// Recovery-reboot wrapper for lines_fullfile_editor_core
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

    #[cfg(debug_assertions)]
    {
        println!("\n=== Opening Lines Editor ===");
        println!("File: {}", target_path.display());
    }

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
        match lines_fullfile_editor_core(
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

/// Ensures a file is in a state the line editor can open for editing.
///
/// # Purpose / Project Context
/// The line editor's loading logic cannot open a completely
/// empty (zero-byte) file (e.g. one created by `touch`). To handle this
/// edge case without changing the editor's loading invariants, any
/// existing zero-byte regular file has a single newline appended so it
/// contains exactly one (empty) line.
///
/// # Arguments
/// * `target_path` - Absolute path to the file to normalize.
///
/// # Returns
/// * `Ok(true)`  - File existed and was empty; a newline was written.
/// * `Ok(false)` - No action required (file missing, not a regular file,
///                 or non-empty).
/// * `Err(_)`    - File existed and was empty, but the newline could not
///                 be written or flushed. The caller must decide whether
///                 to proceed.
///
/// # Notes
/// - No action is taken for non-existent files; file creation is the
///   responsibility of the caller.
/// - A small TOCTOU window exists between the size check and the append;
///   concurrent external writers are not expected for editor targets.
fn ensure_file_is_editor_ready(target_path: &Path) -> Result<bool> {
    // Existence + file-type guard (defensive: do not append to a directory).
    let metadata = match fs::metadata(target_path) {
        Ok(m) => m,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(e) => {
            // Cannot determine state; surface so caller can react.
            return Err(LinesError::Io(e));
        }
    };
    if !metadata.is_file() {
        return Ok(false);
    }
    if metadata.len() != 0 {
        return Ok(false);
    }

    // Known-empty regular file: append exactly one newline.
    let mut file = OpenOptions::new()
        .append(true)
        .open(target_path)
        .map_err(|e| {
            // EFER = Ensure File Editor-Ready (unique per-function prefix)
            io::Error::new(e.kind(), "EFER: cannot open empty file for normalization")
        })?;
    file.write_all(b"\n")
        .map_err(|e| io::Error::new(e.kind(), "EFER: newline write failed"))?;
    file.flush()
        .map_err(|e| io::Error::new(e.kind(), "EFER: flush failed"))?;
    Ok(true)
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
///
/// # Edge Cases
/// - Zero-byte existing files (e.g. created by `touch`) are normalized
///   to contain a single newline before opening, because the line-loader
///   cannot open a truly empty file.
///
pub fn lines_fullfile_editor_core(
    original_file_path: Option<PathBuf>,
    starting_line: Option<usize>,
    use_this_session: Option<PathBuf>,
) -> Result<bool> {
    //  =======================================
    //  Initialization & Bootstrap Lines Editor
    //  =======================================

    // Resolve target file path (all path handling logic extracted)
    let target_path = resolve_target_file_path(original_file_path)?;

    #[cfg(debug_assertions)]
    {
        println!("\n=== Opening Lines Editor ===");
        println!("File: {}", target_path.display());
    }

    // Normalize zero-byte files so the editor's loader can open them.
    // See ensure_file_is_editor_ready for project-context rationale.
    match ensure_file_is_editor_ready(&target_path) {
        Ok(_) => {} // No action, or newline successfully appended.
        Err(_e) => {
            // Could not normalize an existing empty file; log terse
            // diagnostic in debug builds only (no path leaked in prod).
            #[cfg(debug_assertions)]
            eprintln!("lines_fullfile_editor_core: normalization skipped: {}", _e);
            // Continue: the subsequent open may still succeed, and if it
            // fails the editor's normal error path will report it.

            // safe log
            eprintln!("lines_fullfile_editor_core: normalization skipped");
        }
    }

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
            Err(_e) => {
                #[cfg(debug_assertions)]
                eprintln!("lines_fullfile_editor_core: split_timestamp failed: {}", _e);

                // safe log
                eprintln!("lines_fullfile_editor_core: split_timestamp failed");

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

    //  ========================
    //  Set Up & Build The State
    //  ========================

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

    #[cfg(debug_assertions)]
    println!("Read-copy: {}", read_copy_path.display());

    // Initialize window position
    lines_editor_state.line_count_at_top_of_window = 0;
    lines_editor_state.file_position_of_topline_start = 0;
    lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset = 0;

    // Bootstrap initial cursor position, start of file, after "l "
    lines_editor_state.cursor.tui_row = 0;
    lines_editor_state.cursor.tui_visual_col = 3; // Bootstrap Bump: start after padded line nunber (zero-index 3)

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
                lines_editor_state.cursor.tui_visual_col = line_num_width; // Skip over line number display
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

    // Now we can mutably borrow lines_editor_state
    let _ = build_windowmap_nowrap(&mut lines_editor_state, &read_copy)?;

    // Main editor loop
    let mut keep_editor_loop_running = true;

    //  ================
    //  Set Up Main Loop
    //  ================

    // set up pre-allocated input buffere, short for commands
    // and Bucket Brigade! for text input:
    let mut command_buffer = [0u8; WHOLE_COMMAND_BUFFER_SIZE];

    // TODO: use/reuse general 256 buffer?
    // or have buffers in-function and remove 'general' buffers from state?
    let mut text_buffer = [0u8; TEXT_BUCKET_BRIGADE_CHUNKING_BUFFER_SIZE];
    let stdin = io::stdin();
    let mut stdin_handle = stdin.lock(); // Lock stdin once for entire session

    // Defensive: Limit loop iterations to prevent infinite loops
    let mut iteration_count = 0;

    //  ===============================
    //  Main Loop for Full Lines Editor
    //  ===============================
    while keep_editor_loop_running && iteration_count < limits::MAIN_EDITOR_LOOP_COMMANDS {
        iteration_count += 1;

        // ================
        // Bump on Main St.
        // ================
        // This is (also) for move-left handling.
        //
        // To keep the cursor on the text:
        // If on the top (zero index 0-line 0-row) bump to end of line number
        // If not row zero, move to end of previous line.
        //
        // ONLY trigger this if no horizontal scroll offset exists
        // (otherwise we're in the middle of a long line, not at line start)

        let line_num_width = calculate_line_number_width(
            lines_editor_state.line_count_at_top_of_window,
            lines_editor_state.cursor.tui_row,
            lines_editor_state.effective_rows,
        );

        // Check if cursor is in line number area (not in file-window) AND no horizontal offset
        if lines_editor_state.cursor.tui_visual_col < line_num_width
            && lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset == 0
        {
            // on line 0? (top) is cursor off the reservation? If so... Bump it Right!
            if lines_editor_state.cursor.tui_row == 0 {
                lines_editor_state.cursor.tui_visual_col = line_num_width;
                lines_editor_state.tui_window_horizontal_utf8txt_line_char_offset = 0;

                build_windowmap_nowrap(&mut lines_editor_state, &read_copy)?;
            } else {
                // Not at Top? Bump up to previous line end
                execute_command(&mut lines_editor_state, Command::MoveUp(1))?;
                execute_command(&mut lines_editor_state, Command::GotoLineEnd)?;

                build_windowmap_nowrap(&mut lines_editor_state, &read_copy)?;

                // Handle case where moving up puts us at TUI row 0
                if lines_editor_state.cursor.tui_row == 0 {
                    let line_num_width = calculate_line_number_width(
                        lines_editor_state.line_count_at_top_of_window,
                        lines_editor_state.cursor.tui_row,
                        lines_editor_state.effective_rows,
                    );

                    // Ensure cursor is at least past line numbers
                    if lines_editor_state.cursor.tui_visual_col < line_num_width {
                        lines_editor_state.cursor.tui_visual_col = line_num_width;
                    }
                }

                let _ = lines_editor_state.set_info_bar_message("start of line");
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
        } else if lines_editor_state.mode == EditorMode::KeystrokeInputMode {
            //  ====================
            //  Keystroke Input Mode
            //  ====================
            // The ONLY raw-terminal path in the editor. The session method owns
            // the RawTerminal for its entire (RAII) lifetime, runs its own read
            // loop, renders inside that loop, and recovers to Normal mode on ESC,
            // EOF, or read error. It returns Ok(true) so the main loop continues
            // (keystroke-input mode has no quit command).
            //
            // `read_copy` is cloned once at main-loop setup; we pass a borrow so
            // the path is not re-cloned per keystroke.
            keep_editor_loop_running =
                lines_editor_state.handle_keystroke_input_session(&read_copy)?;
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
            if let Ok(Some(file_pos)) = lines_editor_state.get_row_col_file_position(
                lines_editor_state.cursor.tui_row,
                lines_editor_state.cursor.tui_visual_col,
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

// ** Keep This **
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

//     //  // Diagnostics
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
