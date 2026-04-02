use std::ffi::c_void;
use std::os::raw::{c_char, c_uint, c_int};

// Renderer flags from md4x-ansi.h
const MD_ANSI_FLAG_DEBUG: c_uint = 0x0001;
const MD_ANSI_FLAG_SKIP_UTF8_BOM: c_uint = 0x0002;

// MD_DIALECT_ALL from md4x.h
const MD_DIALECT_ALL: c_uint =
    0x0004  // MD_FLAG_PERMISSIVEURLAUTOLINKS
    | 0x0008  // MD_FLAG_PERMISSIVEEMAILAUTOLINKS
    | 0x0400  // MD_FLAG_PERMISSIVEWWWAUTOLINKS
    | 0x0100  // MD_FLAG_TABLES
    | 0x0200  // MD_FLAG_STRIKETHROUGH
    | 0x0800  // MD_FLAG_TASKLISTS
    | 0x1000  // MD_FLAG_LATEXMATHSPANS
    | 0x2000  // MD_FLAG_WIKILINKS
    | 0x4000  // MD_FLAG_UNDERLINE
    | 0x10000 // MD_FLAG_FRONTMATTER
    | 0x20000 // MD_FLAG_COMPONENTS
    | 0x40000 // MD_FLAG_ATTRIBUTES
    | 0x80000; // MD_FLAG_ALERTS

extern "C" {
    fn md_ansi(
        input: *const c_char,
        input_size: c_uint,
        process_output: extern "C" fn(*const c_char, c_uint, *mut c_void),
        userdata: *mut c_void,
        parser_flags: c_uint,
        renderer_flags: c_uint,
    ) -> c_int;
}

extern "C" fn collect_output(text: *const c_char, size: c_uint, userdata: *mut c_void) {
    let buf = unsafe { &mut *(userdata as *mut Vec<u8>) };
    let slice = unsafe { std::slice::from_raw_parts(text as *const u8, size as usize) };
    buf.extend_from_slice(slice);
}

/// Render markdown to ANSI-colored terminal output.
pub fn render_ansi(markdown: &str) -> String {
    let mut output: Vec<u8> = Vec::with_capacity(markdown.len() * 2);
    let userdata = &mut output as *mut Vec<u8> as *mut c_void;

    let result = unsafe {
        md_ansi(
            markdown.as_ptr() as *const c_char,
            markdown.len() as c_uint,
            collect_output,
            userdata,
            MD_DIALECT_ALL,
            MD_ANSI_FLAG_DEBUG | MD_ANSI_FLAG_SKIP_UTF8_BOM,
        )
    };

    if result != 0 {
        return markdown.to_string(); // fallback to raw markdown
    }

    let text = String::from_utf8_lossy(&output).to_string();
    cleanup_ansi(&text)
}

/// Clean up ANSI codes for skim preview compatibility:
/// - Remove dim/dim-reset
/// - Remove background colors
/// - Replace style-off codes (\x1b[22m, \x1b[24m, etc.) with full reset \x1b[0m
///   because skim only understands \x1b[0m for turning off styles
fn cleanup_ansi(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_dim = false;

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Collect the full escape sequence
            let mut seq = String::from(ch);
            if let Some(&'[') = chars.peek() {
                seq.push(chars.next().unwrap());
                while let Some(&c) = chars.peek() {
                    seq.push(chars.next().unwrap());
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            }

            // Dim on → replace with muted gray
            if seq == "\x1b[2m" {
                in_dim = true;
                result.push_str("\x1b[38;5;245m"); // gray
                continue;
            }
            // \x1b[22m = bold off AND dim off
            if seq == "\x1b[22m" {
                if in_dim {
                    in_dim = false;
                    result.push_str("\x1b[0m"); // reset gray
                    continue;
                }
                // Was closing bold → use full reset
                result.push_str("\x1b[0m");
                continue;
            }
            // Skip background colors \x1b[40m..\x1b[47m
            if seq.starts_with("\x1b[4") && seq.len() == 5 && seq.ends_with('m') {
                let digit = seq.chars().nth(3).unwrap_or('0');
                if digit >= '0' && digit <= '7' {
                    continue;
                }
            }
            // Skip extended background colors
            if seq.starts_with("\x1b[48;") {
                continue;
            }
            // Replace style-off codes with full reset
            // \x1b[24m = underline off, \x1b[23m = italic off, \x1b[29m = strikethrough off
            if seq == "\x1b[24m" || seq == "\x1b[23m" || seq == "\x1b[29m" {
                result.push_str("\x1b[0m");
                continue;
            }
            result.push_str(&seq);
        } else {
            result.push(ch);
        }
    }

    result
}
