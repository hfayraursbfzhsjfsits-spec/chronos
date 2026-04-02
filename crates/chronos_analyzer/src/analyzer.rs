use chronos_parser::*;
use crate::types::ChronosType;
use crate::symbol_table::*;
use crate::errors::SemanticError;

pub struct Analyzer {
    pub symbol_table: SymbolTable,
    pub errors: Vec<SemanticError>,
    pub warnings: Vec<SemanticError>,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn analyze(mut self, program: &Program) -> AnalysisResult {
        self.register_declarations(program);

        for decl in &program.declarations {
            self.analyze_declaration(decl);
        }

        AnalysisResult {
            symbol_table: self.symbol_table,
            errors: self.errors,
            warnings: self.warnings,
        }
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn register_declarations(&mut self, program: &Program) {
        for decl in &program.declarations {
            match decl {
                Declaration::Contract(c) => {
                    let mut fields: Vec<(String, ChronosType)> = Vec::new();
                    let mut methods: Vec<FunctionSignature> = Vec::new();

                    for member in &c.members {
                        match member {
                            ContractMember::Field(f) => {
                                let ty = self.resolve_type(&f.type_expr);
                                fields.push((f.name.clone(), ty));
                            }
                            ContractMember::Method(m) => {
                                let sig = self.build_function_signature(m);
                                methods.push(sig);
                            }
                        }
                    }

                    let info = ContractInfo {
                        name: c.name.clone(),
                        traits: c.traits.clone(),
                        fields,
                        methods,
                    };
                    self.symbol_table.register_contract(info);

                    self.symbol_table.define(Symbol {
                        name: c.name.clone(),
                        symbol_type: ChronosType::Contract(c.name.clone()),
                        kind: SymbolKind::Contract,
                        mutable: false,
                        initialized: true,
                        used: false,
                        line: c.span.line,
                        column: c.span.column,
                    });
                }
                Declaration::Function(f) => {
                    let sig = self.build_function_signature(f);
                    self.symbol_table.register_function(sig);

                    self.symbol_table.define(Symbol {
                        name: f.name.clone(),
                        symbol_type: self.resolve_type(&f.return_type),
                        kind: SymbolKind::Function,
                        mutable: false,
                        initialized: true,
                        used: false,
                        line: f.span.line,
                        column: f.span.column,
                    });
                }
                Declaration::Enumeration(e) => {
                    self.symbol_table.define(Symbol {
                        name: e.name.clone(),
                        symbol_type: ChronosType::Enum(e.name.clone()),
                        kind: SymbolKind::Enumeration,
                        mutable: false,
                        initialized: true,
                        used: false,
                        line: e.span.line,
                        column: e.span.column,
                    });
                }
            }
        }
    }

    fn build_function_signature(&self, f: &FnDecl) -> FunctionSignature {
        let params: Vec<(String, ChronosType)> = f
            .params
            .iter()
            .map(|p| (p.name.clone(), self.resolve_type(&p.type_expr)))
            .collect();

        let return_type = self.resolve_type(&f.return_type);

        let annotations: Vec<String> = f
            .annotations
            .iter()
            .map(|a| a.name.clone())
            .collect();

        FunctionSignature {
            name: f.name.clone(),
            params,
            return_type,
            annotations,
        }
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Pass 2: Declaration Analizi
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn analyze_declaration(&mut self, decl: &Declaration) {
        match decl {
            Declaration::Contract(c) => self.analyze_contract(c),
            Declaration::Function(f) => {
                self.symbol_table
                    .push_scope(ScopeType::Function(f.name.clone()));
                self.register_params(&f.params);
                self.analyze_block(&f.body);
                self.check_return(&f.name, &f.return_type, &f.body, f.span);
                self.symbol_table.pop_scope();
            }
            Declaration::Enumeration(_) => {}
        }
    }

    fn analyze_contract(&mut self, contract: &ContractDecl) {
        self.symbol_table
            .push_scope(ScopeType::Contract(contract.name.clone()));

        for member in &contract.members {
            if let ContractMember::Field(f) = member {
                let ty = self.resolve_type(&f.type_expr);
                self.symbol_table.define(Symbol {
                    name: f.name.clone(),
                    symbol_type: ty,
                    kind: SymbolKind::Field,
                    mutable: true,
                    initialized: false,
                    used: false,
                    line: f.span.line,
                    column: f.span.column,
                });
            }
        }

        for member in &contract.members {
            if let ContractMember::Method(m) = member {
                self.analyze_method(m);
            }
        }

        self.symbol_table.pop_scope();
    }

    fn analyze_method(&mut self, method: &FnDecl) {
        self.symbol_table
            .push_scope(ScopeType::Function(method.name.clone()));

        self.register_params(&method.params);
        self.analyze_block(&method.body);
        self.check_return(&method.name, &method.return_type, &method.body, method.span);

        self.symbol_table.pop_scope();
    }

    fn register_params(&mut self, params: &[Param]) {
        for param in params {
            let ty = self.resolve_type(&param.type_expr);
            self.symbol_table.define(Symbol {
                name: param.name.clone(),
                symbol_type: ty,
                kind: SymbolKind::Parameter,
                mutable: false,
                initialized: true,
                used: false,
                line: param.span.line,
                column: param.span.column,
            });
        }
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Block & Statement Analizi
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn analyze_block(&mut self, block: &Block) {
        for stmt in block {
            self.analyze_statement(stmt);
        }
    }

    fn analyze_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let {
                mutable,
                name,
                type_expr,
                initializer,
                span,
                ..
            } => {
                if self.symbol_table.lookup_current_scope(name).is_some() {
                    self.errors.push(SemanticError::AlreadyDeclared {
                        name: name.clone(),
                        span: *span,
                    });
                    return;
                }

                let declared_type = self.resolve_type(type_expr);
                let init_type = self.infer_expr_type(initializer);

                if !declared_type.is_assignable_from(&init_type) {
                    self.errors.push(SemanticError::TypeMismatch {
                        expected: declared_type.to_string(),
                        found: init_type.to_string(),
                        span: *span,
                    });
                }

                self.symbol_table.define(Symbol {
                    name: name.clone(),
                    symbol_type: declared_type,
                    kind: SymbolKind::Variable,
                    mutable: *mutable,
                    initialized: true,
                    used: false,
                    line: span.line,
                    column: span.column,
                });

                self.check_expr(initializer);
            }

            Stmt::Return { value, span: _ } => {
                if let Some(expr) = value {
                    self.check_expr(expr);
                }
            }

            Stmt::Guard {
                condition,
                else_block,
                ..
            } => {
                let _cond_type = self.infer_expr_type(condition);
                self.check_expr(condition);

                self.symbol_table.push_scope(ScopeType::Block);
                self.analyze_block(else_block);
                self.symbol_table.pop_scope();
            }

            Stmt::If {
                condition,
                then_block,
                else_if_blocks,
                else_block,
                ..
            } => {
                self.check_expr(condition);

                self.symbol_table.push_scope(ScopeType::Block);
                self.analyze_block(then_block);
                self.symbol_table.pop_scope();

                for (cond, block) in else_if_blocks {
                    self.check_expr(cond);
                    self.symbol_table.push_scope(ScopeType::Block);
                    self.analyze_block(block);
                    self.symbol_table.pop_scope();
                }

                if let Some(block) = else_block {
                    self.symbol_table.push_scope(ScopeType::Block);
                    self.analyze_block(block);
                    self.symbol_table.pop_scope();
                }
            }

            Stmt::While {
                condition,
                body,
                ..
            } => {
                self.check_expr(condition);

                self.symbol_table.push_scope(ScopeType::Loop);
                self.analyze_block(body);
                self.symbol_table.pop_scope();
            }

            Stmt::Match { value, arms, .. } => {
                self.check_expr(value);
                for arm in arms {
                    self.symbol_table.push_scope(ScopeType::Block);
                    self.analyze_block(&arm.body);
                    self.symbol_table.pop_scope();
                }
            }

            Stmt::Assignment {
                target,
                value,
                span,
            } => {
                if let Expr::Identifier { name, .. } = target {
                    if let Some(sym) = self.symbol_table.lookup(name) {
                        if !sym.mutable {
                            self.errors.push(SemanticError::ImmutableAssignment {
                                name: name.clone(),
                                span: *span,
                            });
                        }
                    } else {
                        self.errors.push(SemanticError::UndefinedVariable {
                            name: name.clone(),
                            span: *span,
                        });
                    }
                    self.symbol_table.mark_used(name);
                }

                self.check_expr(value);
            }

            Stmt::Expression { expr, .. } => {
                self.check_expr(expr);
            }

            Stmt::Break { span } => {
                if !self.is_inside_loop() {
                    self.errors.push(SemanticError::BreakOutsideLoop {
                        span: *span,
                    });
                }
            }

            Stmt::Continue { span } => {
                if !self.is_inside_loop() {
                    self.errors.push(SemanticError::ContinueOutsideLoop {
                        span: *span,
                    });
                }
            }
        }
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn check_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Identifier { name, span } => {
                if self.symbol_table.lookup(name).is_none()
                    && !self.is_known_builtin(name)
                {
                    self.errors.push(SemanticError::UndefinedVariable {
                        name: name.clone(),
                        span: *span,
                    });
                } else {
                    self.symbol_table.mark_used(name);
                }
            }

            Expr::Binary { left, right, op, span } => {
                self.check_expr(left);
                self.check_expr(right);

                let left_type = self.infer_expr_type(left);
                let right_type = self.infer_expr_type(right);

                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        if !left_type.is_numeric() && left_type != ChronosType::Error
                            && !matches!(left_type, ChronosType::Unresolved(_))
                        {
                            self.errors.push(SemanticError::InvalidOperator {
                                op: format!("{:?}", op),
                                left: left_type.to_string(),
                                right: right_type.to_string(),
                                span: *span,
                            });
                        }
                    }
                    _ => {}
                }
            }

            Expr::Unary { operand, .. } => {
                self.check_expr(operand);
            }

            Expr::MethodCall { object, args, .. } => {
                self.check_expr(object);
                for arg in args {
                    self.check_expr(&arg.value);
                }
            }

            Expr::FnCall { callee, args, .. } => {
                self.check_expr(callee);
                for arg in args {
                    self.check_expr(&arg.value);
                }
            }

            Expr::FieldAccess { object, .. } => {
                self.check_expr(object);
            }

            Expr::WithErrorHandler { expr, handler, .. } => {
                self.check_expr(expr);
                self.symbol_table.push_scope(ScopeType::Block);
                self.symbol_table.define(Symbol {
                    name: handler.param.clone(),
                    symbol_type: ChronosType::Unresolved("Error".to_string()),
                    kind: SymbolKind::Parameter,
                    mutable: false,
                    initialized: true,
                    used: false,
                    line: handler.span.line,
                    column: handler.span.column,
                });
                self.analyze_block(&handler.body);
                self.symbol_table.pop_scope();
            }

            Expr::Grouped { inner, .. } => {
                self.check_expr(inner);
            }

            Expr::StructInit { fields, .. } => {
                for (_, value) in fields {
                    self.check_expr(value);
                }
            }

            _ => {}
        }
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn infer_expr_type(&self, expr: &Expr) -> ChronosType {
        match expr {
            Expr::IntLiteral { suffix, .. } => {
                match suffix.as_deref() {
                    Some("i8")  => ChronosType::Int8,
                    Some("i16") => ChronosType::Int16,
                    Some("i32") => ChronosType::Int32,
                    Some("i64") => ChronosType::Int64,
                    Some("u8")  => ChronosType::UInt8,
                    Some("u16") => ChronosType::UInt16,
                    Some("u32") => ChronosType::UInt32,
                    Some("u64") => ChronosType::UInt64,
                    None        => ChronosType::Int64,
                    _           => ChronosType::Error,
                }
            }

            Expr::FloatLiteral { suffix, .. } => {
                match suffix.as_deref() {
                    Some("f32") => ChronosType::Float32,
                    Some("f64") => ChronosType::Float64,
                    None        => ChronosType::Float64,
                    _           => ChronosType::Error,
                }
            }

            Expr::StringLiteral { .. } => ChronosType::StringType,
            Expr::CharLiteral { .. } => ChronosType::Char,
            Expr::BoolLiteral { .. } => ChronosType::Bool,

            Expr::Identifier { name, .. } => {
                if let Some(sym) = self.symbol_table.lookup(name) {
                    sym.symbol_type.clone()
                } else {
                    ChronosType::Unresolved(name.clone())
                }
            }

            Expr::PathExpr { segments, .. } => {
                if let Some(first) = segments.first() {
                    if self.symbol_table.contracts.contains_key(first) {
                        ChronosType::Contract(first.clone())
                    } else {
                        ChronosType::Unresolved(segments.join("::"))
                    }
                } else {
                    ChronosType::Error
                }
            }

            Expr::Binary { left, op, .. } => {
                match op {
                    BinOp::Eq | BinOp::NotEq | BinOp::Lt |
                    BinOp::LtEq | BinOp::Gt | BinOp::GtEq |
                    BinOp::And | BinOp::Or => ChronosType::Bool,
                    _ => self.infer_expr_type(left),
                }
            }

            Expr::Unary { op, operand, .. } => {
                match op {
                    UnaryOp::Not => ChronosType::Bool,
                    UnaryOp::Ref => {
                        let inner = self.infer_expr_type(operand);
                        ChronosType::Reference {
                            mutable: false,
                            inner: Box::new(inner),
                        }
                    }
                    UnaryOp::RefMut => {
                        let inner = self.infer_expr_type(operand);
                        ChronosType::Reference {
                            mutable: true,
                            inner: Box::new(inner),
                        }
                    }
                    UnaryOp::Deref => {
                        let inner = self.infer_expr_type(operand);
                        if let ChronosType::Reference { inner, .. } = inner {
                            *inner
                        } else {
                            inner
                        }
                    }
                    _ => self.infer_expr_type(operand),
                }
            }

            Expr::MethodCall { .. } => ChronosType::Unresolved("method_return".to_string()),
            Expr::FnCall { .. } => ChronosType::Unresolved("fn_return".to_string()),
            Expr::Grouped { inner, .. } => self.infer_expr_type(inner),
            Expr::SelfValue { .. } => ChronosType::SelfType,

            _ => ChronosType::Unresolved("unknown".to_string()),
        }
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Type Resolution
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn resolve_type(&self, type_expr: &TypeExpr) -> ChronosType {
        match type_expr {
            TypeExpr::Named(name, _) => ChronosType::from_name(name),

            TypeExpr::Generic { base, type_args, .. } => {
                let resolved_args: Vec<ChronosType> = type_args
                    .iter()
                    .map(|t| self.resolve_type(t))
                    .collect();

                match base.as_str() {
                    "Result" if resolved_args.len() == 2 => {
                        ChronosType::Result {
                            ok_type: Box::new(resolved_args[0].clone()),
                            err_type: Box::new(resolved_args[1].clone()),
                        }
                    }
                    "Optional" if resolved_args.len() == 1 => {
                        ChronosType::Optional {
                            inner: Box::new(resolved_args[0].clone()),
                        }
                    }
                    "Tuple" => {
                        ChronosType::Tuple { elements: resolved_args }
                    }
                    _ => {
                        ChronosType::Generic {
                            base: base.clone(),
                            type_args: resolved_args,
                        }
                    }
                }
            }

            TypeExpr::Reference { mutable, inner, .. } => {
                ChronosType::Reference {
                    mutable: *mutable,
                    inner: Box::new(self.resolve_type(inner)),
                }
            }

            TypeExpr::Tuple { elements, .. } => {
                ChronosType::Tuple {
                    elements: elements.iter().map(|t| self.resolve_type(t)).collect(),
                }
            }

            TypeExpr::Closure { params, return_type, .. } => {
                ChronosType::Closure {
                    params: params.iter().map(|t| self.resolve_type(t)).collect(),
                    return_type: Box::new(self.resolve_type(return_type)),
                }
            }

            TypeExpr::SelfType(_) => ChronosType::SelfType,
        }
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    //  Return Flow Analysis
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn check_return(
        &mut self,
        fn_name: &str,
        return_type: &TypeExpr,
        body: &Block,
        span: chronos_lexer::Span,
    ) {
        let expected = self.resolve_type(return_type);

        if expected == ChronosType::Void {
            return;
        }

        let guaranteed = self.block_guarantees_return(body);

        if !guaranteed {
            self.errors.push(SemanticError::MissingReturn {
                name: fn_name.to_string(),
                return_type: expected.to_string(),
                span,
            });
        }
    }

    fn block_guarantees_return(&self, block: &Block) -> bool {
        if let Some(last_stmt) = block.last() {
            self.statement_guarantees_return(last_stmt)
        } else {
            false
        }
    }

    fn statement_guarantees_return(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Return { .. } => true,

            Stmt::If {
                then_block,
                else_if_blocks,
                else_block,
                ..
            } => {
                if !self.block_guarantees_return(then_block) {
                    return false;
                }

                for (_, block) in else_if_blocks {
                    if !self.block_guarantees_return(block) {
                        return false;
                    }
                }

                match else_block {
                    Some(block) => self.block_guarantees_return(block),
                    None => false,
                }
            }

            Stmt::Match { arms, .. } => {
                if arms.is_empty() {
                    return false;
                }

                let mut has_default = false;

                for arm in arms {
                    if matches!(arm.pattern, MatchPattern::Default) {
                        has_default = true;
                    }

                    if !self.block_guarantees_return(&arm.body) {
                        return false;
                    }
                }

                has_default
            }

            _ => false,
        }
    }

    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    // ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

    fn is_inside_loop(&self) -> bool {
        self.symbol_table.is_inside_loop()
    }

    fn is_known_builtin(&self, name: &str) -> bool {
        matches!(
            name,
            "StdOut" | "StdErr" | "StdIn"
            | "Encoding" | "LineEnding"
            | "Allocator" | "HeapRegion" | "Lifetime"
            | "Duration" | "ThreadPriority"
            | "Range" | "USize"
            | "Checked"
            | "panic"
            | "assert"
            | "StreamWriter" | "StreamReader"
            | "BufferMode"
            | "Borrow"
            | "Tuple"
            | "ExitCode"
            | "Vector"
            | "Result"
            | "Optional"
            | "Bool"
            | "Closure"
            | "CaptureMode"
            | "String"
            | "Mutex" | "MutexPolicy"
            | "Thread" | "Channel"
            | "TcpStream"
        )
    }
}

pub struct AnalysisResult {
    pub symbol_table: SymbolTable,
    pub errors: Vec<SemanticError>,
    pub warnings: Vec<SemanticError>,
}

impl AnalysisResult {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chronos_lexer::Lexer;
    use chronos_parser::Parser;

    fn analyze_source(input: &str) -> AnalysisResult {
        let tokens = Lexer::new(input).tokenize().expect("Lexer failed");
        let program = Parser::new(tokens).parse().expect("Parser failed");
        Analyzer::new().analyze(&program)
    }

    #[test]
    fn test_valid_let_int() {
        let result = analyze_source(r#"
            contract Main :: EntryPoint {
                fn main() -> Void {
                    let x: Int32 = 42i32;
                }
            }
        "#);
        assert!(result.is_ok(), "Errors: {:?}", result.errors);
    }

    #[test]
    fn test_valid_let_string() {
        let result = analyze_source(r#"
            contract Main :: EntryPoint {
                fn main() -> Void {
                    let name: String = "CHRONOS";
                }
            }
        "#);
        assert!(result.is_ok(), "Errors: {:?}", result.errors);
    }

    #[test]
    fn test_type_mismatch_int_to_string() {
        let result = analyze_source(r#"
            contract Main :: EntryPoint {
                fn main() -> Void {
                    let name: String = 42i32;
                }
            }
        "#);
        assert!(result.has_errors());
        assert!(matches!(result.errors[0], SemanticError::TypeMismatch { .. }));
    }

    #[test]
    fn test_type_mismatch_string_to_int() {
        let result = analyze_source(r#"
            contract Main :: EntryPoint {
                fn main() -> Void {
                    let x: Int32 = "hello";
                }
            }
        "#);
        assert!(result.has_errors());
        assert!(matches!(result.errors[0], SemanticError::TypeMismatch { .. }));
    }

    #[test]
    fn test_duplicate_variable() {
        let result = analyze_source(r#"
            contract Main :: EntryPoint {
                fn main() -> Void {
                    let x: Int32 = 42i32;
                    let x: Int32 = 100i32;
                }
            }
        "#);
        assert!(result.has_errors());
        assert!(matches!(result.errors[0], SemanticError::AlreadyDeclared { .. }));
    }

    #[test]
    fn test_missing_return() {
        let result = analyze_source(r#"
            contract Main :: EntryPoint {
                fn main() -> ExitCode {
                    let x: Int32 = 42i32;
                }
            }
        "#);
        assert!(result.has_errors());
        assert!(matches!(result.errors[0], SemanticError::MissingReturn { .. }));
    }

    #[test]
    fn test_void_function_no_return() {
        let result = analyze_source(r#"
            contract Main :: EntryPoint {
                fn main() -> Void {
                    let x: Int32 = 42i32;
                }
            }
        "#);
        assert!(result.is_ok(), "Errors: {:?}", result.errors);
    }

    #[test]
    fn test_contract_registered() {
        let result = analyze_source(r#"
            contract Main :: EntryPoint {
                field name: String;
                fn main() -> Void {
                    return;
                }
            }
        "#);
        assert!(result.is_ok(), "Errors: {:?}", result.errors);
        assert!(result.symbol_table.contracts.contains_key("Main"));
        let info = result.symbol_table.get_contract("Main").unwrap();
        assert_eq!(info.fields.len(), 1);
        assert_eq!(info.methods.len(), 1);
    }

    #[test]
    fn test_multiple_contracts() {
        let result = analyze_source(r#"
            contract Animal :: Displayable {
                field name: String;
                fn get_name() -> String {
                    return self.name;
                }
            }
            contract Main :: EntryPoint {
                fn main() -> Void {
                    let x: Int32 = 42i32;
                }
            }
        "#);
        assert!(result.is_ok(), "Errors: {:?}", result.errors);
        assert!(result.symbol_table.contracts.contains_key("Animal"));
        assert!(result.symbol_table.contracts.contains_key("Main"));
    }

    #[test]
    fn test_function_with_params() {
        let result = analyze_source(r#"
            contract Main :: EntryPoint {
                fn main(args: Vector<String>) -> Void {
                    let x: Int32 = 42i32;
                }
            }
        "#);
        assert!(result.is_ok(), "Errors: {:?}", result.errors);
    }

    #[test]
    fn test_float_type_match() {
        let result = analyze_source(r#"
            contract Main :: EntryPoint {
                fn main() -> Void {
                    let pi: Float64 = 3.14f64;
                    let e: Float32 = 2.71f32;
                }
            }
        "#);
        assert!(result.is_ok(), "Errors: {:?}", result.errors);
    }

    #[test]
    fn test_float_type_mismatch() {
        let result = analyze_source(r#"
            contract Main :: EntryPoint {
                fn main() -> Void {
                    let pi: Float32 = 3.14f64;
                }
            }
        "#);
        assert!(result.has_errors());
        assert!(matches!(result.errors[0], SemanticError::TypeMismatch { .. }));
    }

    #[test]
    fn test_if_else_guaranteed_return() {
        let result = analyze_source(r#"
            fn classify(n: Int32) -> Int32 {
                if (condition: n > 0i32) => {
                    return 1i32;
                } else => {
                    return 0i32;
                };
            }
        "#);
        assert!(result.is_ok(), "Errors: {:?}", result.errors);
    }

    #[test]
    fn test_match_guaranteed_return() {
        let result = analyze_source(r#"
            fn test(n: Int32) -> Int32 {
                match (value: n) => {
                    case(1i32) => {
                        return 11i32;
                    },
                    default => {
                        return 99i32;
                    }
                };
            }
        "#);
        assert!(result.is_ok(), "Errors: {:?}", result.errors);
    }

    #[test]
    fn test_while_loop() {
        let result = analyze_source(r#"
            contract Main :: EntryPoint {
                fn main() -> Void {
                    let x: Int32 = 10i32;
                    while (condition: x > 0i32) => {
                        break;
                    };
                }
            }
        "#);
        assert!(result.is_ok(), "Errors: {:?}", result.errors);
    }
}
