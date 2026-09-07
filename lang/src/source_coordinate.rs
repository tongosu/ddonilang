//! Internal typed source-coordinate authority.
//!
//! Byte offsets stay authoritative inside the parser/runtime. Line/column
//! values are typed projections only; this module does not define a public
//! wire schema or a snapshot-identity hash algorithm.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Utf16Position {
    /// Zero-based line number, as used by LSP.
    pub line: u32,
    /// Zero-based UTF-16 code-unit offset, as used by LSP.
    pub character: u32,
}

pub fn utf16_position(source: &str, byte_offset: usize) -> Option<Utf16Position> {
    if byte_offset > source.len() || !source.is_char_boundary(byte_offset) {
        return None;
    }
    let prefix = &source[..byte_offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32;
    let line_start = prefix.rfind('\n').map(|index| index + 1).unwrap_or(0);
    let character = source[line_start..byte_offset].encode_utf16().count() as u32;
    Some(Utf16Position { line, character })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_projection_counts_non_bmp_scalar_as_two_code_units() {
        let source = "🙂이름\r\n값";
        assert_eq!(
            utf16_position(source, source.find('이').expect("name")),
            Some(Utf16Position {
                line: 0,
                character: 2,
            })
        );
        assert_eq!(
            utf16_position(source, source.find('값').expect("value")),
            Some(Utf16Position {
                line: 1,
                character: 0,
            })
        );
    }
}
