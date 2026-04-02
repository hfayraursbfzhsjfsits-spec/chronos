use chronos_parser::*;
use crate::bytecode::*;

pub struct Compiler {
    program: CompiledProgram,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            program: CompiledProgram::new(),
        }
    }

    pub fn compile(mut self, ast: &Program) -> CompiledProgram {
        for dir in &ast.module_directives {
            if dir.path.last().map(|s| s.as_str()) == Some("entry") {
                self.program.entry_point = Some(dir.value.clone());
            }
        }

        for decl in &ast.declarations {
            self.compile_declaration(decl);
        }

        self.program
    }

    fn compile_declaration(&mut self, decl: &Declaration) {
        match decl {
            Declaration::Contract(c) => self.compile_contract(c),
            Declaration::Function(f) => {
                let chunk = self.compile_function(f);
                self.program.add_chunk(chunk);
            }
            Declaration::Enumeration(_) => {}
        }
    }

    fn compile_contract(&mut self, contract: &ContractDecl) {
        for member in &contract.members {
            if let ContractMember::Method(method) = member {
                let qualified_name = format!("{}::{}", contract.name, method.name);
                let mut chunk = Chunk::new(qualified_name);

                // Parametreleri store et — ters sıra
                for param in method.params.iter().rev() {
                    chunk.emit(OpCode::Store(param.name.clone()), method.span.line);
                }

                self.compile_block(&method.body, &mut chunk);

                if !matches!(chunk.code.last(), Some(OpCode::Return)) {
                    chunk.emit(OpCode::PushConst(Value::Void), method.span.line);
                    chunk.emit(OpCode::Return, method.span.line);
                }

                self.program.add_chunk(chunk);
            }
        }
    }

    fn compile_function(&self, func: &FnDecl) -> Chunk {
        let mut chunk = Chunk::new(func.name.clone());

        for param in func.params.iter().rev() {
            chunk.emit(OpCode::Store(param.name.clone()), func.span.line);
        }

        self.compile_block(&func.body, &mut chunk);

        if !matches!(chunk.code.last(), Some(OpCode::Return)) {
            chunk.emit(OpCode::PushConst(Value::Void), func.span.line);
            chunk.emit(OpCode::Return, func.span.line);
        }

        chunk
    }

    fn compile_block(&self, block: &Block, chunk: &mut Chunk) {
        for stmt in block {
            self.compile_statement(stmt, chunk);
        }
    }

    fn compile_statement(&self, stmt: &Stmt, chunk: &mut Chunk) {
        match stmt {
            Stmt::Let {
                mutable,
                name,
                initializer,
                span,
                ..
            } => {
                self.compile_expr(initializer, chunk);
                if *mutable {
                    chunk.emit(OpCode::StoreMut(name.clone()), span.line);
                } else {
                    chunk.emit(OpCode::Store(name.clone()), span.line);
                }
            }

            Stmt::Return { value, span } => {
                if let Some(expr) = value {
                    self.compile_expr(expr, chunk);
                } else {
                    chunk.emit(OpCode::PushConst(Value::Void), span.line);
                }
                chunk.emit(OpCode::Return, span.line);
            }

            Stmt::Expression { expr, span } => {
                self.compile_expr(expr, chunk);
                chunk.emit(OpCode::Pop, span.line);
            }

            Stmt::Assignment { target, value, span } => {
                self.compile_expr(value, chunk);
                if let Expr::Identifier { name, .. } = target {
                    chunk.emit(OpCode::Store(name.clone()), span.line);
                }
            }

            Stmt::If {
                condition,
                then_block,
                else_if_blocks,
                else_block,
                span,
            } => {
                self.compile_expr(condition, chunk);
                let jump_if_false = chunk.emit(OpCode::JumpIfFalse(0), span.line);
                self.compile_block(then_block, chunk);
                let jump_end = chunk.emit(OpCode::Jump(0), span.line);
                chunk.patch_jump(jump_if_false, chunk.len());

                let mut end_jumps: Vec<usize> = vec![jump_end];
                for (cond, block) in else_if_blocks {
                    self.compile_expr(cond, chunk);
                    let jif = chunk.emit(OpCode::JumpIfFalse(0), span.line);
                    self.compile_block(block, chunk);
                    let je = chunk.emit(OpCode::Jump(0), span.line);
                    end_jumps.push(je);
                    chunk.patch_jump(jif, chunk.len());
                }

                if let Some(block) = else_block {
                    self.compile_block(block, chunk);
                }

                let end_pos = chunk.len();
                for j in end_jumps {
                    chunk.patch_jump(j, end_pos);
                }
            }

            Stmt::Match { value, arms, span } => {
                self.compile_expr(value, chunk);
                let mut end_jumps: Vec<usize> = Vec::new();

                for arm in arms {
                    match &arm.pattern {
                        MatchPattern::Literal(lit_expr) => {
                            chunk.emit(OpCode::Dup, span.line);
                            self.compile_expr(lit_expr, chunk);
                            chunk.emit(OpCode::Equal, span.line);
                            let skip = chunk.emit(OpCode::JumpIfFalse(0), span.line);
                            chunk.emit(OpCode::Pop, span.line);
                            self.compile_block(&arm.body, chunk);
                            let end = chunk.emit(OpCode::Jump(0), span.line);
                            end_jumps.push(end);
                            chunk.patch_jump(skip, chunk.len());
                        }
                        MatchPattern::Default => {
                            chunk.emit(OpCode::Pop, span.line);
                            self.compile_block(&arm.body, chunk);
                        }
                        MatchPattern::Variant { .. } => {
                            self.compile_block(&arm.body, chunk);
                        }
                    }
                }

                let end_pos = chunk.len();
                for j in end_jumps {
                    chunk.patch_jump(j, end_pos);
                }
            }

            Stmt::Guard { condition, else_block, span } => {
                self.compile_expr(condition, chunk);
                let skip = chunk.emit(OpCode::JumpIfTrue(0), span.line);
                self.compile_block(else_block, chunk);
                chunk.patch_jump(skip, chunk.len());
            }

            Stmt::Break { .. } | Stmt::Continue { .. } => {
                chunk.emit(OpCode::Nop, 0);
            }
        }
    }

    fn compile_expr(&self, expr: &Expr, chunk: &mut Chunk) {
        match expr {
            Expr::IntLiteral { value, suffix, span } => {
                let val = match suffix.as_deref() {
                    Some("i8")  => Value::Int8(*value as i8),
                    Some("i16") => Value::Int16(*value as i16),
                    Some("i32") => Value::Int32(*value as i32),
                    Some("i64") => Value::Int64(*value as i64),
                    Some("u8")  => Value::UInt8(*value as u8),
                    Some("u16") => Value::UInt16(*value as u16),
                    Some("u32") => Value::UInt32(*value as u32),
                    Some("u64") => Value::UInt64(*value as u64),
                    None        => Value::Int64(*value as i64),
                    _           => Value::Int64(*value as i64),
                };
                chunk.emit(OpCode::PushConst(val), span.line);
            }

            Expr::FloatLiteral { value, suffix, span } => {
                let val = match suffix.as_deref() {
                    Some("f32") => Value::Float32(*value as f32),
                    _           => Value::Float64(*value),
                };
                chunk.emit(OpCode::PushConst(val), span.line);
            }

            Expr::StringLiteral { value, span } => {
                chunk.emit(OpCode::PushConst(Value::StringVal(value.clone())), span.line);
            }

            Expr::CharLiteral { value, span } => {
                chunk.emit(OpCode::PushConst(Value::Char(*value)), span.line);
            }

            Expr::BoolLiteral { value, span } => {
                chunk.emit(OpCode::PushConst(Value::Bool(*value)), span.line);
            }

            Expr::Identifier { name, span } => {
                chunk.emit(OpCode::Load(name.clone()), span.line);
            }

            Expr::SelfValue { span } => {
                chunk.emit(OpCode::Load("self".to_string()), span.line);
            }

            Expr::PathExpr { segments, span } => {
                // Path'i fonksiyon çağrısı gibi değerlendirme — sadece değer olarak push
                chunk.emit(OpCode::PushConst(Value::Path(segments.clone())), span.line);
            }

            Expr::Binary { left, op, right, span } => {
                self.compile_expr(left, chunk);
                self.compile_expr(right, chunk);
                let opcode = match op {
                    BinOp::Add   => OpCode::Add,
                    BinOp::Sub   => OpCode::Sub,
                    BinOp::Mul   => OpCode::Mul,
                    BinOp::Div   => OpCode::Div,
                    BinOp::Mod   => OpCode::Mod,
                    BinOp::Eq    => OpCode::Equal,
                    BinOp::NotEq => OpCode::NotEqual,
                    BinOp::Lt    => OpCode::LessThan,
                    BinOp::LtEq  => OpCode::LessEqual,
                    BinOp::Gt    => OpCode::GreaterThan,
                    BinOp::GtEq  => OpCode::GreaterEqual,
                    BinOp::And   => OpCode::And,
                    BinOp::Or    => OpCode::Or,
                };
                chunk.emit(opcode, span.line);
            }

            Expr::Unary { op, operand, span } => {
                self.compile_expr(operand, chunk);
                match op {
                    UnaryOp::Negate => { chunk.emit(OpCode::Negate, span.line); }
                    UnaryOp::Not    => { chunk.emit(OpCode::Not, span.line); }
                    _ => {}
                }
            }

            Expr::MethodCall { object, method, args, span } => {
                // 1. Object'i stack'e koy
                self.compile_expr(object, chunk);

                // 2. Sadece argümanların VALUE'larını stack'e koy
                //    (named arg label'ları ATLA)
                for arg in args {
                    self.compile_expr(&arg.value, chunk);
                }

                chunk.emit(OpCode::CallMethod(method.clone(), args.len()), span.line);
            }

            Expr::FnCall { callee, args, span } => {
                let fn_name = match callee.as_ref() {
                    Expr::Identifier { name, .. } => name.clone(),
                    Expr::PathExpr { segments, .. } => segments.join("::"),
                    _ => "unknown".to_string(),
                };

                // Sadece argümanların VALUE'larını stack'e koy
                for arg in args {
                    self.compile_expr(&arg.value, chunk);
                }

                if self.is_builtin_fn(&fn_name) {
                    chunk.emit(OpCode::CallBuiltin(fn_name, args.len()), span.line);
                } else {
                    chunk.emit(OpCode::Call(fn_name, args.len()), span.line);
                }
            }

            Expr::FieldAccess { object, field, span } => {
                self.compile_expr(object, chunk);
                chunk.emit(OpCode::GetField(field.clone()), span.line);
            }

            Expr::Grouped { inner, .. } => {
                self.compile_expr(inner, chunk);
            }

            Expr::WithErrorHandler { expr, .. } => {
                self.compile_expr(expr, chunk);
            }

            Expr::StructInit { type_name, fields, span } => {
                for (_, value) in fields {
                    self.compile_expr(value, chunk);
                }
                let name = type_name.join("::");
                chunk.emit(OpCode::MakeStruct(name, fields.len()), span.line);
            }
        }
    }

    fn is_builtin_fn(&self, name: &str) -> bool {
        matches!(
            name,
            "StreamWriter::acquire"
            | "StreamWriter::emit"
            | "StreamWriter::release"
            | "String::from"
            | "String::format"
            | "Vector::new"
            | "Vector::with_capacity"
            | "ExitCode::Success"
            | "ExitCode::Failure"
            | "panic!"
            | "assert"
            | "Checked::add"
        )
    }
}
