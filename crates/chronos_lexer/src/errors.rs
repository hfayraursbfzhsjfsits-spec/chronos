use crate::token::Span;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum LexerError {
    #[error("[E0001] Unexpected character '{ch}' at line {line}, column {col}")]
    UnexpectedCharacter {
        ch: char,
        line: usize,
        col: usize,
        span: Span,
    },

    #[error("[E0002] Unterminated string literal at line {line}, column {col}")]
    UnterminatedString {
        line: usize,
        col: usize,
        span: Span,
    },

    #[error("[E0003] Unterminated character literal at line {line}, column {col}")]
    UnterminatedChar {
        line: usize,
        col: usize,
        span: Span,
    },

    #[error("[E0004] Invalid escape sequence '\\{ch}' at line {line}, column {col}")]
    InvalidEscapeSequence {
        ch: char,
        line: usize,
        col: usize,
        span: Span,
    },

    #[error("[E0005] Invalid number literal '{value}' at line {line}, column {col}")]
    InvalidNumberLiteral {
        value: String,
        line: usize,
        col: usize,
        span: Span,
    },

    #[error("[E0006] Invalid integer suffix '{suffix}' at line {line}, column {col}")]
    InvalidIntSuffix {
        suffix: String,
        line: usize,
        col: usize,
        span: Span,
    },

    #[error("[E0007] Empty character literal at line {line}, column {col}")]
    EmptyCharLiteral {
        line: usize,
        col: usize,
        span: Span,
    },

    #[error("[E0008] Multi-character literal at line {line}, column {col} — use String type instead")]
    MultiCharLiteral {
        line: usize,
        col: usize,
        span: Span,
    },
}

impl LexerError {
    pub fn span(&self) -> &Span {
        match self {
            LexerError::UnexpectedCharacter { span, .. } => span,
            LexerError::UnterminatedString { span, .. } => span,
            LexerError::UnterminatedChar { span, .. } => span,
            LexerError::InvalidEscapeSequence { span, .. } => span,
            LexerError::InvalidNumberLiteral { span, .. } => span,
            LexerError::InvalidIntSuffix { span, .. } => span,
            LexerError::EmptyCharLiteral { span, .. } => span,
            LexerError::MultiCharLiteral { span, .. } => span,
        }
    }
}
