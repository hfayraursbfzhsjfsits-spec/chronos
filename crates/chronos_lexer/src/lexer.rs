use crate::token::*;
use crate::errors::LexerError;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Lexer Struct
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub struct Lexer {
    source: Vec<char>,
    tokens: Vec<Token>,
    errors: Vec<LexerError>,
    current: usize,
    start: usize,
    line: usize,
    column: usize,
    start_line: usize,
    start_column: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            tokens: Vec::new(),
            errors: Vec::new(),
            current: 0,
            start: 0,
            line: 1,
            column: 1,
            start_line: 1,
            start_column: 1,
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, Vec<LexerError>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.start_line = self.line;
            self.start_column = self.column;
            self.scan_token();
        }

        self.tokens.push(Token::new(
            TokenKind::EOF,
            Span::new(self.current, self.current, self.line, self.column),
            String::new(),
        ));

        if self.errors.is_empty() {
            Ok(self.tokens)
        } else {
            Err(self.errors)
        }
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Ana Tarama Fonksiyonu
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn scan_token(&mut self) {
        let ch = self.advance();

        match ch {
            // ── Whitespace ──
            ' ' | '\t' | '\r' => {}
            '\n' => {
                self.line += 1;
                self.column = 1;
            }

            // ── Single-char tokens ──
            '(' => self.add_token(TokenKind::LParen),
            ')' => self.add_token(TokenKind::RParen),
            '{' => self.add_token(TokenKind::LBrace),
            '}' => self.add_token(TokenKind::RBrace),
            '[' => self.add_token(TokenKind::LBracket),
            ']' => self.add_token(TokenKind::RBracket),
            ';' => self.add_token(TokenKind::Semicolon),
            ',' => self.add_token(TokenKind::Comma),
            '%' => self.add_token(TokenKind::Percent),
            '+' => self.add_token(TokenKind::Plus),
            '*' => self.add_token(TokenKind::Star),

            // ── @ ──
            '@' => self.add_token(TokenKind::At),

            // ── Colon or ColonColon ──
            ':' => {
                if self.match_char(':') {
                    self.add_token(TokenKind::ColonColon);
                } else {
                    self.add_token(TokenKind::Colon);
                }
            }

            // ── Dot or DotDot ──
            '.' => {
                if self.match_char('.') {
                    self.add_token(TokenKind::DotDot);
                } else {
                    self.add_token(TokenKind::Dot);
                }
            }

            // ── Minus or Arrow ──
            '-' => {
                if self.match_char('>') {
                    self.add_token(TokenKind::Arrow);
                } else {
                    self.add_token(TokenKind::Minus);
                }
            }

            // ── Equal, FatArrow, EqEq ──
            '=' => {
                if self.match_char('>') {
                    self.add_token(TokenKind::FatArrow);
                } else if self.match_char('=') {
                    self.add_token(TokenKind::EqEq);
                } else {
                    self.add_token(TokenKind::Eq);
                }
            }

            // ── Bang or NotEq ──
            '!' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::NotEq);
                } else {
                    self.add_token(TokenKind::Bang);
                }
            }

            // ── Less, LessEq ──
            '<' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::LtEq);
                } else {
                    self.add_token(TokenKind::Lt);
                }
            }

            // ── Greater, GreaterEq ──
            '>' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::GtEq);
                } else {
                    self.add_token(TokenKind::Gt);
                }
            }

            // ── Amp or AmpAmp ──
            '&' => {
                if self.match_char('&') {
                    self.add_token(TokenKind::AmpAmp);
                } else {
                    self.add_token(TokenKind::Amp);
                }
            }

            // ── Pipe or PipePipe ──
            '|' => {
                if self.match_char('|') {
                    self.add_token(TokenKind::PipePipe);
                } else {
                    self.add_token(TokenKind::Pipe);
                }
            }

            // ── Slash or Comment ──
            '/' => {
                if self.match_char('/') {
                    self.scan_comment();
                } else {
                    self.add_token(TokenKind::Slash);
                }
            }

            // ── Hash → #! ──
            '#' => {
                if self.match_char('!') {
                    self.add_token(TokenKind::HashBang);
                } else {
                    self.errors.push(LexerError::UnexpectedCharacter {
                        ch: '#',
                        line: self.start_line,
                        col: self.start_column,
                        span: self.current_span(),
                    });
                }
            }

            // ── String literal ──
            '"' => self.scan_string(),

            // ── Char literal ──
            '\'' => self.scan_char(),

            // ── Number literal ──
            '0'..='9' => self.scan_number(),

            // ── Identifier / keyword ──
            'a'..='z' | 'A'..='Z' | '_' => self.scan_identifier(),

            // ── Bilinmeyen karakter ──
            _ => {
                self.errors.push(LexerError::UnexpectedCharacter {
                    ch,
                    line: self.start_line,
                    col: self.start_column,
                    span: self.current_span(),
                });
            }
        }
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Comment Tarama
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn scan_comment(&mut self) {
        let start = self.current;
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
        let text: String = self.source[start..self.current].iter().collect();
        self.add_token(TokenKind::Comment(text.trim().to_string()));
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  String Literal Tarama
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn scan_string(&mut self) {
        let mut value = String::new();

        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                self.errors.push(LexerError::UnterminatedString {
                    line: self.start_line,
                    col: self.start_column,
                    span: self.current_span(),
                });
                return;
            }

            if self.peek() == '\\' {
                self.advance();
                if self.is_at_end() {
                    self.errors.push(LexerError::UnterminatedString {
                        line: self.start_line,
                        col: self.start_column,
                        span: self.current_span(),
                    });
                    return;
                }
                let escape = self.advance();
                match escape {
                    'n'  => value.push('\n'),
                    't'  => value.push('\t'),
                    'r'  => value.push('\r'),
                    '\\' => value.push('\\'),
                    '"'  => value.push('"'),
                    '0'  => value.push('\0'),
                    _ => {
                        self.errors.push(LexerError::InvalidEscapeSequence {
                            ch: escape,
                            line: self.line,
                            col: self.column - 1,
                            span: self.current_span(),
                        });
                        return;
                    }
                }
            } else {
                value.push(self.advance());
            }
        }

        if self.is_at_end() {
            self.errors.push(LexerError::UnterminatedString {
                line: self.start_line,
                col: self.start_column,
                span: self.current_span(),
            });
            return;
        }

        self.advance(); // kapanış "
        self.add_token(TokenKind::LitString(value));
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Char Literal Tarama
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn scan_char(&mut self) {
        if self.is_at_end() || self.peek() == '\'' {
            if !self.is_at_end() { self.advance(); }
            self.errors.push(LexerError::EmptyCharLiteral {
                line: self.start_line,
                col: self.start_column,
                span: self.current_span(),
            });
            return;
        }

        let ch;
        if self.peek() == '\\' {
            self.advance();
            if self.is_at_end() {
                self.errors.push(LexerError::UnterminatedChar {
                    line: self.start_line,
                    col: self.start_column,
                    span: self.current_span(),
                });
                return;
            }
            let escape = self.advance();
            ch = match escape {
                'n'  => '\n',
                't'  => '\t',
                'r'  => '\r',
                '\\' => '\\',
                '\'' => '\'',
                '0'  => '\0',
                _ => {
                    self.errors.push(LexerError::InvalidEscapeSequence {
                        ch: escape,
                        line: self.line,
                        col: self.column - 1,
                        span: self.current_span(),
                    });
                    return;
                }
            };
        } else {
            ch = self.advance();
        }

        if !self.is_at_end() && self.peek() != '\'' {
            while !self.is_at_end() && self.peek() != '\'' {
                self.advance();
            }
            if !self.is_at_end() { self.advance(); }
            self.errors.push(LexerError::MultiCharLiteral {
                line: self.start_line,
                col: self.start_column,
                span: self.current_span(),
            });
            return;
        }

        if self.is_at_end() {
            self.errors.push(LexerError::UnterminatedChar {
                line: self.start_line,
                col: self.start_column,
                span: self.current_span(),
            });
            return;
        }

        self.advance(); // kapanış '
        self.add_token(TokenKind::LitChar(ch));
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Number Literal Tarama
    //  Desteklenenler: 42, 42i32, 0xFF, 100_000u64,
    //                  3.14, 3.14f32
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn scan_number(&mut self) {
        let start_pos = self.current - 1;

        // ── Hex: 0x... ──
        if self.source[start_pos] == '0'
            && !self.is_at_end()
            && (self.peek() == 'x' || self.peek() == 'X')
        {
            self.advance(); // 'x' consume
            let hex_start = self.current;

            while !self.is_at_end() && (self.peek().is_ascii_hexdigit() || self.peek() == '_') {
                self.advance();
            }

            let hex_str: String = self.source[hex_start..self.current]
                .iter()
                .filter(|c| **c != '_')
                .collect();

            match u64::from_str_radix(&hex_str, 16) {
                Ok(val) => {
                    self.add_token(TokenKind::LitInteger {
                        value: val as i128,
                        suffix: None,
                    });
                }
                Err(_) => {
                    let raw: String = self.source[start_pos..self.current].iter().collect();
                    self.errors.push(LexerError::InvalidNumberLiteral {
                        value: raw,
                        line: self.start_line,
                        col: self.start_column,
                        span: self.current_span(),
                    });
                }
            }
            return;
        }

        // ── Regular digits ──
        while !self.is_at_end() && (self.peek().is_ascii_digit() || self.peek() == '_') {
            self.advance();
        }

        let mut is_float = false;
        if !self.is_at_end()
            && self.peek() == '.'
            && self.peek_next().map_or(false, |c| c.is_ascii_digit())
        {
            is_float = true;
            self.advance(); // '.' consume
            while !self.is_at_end() && (self.peek().is_ascii_digit() || self.peek() == '_') {
                self.advance();
            }
        }

        let num_end = self.current;
        let num_str: String = self.source[start_pos..num_end]
            .iter()
            .filter(|c| **c != '_')
            .collect();

        if !self.is_at_end() && (self.peek() == 'i' || self.peek() == 'u' || self.peek() == 'f') {
            let suffix_start = self.current;
            self.advance();
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                self.advance();
            }
            let suffix_str: String = self.source[suffix_start..self.current].iter().collect();

            if is_float || suffix_str.starts_with('f') {
                let float_suffix = match suffix_str.as_str() {
                    "f32" => Some(FloatSuffix::F32),
                    "f64" => Some(FloatSuffix::F64),
                    _ => {
                        self.errors.push(LexerError::InvalidIntSuffix {
                            suffix: suffix_str,
                            line: self.start_line,
                            col: self.start_column,
                            span: self.current_span(),
                        });
                        return;
                    }
                };
                match num_str.parse::<f64>() {
                    Ok(val) => self.add_token(TokenKind::LitFloat {
                        value: val,
                        suffix: float_suffix,
                    }),
                    Err(_) => {
                        self.errors.push(LexerError::InvalidNumberLiteral {
                            value: num_str,
                            line: self.start_line,
                            col: self.start_column,
                            span: self.current_span(),
                        });
                    }
                }
            } else {
                let int_suffix = match suffix_str.as_str() {
                    "i8"  => Some(IntSuffix::I8),
                    "i16" => Some(IntSuffix::I16),
                    "i32" => Some(IntSuffix::I32),
                    "i64" => Some(IntSuffix::I64),
                    "u8"  => Some(IntSuffix::U8),
                    "u16" => Some(IntSuffix::U16),
                    "u32" => Some(IntSuffix::U32),
                    "u64" => Some(IntSuffix::U64),
                    _ => {
                        self.errors.push(LexerError::InvalidIntSuffix {
                            suffix: suffix_str,
                            line: self.start_line,
                            col: self.start_column,
                            span: self.current_span(),
                        });
                        return;
                    }
                };
                match num_str.parse::<i128>() {
                    Ok(val) => self.add_token(TokenKind::LitInteger {
                        value: val,
                        suffix: int_suffix,
                    }),
                    Err(_) => {
                        self.errors.push(LexerError::InvalidNumberLiteral {
                            value: num_str,
                            line: self.start_line,
                            col: self.start_column,
                            span: self.current_span(),
                        });
                    }
                }
            }
        } else if is_float {
            match num_str.parse::<f64>() {
                Ok(val) => self.add_token(TokenKind::LitFloat {
                    value: val,
                    suffix: None,
                }),
                Err(_) => {
                    self.errors.push(LexerError::InvalidNumberLiteral {
                        value: num_str,
                        line: self.start_line,
                        col: self.start_column,
                        span: self.current_span(),
                    });
                }
            }
        } else {
            match num_str.parse::<i128>() {
                Ok(val) => self.add_token(TokenKind::LitInteger {
                    value: val,
                    suffix: None,
                }),
                Err(_) => {
                    self.errors.push(LexerError::InvalidNumberLiteral {
                        value: num_str,
                        line: self.start_line,
                        col: self.start_column,
                        span: self.current_span(),
                    });
                }
            }
        }
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Identifier / Keyword Tarama
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn scan_identifier(&mut self) {
        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            self.advance();
        }

        let text: String = self.source[self.start..self.current].iter().collect();

        let kind = if text == "_" {
            TokenKind::Underscore
        } else if let Some(keyword) = lookup_keyword(&text) {
            keyword
        } else {
            TokenKind::Ident(text)
        };

        self.add_token(kind);
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn advance(&mut self) -> char {
        let ch = self.source[self.current];
        self.current += 1;
        self.column += 1;
        ch
    }

    fn peek(&self) -> char {
        if self.is_at_end() { '\0' } else { self.source[self.current] }
    }

    fn peek_next(&self) -> Option<char> {
        if self.current + 1 >= self.source.len() {
            None
        } else {
            Some(self.source[self.current + 1])
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            false
        } else {
            self.current += 1;
            self.column += 1;
            true
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn current_span(&self) -> Span {
        Span::new(self.start, self.current, self.start_line, self.start_column)
    }

    fn add_token(&mut self, kind: TokenKind) {
        let lexeme: String = self.source[self.start..self.current].iter().collect();
        let span = self.current_span();
        self.tokens.push(Token::new(kind, span, lexeme));
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Unit Tests
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(input: &str) -> Vec<Token> {
        Lexer::new(input).tokenize().expect("Lexer should not fail")
    }

    fn kinds(input: &str) -> Vec<TokenKind> {
        lex(input).into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn test_empty() {
        assert_eq!(kinds(""), vec![TokenKind::EOF]);
    }

    #[test]
    fn test_keywords() {
        assert_eq!(
            kinds("contract fn let mut return"),
            vec![
                TokenKind::Contract,
                TokenKind::Fn,
                TokenKind::Let,
                TokenKind::Mut,
                TokenKind::Return,
                TokenKind::EOF,
            ]
        );
    }

    #[test]
    fn test_type_keywords() {
        assert_eq!(
            kinds("Int32 Float64 Bool String Void"),
            vec![
                TokenKind::TyInt32,
                TokenKind::TyFloat64,
                TokenKind::TyBool,
                TokenKind::TyString,
                TokenKind::TyVoid,
                TokenKind::EOF,
            ]
        );
    }

    #[test]
    fn test_integer_with_suffix() {
        assert_eq!(
            kinds("42i32"),
            vec![
                TokenKind::LitInteger { value: 42, suffix: Some(IntSuffix::I32) },
                TokenKind::EOF,
            ]
        );
    }

    #[test]
    fn test_integer_with_underscores() {
        assert_eq!(
            kinds("100_000u64"),
            vec![
                TokenKind::LitInteger { value: 100_000, suffix: Some(IntSuffix::U64) },
                TokenKind::EOF,
            ]
        );
    }

    #[test]
    fn test_hex_literal() {
        assert_eq!(
            kinds("0xFF"),
            vec![
                TokenKind::LitInteger { value: 255, suffix: None },
                TokenKind::EOF,
            ]
        );
    }

    #[test]
    fn test_float_with_suffix() {
        assert_eq!(
            kinds("3.14f32"),
            vec![
                TokenKind::LitFloat { value: 3.14, suffix: Some(FloatSuffix::F32) },
                TokenKind::EOF,
            ]
        );
    }

    #[test]
    fn test_string_literal() {
        assert_eq!(
            kinds("\"hello world\""),
            vec![
                TokenKind::LitString("hello world".to_string()),
                TokenKind::EOF,
            ]
        );
    }

    #[test]
    fn test_string_escape() {
        assert_eq!(
            kinds("\"line1\\nline2\""),
            vec![
                TokenKind::LitString("line1\nline2".to_string()),
                TokenKind::EOF,
            ]
        );
    }

    #[test]
    fn test_char_literal() {
        assert_eq!(
            kinds("'A'"),
            vec![
                TokenKind::LitChar('A'),
                TokenKind::EOF,
            ]
        );
    }

    #[test]
    fn test_operators() {
        assert_eq!(
            kinds(":: -> => == !="),
            vec![
                TokenKind::ColonColon,
                TokenKind::Arrow,
                TokenKind::FatArrow,
                TokenKind::EqEq,
                TokenKind::NotEq,
                TokenKind::EOF,
            ]
        );
    }

    #[test]
    fn test_module_directive() {
        assert_eq!(
            kinds("#![module::entry(main)]"),
            vec![
                TokenKind::HashBang,
                TokenKind::LBracket,
                TokenKind::Ident("module".to_string()),
                TokenKind::ColonColon,
                TokenKind::Ident("entry".to_string()),
                TokenKind::LParen,
                TokenKind::Ident("main".to_string()),
                TokenKind::RParen,
                TokenKind::RBracket,
                TokenKind::EOF,
            ]
        );
    }

    #[test]
    fn test_annotation() {
        assert_eq!(
            kinds("@require"),
            vec![
                TokenKind::At,
                TokenKind::Ident("require".to_string()),
                TokenKind::EOF,
            ]
        );
    }

    #[test]
    fn test_let_statement() {
        assert_eq!(
            kinds("let x: Int32 = 42i32;"),
            vec![
                TokenKind::Let,
                TokenKind::Ident("x".to_string()),
                TokenKind::Colon,
                TokenKind::TyInt32,
                TokenKind::Eq,
                TokenKind::LitInteger { value: 42, suffix: Some(IntSuffix::I32) },
                TokenKind::Semicolon,
                TokenKind::EOF,
            ]
        );
    }

    #[test]
    fn test_contract_declaration() {
        assert_eq!(
            kinds("contract Main :: EntryPoint {"),
            vec![
                TokenKind::Contract,
                TokenKind::Ident("Main".to_string()),
                TokenKind::ColonColon,
                TokenKind::Ident("EntryPoint".to_string()),
                TokenKind::LBrace,
                TokenKind::EOF,
            ]
        );
    }

    #[test]
    fn test_comment_skipped_as_token() {
        let tokens = lex("let x // this is a comment\n");
        let non_comment: Vec<&TokenKind> = tokens
            .iter()
            .map(|t| &t.kind)
            .filter(|k| !matches!(k, TokenKind::Comment(_) | TokenKind::EOF))
            .collect();
        assert_eq!(non_comment.len(), 2); // Let, Ident("x")
    }
}
