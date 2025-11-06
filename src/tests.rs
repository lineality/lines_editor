// tests.rs (keen this in src/ with main.rs)

#[cfg(test)]
use crate::lines_editor_module::double_width::is_double_width;

#[cfg(test)]
use crate::lines_editor_module::*;

#[cfg(test)]
use std::env;

#[cfg(test)]
use std::fs::File;

// #[cfg(test)]
// use std::io::BufRead;

// #[cfg(test)]
// use std::io::BufReader;

#[cfg(test)]
use std::io::{self};

#[cfg(test)]
use std::path::{Path, PathBuf};

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

/// Diagnostic function to print contents of test files
#[cfg(test)]
// fn print_test_file_contents(file_path: &Path) -> io::Result<()> {
//     println!("=== File Contents: {} ===", file_path.display());
//     let file = File::open(file_path)?;
//     let reader = BufReader::new(file);

//     for (index, line) in reader.lines().enumerate() {
//         let line = line?;
//         println!("{:4}: {}", index + 1, line);
//     }

//     // Get file metadata
//     let metadata = std::fs::metadata(file_path)?;
//     println!("\nFile size: {} bytes", metadata.len());

//     Ok(())
// }
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

// collides with assert test in function
// // Modify the test to include more diagnostics
// #[test]
// fn test_build_windowmap_nowrap_basic() -> io::Result<()> {
//     // Create test files
//     let test_files = create_test_files_with_id("test_build_windowmap_nowrap_basic")?;
//     let basic_file = &test_files[0]; // basic_short.txt

//     // Print file contents for debugging
//     print_test_file_contents(basic_file)?;

//     // Create editor state
//     let mut state = EditorState::new();
//     state.line_count_at_top_of_window = 0;
//     state.file_position_of_topline_start = 0;
//     state.tui_window_horizontal_utf8txt_line_char_offset = 0;

//     // Debug: print file path and existence
//     println!("Test file path: {:?}", basic_file);
//     println!("File exists: {}", basic_file.exists());

//     // Verify file is readable
//     let file = File::open(basic_file)?;
//     let reader = BufReader::new(file);
//     let line_count = reader.lines().count();
//     println!("Line count in file: {}", line_count);

//     // Build window
//     let result = build_windowmap_nowrap(&mut state, basic_file);

//     // Debug: print detailed error if failed
//     if let Err(ref e) = result {
//         println!("Build window failed: {}", e);
//     }

//     assert!(result.is_ok(), "Should build window successfully");

//     let lines_processed = result.unwrap();
//     println!("Lines processed: {}", lines_processed);

//     // Debug: print buffer contents
//     for i in 0..5 {
//         if state.display_utf8txt_buffer_lengths[i] > 0 {
//             let content =
//                 &state.utf8_txt_display_buffers[i][..state.display_utf8txt_buffer_lengths[i]];
//             println!("Row {}: {:?}", i, String::from_utf8_lossy(content));
//         }
//     }

//     assert!(lines_processed > 0, "Should process at least one line");

//     // Verify first line has content
//     assert!(
//         state.display_utf8txt_buffer_lengths[0] > 0,
//         "First row should have content"
//     );

//     // Verify line number "1 " appears at start
//     let first_row = &state.utf8_txt_display_buffers[0];
//     assert_eq!(first_row[1], b'1', "Should start with line number 1");
//     assert_eq!(first_row[0], b' ', "Should have space after line number");

//     // Verify WindowMap has been populated
//     let map_entry = state.window_map.get_row_col_file_position(0, 3).unwrap();
//     assert!(map_entry.is_some(), "Character position should be mapped");

//     Ok(())
// }

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

        let display_width = calculate_display_width(content).expect("Should calculate width");
        assert_eq!(display_width, 10, "Should be 10 display columns total");

        // Now test with the editor
        let mut state = EditorState::new();
        state.line_count_at_top_of_window = 0;
        state.file_position_of_topline_start = 0;
        state.tui_window_horizontal_utf8txt_line_char_offset = 0;

        let result = build_windowmap_nowrap(&mut state, &test_path);
        assert!(result.is_ok(), "Build window should succeed");

        // The display buffer should contain the line with line number
        let first_row_len = state.display_utf8txt_buffer_lengths[0];
        assert!(first_row_len > 0, "First row should have content");

        // Verify content is valid UTF-8
        let first_row_content = &state.utf8_txt_display_buffers[0][..first_row_len];
        let first_row_str = std::str::from_utf8(first_row_content).expect("Should be valid UTF-8");

        assert!(
            first_row_str.contains("世界"),
            "Should contain Chinese characters"
        );

        // Display width should fit within terminal
        let row_display_width =
            calculate_display_width(first_row_str).expect("Should calculate display width");
        assert!(
            row_display_width <= 80,
            "Display width {} should not exceed terminal width 80",
            row_display_width
        );

        Ok(())
    }
}

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

#[cfg(test)]
mod test_parse_movement {
    use super::*;
    #[test]
    fn test_parse_movement_with_count() {
        let mut state = EditorState::new();
        // Test basic movements
        assert_eq!(
            state.parse_commands_for_normal_visualselect_modes("j", EditorMode::Normal),
            Command::MoveDown(1)
        );

        assert_eq!(
            state.parse_commands_for_normal_visualselect_modes("5j", EditorMode::Normal),
            Command::MoveDown(5)
        );

        assert_eq!(
            state.parse_commands_for_normal_visualselect_modes("10k", EditorMode::Normal),
            Command::MoveUp(10)
        );

        assert_eq!(
            state.parse_commands_for_normal_visualselect_modes("3h", EditorMode::Normal),
            Command::MoveLeft(3)
        );

        assert_eq!(
            state.parse_commands_for_normal_visualselect_modes("7l", EditorMode::Normal),
            Command::MoveRight(7)
        );

        // Test with whitespace
        assert_eq!(
            state.parse_commands_for_normal_visualselect_modes("  5j  ", EditorMode::Normal),
            Command::MoveDown(5)
        );

        // Test with whitespace
        assert_eq!(
            state.parse_commands_for_normal_visualselect_modes("  10j  ", EditorMode::Normal),
            Command::MoveDown(10)
        );
    }
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

#[cfg(test)]
mod hex_display_tests {
    use super::*;

    /// Test byte_to_display_char covers all ranges
    #[cfg(test)]
    #[test]
    fn test_byte_to_display_char_printable_ascii() {
        // Printable ASCII
        assert_eq!(byte_to_display_char(0x41), 'A');
        assert_eq!(byte_to_display_char(0x61), 'a');
        assert_eq!(byte_to_display_char(0x30), '0');
        assert_eq!(byte_to_display_char(0x7E), '~'); // Last printable
    }

    #[cfg(test)]
    #[test]
    fn test_byte_to_display_char_control_chars() {
        // Control characters
        assert_eq!(byte_to_display_char(0x09), '␉'); // Tab
        assert_eq!(byte_to_display_char(0x0A), '␊'); // LF
        assert_eq!(byte_to_display_char(0x0D), '␍'); // CR
        assert_eq!(byte_to_display_char(0x20), '⎕'); // Space (visible)
    }

    #[cfg(test)]
    #[test]
    fn test_byte_to_display_char_unprintable() {
        // Unprintable bytes
        assert_eq!(byte_to_display_char(0x00), '▚'); // NULL
        assert_eq!(byte_to_display_char(0x1F), '▚'); // Unit separator
        assert_eq!(byte_to_display_char(0x7F), '▚'); // DEL
        assert_eq!(byte_to_display_char(0x80), '▚'); // High byte
        assert_eq!(byte_to_display_char(0xFF), '▚'); // Max byte
    }

    #[cfg(test)]
    #[test]
    fn test_hex_cursor_row_col_calculation() {
        let mut cursor = HexCursor::new();

        // First row, first column
        assert_eq!(cursor.current_row(), 0);
        assert_eq!(cursor.current_col(), 0);

        // First row, middle
        cursor.byte_offset_linear_file_absolute_position = 13;
        assert_eq!(cursor.current_row(), 0);
        assert_eq!(cursor.current_col(), 13);

        // First row, last column (25)
        cursor.byte_offset_linear_file_absolute_position = 25;
        assert_eq!(cursor.current_row(), 0);
        assert_eq!(cursor.current_col(), 25);

        // Second row, first column
        cursor.byte_offset_linear_file_absolute_position = 26;
        assert_eq!(cursor.current_row(), 1);
        assert_eq!(cursor.current_col(), 0);

        // Second row, middle
        cursor.byte_offset_linear_file_absolute_position = 39;
        assert_eq!(cursor.current_row(), 1);
        assert_eq!(cursor.current_col(), 13);
    }

    #[cfg(test)]
    #[test]
    fn test_hex_cursor_boundaries() {
        let cursor = HexCursor::new();

        // Verify constants
        assert_eq!(cursor.bytes_per_row, 26);
        assert_eq!(cursor.byte_offset_linear_file_absolute_position, 0);

        // Column should never exceed bytes_per_row - 1
        let test_offset = 1000;
        let test_cursor = HexCursor {
            byte_offset_linear_file_absolute_position: test_offset,
            bytes_per_row: 26,
        };
        assert!(test_cursor.current_col() < 26);
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod pasty_file_append_tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    /// Helper function to create a test file with specific content
    ///
    /// # Arguments
    /// * `path` - Path where test file should be created
    /// * `content` - Byte content to write to file
    ///
    /// # Returns
    /// * `Ok(())` if file created successfully
    /// * `Err(io::Error)` if creation failed
    fn create_test_file(path: &Path, content: &[u8]) -> io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(content)?;
        file.flush()?;
        Ok(())
    }

    /// Helper function to read entire file content into Vec<u8>
    ///
    /// # Arguments
    /// * `path` - Path to file to read
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` containing file content
    /// * `Err(io::Error)` if read failed
    fn read_file_content(path: &Path) -> io::Result<Vec<u8>> {
        fs::read(path)
    }

    /// Test: Copy a simple range of bytes from middle of source file
    #[test]
    fn test_append_bytes_simple_range() {
        let temp_dir = std::env::temp_dir();
        let source_path = temp_dir.join("test_source_simple.txt");
        let target_path = temp_dir.join("test_target_simple.txt");

        // Create source file with content "ABCDEFGHIJ" (10 bytes)
        create_test_file(&source_path, b"ABCDEFGHIJ").expect("Failed to create source");

        // Create empty target file
        create_test_file(&target_path, b"").expect("Failed to create target");

        // Copy bytes 2-5 (zero-indexed, inclusive): should copy "CDEF"
        // Position 2='C', 3='D', 4='E', 5='F'
        let result = append_bytes_from_file_to_file(&source_path, 2, 5, &target_path);

        #[cfg(test)]
        assert!(result.is_ok(), "Function should succeed");

        // Verify target file contains exactly "CDEF"
        let content = read_file_content(&target_path).expect("Failed to read target");

        #[cfg(test)]
        assert_eq!(content, b"CDEF", "Target should contain copied bytes");

        // Cleanup
        let _ = fs::remove_file(&source_path);
        let _ = fs::remove_file(&target_path);
    }

    /// Test: Append bytes to target file that already has existing content
    #[test]
    fn test_append_to_existing_content() {
        let temp_dir = std::env::temp_dir();
        let source_path = temp_dir.join("test_source_append.txt");
        let target_path = temp_dir.join("test_target_append.txt");

        // Create source file
        create_test_file(&source_path, b"ABCDEFGHIJ").expect("Failed to create source");

        // Create target file with existing content
        create_test_file(&target_path, b"EXISTING").expect("Failed to create target");

        // Append bytes 0-2: "ABC"
        let result = append_bytes_from_file_to_file(&source_path, 0, 2, &target_path);

        #[cfg(test)]
        assert!(result.is_ok(), "Function should succeed");

        // Verify target file contains "EXISTING" + "ABC" = "EXISTINGABC"
        let content = read_file_content(&target_path).expect("Failed to read target");

        #[cfg(test)]
        assert_eq!(
            content, b"EXISTINGABC",
            "Target should contain original content plus appended bytes"
        );

        // Cleanup
        let _ = fs::remove_file(&source_path);
        let _ = fs::remove_file(&target_path);
    }

    /// Test: Source file doesn't exist - should return Ok gracefully (no-op)
    #[test]
    fn test_source_file_not_exists() {
        let temp_dir = std::env::temp_dir();
        let source_path = temp_dir.join("test_nonexistent_source.txt");
        let target_path = temp_dir.join("test_target_nonesource.txt");

        // Ensure source doesn't exist
        let _ = fs::remove_file(&source_path);

        // Try to copy - should return Ok with no action (graceful no-op)
        let result = append_bytes_from_file_to_file(&source_path, 0, 10, &target_path);

        #[cfg(test)]
        assert!(result.is_ok(), "Should return Ok when source doesn't exist");

        // Cleanup
        let _ = fs::remove_file(&target_path);
    }

    /// Test: Target file doesn't exist - should be created automatically
    #[test]
    fn test_target_file_created() {
        let temp_dir = std::env::temp_dir();
        let source_path = temp_dir.join("test_source_create.txt");
        let target_path = temp_dir.join("test_target_create_new.txt");

        // Create source file
        create_test_file(&source_path, b"ABCDEFGHIJ").expect("Failed to create source");

        // Ensure target doesn't exist
        let _ = fs::remove_file(&target_path);

        // Copy bytes - should create target file automatically
        let result = append_bytes_from_file_to_file(&source_path, 0, 4, &target_path);

        #[cfg(test)]
        assert!(result.is_ok(), "Function should succeed");

        // Verify target was created and contains correct data
        #[cfg(test)]
        assert!(target_path.exists(), "Target file should be created");

        let content = read_file_content(&target_path).expect("Failed to read target");

        #[cfg(test)]
        assert_eq!(content, b"ABCDE", "Target should contain copied bytes");

        // Cleanup
        let _ = fs::remove_file(&source_path);
        let _ = fs::remove_file(&target_path);
    }

    /// Test: Start position beyond file size - should return Ok gracefully
    #[test]
    fn test_start_position_beyond_file() {
        let temp_dir = std::env::temp_dir();
        let source_path = temp_dir.join("test_source_beyond.txt");
        let target_path = temp_dir.join("test_target_beyond.txt");

        // Create small source file (3 bytes)
        create_test_file(&source_path, b"ABC").expect("Failed to create source");

        // Create target file
        create_test_file(&target_path, b"").expect("Failed to create target");

        // Try to copy from position 100 (way beyond file size of 3 bytes)
        // Should seek successfully but immediately hit EOF when trying to read
        let result = append_bytes_from_file_to_file(&source_path, 100, 110, &target_path);

        #[cfg(test)]
        assert!(result.is_ok(), "Should return Ok when start is beyond EOF");

        // Verify target is still empty (no bytes were copied)
        let content = read_file_content(&target_path).expect("Failed to read target");

        #[cfg(test)]
        assert_eq!(content.len(), 0, "Target should remain empty");

        // Cleanup
        let _ = fs::remove_file(&source_path);
        let _ = fs::remove_file(&target_path);
    }

    /// Test: End position beyond file size - should copy until EOF then stop
    #[test]
    fn test_end_position_beyond_file() {
        let temp_dir = std::env::temp_dir();
        let source_path = temp_dir.join("test_source_eof.txt");
        let target_path = temp_dir.join("test_target_eof.txt");

        // Create source file with 5 bytes: "ABCDE"
        create_test_file(&source_path, b"ABCDE").expect("Failed to create source");

        // Create target file
        create_test_file(&target_path, b"").expect("Failed to create target");

        // Try to copy bytes 2-100 (end is way beyond EOF at position 4)
        // Should copy position 2='C', 3='D', 4='E', then hit EOF and stop gracefully
        let result = append_bytes_from_file_to_file(&source_path, 2, 100, &target_path);

        #[cfg(test)]
        assert!(result.is_ok(), "Should return Ok and stop at EOF");

        // Verify target contains only available bytes: "CDE"
        let content = read_file_content(&target_path).expect("Failed to read target");

        #[cfg(test)]
        assert_eq!(content, b"CDE", "Target should contain bytes until EOF");

        // Cleanup
        let _ = fs::remove_file(&source_path);
        let _ = fs::remove_file(&target_path);
    }

    /// Test: Copy single byte (start == end)
    #[test]
    fn test_copy_single_byte() {
        let temp_dir = std::env::temp_dir();
        let source_path = temp_dir.join("test_source_single.txt");
        let target_path = temp_dir.join("test_target_single.txt");

        // Create source file
        create_test_file(&source_path, b"ABCDEFGHIJ").expect("Failed to create source");

        // Create target file
        create_test_file(&target_path, b"").expect("Failed to create target");

        // Copy single byte at position 3: should copy "D" (0=A,1=B,2=C,3=D)
        let result = append_bytes_from_file_to_file(&source_path, 3, 3, &target_path);

        #[cfg(test)]
        assert!(result.is_ok(), "Function should succeed");

        // Verify target contains exactly single byte "D"
        let content = read_file_content(&target_path).expect("Failed to read target");

        #[cfg(test)]
        assert_eq!(content, b"D", "Target should contain single byte");

        // Cleanup
        let _ = fs::remove_file(&source_path);
        let _ = fs::remove_file(&target_path);
    }

    /// Test: Invalid range (start > end) - should return error
    #[test]
    fn test_invalid_range_start_greater_than_end() {
        let temp_dir = std::env::temp_dir();
        let source_path = temp_dir.join("test_source_invalid.txt");
        let target_path = temp_dir.join("test_target_invalid.txt");

        // Create source file
        create_test_file(&source_path, b"ABCDEFGHIJ").expect("Failed to create source");

        // Try to copy with start > end (invalid range)
        let result = append_bytes_from_file_to_file(&source_path, 10, 5, &target_path);

        #[cfg(test)]
        assert!(result.is_err(), "Should return error when start > end");

        // Cleanup
        let _ = fs::remove_file(&source_path);
        let _ = fs::remove_file(&target_path);
    }

    /// Test: Copy entire file from beginning to end
    #[test]
    fn test_copy_entire_file() {
        let temp_dir = std::env::temp_dir();
        let source_path = temp_dir.join("test_source_full.txt");
        let target_path = temp_dir.join("test_target_full.txt");

        let test_content = b"The quick brown fox jumps over the lazy dog";

        // Create source file
        create_test_file(&source_path, test_content).expect("Failed to create source");

        // Create target file
        create_test_file(&target_path, b"").expect("Failed to create target");

        // Copy entire file (position 0 to position length-1)
        let result = append_bytes_from_file_to_file(
            &source_path,
            0,
            (test_content.len() - 1) as u64,
            &target_path,
        );

        #[cfg(test)]
        assert!(result.is_ok(), "Function should succeed");

        // Verify target contains entire source content
        let content = read_file_content(&target_path).expect("Failed to read target");

        #[cfg(test)]
        assert_eq!(
            content, test_content,
            "Target should contain entire source file"
        );

        // Cleanup
        let _ = fs::remove_file(&source_path);
        let _ = fs::remove_file(&target_path);
    }

    /// Test: Multiple sequential appends to same target file
    #[test]
    fn test_multiple_appends() {
        let temp_dir = std::env::temp_dir();
        let source_path = temp_dir.join("test_source_multi.txt");
        let target_path = temp_dir.join("test_target_multi.txt");

        // Create source file
        create_test_file(&source_path, b"ABCDEFGHIJ").expect("Failed to create source");

        // Create target file with initial content
        create_test_file(&target_path, b"START_").expect("Failed to create target");

        // First append: bytes 0-2 ("ABC")
        let result1 = append_bytes_from_file_to_file(&source_path, 0, 2, &target_path);

        #[cfg(test)]
        assert!(result1.is_ok(), "First append should succeed");

        // Second append: bytes 5-7 ("FGH")
        let result2 = append_bytes_from_file_to_file(&source_path, 5, 7, &target_path);

        #[cfg(test)]
        assert!(result2.is_ok(), "Second append should succeed");

        // Third append: bytes 9-9 ("J")
        let result3 = append_bytes_from_file_to_file(&source_path, 9, 9, &target_path);

        #[cfg(test)]
        assert!(result3.is_ok(), "Third append should succeed");

        // Verify target contains: "START_" + "ABC" + "FGH" + "J" = "START_ABCFGHJ"
        let content = read_file_content(&target_path).expect("Failed to read target");

        #[cfg(test)]
        assert_eq!(
            content, b"START_ABCFGHJ",
            "Target should contain all appended bytes"
        );

        // Cleanup
        let _ = fs::remove_file(&source_path);
        let _ = fs::remove_file(&target_path);
    }

    /// Test: Copy first byte of file (position 0)
    #[test]
    fn test_copy_first_byte() {
        let temp_dir = std::env::temp_dir();
        let source_path = temp_dir.join("test_source_first.txt");
        let target_path = temp_dir.join("test_target_first.txt");

        // Create source file
        create_test_file(&source_path, b"ABCDEFGHIJ").expect("Failed to create source");

        // Create target file
        create_test_file(&target_path, b"").expect("Failed to create target");

        // Copy first byte (position 0): should copy "A"
        let result = append_bytes_from_file_to_file(&source_path, 0, 0, &target_path);

        #[cfg(test)]
        assert!(result.is_ok(), "Function should succeed");

        // Verify target contains exactly "A"
        let content = read_file_content(&target_path).expect("Failed to read target");

        #[cfg(test)]
        assert_eq!(content, b"A", "Target should contain first byte");

        // Cleanup
        let _ = fs::remove_file(&source_path);
        let _ = fs::remove_file(&target_path);
    }

    /// Test: Copy last byte of file
    #[test]
    fn test_copy_last_byte() {
        let temp_dir = std::env::temp_dir();
        let source_path = temp_dir.join("test_source_last.txt");
        let target_path = temp_dir.join("test_target_last.txt");

        // Create source file with 10 bytes
        create_test_file(&source_path, b"ABCDEFGHIJ").expect("Failed to create source");

        // Create target file
        create_test_file(&target_path, b"").expect("Failed to create target");

        // Copy last byte (position 9): should copy "J"
        let result = append_bytes_from_file_to_file(&source_path, 9, 9, &target_path);

        #[cfg(test)]
        assert!(result.is_ok(), "Function should succeed");

        // Verify target contains exactly "J"
        let content = read_file_content(&target_path).expect("Failed to read target");

        #[cfg(test)]
        assert_eq!(content, b"J", "Target should contain last byte");

        // Cleanup
        let _ = fs::remove_file(&source_path);
        let _ = fs::remove_file(&target_path);
    }

    /// Test: Empty source file - should return Ok gracefully
    #[test]
    fn test_empty_source_file() {
        let temp_dir = std::env::temp_dir();
        let source_path = temp_dir.join("test_source_empty.txt");
        let target_path = temp_dir.join("test_target_empty.txt");

        // Create empty source file (0 bytes)
        create_test_file(&source_path, b"").expect("Failed to create source");

        // Create target file
        create_test_file(&target_path, b"PREFIX_").expect("Failed to create target");

        // Try to copy bytes 0-10 from empty file
        // Should immediately hit EOF when trying to read first byte
        let result = append_bytes_from_file_to_file(&source_path, 0, 10, &target_path);

        #[cfg(test)]
        assert!(result.is_ok(), "Should return Ok when source is empty");

        // Verify target still contains only original content
        let content = read_file_content(&target_path).expect("Failed to read target");

        #[cfg(test)]
        assert_eq!(content, b"PREFIX_", "Target should remain unchanged");

        // Cleanup
        let _ = fs::remove_file(&source_path);
        let _ = fs::remove_file(&target_path);
    }
}

// =============================================================================
// TESTS
// =============================================================================

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Global counter for unique test identifiers
    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    /// Helper: Get project root test_files directory
    /// Creates it if it doesn't exist
    fn get_test_files_dir() -> io::Result<PathBuf> {
        // Get the project root (assuming tests run from project root)
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("test_files");
        path.push("clipboard_filename_tests");

        // Create directory if it doesn't exist
        fs::create_dir_all(&path)?;

        Ok(path)
    }

    /// Helper: Get unique test subdirectory to avoid race conditions
    /// Each test gets its own isolated directory
    fn get_unique_test_dir() -> io::Result<PathBuf> {
        let base_dir = get_test_files_dir()?;

        // Create unique subdirectory using counter and thread ID
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let thread_id = std::thread::current().id();
        let unique_name = format!("test_{}_{:?}", test_id, thread_id);

        let test_dir = base_dir.join(unique_name);
        fs::create_dir_all(&test_dir)?;

        Ok(test_dir)
    }

    /// Helper: Create test file with content in given directory
    /// Only creates if file doesn't already exist
    fn ensure_test_file(dir: &Path, name: &str, content: &[u8]) -> io::Result<PathBuf> {
        let file_path = dir.join(name);

        // Only create if doesn't exist
        if !file_path.exists() {
            let mut file = File::create(&file_path)?;
            file.write_all(content)?;
            file.flush()?;
        }

        Ok(file_path)
    }

    #[test]
    fn test_basic_alphanumeric_extraction() {
        let test_dir = match get_unique_test_dir() {
            Ok(dir) => dir,
            Err(e) => {
                eprintln!("Failed to create test directory: {}", e);
                return;
            }
        };

        let source_file = match ensure_test_file(&test_dir, "source_basic.txt", b"Hello World 123")
        {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Failed to create test file: {}", e);
                return;
            }
        };

        let result = generate_clipboard_filename(0, 15, &source_file, &test_dir);

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        let filename = result.unwrap();
        assert_eq!(filename, "HelloWorld123");
    }

    #[test]
    fn test_fallback_to_item() {
        let test_dir = match get_unique_test_dir() {
            Ok(dir) => dir,
            Err(e) => {
                eprintln!("Failed to create test directory: {}", e);
                return;
            }
        };

        let source_file = match ensure_test_file(&test_dir, "source_symbols.txt", b"!@#$%^&*()") {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Failed to create test file: {}", e);
                return;
            }
        };

        let result = generate_clipboard_filename(0, 10, &source_file, &test_dir);

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        let filename = result.unwrap();
        assert_eq!(filename, "item");
    }

    #[test]
    fn test_max_16_characters() {
        let test_dir = match get_unique_test_dir() {
            Ok(dir) => dir,
            Err(e) => {
                eprintln!("Failed to create test directory: {}", e);
                return;
            }
        };

        let source_file =
            match ensure_test_file(&test_dir, "source_long.txt", b"abcdefghijklmnopqrstuvwxyz") {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("Failed to create test file: {}", e);
                    return;
                }
            };

        let result = generate_clipboard_filename(0, 26, &source_file, &test_dir);

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        let filename = result.unwrap();
        assert_eq!(
            filename.len(),
            16,
            "Expected 16 chars, got: {}",
            filename.len()
        );
        assert_eq!(filename, "abcdefghijklmnop");
    }

    #[test]
    fn test_conflict_resolution() {
        let test_dir = match get_unique_test_dir() {
            Ok(dir) => dir,
            Err(e) => {
                eprintln!("Failed to create test directory: {}", e);
                return;
            }
        };

        let source_file = match ensure_test_file(&test_dir, "source_conflict.txt", b"testname") {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Failed to create test file: {}", e);
                return;
            }
        };

        // Create conflicting file "testname"
        let conflict_file = test_dir.join("testname");
        if let Err(e) = fs::write(&conflict_file, b"existing content") {
            eprintln!("Failed to create conflict file: {}", e);
            return;
        }

        let result = generate_clipboard_filename(0, 8, &source_file, &test_dir);

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        let filename = result.unwrap();
        assert_eq!(filename, "testname_2");
    }

    // #[test]
    // fn test_invalid_byte_range() {
    //     let test_dir = match get_unique_test_dir() {
    //         Ok(dir) => dir,
    //         Err(e) => {
    //             eprintln!("Failed to create test directory: {}", e);
    //             return;
    //         }
    //     };

    //     let source_file = match ensure_test_file(&test_dir, "source_invalid.txt", b"test content") {
    //         Ok(path) => path,
    //         Err(e) => {
    //             eprintln!("Failed to create test file: {}", e);
    //             return;
    //         }
    //     };

    //     // start_byte > end_byte should return error
    //     let result = generate_clipboard_filename(10, 5, &source_file, &test_dir);

    //     assert!(
    //         result.is_err(),
    //         "Expected Err for invalid range, got: {:?}",
    //         result
    //     );
    // }

    #[test]
    fn test_multiple_conflicts() {
        let test_dir = match get_unique_test_dir() {
            Ok(dir) => dir,
            Err(e) => {
                eprintln!("Failed to create test directory: {}", e);
                return;
            }
        };

        let source_file = match ensure_test_file(&test_dir, "source_multi.txt", b"duplicate") {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Failed to create test file: {}", e);
                return;
            }
        };

        // Create multiple conflicting files
        let _ = fs::write(test_dir.join("duplicate"), b"content1");
        let _ = fs::write(test_dir.join("duplicate_2"), b"content2");
        let _ = fs::write(test_dir.join("duplicate_3"), b"content3");

        let result = generate_clipboard_filename(0, 9, &source_file, &test_dir);

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        let filename = result.unwrap();
        assert_eq!(filename, "duplicate_4");
    }

    #[test]
    fn test_mixed_alphanumeric_and_symbols() {
        let test_dir = match get_unique_test_dir() {
            Ok(dir) => dir,
            Err(e) => {
                eprintln!("Failed to create test directory: {}", e);
                return;
            }
        };

        let source_file = match ensure_test_file(&test_dir, "source_mixed.txt", b"abc-123-xyz!@#") {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Failed to create test file: {}", e);
                return;
            }
        };

        let result = generate_clipboard_filename(0, 14, &source_file, &test_dir);

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        let filename = result.unwrap();
        assert_eq!(filename, "abc123xyz"); // Only alphanumeric extracted
    }

    #[test]
    fn test_partial_selection() {
        let test_dir = match get_unique_test_dir() {
            Ok(dir) => dir,
            Err(e) => {
                eprintln!("Failed to create test directory: {}", e);
                return;
            }
        };

        let source_file =
            match ensure_test_file(&test_dir, "source_partial.txt", b"0123456789abcdefghij") {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("Failed to create test file: {}", e);
                    return;
                }
            };

        // Select only middle portion: bytes 5-10 = "56789a"
        let result = generate_clipboard_filename(5, 11, &source_file, &test_dir);

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        let filename = result.unwrap();
        assert_eq!(filename, "56789ab");
    }
    #[test]
    fn test_single_byte_selection() {
        // Renamed - not empty anymore
        let test_dir = match get_unique_test_dir() {
            Ok(dir) => dir,
            Err(e) => {
                eprintln!("Failed to create test directory: {}", e);
                return;
            }
        };

        let source_file = match ensure_test_file(&test_dir, "source_single.txt", b"some content") {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Failed to create test file: {}", e);
                return;
            }
        };

        // Single-byte selection: start_byte == end_byte (position 5 = 'c')
        let result = generate_clipboard_filename(5, 5, &source_file, &test_dir);

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        let filename = result.unwrap();
        assert_eq!(filename, "c"); // Now extracts the single character
    }
    #[test]
    fn test_non_alphanumeric_fallback() {
        let test_dir = match get_unique_test_dir() {
            Ok(dir) => dir,
            Err(e) => {
                eprintln!("Failed to create test directory: {}", e);
                return;
            }
        };

        // File with only punctuation
        let source_file = match ensure_test_file(&test_dir, "source_punct.txt", b"!@#$%^&*()") {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Failed to create test file: {}", e);
                return;
            }
        };

        // Select bytes 0-9 (all punctuation, no alphanumeric)
        let result = generate_clipboard_filename(0, 9, &source_file, &test_dir);

        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        let filename = result.unwrap();
        assert_eq!(filename, "item"); // Should fall back to "item"
    }
}

#[cfg(test)]
mod hexedit_tests {
    use crate::buttons_reversible_edit_changelog_module::*;

    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;

    /// Helper: Creates a test file with known content
    fn create_test_file(filename: &str, content: &[u8]) -> PathBuf {
        let mut test_dir = std::env::current_dir().expect("Cannot get current dir");
        test_dir.push("test_files");

        fs::create_dir_all(&test_dir).expect("Cannot create test_files directory");

        let mut file_path = test_dir.clone();
        file_path.push(filename);

        let mut file = fs::File::create(&file_path).expect("Cannot create test file");
        file.write_all(content).expect("Cannot write test content");
        file.flush().expect("Cannot flush test file");

        file_path
    }

    /// Helper: Cleans up test file and associated directories
    fn cleanup_test_file(file_path: &Path) {
        let _ = fs::remove_file(file_path);

        if let Ok(changelog_dir) = get_undo_changelog_directory_path(file_path) {
            let _ = fs::remove_dir_all(&changelog_dir);
        }

        if let Some(parent) = file_path.parent() {
            if let Some(filename) = file_path.file_stem() {
                let mut redo_dir = parent.to_path_buf();
                redo_dir.push(format!("redo_{}", filename.to_string_lossy()));
                let _ = fs::remove_dir_all(&redo_dir);
            }
        }
    }

    /// Helper: Creates a minimal EditorState for testing hex edit
    ///
    /// # Arguments
    /// * `file_path` - Path to file being edited
    /// * `cursor_position` - Initial cursor byte offset
    ///
    /// # Returns
    /// EditorState with minimal required fields initialized
    /// Helper: Creates a minimal EditorState for testing hex edit
    fn create_test_editor_state(file_path: PathBuf, cursor_position: usize) -> EditorState {
        EditorState {
            the_last_command: None,                      // ???
            session_directory_path: None,                // ???
            mode: EditorMode::HexMode,                   // Correct?
            original_file_path: Some(file_path.clone()), // ???
            read_copy_path: Some(file_path),
            effective_rows: 40, // ??? What value?
            effective_cols: 77, // ??? What value?
            windowmap_positions: [[None; MAX_TUI_COLS]; MAX_TUI_ROWS],
            windowmap_line_byte_start_end_position_pairs: [None; MAX_TUI_ROWS],
            security_mode: false,

            cursor: WindowPosition { row: 0, col: 0 },
            next_move_right_is_past_newline: false,
            selection_start: None,
            selection_rowline_start: 0,
            is_modified: false,

            // Position tracking - all zeros OK for test?
            line_count_at_top_of_window: 0,
            file_position_of_topline_start: 0,
            file_position_of_vis_select_start: 0,
            file_position_of_vis_select_end: 0,
            tui_window_horizontal_utf8txt_line_char_offset: 0,
            in_row_abs_horizontal_0_index_cursor_position: 2,

            // Display buffers
            utf8_txt_display_buffers: [[0u8; 182]; 45],
            display_utf8txt_buffer_lengths: [0usize; 45],

            // Hex cursor - this is what we're testing
            hex_cursor: HexCursor {
                byte_offset_linear_file_absolute_position: cursor_position,
                // nibble_position: 0, // ??? Is this field correct?
                bytes_per_row: 80,
            },

            eof_fileline_tuirow_tuple: None,
            info_bar_message_buffer: [0u8; INFOBAR_MESSAGE_BUFFER_SIZE],
        }
    }

    /// Test 1: Happy path - hex edit creates undo log
    #[test]
    fn test_hex_edit_creates_undo_log() {
        let test_content = vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x42, 0x66, 0x77, 0x88, 0x99];
        let file_path = create_test_file("test_hex_edit_undo_1.bin", &test_content);

        let mut editor = create_test_editor_state(file_path.clone(), 5);

        // Perform hex edit: change position 5 from 0x42 to 0x99
        let result = editor.write_n_log_hex_edit_in_place(5, 0x99);

        assert!(
            result.is_ok(),
            "Hex edit should succeed: {:?}",
            result.err()
        );

        // Verify new byte written
        let new_byte =
            read_single_byte_from_file(&file_path, 5).expect("Should read byte after edit");
        assert_eq!(new_byte, 0x99, "Position 5 should contain 0x99 after edit");

        // Verify undo log directory exists
        let changelog_dir =
            get_undo_changelog_directory_path(&file_path).expect("Should get changelog directory");
        assert!(
            changelog_dir.exists(),
            "Changelog directory should exist: {:?}",
            changelog_dir
        );

        // Verify at least one undo log file exists
        let log_files: Vec<_> = fs::read_dir(&changelog_dir)
            .expect("Should read changelog directory")
            .filter_map(|entry| entry.ok())
            .collect();
        assert!(
            !log_files.is_empty(),
            "Should have at least one undo log file"
        );

        // Verify redo stack is empty
        if let Some(parent) = file_path.parent() {
            if let Some(filename) = file_path.file_stem() {
                let mut redo_dir = parent.to_path_buf();
                redo_dir.push(format!("redo_{}", filename.to_string_lossy()));

                if redo_dir.exists() {
                    let redo_files: Vec<_> = fs::read_dir(&redo_dir)
                        .expect("Should read redo directory")
                        .filter_map(|entry| entry.ok())
                        .collect();
                    assert!(
                        redo_files.is_empty(),
                        "Redo directory should be empty after user edit"
                    );
                }
            }
        }

        cleanup_test_file(&file_path);
    }

    /// Test 2: Boundary error - edit past EOF
    #[test]
    fn test_hex_edit_past_eof_fails() {
        let test_content = vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99];
        let file_path = create_test_file("test_hex_edit_undo_2.bin", &test_content);

        let mut editor = create_test_editor_state(file_path.clone(), 20);

        // Attempt to edit past EOF
        let result = editor.write_n_log_hex_edit_in_place(20, 0xAA);

        assert!(result.is_err(), "Editing past EOF should fail");

        // Verify file unchanged
        let byte_5 =
            read_single_byte_from_file(&file_path, 5).expect("Should read byte from original file");
        assert_eq!(byte_5, 0x55, "File should be unchanged after failed edit");

        cleanup_test_file(&file_path);
    }

    /// Test 3: Write permission error - readonly file
    #[test]
    fn test_hex_edit_readonly_file_fails() {
        let test_content = vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99];
        let file_path = create_test_file("test_hex_edit_undo_3.bin", &test_content);

        // Set file to read-only
        let mut perms = fs::metadata(&file_path)
            .expect("Should get file metadata")
            .permissions();
        perms.set_readonly(true);
        fs::set_permissions(&file_path, perms).expect("Should set read-only permissions");

        let mut editor = create_test_editor_state(file_path.clone(), 5);

        // Attempt to edit read-only file
        let result = editor.write_n_log_hex_edit_in_place(5, 0xBB);

        assert!(result.is_err(), "Editing read-only file should fail");

        // Verify file unchanged
        let byte_5 =
            read_single_byte_from_file(&file_path, 5).expect("Should read byte from original file");
        assert_eq!(byte_5, 0x55, "Read-only file should be unchanged");

        // Remove read-only flag before cleanup
        let mut perms = fs::metadata(&file_path)
            .expect("Should get file metadata for cleanup")
            .permissions();
        perms.set_readonly(false);
        let _ = fs::set_permissions(&file_path, perms);

        cleanup_test_file(&file_path);
    }
}

// ============================================================================
// SAVE-AS-COPY OPERATION: Test Suite (start)
// ============================================================================

#[cfg(test)]
mod saveas_tests {
    use super::*;
    use std::fs;
    use std::io::{ErrorKind, Write};
    use std::path::PathBuf;

    // ========================================================================
    // Test Helper Functions
    // ========================================================================

    /// Creates a temporary directory for test isolation
    ///
    /// # Purpose
    /// Provides isolated filesystem space for each test to prevent
    /// cross-test contamination. Uses OS temp directory with unique names.
    ///
    /// # Returns
    /// Absolute path to temporary directory (created, empty, writable)
    ///
    /// # Cleanup
    /// Caller responsible for cleanup (use `cleanup_test_dir()`)
    fn create_test_dir() -> PathBuf {
        let mut temp_dir = std::env::temp_dir();
        // Add unique identifier to prevent collision between parallel tests
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time error")
            .as_nanos();
        temp_dir.push(format!("lines_test_{}", unique_id));

        fs::create_dir_all(&temp_dir).expect("Failed to create test directory");
        temp_dir
    }

    /// Removes test directory and all contents
    ///
    /// # Purpose
    /// Cleans up test artifacts after test completes.
    /// Best-effort cleanup: ignores errors (test isolation is goal).
    ///
    /// # Arguments
    /// * `dir` - Directory to remove (should be test directory)
    fn cleanup_test_dir(dir: &PathBuf) {
        // Best-effort cleanup: ignore errors
        // Tests run in isolated temp directories, cleanup failures aren't critical
        let _ = fs::remove_dir_all(dir);
    }

    /// Creates a test file with specified content
    ///
    /// # Purpose
    /// Creates test fixture files with known content for verification.
    ///
    /// # Arguments
    /// * `path` - Where to create file (must be absolute)
    /// * `content` - Content to write to file
    ///
    /// # Returns
    /// * `Ok(())` - File created successfully
    /// * `Err(io::Error)` - Creation or write failed
    fn create_test_file(path: &PathBuf, content: &[u8]) -> io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(content)?;
        file.flush()?;
        Ok(())
    }

    /// Reads entire file content into Vec
    ///
    /// # Purpose
    /// Reads test files for content verification after copy.
    ///
    /// # Arguments
    /// * `path` - File to read
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - File content
    /// * `Err(io::Error)` - Read failed
    ///
    /// # Note
    /// Only for test files (small sizes). Production code uses chunked reading.
    fn read_test_file(path: &PathBuf) -> io::Result<Vec<u8>> {
        fs::read(path)
    }

    // ========================================================================
    // Success Case Tests
    // ========================================================================

    #[test]
    fn test_copy_simple_text_file() {
        // Test: Successful copy of simple text file
        // Validates: Basic copy operation, content preservation, status code

        let test_dir = create_test_dir();
        let source_path = test_dir.join("source.txt");
        let dest_path = test_dir.join("destination.txt");

        // Create source file with test content
        let test_content = b"Hello, Lines Editor!\nThis is a test file.\n";
        create_test_file(&source_path, test_content).expect("Failed to create test source file");

        // Execute copy operation
        let result = save_file_as_newfile_with_newname(&source_path, &dest_path);

        // Verify: Operation succeeded with Copied status
        assert!(result.is_ok(), "Copy operation should succeed");
        let (status, message) = result.unwrap();
        assert_eq!(status, FileOperationStatus::Copied);
        assert_eq!(message, "copied");

        // Verify: Destination file exists
        assert!(dest_path.exists(), "Destination file should exist");

        // Verify: Content matches exactly
        let dest_content = read_test_file(&dest_path).expect("Failed to read destination file");
        assert_eq!(
            dest_content, test_content,
            "Destination content should match source"
        );

        // Verify: Source file unchanged
        let source_content = read_test_file(&source_path).expect("Failed to read source file");
        assert_eq!(
            source_content, test_content,
            "Source file should be unchanged"
        );

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_copy_empty_file() {
        // Test: Copy empty file (0 bytes)
        // Edge case: Validates EOF detection on first read

        let test_dir = create_test_dir();
        let source_path = test_dir.join("empty.txt");
        let dest_path = test_dir.join("empty_copy.txt");

        // Create empty source file
        create_test_file(&source_path, b"").expect("Failed to create empty source file");

        // Execute copy operation
        let result = save_file_as_newfile_with_newname(&source_path, &dest_path);

        // Verify: Operation succeeded
        assert!(result.is_ok(), "Empty file copy should succeed");
        let (status, _) = result.unwrap();
        assert_eq!(status, FileOperationStatus::Copied);

        // Verify: Destination exists and is empty
        assert!(dest_path.exists(), "Destination should exist");
        let dest_content = read_test_file(&dest_path).expect("Failed to read destination");
        assert_eq!(dest_content.len(), 0, "Destination should be empty");

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_copy_binary_file() {
        // Test: Copy file with binary (non-UTF8) content
        // Validates: Binary safety, no text assumptions

        let test_dir = create_test_dir();
        let source_path = test_dir.join("binary.dat");
        let dest_path = test_dir.join("binary_copy.dat");

        // Create binary content (not valid UTF-8)
        let binary_content: Vec<u8> = (0..=255).collect();
        create_test_file(&source_path, &binary_content)
            .expect("Failed to create binary source file");

        // Execute copy operation
        let result = save_file_as_newfile_with_newname(&source_path, &dest_path);

        // Verify: Operation succeeded
        assert!(result.is_ok(), "Binary file copy should succeed");
        let (status, _) = result.unwrap();
        assert_eq!(status, FileOperationStatus::Copied);

        // Verify: Binary content preserved exactly
        let dest_content = read_test_file(&dest_path).expect("Failed to read destination");
        assert_eq!(
            dest_content, binary_content,
            "Binary content should match exactly"
        );

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_copy_multi_chunk_file() {
        // Test: Copy file larger than buffer size (requires multiple chunks)
        // Validates: Chunked reading/writing, loop logic, EOF detection

        let test_dir = create_test_dir();
        let source_path = test_dir.join("large.txt");
        let dest_path = test_dir.join("large_copy.txt");

        // Create content larger than buffer (8KB buffer, make 20KB file)
        let chunk_size = 1024; // 1KB
        let num_chunks = 20; // 20KB total
        let mut large_content = Vec::with_capacity(chunk_size * num_chunks);
        for i in 0..num_chunks {
            // Create varied content to detect corruption
            let line = format!("Line {} in chunk {}\n", i, i / 10);
            large_content.extend_from_slice(line.as_bytes());
            // Pad to 1KB
            while large_content.len() < (i + 1) * chunk_size {
                large_content.push(b'x');
            }
        }

        create_test_file(&source_path, &large_content).expect("Failed to create large source file");

        // Execute copy operation
        let result = save_file_as_newfile_with_newname(&source_path, &dest_path);

        // Verify: Operation succeeded
        assert!(result.is_ok(), "Large file copy should succeed");
        let (status, _) = result.unwrap();
        assert_eq!(status, FileOperationStatus::Copied);

        // Verify: All content copied correctly
        let dest_content = read_test_file(&dest_path).expect("Failed to read destination");
        assert_eq!(
            dest_content.len(),
            large_content.len(),
            "Destination size should match source"
        );
        assert_eq!(
            dest_content, large_content,
            "Large file content should match exactly"
        );

        cleanup_test_dir(&test_dir);
    }

    // ========================================================================
    // Predicated Outcome Tests (Expected Non-Error Cases)
    // ========================================================================

    #[test]
    fn test_source_not_found() {
        // Test: Source file doesn't exist
        // Predicated outcome: Should return OriginalNotFound status, not error

        let test_dir = create_test_dir();
        let source_path = test_dir.join("nonexistent.txt");
        let dest_path = test_dir.join("destination.txt");

        // Do NOT create source file - it doesn't exist

        // Execute copy operation
        let result = save_file_as_newfile_with_newname(&source_path, &dest_path);

        // Verify: Returns Ok with OriginalNotFound status (not an error)
        assert!(result.is_ok(), "Should return Ok for predicated outcome");
        let (status, message) = result.unwrap();
        assert_eq!(status, FileOperationStatus::OriginalNotFound);
        assert_eq!(message, "original not found");

        // Verify: Destination was not created
        assert!(
            !dest_path.exists(),
            "Destination should not be created when source missing"
        );

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_destination_already_exists() {
        // Test: Destination file already exists (no-overwrite policy)
        // Predicated outcome: Should return AlreadyExisted status, not error

        let test_dir = create_test_dir();
        let source_path = test_dir.join("source.txt");
        let dest_path = test_dir.join("destination.txt");

        // Create both source and destination files
        let source_content = b"Source content";
        let dest_content = b"Existing destination content - should not be overwritten";
        create_test_file(&source_path, source_content).expect("Failed to create source");
        create_test_file(&dest_path, dest_content).expect("Failed to create destination");

        // Execute copy operation
        let result = save_file_as_newfile_with_newname(&source_path, &dest_path);

        // Verify: Returns Ok with AlreadyExisted status (not an error)
        assert!(result.is_ok(), "Should return Ok for predicated outcome");
        let (status, message) = result.unwrap();
        assert_eq!(status, FileOperationStatus::AlreadyExisted);
        assert_eq!(message, "already existed");

        // Verify: Destination content unchanged (no overwrite)
        let final_dest_content = read_test_file(&dest_path).expect("Failed to read destination");
        assert_eq!(
            final_dest_content, dest_content,
            "Destination should be unchanged (no overwrite)"
        );

        // Verify: Source content unchanged
        let final_source_content = read_test_file(&source_path).expect("Failed to read source");
        assert_eq!(
            final_source_content, source_content,
            "Source should be unchanged"
        );

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_source_is_directory() {
        // Test: Source path points to directory, not file
        // Edge case: Should return error (not a valid file)

        let test_dir = create_test_dir();
        let source_path = test_dir.join("source_dir");
        let dest_path = test_dir.join("destination.txt");

        // Create directory at source path
        fs::create_dir(&source_path).expect("Failed to create source directory");

        // Execute copy operation
        let result = save_file_as_newfile_with_newname(&source_path, &dest_path);

        // Verify: Returns error (not a file)
        assert!(result.is_err(), "Should return error for directory source");
        match result {
            Err(LinesError::InvalidInput(_)) => { /* Expected */ }
            _ => panic!("Expected InvalidInput error for directory source"),
        }

        cleanup_test_dir(&test_dir);
    }

    // ========================================================================
    // Path Validation Tests
    // ========================================================================

    // ========================================================================
    // Helper Function Tests
    // ========================================================================

    #[test]
    fn test_is_retryable_error_interrupted() {
        // Test: ErrorKind::Interrupted is retryable
        let error = io::Error::new(ErrorKind::Interrupted, "test");
        assert!(
            is_retryable_error(&error),
            "Interrupted should be retryable"
        );
    }

    #[test]
    fn test_is_retryable_error_would_block() {
        // Test: ErrorKind::WouldBlock is retryable
        let error = io::Error::new(ErrorKind::WouldBlock, "test");
        assert!(is_retryable_error(&error), "WouldBlock should be retryable");
    }

    #[test]
    fn test_is_retryable_error_timed_out() {
        // Test: ErrorKind::TimedOut is retryable
        let error = io::Error::new(ErrorKind::TimedOut, "test");
        assert!(is_retryable_error(&error), "TimedOut should be retryable");
    }

    #[test]
    fn test_is_retryable_error_not_found() {
        // Test: ErrorKind::NotFound is NOT retryable
        let error = io::Error::new(ErrorKind::NotFound, "test");
        assert!(
            !is_retryable_error(&error),
            "NotFound should not be retryable"
        );
    }

    #[test]
    fn test_is_retryable_error_permission_denied() {
        // Test: ErrorKind::PermissionDenied is NOT retryable
        let error = io::Error::new(ErrorKind::PermissionDenied, "test");
        assert!(
            !is_retryable_error(&error),
            "PermissionDenied should not be retryable"
        );
    }

    #[test]
    fn test_is_retryable_error_already_exists() {
        // Test: ErrorKind::AlreadyExists is NOT retryable
        let error = io::Error::new(ErrorKind::AlreadyExists, "test");
        assert!(
            !is_retryable_error(&error),
            "AlreadyExists should not be retryable"
        );
    }

    #[test]
    fn test_retry_operation_success_first_try() {
        // Test: Operation succeeds on first attempt
        let mut attempts = 0;
        let result = retry_operation(
            || {
                attempts += 1;
                Ok::<i32, io::Error>(42)
            },
            3,
        );

        assert!(result.is_ok(), "Should succeed on first try");
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts, 1, "Should only attempt once");
    }

    #[test]
    fn test_retry_operation_success_after_retry() {
        // Test: Operation fails once with retryable error, then succeeds
        let mut attempts = 0;
        let result = retry_operation(
            || {
                attempts += 1;
                if attempts == 1 {
                    Err(io::Error::new(ErrorKind::Interrupted, "retry me"))
                } else {
                    Ok::<i32, io::Error>(42)
                }
            },
            3,
        );

        assert!(result.is_ok(), "Should succeed after retry");
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts, 2, "Should attempt twice");
    }

    #[test]
    fn test_retry_operation_permanent_error() {
        // Test: Operation fails with permanent error (should not retry)
        let mut attempts = 0;
        let result = retry_operation(
            || {
                attempts += 1;
                Err::<i32, io::Error>(io::Error::new(ErrorKind::NotFound, "permanent"))
            },
            3,
        );

        assert!(result.is_err(), "Should fail with permanent error");
        assert_eq!(attempts, 1, "Should not retry permanent errors");
    }

    #[test]
    fn test_retry_operation_exhausted_retries() {
        // Test: Operation fails with retryable error all 3 times
        let mut attempts = 0;
        let result = retry_operation(
            || {
                attempts += 1;
                Err::<i32, io::Error>(io::Error::new(ErrorKind::Interrupted, "always fails"))
            },
            3,
        );

        assert!(result.is_err(), "Should fail after exhausting retries");
        assert_eq!(attempts, 3, "Should attempt max times");
    }

    #[test]
    #[should_panic(expected = "max_attempts must be greater than 0")]
    fn test_retry_operation_zero_attempts_debug() {
        // Test: Zero max_attempts should trigger debug_assert in debug builds
        // Note: This only panics in debug builds
        let result = retry_operation(|| Ok::<i32, io::Error>(42), 0);
        // In release builds, this returns error instead of panicking
        let _ = result;
    }

    // ========================================================================
    // FileOperationStatus Tests
    // ========================================================================

    #[test]
    fn test_file_operation_status_display() {
        // Test: Display trait implementation for all status codes
        assert_eq!(format!("{}", FileOperationStatus::Copied), "copied");
        assert_eq!(
            format!("{}", FileOperationStatus::AlreadyExisted),
            "already existed"
        );
        assert_eq!(
            format!("{}", FileOperationStatus::OriginalNotFound),
            "original not found"
        );
        assert_eq!(
            format!("{}", FileOperationStatus::OriginalUnavailable),
            "original unavailable"
        );
        assert_eq!(
            format!("{}", FileOperationStatus::DestinationUnavailable),
            "destination unavailable"
        );
    }

    #[test]
    fn test_file_operation_status_equality() {
        // Test: Equality comparison for status codes
        assert_eq!(FileOperationStatus::Copied, FileOperationStatus::Copied);
        assert_ne!(
            FileOperationStatus::Copied,
            FileOperationStatus::AlreadyExisted
        );
    }

    #[test]
    fn test_file_operation_status_clone() {
        // Test: Clone trait (should be cheap copy)
        let status = FileOperationStatus::Copied;
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    #[test]
    fn test_file_operation_status_debug() {
        // Test: Debug trait implementation
        let status = FileOperationStatus::Copied;
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("Copied"));
    }

    // ========================================================================
    // Integration Tests (Full Workflow)
    // ========================================================================

    #[test]
    fn test_multiple_copies_same_source() {
        // Test: Copy same source to multiple destinations
        // Validates: Source can be read multiple times

        let test_dir = create_test_dir();
        let source_path = test_dir.join("source.txt");
        let dest1_path = test_dir.join("dest1.txt");
        let dest2_path = test_dir.join("dest2.txt");

        let content = b"Shared source content";
        create_test_file(&source_path, content).expect("Failed to create source");

        // Copy to first destination
        let result1 = save_file_as_newfile_with_newname(&source_path, &dest1_path);
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap().0, FileOperationStatus::Copied);

        // Copy to second destination (same source)
        let result2 = save_file_as_newfile_with_newname(&source_path, &dest2_path);
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap().0, FileOperationStatus::Copied);

        // Verify both destinations have correct content
        let dest1_content = read_test_file(&dest1_path).expect("Read dest1");
        let dest2_content = read_test_file(&dest2_path).expect("Read dest2");
        assert_eq!(dest1_content, content);
        assert_eq!(dest2_content, content);

        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_copy_then_copy_again() {
        // Test: Attempt to copy again to same destination (should get AlreadyExisted)

        let test_dir = create_test_dir();
        let source_path = test_dir.join("source.txt");
        let dest_path = test_dir.join("destination.txt");

        create_test_file(&source_path, b"content").expect("Failed to create source");

        // First copy: should succeed
        let result1 = save_file_as_newfile_with_newname(&source_path, &dest_path);
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap().0, FileOperationStatus::Copied);

        // Second copy to same destination: should get AlreadyExisted
        let result2 = save_file_as_newfile_with_newname(&source_path, &dest_path);
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap().0, FileOperationStatus::AlreadyExisted);

        cleanup_test_dir(&test_dir);
    }
}

// ============================================================================
// (end) SAVE-AS-COPY OPERATION: Test Suite
// ============================================================================

#[cfg(test)]
mod byte_utf8_newline_tests {
    // use super::*;

    #[test]
    fn test_utf8_length_ascii() {
        // Test ASCII character (1 byte)
        // Setup: Create temp file with "a"
        // Assert: length = 1
    }

    #[test]
    fn test_utf8_length_two_byte() {
        // Test 2-byte character (e.g., ñ = C3 B1)
        // Assert: length = 2
    }

    #[test]
    fn test_utf8_length_three_byte() {
        // Test 3-byte character (e.g., 世 = E4 B8 96)
        // Assert: length = 3
    }

    #[test]
    fn test_utf8_length_four_byte() {
        // Test 4-byte character (e.g., 𝄞 = F0 9D 84 9E)
        // Assert: length = 4
    }

    #[test]
    fn test_utf8_invalid_continuation_byte() {
        // Test invalid UTF-8 (0x80 continuation byte as first byte)
        // Assert: length = 1 (defensive fallback)
    }

    #[test]
    fn test_is_next_byte_newline_multibyte_at_end() {
        // File: "ab世\n" where 世 is at line end
        // Cursor on 世
        // Assert: is_next_byte_newline() = true
    }

    #[test]
    fn test_is_next_byte_newline_multibyte_not_at_end() {
        // File: "世ab\n" where 世 is at line start
        // Cursor on 世
        // Assert: is_next_byte_newline() = false
    }
}
