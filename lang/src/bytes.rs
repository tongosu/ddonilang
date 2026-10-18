//! RD-656 exact frontdoor binding. Runtime consumes the admitted operation,
//! never a guessed spelling or an independently resolved display name.
use crate::{ArgBinding, ExprKind, Literal, ParseError, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BytesConversion {
    EncodeUtf8Exact,
    DecodeUtf8Nfc,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_binding_keeps_one_exact_operation() {
        let source = "(글=\"가😀\") UTF8바이트열로 보여주기.";
        let program = crate::parse_v25(source, "한글😀.ddn").unwrap();
        assert_eq!(program.origin.bytes_calls.len(), 1);
        assert_eq!(
            program
                .origin
                .bytes_calls
                .values()
                .copied()
                .collect::<Vec<_>>(),
            vec![BytesConversion::EncodeUtf8Exact]
        );
    }

    #[test]
    fn bytes_binding_rejects_competing_definition_identity() {
        let source = "UTF8바이트열로 (글:글 --> 글) := 글.\n(글=\"A\") UTF8바이트열로 보여주기.";
        assert!(crate::parse_v25(source, "collision.ddn").is_err());
    }

    #[test]
    fn bytes_binding_does_not_admit_inferred_conjugations() {
        for form in ["UTF8바이트열로기", "UTF8바이트열로하기", "UTF8바이트열로고"]
        {
            let source = format!("(글=\"A\") {form} 보여주기.");
            assert!(crate::parse_v25(&source, "alias.ddn").is_err(), "{form}");
        }
    }

    #[test]
    fn bytes_literal_spans_preserve_utf8_offsets_after_korean_and_emoji() {
        let prefix = "\"한글😀\" 보여주기.\n";
        let literal = "바이트열{0x00, 0xFF}";
        let source = format!("{prefix}{literal} 보여주기.");
        let tokens = crate::Lexer::new(&source).tokenize().unwrap();
        let token = tokens
            .iter()
            .find(|token| matches!(token.kind, crate::TokenKind::BytesLit(_)))
            .unwrap();
        assert_eq!(token.span.start, prefix.len());
        assert_eq!(token.span.end, prefix.len() + literal.len());
        assert_eq!(&source[token.span.start..token.span.end], literal);
        assert_eq!(token.kind, crate::TokenKind::BytesLit(vec![0, 255]));

        let invalid = format!("{prefix}바이트열{{0xGG}} 보여주기.");
        let error = crate::Lexer::new(&invalid).tokenize().unwrap_err();
        assert_eq!(error.pos, prefix.len() + "바이트열{".len());
        assert!(invalid.is_char_boundary(error.pos));
        assert!(error.message.contains("E_BYTES_LITERAL_INVALID"));
    }
}

impl BytesConversion {
    pub fn from_registered_surface(surface: &str) -> Option<Self> {
        match surface {
            "UTF8바이트열로" => Some(Self::EncodeUtf8Exact),
            "UTF8NFC글로" => Some(Self::DecodeUtf8Nfc),
            _ => None,
        }
    }

    pub fn required_pin(self) -> &'static str {
        match self {
            Self::EncodeUtf8Exact => "글",
            Self::DecodeUtf8Nfc => "바이트열",
        }
    }

    pub fn validate(self, args: &[ArgBinding], span: Span) -> Result<(), ParseError> {
        let fail = |message: &str| ParseError {
            span,
            message: message.to_string(),
        };
        if args.len() != 1 || args[0].resolved_pin.as_deref() != Some(self.required_pin()) {
            return Err(fail(
                "E_BYTES_ARGUMENT_BINDING: 정확한 필수 핀 하나가 필요합니다",
            ));
        }
        if let ExprKind::Literal(literal) = &args[0].expr.kind {
            let valid = matches!(
                (self, literal),
                (Self::EncodeUtf8Exact, Literal::String(_))
                    | (Self::DecodeUtf8Nfc, Literal::Bytes(_))
            );
            if !valid {
                return Err(fail(
                    "E_TYPE_MISMATCH: UTF-8 변환의 인자 타입이 일치하지 않습니다",
                ));
            }
        }
        Ok(())
    }
}
