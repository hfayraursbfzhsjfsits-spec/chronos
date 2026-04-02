use chronos_lexer::Span;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum SemanticError {
    // ── Type Errors ──
    #[error("[S0001] Type mismatch: expected '{expected}', found '{found}' at {span}")]
    TypeMismatch {
        expected: String,
        found: String,
        span: Span,
    },

    #[error("[S0002] Cannot apply operator '{op}' to types '{left}' and '{right}' at {span}")]
    InvalidOperator {
        op: String,
        left: String,
        right: String,
        span: Span,
    },

    // ── Scope Errors ──
    #[error("[S0003] Undefined variable '{name}' at {span}")]
    UndefinedVariable {
        name: String,
        span: Span,
    },

    #[error("[S0004] Undefined function '{name}' at {span}")]
    UndefinedFunction {
        name: String,
        span: Span,
    },

    #[error("[S0005] Undefined contract '{name}' at {span}")]
    UndefinedContract {
        name: String,
        span: Span,
    },

    #[error("[S0006] Variable '{name}' already declared in this scope at {span}")]
    AlreadyDeclared {
        name: String,
        span: Span,
    },

    // ── Mutability Errors ──
    #[error("[S0007] Cannot assign to immutable variable '{name}' at {span} — declare with 'let mut'")]
    ImmutableAssignment {
        name: String,
        span: Span,
    },

    #[error("[S0008] Cannot take mutable reference to immutable variable '{name}' at {span}")]
    ImmutableBorrow {
        name: String,
        span: Span,
    },

    // ── Function Errors ──
    #[error("[S0009] Function '{name}' expects {expected} arguments, found {found} at {span}")]
    ArgumentCountMismatch {
        name: String,
        expected: usize,
        found: usize,
        span: Span,
    },

    #[error("[S0010] Missing return statement in function '{name}' — expected return type '{return_type}' at {span}")]
    MissingReturn {
        name: String,
        return_type: String,
        span: Span,
    },

    #[error("[S0011] Return type mismatch in function '{name}': expected '{expected}', found '{found}' at {span}")]
    ReturnTypeMismatch {
        name: String,
        expected: String,
        found: String,
        span: Span,
    },

    // ── Contract Errors ──
    #[error("[S0012] Field '{field}' not found in contract '{contract}' at {span}")]
    UndefinedField {
        field: String,
        contract: String,
        span: Span,
    },

    #[error("[S0013] Method '{method}' not found in contract '{contract}' at {span}")]
    UndefinedMethod {
        method: String,
        contract: String,
        span: Span,
    },

    // ── General ──
    #[error("[S0014] Break statement outside of loop at {span}")]
    BreakOutsideLoop {
        span: Span,
    },

    #[error("[S0015] Continue statement outside of loop at {span}")]
    ContinueOutsideLoop {
        span: Span,
    },

    #[error("[W0001] Unused variable '{name}' at line {line} — prefix with '_' to suppress")]
    UnusedVariable {
        name: String,
        line: usize,
    },
}

impl SemanticError {
    pub fn is_warning(&self) -> bool {
        matches!(self, SemanticError::UnusedVariable { .. })
    }
}
