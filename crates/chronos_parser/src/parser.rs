use chronos_lexer::{Token, TokenKind, Span};
use crate::ast::*;
use crate::errors::ParseError;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Parser Struct
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(mut self) -> Result<Program, Vec<ParseError>> {
        let mut errors: Vec<ParseError> = Vec::new();
        let mut module_directives: Vec<ModuleDirective> = Vec::new();
        let mut require_statements: Vec<RequireStatement> = Vec::new();
        let mut declarations: Vec<Declaration> = Vec::new();

        while !self.is_at_end() {
            match self.peek_kind() {
                // #![...]
                TokenKind::HashBang => {
                    match self.parse_module_directive() {
                        Ok(dir) => module_directives.push(dir),
                        Err(e) => { errors.push(e); self.synchronize(); }
                    }
                }
                // @require
                TokenKind::At if self.peek_next_kind() == Some("require") => {
                    match self.parse_require_statement() {
                        Ok(req) => require_statements.push(req),
                        Err(e) => { errors.push(e); self.synchronize(); }
                    }
                }
                // @annotation (contract/fn'den önce gelebilir)
                TokenKind::At => {
                    match self.parse_declaration() {
                        Ok(decl) => declarations.push(decl),
                        Err(e) => { errors.push(e); self.synchronize(); }
                    }
                }
                // contract
                TokenKind::Contract => {
                    match self.parse_declaration() {
                        Ok(decl) => declarations.push(decl),
                        Err(e) => { errors.push(e); self.synchronize(); }
                    }
                }
                // fn
                TokenKind::Fn => {
                    match self.parse_declaration() {
                        Ok(decl) => declarations.push(decl),
                        Err(e) => { errors.push(e); self.synchronize(); }
                    }
                }
                // enumeration
                TokenKind::Enumeration => {
                    match self.parse_declaration() {
                        Ok(decl) => declarations.push(decl),
                        Err(e) => { errors.push(e); self.synchronize(); }
                    }
                }
                // EOF
                TokenKind::EOF => break,
                // Comment — atla
                TokenKind::Comment(_) => { self.advance(); }
                // Bilinmeyen
                _ => {
                    let tok = self.advance();
                    errors.push(ParseError::UnexpectedToken {
                        token: tok.lexeme.clone(),
                        span: tok.span,
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(Program { module_directives, require_statements, declarations })
        } else {
            Err(errors)
        }
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Module Directive: #![module::entry(main)]
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn parse_module_directive(&mut self) -> Result<ModuleDirective, ParseError> {
        let start_span = self.current_span();
        self.expect(TokenKind::HashBang)?;
        self.expect(TokenKind::LBracket)?;

        let mut path: Vec<String> = Vec::new();
        path.push(self.expect_identifier()?);

        while self.check(&TokenKind::ColonColon) {
            self.advance();
            path.push(self.expect_identifier()?);
        }

        self.expect(TokenKind::LParen)?;

        let value = if let Some(s) = self.try_string_literal() {
            s
        } else {
            self.expect_identifier()?
        };

        self.expect(TokenKind::RParen)?;
        self.expect(TokenKind::RBracket)?;

        Ok(ModuleDirective { path, value, span: start_span })
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Require: @require core::io::{ X, Y };
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn parse_require_statement(&mut self) -> Result<RequireStatement, ParseError> {
        let start_span = self.current_span();
        self.expect(TokenKind::At)?;
        self.expect_ident_matching("require")?;

        let mut module_path: Vec<String> = Vec::new();
        module_path.push(self.expect_any_identifier()?);

        while self.check(&TokenKind::ColonColon) {
            self.advance();
            if self.check(&TokenKind::LBrace) {
                break;
            }
            module_path.push(self.expect_any_identifier()?);
        }

        self.expect(TokenKind::LBrace)?;

        let mut imports: Vec<String> = Vec::new();
        if !self.check(&TokenKind::RBrace) {
            imports.push(self.expect_any_identifier()?);
            while self.check(&TokenKind::Comma) {
                self.advance();
                if self.check(&TokenKind::RBrace) { break; }
                imports.push(self.expect_any_identifier()?);
            }
        }

        self.expect(TokenKind::RBrace)?;
        self.expect(TokenKind::Semicolon)?;

        Ok(RequireStatement { module_path, imports, span: start_span })
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Declaration (contract, fn, enum)
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn parse_declaration(&mut self) -> Result<Declaration, ParseError> {
        let annotations = self.parse_annotations()?;

        match self.peek_kind() {
            TokenKind::Contract => {
                let decl = self.parse_contract()?;
                // İlk annotations'ı contract'ın ilk method'una atamak yerine
                // şimdilik yok sayıyoruz (contract-level annotation)
                if !annotations.is_empty() {
                    // Contract-level annotations — ileride kullanılacak
                }
                let _ = annotations; // suppress warning
                Ok(Declaration::Contract(decl))
            }
            TokenKind::Fn => {
                let decl = self.parse_function(annotations)?;
                Ok(Declaration::Function(decl))
            }
            TokenKind::Enumeration => {
                let decl = self.parse_enumeration()?;
                Ok(Declaration::Enumeration(decl))
            }
            _ => {
                let tok = self.advance();
                Err(ParseError::UnexpectedToken {
                    token: tok.lexeme.clone(),
                    span: tok.span,
                })
            }
        }
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Annotations: @static, @visibility(public)
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn parse_annotations(&mut self) -> Result<Vec<Annotation>, ParseError> {
        let mut annotations: Vec<Annotation> = Vec::new();

        while self.check(&TokenKind::At) {
            // @require'ı annotation olarak parse etme
            if self.peek_next_kind() == Some("require") {
                break;
            }
            annotations.push(self.parse_single_annotation()?);
        }

        Ok(annotations)
    }

    fn parse_single_annotation(&mut self) -> Result<Annotation, ParseError> {
        let start_span = self.current_span();
        self.expect(TokenKind::At)?;
        let name = self.expect_identifier()?;

        let mut args: Vec<Expr> = Vec::new();
        if self.check(&TokenKind::LParen) {
            self.advance();
            if !self.check(&TokenKind::RParen) {
                args.push(self.parse_expression()?);
                while self.check(&TokenKind::Comma) {
                    self.advance();
                    if self.check(&TokenKind::RParen) { break; }
                    args.push(self.parse_expression()?);
                }
            }
            self.expect(TokenKind::RParen)?;
        }

        Ok(Annotation { name, args, span: start_span })
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Contract: contract Main :: EntryPoint { ... }
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn parse_contract(&mut self) -> Result<ContractDecl, ParseError> {
        let start_span = self.current_span();
        self.expect(TokenKind::Contract)?;
        let name = self.expect_identifier()?;

        let mut traits: Vec<String> = Vec::new();
        if self.check(&TokenKind::ColonColon) {
            self.advance();
            traits.push(self.expect_identifier()?);
            while self.check(&TokenKind::Comma) {
                self.advance();
                traits.push(self.expect_identifier()?);
            }
        }

        self.expect(TokenKind::LBrace)?;

        let mut members: Vec<ContractMember> = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            self.skip_comments();
            if self.check(&TokenKind::RBrace) { break; }

            let annotations = self.parse_annotations()?;

            match self.peek_kind() {
                TokenKind::Field => {
                    let field = self.parse_field(annotations)?;
                    members.push(ContractMember::Field(field));
                }
                TokenKind::Fn => {
                    let method = self.parse_function(annotations)?;
                    members.push(ContractMember::Method(method));
                }
                _ => {
                    let tok = self.advance();
                    return Err(ParseError::UnexpectedToken {
                        token: tok.lexeme.clone(),
                        span: tok.span,
                    });
                }
            }
        }

        self.expect(TokenKind::RBrace)?;

        Ok(ContractDecl { name, traits, members, span: start_span })
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Field: field name: String;
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn parse_field(&mut self, annotations: Vec<Annotation>) -> Result<FieldDecl, ParseError> {
        let start_span = self.current_span();
        self.expect(TokenKind::Field)?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::Colon)?;
        let type_expr = self.parse_type_expr()?;
        self.expect(TokenKind::Semicolon)?;

        Ok(FieldDecl { annotations, name, type_expr, span: start_span })
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Function
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn parse_function(&mut self, annotations: Vec<Annotation>) -> Result<FnDecl, ParseError> {
        let start_span = self.current_span();
        self.expect(TokenKind::Fn)?;
        let name = self.expect_identifier()?;

        self.expect(TokenKind::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(TokenKind::RParen)?;

        self.expect(TokenKind::Arrow)?;
        let return_type = self.parse_type_expr()?;

        let body = self.parse_block()?;

        Ok(FnDecl { annotations, name, params, return_type, body, span: start_span })
    }

    fn parse_param_list(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params: Vec<Param> = Vec::new();

        if self.check(&TokenKind::RParen) {
            return Ok(params);
        }

        // İlk parametre öncesi annotation olabilir (@constrain gibi)
        while self.check(&TokenKind::At) {
            self.parse_single_annotation()?; // şimdilik yok say
        }

        params.push(self.parse_single_param()?);

        while self.check(&TokenKind::Comma) {
            self.advance();
            if self.check(&TokenKind::RParen) { break; }
            while self.check(&TokenKind::At) {
                self.parse_single_annotation()?;
            }
            params.push(self.parse_single_param()?);
        }

        Ok(params)
    }

    fn parse_single_param(&mut self) -> Result<Param, ParseError> {
        let start_span = self.current_span();

        let name = self.expect_any_identifier()?;
        self.expect(TokenKind::Colon)?;
        let type_expr = self.parse_type_expr()?;

        Ok(Param { name, type_expr, span: start_span })
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Enumeration
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn parse_enumeration(&mut self) -> Result<EnumDecl, ParseError> {
        let start_span = self.current_span();
        self.expect(TokenKind::Enumeration)?;
        let name = self.expect_identifier()?;

        let mut traits: Vec<String> = Vec::new();
        if self.check(&TokenKind::ColonColon) {
            self.advance();
            traits.push(self.expect_identifier()?);
            while self.check(&TokenKind::Comma) {
                self.advance();
                traits.push(self.expect_identifier()?);
            }
        }

        self.expect(TokenKind::LBrace)?;

        let mut variants: Vec<EnumVariant> = Vec::new();
        let mut methods: Vec<FnDecl> = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            self.skip_comments();
            if self.check(&TokenKind::RBrace) { break; }

            let annotations = self.parse_annotations()?;

            match self.peek_kind() {
                TokenKind::Variant => {
                    variants.push(self.parse_enum_variant()?);
                }
                TokenKind::Fn => {
                    methods.push(self.parse_function(annotations)?);
                }
                _ => {
                    let tok = self.advance();
                    return Err(ParseError::UnexpectedToken {
                        token: tok.lexeme.clone(),
                        span: tok.span,
                    });
                }
            }
        }

        self.expect(TokenKind::RBrace)?;

        Ok(EnumDecl { name, traits, variants, methods, span: start_span })
    }

    fn parse_enum_variant(&mut self) -> Result<EnumVariant, ParseError> {
        let start_span = self.current_span();
        self.expect(TokenKind::Variant)?;
        let name = self.expect_identifier()?;

        let mut fields: Vec<FieldDecl> = Vec::new();
        if self.check(&TokenKind::LBrace) {
            self.advance();
            while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                self.skip_comments();
                if self.check(&TokenKind::RBrace) { break; }
                let field = self.parse_field(Vec::new())?;
                fields.push(field);
            }
            self.expect(TokenKind::RBrace)?;
        }

        // Virgül opsiyonel (variant'lar arası)
        if self.check(&TokenKind::Comma) { self.advance(); }

        Ok(EnumVariant { name, fields, span: start_span })
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Type Expressions
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn parse_type_expr(&mut self) -> Result<TypeExpr, ParseError> {
        // &T veya &mut T
        if self.check(&TokenKind::Amp) {
            let start_span = self.current_span();
            self.advance();
            let mutable = if self.check(&TokenKind::Mut) {
                self.advance();
                true
            } else {
                false
            };
            let inner = self.parse_type_expr()?;
            return Ok(TypeExpr::Reference {
                mutable,
                inner: Box::new(inner),
                span: start_span,
            });
        }

        // Self
        if self.check(&TokenKind::SelfType) {
            let span = self.current_span();
            self.advance();
            return Ok(TypeExpr::SelfType(span));
        }

        // Named type (possibly generic)
        let span = self.current_span();
        let name = self.expect_any_identifier()?;

        // Generic: Vector<Int32>, Result<String, Error>
        if self.check(&TokenKind::Lt) {
            self.advance();
            let mut type_args: Vec<TypeExpr> = Vec::new();
            type_args.push(self.parse_type_expr()?);
            while self.check(&TokenKind::Comma) {
                self.advance();
                type_args.push(self.parse_type_expr()?);
            }
            self.expect(TokenKind::Gt)?;

            return Ok(TypeExpr::Generic { base: name, type_args, span });
        }

        Ok(TypeExpr::Named(name, span))
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Block: { statement; statement; ... }
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        self.expect(TokenKind::LBrace)?;

        let mut stmts: Vec<Stmt> = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            self.skip_comments();
            if self.check(&TokenKind::RBrace) { break; }
            stmts.push(self.parse_statement()?);
        }

        self.expect(TokenKind::RBrace)?;
        Ok(stmts)
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Statements
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        self.skip_comments();

        match self.peek_kind() {
            TokenKind::Let => self.parse_let_statement(),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::Guard => self.parse_guard_statement(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::Match => self.parse_match_statement(),
            TokenKind::Break => {
                let span = self.current_span();
                self.advance();
                self.expect(TokenKind::Semicolon)?;
                Ok(Stmt::Break { span })
            }
            TokenKind::Continue => {
                let span = self.current_span();
                self.advance();
                self.expect(TokenKind::Semicolon)?;
                Ok(Stmt::Continue { span })
            }
            _ => self.parse_expression_statement(),
        }
    }

    // ── Let Statement ──
    fn parse_let_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect(TokenKind::Let)?;

        let mutable = if self.check(&TokenKind::Mut) {
            self.advance();
            true
        } else {
            false
        };

        let name = self.expect_identifier()?;
        self.expect(TokenKind::Colon)?;
        let type_expr = self.parse_type_expr()?;
        self.expect(TokenKind::Eq)?;
        let initializer = self.parse_expression()?;

        let error_handler = if self.check(&TokenKind::FatArrow) {
            Some(self.parse_error_handler()?)
        } else {
            None
        };

        self.expect(TokenKind::Semicolon)?;

        Ok(Stmt::Let {
            mutable,
            name,
            type_expr,
            initializer,
            error_handler,
            span: start_span,
        })
    }

    // ── Return Statement ──
    fn parse_return_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect(TokenKind::Return)?;

        let value = if !self.check(&TokenKind::Semicolon) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.expect(TokenKind::Semicolon)?;

        Ok(Stmt::Return { value, span: start_span })
    }

    // ── Guard Statement ──
    fn parse_guard_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect(TokenKind::Guard)?;
        let condition = self.parse_expression()?;
        self.expect_ident_matching("else")?;
        let else_block = self.parse_block()?;
        self.expect(TokenKind::Semicolon)?;

        Ok(Stmt::Guard { condition, else_block, span: start_span })
    }

    // ── If Statement ──
    fn parse_if_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect(TokenKind::If)?;

        // if (condition: expr) => { }
        self.expect(TokenKind::LParen)?;
        // "condition:" label opsiyonel
        if self.is_named_arg_ahead() {
            self.advance(); // label ismi
            self.advance(); // ':'
        }
        let condition = self.parse_expression()?;
        self.expect(TokenKind::RParen)?;

        if self.check(&TokenKind::FatArrow) { self.advance(); }
        let then_block = self.parse_block()?;

        let mut else_if_blocks: Vec<(Expr, Block)> = Vec::new();
        let mut else_block: Option<Block> = None;

        while self.check(&TokenKind::Else) {
            self.advance();
            if self.check(&TokenKind::If) {
                self.advance();
                self.expect(TokenKind::LParen)?;
                if self.is_named_arg_ahead() {
                    self.advance();
                    self.advance();
                }
                let cond = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                if self.check(&TokenKind::FatArrow) { self.advance(); }
                let block = self.parse_block()?;
                else_if_blocks.push((cond, block));
            } else {
                if self.check(&TokenKind::FatArrow) { self.advance(); }
                else_block = Some(self.parse_block()?);
                break;
            }
        }

        // Trailing semicolon opsiyonel
        if self.check(&TokenKind::Semicolon) { self.advance(); }

        Ok(Stmt::If {
            condition,
            then_block,
            else_if_blocks,
            else_block,
            span: start_span,
        })
    }

    // ── Match Statement ──
    fn parse_match_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect(TokenKind::Match)?;

        self.expect(TokenKind::LParen)?;
        if self.is_named_arg_ahead() {
            self.advance();
            self.advance();
        }
        let value = self.parse_expression()?;
        self.expect(TokenKind::RParen)?;

        if self.check(&TokenKind::FatArrow) { self.advance(); }

        self.expect(TokenKind::LBrace)?;

        let mut arms: Vec<MatchArm> = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            self.skip_comments();
            if self.check(&TokenKind::RBrace) { break; }
            arms.push(self.parse_match_arm()?);
        }

        self.expect(TokenKind::RBrace)?;
        if self.check(&TokenKind::Semicolon) { self.advance(); }

        Ok(Stmt::Match { value, arms, span: start_span })
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm, ParseError> {
        let start_span = self.current_span();

        let pattern = if self.check(&TokenKind::Default) {
            self.advance();
            MatchPattern::Default
        } else if self.check(&TokenKind::Case) {
            self.advance();
            self.expect(TokenKind::LParen)?;
            let expr = self.parse_expression()?;
            self.expect(TokenKind::RParen)?;
            MatchPattern::Literal(expr)
        } else {
            let tok = self.advance();
            return Err(ParseError::UnexpectedToken {
                token: tok.lexeme.clone(),
                span: tok.span,
            });
        };

        if self.check(&TokenKind::FatArrow) { self.advance(); }
        let body = self.parse_block()?;
        if self.check(&TokenKind::Comma) { self.advance(); }

        Ok(MatchArm { pattern, body, span: start_span })
    }

    // ── Error Handler: => |err| { ... } ──
    fn parse_error_handler(&mut self) -> Result<ErrorHandler, ParseError> {
        let start_span = self.current_span();
        self.expect(TokenKind::FatArrow)?;
        self.expect(TokenKind::Pipe)?;
        let param = self.expect_identifier()?;
        self.expect(TokenKind::Pipe)?;
        let body = self.parse_block()?;

        Ok(ErrorHandler { param, body, span: start_span })
    }

    // ── Expression Statement ──
    fn parse_expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        let expr = self.parse_expression()?;

        // Assignment: expr = value;
        if self.check(&TokenKind::Eq) {
            self.advance();
            let value = self.parse_expression()?;
            self.expect(TokenKind::Semicolon)?;
            return Ok(Stmt::Assignment {
                target: expr,
                value,
                span: start_span,
            });
        }

        // Error handler on expression
        if self.check(&TokenKind::FatArrow) {
            let handler = self.parse_error_handler()?;
            let span = start_span;
            let wrapped = Expr::WithErrorHandler {
                expr: Box::new(expr),
                handler,
                span,
            };
            self.expect(TokenKind::Semicolon)?;
            return Ok(Stmt::Expression { expr: wrapped, span: start_span });
        }

        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Expression { expr, span: start_span })
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Expressions — Pratt parser (precedence climbing)
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_or_expr()
    }

    fn parse_or_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and_expr()?;

        while self.check(&TokenKind::PipePipe) {
            let span = self.current_span();
            self.advance();
            let right = self.parse_and_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::Or,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_comparison_expr()?;

        while self.check(&TokenKind::AmpAmp) {
            let span = self.current_span();
            self.advance();
            let right = self.parse_comparison_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::And,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_comparison_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_additive_expr()?;

        loop {
            let op = match self.peek_kind() {
                TokenKind::EqEq  => BinOp::Eq,
                TokenKind::NotEq => BinOp::NotEq,
                TokenKind::Lt    => BinOp::Lt,
                TokenKind::LtEq  => BinOp::LtEq,
                TokenKind::Gt    => BinOp::Gt,
                TokenKind::GtEq  => BinOp::GtEq,
                _ => break,
            };
            let span = self.current_span();
            self.advance();
            let right = self.parse_additive_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_additive_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative_expr()?;

        loop {
            let op = match self.peek_kind() {
                TokenKind::Plus  => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            let span = self.current_span();
            self.advance();
            let right = self.parse_multiplicative_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_multiplicative_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary_expr()?;

        loop {
            let op = match self.peek_kind() {
                TokenKind::Star    => BinOp::Mul,
                TokenKind::Slash   => BinOp::Div,
                TokenKind::Percent => BinOp::Mod,
                _ => break,
            };
            let span = self.current_span();
            self.advance();
            let right = self.parse_unary_expr()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr, ParseError> {
        match self.peek_kind() {
            TokenKind::Bang => {
                let span = self.current_span();
                self.advance();
                let operand = self.parse_unary_expr()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                    span,
                })
            }
            TokenKind::Minus => {
                let span = self.current_span();
                self.advance();
                let operand = self.parse_unary_expr()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Negate,
                    operand: Box::new(operand),
                    span,
                })
            }
            TokenKind::Star => {
                let span = self.current_span();
                self.advance();
                let operand = self.parse_unary_expr()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Deref,
                    operand: Box::new(operand),
                    span,
                })
            }
            TokenKind::Amp => {
                let span = self.current_span();
                self.advance();
                if self.check(&TokenKind::Mut) {
                    self.advance();
                    let operand = self.parse_unary_expr()?;
                    Ok(Expr::Unary {
                        op: UnaryOp::RefMut,
                        operand: Box::new(operand),
                        span,
                    })
                } else {
                    let operand = self.parse_unary_expr()?;
                    Ok(Expr::Unary {
                        op: UnaryOp::Ref,
                        operand: Box::new(operand),
                        span,
                    })
                }
            }
            _ => self.parse_postfix_expr(),
        }
    }

    fn parse_postfix_expr(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary_expr()?;

        loop {
            if self.check(&TokenKind::ColonColon) {
                self.advance();

                // Identifier sonrası ( gelirse → method call
                let member = self.expect_identifier()?;

                if self.check(&TokenKind::LParen) {
                    self.advance();
                    let args = self.parse_arg_list()?;
                    self.expect(TokenKind::RParen)?;
                    let span = self.current_span();
                    expr = Expr::MethodCall {
                        object: Box::new(expr),
                        method: member,
                        args,
                        span,
                    };
                } else if self.check(&TokenKind::LBrace) {
                    // Struct init: Self { field: value, ... }
                    let mut path = match &expr {
                        Expr::Identifier { name, .. } => vec![name.clone()],
                        Expr::PathExpr { segments, .. } => segments.clone(),
                        _ => vec![],
                    };
                    path.push(member);

                    self.advance(); // {
                    let mut fields: Vec<(String, Expr)> = Vec::new();
                    while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
                        let fname = self.expect_identifier()?;
                        self.expect(TokenKind::Colon)?;
                        let fvalue = self.parse_expression()?;
                        fields.push((fname, fvalue));
                        if self.check(&TokenKind::Comma) { self.advance(); }
                    }
                    self.expect(TokenKind::RBrace)?;
                    let span = self.current_span();
                    expr = Expr::StructInit {
                        type_name: path,
                        fields,
                        span,
                    };
                } else {
                    // Path expression: ExitCode::Success
                    let span = self.current_span();
                    let mut segments = match expr {
                        Expr::Identifier { name, .. } => vec![name],
                        Expr::PathExpr { segments, .. } => segments,
                        other => vec![format!("{:?}", other)],
                    };
                    segments.push(member);
                    expr = Expr::PathExpr { segments, span };
                }
            } else if self.check(&TokenKind::Dot) {
                self.advance();
                let field = self.expect_identifier()?;

                if self.check(&TokenKind::LParen) {
                    self.advance();
                    let args = self.parse_arg_list()?;
                    self.expect(TokenKind::RParen)?;
                    let span = self.current_span();
                    expr = Expr::MethodCall {
                        object: Box::new(expr),
                        method: field,
                        args,
                        span,
                    };
                } else {
                    let span = self.current_span();
                    expr = Expr::FieldAccess {
                        object: Box::new(expr),
                        field,
                        span,
                    };
                }
            } else if self.check(&TokenKind::LParen) {
                self.advance();
                let args = self.parse_arg_list()?;
                self.expect(TokenKind::RParen)?;
                let span = self.current_span();
                expr = Expr::FnCall {
                    callee: Box::new(expr),
                    args,
                    span,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary_expr(&mut self) -> Result<Expr, ParseError> {
        let tok = self.peek().clone();

        match &tok.kind {
            // Integer literal
            TokenKind::LitInteger { value, suffix } => {
                let v = *value;
                let s = suffix.map(|s| format!("{}", s));
                let span = tok.span;
                self.advance();
                Ok(Expr::IntLiteral { value: v, suffix: s, span })
            }

            // Float literal
            TokenKind::LitFloat { value, suffix } => {
                let v = *value;
                let s = suffix.map(|s| format!("{}", s));
                let span = tok.span;
                self.advance();
                Ok(Expr::FloatLiteral { value: v, suffix: s, span })
            }

            // String literal
            TokenKind::LitString(s) => {
                let val = s.clone();
                let span = tok.span;
                self.advance();
                Ok(Expr::StringLiteral { value: val, span })
            }

            // Char literal
            TokenKind::LitChar(c) => {
                let val = *c;
                let span = tok.span;
                self.advance();
                Ok(Expr::CharLiteral { value: val, span })
            }

            // self
            TokenKind::SelfValue => {
                let span = tok.span;
                self.advance();
                Ok(Expr::SelfValue { span })
            }

            // Self (type olarak expression'da kullanıldığında)
            TokenKind::SelfType => {
                let span = tok.span;
                self.advance();
                Ok(Expr::Identifier { name: "Self".to_string(), span })
            }

            // Identifier (veya type keyword olarak identifier)
            TokenKind::Ident(_) => {
                let name = self.expect_identifier()?;
                let span = tok.span;
                Ok(Expr::Identifier { name, span })
            }

            // Type keywords da identifier olarak kullanılabilir (Int32, String vs.)
            _ if tok.kind.is_type_keyword() => {
                let name = tok.lexeme.clone();
                let span = tok.span;
                self.advance();
                Ok(Expr::Identifier { name, span })
            }

            // Grouped expression: (expr)
            TokenKind::LParen => {
                let span = tok.span;
                self.advance();
                let inner = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr::Grouped {
                    inner: Box::new(inner),
                    span,
                })
            }

            // Underscore
            TokenKind::Underscore => {
                let span = tok.span;
                self.advance();
                Ok(Expr::Identifier { name: "_".to_string(), span })
            }

            _ => {
                let t = self.advance();
                Err(ParseError::ExpectedExpression { span: t.span })
            }
        }
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Argument Lists: (name: value, name: value)
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn parse_arg_list(&mut self) -> Result<Vec<NamedArg>, ParseError> {
        let mut args: Vec<NamedArg> = Vec::new();

        if self.check(&TokenKind::RParen) {
            return Ok(args);
        }

        args.push(self.parse_single_arg()?);

        while self.check(&TokenKind::Comma) {
            self.advance();
            if self.check(&TokenKind::RParen) { break; }
            args.push(self.parse_single_arg()?);
        }

        Ok(args)
    }

    fn parse_single_arg(&mut self) -> Result<NamedArg, ParseError> {
        let start_span = self.current_span();

        if self.is_named_arg_ahead() {
            let name = self.expect_identifier()?;
            self.expect(TokenKind::Colon)?;
            let value = self.parse_expression()?;
            Ok(NamedArg { name: Some(name), value, span: start_span })
        } else {
            let value = self.parse_expression()?;
            Ok(NamedArg { name: None, value, span: start_span })
        }
    }

    fn is_named_arg_ahead(&self) -> bool {
        if let TokenKind::Ident(_) = self.peek_kind() {
            if self.current + 1 < self.tokens.len() {
                return matches!(self.tokens[self.current + 1].kind, TokenKind::Colon);
            }
        }
        false
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Helper Functions
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn peek_kind(&self) -> TokenKind {
        self.tokens[self.current].kind.clone()
    }

    fn peek_next_kind(&self) -> Option<&str> {
        if self.current + 1 < self.tokens.len() {
            if let TokenKind::Ident(ref s) = self.tokens[self.current + 1].kind {
                return Some(s.as_str());
            }
        }
        None
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens[self.current].clone();
        if self.current < self.tokens.len() - 1 {
            self.current += 1;
        }
        tok
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.tokens[self.current].kind) == std::mem::discriminant(kind)
    }

    fn is_at_end(&self) -> bool {
        matches!(self.tokens[self.current].kind, TokenKind::EOF)
    }

    fn current_span(&self) -> Span {
        self.tokens[self.current].span
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, ParseError> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            let tok = self.peek().clone();
            Err(ParseError::ExpectedToken {
                expected: format!("{:?}", kind),
                found: tok.lexeme.clone(),
                span: tok.span,
            })
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        let tok = self.peek().clone();
        match &tok.kind {
            TokenKind::Ident(name) => {
                let n = name.clone();
                self.advance();
                Ok(n)
            }
            _ => Err(ParseError::ExpectedIdentifier {
                found: tok.lexeme.clone(),
                span: tok.span,
            })
        }
    }

    fn expect_any_identifier(&mut self) -> Result<String, ParseError> {
        let tok = self.peek().clone();
        match &tok.kind {
            TokenKind::Ident(name) => {
                let n = name.clone();
                self.advance();
                Ok(n)
            }
            // Type keywords identifier olarak kabul et
            _ if tok.kind.is_type_keyword() => {
                let n = tok.lexeme.clone();
                self.advance();
                Ok(n)
            }
            _ => Err(ParseError::ExpectedIdentifier {
                found: tok.lexeme.clone(),
                span: tok.span,
            })
        }
    }

    fn expect_ident_matching(&mut self, expected: &str) -> Result<(), ParseError> {
        let tok = self.peek().clone();
        match &tok.kind {
            TokenKind::Ident(name) if name == expected => {
                self.advance();
                Ok(())
            }
            // "else" is a keyword, handle it
            TokenKind::Else if expected == "else" => {
                self.advance();
                Ok(())
            }
            _ => Err(ParseError::ExpectedToken {
                expected: expected.to_string(),
                found: tok.lexeme.clone(),
                span: tok.span,
            })
        }
    }

    fn try_string_literal(&mut self) -> Option<String> {
        if let TokenKind::LitString(s) = &self.peek().kind {
            let val = s.clone();
            self.advance();
            Some(val)
        } else {
            None
        }
    }

    fn skip_comments(&mut self) {
        while matches!(self.peek_kind(), TokenKind::Comment(_)) {
            self.advance();
        }
    }

    fn synchronize(&mut self) {
        while !self.is_at_end() {
            if matches!(self.peek_kind(), TokenKind::Semicolon) {
                self.advance();
                return;
            }
            match self.peek_kind() {
                TokenKind::Contract | TokenKind::Fn | TokenKind::Let |
                TokenKind::Return | TokenKind::If | TokenKind::Match |
                TokenKind::Guard | TokenKind::Enumeration => return,
                _ => { self.advance(); }
            }
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Unit Tests
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[cfg(test)]
mod tests {
    use super::*;
    use chronos_lexer::Lexer;

    fn parse(input: &str) -> Program {
        let tokens = Lexer::new(input).tokenize().expect("Lexer failed");
        Parser::new(tokens).parse().expect("Parser failed")
    }

    #[test]
    fn test_module_directive() {
        let prog = parse("#![module::entry(main)]");
        assert_eq!(prog.module_directives.len(), 1);
        assert_eq!(prog.module_directives[0].path, vec!["module", "entry"]);
        assert_eq!(prog.module_directives[0].value, "main");
    }

    #[test]
    fn test_require_statement() {
        let prog = parse("@require core::io::{ StreamWriter, BufferMode };");
        assert_eq!(prog.require_statements.len(), 1);
        assert_eq!(prog.require_statements[0].module_path, vec!["core", "io"]);
        assert_eq!(prog.require_statements[0].imports, vec!["StreamWriter", "BufferMode"]);
    }

    #[test]
    fn test_empty_contract() {
        let prog = parse("contract Main :: EntryPoint { }");
        assert_eq!(prog.declarations.len(), 1);
        match &prog.declarations[0] {
            Declaration::Contract(c) => {
                assert_eq!(c.name, "Main");
                assert_eq!(c.traits, vec!["EntryPoint"]);
            }
            _ => panic!("Expected contract"),
        }
    }

    #[test]
    fn test_contract_with_field() {
        let prog = parse(r#"
            contract Animal :: Displayable {
                field name: String;
            }
        "#);
        match &prog.declarations[0] {
            Declaration::Contract(c) => {
                assert_eq!(c.members.len(), 1);
                match &c.members[0] {
                    ContractMember::Field(f) => {
                        assert_eq!(f.name, "name");
                    }
                    _ => panic!("Expected field"),
                }
            }
            _ => panic!("Expected contract"),
        }
    }

    #[test]
    fn test_function_in_contract() {
        let prog = parse(r#"
            contract Main :: EntryPoint {
                fn main(args: Vector<String>) -> ExitCode {
                    return ExitCode::Success(0x00);
                }
            }
        "#);
        match &prog.declarations[0] {
            Declaration::Contract(c) => {
                assert_eq!(c.members.len(), 1);
                match &c.members[0] {
                    ContractMember::Method(m) => {
                        assert_eq!(m.name, "main");
                        assert_eq!(m.params.len(), 1);
                        assert_eq!(m.params[0].name, "args");
                    }
                    _ => panic!("Expected method"),
                }
            }
            _ => panic!("Expected contract"),
        }
    }

    #[test]
    fn test_let_statement() {
        let prog = parse(r#"
            contract Main :: EntryPoint {
                fn main() -> Void {
                    let x: Int32 = 42i32;
                }
            }
        "#);
        match &prog.declarations[0] {
            Declaration::Contract(c) => {
                match &c.members[0] {
                    ContractMember::Method(m) => {
                        assert_eq!(m.body.len(), 1);
                        match &m.body[0] {
                            Stmt::Let { name, mutable, .. } => {
                                assert_eq!(name, "x");
                                assert_eq!(*mutable, false);
                            }
                            _ => panic!("Expected let"),
                        }
                    }
                    _ => panic!("Expected method"),
                }
            }
            _ => panic!("Expected contract"),
        }
    }

    #[test]
    fn test_annotated_function() {
        let prog = parse(r#"
            contract Main :: EntryPoint {
                @static
                @throws(IOError)
                fn main() -> Void {
                    return;
                }
            }
        "#);
        match &prog.declarations[0] {
            Declaration::Contract(c) => {
                match &c.members[0] {
                    ContractMember::Method(m) => {
                        assert_eq!(m.annotations.len(), 2);
                        assert_eq!(m.annotations[0].name, "static");
                        assert_eq!(m.annotations[1].name, "throws");
                    }
                    _ => panic!("Expected method"),
                }
            }
            _ => panic!("Expected contract"),
        }
    }

    #[test]
    fn test_method_call_expression() {
        let prog = parse(r#"
            contract Main :: EntryPoint {
                fn main() -> Void {
                    let x: Int32 = Checked::add(left: 1i32, right: 2i32);
                }
            }
        "#);
        match &prog.declarations[0] {
            Declaration::Contract(c) => {
                match &c.members[0] {
                    ContractMember::Method(m) => {
                        match &m.body[0] {
                            Stmt::Let { initializer, .. } => {
                                match initializer {
                                    Expr::MethodCall { method, args, .. } => {
                                        assert_eq!(method, "add");
                                        assert_eq!(args.len(), 2);
                                    }
                                    _ => panic!("Expected method call, got {:?}", initializer),
                                }
                            }
                            _ => panic!("Expected let"),
                        }
                    }
                    _ => panic!("Expected method"),
                }
            }
            _ => panic!("Expected contract"),
        }
    }

    #[test]
    fn test_binary_expression() {
        let prog = parse(r#"
            contract Main :: EntryPoint {
                fn main() -> Void {
                    let result: Bool = x > 0i32;
                }
            }
        "#);
        match &prog.declarations[0] {
            Declaration::Contract(c) => {
                match &c.members[0] {
                    ContractMember::Method(m) => {
                        match &m.body[0] {
                            Stmt::Let { initializer, .. } => {
                                match initializer {
                                    Expr::Binary { op, .. } => {
                                        assert_eq!(*op, BinOp::Gt);
                                    }
                                    _ => panic!("Expected binary expr"),
                                }
                            }
                            _ => panic!("Expected let"),
                        }
                    }
                    _ => panic!("Expected method"),
                }
            }
            _ => panic!("Expected contract"),
        }
    }

    #[test]
    fn test_full_program() {
        let prog = parse(r#"
            #![module::entry(main)]
            @require core::io::{ StreamWriter, BufferMode };
            @require core::types::{ String, Int32, ExitCode };

            contract Main :: EntryPoint {
                @static
                fn main(args: Vector<String>) -> ExitCode {
                    let x: Int32 = 42i32;
                    let name: String = "CHRONOS";
                    return ExitCode::Success(0x00);
                }
            }
        "#);

        assert_eq!(prog.module_directives.len(), 1);
        assert_eq!(prog.require_statements.len(), 2);
        assert_eq!(prog.declarations.len(), 1);
    }
}
