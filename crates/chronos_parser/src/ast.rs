use chronos_lexer::Span;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Program — kök düğüm
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone)]
pub struct Program {
    pub module_directives: Vec<ModuleDirective>,
    pub require_statements: Vec<RequireStatement>,
    pub declarations: Vec<Declaration>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Module Directive — #![module::entry(main)]
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone)]
pub struct ModuleDirective {
    pub path: Vec<String>,   // ["module", "entry"]
    pub value: String,       // "main"
    pub span: Span,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Require Statement — @require core::io::{ ... };
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone)]
pub struct RequireStatement {
    pub module_path: Vec<String>,  // ["core", "io"]
    pub imports: Vec<String>,      // ["StreamWriter", "BufferMode"]
    pub span: Span,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Top-Level Declarations
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone)]
pub enum Declaration {
    Contract(ContractDecl),
    Function(FnDecl),
    Enumeration(EnumDecl),
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Contract Declaration
//  contract Main :: EntryPoint, Displayable { ... }
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone)]
pub struct ContractDecl {
    pub name: String,
    pub traits: Vec<String>,
    pub members: Vec<ContractMember>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ContractMember {
    Field(FieldDecl),
    Method(FnDecl),
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Field Declaration
//  @visibility(private) field name: String;
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub type_expr: TypeExpr,
    pub span: Span,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Function Declaration
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub annotations: Vec<Annotation>,
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: TypeExpr,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_expr: TypeExpr,
    pub span: Span,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Enumeration Declaration
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    pub traits: Vec<String>,
    pub variants: Vec<EnumVariant>,
    pub methods: Vec<FnDecl>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<FieldDecl>,
    pub span: Span,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Annotation — @static, @visibility(public), etc.
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone)]
pub struct Annotation {
    pub name: String,
    pub args: Vec<Expr>,
    pub span: Span,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Type Expressions
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone)]
pub enum TypeExpr {
    /// Int32, String, Bool, Void, etc.
    Named(String, Span),

    /// Vector<Int32>, Result<String, Error>
    Generic {
        base: String,
        type_args: Vec<TypeExpr>,
        span: Span,
    },

    /// &String, &mut Int32
    Reference {
        mutable: bool,
        inner: Box<TypeExpr>,
        span: Span,
    },

    /// Tuple<Int32, String>
    Tuple {
        elements: Vec<TypeExpr>,
        span: Span,
    },

    /// Closure<(Int32) -> Int32>
    Closure {
        params: Vec<TypeExpr>,
        return_type: Box<TypeExpr>,
        span: Span,
    },

    /// Self keyword
    SelfType(Span),
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Statements
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pub type Block = Vec<Stmt>;

#[derive(Debug, Clone)]
pub enum Stmt {
    /// let x: Int32 = 42i32;
    Let {
        mutable: bool,
        name: String,
        type_expr: TypeExpr,
        initializer: Expr,
        error_handler: Option<ErrorHandler>,
        span: Span,
    },

    /// return ExitCode::Success(0x00);
    Return {
        value: Option<Expr>,
        span: Span,
    },

    /// guard condition else { ... };
    Guard {
        condition: Expr,
        else_block: Block,
        span: Span,
    },

    /// if (condition: ...) => { } else => { };
    If {
        condition: Expr,
        then_block: Block,
        else_if_blocks: Vec<(Expr, Block)>,
        else_block: Option<Block>,
        span: Span,
    },

    /// match (value: x) => { case(...) => { }, ... };
    Match {
        value: Expr,
        arms: Vec<MatchArm>,
        span: Span,
    },

    /// break / continue
    Break { span: Span },
    Continue { span: Span },

    /// Expression statement — foo.bar();
    Expression {
        expr: Expr,
        span: Span,
    },

    /// Assignment — x = 10;
    Assignment {
        target: Expr,
        value: Expr,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum MatchPattern {
    /// case(42i32)
    Literal(Expr),
    /// case(Self::ConnectionRefused(h, p, _))
    Variant {
        path: Vec<String>,
        bindings: Vec<String>,
    },
    /// default
    Default,
}

#[derive(Debug, Clone)]
pub struct ErrorHandler {
    pub param: String,
    pub body: Block,
    pub span: Span,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Expressions
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone)]
pub enum Expr {
    /// 42i32, 3.14f64
    IntLiteral {
        value: i128,
        suffix: Option<String>,
        span: Span,
    },
    FloatLiteral {
        value: f64,
        suffix: Option<String>,
        span: Span,
    },

    /// "hello"
    StringLiteral {
        value: String,
        span: Span,
    },

    /// 'A'
    CharLiteral {
        value: char,
        span: Span,
    },

    /// true / false veya Bool::True / Bool::False
    BoolLiteral {
        value: bool,
        span: Span,
    },

    /// x, name, my_var
    Identifier {
        name: String,
        span: Span,
    },

    /// Self
    SelfValue {
        span: Span,
    },

    /// ExitCode::Success  veya  core::io::StreamWriter
    PathExpr {
        segments: Vec<String>,
        span: Span,
    },

    /// x + y, a > b, etc.
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
        span: Span,
    },

    /// !x, -y, &x, &mut x, *x
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
        span: Span,
    },

    /// StreamWriter::acquire(target: StdOut, mode: ...)
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<NamedArg>,
        span: Span,
    },

    /// foo(bar, baz)
    FnCall {
        callee: Box<Expr>,
        args: Vec<NamedArg>,
        span: Span,
    },

    /// object.field
    FieldAccess {
        object: Box<Expr>,
        field: String,
        span: Span,
    },

    /// expr => |err| { ... }
    WithErrorHandler {
        expr: Box<Expr>,
        handler: ErrorHandler,
        span: Span,
    },

    /// Self { name: name, age: age }
    StructInit {
        type_name: Vec<String>,
        fields: Vec<(String, Expr)>,
        span: Span,
    },

    /// Grouped: (expr)
    Grouped {
        inner: Box<Expr>,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub struct NamedArg {
    pub name: Option<String>,  // None ise positional
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, NotEq, Lt, LtEq, Gt, GtEq,
    And, Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Negate,     // -
    Not,        // !
    Ref,        // &
    RefMut,     // &mut
    Deref,      // *
}
