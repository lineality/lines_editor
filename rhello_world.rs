/// // cargoComment
fn main() {
    /// // rhu rhou rhaggy...
    println!("Hello, world!");
}

/*
// Comment alpha
// Comment b
// Comment 13
*/

// fn main2() {
//     println!("Hello, world!");
// }


if !is_plain_text {
    let highlight = buffy_get_syntax_highlight(byte_pos, row_content);

    match highlight {
        SyntaxHighlight::SyntaxSymbol => {
            // Single character in cyan. Write ANSI, char, reset.
            stdout.write_all(BLUE).map_err(|e| {
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
            stdout.write_all(RESET).map_err(|e| {
                LinesError::DisplayError(stack_format_it(
                    "rURWC syn write: {}",
                    &[&e.to_string()],
                    "rURWC syn write",
                ))
            })?;
