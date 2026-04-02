use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum VMError {
    #[error("[VM0001] Stack underflow — attempted to pop from empty stack")]
    StackUnderflow,

    #[error("[VM0002] Division by zero")]
    DivisionByZero,

    #[error("[VM0003] Type error: {message}")]
    TypeError { message: String },

    #[error("[VM0004] Undefined variable: '{name}'")]
    UndefinedVariable { name: String },

    #[error("[VM0005] Chunk not found: '{name}'")]
    ChunkNotFound { name: String },

    #[error("[VM0006] Panic: {message}")]
    Panic { message: String },

    #[error("[VM0007] Infinite loop detected — exceeded iteration limit")]
    InfiniteLoop,

    #[error("[VM0008] Immutable variable: '{name}' cannot be reassigned")]
    ImmutableVariable { name: String },
}
