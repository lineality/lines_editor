//! buffy.rs - Zero-heap formatting for TUI applications
//! https://github.com/lineality/buffy_stack_format_write_module
//!
//! ## HEAP ALLOCATION: NONE
//!
//! This module performs string formatting and terminal output with ZERO heap
//! allocation. All operations use stack-allocated buffers. No String, no Vec,
//! no .to_string(), no dynamic memory allocation.
//!
//! ## Design Philosophy
//! - User provides output buffers (for string building)
//! - Direct write to terminal (for output functions)
//! - All conversions happen on stack
//! - Functions return &str borrowing from provided buffers
//!
//! ## Memory Model
//! - Number conversions: Stack buffers (max 20 bytes per number)
//! - ANSI styling: Stack buffers (max 64 bytes)
//! - Template processing: Read-only, no allocation
//! - Terminal output: Direct write, no intermediate storage
//! - String building: User-provided buffer, zero allocation
//!
//! ## Limitations (By Design)
//! - Max 8 format arguments per call (prevents stack overflow)
//! - User must provide adequate output buffer (we validate and return None if too small)
//! - Max 64 characters width for alignment (prevents runaway padding)

use std::io::{self, Write};
use std::path::Path;

/*
```
print!() and println!() - are macros that use format!() internally,

write!() and writeln!() - These are also macros that also use format!() internally(?)
e.g. if you write!(f, "{}", value)

write!() macro does format the string, but writes directly to the destination without creating an intermediate String...but still uses format!() internally(?)

using write!(stdout, "{}", formatted) because that still uses format!

use stdout.write_all() writes bytes directly; no format!, no heap-dynamic memory
```
*/

/*
Sample Wrappers

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

*/

// =============================================================================
// TYPES
// =============================================================================

/// ANSI styling for terminal text
///
/// ## Project Context
/// Represents visual formatting for TUI elements (menus, errors, highlights).
/// All fields are compile-time constants (&'static str) - no allocation.
///
/// Memory: should be all stack, no heap
/// All style codes are static string slices pointing to program binary.
#[derive(Debug, Clone, Copy, Default)]
pub struct BuffyStyles {
    pub fg_color: Option<&'static str>,
    pub bg_color: Option<&'static str>,
    pub bold: bool,
    pub underline: bool,
    pub italic: bool,
    pub dim: bool,
}

/// Format arguments that can be converted to strings without heap allocation
///
/// ## Project Context
/// Represents values to insert into format templates. Each variant handles
/// a specific type with stack-based conversion.
///
/// Memory: should be all stack, no heap
/// All variants store values directly (no pointers to heap).
/// String variants are references to existing data (no new allocation).
///
/// ## Supported Types
/// - Str: Existing string slices
/// - U8, U16, U32, U64, Usize: Unsigned integers (stack-converted)
/// - I8, I16, I32, I64, Isize: Signed integers (stack-converted)
/// - U8Hex, U16Hex, U32Hex: Hex formatting (stack-converted)
/// - Bool: true/false
/// - Char: Single character
/// - Path: File paths (borrowed reference)
/// - Styled variants: Include ANSI styling
#[derive(Debug, Clone)]
pub enum BuffyFormatArg<'a> {
    // // Unsigned integers
    Str(&'a str),
    // U8(u8),
    // U16(u16),
    // U32(u32),
    // U64(u64),
    Usize(usize),

    // Signed integers
    // I8(i8),
    // I16(i16),
    // I32(i32),
    // I64(i64),
    // Isize(isize),

    // // Hex formatting
    // U8Hex(u8),
    // U16Hex(u16),
    // U32Hex(u32),

    // // Other types
    // Bool(bool),
    // Char(char),
    Path(&'a Path),

    // Styled variants (adds ANSI codes)
    CharStyled(char, BuffyStyles),
    StrStyled(&'a str, BuffyStyles),
    // U8Styled(u8, BuffyStyles),
    // U64Styled(u64, BuffyStyles),
    // UsizeStyled(usize, BuffyStyles),
    // U8HexStyled(u8, BuffyStyles),
}

// =============================================================================
// INTERNAL HELPERS - Stack-based conversions
// =============================================================================

/// Converts u64 to decimal string in provided stack buffer
///
/// Memory: should be all stack, no heap
/// Writes digits directly into user's buffer, returns slice of that buffer.
///
/// ## Parameters
/// - value: Number to convert
/// - buf: Stack buffer to write into (min 20 bytes for u64::MAX)
///
/// ## Returns
/// - Some(&str): Formatted number borrowing from buf
/// - None: Buffer too small
fn format_u64_to_buffer<'a>(value: u64, buf: &'a mut [u8]) -> Option<&'a str> {
    if buf.is_empty() {
        return None;
    }

    if value == 0 {
        buf[0] = b'0';
        return std::str::from_utf8(&buf[..1]).ok();
    }

    let mut num = value;
    let mut temp = [0u8; 20]; // Stack buffer for digit reversal
    let mut pos = 0;

    while num > 0 {
        temp[pos] = b'0' + (num % 10) as u8;
        num /= 10;
        pos += 1;
    }

    if pos > buf.len() {
        return None;
    }

    // Reverse digits into output buffer
    for i in 0..pos {
        buf[i] = temp[pos - 1 - i];
    }

    std::str::from_utf8(&buf[..pos]).ok()
}

// /// Converts i64 to decimal string with sign in provided stack buffer
// ///
// /// Memory: should be all stack, no heap
// fn format_i64_to_buffer<'a>(value: i64, buf: &'a mut [u8]) -> Option<&'a str> {
//     if buf.is_empty() {
//         return None;
//     }

//     if value == 0 {
//         buf[0] = b'0';
//         return std::str::from_utf8(&buf[..1]).ok();
//     }

//     let (is_negative, abs_value) = if value < 0 {
//         (true, value.wrapping_abs() as u64)
//     } else {
//         (false, value as u64)
//     };

//     let mut temp = [0u8; 20];
//     let mut pos = 0;
//     let mut num = abs_value;

//     while num > 0 {
//         temp[pos] = b'0' + (num % 10) as u8;
//         num /= 10;
//         pos += 1;
//     }

//     let total_len = if is_negative { pos + 1 } else { pos };

//     if total_len > buf.len() {
//         return None;
//     }

//     let mut buf_pos = 0;

//     if is_negative {
//         buf[buf_pos] = b'-';
//         buf_pos += 1;
//     }

//     for i in 0..pos {
//         buf[buf_pos + i] = temp[pos - 1 - i];
//     }

//     std::str::from_utf8(&buf[..total_len]).ok()
// }

// /// Converts u8 to 2-digit uppercase hex in provided stack buffer
// ///
// /// Memory: should be all stack, no heap
// fn format_u8_hex_to_buffer<'a>(value: u8, buf: &'a mut [u8]) -> Option<&'a str> {
//     if buf.len() < 2 {
//         return None;
//     }

//     let hex_chars = b"0123456789ABCDEF";
//     buf[0] = hex_chars[(value >> 4) as usize];
//     buf[1] = hex_chars[(value & 0x0F) as usize];

//     std::str::from_utf8(&buf[..2]).ok()
// }

// /// Converts u16 to 4-digit uppercase hex in provided stack buffer
// ///
// /// Memory: should be all stack, no heap
// fn format_u16_hex_to_buffer<'a>(value: u16, buf: &'a mut [u8]) -> Option<&'a str> {
//     if buf.len() < 4 {
//         return None;
//     }

//     let hex_chars = b"0123456789ABCDEF";
//     buf[0] = hex_chars[((value >> 12) & 0x0F) as usize];
//     buf[1] = hex_chars[((value >> 8) & 0x0F) as usize];
//     buf[2] = hex_chars[((value >> 4) & 0x0F) as usize];
//     buf[3] = hex_chars[(value & 0x0F) as usize];

//     std::str::from_utf8(&buf[..4]).ok()
// }

// /// Converts u32 to 8-digit uppercase hex in provided stack buffer
// ///
// /// Memory: should be all stack, no heap
// fn format_u32_hex_to_buffer<'a>(value: u32, buf: &'a mut [u8]) -> Option<&'a str> {
//     if buf.len() < 8 {
//         return None;
//     }

//     let hex_chars = b"0123456789ABCDEF";
//     for i in 0..8 {
//         let shift = 28 - (i * 4);
//         buf[i] = hex_chars[((value >> shift) & 0x0F) as usize];
//     }

//     std::str::from_utf8(&buf[..8]).ok()
// }

/// Converts BuffyStyles to ANSI escape sequences in provided stack buffer
///
/// Memory: should be all stack, no heap
/// Concatenates ANSI codes directly into buffer.
pub fn style_to_ansi<'a>(style: BuffyStyles, buf: &'a mut [u8]) -> Option<&'a str> {
    let mut pos = 0;

    if style.bold {
        let code = b"\x1b[1m";
        if pos + code.len() > buf.len() {
            return None;
        }
        buf[pos..pos + code.len()].copy_from_slice(code);
        pos += code.len();
    }

    if style.underline {
        let code = b"\x1b[4m";
        if pos + code.len() > buf.len() {
            return None;
        }
        buf[pos..pos + code.len()].copy_from_slice(code);
        pos += code.len();
    }

    if style.italic {
        let code = b"\x1b[3m";
        if pos + code.len() > buf.len() {
            return None;
        }
        buf[pos..pos + code.len()].copy_from_slice(code);
        pos += code.len();
    }

    if style.dim {
        let code = b"\x1b[2m";
        if pos + code.len() > buf.len() {
            return None;
        }
        buf[pos..pos + code.len()].copy_from_slice(code);
        pos += code.len();
    }

    if let Some(fg) = style.fg_color {
        let code = fg.as_bytes();
        if pos + code.len() > buf.len() {
            return None;
        }
        buf[pos..pos + code.len()].copy_from_slice(code);
        pos += code.len();
    }

    if let Some(bg) = style.bg_color {
        let code = bg.as_bytes();
        if pos + code.len() > buf.len() {
            return None;
        }
        buf[pos..pos + code.len()].copy_from_slice(code);
        pos += code.len();
    }

    std::str::from_utf8(&buf[..pos]).ok()
}

// =============================================================================
// ALIGNMENT SUPPORT
// =============================================================================

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

/// Parse format specifier from placeholder text
/// Examples: "" -> no alignment, "<5" -> left 5, ">10" -> right 10
fn parse_format_spec(placeholder: &str) -> Option<FormatSpec> {
    if placeholder.is_empty() {
        return Some(FormatSpec {
            alignment: Alignment::Left,
            width: None,
        });
    }

    if !placeholder.starts_with(':') {
        return None;
    }

    let spec = &placeholder[1..];

    if spec.is_empty() {
        return Some(FormatSpec {
            alignment: Alignment::Left,
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
        (Alignment::Right, spec)
    } else {
        return None;
    };

    let width = if width_str.is_empty() {
        None
    } else {
        match width_str.parse::<usize>() {
            Ok(w) if w <= 64 => Some(w),
            _ => return None,
        }
    };

    Some(FormatSpec { alignment, width })
}

/// Apply alignment to a value, writing result to buffer
/// Returns number of bytes written, or None if buffer too small
fn apply_alignment<'a>(value: &str, spec: FormatSpec, buf: &'a mut [u8]) -> Option<&'a str> {
    let width = match spec.width {
        Some(w) => w,
        None => {
            // No width specified, just copy value
            let value_bytes = value.as_bytes();
            if value_bytes.len() > buf.len() {
                return None;
            }
            buf[..value_bytes.len()].copy_from_slice(value_bytes);
            return std::str::from_utf8(&buf[..value_bytes.len()]).ok();
        }
    };

    let value_len = value.len();

    if value_len >= width {
        // Value already meets or exceeds width
        if value_len > buf.len() {
            return None;
        }
        buf[..value_len].copy_from_slice(value.as_bytes());
        return std::str::from_utf8(&buf[..value_len]).ok();
    }

    if width > buf.len() {
        return None;
    }

    let padding = width - value_len;

    match spec.alignment {
        Alignment::Left => {
            // Value then spaces
            buf[..value_len].copy_from_slice(value.as_bytes());
            for i in value_len..width {
                buf[i] = b' ';
            }
        }
        Alignment::Right => {
            // Spaces then value
            for i in 0..padding {
                buf[i] = b' ';
            }
            buf[padding..width].copy_from_slice(value.as_bytes());
        }
        Alignment::Center => {
            // Spaces, value, spaces
            let left_pad = padding / 2;
            // Right pad not needed - calculated as (width - left_pad - value_len)
            for i in 0..left_pad {
                buf[i] = b' ';
            }
            buf[left_pad..left_pad + value_len].copy_from_slice(value.as_bytes());
            for i in (left_pad + value_len)..width {
                buf[i] = b' ';
            }
        }
    }

    std::str::from_utf8(&buf[..width]).ok()
}

// =============================================================================
// DIRECT TERMINAL OUTPUT - TRUE ZERO HEAP
// =============================================================================

/// Writes formatted output directly to stdout without any intermediate allocation.
///
/// ## Project Context
/// Primary output function for TUI. Processes format template and writes
/// results directly to terminal as it goes. No String building, no Vec,
/// no intermediate storage.
///
/// Memory: should be all stack, no heap
/// All conversions use stack buffers. Output written directly to stdout.
///
/// ## Operation
/// 1. Parse template piece by piece
/// 2. For literal text: write directly
/// 3. For placeholders: convert arg on stack, write result
/// 4. Continue until template exhausted
///
/// ## Safety & Error Handling
/// - No panic: Returns io::Error on write failure
/// - Bounded: Max 8 arguments (prevents stack overflow)
/// - Validates: All conversions checked, returns error on failure
/// - Non-critical: Caller can continue on error
///
/// ## Parameters
/// - template: Format string with {} or {:<N}/{:>N}/{:^N} placeholders
/// - args: Slice of BuffyFormatArg values (max 8)
///
/// ## Returns
/// - Ok(()): Successfully written to stdout
/// - Err(io::Error): Write failed or format error
///
/// ## Examples
/// ```rust
/// // Simple text
/// buffy_print("Hello world", &[])?;
///
/// // With number
/// buffy_print("Count: {}", &[BuffyFormatArg::U64(42)])?;
///
/// // With styling
/// buffy_print("Status: {}", &[BuffyFormatArg::StrStyled("OK", BuffyStyles::bold_red())])?;
///
/// // With alignment
/// buffy_print("{:<10} {:>5}", &[BuffyFormatArg::Str("Name"), BuffyFormatArg::U32(123)])?;
/// ```
pub fn buffy_print(template: &str, args: &[BuffyFormatArg]) -> io::Result<()> {
    const MAX_ARGS: usize = 8;

    if args.len() > MAX_ARGS {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Too many arguments (max 8)",
        ));
    }

    let mut stdout = io::stdout();
    let mut arg_index = 0;
    let mut pos = 0;

    // Stack buffers for conversions
    let mut num_buf = [0u8; 20];
    let mut style_buf = [0u8; 64];
    let mut align_buf = [0u8; 128];

    while pos < template.len() {
        // Find next placeholder
        if let Some(brace_pos) = template[pos..].find('{') {
            let absolute_brace = pos + brace_pos;

            // Write literal text before placeholder
            if brace_pos > 0 {
                stdout.write_all(template[pos..absolute_brace].as_bytes())?;
            }

            // Find closing brace
            if let Some(close_pos) = template[absolute_brace..].find('}') {
                let absolute_close = absolute_brace + close_pos;
                let placeholder = &template[absolute_brace + 1..absolute_close];

                // Parse format spec
                let spec = match parse_format_spec(placeholder) {
                    Some(s) => s,
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "Invalid format specifier",
                        ));
                    }
                };

                // Get argument
                if arg_index >= args.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Not enough arguments for format string",
                    ));
                }

                // Convert argument to string (on stack)
                let (value_str, has_style, style) = match &args[arg_index] {
                    BuffyFormatArg::Str(s) => (*s, false, BuffyStyles::default()),
                    // BuffyFormatArg::U8(n) => {
                    //     let s = format_u64_to_buffer(*n as u64, &mut num_buf).ok_or_else(|| {
                    //         io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                    //     })?;
                    //     (s, false, BuffyStyles::default())
                    // }
                    // BuffyFormatArg::U16(n) => {
                    //     let s = format_u64_to_buffer(*n as u64, &mut num_buf).ok_or_else(|| {
                    //         io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                    //     })?;
                    //     (s, false, BuffyStyles::default())
                    // }
                    // BuffyFormatArg::U32(n) => {
                    //     let s = format_u64_to_buffer(*n as u64, &mut num_buf).ok_or_else(|| {
                    //         io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                    //     })?;
                    //     (s, false, BuffyStyles::default())
                    // }
                    // BuffyFormatArg::U64(n) => {
                    //     let s = format_u64_to_buffer(*n, &mut num_buf).ok_or_else(|| {
                    //         io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                    //     })?;
                    //     (s, false, BuffyStyles::default())
                    // }
                    BuffyFormatArg::Usize(n) => {
                        let s = format_u64_to_buffer(*n as u64, &mut num_buf).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                        })?;
                        (s, false, BuffyStyles::default())
                    }
                    // BuffyFormatArg::I8(n) => {
                    //     let s = format_i64_to_buffer(*n as i64, &mut num_buf).ok_or_else(|| {
                    //         io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                    //     })?;
                    //     (s, false, BuffyStyles::default())
                    // }
                    // BuffyFormatArg::I16(n) => {
                    //     let s = format_i64_to_buffer(*n as i64, &mut num_buf).ok_or_else(|| {
                    //         io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                    //     })?;
                    //     (s, false, BuffyStyles::default())
                    // }
                    // BuffyFormatArg::I32(n) => {
                    //     let s = format_i64_to_buffer(*n as i64, &mut num_buf).ok_or_else(|| {
                    //         io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                    //     })?;
                    //     (s, false, BuffyStyles::default())
                    // }
                    // BuffyFormatArg::I64(n) => {
                    //     let s = format_i64_to_buffer(*n, &mut num_buf).ok_or_else(|| {
                    //         io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                    //     })?;
                    //     (s, false, BuffyStyles::default())
                    // }
                    // BuffyFormatArg::Isize(n) => {
                    //     let s = format_i64_to_buffer(*n as i64, &mut num_buf).ok_or_else(|| {
                    //         io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                    //     })?;
                    //     (s, false, BuffyStyles::default())
                    // }
                    // BuffyFormatArg::U8Hex(n) => {
                    //     let s = format_u8_hex_to_buffer(*n, &mut num_buf).ok_or_else(|| {
                    //         io::Error::new(io::ErrorKind::Other, "Hex conversion failed")
                    //     })?;
                    //     (s, false, BuffyStyles::default())
                    // }
                    // BuffyFormatArg::U16Hex(n) => {
                    //     let s = format_u16_hex_to_buffer(*n, &mut num_buf).ok_or_else(|| {
                    //         io::Error::new(io::ErrorKind::Other, "Hex conversion failed")
                    //     })?;
                    //     (s, false, BuffyStyles::default())
                    // }
                    // BuffyFormatArg::U32Hex(n) => {
                    //     let s = format_u32_hex_to_buffer(*n, &mut num_buf).ok_or_else(|| {
                    //         io::Error::new(io::ErrorKind::Other, "Hex conversion failed")
                    //     })?;
                    //     (s, false, BuffyStyles::default())
                    // }
                    // BuffyFormatArg::Bool(b) => (
                    //     if *b { "true" } else { "false" },
                    //     false,
                    //     BuffyStyles::default(),
                    // ),
                    // BuffyFormatArg::Char(c) => {
                    //     let mut char_buf = [0u8; 4];
                    //     let char_str = c.encode_utf8(&mut char_buf);
                    //     let len = char_str.len();
                    //     num_buf[..len].copy_from_slice(char_str.as_bytes());
                    //     let s = std::str::from_utf8(&num_buf[..len]).map_err(|_| {
                    //         io::Error::new(io::ErrorKind::Other, "Char conversion failed")
                    //     })?;
                    //     (s, false, BuffyStyles::default())
                    // }
                    BuffyFormatArg::Path(p) => {
                        let s = p.to_str().ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Path conversion failed")
                        })?;
                        (s, false, BuffyStyles::default())
                    }

                    // Styled variants
                    BuffyFormatArg::CharStyled(c, st) => {
                        let mut char_buf = [0u8; 4];
                        let char_str = c.encode_utf8(&mut char_buf);
                        let len = char_str.len();
                        num_buf[..len].copy_from_slice(char_str.as_bytes());
                        let s = std::str::from_utf8(&num_buf[..len]).map_err(|_| {
                            io::Error::new(io::ErrorKind::Other, "Char conversion failed")
                        })?;
                        (s, true, *st)
                    }
                    BuffyFormatArg::StrStyled(s, st) => (*s, true, *st),
                    // BuffyFormatArg::U8Styled(n, st) => {
                    //     let s = format_u64_to_buffer(*n as u64, &mut num_buf).ok_or_else(|| {
                    //         io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                    //     })?;
                    //     (s, true, *st)
                    // }
                    // BuffyFormatArg::U64Styled(n, st) => {
                    //     let s = format_u64_to_buffer(*n, &mut num_buf).ok_or_else(|| {
                    //         io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                    //     })?;
                    //     (s, true, *st)
                    // }
                    // BuffyFormatArg::UsizeStyled(n, st) => {
                    //     let s = format_u64_to_buffer(*n as u64, &mut num_buf).ok_or_else(|| {
                    //         io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                    //     })?;
                    //     (s, true, *st)
                    // }
                    // BuffyFormatArg::U8HexStyled(n, st) => {
                    //     let s = format_u8_hex_to_buffer(*n, &mut num_buf).ok_or_else(|| {
                    //         io::Error::new(io::ErrorKind::Other, "Hex conversion failed")
                    //     })?;
                    //     (s, true, *st)
                    // }
                };

                // Apply style if needed
                if has_style {
                    let ansi = style_to_ansi(style, &mut style_buf).ok_or_else(|| {
                        io::Error::new(io::ErrorKind::Other, "BuffyStyles conversion failed")
                    })?;
                    stdout.write_all(ansi.as_bytes())?;
                }

                // Apply alignment and write
                let aligned = apply_alignment(value_str, spec, &mut align_buf)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Alignment failed"))?;
                stdout.write_all(aligned.as_bytes())?;

                // Reset style if needed
                if has_style {
                    stdout.write_all(b"\x1b[0m")?;
                }

                arg_index += 1;
                pos = absolute_close + 1;
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unclosed brace in format string",
                ));
            }
        } else {
            // No more placeholders, write remaining literal text
            stdout.write_all(template[pos..].as_bytes())?;
            break;
        }
    }

    Ok(())
}

/// Writes formatted output to stdout with newline and flush.
///
/// Memory: should be all stack, no heap
/// Calls buffy_print() then writes newline and flushes.
pub fn buffy_println(template: &str, args: &[BuffyFormatArg]) -> io::Result<()> {
    buffy_print(template, args)?;
    let mut stdout = io::stdout();
    stdout.write_all(b"\n")?;
    stdout.flush()
}

// /// Writes formatted output to any writer.
// ///
// /// Memory: should be all stack, no heap
// /// Same direct-write logic as buffy_print() but writes to provided writer.
// ///
// /// Writes formatted output to any writer (file, buffer, stream, stderr, etc.) with zero heap allocation.
// ///
// /// ## Project Context
// /// Generic output function for writing formatted text to destinations other than stdout.
// /// Used for file logging, buffer building, network streams, or stderr output while
// /// maintaining zero heap allocation guarantee. This is the underlying mechanism that
// /// `buffy_print()` uses internally with stdout.
// ///
// /// ## Memory: ZERO HEAP
// /// All conversions use stack buffers. Output written directly to provided writer.
// /// No String objects, no Vec allocations, no intermediate storage.
// ///
// /// ## Operation Flow
// /// 1. Parse template string for {} placeholders
// /// 2. For literal text: write directly to writer
// /// 3. For placeholders: convert arg on stack, write result to writer
// /// 4. Apply alignment/styling as specified
// /// 5. Continue until template exhausted
// ///
// /// ## Safety & Error Handling
// /// - No panic: Returns io::Error on write or format failure
// /// - Bounded: Max 8 arguments (prevents stack overflow)
// /// - Validates: All conversions checked, returns Err on failure
// /// - Non-critical: Caller can handle error and continue
// /// - Production-safe: No debug info leakage in error messages
// ///
// /// ## Parameters
// /// - `writer`: Mutable reference to any type implementing `Write` trait
// ///   (File, Vec<u8>, Stderr, TcpStream, BufWriter, etc.)
// /// - `template`: Format string with {} or {:<N}/{:>N}/{:^N} placeholders
// ///   - `{}` - default formatting
// ///   - `{:<N}` - left-align in N characters
// ///   - `{:>N}` - right-align in N characters
// ///   - `{:^N}` - center-align in N characters
// /// - `args`: Slice of BuffyFormatArg values (max 8 per call)
// ///   - U8, U16, U32, U64, Usize - unsigned integers
// ///   - I8, I16, I32, I64, Isize - signed integers
// ///   - U8Hex, U16Hex, U32Hex - hexadecimal formatting
// ///   - Str - string slices
// ///   - Bool - true/false
// ///   - Char - single characters
// ///   - Path - file paths
// ///   - Styled variants - include ANSI color codes
// ///
// /// ## Returns
// /// - `Ok(())`: Successfully written to writer
// /// - `Err(io::Error)`: Write failed, format error, or buffer too small
// ///
// /// ## When to Use vs `buffy_print()`
// /// - Use `buffy_print()`: Writing to terminal/stdout (most common TUI case)
// /// - Use `buffy_write_basic()`: Writing to files, buffers, stderr, or network streams
// ///
// /// ## Limitations
// /// - Max 8 arguments per call (call multiple times if needed)
// /// - Max 64 characters width for alignment
// /// - Template placeholders must match arg count exactly
// /// - Writer must have capacity for output (or return error)
// ///
// /// ## Examples
// ///
// /// ### File Logging
// /// ```rust
// /// use std::fs::File;
// ///
// /// let mut log = File::create("app.log")?;
// /// buffy_write_basic(
// ///     &mut log,
// ///     "[{}] User {} logged in at {}\n",
// ///     &[
// ///         BuffyFormatArg::Str("INFO"),
// ///         BuffyFormatArg::U32(1001),
// ///         BuffyFormatArg::Str("2025-01-15"),
// ///     ]
// /// )?;
// /// log.flush()?;
// /// ```
// ///
// /// ### Error to Stderr
// /// ```rust
// /// use std::io::stderr;
// ///
// /// let mut err = stderr();
// /// buffy_write_basic(
// ///     &mut err,
// ///     "ERROR: Failed to open file (code: {})\n",
// ///     &[BuffyFormatArg::U32(404)]
// /// )?;
// /// ```
// ///
// /// ### Building String in Buffer
// /// ```rust
// /// let mut buffer = Vec::<u8>::new();
// /// buffy_write_basic(
// ///     &mut buffer,
// ///     "Report: {} items processed, {} errors\n",
// ///     &[
// ///         BuffyFormatArg::U64(1000),
// ///         BuffyFormatArg::U32(3),
// ///     ]
// /// )?;
// /// let report = String::from_utf8(buffer)?;
// /// ```
// ///
// /// ### Hex Dump to File
// /// ```rust
// /// let mut dump = File::create("memory.hex")?;
// /// let bytes: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
// ///
// /// buffy_write_basic(
// ///     &mut dump,
// ///     "0x{} 0x{} 0x{} 0x{}\n",
// ///     &[
// ///         BuffyFormatArg::U8Hex(bytes[0]),
// ///         BuffyFormatArg::U8Hex(bytes[1]),
// ///         BuffyFormatArg::U8Hex(bytes[2]),
// ///         BuffyFormatArg::U8Hex(bytes[3]),
// ///     ]
// /// )?;
// /// ```
// ///
// /// ### Styled Output to Stderr
// /// ```rust
// /// let mut err = stderr();
// /// buffy_write_basic(
// ///     &mut err,
// ///     "{}: Operation failed\n",
// ///     &[BuffyFormatArg::StrStyled(
// ///         "CRITICAL",
// ///         BuffyStyles {
// ///             fg_color: Some("\x1b[31m"), // RED
// ///             bold: true,
// ///             ..Default::default()
// ///         }
// ///     )]
// /// )?;
// /// ```
// ///
// /// ### Aligned Table to File
// /// ```rust
// /// let mut table = File::create("report.txt")?;
// ///
// /// // Header
// /// buffy_write_basic(
// ///     &mut table,
// ///     "{:<15} {:>10} {:>10}\n",
// ///     &[
// ///         BuffyFormatArg::Str("Item"),
// ///         BuffyFormatArg::Str("Quantity"),
// ///         BuffyFormatArg::Str("Price"),
// ///     ]
// /// )?;
// ///
// /// // Data row
// /// buffy_write_basic(
// ///     &mut table,
// ///     "{:<15} {:>10} {:>10}\n",
// ///     &[
// ///         BuffyFormatArg::Str("Widget"),
// ///         BuffyFormatArg::U32(42),
// ///         BuffyFormatArg::U32(299),
// ///     ]
// /// )?;
// /// ```
// ///
// /// ### Network Protocol Message
// /// ```rust
// /// use std::net::TcpStream;
// ///
// /// let mut stream = TcpStream::connect("127.0.0.1:8080")?;
// /// buffy_write_basic(
// ///     &mut stream,
// ///     "MSG {} LEN {} DATA {}\r\n",
// ///     &[
// ///         BuffyFormatArg::U32(1001),
// ///         BuffyFormatArg::U32(payload.len()),
// ///         BuffyFormatArg::Str(payload),
// ///     ]
// /// )?;
// /// stream.flush()?;
// /// ```
// ///
// /// ## Error Handling Pattern
// /// ```rust
// /// match buffy_write_basic(&mut file, "Value: {}\n", &[BuffyFormatArg::U32(x)]) {
// ///     Ok(()) => { /* continue */ },
// ///     Err(e) => {
// ///         // Log to stderr, don't panic production code
// ///         let mut err = stderr();
// ///         let _ = buffy_write_basic(
// ///             &mut err,
// ///             "Write failed (recovered)\n",
// ///             &[]
// ///         );
// ///         // Continue with fallback behavior
// ///     }
// /// }
// /// ```
// pub fn buffy_write_basic<W: Write>(
//     writer: &mut W,
//     template: &str,
//     args: &[BuffyFormatArg],
// ) -> io::Result<()> {
//     const MAX_ARGS: usize = 8;

//     if args.len() > MAX_ARGS {
//         return Err(io::Error::new(
//             io::ErrorKind::InvalidInput,
//             "Too many arguments (max 8)",
//         ));
//     }

//     let mut arg_index = 0;
//     let mut pos = 0;

//     // Stack buffers for conversions
//     let mut num_buf = [0u8; 20];
//     let mut style_buf = [0u8; 64];
//     let mut align_buf = [0u8; 128];

//     while pos < template.len() {
//         if let Some(brace_pos) = template[pos..].find('{') {
//             let absolute_brace = pos + brace_pos;

//             if brace_pos > 0 {
//                 writer.write_all(template[pos..absolute_brace].as_bytes())?;
//             }

//             if let Some(close_pos) = template[absolute_brace..].find('}') {
//                 let absolute_close = absolute_brace + close_pos;
//                 let placeholder = &template[absolute_brace + 1..absolute_close];

//                 let spec = match parse_format_spec(placeholder) {
//                     Some(s) => s,
//                     None => {
//                         return Err(io::Error::new(
//                             io::ErrorKind::InvalidInput,
//                             "Invalid format specifier",
//                         ));
//                     }
//                 };

//                 if arg_index >= args.len() {
//                     return Err(io::Error::new(
//                         io::ErrorKind::InvalidInput,
//                         "Not enough arguments for format string",
//                     ));
//                 }

//                 // Convert argument (same logic as buffy_print)
//                 let (value_str, has_style, style) = match &args[arg_index] {
//                     BuffyFormatArg::Str(s) => (*s, false, BuffyStyles::default()),
//                     // BuffyFormatArg::U8(n) => {
//                     //     let s = format_u64_to_buffer(*n as u64, &mut num_buf).ok_or_else(|| {
//                     //         io::Error::new(io::ErrorKind::Other, "Number conversion failed")
//                     //     })?;
//                     //     (s, false, BuffyStyles::default())
//                     // }
//                     BuffyFormatArg::U64(n) => {
//                         let s = format_u64_to_buffer(*n, &mut num_buf).ok_or_else(|| {
//                             io::Error::new(io::ErrorKind::Other, "Number conversion failed")
//                         })?;
//                         (s, false, BuffyStyles::default())
//                     }
//                     BuffyFormatArg::U8Hex(n) => {
//                         let s = format_u8_hex_to_buffer(*n, &mut num_buf).ok_or_else(|| {
//                             io::Error::new(io::ErrorKind::Other, "Hex conversion failed")
//                         })?;
//                         (s, false, BuffyStyles::default())
//                     }
//                     BuffyFormatArg::Bool(b) => (
//                         if *b { "true" } else { "false" },
//                         false,
//                         BuffyStyles::default(),
//                     ),
//                     BuffyFormatArg::Char(c) => {
//                         let mut char_buf = [0u8; 4];
//                         let char_str = c.encode_utf8(&mut char_buf);
//                         let len = char_str.len();
//                         num_buf[..len].copy_from_slice(char_str.as_bytes());
//                         let s = std::str::from_utf8(&num_buf[..len]).map_err(|_| {
//                             io::Error::new(io::ErrorKind::Other, "Char conversion failed")
//                         })?;
//                         (s, false, BuffyStyles::default())
//                     }

//                     // Add other types as needed (same as buffy_print)
//                     BuffyFormatArg::StrStyled(s, st) => (*s, true, *st),
//                     BuffyFormatArg::U8HexStyled(n, st) => {
//                         let s = format_u8_hex_to_buffer(*n, &mut num_buf).ok_or_else(|| {
//                             io::Error::new(io::ErrorKind::Other, "Hex conversion failed")
//                         })?;
//                         (s, true, *st)
//                     }

//                     // Add remaining types as needed
//                     _ => {
//                         return Err(io::Error::new(
//                             io::ErrorKind::Other,
//                             "Unsupported argument type",
//                         ));
//                     }
//                 };

//                 if has_style {
//                     let ansi = style_to_ansi(style, &mut style_buf).ok_or_else(|| {
//                         io::Error::new(io::ErrorKind::Other, "BuffyStyles conversion failed")
//                     })?;
//                     writer.write_all(ansi.as_bytes())?;
//                 }

//                 let aligned = apply_alignment(value_str, spec, &mut align_buf)
//                     .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Alignment failed"))?;
//                 writer.write_all(aligned.as_bytes())?;

//                 if has_style {
//                     writer.write_all(b"\x1b[0m")?;
//                 }

//                 arg_index += 1;
//                 pos = absolute_close + 1;
//             } else {
//                 return Err(io::Error::new(
//                     io::ErrorKind::InvalidInput,
//                     "Unclosed brace in format string",
//                 ));
//             }
//         } else {
//             writer.write_all(template[pos..].as_bytes())?;
//             break;
//         }
//     }

//     Ok(())
// }

// /// Repeats a character N times, writing directly to writer.
// ///
// /// ## Project Context
// /// Helper for drawing horizontal lines, borders, and padding in TUI.
// /// Avoids String::repeat() which allocates heap.
// ///
// /// Memory: should be all stack, no heap
// /// Uses 64-byte stack buffer, writes in chunks if repeat count exceeds buffer.
// ///
// /// ## Parameters
// /// - writer: Output destination
// /// - ch: Character to repeat (any UTF-8 character)
// /// - count: Number of repetitions
// ///
// /// ## Returns
// /// - Ok(()): Successfully written
// /// - Err(io::Error): Write failed
// ///
// /// ## Examples
// /// ```rust
// /// // Horizontal line
// /// buffy_repeat(&mut stdout, '=', 70)?;
// /// buffy_println("", &[])?;
// ///
// /// // Padding
// /// buffy_repeat(&mut stdout, ' ', 4)?;
// /// buffy_print("Indented text", &[])?;
// /// ```
// pub fn buffy_repeat<W: Write>(writer: &mut W, ch: char, count: usize) -> io::Result<()> {
//     if count == 0 {
//         return Ok(());
//     }

//     // Encode character to UTF-8 on stack
//     let mut char_buf = [0u8; 4];
//     let char_str = ch.encode_utf8(&mut char_buf);
//     let char_len = char_str.len();

//     // Use 64-byte stack buffer for batching
//     let mut buf = [0u8; 64];
//     let chars_per_batch = buf.len() / char_len;

//     if chars_per_batch == 0 {
//         return Err(io::Error::new(
//             io::ErrorKind::InvalidInput,
//             "Character too large for buffer",
//         ));
//     }

//     // Fill buffer with character pattern
//     let mut buf_pos = 0;
//     for _ in 0..chars_per_batch {
//         if buf_pos + char_len <= buf.len() {
//             buf[buf_pos..buf_pos + char_len].copy_from_slice(&char_buf[..char_len]);
//             buf_pos += char_len;
//         }
//     }
//     let batch_size = buf_pos;

//     // Write full batches
//     let full_batches = count / chars_per_batch;
//     for _ in 0..full_batches {
//         writer.write_all(&buf[..batch_size])?;
//     }

//     // Write remaining characters
//     let remaining = count % chars_per_batch;
//     for _ in 0..remaining {
//         writer.write_all(&char_buf[..char_len])?;
//     }

//     Ok(())
// }

// /// Writes a single styled text chunk directly to writer.
// ///
// /// Memory: should be all stack, no heap
// /// Writes ANSI codes (if any), text, and reset directly.
// pub fn buffy_write_styled<W: Write>(
//     writer: &mut W,
//     text: &str,
//     style: Option<BuffyStyles>,
// ) -> io::Result<()> {
//     if let Some(s) = style {
//         let mut style_buf = [0u8; 64];
//         if let Some(ansi) = style_to_ansi(s, &mut style_buf) {
//             writer.write_all(ansi.as_bytes())?;
//         }
//     }

//     writer.write_all(text.as_bytes())?;

//     if style.is_some() {
//         writer.write_all(b"\x1b[0m")?;
//     }

//     Ok(())
// }

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod buffy_format_tests {
    use super::*;

    #[test]
    fn test_format_u64() {
        let mut buf = [0u8; 20];
        let result = format_u64_to_buffer(42, &mut buf);
        assert_eq!(result, Some("42"));
    }

    // #[test]
    // fn test_format_i64_negative() {
    //     let mut buf = [0u8; 20];
    //     let result = format_i64_to_buffer(-42, &mut buf);
    //     assert_eq!(result, Some("-42"));
    // }

    // #[test]
    // fn test_format_hex() {
    //     let mut buf = [0u8; 8];z
    //     let result = format_u8_hex_to_buffer(0xFF, &mut buf);
    //     assert_eq!(result, Some("FF"));
    // }

    #[test]
    fn test_alignment_left() {
        let mut buf = [0u8; 10];
        let spec = FormatSpec {
            alignment: Alignment::Left,
            width: Some(5),
        };
        let result = apply_alignment("AB", spec, &mut buf);
        assert_eq!(result, Some("AB   "));
    }

    #[test]
    fn test_alignment_right() {
        let mut buf = [0u8; 10];
        let spec = FormatSpec {
            alignment: Alignment::Right,
            width: Some(5),
        };
        let result = apply_alignment("AB", spec, &mut buf);
        assert_eq!(result, Some("   AB"));
    }
}

// =============================================================================
// SYNTAX HIGHLIGHTING SUPPORT
// =============================================================================
//
// ## Project Context
// This module section provides minimal two-category syntax highlighting for
// a TUI text editor. The design is intentionally minimal:
//
//   Category 1 - Syntax symbols (cyan):  ( ) [ ] { } < > = : ; " ' \ & ! # / * , `
//   Category 2 - Definition words (yellow): fn , def , let , struct , enum ,
//                class , impl , type , const , static , pub , use , mod
//
// The system highlights by DEFAULT and opts OUT for plain-text extensions.
// This is simpler than trying to enumerate all code file extensions.
//
// Plain text extensions that opt OUT of highlighting: .txt  .log
//
// ## Design Constraints (No Heap)
// - No String, no Vec, no dynamic allocation
// - All matching done against &'static str constants
// - Extension comparison done on raw bytes
// - SyntaxHighlight is a stack-only enum
//
// ## Integration Point
// These functions are called from render_utf8txt_row_with_cursor() in the
// editor rendering pipeline. buffy_is_plain_text_extension() is called once
// per row before the character loop. buffy_get_syntax_highlight() is called
// once per character position inside the loop, only when not plain text,
// and only when the character is not already handled by cursor or selection
// highlighting (those take priority).
//
// ## What This Does NOT Do
// - No string/comment context detection: keywords inside string literals
//   will be highlighted (acceptable for minimal system)
// - No language detection beyond plain-text opt-out
// - No background highlighting: foreground colour only (word colouring)
// - No per-token parser: pure positional byte matching

/// Two-category syntax highlight result.
///
/// ## Project Context
/// Returned by buffy_get_syntax_highlight() to indicate what foreground
/// colour (if any) should be applied to the character at a given position.
///
/// ## Variants
/// - None:           No highlighting. Write character directly.
/// - SyntaxSymbol:   Single punctuation/operator character. Render in cyan.
/// - DefinitionWord: Start of a definition keyword (e.g. "fn ", "let ").
///                   The entire keyword including its trailing space is
///                   consumed as one logical token. Render in yellow.
///
/// ## Memory
/// Stack-only enum. No heap. Safe to copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxHighlight {
    /// No highlighting applies. Write the character with no ANSI codes.
    None,

    /// Character is a syntax punctuation symbol.
    /// Caller should render it in cyan.
    /// Applies to: ( ) [ ] { } < > = : ; " ' \ & ! # / * , `
    SyntaxSymbol,

    /// Character starts a definition keyword (including trailing space).
    /// Caller should render the entire keyword span in yellow.
    /// The `keyword_byte_len` field tells the caller how many bytes
    /// the full keyword token occupies (e.g. "fn " = 3 bytes).
    /// Caller advances its byte position by this amount after writing.
    DefinitionWord {
        /// Number of bytes in the matched keyword including trailing space.
        /// Example: "fn " -> 3, "struct " -> 7, "static " -> 7
        keyword_byte_len: usize,
    },
}

/// Returns true if the file at the given path has a plain-text extension
/// that should NOT receive syntax highlighting.
///
/// ## Project Context
/// Called once per rendered row, before the character loop, to decide
/// whether to attempt syntax highlighting at all. The opt-out list is
/// intentionally short: only file types that are truly plain prose where
/// keyword colouring would be distracting or misleading.
///
/// Opt-out extensions (no highlighting):
///   .txt   .log
///
/// Everything else (including no extension, or None path) receives
/// highlighting. This default-highlight approach is simpler than trying
/// to enumerate every possible code file extension.
///
/// ## Behaviour on Edge Cases
/// - path is None:          returns false  (highlight by default)
/// - path has no extension: returns false  (highlight by default)
/// - extension not valid UTF-8: returns false (highlight by default, safe)
/// - extension is ".TXT" (uppercase): returns false (case-sensitive match,
///   no heap conversion, conservative: only exact lowercase match opts out)
///
/// ## Memory
/// No heap. Extension slice borrowed from path. Comparison on &[u8] bytes.
///
/// ## Arguments
/// * `path` - Optional reference to the file path from EditorState.
///            Typically &state.original_file_path (which is Option<PathBuf>).
///            Pass as path.as_deref() to get Option<&Path>.
///
/// ## Returns
/// * true  - plain text, skip syntax highlighting
/// * false - not plain text (or unknown), apply syntax highlighting
pub fn buffy_is_plain_text_extension(path: Option<&Path>) -> bool {
    // Plain-text extensions that opt OUT of syntax highlighting.
    // Matched as raw bytes against the file extension.
    // Case-sensitive: ".txt" matches, ".TXT" does not.
    // Extend this list conservatively: only add extensions where keyword
    // colouring would be actively misleading or distracting.
    const PLAIN_TEXT_EXTENSIONS: &[&[u8]] = &[b"txt", b"log"];

    // Defensive: no path means we cannot determine extension.
    // Default to highlight (return false = not plain text).
    let path = match path {
        Some(p) => p,
        None => return false,
    };

    // Extract extension as a str slice (no allocation).
    // Path::extension() returns Option<&OsStr>.
    // OsStr::as_encoded_bytes() gives us raw bytes without allocation.
    let ext_bytes = match path.extension() {
        Some(ext) => ext.as_encoded_bytes(),
        None => return false, // No extension: highlight by default
    };

    // Compare extension bytes against the opt-out list.
    // Linear scan over a tiny fixed list: no overhead worth optimising.
    for &plain_ext in PLAIN_TEXT_EXTENSIONS {
        if ext_bytes == plain_ext {
            return true; // Plain text: skip highlighting
        }
    }

    false // Not in opt-out list: apply highlighting
}

/// Returns the syntax highlight category for the character at the given
/// byte position within a row string.
///
/// ## Project Context
/// Called inside the character rendering loop of render_utf8txt_row_with_cursor().
/// The caller has already handled cursor and visual-selection priority.
/// This function is only reached for "normal" characters that need
/// syntax highlight checking.
///
/// ## Two-Category System
///
/// ### Category 1: SyntaxSymbol (cyan)
/// Single-character punctuation/operator symbols.
/// Matched character-by-character. The caller writes one character and
/// advances by one UTF-8 character (1-4 bytes).
///
/// Symbol set:  ( ) [ ] { } < > = : ; " ' \ & ! # / * , `
///
/// ### Category 2: DefinitionWord (yellow)
/// Multi-character keywords, each followed by a space (space is part of
/// the match and is highlighted together with the keyword). The space is
/// included because:
/// - It makes the match unambiguous (avoids matching "type" inside "typeof")
/// - The space is visually invisible so including it in the coloured span
///   costs nothing visually
/// - No need to abstractly separate the space from the keyword
///
/// Keyword set (each stored with trailing space as part of the literal):
///   "fn "  "def "  "let "  "struct "  "enum "  "class "
///   "impl "  "type "  "const "  "static "  "pub "  "use "  "mod "
///
/// When a keyword matches, SyntaxHighlight::DefinitionWord { keyword_byte_len }
/// is returned. The caller must:
///   1. Write the entire keyword span (keyword_byte_len bytes) in yellow
///   2. Advance its byte position by keyword_byte_len
///   3. Advance its character counter by the number of chars in the keyword
///      (caller can count these, or use the provided char count - see note)
///
/// ## Priority
/// SyntaxSymbol is checked first. In practice there is no overlap
/// (no keyword begins with a syntax symbol character), but checking
/// symbols first is cheaper (single char lookup vs prefix scan).
///
/// ## Cursor and Selection Priority
/// This function does NOT check cursor or selection state. The caller
/// is responsible for checking those conditions BEFORE calling this
/// function. If the character is under the cursor or inside a visual
/// selection, the caller should NOT call this function.
///
/// ## Byte Position vs Character Index
/// `byte_pos` is the byte offset of the current character's first byte
/// within `row_content`. This is what is needed for prefix matching
/// (slicing `row_content` from `byte_pos` forward).
///
/// The caller tracks the character index (for cursor column comparison)
/// as a SEPARATE counter that increments once per complete UTF-8 character.
/// That character index is NOT passed to this function and NOT used here.
/// See render_utf8txt_row_with_cursor() for the byte-vs-char tracking pattern.
///
/// ## Memory
/// No heap. All keyword literals are &'static str. Matching is byte
/// comparison via str::starts_with() on a sub-slice.
///
/// ## Arguments
/// * `byte_pos`    - Byte offset of the current character's first byte
///                   within `row_content`. Must be a valid UTF-8 boundary.
/// * `row_content` - The full row string being rendered (content portion only,
///                   line number prefix already excluded by caller).
///
/// ## Returns
/// * SyntaxHighlight::None            - no highlighting, write char normally
/// * SyntaxHighlight::SyntaxSymbol    - render this char in cyan
/// * SyntaxHighlight::DefinitionWord { keyword_byte_len }
///                                    - render keyword_byte_len bytes in yellow,
///                                      then advance byte_pos by keyword_byte_len
pub fn buffy_get_syntax_highlight(byte_pos: usize, row_content: &str) -> SyntaxHighlight {
    // -------------------------------------------------------------------------
    // BOUNDS CHECK: byte_pos must be within row_content
    // -------------------------------------------------------------------------
    // Defensive: if byte_pos is out of range, return None safely.
    // This should not happen if the caller iterates correctly, but hardware
    // faults or logic errors must not cause a panic in production.
    if byte_pos >= row_content.len() {
        return SyntaxHighlight::None;
    }

    // -------------------------------------------------------------------------
    // DEFINITION KEYWORDS (Category 2, yellow)
    // Checked BEFORE symbol check for one reason: keyword matching requires
    // reading ahead multiple bytes anyway, and we want to colour the whole
    // keyword span (not just its first character) as a single token.
    //
    // Each entry is the complete match token: keyword + trailing space.
    // The trailing space is part of the highlighted span.
    // "fn " = 3 bytes, "struct " = 7 bytes, "static " = 7 bytes, etc.
    //
    // Order: longer keywords first where a shorter keyword is a prefix of
    // a longer one (none exist in this set currently, but "mod" vs nothing
    // is fine). Order does not otherwise affect correctness.
    // -------------------------------------------------------------------------
    const DEFINITION_KEYWORDS: &[&str] = &[
        "static ", // 7 bytes - before "struct" to avoid any future ambiguity
        "struct ", // 7 bytes
        "class ",  // 6 bytes
        "const ",  // 6 bytes
        "impl ",   // 5 bytes
        "type ",   // 5 bytes
        "enum ",   // 5 bytes
        "use ",    // 4 bytes
        "pub ",    // 4 bytes
        "mod ",    // 4 bytes
        "let ",    // 4 bytes
        "def ",    // 4 bytes
        "fn ",     // 3 bytes
        // Other
        "for ",   //
        "while ", //
        "if ",    //
        "loop ",  //
    ];

    // Slice the row content from the current byte position forward.
    // No allocation: this is a borrowed sub-slice of the existing &str.
    let remaining = &row_content[byte_pos..];

    // Scan keyword list. Linear scan over a tiny fixed list.
    for &keyword in DEFINITION_KEYWORDS {
        if remaining.starts_with(keyword) {
            return SyntaxHighlight::DefinitionWord {
                keyword_byte_len: keyword.len(),
            };
        }
    }

    // -------------------------------------------------------------------------
    // SYNTAX SYMBOLS (Category 1, cyan)
    // Single-character punctuation that makes code structure visible.
    // Checked after keywords so that the first character of a keyword
    // (which is always alphabetic) never reaches this check anyway.
    // In practice: no overlap is possible (no keyword starts with a symbol).
    //
    // The character at byte_pos is extracted by getting the first char
    // of the remaining slice. For ASCII symbols this is always 1 byte.
    // For safety we use chars().next() which handles UTF-8 correctly.
    // -------------------------------------------------------------------------
    const SYNTAX_SYMBOLS: &[char] = &[
        '(', ')', '[', ']', '{', '}', '<', '>', '=', ':', ';', '"', '\'', '\\', '&', '!', '#', '/',
        '*', ',', '`',
    ];

    // Get the first character at this byte position.
    // chars().next() is safe: we already checked byte_pos < row_content.len()
    // and remaining is a valid UTF-8 sub-slice.
    if let Some(ch) = remaining.chars().next() {
        for &symbol in SYNTAX_SYMBOLS {
            if ch == symbol {
                return SyntaxHighlight::SyntaxSymbol;
            }
        }
    }

    // -------------------------------------------------------------------------
    // No match: plain character, no highlighting.
    // -------------------------------------------------------------------------
    SyntaxHighlight::None
}

// =============================================================================
// TESTS: Syntax Highlighting
// =============================================================================

#[cfg(test)]
mod syntax_highlight_tests {
    use super::*;
    use std::path::Path;

    // --- buffy_is_plain_text_extension ---

    #[test]
    fn test_plain_text_extension_txt_is_plain() {
        let path = Path::new("readme.txt");
        assert!(
            buffy_is_plain_text_extension(Some(path)),
            "buffy_is_plain_text_extension: .txt should be plain text"
        );
    }

    #[test]
    fn test_plain_text_extension_log_is_plain() {
        let path = Path::new("app.log");
        assert!(
            buffy_is_plain_text_extension(Some(path)),
            "buffy_is_plain_text_extension: .log should be plain text"
        );
    }

    #[test]
    fn test_plain_text_extension_rs_is_not_plain() {
        let path = Path::new("main.rs");
        assert!(
            !buffy_is_plain_text_extension(Some(path)),
            "buffy_is_plain_text_extension: .rs should NOT be plain text"
        );
    }

    #[test]
    fn test_plain_text_extension_py_is_not_plain() {
        let path = Path::new("script.py");
        assert!(
            !buffy_is_plain_text_extension(Some(path)),
            "buffy_is_plain_text_extension: .py should NOT be plain text"
        );
    }

    #[test]
    fn test_plain_text_extension_none_path_is_not_plain() {
        assert!(
            !buffy_is_plain_text_extension(None),
            "buffy_is_plain_text_extension: None path should default to not plain (highlight)"
        );
    }

    #[test]
    fn test_plain_text_extension_no_extension_is_not_plain() {
        let path = Path::new("Makefile");
        assert!(
            !buffy_is_plain_text_extension(Some(path)),
            "buffy_is_plain_text_extension: no extension should default to not plain (highlight)"
        );
    }

    #[test]
    fn test_plain_text_extension_uppercase_txt_is_not_plain() {
        // Case-sensitive match: .TXT does not opt out (conservative, no heap conversion)
        let path = Path::new("readme.TXT");
        assert!(
            !buffy_is_plain_text_extension(Some(path)),
            "buffy_is_plain_text_extension: .TXT uppercase should not match (case-sensitive)"
        );
    }

    // --- buffy_get_syntax_highlight: SyntaxSymbol ---

    #[test]
    fn test_syntax_highlight_open_paren_is_symbol() {
        let row = "foo(bar)";
        let result = buffy_get_syntax_highlight(3, row);
        assert_eq!(
            result,
            SyntaxHighlight::SyntaxSymbol,
            "buffy_get_syntax_highlight: '(' should be SyntaxSymbol"
        );
    }

    #[test]
    fn test_syntax_highlight_close_brace_is_symbol() {
        let row = "fn foo() {}";
        // '}' is at byte index 10
        let result = buffy_get_syntax_highlight(10, row);
        assert_eq!(
            result,
            SyntaxHighlight::SyntaxSymbol,
            "buffy_get_syntax_highlight: '}}' should be SyntaxSymbol"
        );
    }

    #[test]
    fn test_syntax_highlight_colon_is_symbol() {
        let row = "x: u32";
        let result = buffy_get_syntax_highlight(1, row);
        assert_eq!(
            result,
            SyntaxHighlight::SyntaxSymbol,
            "buffy_get_syntax_highlight: ':' should be SyntaxSymbol"
        );
    }

    #[test]
    fn test_syntax_highlight_plain_letter_is_none() {
        let row = "hello";
        let result = buffy_get_syntax_highlight(0, row);
        assert_eq!(
            result,
            SyntaxHighlight::None,
            "buffy_get_syntax_highlight: plain letter should be None"
        );
    }

    // --- buffy_get_syntax_highlight: DefinitionWord ---

    #[test]
    fn test_syntax_highlight_fn_keyword() {
        let row = "fn main() {}";
        let result = buffy_get_syntax_highlight(0, row);
        assert_eq!(
            result,
            SyntaxHighlight::DefinitionWord {
                keyword_byte_len: 3
            },
            "buffy_get_syntax_highlight: 'fn ' should be DefinitionWord with byte_len 3"
        );
    }

    #[test]
    fn test_syntax_highlight_struct_keyword() {
        let row = "struct Foo {";
        let result = buffy_get_syntax_highlight(0, row);
        assert_eq!(
            result,
            SyntaxHighlight::DefinitionWord {
                keyword_byte_len: 7
            },
            "buffy_get_syntax_highlight: 'struct ' should be DefinitionWord with byte_len 7"
        );
    }

    #[test]
    fn test_syntax_highlight_let_keyword() {
        let row = "    let x = 5;";
        // "let " starts at byte 4
        let result = buffy_get_syntax_highlight(4, row);
        assert_eq!(
            result,
            SyntaxHighlight::DefinitionWord {
                keyword_byte_len: 4
            },
            "buffy_get_syntax_highlight: 'let ' should be DefinitionWord with byte_len 4"
        );
    }

    #[test]
    fn test_syntax_highlight_static_keyword() {
        let row = "static FOO: u32 = 1;";
        let result = buffy_get_syntax_highlight(0, row);
        assert_eq!(
            result,
            SyntaxHighlight::DefinitionWord {
                keyword_byte_len: 7
            },
            "buffy_get_syntax_highlight: 'static ' should be DefinitionWord with byte_len 7"
        );
    }

    #[test]
    fn test_syntax_highlight_pub_keyword() {
        let row = "pub fn foo() {}";
        let result = buffy_get_syntax_highlight(0, row);
        assert_eq!(
            result,
            SyntaxHighlight::DefinitionWord {
                keyword_byte_len: 4
            },
            "buffy_get_syntax_highlight: 'pub ' should be DefinitionWord with byte_len 4"
        );
    }

    #[test]
    fn test_syntax_highlight_fn_not_at_start_of_word() {
        // "fn" appears inside "unfn" - no space before it, but we match from byte_pos
        // If byte_pos points mid-word, we still match if it starts with "fn "
        // This is the known limitation: no word-boundary check.
        // This test documents actual behaviour (not asserting it is wrong,
        // just documenting that context-free matching is the design).
        let row = "xfn foo()";
        // byte_pos=1 points to 'f' of "fn foo()"
        let result = buffy_get_syntax_highlight(1, row);
        // "fn " starts at byte 1: this WILL match (no word boundary check by design)
        assert_eq!(
            result,
            SyntaxHighlight::DefinitionWord {
                keyword_byte_len: 3
            },
            "buffy_get_syntax_highlight: known behaviour: no word-boundary check, 'fn ' matches mid-string"
        );
    }

    #[test]
    fn test_syntax_highlight_out_of_bounds_returns_none() {
        let row = "hi";
        // byte_pos beyond string length
        let result = buffy_get_syntax_highlight(99, row);
        assert_eq!(
            result,
            SyntaxHighlight::None,
            "buffy_get_syntax_highlight: out-of-bounds byte_pos should return None safely"
        );
    }

    #[test]
    fn test_syntax_highlight_empty_string_returns_none() {
        let result = buffy_get_syntax_highlight(0, "");
        assert_eq!(
            result,
            SyntaxHighlight::None,
            "buffy_get_syntax_highlight: empty string should return None safely"
        );
    }

    #[test]
    fn test_syntax_highlight_multibyte_char_is_none() {
        // '世' is 3 bytes (E4 B8 96), not a symbol or keyword, should be None
        let row = "世界";
        let result = buffy_get_syntax_highlight(0, row);
        assert_eq!(
            result,
            SyntaxHighlight::None,
            "buffy_get_syntax_highlight: multi-byte non-symbol char should be None"
        );
    }
}

// // main.rs - Demonstration of zero-heap buffy formatting

// // mod buffy;
// // use buffy::*;
// // use std::io::{self, Write};

// // ANSI color codes (compile-time constants, no allocation)
// const RED: &str = "\x1b[31m";
// const GREEN: &str = "\x1b[32m";
// const YELLOW: &str = "\x1b[33m";
// // const BLUE: &str = "\x1b[34m";
// const CYAN: &str = "\x1b[36m";
// // const MAGENTA: &str = "\x1b[35m";
// const GRAY: &str = "\x1b[90m";
// // const WHITE: &str = "\x1b[37m";
// const BG_WHITE: &str = "\x1b[47m";
// // const BG_BLACK: &str = "\x1b[40m";
// const BOLD: &str = "\x1b[1m";
// const RESET: &str = "\x1b[0m";

// /// Demonstrates header formatting
// fn demo_header() -> io::Result<()> {
//     // No heap: literal string, direct output
//     buffy_repeat(&mut io::stdout(), '=', 70)?;
//     buffy_println("", &[])?;

//     // No heap: BOLD and RESET are static &str, written directly
//     buffy_println(
//         "{}BUFFY - ZERO HEAP FORMATTING DEMONSTRATION{}",
//         &[BuffyFormatArg::Str(BOLD), BuffyFormatArg::Str(RESET)],
//     )?;

//     buffy_repeat(&mut io::stdout(), '=', 70)?;
//     buffy_println("", &[])?;

//     // No heap: All text written directly to terminal
//     buffy_println("All output in this demo uses ZERO heap allocation:", &[])?;
//     buffy_println("  - No String objects created", &[])?;
//     buffy_println("  - No Vec allocations", &[])?;
//     buffy_println("  - No .to_string() calls", &[])?;
//     buffy_println("  - All conversions use stack buffers", &[])?;
//     buffy_println("  - Output written directly to terminal", &[])?;
//     buffy_println("", &[])?;

//     Ok(())
// }

// /// Demonstrates basic data types
// fn demo_basic_types() -> io::Result<()> {
//     buffy_println(
//         "{}1. BASIC DATA TYPES{}",
//         &[BuffyFormatArg::Str(BOLD), BuffyFormatArg::Str(RESET)],
//     )?;
//     buffy_repeat(&mut io::stdout(), '-', 70)?;
//     buffy_println("", &[])?;
//     // String slice - no allocation (borrowing existing data)
//     buffy_println("  String: {}", &[BuffyFormatArg::Str("Hello, world!")])?;

//     // Boolean - no allocation (writes "true" or "false" directly)
//     buffy_println("  Boolean true: {}", &[BuffyFormatArg::Bool(true)])?;
//     buffy_println("  Boolean false: {}", &[BuffyFormatArg::Bool(false)])?;

//     // Character - no allocation (encodes to UTF-8 on stack)
//     buffy_println("  Character: {}", &[BuffyFormatArg::Char('A')])?;
//     buffy_println("  Unicode char: {}", &[BuffyFormatArg::Char('→')])?;

//     // Multiple arguments
//     buffy_println(
//         "  Multiple: {} + {} = {}",
//         &[
//             BuffyFormatArg::Str("Hello"),
//             BuffyFormatArg::Char(','),
//             BuffyFormatArg::Str("world"),
//         ],
//     )?;

//     buffy_println("", &[])?;
//     Ok(())
// }

// /// Demonstrates unsigned integer formatting
// fn demo_numbers() -> io::Result<()> {
//     buffy_println(
//         "{}2. UNSIGNED INTEGERS{}",
//         &[BuffyFormatArg::Str(BOLD), BuffyFormatArg::Str(RESET)],
//     )?;
//     buffy_repeat(&mut io::stdout(), '-', 70)?;
//     buffy_println("", &[])?;
//     // All numbers converted to decimal strings on stack, no heap
//     buffy_println("  U8:    {}", &[BuffyFormatArg::U8(255)])?;
//     buffy_println("  U16:   {}", &[BuffyFormatArg::U16(65535)])?;
//     buffy_println("  U32:   {}", &[BuffyFormatArg::U32(4294967295)])?;
//     buffy_println("  U64:   {}", &[BuffyFormatArg::U64(18446744073709551615)])?;
//     buffy_println("  Usize: {}", &[BuffyFormatArg::Usize(123456789)])?;

//     // Real-world examples
//     let file_size: u64 = 1048576;
//     buffy_println("  File size: {} bytes", &[BuffyFormatArg::U64(file_size)])?;

//     let buffer_size: usize = 4096;
//     let current_pos: usize = 2048;
//     buffy_println(
//         "  Position: {} / {}",
//         &[BuffyFormatArg::Usize(current_pos), BuffyFormatArg::Usize(buffer_size)],
//     )?;

//     buffy_println("", &[])?;
//     Ok(())
// }

// /// Demonstrates signed integer formatting
// fn demo_signed_numbers() -> io::Result<()> {
//     buffy_println(
//         "{}3. SIGNED INTEGERS{}",
//         &[BuffyFormatArg::Str(BOLD), BuffyFormatArg::Str(RESET)],
//     )?;
//     buffy_repeat(&mut io::stdout(), '-', 70)?;
//     buffy_println("", &[])?;
//     // Signed numbers - negative sign handled on stack
//     buffy_println("  I8 positive:  {}", &[BuffyFormatArg::I8(127)])?;
//     buffy_println("  I8 negative:  {}", &[BuffyFormatArg::I8(-128)])?;
//     buffy_println("  I16 positive: {}", &[BuffyFormatArg::I16(32767)])?;
//     buffy_println("  I16 negative: {}", &[BuffyFormatArg::I16(-32768)])?;
//     buffy_println("  I32 positive: {}", &[BuffyFormatArg::I32(2147483647)])?;
//     buffy_println("  I32 negative: {}", &[BuffyFormatArg::I32(-2147483648)])?;
//     buffy_println("  I64 positive: {}", &[BuffyFormatArg::I64(9223372036854775807)])?;
//     buffy_println(
//         "  I64 negative: {}",
//         &[BuffyFormatArg::I64(-9223372036854775808)],
//     )?;

//     // Real-world: temperature offset
//     let temperature_offset: i32 = -15;
//     buffy_println(
//         "  Temperature offset: {}°C",
//         &[BuffyFormatArg::I32(temperature_offset)],
//     )?;

//     buffy_println("", &[])?;
//     Ok(())
// }

// /// Demonstrates hexadecimal formatting
// fn demo_hex_formatting() -> io::Result<()> {
//     buffy_println(
//         "{}4. HEXADECIMAL FORMATTING{}",
//         &[BuffyFormatArg::Str(BOLD), BuffyFormatArg::Str(RESET)],
//     )?;
//     buffy_repeat(&mut io::stdout(), '-', 70)?;
//     buffy_println("", &[])?;
//     // Hex conversion on stack, no heap
//     buffy_println("  U8 hex:  0x{}", &[BuffyFormatArg::U8Hex(0xFF)])?;
//     buffy_println("  U16 hex: 0x{}", &[BuffyFormatArg::U16Hex(0xABCD)])?;
//     buffy_println("  U32 hex: 0x{}", &[BuffyFormatArg::U32Hex(0xDEADBEEF)])?;

//     // Memory dump style (like in hex editor)
//     buffy_println(
//         "  Memory bytes: {} {} {} {} {}",
//         &[
//             BuffyFormatArg::U8Hex(0x48), // 'H'
//             BuffyFormatArg::U8Hex(0x65), // 'e'
//             BuffyFormatArg::U8Hex(0x6C), // 'l'
//             BuffyFormatArg::U8Hex(0x6C), // 'l'
//             BuffyFormatArg::U8Hex(0x6F), // 'o'
//         ],
//     )?;

//     // Address display
//     let memory_address: u32 = 0x00401000;
//     buffy_println("  Address: 0x{}", &[BuffyFormatArg::U32Hex(memory_address)])?;

//     buffy_println("", &[])?;
//     Ok(())
// }

// /// Demonstrates styled (colored) output
// fn demo_styled_output() -> io::Result<()> {
//     buffy_println(
//         "{}5. STYLED OUTPUT (ANSI COLORS){}",
//         &[BuffyFormatArg::Str(BOLD), BuffyFormatArg::Str(RESET)],
//     )?;
//     buffy_repeat(&mut io::stdout(), '-', 70)?;
//     buffy_println("", &[])?;
//     // Styled text - ANSI codes written directly, no string building
//     buffy_println(
//         "  Status: {}",
//         &[BuffyFormatArg::StrStyled(
//             "OK",
//             BuffyStyles {
//                 fg_color: Some(GREEN),
//                 bold: true,
//                 ..Default::default()
//             },
//         )],
//     )?;

//     buffy_println(
//         "  Status: {}",
//         &[BuffyFormatArg::StrStyled(
//             "ERROR",
//             BuffyStyles {
//                 fg_color: Some(RED),
//                 bold: true,
//                 ..Default::default()
//             },
//         )],
//     )?;

//     buffy_println(
//         "  Status: {}",
//         &[BuffyFormatArg::StrStyled(
//             "WARNING",
//             BuffyStyles {
//                 fg_color: Some(YELLOW),
//                 bold: true,
//                 ..Default::default()
//             },
//         )],
//     )?;

//     // Styled numbers
//     let count: u64 = 42;
//     buffy_println(
//         "  Count: {}",
//         &[BuffyFormatArg::U64Styled(
//             count,
//             BuffyStyles {
//                 fg_color: Some(CYAN),
//                 bold: false,
//                 ..Default::default()
//             },
//         )],
//     )?;

//     // Styled hex with background (like cursor highlight)
//     let highlighted_byte: u8 = 0xFF;
//     buffy_println(
//         "  Highlighted byte: {}",
//         &[BuffyFormatArg::U8HexStyled(
//             highlighted_byte,
//             BuffyStyles {
//                 fg_color: Some(RED),
//                 bg_color: Some(BG_WHITE),
//                 bold: true,
//                 ..Default::default()
//             },
//         )],
//     )?;

//     buffy_println("", &[])?;
//     Ok(())
// }

// /// Demonstrates alignment for tables
// fn demo_alignment() -> io::Result<()> {
//     buffy_println(
//         "{}6. ALIGNMENT (TABLES & COLUMNS){}",
//         &[BuffyFormatArg::Str(BOLD), BuffyFormatArg::Str(RESET)],
//     )?;
//     buffy_repeat(&mut io::stdout(), '-', 70)?;
//     buffy_println("", &[])?;
//     // Table header
//     buffy_println(
//         "  {:<15} {:>10} {:^8}",
//         &[
//             BuffyFormatArg::Str("NAME"),
//             BuffyFormatArg::Str("SIZE"),
//             BuffyFormatArg::Str("STATUS"),
//         ],
//     )?;
//     buffy_repeat(&mut io::stdout(), '-', 35)?;
//     buffy_println("", &[])?; // ← ADD THIS

//     // Table rows - alignment applied on stack
//     buffy_println(
//         "  {:<15} {:>10} {:^8}",
//         &[
//             BuffyFormatArg::Str("config.txt"),
//             BuffyFormatArg::U64(1024),
//             BuffyFormatArg::Str("OK"),
//         ],
//     )?;

//     buffy_println(
//         "  {:<15} {:>10} {:^8}",
//         &[
//             BuffyFormatArg::Str("data.bin"),
//             BuffyFormatArg::U64(524288),
//             BuffyFormatArg::Str("OK"),
//         ],
//     )?;

//     buffy_println(
//         "  {:<15} {:>10} {:^8}",
//         &[
//             BuffyFormatArg::Str("large.dat"),
//             BuffyFormatArg::U64(104857600),
//             BuffyFormatArg::Str("OK"),
//         ],
//     )?;

//     buffy_println("", &[])?;

//     // Financial-style right-aligned numbers
//     buffy_println("  Financial Report:", &[])?;
//     buffy_println("    Income:  ${:>10}", &[BuffyFormatArg::U64(50000)])?;
//     buffy_println("    Expense: ${:>10}", &[BuffyFormatArg::U64(30000)])?;
//     buffy_println("    Profit:  ${:>10}", &[BuffyFormatArg::U64(20000)])?;

//     buffy_println("", &[])?;
//     Ok(())
// }

// /// Demonstrates TUI legend/menu (real-world use case)
// fn demo_tui_legend() -> io::Result<()> {
//     buffy_println(
//         "{}7. TUI LEGEND/MENU{}",
//         &[BuffyFormatArg::Str(BOLD), BuffyFormatArg::Str(RESET)],
//     )?;
//     buffy_repeat(&mut io::stdout(), '-', 70)?;
//     buffy_println("", &[])?;
//     // Build legend with hotkey highlighting (max 8 args per call)
//     // No heap - colors are static &str, written directly

//     buffy_print("  ", &[])?;

//     // First part: quit, save, undo, delete
//     buffy_print(
//         "{}q{}uit {}s{}ave {}u{}ndo {}d{}el ",
//         &[
//             BuffyFormatArg::Str(RED),
//             BuffyFormatArg::Str(YELLOW),
//             BuffyFormatArg::Str(RED),
//             BuffyFormatArg::Str(YELLOW),
//             BuffyFormatArg::Str(RED),
//             BuffyFormatArg::Str(YELLOW),
//             BuffyFormatArg::Str(RED),
//             BuffyFormatArg::Str(YELLOW),
//         ],
//     )?;

//     // Second part: insert, view, hex, help
//     buffy_print(
//         "{}i{}ns {}v{}iew {}h{}ex {}?{}help",
//         &[
//             BuffyFormatArg::Str(RED),
//             BuffyFormatArg::Str(YELLOW),
//             BuffyFormatArg::Str(RED),
//             BuffyFormatArg::Str(YELLOW),
//             BuffyFormatArg::Str(RED),
//             BuffyFormatArg::Str(YELLOW),
//             BuffyFormatArg::Str(RED),
//             BuffyFormatArg::Str(YELLOW),
//         ],
//     )?;

//     buffy_println("{}", &[BuffyFormatArg::Str(RESET)])?;
//     buffy_println("", &[])?;

//     Ok(())
// }

// /// Demonstrates hex editor line (complex real-world example)
// fn demo_hex_editor_line() -> io::Result<()> {
//     buffy_println(
//         "{}8. HEX EDITOR LINE{}",
//         &[BuffyFormatArg::Str(BOLD), BuffyFormatArg::Str(RESET)],
//     )?;
//     buffy_repeat(&mut io::stdout(), '-', 70)?;
//     buffy_println("", &[])?;
//     let bytes: [u8; 6] = [0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20]; // "Hello"
//     let offset: u32 = 0x00000000;
//     let cursor_position: usize = 4; // Highlight 5th byte

//     // Offset column (styled cyan)
//     buffy_print("0x{}", &[BuffyFormatArg::U32Hex(offset)])?;

//     // // example of alternative to buffy-print
//     // stdout.write_all(b" | ")?;

//     buffy_print(" | ", &[])?;

//     // Hex column with cursor highlighting
//     for (i, byte) in bytes.iter().enumerate() {
//         if i == cursor_position {
//             // Cursor position - highlighted
//             buffy_print(
//                 "{}",
//                 &[BuffyFormatArg::U8HexStyled(
//                     *byte,
//                     BuffyStyles {
//                         fg_color: Some(RED),
//                         bg_color: Some(BG_WHITE),
//                         bold: true,
//                         ..Default::default()
//                     },
//                 )],
//             )?;
//         } else {
//             // Normal hex byte
//             buffy_print("{}", &[BuffyFormatArg::U8Hex(*byte)])?;
//         }
//         buffy_print(" ", &[])?;
//     }

//     buffy_print("| ", &[])?;

//     // ASCII column
//     for (i, byte) in bytes.iter().enumerate() {
//         let ch = if *byte >= 0x20 && *byte <= 0x7E {
//             *byte as char
//         } else {
//             '.'
//         };

//         if i == cursor_position {
//             // Cursor position - highlighted
//             buffy_print(
//                 "{}",
//                 &[BuffyFormatArg::CharStyled(
//                     // ← Use CharStyled, not U8HexStyled!
//                     ch, // ← Use ch, not *byte!
//                     BuffyStyles {
//                         fg_color: Some(RED),
//                         bg_color: Some(BG_WHITE),
//                         bold: true,
//                         ..Default::default()
//                     },
//                 )],
//             )?;
//         } else {
//             buffy_print("{}", &[BuffyFormatArg::Char(ch)])?;
//         }
//     }

//     buffy_print("\n", &[])?;

//     buffy_println("", &[])?;
//     Ok(())
// }

// fn demo_incremental_output() -> io::Result<()> {
//     buffy_println(
//         "{}9. INCREMENTAL OUTPUT{}",
//         &[BuffyFormatArg::Str(BOLD), BuffyFormatArg::Str(RESET)],
//     )?;
//     buffy_repeat(&mut io::stdout(), '-', 70)?;
//     buffy_println("", &[])?;

//     let mut stdout = io::stdout(); // ← Keep this!

//     buffy_print("  ", &[])?;
//     buffy_print("Loading file ", &[])?;

//     let filename = "config.txt";
//     buffy_print(
//         "{}",
//         &[BuffyFormatArg::StrStyled(
//             filename,
//             BuffyStyles {
//                 fg_color: Some(CYAN),
//                 bold: true,
//                 ..Default::default()
//             },
//         )],
//     )?;

//     buffy_print("... ", &[])?;

//     // Progress - note: add a flush after each output.
//     for i in 1..=3 {
//         buffy_print("{}%", &[BuffyFormatArg::U8(i * 33)])?;
//         buffy_print(".", &[])?;
//         stdout.flush()?;
//         std::thread::sleep(std::time::Duration::from_millis(1000));
//     }

//     buffy_println(" Done!", &[])?;

//     // Progress bar - ADD FLUSH INSIDE LOOP!
//     buffy_print("  Progress: [", &[])?;
//     for i in 0..10 {
//         if i < 7 {
//             buffy_write_styled(
//                 &mut stdout,
//                 "█",
//                 Some(BuffyStyles {
//                     fg_color: Some(GREEN),
//                     ..Default::default()
//                 }),
//             )?;
//         } else {
//             buffy_write_styled(
//                 &mut stdout,
//                 "░",
//                 Some(BuffyStyles {
//                     fg_color: Some(GRAY),
//                     ..Default::default()
//                 }),
//             )?;
//         }
//         stdout.flush()?;
//         std::thread::sleep(std::time::Duration::from_millis(100));
//     }
//     buffy_println("] {}%", &[BuffyFormatArg::U8(70)])?;

//     buffy_println("", &[])?;
//     Ok(())
// }

// /// Demonstrates error message display
// fn demo_error_display() -> io::Result<()> {
//     buffy_println(
//         "{}10. ERROR MESSAGES{}",
//         &[BuffyFormatArg::Str(BOLD), BuffyFormatArg::Str(RESET)],
//     )?;
//     buffy_repeat(&mut io::stdout(), '-', 70)?;
//     buffy_println("", &[])?;
//     // Error with styled prefix
//     buffy_println(
//         "  {}: File not found",
//         &[BuffyFormatArg::StrStyled(
//             "ERROR",
//             BuffyStyles {
//                 fg_color: Some(RED),
//                 bold: true,
//                 ..Default::default()
//             },
//         )],
//     )?;

//     // Error with context
//     let error_code: u32 = 404;
//     let line_number: u32 = 42;
//     buffy_println(
//         "  {}: Invalid syntax at line {} (code: {})",
//         &[
//             BuffyFormatArg::StrStyled(
//                 "ERROR",
//                 BuffyStyles {
//                     fg_color: Some(RED),
//                     bold: true,
//                     ..Default::default()
//                 },
//             ),
//             BuffyFormatArg::U32(line_number),
//             BuffyFormatArg::U32(error_code),
//         ],
//     )?;

//     // Warning
//     buffy_println(
//         "  {}: Low memory available",
//         &[BuffyFormatArg::StrStyled(
//             "WARNING",
//             BuffyStyles {
//                 fg_color: Some(YELLOW),
//                 bold: true,
//                 ..Default::default()
//             },
//         )],
//     )?;

//     buffy_println("", &[])?;
//     Ok(())
// }

// /// Demonstrates footer
// fn demo_footer() -> io::Result<()> {
//     buffy_repeat(&mut io::stdout(), '=', 70)?;
//     buffy_println("", &[])?;
//     buffy_println(
//         "{}DEMONSTRATION COMPLETE{}",
//         &[BuffyFormatArg::Str(BOLD), BuffyFormatArg::Str(RESET)],
//     )?;
//     buffy_repeat(&mut io::stdout(), '=', 70)?;

//     buffy_println("", &[])?;

//     buffy_println(
//         "{}Memory Usage Summary:{}",
//         &[BuffyFormatArg::Str(BOLD), BuffyFormatArg::Str(RESET)],
//     )?;
//     buffy_println(
//         "  Heap allocations: {}",
//         &[BuffyFormatArg::StrStyled(
//             "ZERO",
//             BuffyStyles {
//                 fg_color: Some(GREEN),
//                 bold: true,
//                 ..Default::default()
//             },
//         )],
//     )?;
//     buffy_println(
//         "  String objects: {}",
//         &[BuffyFormatArg::StrStyled(
//             "ZERO",
//             BuffyStyles {
//                 fg_color: Some(GREEN),
//                 bold: true,
//                 ..Default::default()
//             },
//         )],
//     )?;
//     buffy_println(
//         "  Vec allocations: {}",
//         &[BuffyFormatArg::StrStyled(
//             "ZERO",
//             BuffyStyles {
//                 fg_color: Some(GREEN),
//                 bold: true,
//                 ..Default::default()
//             },
//         )],
//     )?;
//     buffy_println(
//         "  All formatting: {}",
//         &[BuffyFormatArg::StrStyled(
//             "STACK ONLY",
//             BuffyStyles {
//                 fg_color: Some(GREEN),
//                 bold: true,
//                 ..Default::default()
//             },
//         )],
//     )?;

//     Ok(())
// }

// fn main() -> io::Result<()> {
//     // All output goes directly to terminal with ZERO heap allocation

//     demo_header()?;
//     demo_basic_types()?;
//     demo_numbers()?;
//     demo_signed_numbers()?;
//     demo_hex_formatting()?;
//     demo_styled_output()?;
//     demo_alignment()?;
//     demo_tui_legend()?;
//     demo_hex_editor_line()?;
//     demo_incremental_output()?;
//     demo_error_display()?;
//     demo_footer()?;

//     Ok(())
// }
