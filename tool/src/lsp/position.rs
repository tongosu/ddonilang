use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionEncoding {
    Utf8,
    Utf16,
    Utf32,
}

impl PositionEncoding {
    pub fn from_lsp_name(name: &str) -> Option<Self> {
        match name {
            "utf-8" => Some(Self::Utf8),
            "utf-16" => Some(Self::Utf16),
            "utf-32" => Some(Self::Utf32),
            _ => None,
        }
    }

    pub fn as_lsp_name(self) -> &'static str {
        match self {
            Self::Utf8 => "utf-8",
            Self::Utf16 => "utf-16",
            Self::Utf32 => "utf-32",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LspRange {
    pub start: LspPosition,
    pub end: LspPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteRange {
    pub start: usize,
    pub end: usize,
}

impl ByteRange {
    pub fn new(start: usize, end: usize) -> Result<Self, PositionError> {
        if start > end {
            return Err(PositionError::RangeReversed);
        }
        Ok(Self { start, end })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PositionError {
    InvalidUtf8,
    LineOutOfRange,
    CharacterOutOfRange,
    ContinuationByte,
    Midpoint,
    RangeReversed,
    ByteOutOfRange,
}

impl fmt::Display for PositionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidUtf8 => "source bytes가 UTF-8이 아닙니다",
            Self::LineOutOfRange => "line이 문서 범위를 벗어났습니다",
            Self::CharacterOutOfRange => "character가 line 범위를 벗어났습니다",
            Self::ContinuationByte => "UTF-8 continuation byte 내부입니다",
            Self::Midpoint => "UTF-16/UTF-32 좌표가 scalar 중간입니다",
            Self::RangeReversed => "range start가 end보다 큽니다",
            Self::ByteOutOfRange => "byte offset이 문서 범위를 벗어났습니다",
        };
        f.write_str(message)
    }
}

impl std::error::Error for PositionError {}

pub fn position_to_byte_offset(
    source: &str,
    position: LspPosition,
    encoding: PositionEncoding,
) -> Result<usize, PositionError> {
    let lines = line_spans(source);
    let line = lines
        .get(position.line as usize)
        .ok_or(PositionError::LineOutOfRange)?;
    let mut units = 0u32;
    for (offset, character) in source[line.start..line.content_end].char_indices() {
        if units == position.character {
            return Ok(line.start + offset);
        }
        let next = units.saturating_add(character_units(character, encoding));
        if position.character < next {
            return Err(if encoding == PositionEncoding::Utf8 {
                PositionError::ContinuationByte
            } else {
                PositionError::Midpoint
            });
        }
        units = next;
    }
    if units == position.character {
        Ok(line.content_end)
    } else {
        Err(PositionError::CharacterOutOfRange)
    }
}

pub fn byte_offset_to_position(
    source: &str,
    byte_offset: usize,
    encoding: PositionEncoding,
) -> Result<LspPosition, PositionError> {
    if byte_offset > source.len() {
        return Err(PositionError::ByteOutOfRange);
    }
    if !source.is_char_boundary(byte_offset) {
        return Err(PositionError::ContinuationByte);
    }
    for (line_number, line) in line_spans(source).iter().enumerate() {
        if byte_offset >= line.start && byte_offset <= line.content_end {
            let prefix = &source[line.start..byte_offset];
            let character = prefix
                .chars()
                .map(|value| character_units(value, encoding))
                .sum();
            return Ok(LspPosition {
                line: line_number as u32,
                character,
            });
        }
        if byte_offset > line.content_end && byte_offset < line.next_start {
            return Err(PositionError::Midpoint);
        }
    }
    Err(PositionError::ByteOutOfRange)
}

pub fn range_to_byte_range(
    source: &str,
    range: LspRange,
    encoding: PositionEncoding,
) -> Result<ByteRange, PositionError> {
    let start = position_to_byte_offset(source, range.start, encoding)?;
    let end = position_to_byte_offset(source, range.end, encoding)?;
    ByteRange::new(start, end)
}

pub fn byte_range_to_lsp_range(
    source: &str,
    range: ByteRange,
    encoding: PositionEncoding,
) -> Result<LspRange, PositionError> {
    let start = byte_offset_to_position(source, range.start, encoding)?;
    let end = byte_offset_to_position(source, range.end, encoding)?;
    if start.line > end.line || (start.line == end.line && start.character > end.character) {
        return Err(PositionError::RangeReversed);
    }
    Ok(LspRange { start, end })
}

#[derive(Debug, Clone, Copy)]
struct LineSpan {
    start: usize,
    content_end: usize,
    next_start: usize,
}

fn line_spans(source: &str) -> Vec<LineSpan> {
    let bytes = source.as_bytes();
    let mut spans = Vec::new();
    let mut start = 0;
    for (index, byte) in bytes.iter().enumerate() {
        if *byte == b'\n' {
            let content_end = if index > start && bytes[index - 1] == b'\r' {
                index - 1
            } else {
                index
            };
            spans.push(LineSpan {
                start,
                content_end,
                next_start: index + 1,
            });
            start = index + 1;
        }
    }
    spans.push(LineSpan {
        start,
        content_end: bytes.len(),
        next_start: bytes.len(),
    });
    spans
}

fn character_units(character: char, encoding: PositionEncoding) -> u32 {
    match encoding {
        PositionEncoding::Utf8 => character.len_utf8() as u32,
        PositionEncoding::Utf16 => character.len_utf16() as u32,
        PositionEncoding::Utf32 => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_encodings_round_trip_hangul_and_non_bmp() {
        let source = "A한🙂\r\n끝\n";
        let byte = source.find('🙂').unwrap();
        for encoding in [
            PositionEncoding::Utf8,
            PositionEncoding::Utf16,
            PositionEncoding::Utf32,
        ] {
            let position = byte_offset_to_position(source, byte, encoding).unwrap();
            assert_eq!(
                position_to_byte_offset(source, position, encoding),
                Ok(byte)
            );
        }
        assert_eq!(
            byte_offset_to_position(source, source.find('끝').unwrap(), PositionEncoding::Utf16),
            Ok(LspPosition {
                line: 1,
                character: 0
            })
        );
    }

    #[test]
    fn invalid_midpoints_are_rejected() {
        let source = "🙂";
        assert_eq!(
            position_to_byte_offset(
                source,
                LspPosition {
                    line: 0,
                    character: 1,
                },
                PositionEncoding::Utf16,
            ),
            Err(PositionError::Midpoint)
        );
        assert_eq!(
            position_to_byte_offset(
                source,
                LspPosition {
                    line: 0,
                    character: 1,
                },
                PositionEncoding::Utf8,
            ),
            Err(PositionError::ContinuationByte)
        );
    }
}
