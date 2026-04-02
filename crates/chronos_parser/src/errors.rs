use chronos_lexer::Span;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("[P0001] Expected {expected}, found '{found}' at {span}")]
    ExpectedToken {
        expected: String,
        found: String,
        span: Span,
    },

    #[error("[P0002] Unexpected token '{token}' at {span}")]
    UnexpectedToken {
        token: String,
        span: Span,
    },

    #[error("[P0003] Expected identifier, found '{found}' at {span}")]
    ExpectedIdentifier {
        found: String,
        span: Span,
    },

    #[error("[P0004] Expected type expression at {span}")]
    ExpectedType {
        span: Span,
    },

    #[error("[P0005] Expected expression at {span}")]
    ExpectedExpression {
        span: Span,
    },

    #[error("[P0006] Expected block '{{' at {span}")]
    ExpectedBlock {
        span: Span,
    },

    #[error("[P0007] Unexpected end of file")]
    UnexpectedEOF,

    #[error("[P0008] Invalid annotation at {span}")]
    InvalidAnnotation {
        span: Span,
    },

    #[error("[P0009] Expected semicolon ';' at {span}")]
    ExpectedSemicolon {
        span: Span,
    },
}
