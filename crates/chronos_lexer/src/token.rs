use std::fmt;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Span — kaynak koddaki pozisyon bilgisi
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self { start, end, line, column }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Literal Suffix Tipleri
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntSuffix {
    I8, I16, I32, I64,
    U8, U16, U32, U64,
}

impl fmt::Display for IntSuffix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntSuffix::I8  => write!(f, "i8"),
            IntSuffix::I16 => write!(f, "i16"),
            IntSuffix::I32 => write!(f, "i32"),
            IntSuffix::I64 => write!(f, "i64"),
            IntSuffix::U8  => write!(f, "u8"),
            IntSuffix::U16 => write!(f, "u16"),
            IntSuffix::U32 => write!(f, "u32"),
            IntSuffix::U64 => write!(f, "u64"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatSuffix {
    F32, F64,
}

impl fmt::Display for FloatSuffix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FloatSuffix::F32 => write!(f, "f32"),
            FloatSuffix::F64 => write!(f, "f64"),
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Token
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub lexeme: String,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, lexeme: String) -> Self {
        Self { kind, span, lexeme }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ─── Language Keywords ───────────────────────
    Contract,
    Fn,
    Let,
    Mut,
    Return,
    If,
    While,
    Else,
    Match,
    Case,
    Default,
    Guard,
    Field,
    Throw,
    Propagate,
    Break,
    Continue,
    Enumeration,
    Variant,
    SelfValue,      // self
    SelfType,       // Self
    As,

    // ─── Built-in Type Keywords ──────────────────
    TyInt8,    TyInt16,   TyInt32,   TyInt64,
    TyUInt8,   TyUInt16,  TyUInt32,  TyUInt64,
    TyFloat32, TyFloat64,
    TyBool,    TyChar,    TyString,  TyVoid,

    // ─── Literals ────────────────────────────────
    LitInteger { value: i128, suffix: Option<IntSuffix> },
    LitFloat   { value: f64,  suffix: Option<FloatSuffix> },
    LitString(String),
    LitChar(char),

    // ─── Identifier ─────────────────────────────
    Ident(String),

    // ─── Arithmetic Operators ───────────────────
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Percent,    // %

    // ─── Comparison Operators ───────────────────
    Eq,         // =
    EqEq,       // ==
    NotEq,      // !=
    Lt,         // <
    LtEq,       // <=
    Gt,         // >
    GtEq,       // >=

    // ─── Logical / Bitwise ──────────────────────
    Bang,       // !
    Amp,        // &
    AmpAmp,     // &&
    Pipe,       // |
    PipePipe,   // ||

    // ─── Delimiters ─────────────────────────────
    LParen,     // (
    RParen,     // )
    LBrace,     // {
    RBrace,     // }
    LBracket,   // [
    RBracket,   // ]

    // ─── Punctuation ────────────────────────────
    Semicolon,  // ;
    Colon,      // :
    ColonColon, // ::
    Comma,      // ,
    Dot,        // .
    DotDot,     // ..
    Arrow,      // ->
    FatArrow,   // =>
    At,         // @
    HashBang,   // #!
    Underscore, // _

    // ─── Special ────────────────────────────────
    Comment(String),
    EOF,
}

impl TokenKind {
    pub fn is_keyword(&self) -> bool {
        matches!(self,
            TokenKind::Contract | TokenKind::Fn | TokenKind::Let |
            TokenKind::Mut | TokenKind::Return | TokenKind::If |
            TokenKind::While | TokenKind::Else | TokenKind::Match | TokenKind::Case |
            TokenKind::Default | TokenKind::Guard | TokenKind::Field |
            TokenKind::Throw | TokenKind::Propagate | TokenKind::Break |
            TokenKind::Continue | TokenKind::Enumeration | TokenKind::Variant |
            TokenKind::SelfValue | TokenKind::SelfType | TokenKind::As
        )
    }

    pub fn is_type_keyword(&self) -> bool {
        matches!(self,
            TokenKind::TyInt8  | TokenKind::TyInt16  | TokenKind::TyInt32  | TokenKind::TyInt64  |
            TokenKind::TyUInt8 | TokenKind::TyUInt16 | TokenKind::TyUInt32 | TokenKind::TyUInt64 |
            TokenKind::TyFloat32 | TokenKind::TyFloat64 |
            TokenKind::TyBool | TokenKind::TyChar | TokenKind::TyString | TokenKind::TyVoid
        )
    }

    pub fn is_literal(&self) -> bool {
        matches!(self,
            TokenKind::LitInteger { .. } | TokenKind::LitFloat { .. } |
            TokenKind::LitString(_) | TokenKind::LitChar(_)
        )
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub fn lookup_keyword(ident: &str) -> Option<TokenKind> {
    match ident {
        // Language keywords
        "contract"    => Some(TokenKind::Contract),
        "fn"          => Some(TokenKind::Fn),
        "let"         => Some(TokenKind::Let),
        "mut"         => Some(TokenKind::Mut),
        "return"      => Some(TokenKind::Return),
        "if"          => Some(TokenKind::If),
        "while"       => Some(TokenKind::While),
        "else"        => Some(TokenKind::Else),
        "match"       => Some(TokenKind::Match),
        "case"        => Some(TokenKind::Case),
        "default"     => Some(TokenKind::Default),
        "guard"       => Some(TokenKind::Guard),
        "field"       => Some(TokenKind::Field),
        "throw"       => Some(TokenKind::Throw),
        "propagate"   => Some(TokenKind::Propagate),
        "break"       => Some(TokenKind::Break),
        "continue"    => Some(TokenKind::Continue),
        "enumeration" => Some(TokenKind::Enumeration),
        "variant"     => Some(TokenKind::Variant),
        "self"        => Some(TokenKind::SelfValue),
        "Self"        => Some(TokenKind::SelfType),
        "as"          => Some(TokenKind::As),

        // Built-in type keywords
        "Int8"    => Some(TokenKind::TyInt8),
        "Int16"   => Some(TokenKind::TyInt16),
        "Int32"   => Some(TokenKind::TyInt32),
        "Int64"   => Some(TokenKind::TyInt64),
        "UInt8"   => Some(TokenKind::TyUInt8),
        "UInt16"  => Some(TokenKind::TyUInt16),
        "UInt32"  => Some(TokenKind::TyUInt32),
        "UInt64"  => Some(TokenKind::TyUInt64),
        "Float32" => Some(TokenKind::TyFloat32),
        "Float64" => Some(TokenKind::TyFloat64),
        "Bool"    => Some(TokenKind::TyBool),
        "Char"    => Some(TokenKind::TyChar),
        "String"  => Some(TokenKind::TyString),
        "Void"    => Some(TokenKind::TyVoid),

        _ => None,
    }
}
