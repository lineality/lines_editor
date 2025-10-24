// tests.rs (keen this in src/ with main.rs)

#[cfg(test)]
use crate::lines_editor_module::double_width::{calculate_display_width, is_double_width};

#[cfg(test)]
use crate::lines_editor_module::*;

#[cfg(test)]
use std::env;

#[cfg(test)]
use std::fs::File;

#[cfg(test)]
use std::io::BufRead;

#[cfg(test)]
use std::io::BufReader;

#[cfg(test)]
use std::io::{self};

#[cfg(test)]
use std::path::{Path, PathBuf};

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
// pub fn render_tui_to_writer<W: Write>(state: &EditorState, writer: &mut W) -> Result<()> {
//     // Top legend
//     let legend = format_navigation_legend()?;
//     writeln!(writer, "{}", legend)
//         .map_err(|e| LinesError::DisplayError(format!("Write failed: {}", e)))?;

//     // Content rows
//     for row in 0..state.effective_rows {
//         if state.display_buffer_lengths[row] > 0 {
//             let row_content = &state.display_buffers[row][..state.display_buffer_lengths[row]];

//             match std::str::from_utf8(row_content) {
//                 Ok(row_str) => {
//                     // ONLY CHANGE: Apply cursor highlighting if cursor is on this row
//                     let display_str = render_row_with_cursor(state, row, row_str);
//                     writeln!(writer, "{}", display_str)
//                 }
//                 Err(_) => writeln!(writer, "�"),
//             }
//             .map_err(|e| LinesError::DisplayError(format!("Write failed: {}", e)))?;
//         } else {
//             // ONLY CHANGE: Show cursor on empty rows if cursor is here
//             if row == state.cursor.row {
//                 writeln!(
//                     writer,
//                     "{}{}{}█{}",
//                     "\x1b[1m", "\x1b[31m", "\x1b[47m", "\x1b[0m"
//                 )
//             } else {
//                 writeln!(writer)
//             }
//             .map_err(|e| LinesError::DisplayError(format!("Write failed: {}", e)))?;
//         }
//     }

//     // Bottom info bar
//     let info_bar = format_info_bar(state)?;
//     write!(writer, "{}", info_bar)
//         .map_err(|e| LinesError::DisplayError(format!("Write failed: {}", e)))?;

//     writer
//         .flush()
//         .map_err(|e| LinesError::DisplayError(format!("Flush failed: {}", e)))?;

//     Ok(())
// }

/// Creates test files in project ./test_files/ directory
/// Files are NEVER deleted - they persist for manual inspection
/// If files already exist, they are reused
///
/// # Directory Structure
/// ```
/// ./test_files/
///   ├── basic_short.txt
///   ├── long_lines.txt
///   ├── mixed_utf8.txt
///   └── edge_cases.txt
/// ```
///
/// # Returns
/// * `Ok(Vec<PathBuf>)` - Absolute paths to test files
/// * `Err(io::Error)` - If directory creation or file writing fails
#[cfg(test)]
pub fn create_test_files_with_id(_test_name: &str) -> io::Result<Vec<PathBuf>> {
    use std::fs::{self, File};
    use std::io::Write;

    // Get current working directory
    let cwd = env::current_dir()?;

    // Create test_files directory in project root
    let test_dir = cwd.join("test_files");
    fs::create_dir_all(&test_dir)?;

    println!("Test files directory: {}", test_dir.display());

    let mut test_files = Vec::with_capacity(4);

    // Test File 1: basic_short.txt
    {
        let path = test_dir.join("basic_short.txt");

        // Only create if it doesn't exist
        if !path.exists() {
            println!("Creating: {}", path.display());
            let mut file = File::create(&path)?;

            writeln!(file, "Line 1: Hello, world!")?;
            writeln!(file, "Line 2: This is a test.")?;
            writeln!(file, "Line 3: Short line.")?;
            writeln!(file, "Line 4: Another short test line.")?;
            writeln!(file, "Line 5: Fifth line here.")?;
            writeln!(file, "Line 6: Almost done.")?;
            writeln!(file, "Line 7: Lucky seven.")?;
            writeln!(file, "Line 8: Eight is great.")?;
            writeln!(file, "Line 9: Nine is fine.")?;
            writeln!(file, "Line 10: Double digits!")?;
            writeln!(file, "Line 11: Eleven.")?;
            writeln!(file, "Line 12: Twelve.")?;
            writeln!(file, "Line 13: Thirteen.")?;
            writeln!(file, "Line 14: Fourteen.")?;
            writeln!(file, "Line 15: Fifteen.")?;
            writeln!(file, "Line 16: Sixteen.")?;
            writeln!(file, "Line 17: Seventeen.")?;
            writeln!(file, "Line 18: Eighteen.")?;
            writeln!(file, "Line 19: Nineteen.")?;
            writeln!(file, "Line 20: Twenty.")?;
            writeln!(file, "Line 21: Twenty-one.")?;
            writeln!(file, "Line 22: Twenty-two.")?;
            writeln!(file, "Line 23: Last line for now.")?;

            file.flush()?;
        } else {
            println!("Reusing existing: {}", path.display());
        }

        test_files.push(path);
    }

    // Test File 2: long_lines.txt
    {
        let path = test_dir.join("long_lines.txt");

        if !path.exists() {
            println!("Creating: {}", path.display());
            let mut file = File::create(&path)?;

            writeln!(file, "Line 1: {}", "A".repeat(100))?;
            writeln!(
                file,
                "Line 2: The quick brown fox jumps over the lazy dog. The quick brown fox jumps over the lazy dog again."
            )?;
            writeln!(file, "Line 3: {}", "0123456789".repeat(12))?;
            writeln!(file, "Line 4: Short.")?;
            writeln!(file, "Line 5: {}", "Long_word_".repeat(15))?;

            file.flush()?;
        } else {
            println!("Reusing existing: {}", path.display());
        }

        test_files.push(path);
    }

    // Test File 3: mixed_utf8.txt
    {
        let path = test_dir.join("mixed_utf8.txt");

        if !path.exists() {
            println!("Creating: {}", path.display());
            let mut file = File::create(&path)?;

            writeln!(file, "Line 1: Hello 世界")?;
            writeln!(file, "Line 2: こんにちは")?;
            writeln!(file, "Line 3: Test カタカナ Test")?;
            writeln!(file, "Line 4: Café résumé")?;
            writeln!(file, "Line 5: 한글 Korean")?;
            writeln!(file, "Line 6: Mix 中文 and English")?;
            writeln!(file, "Line 7: Numbers ０１２３４")?;

            file.flush()?;
        } else {
            println!("Reusing existing: {}", path.display());
        }

        test_files.push(path);
    }

    // Test File 4: edge_cases.txt
    {
        let path = test_dir.join("edge_cases.txt");

        if !path.exists() {
            println!("Creating: {}", path.display());
            let mut file = File::create(&path)?;

            writeln!(file, "")?;
            writeln!(file, "A")?;
            writeln!(file, "\t")?;
            writeln!(file, "Before\ttab\tafter")?;
            writeln!(file, " ")?;
            writeln!(file, "    Indented")?;
            writeln!(file, "Trailing    ")?;
            writeln!(file, "")?;
            writeln!(file, "Normal line after empties")?;

            file.flush()?;
        } else {
            println!("Reusing existing: {}", path.display());
        }

        test_files.push(path);
    }

    // Test File 5: shorty.txt
    {
        let path = test_dir.join("shorty.txt");

        // Only create if it doesn't exist
        if !path.exists() {
            println!("Creating: {}", path.display());
            let mut file = File::create(&path)?;

            writeln!(file, "Line 1: Hello, world!")?;
            writeln!(file, "Line 2: This is a test.")?;
            writeln!(file, "Line 3: Short line.")?;
            file.flush()?;
        } else {
            println!("Reusing existing: {}", path.display());
        }

        test_files.push(path);
    }

    // Defensive: Verify all files exist and have content
    for path in &test_files {
        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Test file does not exist: {:?}", path),
            ));
        }

        let metadata = fs::metadata(path)?;
        if metadata.len() == 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Test file is empty: {:?}", path),
            ));
        }
    }

    Ok(test_files)
}

// /// Prints the expected window output for a test file
// ///
// /// # Purpose
// /// Helper function to visualize what build_windowmap_nowrap SHOULD produce
// /// for a given test file. This helps us verify our implementation.
// ///
// /// # Arguments
// /// * `test_file` - Path to test file
// /// * `start_line` - Line number to start display from (0-indexed)
// /// * `horizontal_offset` - Character offset for NoWrap mode
// ///
// /// # Example Output
// /// Shows what should appear in each display buffer row:
// /// ```
// /// Row 0: "1 Line 1: Hello, world!"
// /// Row 1: "2 Line 2: This is a test."
// /// ```
// fn print_expected_window(
//     test_file: &Path,
//     start_line: usize,
//     horizontal_offset: usize,
// ) -> io::Result<()> {
//     println!(
//         "\nExpected window for: {:?}",
//         test_file.file_name().unwrap_or_default()
//     );
//     println!(
//         "Start line: {}, Horizontal offset: {}",
//         start_line, horizontal_offset
//     );

//     let file = File::open(test_file)?;
//     let reader = BufReader::new(file);
//     let mut current_line = 0;
//     let mut display_row = 0;
//     const MAX_DISPLAY_ROWS: usize = 21;
//     const MAX_DISPLAY_COLS: usize = 77; // 80 - 3 for UI elements

//     for line in reader.lines() {
//         let line = line?;

//         // Skip lines before our window start
//         if current_line < start_line {
//             current_line += 1;
//             continue;
//         }

//         // Stop if we've filled the display
//         if display_row >= MAX_DISPLAY_ROWS {
//             break;
//         }

//         // Format line number (starting from 1 for display)
//         let line_num_str = format!("{} ", current_line + 1);
//         let line_num_width = line_num_str.len();

//         // Calculate available space for text after line number
//         let available_width = MAX_DISPLAY_COLS.saturating_sub(line_num_width);

//         // Get the portion of line to display (respecting horizontal offset)
//         let line_chars: Vec<char> = line.chars().collect();
//         let visible_text = if horizontal_offset < line_chars.len() {
//             let end_idx = (horizontal_offset + available_width).min(line_chars.len());
//             line_chars[horizontal_offset..end_idx]
//                 .iter()
//                 .collect::<String>()
//         } else {
//             String::new() // Horizontal offset past end of line
//         };

//         println!(
//             "Row {:2}: \"{}{}\"",
//             display_row, line_num_str, visible_text
//         );

//         display_row += 1;
//         current_line += 1;
//     }

//     // Fill remaining rows with empty
//     while display_row < MAX_DISPLAY_ROWS {
//         println!("Row {:2}: (empty)", display_row);
//         display_row += 1;
//     }

//     Ok(())
// }

/// Diagnostic function to print contents of test files
#[cfg(test)]
fn print_test_file_contents(file_path: &Path) -> io::Result<()> {
    println!("=== File Contents: {} ===", file_path.display());
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        println!("{:4}: {}", index + 1, line);
    }

    // Get file metadata
    let metadata = std::fs::metadata(file_path)?;
    println!("\nFile size: {} bytes", metadata.len());

    Ok(())
}

// #[cfg(test)]
// mod display_window_tests2 {
//     use super::*;

//     // #[test]
//     // fn test_display_window_basic() -> io::Result<()> {
//     //     // Use unique test files
//     //     let test_files = create_test_files_with_id("display_basic")?;
//     //     let basic_file = &test_files[0];

//     //     let mut state = EditorState::new();
//     //     state.line_count_at_top_of_window = 0;
//     //     state.file_position_of_topline_start = 0;
//     //     state.horizontal_line_char_offset = 0;

//     //     let lines_processed = build_windowmap_nowrap(&mut state, basic_file)?;
//     //     assert!(lines_processed > 0, "Should process at least one line");

//     //     let mut buffer = Vec::new();
//     //     display_window_to_writer(&state, &mut buffer)?;

//     //     let output = String::from_utf8_lossy(&buffer);

//     //     assert!(
//     //         output.contains("1 Line 1: Hello, world!"),
//     //         "Output should contain first line with line number"
//     //     );

//     //     Ok(())
//     // }

//     // #[test]
//     // fn test_display_window_utf8() -> io::Result<()> {
//     //     // Use unique test files
//     //     let test_files = create_test_files_with_id("display_utf8")?;
//     //     let mixed_utf8_file = &test_files[2];

//     //     let mut state = EditorState::new();
//     //     state.line_count_at_top_of_window = 0;
//     //     state.file_position_of_topline_start = 0;
//     //     state.horizontal_line_char_offset = 0;

//     //     let lines_processed = build_windowmap_nowrap(&mut state, mixed_utf8_file)?;
//     //     assert!(lines_processed > 0, "Should process at least one line");

//     //     let mut buffer = Vec::new();
//     //     display_window_to_writer(&state, &mut buffer)?;

//     //     let output = String::from_utf8_lossy(&buffer);

//     //     assert!(
//     //         output.contains("1 Line 1: Hello 世界"),
//     //         "Should handle CJK characters with line number"
//     //     );

//     //     Ok(())
//     // }
// }

#[cfg(test)]
mod build_window_tests4 {
    use super::*; // ← Line 1: import from tests.rs

    #[test]
    fn test_build_windowmap_nowrap_basic() {
        // Use unique test files for this test
        let test_files = create_test_files_with_id("build_window_test").unwrap();
        let basic_file = &test_files[0];

        let mut state = EditorState::new();
        state.line_count_at_top_of_window = 0;
        state.file_position_of_topline_start = 0;
        state.horizontal_line_char_offset = 0;

        let result = build_windowmap_nowrap(&mut state, basic_file);
        assert!(result.is_ok(), "Should build window successfully");

        let lines_processed = result.unwrap();
        assert!(lines_processed > 0, "Should process at least one line");

        assert!(
            state.display_buffer_lengths[0] > 0,
            "First row should have content"
        );

        let first_row = &state.display_buffers[0];
        assert_eq!(first_row[0], b'1', "Should start with line number 1");
        assert_eq!(first_row[1], b' ', "Should have space after line number");

        let map_entry = state.window_map.get_file_position(0, 2).unwrap();
        assert!(map_entry.is_some(), "Character position should be mapped");
    }
}

/// Debug helper for build_windowmap_nowrap test
#[cfg(test)]
mod build_window_tests3 {
    use super::*;

    #[test]
    fn test_build_windowmap_nowrap_basic() {
        // Create test file
        let test_files = create_test_files_with_id("test_build_windowmap_nowrap_basic").unwrap();
        let basic_file = &test_files[0]; // basic_short.txt

        // Create editor state
        let mut state = EditorState::new();
        state.line_count_at_top_of_window = 0;
        state.file_position_of_topline_start = 0;
        state.horizontal_line_char_offset = 0;

        // Debug: print file path
        println!("Test file path: {:?}", basic_file);
        println!("File exists: {}", basic_file.exists());

        // Build window
        let result = build_windowmap_nowrap(&mut state, basic_file);

        // Debug: print detailed error if failed
        if let Err(ref e) = result {
            println!("Build window failed: {}", e);
        }

        assert!(result.is_ok(), "Should build window successfully");

        let lines_processed = result.unwrap();
        println!("Lines processed: {}", lines_processed);

        // Debug: print buffer contents
        for i in 0..5 {
            if state.display_buffer_lengths[i] > 0 {
                let content = &state.display_buffers[i][..state.display_buffer_lengths[i]];
                println!("Row {}: {:?}", i, String::from_utf8_lossy(content));
            }
        }

        assert!(lines_processed > 0, "Should process at least one line");

        // Verify first line has content
        assert!(
            state.display_buffer_lengths[0] > 0,
            "First row should have content"
        );

        // Verify line number "1 " appears at start
        let first_row = &state.display_buffers[0];
        assert_eq!(first_row[0], b'1', "Should start with line number 1");
        assert_eq!(first_row[1], b' ', "Should have space after line number");

        // Verify WindowMap has been populated
        let map_entry = state.window_map.get_file_position(0, 2).unwrap();
        assert!(map_entry.is_some(), "Character position should be mapped");
    }
}

#[cfg(test)]
mod editor_state_tests {
    use super::*;

    #[test]
    fn test_editor_state_creation() {
        let state = EditorState::new();
        assert_eq!(state.mode, EditorMode::Normal);
        assert_eq!(state.terminal_rows, DEFAULT_ROWS);
        assert_eq!(state.terminal_cols, DEFAULT_COLS);
        // assert_eq!(state.filetui_windowmap_buffer_used, 0);
        assert!(!state.is_modified);
    }

    #[test]
    fn test_resize_terminal_valid() {
        let mut state = EditorState::new();
        let result = state.resize_terminal(30, 100);
        assert!(result.is_ok());
        assert_eq!(state.terminal_rows, 30);
        assert_eq!(state.terminal_cols, 100);
        assert_eq!(state.effective_rows, 27); // 30 - 3
        assert_eq!(state.effective_cols, 97); // 100 - 3
    }

    #[test]
    fn test_resize_terminal_too_large() {
        let mut state = EditorState::new();
        let result = state.resize_terminal(200, 100); // 200 > MAX_ROWS
        assert!(result.is_err());
    }

    #[test]
    fn test_window_map_bounds_checking() {
        let map = WindowMap::new();

        // Valid access
        let result = map.get_file_position(0, 0);
        assert!(result.is_ok());

        // Out of bounds access
        let result = map.get_file_position(DEFAULT_ROWS, 0);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod test_file_tests {
    use super::*;

    #[test]
    fn test_create_test_files() {
        let result = create_test_files_with_id("test_create_test_files");
        assert!(result.is_ok(), "Should create test files successfully");

        let files = result.unwrap();
        assert_eq!(files.len(), 5, "Should create 5 test files");

        // Verify each file exists and has content
        for path in &files {
            assert!(path.exists(), "File {:?} should exist", path);

            let metadata = std::fs::metadata(path).unwrap();
            assert!(metadata.len() > 0, "File {:?} should have content", path);
        }
    }
}

#[cfg(test)]
mod char_width_tests {
    use super::*; // ← Line 1: import from tests.rs

    #[test]
    fn test_ascii_characters() {
        // All ASCII characters should be single-width
        for c in 0x20..0x7F {
            let ch = char::from_u32(c).expect("Valid ASCII character");
            assert_eq!(
                is_double_width(ch),
                false,
                "ASCII '{}' should be single-width",
                ch
            );
        }
    }

    #[test]
    fn test_cjk_ideographs() {
        // Common CJK characters should be double-width
        let test_chars = ['中', '文', '字', '日', '本', '語', '한', '글'];
        for &c in &test_chars {
            assert_eq!(
                is_double_width(c),
                true,
                "CJK '{}' should be double-width",
                c
            );
        }
    }

    #[test]
    fn test_hiragana_katakana() {
        // Hiragana
        assert_eq!(is_double_width('あ'), true);
        assert_eq!(is_double_width('い'), true);
        assert_eq!(is_double_width('う'), true);

        // Katakana
        assert_eq!(is_double_width('ア'), true);
        assert_eq!(is_double_width('イ'), true);
        assert_eq!(is_double_width('ウ'), true);
    }

    #[test]
    fn test_fullwidth_forms() {
        // Fullwidth Latin letters
        assert_eq!(is_double_width('Ａ'), true);
        assert_eq!(is_double_width('Ｂ'), true);
        assert_eq!(is_double_width('１'), true);
        assert_eq!(is_double_width('２'), true);
    }

    #[test]
    fn test_calculate_display_width() {
        assert_eq!(calculate_display_width("Hello"), Some(5));
        assert_eq!(calculate_display_width("你好"), Some(4));
        assert_eq!(calculate_display_width("Hello世界"), Some(9));
        assert_eq!(calculate_display_width(""), Some(0));
        assert_eq!(calculate_display_width("ＡＢＣ"), Some(6));
    }

    #[test]
    fn test_mixed_width_string() {
        let mixed = "Hello 世界 World";
        let expected = 5 + 1 + 2 + 2 + 1 + 5; // "Hello" + " " + "世界" + " " + "World"
        assert_eq!(calculate_display_width(mixed), Some(expected));
    }

    #[test]
    fn test_edge_cases() {
        // Control characters
        assert_eq!(is_double_width('\n'), false);
        assert_eq!(is_double_width('\t'), false);
        assert_eq!(is_double_width('\r'), false);

        // Space
        assert_eq!(is_double_width(' '), false);

        // Emoji (most are not double-width in our definition)
        assert_eq!(is_double_width('😀'), false);
    }
}

// /// integration tests for display_window
// #[cfg(test)]
// mod display_window_tests1 {
//     use super::*;

//     // #[test]
//     // fn test_display_window_basic() -> io::Result<()> {
//     //     // Create test files
//     //     let test_files = create_test_files_with_id("test_display_window_basic")?;
//     //     let basic_file = &test_files[0]; // basic_short.txt

//     //     // Create and populate editor state
//     //     let mut state = EditorState::new();
//     //     state.line_count_at_top_of_window = 0;
//     //     state.file_position_of_topline_start = 0;
//     //     state.horizontal_line_char_offset = 0;

//     //     // Build window map
//     //     let lines_processed = build_windowmap_nowrap(&mut state, basic_file)?;
//     //     assert!(lines_processed > 0, "Should process at least one line");

//     //     // Capture output using the writer version
//     //     let mut buffer = Vec::new();
//     //     display_window_to_writer(&state, &mut buffer)?;

//     //     // Convert captured output to string
//     //     let output = String::from_utf8_lossy(&buffer);

//     //     // Debug: print what we captured
//     //     println!("Captured output:\n{}", output);

//     //     // Verify content
//     //     assert!(
//     //         output.contains("1 Line 1: Hello, world!"),
//     //         "Output should contain first line with line number"
//     //     );
//     //     assert!(
//     //         output.lines().count() >= 18,
//     //         "Should have at least 18 lines"
//     //     );

//     //     Ok(())
//     // }

//     // #[test]
//     // fn test_display_window_utf8() -> io::Result<()> {
//     //     let test_files = create_test_files_with_id("test_display_window_utf8")?;
//     //     let mixed_utf8_file = &test_files[2]; // mixed_utf8.txt

//     //     let mut state = EditorState::new();
//     //     state.line_count_at_top_of_window = 0;
//     //     state.file_position_of_topline_start = 0;
//     //     state.horizontal_line_char_offset = 0;

//     //     // Build window map
//     //     let lines_processed = build_windowmap_nowrap(&mut state, mixed_utf8_file)?;
//     //     assert!(lines_processed > 0, "Should process at least one line");

//     //     // Capture output using the writer version
//     //     let mut buffer = Vec::new();
//     //     display_window_to_writer(&state, &mut buffer)?;

//     //     // Convert captured output to string
//     //     let output = String::from_utf8_lossy(&buffer);

//     //     // Debug: print what we captured
//     //     println!("Captured UTF-8 output:\n{}", output);

//     //     // Verify UTF-8 content - check the actual formatted line
//     //     assert!(
//     //         output.contains("1 Line 1: Hello 世界"),
//     //         "Should handle CJK characters with line number"
//     //     );
//     //     assert!(
//     //         output.contains("2 Line 2: こんにちは"),
//     //         "Should handle Hiragana with line number"
//     //     );

//     //     Ok(())
//     // }
// }

// Modify the test to include more diagnostics
#[test]
fn test_build_windowmap_nowrap_basic() -> io::Result<()> {
    // Create test files
    let test_files = create_test_files_with_id("test_build_windowmap_nowrap_basic")?;
    let basic_file = &test_files[0]; // basic_short.txt

    // Print file contents for debugging
    print_test_file_contents(basic_file)?;

    // Create editor state
    let mut state = EditorState::new();
    state.line_count_at_top_of_window = 0;
    state.file_position_of_topline_start = 0;
    state.horizontal_line_char_offset = 0;

    // Debug: print file path and existence
    println!("Test file path: {:?}", basic_file);
    println!("File exists: {}", basic_file.exists());

    // Verify file is readable
    let file = File::open(basic_file)?;
    let reader = BufReader::new(file);
    let line_count = reader.lines().count();
    println!("Line count in file: {}", line_count);

    // Build window
    let result = build_windowmap_nowrap(&mut state, basic_file);

    // Debug: print detailed error if failed
    if let Err(ref e) = result {
        println!("Build window failed: {}", e);
    }

    assert!(result.is_ok(), "Should build window successfully");

    let lines_processed = result.unwrap();
    println!("Lines processed: {}", lines_processed);

    // Debug: print buffer contents
    for i in 0..5 {
        if state.display_buffer_lengths[i] > 0 {
            let content = &state.display_buffers[i][..state.display_buffer_lengths[i]];
            println!("Row {}: {:?}", i, String::from_utf8_lossy(content));
        }
    }

    assert!(lines_processed > 0, "Should process at least one line");

    // Verify first line has content
    assert!(
        state.display_buffer_lengths[0] > 0,
        "First row should have content"
    );

    // Verify line number "1 " appears at start
    let first_row = &state.display_buffers[0];
    assert_eq!(first_row[0], b'1', "Should start with line number 1");
    assert_eq!(first_row[1], b' ', "Should have space after line number");

    // Verify WindowMap has been populated
    let map_entry = state.window_map.get_file_position(0, 2).unwrap();
    assert!(map_entry.is_some(), "Character position should be mapped");

    Ok(())
}

#[cfg(test)]
mod revised_critical_distinction_tests {
    use super::*;

    #[test]
    fn test_bytes_vs_chars_vs_columns() -> io::Result<()> {
        // This test demonstrates the three different measurements
        let test_files = create_test_files_with_id("measurements")?;
        let test_path = &test_files[2]; // mixed_utf8.txt

        // mixed_utf8.txt has:
        // "Line 1: Hello 世界" where:
        // - 世界 = 6 bytes, 2 chars, 4 display columns
        let content = "Hello 世界";

        // Verify measurements on the string itself
        let bytes = content.as_bytes();
        assert_eq!(bytes.len(), 12, "Should be 12 bytes total");

        let char_count = content.chars().count();
        assert_eq!(char_count, 8, "Should be 8 characters total");

        let display_width =
            double_width::calculate_display_width(content).expect("Should calculate width");
        assert_eq!(display_width, 10, "Should be 10 display columns total");

        // Now test with the editor
        let mut state = EditorState::new();
        state.line_count_at_top_of_window = 0;
        state.file_position_of_topline_start = 0;
        state.horizontal_line_char_offset = 0;

        let result = build_windowmap_nowrap(&mut state, &test_path);
        assert!(result.is_ok(), "Build window should succeed");

        // The display buffer should contain the line with line number
        let first_row_len = state.display_buffer_lengths[0];
        assert!(first_row_len > 0, "First row should have content");

        // Verify content is valid UTF-8
        let first_row_content = &state.display_buffers[0][..first_row_len];
        let first_row_str = std::str::from_utf8(first_row_content).expect("Should be valid UTF-8");

        // Should contain the line number "1 " and the text
        assert!(
            first_row_str.starts_with("1 "),
            "Should start with line number"
        );
        assert!(
            first_row_str.contains("世界"),
            "Should contain Chinese characters"
        );

        // Display width should fit within terminal
        let row_display_width = double_width::calculate_display_width(first_row_str)
            .expect("Should calculate display width");
        assert!(
            row_display_width <= 80,
            "Display width {} should not exceed terminal width 80",
            row_display_width
        );

        Ok(())
    }

    #[test]
    fn test_buffer_vs_terminal_size() -> io::Result<()> {
        // This test verifies buffer size is independent of terminal size
        let test_files = create_test_files_with_id("buffer_terminal")?;
        let test_path = &test_files[2]; // mixed_utf8.txt has double-width chars

        // Test with NARROW terminal
        let mut state = EditorState::new();
        state
            .resize_terminal(24, 40)
            .expect("Failed to resize to narrow");

        state.line_count_at_top_of_window = 0;
        state.file_position_of_topline_start = 0;
        state.horizontal_line_char_offset = 0;

        let result = build_windowmap_nowrap(&mut state, &test_path);
        assert!(result.is_ok(), "Should handle narrow terminal");

        // First row should be limited by display columns (40), not buffer size (182)
        let first_row_len = state.display_buffer_lengths[0];
        assert!(first_row_len <= 182, "Should not exceed buffer size");
        assert!(first_row_len > 0, "Should have content");

        let first_row_content = &state.display_buffers[0][..first_row_len];
        let first_row_str = std::str::from_utf8(first_row_content).expect("Should be valid UTF-8");

        let display_width =
            double_width::calculate_display_width(first_row_str).expect("Should calculate width");

        assert!(
            display_width <= 40,
            "Display width {} should not exceed terminal width 40",
            display_width
        );

        Ok(())
    }
}

#[cfg(test)]
mod revised_boundary_tests {
    use super::*; // ← Line 1: import from tests.rs

    use std::fs;

    #[test]
    fn test_double_width_at_boundary() -> io::Result<()> {
        // Create a custom test file for this specific case
        let test_dir = env::current_dir()?.join("test_files");
        fs::create_dir_all(&test_dir)?;
        let test_path = test_dir.join("boundary_test.txt");

        // Create a line with double-width character near terminal edge
        let mut line = String::from("x").repeat(75);
        line.push('中'); // Double-width character (takes 2 display columns)
        line.push_str("yyy");

        fs::write(&test_path, &line)?;

        // ADD DIAGNOSTIC: Show what we created
        println!("Created boundary test file: {}", test_path.display());
        println!("Line content: {:?}", line);
        println!("Line byte length: {}", line.len());
        println!("Line char count: {}", line.chars().count());

        // Calculate expected display width
        let expected_display_width = 75 + 2 + 3; // 75 x's + 中(2 cols) + 3 y's = 80
        println!("Expected display width: {}", expected_display_width);

        // Create 80-column terminal
        let mut state = EditorState::new();
        state.resize_terminal(24, 80)?;

        // ADD DIAGNOSTIC: Show state after resize
        println!("Terminal cols: {}", state.terminal_cols);
        println!("Effective cols: {}", state.effective_cols);
        println!("WindowMap valid_cols: {}", state.window_map.valid_cols);

        state.line_count_at_top_of_window = 0;
        state.file_position_of_topline_start = 0;
        state.horizontal_line_char_offset = 0;

        // Build window
        let result = build_windowmap_nowrap(&mut state, &test_path);

        // Debug output if it fails
        if let Err(ref e) = result {
            println!("Build window error: {}", e);
            println!("Error details: {:?}", e);

            // ADD DIAGNOSTIC: Show where in the process we failed
            println!("\nState at failure:");
            println!(
                "- display_buffer_lengths[0]: {}",
                state.display_buffer_lengths[0]
            );
            if state.display_buffer_lengths[0] > 0 {
                let partial_content = &state.display_buffers[0][..state.display_buffer_lengths[0]];
                if let Ok(s) = std::str::from_utf8(partial_content) {
                    println!("- Partial row content: {:?}", s);
                }
            }
        }

        assert!(result.is_ok(), "Should build window successfully");

        // Verify no invalid UTF-8 (which would indicate a split)
        let row_content = &state.display_buffers[0][..state.display_buffer_lengths[0]];
        let parse_result = std::str::from_utf8(row_content);

        assert!(
            parse_result.is_ok(),
            "Row should not contain split UTF-8 sequences: {:?}",
            parse_result.err()
        );

        // REMOVED THE FAULTY WINDOW MAP CHECK that tried to access column 77
        // when valid columns are 0-76 (77 total, but 0-indexed)

        // Instead, let's verify what actually got placed in the window
        let row_str = parse_result.unwrap();
        println!("Final row content: {:?}", row_str);
        println!(
            "Final row display width: {:?}",
            double_width::calculate_display_width(row_str)
        );

        // The row should fit within terminal width
        let display_width =
            double_width::calculate_display_width(row_str).expect("Should calculate display width");
        assert!(
            display_width <= 80,
            "Display width {} should not exceed terminal width 80",
            display_width
        );

        Ok(())
    }
}

#[cfg(test)]
mod revised_terminal_width_tests {
    use super::*;

    #[test]
    fn test_terminal_width_limiting() -> io::Result<()> {
        // Use long_lines.txt which has lines exceeding 80 chars
        let test_files = create_test_files_with_id("width_limiting")?;
        let test_path = &test_files[1]; // long_lines.txt

        // Create editor state with specific terminal size
        let mut state = EditorState::new();
        state.resize_terminal(24, 80)?;
        state.line_count_at_top_of_window = 0;
        state.file_position_of_topline_start = 0;
        state.horizontal_line_char_offset = 0;

        // Build window
        let result = build_windowmap_nowrap(&mut state, &test_path);
        assert!(result.is_ok(), "Should build window successfully");

        // Verify no row exceeds terminal width
        for row in 0..state.effective_rows {
            if state.display_buffer_lengths[row] > 0 {
                let row_content = &state.display_buffers[row][..state.display_buffer_lengths[row]];

                // Verify valid UTF-8
                let row_str = std::str::from_utf8(row_content)
                    .expect(&format!("Row {} should be valid UTF-8", row));

                // Check display width
                let display_width = double_width::calculate_display_width(row_str)
                    .expect(&format!("Row {} should have calculable width", row));

                assert!(
                    display_width <= 80,
                    "Row {} display width {} exceeds terminal width 80",
                    row,
                    display_width
                );
            }
        }

        Ok(())
    }

    #[test]
    fn test_various_terminal_sizes() -> io::Result<()> {
        let test_files = create_test_files_with_id("various_sizes")?;
        let test_path = &test_files[0]; // basic_short.txt

        // Test multiple terminal sizes
        let terminal_sizes = vec![
            (24, 40),  // Narrow
            (24, 80),  // Standard
            (24, 120), // Wide
            (45, 157), // Maximum
        ];

        for (rows, cols) in terminal_sizes {
            let mut state = EditorState::new();
            state.resize_terminal(rows, cols)?;
            state.line_count_at_top_of_window = 0;
            state.file_position_of_topline_start = 0;
            state.horizontal_line_char_offset = 0;

            let result = build_windowmap_nowrap(&mut state, &test_path);
            assert!(
                result.is_ok(),
                "Should build window for terminal size {}x{}",
                cols,
                rows
            );

            // Verify all rows respect terminal width
            for row in 0..state.effective_rows {
                if state.display_buffer_lengths[row] > 0 {
                    let row_content =
                        &state.display_buffers[row][..state.display_buffer_lengths[row]];
                    let row_str = std::str::from_utf8(row_content)
                        .expect(&format!("Row {} should be valid UTF-8", row));
                    let display_width =
                        double_width::calculate_display_width(row_str).ok_or_else(|| {
                            io::Error::new(
                                io::ErrorKind::Other,
                                format!("Could not calculate width for row {}", row),
                            )
                        })?;

                    assert!(
                        display_width <= cols,
                        "Row {} width {} exceeds terminal width {} for size {}x{}",
                        row,
                        display_width,
                        cols,
                        cols,
                        rows
                    );
                }
            }
        }

        Ok(())
    }
}

// #[cfg(test)]
// mod revised_display_integration_tests {
//     use super::*;

//     // #[test]
//     // fn test_double_width_character_display() -> io::Result<()> {
//     //     // Use mixed_utf8.txt which has various character widths
//     //     let test_files = create_test_files_with_id("double_width_display")?;
//     //     let test_path = &test_files[2]; // mixed_utf8.txt

//     //     // Create editor state
//     //     let mut state = EditorState::new();
//     //     state.line_count_at_top_of_window = 0;
//     //     state.file_position_of_topline_start = 0;
//     //     state.horizontal_line_char_offset = 0;

//     //     // Build window with the test file
//     //     let result = build_windowmap_nowrap(&mut state, &test_path);
//     //     assert!(result.is_ok(), "Should build window successfully");
//     //     let lines_processed = result.unwrap();

//     //     // Verify display (capture to buffer for testing)
//     //     let mut buffer = Vec::new();
//     //     let display_result = display_window_to_writer(&state, &mut buffer);
//     //     assert!(display_result.is_ok(), "Should display successfully");

//     //     let output = String::from_utf8_lossy(&buffer);

//     //     // Verify specific content
//     //     assert!(output.contains("世界"), "Should display Chinese characters");
//     //     assert!(output.contains("こんにちは"), "Should display Hiragana");
//     //     assert!(output.contains("한글"), "Should display Hangul");

//     //     // Verify line numbers are present
//     //     assert!(output.contains("1 "), "Should have line number 1");
//     //     assert!(output.contains("2 "), "Should have line number 2");

//     //     // Verify WindowMap was populated for double-width characters
//     //     // Check that Chinese characters in first line are properly mapped
//     //     for row in 0..3 {
//     //         if state.display_buffer_lengths[row] > 0 {
//     //             // Check that we can get file positions from the map
//     //             let pos = state.window_map.get_file_position(row, 5)?;
//     //             assert!(
//     //                 pos.is_some() || row >= lines_processed,
//     //                 "Row {} col 5 should have file position or be empty",
//     //                 row
//     //             );
//     //         }
//     //     }

//     //     Ok(())
//     // }

//     // #[test]
//     // fn test_empty_lines_display() -> io::Result<()> {
//     //     // Use edge_cases.txt which has empty lines
//     //     let test_files = create_test_files_with_id("empty_lines")?;
//     //     let test_path = &test_files[3]; // edge_cases.txt

//     //     let mut state = EditorState::new();
//     //     state.line_count_at_top_of_window = 0;
//     //     state.file_position_of_topline_start = 0;
//     //     state.horizontal_line_char_offset = 0;

//     //     let result = build_windowmap_nowrap(&mut state, &test_path);
//     //     assert!(result.is_ok(), "Should handle empty lines");

//     //     // Verify we processed multiple lines including empties
//     //     let lines_processed = result.unwrap();
//     //     assert!(lines_processed > 1, "Should process multiple lines");

//     //     // Capture display output
//     //     let mut buffer = Vec::new();
//     //     display_window_to_writer(&state, &mut buffer)?;
//     //     let output = String::from_utf8_lossy(&buffer);

//     //     // Empty lines should still have line numbers
//     //     let line_count = output.lines().count();
//     //     assert!(
//     //         line_count >= lines_processed,
//     //         "Should display all processed lines"
//     //     );

//     //     Ok(())
//     // }
// }

#[cfg(test)]
mod timestamp_tests {
    use super::*;

    #[test]
    fn test_days_to_ymd_boundary_conditions() {
        // Test 1: Zero days (epoch start: 1970-01-01)
        let (year, month, day) = days_to_ymd(0);
        assert_eq!(year, 1970, "Year should be 1970 at epoch");
        assert_eq!(month, 1, "Month should be January at epoch");
        assert_eq!(day, 1, "Day should be 1 at epoch");

        // Test 2: One day after epoch (1970-01-02)
        let (year, month, day) = days_to_ymd(1);
        assert_eq!(year, 1970, "Year should be 1970");
        assert_eq!(month, 1, "Month should be January");
        assert_eq!(day, 2, "Day should be 2");

        // Test 3: Known leap year - Feb 29, 2024
        // Calculation: Days from 1970-01-01 to 2024-02-29
        // Method: Count complete years (1970-2023) + days in 2024 (Jan 31 + Feb 29)
        let days_to_2024_feb_29 = calculate_days_to_date(2024, 2, 29);
        let (year, month, day) = days_to_ymd(days_to_2024_feb_29);
        assert_eq!(year, 2024, "Year should be 2024");
        assert_eq!(month, 2, "Month should be February");
        assert_eq!(day, 29, "Day should be 29 (leap day)");

        // Test 4: Non-leap year (2023-02-28, no Feb 29)
        let days_to_2023_feb_28 = calculate_days_to_date(2023, 2, 28);
        let (year, month, day) = days_to_ymd(days_to_2023_feb_28);
        assert_eq!(year, 2023, "Year should be 2023");
        assert_eq!(month, 2, "Month should be February");
        assert_eq!(day, 28, "Day should be 28");

        // Test 5: End of year (2023-12-31)
        let days_to_2023_dec_31 = calculate_days_to_date(2023, 12, 31);
        let (year, month, day) = days_to_ymd(days_to_2023_dec_31);
        assert_eq!(year, 2023, "Year should be 2023");
        assert_eq!(month, 12, "Month should be December");
        assert_eq!(day, 31, "Day should be 31");

        // Test 6: Start of 2024 (2024-01-01)
        let days_to_2024_jan_01 = calculate_days_to_date(2024, 1, 1);
        let (year, month, day) = days_to_ymd(days_to_2024_jan_01);
        assert_eq!(year, 2024, "Year should be 2024");
        assert_eq!(month, 1, "Month should be January");
        assert_eq!(day, 1, "Day should be 1");
    }

    #[test]
    fn test_days_to_ymd_extreme_input() {
        // Test with absurdly large input (cosmic ray corruption scenario)
        let huge_days = u64::MAX / 2; // Very large but won't overflow arithmetic

        // Should return fallback date without panicking
        let (year, month, day) = days_to_ymd(huge_days);

        // Should hit iteration limit and return fallback
        assert_eq!(year, 9999, "Should return max year fallback");
        assert_eq!(month, 12, "Should return December as fallback");
        assert_eq!(day, 31, "Should return last day as fallback");
    }

    #[test]
    fn test_leap_year_calculations() {
        let is_leap_year = |y: u32| -> bool { (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0) };

        // Test standard leap years
        assert!(is_leap_year(2024), "2024 should be leap year");
        assert!(
            is_leap_year(2000),
            "2000 should be leap year (divisible by 400)"
        );
        assert!(is_leap_year(2020), "2020 should be leap year");

        // Test non-leap years
        assert!(!is_leap_year(2023), "2023 should NOT be leap year");
        assert!(
            !is_leap_year(1900),
            "1900 should NOT be leap year (century rule)"
        );
        assert!(
            !is_leap_year(2100),
            "2100 should NOT be leap year (century rule)"
        );
        assert!(!is_leap_year(2001), "2001 should NOT be leap year");
    }

    #[test]
    fn test_century_leap_years() {
        // Test the century rule (divisible by 100 but not 400)
        let is_leap_year = |y: u32| -> bool { (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0) };

        assert!(!is_leap_year(1800), "1800 should NOT be leap year");
        assert!(!is_leap_year(1900), "1900 should NOT be leap year");
        assert!(is_leap_year(2000), "2000 SHOULD be leap year");
        assert!(!is_leap_year(2100), "2100 should NOT be leap year");
        assert!(!is_leap_year(2200), "2200 should NOT be leap year");
        assert!(!is_leap_year(2300), "2300 should NOT be leap year");
        assert!(is_leap_year(2400), "2400 SHOULD be leap year");
    }

    /// Helper function to calculate days from epoch to a specific date
    /// This is used for test validation - it implements the SAME logic as days_to_ymd
    /// but in reverse, so we can verify our function works correctly.
    ///
    /// # Arguments
    /// * `target_year` - Year (e.g., 2024)
    /// * `target_month` - Month (1-12)
    /// * `target_day` - Day (1-31)
    ///
    /// # Returns
    /// * `u64` - Number of days since 1970-01-01
    fn calculate_days_to_date(target_year: u32, target_month: u32, target_day: u32) -> u64 {
        const EPOCH_YEAR: u32 = 1970;

        let is_leap_year = |y: u32| -> bool { (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0) };

        // Count days in complete years from 1970 to target_year - 1
        let mut total_days = 0u64;

        // Bounded loop: maximum (target_year - 1970) iterations
        let year_diff = target_year.saturating_sub(EPOCH_YEAR);
        for year_offset in 0..year_diff {
            let year = EPOCH_YEAR + year_offset;
            let days_in_year = if is_leap_year(year) { 366 } else { 365 };
            total_days += days_in_year;
        }

        // Add days for complete months in target year
        const DAYS_IN_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        const DAYS_IN_MONTH_LEAP: [u32; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

        let days_in_months = if is_leap_year(target_year) {
            &DAYS_IN_MONTH_LEAP
        } else {
            &DAYS_IN_MONTH
        };

        // Add complete months (bounded: max 12 iterations)
        for month_index in 0..(target_month - 1) as usize {
            if month_index < 12 {
                total_days += days_in_months[month_index] as u64;
            }
        }

        // Add remaining days (minus 1 because day 1 is day 0 in our count)
        total_days += (target_day - 1) as u64;

        total_days
    }

    #[test]
    fn test_helper_calculate_days_to_date() {
        // Verify our helper function with known values

        // Epoch start: 0 days
        assert_eq!(
            calculate_days_to_date(1970, 1, 1),
            0,
            "Epoch should be 0 days"
        );

        // One day after epoch
        assert_eq!(
            calculate_days_to_date(1970, 1, 2),
            1,
            "Jan 2, 1970 should be 1 day"
        );

        // End of January 1970
        assert_eq!(
            calculate_days_to_date(1970, 1, 31),
            30,
            "Jan 31, 1970 should be 30 days"
        );

        // Start of February 1970
        assert_eq!(
            calculate_days_to_date(1970, 2, 1),
            31,
            "Feb 1, 1970 should be 31 days"
        );

        // One complete year
        assert_eq!(
            calculate_days_to_date(1971, 1, 1),
            365,
            "Jan 1, 1971 should be 365 days"
        );
    }

    #[test]
    fn test_roundtrip_date_conversion() {
        // Test that converting TO days and back FROM days gives the same result

        let test_dates = [
            (1970, 1, 1),   // Epoch
            (1970, 12, 31), // End of first year
            (2000, 1, 1),   // Y2K
            (2000, 2, 29),  // Leap day
            (2023, 6, 15),  // Random recent date
            (2024, 2, 29),  // Recent leap day
        ];

        for (expected_year, expected_month, expected_day) in test_dates.iter() {
            let days = calculate_days_to_date(*expected_year, *expected_month, *expected_day);
            let (year, month, day) = days_to_ymd(days);

            assert_eq!(
                year, *expected_year,
                "Year mismatch for {}-{:02}-{:02}",
                expected_year, expected_month, expected_day
            );
            assert_eq!(
                month, *expected_month,
                "Month mismatch for {}-{:02}-{:02}",
                expected_year, expected_month, expected_day
            );
            assert_eq!(
                day, *expected_day,
                "Day mismatch for {}-{:02}-{:02}",
                expected_year, expected_month, expected_day
            );
        }
    }
}

// #[cfg(test)]
// mod command_parsing_tests {
//     use super::*;

//     #[test]
//     fn test_parse_movement_with_count() {
//         // Test basic movements
//         assert_eq!(parse_command("j", EditorMode::Normal), Command::MoveDown(1));

//         assert_eq!(
//             parse_command("5j", EditorMode::Normal),
//             Command::MoveDown(5)
//         );

//         assert_eq!(
//             parse_command("10k", EditorMode::Normal),
//             Command::MoveUp(10)
//         );

//         assert_eq!(
//             parse_command("3h", EditorMode::Normal),
//             Command::MoveLeft(3)
//         );

//         assert_eq!(
//             parse_command("7l", EditorMode::Normal),
//             Command::MoveRight(7)
//         );

//         // Test with whitespace
//         assert_eq!(
//             parse_command("  5j  ", EditorMode::Normal),
//             Command::MoveDown(5)
//         );

//         // Test large counts
//         assert_eq!(
//             parse_command("50j", EditorMode::Normal),
//             Command::MoveDown(50)
//         );

//         // Test count capping at 1000
//         assert_eq!(
//             parse_command("9999j", EditorMode::Normal),
//             Command::MoveDown(1000)
//         );
//     }
// }

// #[cfg(test)]
// mod command_parsing_tests {
//     use super::*;

//     #[test]
//     fn test_parse_movement_with_count() {
//         // Test basic movements
//         assert_eq!(parse_command("j", EditorMode::Normal), Command::MoveDown(1));

//         assert_eq!(
//             parse_command("5j", EditorMode::Normal),
//             Command::MoveDown(5)
//         );

//         assert_eq!(
//             parse_command("10k", EditorMode::Normal),
//             Command::MoveUp(10)
//         );

//         assert_eq!(
//             parse_command("3h", EditorMode::Normal),
//             Command::MoveLeft(3)
//         );

//         assert_eq!(
//             parse_command("7l", EditorMode::Normal),
//             Command::MoveRight(7)
//         );

//         // Test with whitespace
//         assert_eq!(
//             parse_command("  5j  ", EditorMode::Normal),
//             Command::MoveDown(5)
//         );

//         // Test large counts (capped at 100 in your current code)
//         assert_eq!(
//             parse_command("50j", EditorMode::Normal),
//             Command::MoveDown(50)
//         );

//         // Test count capping at 100
//         assert_eq!(
//             parse_command("9999j", EditorMode::Normal),
//             Command::MoveDown(100) // Changed from 1000 to 100
//         );
//         // Test basic movements
//         assert_eq!(parse_command("j", EditorMode::Normal), Command::MoveDown(1));

//         assert_eq!(
//             parse_command("5j", EditorMode::Normal),
//             Command::MoveDown(5)
//         );

//         assert_eq!(
//             parse_command("10k", EditorMode::Normal),
//             Command::MoveUp(10)
//         );

//         // Test large counts
//         assert_eq!(
//             parse_command("1000j", EditorMode::Normal),
//             Command::MoveDown(1000)
//         );

//         assert_eq!(
//             parse_command("50000k", EditorMode::Normal),
//             Command::MoveUp(50000)
//         );
//     }
// }
#[cfg(test)]
mod test_parse_movement {
    use super::*;
    #[test]
    fn test_parse_movement_with_count() {
        let mut state = EditorState::new();
        // Test basic movements
        assert_eq!(
            state.parse_command("j", EditorMode::Normal),
            Command::MoveDown(1)
        );

        assert_eq!(
            state.parse_command("5j", EditorMode::Normal),
            Command::MoveDown(5)
        );

        assert_eq!(
            state.parse_command("10k", EditorMode::Normal),
            Command::MoveUp(10)
        );

        assert_eq!(
            state.parse_command("3h", EditorMode::Normal),
            Command::MoveLeft(3)
        );

        assert_eq!(
            state.parse_command("7l", EditorMode::Normal),
            Command::MoveRight(7)
        );

        // Test with whitespace
        assert_eq!(
            state.parse_command("  5j  ", EditorMode::Normal),
            Command::MoveDown(5)
        );

        // Test with whitespace
        assert_eq!(
            state.parse_command("  10j  ", EditorMode::Normal),
            Command::MoveDown(10)
        );

        // // Test large counts
        // assert_eq!(
        //     parse_command("1000j", EditorMode::Normal),
        //     Command::MoveDown(1000)
        // );

        // // Test very large counts (capped at usize::MAX / 2)
        // assert_eq!(
        //     parse_command("50000k", EditorMode::Normal),
        //     Command::MoveUp(50000)
        // );
        //
        //
        //
    }
}
// #[test]
// fn test_cursor_at_eol() {
//     // Create a simple test file
//     let test_files = create_test_files_with_id("cursor_eol").unwrap();
//     let test_file = &test_files[0]; // basic_short.txt

//     // Create session directory
//     let session_timestamp = FixedSize32Timestamp::from_str("24_01_01_00_00_00").unwrap();

//     // Initialize state
//     let mut state = EditorState::new();

//     // Initialize session directory
//     initialize_session_directory(&mut state, session_timestamp).unwrap();

//     let session_dir = state.session_directory_path.as_ref().unwrap();
//     let read_copy =
//         create_a_readcopy_of_file(test_file, session_dir, "24_01_01_00_00_00".to_string()).unwrap();

//     state.read_copy_path = Some(read_copy.clone());
//     state.original_file_path = Some(test_file.clone());

//     // Build window
//     build_windowmap_nowrap(&mut state, &read_copy).unwrap();

//     // Test 1: Can we get position at start of line 1?
//     let pos_start = state.window_map.get_file_position(0, 0).unwrap();
//     println!("Position at (0,0): {:?}", pos_start);
//     assert!(pos_start.is_some(), "Should have position at line start");

//     // Test 2: Can we get position at END of line 1?
//     // Line 1 is "Line 1: Hello, world!" (21 chars)
//     // After line number "1 " (2 chars), text starts at col 2
//     // Text is 21 chars, so last char is at col 2+20=22
//     // EOL position should be at col 23

//     let last_char_col = 22; // Position of last visible character
//     let eol_col = 23; // Position AFTER last character

//     let pos_last_char = state
//         .window_map
//         .get_file_position(0, last_char_col)
//         .unwrap();
//     println!(
//         "Position at last char (0,{}): {:?}",
//         last_char_col, pos_last_char
//     );

//     let pos_eol = state.window_map.get_file_position(0, eol_col).unwrap();
//     println!("Position at EOL (0,{}): {:?}", eol_col, pos_eol);

//     // This will tell us if EOL mapping is working
//     assert!(pos_eol.is_some(), "Should have position at end of line");
// }
// #[test]
// fn test_move_cursor_to_eol() {
//     let test_files = create_test_files_with_id("cursor_move_eol").unwrap();
//     let test_file = &test_files[0];

//     let session_timestamp = FixedSize32Timestamp::from_str("24_01_01_00_00_01").unwrap();
//     let mut state = EditorState::new();
//     initialize_session_directory(&mut state, session_timestamp).unwrap();

//     let session_dir = state.session_directory_path.as_ref().unwrap();
//     let read_copy =
//         create_a_readcopy_of_file(test_file, session_dir, "24_01_01_00_00_01".to_string()).unwrap();

//     state.read_copy_path = Some(read_copy.clone());
//     state.original_file_path = Some(test_file.clone());
//     build_windowmap_nowrap(&mut state, &read_copy).unwrap();

//     // Start at beginning of line
//     state.cursor.row = 0;
//     state.cursor.col = 0;

//     // Try to move right past the end of line
//     let command = Command::MoveRight(100); // Move way past end
//     let result = execute_command(&mut state, command);

//     println!(
//         "After moving right: cursor at ({}, {})",
//         state.cursor.row, state.cursor.col
//     );

//     // Check we can get file position at cursor
//     let pos = state
//         .window_map
//         .get_file_position(state.cursor.row, state.cursor.col);
//     println!("File position at cursor: {:?}", pos);

//     assert!(result.is_ok(), "Move command should succeed");
//     assert!(
//         pos.unwrap().is_some(),
//         "Should have valid position at cursor"
//     );
// }

// #[test]
// fn test_insert_at_eol() {
//     let test_files = create_test_files_with_id("insert_eol").unwrap();
//     let test_file = &test_files[0];

//     let session_timestamp = FixedSize32Timestamp::from_str("24_01_01_00_00_02").unwrap();
//     let mut state = EditorState::new();
//     initialize_session_directory(&mut state, session_timestamp).unwrap();

//     let session_dir = state.session_directory_path.as_ref().unwrap();
//     let read_copy =
//         create_a_readcopy_of_file(test_file, session_dir, "24_01_01_00_00_02".to_string()).unwrap();

//     state.read_copy_path = Some(read_copy.clone());
//     state.original_file_path = Some(test_file.clone());
//     build_windowmap_nowrap(&mut state, &read_copy).unwrap();

//     // Move to end of first line
//     state.cursor.row = 0;
//     state.cursor.col = 23; // After "Line 1: Hello, world!"

//     // Enter insert mode
//     state.mode = EditorMode::Insert;

//     // Try to insert text at EOL
//     let text = " ADDED";
//     let result = insert_text_chunk_at_cursor_position(&mut state, &read_copy, text.as_bytes());

//     println!("Insert result: {:?}", result);
//     assert!(result.is_ok(), "Should be able to insert at EOL");

//     // Verify text was added
//     let mut file = std::fs::File::open(&read_copy).unwrap();
//     let mut contents = String::new();
//     std::io::Read::read_to_string(&mut file, &mut contents).unwrap();

//     println!("File after insert:\n{}", contents);
//     assert!(
//         contents.contains("Hello, world! ADDED"),
//         "Text should be appended"
//     );
// }

#[test]
fn test_eol_mapping_simple() {
    use std::env;
    // use std::path::PathBuf;

    // Use existing test file
    let cwd = env::current_dir().unwrap();
    let test_file = cwd.join("test_files/basic_short.txt");

    // Create unique session dir that won't conflict
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();
    let session_ts =
        FixedSize32Timestamp::from_str(&format!("24_01_01_{:06}", timestamp % 1000000)).unwrap();

    let mut state = EditorState::new();
    initialize_session_directory(&mut state, session_ts).unwrap();

    let session_dir = state.session_directory_path.as_ref().unwrap();
    let read_copy =
        create_a_readcopy_of_file(&test_file, session_dir, format!("test_{}", timestamp)).unwrap();

    state.read_copy_path = Some(read_copy.clone());
    state.original_file_path = Some(test_file.clone());

    // Build window
    let lines = build_windowmap_nowrap(&mut state, &read_copy).unwrap();
    println!("Built window with {} lines", lines);

    // Line 1 is "Line 1: Hello, world!" - check what columns are mapped
    for col in 0..30 {
        match state.window_map.get_file_position(0, col) {
            Ok(Some(pos)) => println!(
                "Col {} -> byte_offset: {}, byte_in_line: {}",
                col, pos.byte_offset, pos.byte_in_line
            ),
            Ok(None) => println!("Col {} -> None (unmapped)", col),
            Err(e) => println!("Col {} -> Error: {}", col, e),
        }
    }
}

#[test]
fn test_cursor_movement_to_eol() {
    use std::env;
    // use std::path::PathBuf;

    let cwd = env::current_dir().unwrap();
    let test_file = cwd.join("test_files/basic_short.txt");

    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();
    let session_ts =
        FixedSize32Timestamp::from_str(&format!("24_01_01_{:06}", timestamp % 1000000)).unwrap();

    let mut state = EditorState::new();
    initialize_session_directory(&mut state, session_ts).unwrap();

    let session_dir = state.session_directory_path.as_ref().unwrap();
    let read_copy =
        create_a_readcopy_of_file(&test_file, session_dir, format!("test_{}", timestamp)).unwrap();

    state.read_copy_path = Some(read_copy.clone());
    state.original_file_path = Some(test_file.clone());
    build_windowmap_nowrap(&mut state, &read_copy).unwrap();

    // Start at beginning
    state.cursor.row = 0;
    state.cursor.col = 0;

    println!(
        "Starting cursor: ({}, {})",
        state.cursor.row, state.cursor.col
    );

    // Try to move right 25 times (should land at col 23, the EOL position)
    let result = execute_command(&mut state, Command::MoveRight(25));

    println!(
        "After MoveRight(25): cursor at ({}, {})",
        state.cursor.row, state.cursor.col
    );
    println!("Command result: {:?}", result);

    // Can we get file position at cursor?
    match state
        .window_map
        .get_file_position(state.cursor.row, state.cursor.col)
    {
        Ok(Some(pos)) => println!(
            "SUCCESS: File position at cursor: byte_offset={}, byte_in_line={}",
            pos.byte_offset, pos.byte_in_line
        ),
        Ok(None) => println!("ERROR: No file position at cursor!"),
        Err(e) => println!("ERROR getting position: {}", e),
    }

    assert!(result.is_ok());
}

#[test]
fn test_cursor_movement_to_eol2() {
    /*
    THis is trying to test if the cursor position stops
    at EOL.
    As yet... is not clear what the behavior 'should' be.

     */
    use std::env;
    // use std::path::PathBuf;

    let cwd = env::current_dir().unwrap();
    let test_file = cwd.join("test_files/basic_short.txt");

    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();
    let session_ts =
        FixedSize32Timestamp::from_str(&format!("24_01_01_{:06}", timestamp % 1000000)).unwrap();

    let mut state = EditorState::new();
    initialize_session_directory(&mut state, session_ts).unwrap();

    let session_dir = state.session_directory_path.as_ref().unwrap();
    let read_copy =
        create_a_readcopy_of_file(&test_file, session_dir, format!("test_{}", timestamp)).unwrap();

    state.read_copy_path = Some(read_copy.clone());
    state.original_file_path = Some(test_file.clone());
    build_windowmap_nowrap(&mut state, &read_copy).unwrap();

    // Start at FIRST VALID position (col 2, after line number)
    state.cursor.row = 0;
    state.cursor.col = 0;

    println!(
        "Starting cursor: ({}, {})",
        state.cursor.row, state.cursor.col
    );

    // Try to move right 25 times (should land at col 23, the EOL position)
    let result = execute_command(&mut state, Command::MoveRight(25));

    println!(
        "After MoveRight(25): cursor at ({}, {})",
        state.cursor.row, state.cursor.col
    );
    println!("Command result: {:?}", result);

    // Can we get file position at cursor?
    match state
        .window_map
        .get_file_position(state.cursor.row, state.cursor.col)
    {
        Ok(Some(pos)) => println!(
            "SUCCESS: File position at cursor: byte_offset={}, byte_in_line={}",
            pos.byte_offset, pos.byte_in_line
        ),
        Ok(None) => println!("ERROR: No file position at cursor!"),
        Err(e) => println!("ERROR getting position: {}", e),
    }

    /*
    This line:
        "1 Line 1: Hello, world!"

    contais 23 characters.

    One reason to not force cursor to line length
    is that you can't go down past a shorter line. (maybe)
     *
     */

    assert!(result.is_ok());
    assert_eq!(
        state.cursor.col,
        25, // 23? What should it be? TODO
        "Cursor should be at EOL position (col 23)"
    );
}

#[test]
fn test_insert_at_eol_works() {
    use std::env;
    // use std::path::PathBuf;

    let cwd = env::current_dir().unwrap();
    let test_file = cwd.join("test_files/basic_short.txt");

    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();
    let session_ts =
        FixedSize32Timestamp::from_str(&format!("24_01_01_{:06}", timestamp % 1000000)).unwrap();

    let mut state = EditorState::new();
    initialize_session_directory(&mut state, session_ts).unwrap();

    let session_dir = state.session_directory_path.as_ref().unwrap();
    let read_copy =
        create_a_readcopy_of_file(&test_file, session_dir, format!("test_{}", timestamp)).unwrap();

    state.read_copy_path = Some(read_copy.clone());
    state.original_file_path = Some(test_file.clone());
    build_windowmap_nowrap(&mut state, &read_copy).unwrap();

    // Move to EOL
    state.cursor.row = 0;
    state.cursor.col = 23;
    state.mode = EditorMode::Insert;

    println!(
        "Cursor at EOL: ({}, {})",
        state.cursor.row, state.cursor.col
    );

    // Insert text at EOL
    let text = " ADDED";
    let result = insert_text_chunk_at_cursor_position(&mut state, &read_copy, text.as_bytes());

    println!("Insert result: {:?}", result);

    // Read file and check
    let contents = std::fs::read_to_string(&read_copy).unwrap();
    println!(
        "First line after insert: {}",
        contents.lines().next().unwrap()
    );

    assert!(result.is_ok(), "Insert should succeed");
    assert!(
        contents.contains("world! ADDED"),
        "Text should be appended to line"
    );
}

// ============================================================================
// TEST insert_file
// ============================================================================

#[cfg(test)]
mod insert_file_tests {
    use super::*;
    use std::fs;
    use std::io::Read;
    use std::io::Write;

    /// Creates test_files directory and returns path
    ///
    /// # Returns
    /// Absolute path to test_files/ directory in current working directory
    fn setup_test_dir() -> io::Result<PathBuf> {
        let test_dir = std::env::current_dir()?.join("test_files");
        fs::create_dir_all(&test_dir)?;
        Ok(test_dir)
    }

    /// Removes test file if it exists
    ///
    /// Ignores errors (file may not exist)
    fn cleanup_test_file(path: &Path) {
        let _ = fs::remove_file(path);
    }

    /// Test: File reading in 256-byte chunks (simulates bucket brigade)
    ///
    /// Verifies:
    /// - Small file (< 256 bytes) read in one chunk
    /// - Total bytes read equals file size
    /// - EOF detected correctly
    #[test]
    fn test_read_small_file_in_chunks() -> io::Result<()> {
        let test_dir = setup_test_dir()?;
        let source_path = test_dir.join("small_source.txt");

        // Create small test file
        let content = "Hello, this is a small test file.\nLine 2\nLine 3";
        {
            let mut file = File::create(&source_path)?;
            file.write_all(content.as_bytes())?;
            file.flush()?;
        }

        // Read in 256-byte chunks
        let mut file = File::open(&source_path)?;
        let mut buffer = [0u8; 256];
        let mut total_read = 0;
        let mut chunk_count = 0;

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break; // EOF
            }
            total_read += bytes_read;
            chunk_count += 1;

            // Verify chunk doesn't exceed buffer
            assert!(bytes_read <= 256);
        }

        // Verify correct amount read
        assert_eq!(total_read, content.len());
        assert_eq!(chunk_count, 1); // Small file = one chunk

        cleanup_test_file(&source_path);
        Ok(())
    }

    /// Test: Large file requires multiple chunks
    ///
    /// Verifies:
    /// - File > 256 bytes splits into multiple chunks
    /// - All chunks read correctly
    /// - Total bytes equals original file size
    #[test]
    fn test_read_large_file_multiple_chunks() -> io::Result<()> {
        let test_dir = setup_test_dir()?;
        let source_path = test_dir.join("large_source.txt");

        // Create file larger than one chunk (1000 bytes)
        let content = "A".repeat(1000);
        {
            let mut file = File::create(&source_path)?;
            file.write_all(content.as_bytes())?;
            file.flush()?;
        }

        // Read in 256-byte chunks
        let mut file = File::open(&source_path)?;
        let mut buffer = [0u8; 256];
        let mut chunks = Vec::new();

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            chunks.push(bytes_read);
            assert!(bytes_read <= 256);
        }

        // Should have multiple chunks: 1000 / 256 = 3.9, so 4 chunks
        assert!(
            chunks.len() >= 4,
            "Expected at least 4 chunks, got {}",
            chunks.len()
        );

        // Last chunk should be partial: 1000 % 256 = 232
        assert_eq!(*chunks.last().unwrap(), 232);

        // Sum should equal original content
        let total: usize = chunks.iter().sum();
        assert_eq!(total, 1000);

        cleanup_test_file(&source_path);
        Ok(())
    }

    /// Test: Empty file handling
    ///
    /// Verifies:
    /// - Empty file returns 0 bytes on first read
    /// - No chunks processed
    /// - No errors occur
    #[test]
    fn test_read_empty_file() -> io::Result<()> {
        let test_dir = setup_test_dir()?;
        let source_path = test_dir.join("empty_source.txt");

        // Create empty file
        {
            let _file = File::create(&source_path)?;
        }

        // Verify it's empty
        let metadata = fs::metadata(&source_path)?;
        assert_eq!(metadata.len(), 0);

        // Read should immediately return 0 (EOF)
        let mut file = File::open(&source_path)?;
        let mut buffer = [0u8; 256];
        let bytes_read = file.read(&mut buffer)?;

        assert_eq!(bytes_read, 0); // Immediate EOF

        cleanup_test_file(&source_path);
        Ok(())
    }

    /// Test: File existence checking
    ///
    /// Verifies:
    /// - Non-existent file is detected
    /// - No panic, no crash
    #[test]
    fn test_nonexistent_file() {
        let test_dir = std::env::current_dir()
            .expect("Cannot get cwd")
            .join("test_files");
        let nonexistent = test_dir.join("this_file_does_not_exist.txt");

        // Verify file doesn't exist
        assert!(!nonexistent.exists());
    }

    /// Test: Chunk boundary with multi-byte UTF-8
    ///
    /// Verifies:
    /// - Multi-byte characters split across chunk boundaries
    /// - Bytes read correctly even if UTF-8 is incomplete at boundary
    /// - Total bytes equals file size
    #[test]
    fn test_chunk_boundary_utf8() -> io::Result<()> {
        let test_dir = setup_test_dir()?;
        let source_path = test_dir.join("utf8_boundary.txt");

        // Create file with multi-byte UTF-8 that will likely split across 256-byte boundary
        // Using 3-byte UTF-8 characters (e.g., '€' = E2 82 AC)
        let content = "€".repeat(100); // 300 bytes total (100 * 3 bytes each)

        {
            let mut file = File::create(&source_path)?;
            file.write_all(content.as_bytes())?;
            file.flush()?;
        }

        // Read in chunks
        let mut file = File::open(&source_path)?;
        let mut buffer = [0u8; 256];
        let mut total_read = 0;
        let mut chunk_count = 0;

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            total_read += bytes_read;
            chunk_count += 1;
        }

        // Verify total bytes correct (300 bytes)
        assert_eq!(total_read, 300);

        // Should be 2 chunks: 256 + 44
        assert_eq!(chunk_count, 2);

        cleanup_test_file(&source_path);
        Ok(())
    }

    /// Test: Binary file handling
    ///
    /// Verifies:
    /// - Binary data (non-UTF-8) read correctly
    /// - No UTF-8 decoding errors (we work at byte level)
    /// - All bytes preserved
    #[test]
    fn test_binary_file() -> io::Result<()> {
        let test_dir = setup_test_dir()?;
        let source_path = test_dir.join("binary_file.bin");

        // Create binary file with non-UTF-8 bytes
        let binary_data: Vec<u8> = (0..=255).collect(); // All possible byte values

        {
            let mut file = File::create(&source_path)?;
            file.write_all(&binary_data)?;
            file.flush()?;
        }

        // Read in chunks
        let mut file = File::open(&source_path)?;
        let mut buffer = [0u8; 256];
        let mut total_read = 0;

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            total_read += bytes_read;
        }

        // Should read all 256 bytes in one chunk
        assert_eq!(total_read, 256);

        cleanup_test_file(&source_path);
        Ok(())
    }

    /// Test: Exact chunk size (256 bytes)
    ///
    /// Verifies:
    /// - File exactly 256 bytes requires one full chunk
    /// - Second read returns 0 (EOF)
    #[test]
    fn test_exact_chunk_size() -> io::Result<()> {
        let test_dir = setup_test_dir()?;
        let source_path = test_dir.join("exact_256.txt");

        // Create file exactly 256 bytes
        let content = "X".repeat(256);

        {
            let mut file = File::create(&source_path)?;
            file.write_all(content.as_bytes())?;
            file.flush()?;
        }

        let mut file = File::open(&source_path)?;
        let mut buffer = [0u8; 256];

        // First read should get all 256 bytes
        let first_read = file.read(&mut buffer)?;
        assert_eq!(first_read, 256);

        // Second read should be EOF
        let second_read = file.read(&mut buffer)?;
        assert_eq!(second_read, 0);

        cleanup_test_file(&source_path);
        Ok(())
    }
}
