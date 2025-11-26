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
    // Unsigned integers
    Str(&'a str),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Usize(usize),

    // Signed integers
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Isize(isize),

    // Hex formatting
    U8Hex(u8),
    U16Hex(u16),
    U32Hex(u32),

    // Other types
    Bool(bool),
    Char(char),
    Path(&'a Path),

    // Styled variants (adds ANSI codes)
    CharStyled(char, BuffyStyles),
    StrStyled(&'a str, BuffyStyles),
    U8Styled(u8, BuffyStyles),
    U64Styled(u64, BuffyStyles),
    UsizeStyled(usize, BuffyStyles),
    U8HexStyled(u8, BuffyStyles),
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

/// Converts i64 to decimal string with sign in provided stack buffer
///
/// Memory: should be all stack, no heap
fn format_i64_to_buffer<'a>(value: i64, buf: &'a mut [u8]) -> Option<&'a str> {
    if buf.is_empty() {
        return None;
    }

    if value == 0 {
        buf[0] = b'0';
        return std::str::from_utf8(&buf[..1]).ok();
    }

    let (is_negative, abs_value) = if value < 0 {
        (true, value.wrapping_abs() as u64)
    } else {
        (false, value as u64)
    };

    let mut temp = [0u8; 20];
    let mut pos = 0;
    let mut num = abs_value;

    while num > 0 {
        temp[pos] = b'0' + (num % 10) as u8;
        num /= 10;
        pos += 1;
    }

    let total_len = if is_negative { pos + 1 } else { pos };

    if total_len > buf.len() {
        return None;
    }

    let mut buf_pos = 0;

    if is_negative {
        buf[buf_pos] = b'-';
        buf_pos += 1;
    }

    for i in 0..pos {
        buf[buf_pos + i] = temp[pos - 1 - i];
    }

    std::str::from_utf8(&buf[..total_len]).ok()
}

/// Converts u8 to 2-digit uppercase hex in provided stack buffer
///
/// Memory: should be all stack, no heap
fn format_u8_hex_to_buffer<'a>(value: u8, buf: &'a mut [u8]) -> Option<&'a str> {
    if buf.len() < 2 {
        return None;
    }

    let hex_chars = b"0123456789ABCDEF";
    buf[0] = hex_chars[(value >> 4) as usize];
    buf[1] = hex_chars[(value & 0x0F) as usize];

    std::str::from_utf8(&buf[..2]).ok()
}

/// Converts u16 to 4-digit uppercase hex in provided stack buffer
///
/// Memory: should be all stack, no heap
fn format_u16_hex_to_buffer<'a>(value: u16, buf: &'a mut [u8]) -> Option<&'a str> {
    if buf.len() < 4 {
        return None;
    }

    let hex_chars = b"0123456789ABCDEF";
    buf[0] = hex_chars[((value >> 12) & 0x0F) as usize];
    buf[1] = hex_chars[((value >> 8) & 0x0F) as usize];
    buf[2] = hex_chars[((value >> 4) & 0x0F) as usize];
    buf[3] = hex_chars[(value & 0x0F) as usize];

    std::str::from_utf8(&buf[..4]).ok()
}

/// Converts u32 to 8-digit uppercase hex in provided stack buffer
///
/// Memory: should be all stack, no heap
fn format_u32_hex_to_buffer<'a>(value: u32, buf: &'a mut [u8]) -> Option<&'a str> {
    if buf.len() < 8 {
        return None;
    }

    let hex_chars = b"0123456789ABCDEF";
    for i in 0..8 {
        let shift = 28 - (i * 4);
        buf[i] = hex_chars[((value >> shift) & 0x0F) as usize];
    }

    std::str::from_utf8(&buf[..8]).ok()
}

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
                    BuffyFormatArg::U8(n) => {
                        let s = format_u64_to_buffer(*n as u64, &mut num_buf).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                        })?;
                        (s, false, BuffyStyles::default())
                    }
                    BuffyFormatArg::U16(n) => {
                        let s = format_u64_to_buffer(*n as u64, &mut num_buf).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                        })?;
                        (s, false, BuffyStyles::default())
                    }
                    BuffyFormatArg::U32(n) => {
                        let s = format_u64_to_buffer(*n as u64, &mut num_buf).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                        })?;
                        (s, false, BuffyStyles::default())
                    }
                    BuffyFormatArg::U64(n) => {
                        let s = format_u64_to_buffer(*n, &mut num_buf).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                        })?;
                        (s, false, BuffyStyles::default())
                    }
                    BuffyFormatArg::Usize(n) => {
                        let s = format_u64_to_buffer(*n as u64, &mut num_buf).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                        })?;
                        (s, false, BuffyStyles::default())
                    }
                    BuffyFormatArg::I8(n) => {
                        let s = format_i64_to_buffer(*n as i64, &mut num_buf).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                        })?;
                        (s, false, BuffyStyles::default())
                    }
                    BuffyFormatArg::I16(n) => {
                        let s = format_i64_to_buffer(*n as i64, &mut num_buf).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                        })?;
                        (s, false, BuffyStyles::default())
                    }
                    BuffyFormatArg::I32(n) => {
                        let s = format_i64_to_buffer(*n as i64, &mut num_buf).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                        })?;
                        (s, false, BuffyStyles::default())
                    }
                    BuffyFormatArg::I64(n) => {
                        let s = format_i64_to_buffer(*n, &mut num_buf).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                        })?;
                        (s, false, BuffyStyles::default())
                    }
                    BuffyFormatArg::Isize(n) => {
                        let s = format_i64_to_buffer(*n as i64, &mut num_buf).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                        })?;
                        (s, false, BuffyStyles::default())
                    }
                    BuffyFormatArg::U8Hex(n) => {
                        let s = format_u8_hex_to_buffer(*n, &mut num_buf).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Hex conversion failed")
                        })?;
                        (s, false, BuffyStyles::default())
                    }
                    BuffyFormatArg::U16Hex(n) => {
                        let s = format_u16_hex_to_buffer(*n, &mut num_buf).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Hex conversion failed")
                        })?;
                        (s, false, BuffyStyles::default())
                    }
                    BuffyFormatArg::U32Hex(n) => {
                        let s = format_u32_hex_to_buffer(*n, &mut num_buf).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Hex conversion failed")
                        })?;
                        (s, false, BuffyStyles::default())
                    }
                    BuffyFormatArg::Bool(b) => (
                        if *b { "true" } else { "false" },
                        false,
                        BuffyStyles::default(),
                    ),
                    BuffyFormatArg::Char(c) => {
                        let mut char_buf = [0u8; 4];
                        let char_str = c.encode_utf8(&mut char_buf);
                        let len = char_str.len();
                        num_buf[..len].copy_from_slice(char_str.as_bytes());
                        let s = std::str::from_utf8(&num_buf[..len]).map_err(|_| {
                            io::Error::new(io::ErrorKind::Other, "Char conversion failed")
                        })?;
                        (s, false, BuffyStyles::default())
                    }
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
                    BuffyFormatArg::U8Styled(n, st) => {
                        let s = format_u64_to_buffer(*n as u64, &mut num_buf).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                        })?;
                        (s, true, *st)
                    }
                    BuffyFormatArg::U64Styled(n, st) => {
                        let s = format_u64_to_buffer(*n, &mut num_buf).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                        })?;
                        (s, true, *st)
                    }
                    BuffyFormatArg::UsizeStyled(n, st) => {
                        let s = format_u64_to_buffer(*n as u64, &mut num_buf).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                        })?;
                        (s, true, *st)
                    }
                    BuffyFormatArg::U8HexStyled(n, st) => {
                        let s = format_u8_hex_to_buffer(*n, &mut num_buf).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Hex conversion failed")
                        })?;
                        (s, true, *st)
                    }
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

/// Writes formatted output to any writer.
///
/// Memory: should be all stack, no heap
/// Same direct-write logic as buffy_print() but writes to provided writer.
pub fn buffy_write_basic<W: Write>(
    writer: &mut W,
    template: &str,
    args: &[BuffyFormatArg],
) -> io::Result<()> {
    const MAX_ARGS: usize = 8;

    if args.len() > MAX_ARGS {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Too many arguments (max 8)",
        ));
    }

    let mut arg_index = 0;
    let mut pos = 0;

    // Stack buffers for conversions
    let mut num_buf = [0u8; 20];
    let mut style_buf = [0u8; 64];
    let mut align_buf = [0u8; 128];

    while pos < template.len() {
        if let Some(brace_pos) = template[pos..].find('{') {
            let absolute_brace = pos + brace_pos;

            if brace_pos > 0 {
                writer.write_all(template[pos..absolute_brace].as_bytes())?;
            }

            if let Some(close_pos) = template[absolute_brace..].find('}') {
                let absolute_close = absolute_brace + close_pos;
                let placeholder = &template[absolute_brace + 1..absolute_close];

                let spec = match parse_format_spec(placeholder) {
                    Some(s) => s,
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "Invalid format specifier",
                        ));
                    }
                };

                if arg_index >= args.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Not enough arguments for format string",
                    ));
                }

                // Convert argument (same logic as buffy_print)
                let (value_str, has_style, style) = match &args[arg_index] {
                    BuffyFormatArg::Str(s) => (*s, false, BuffyStyles::default()),
                    BuffyFormatArg::U8(n) => {
                        let s = format_u64_to_buffer(*n as u64, &mut num_buf).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                        })?;
                        (s, false, BuffyStyles::default())
                    }
                    BuffyFormatArg::U64(n) => {
                        let s = format_u64_to_buffer(*n, &mut num_buf).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Number conversion failed")
                        })?;
                        (s, false, BuffyStyles::default())
                    }
                    BuffyFormatArg::U8Hex(n) => {
                        let s = format_u8_hex_to_buffer(*n, &mut num_buf).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Hex conversion failed")
                        })?;
                        (s, false, BuffyStyles::default())
                    }
                    BuffyFormatArg::Bool(b) => (
                        if *b { "true" } else { "false" },
                        false,
                        BuffyStyles::default(),
                    ),
                    BuffyFormatArg::Char(c) => {
                        let mut char_buf = [0u8; 4];
                        let char_str = c.encode_utf8(&mut char_buf);
                        let len = char_str.len();
                        num_buf[..len].copy_from_slice(char_str.as_bytes());
                        let s = std::str::from_utf8(&num_buf[..len]).map_err(|_| {
                            io::Error::new(io::ErrorKind::Other, "Char conversion failed")
                        })?;
                        (s, false, BuffyStyles::default())
                    }

                    // Add other types as needed (same as buffy_print)
                    BuffyFormatArg::StrStyled(s, st) => (*s, true, *st),
                    BuffyFormatArg::U8HexStyled(n, st) => {
                        let s = format_u8_hex_to_buffer(*n, &mut num_buf).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::Other, "Hex conversion failed")
                        })?;
                        (s, true, *st)
                    }

                    // Add remaining types as needed
                    _ => {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            "Unsupported argument type",
                        ));
                    }
                };

                if has_style {
                    let ansi = style_to_ansi(style, &mut style_buf).ok_or_else(|| {
                        io::Error::new(io::ErrorKind::Other, "BuffyStyles conversion failed")
                    })?;
                    writer.write_all(ansi.as_bytes())?;
                }

                let aligned = apply_alignment(value_str, spec, &mut align_buf)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Alignment failed"))?;
                writer.write_all(aligned.as_bytes())?;

                if has_style {
                    writer.write_all(b"\x1b[0m")?;
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
            writer.write_all(template[pos..].as_bytes())?;
            break;
        }
    }

    Ok(())
}

/// Repeats a character N times, writing directly to writer.
///
/// ## Project Context
/// Helper for drawing horizontal lines, borders, and padding in TUI.
/// Avoids String::repeat() which allocates heap.
///
/// Memory: should be all stack, no heap
/// Uses 64-byte stack buffer, writes in chunks if repeat count exceeds buffer.
///
/// ## Parameters
/// - writer: Output destination
/// - ch: Character to repeat (any UTF-8 character)
/// - count: Number of repetitions
///
/// ## Returns
/// - Ok(()): Successfully written
/// - Err(io::Error): Write failed
///
/// ## Examples
/// ```rust
/// // Horizontal line
/// buffy_repeat(&mut stdout, '=', 70)?;
/// buffy_println("", &[])?;
///
/// // Padding
/// buffy_repeat(&mut stdout, ' ', 4)?;
/// buffy_print("Indented text", &[])?;
/// ```
pub fn buffy_repeat<W: Write>(writer: &mut W, ch: char, count: usize) -> io::Result<()> {
    if count == 0 {
        return Ok(());
    }

    // Encode character to UTF-8 on stack
    let mut char_buf = [0u8; 4];
    let char_str = ch.encode_utf8(&mut char_buf);
    let char_len = char_str.len();

    // Use 64-byte stack buffer for batching
    let mut buf = [0u8; 64];
    let chars_per_batch = buf.len() / char_len;

    if chars_per_batch == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Character too large for buffer",
        ));
    }

    // Fill buffer with character pattern
    let mut buf_pos = 0;
    for _ in 0..chars_per_batch {
        if buf_pos + char_len <= buf.len() {
            buf[buf_pos..buf_pos + char_len].copy_from_slice(&char_buf[..char_len]);
            buf_pos += char_len;
        }
    }
    let batch_size = buf_pos;

    // Write full batches
    let full_batches = count / chars_per_batch;
    for _ in 0..full_batches {
        writer.write_all(&buf[..batch_size])?;
    }

    // Write remaining characters
    let remaining = count % chars_per_batch;
    for _ in 0..remaining {
        writer.write_all(&char_buf[..char_len])?;
    }

    Ok(())
}

/// Writes a single styled text chunk directly to writer.
///
/// Memory: should be all stack, no heap
/// Writes ANSI codes (if any), text, and reset directly.
pub fn buffy_write_styled<W: Write>(
    writer: &mut W,
    text: &str,
    style: Option<BuffyStyles>,
) -> io::Result<()> {
    if let Some(s) = style {
        let mut style_buf = [0u8; 64];
        if let Some(ansi) = style_to_ansi(s, &mut style_buf) {
            writer.write_all(ansi.as_bytes())?;
        }
    }

    writer.write_all(text.as_bytes())?;

    if style.is_some() {
        writer.write_all(b"\x1b[0m")?;
    }

    Ok(())
}

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

    #[test]
    fn test_format_i64_negative() {
        let mut buf = [0u8; 20];
        let result = format_i64_to_buffer(-42, &mut buf);
        assert_eq!(result, Some("-42"));
    }

    #[test]
    fn test_format_hex() {
        let mut buf = [0u8; 8];
        let result = format_u8_hex_to_buffer(0xFF, &mut buf);
        assert_eq!(result, Some("FF"));
    }

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
