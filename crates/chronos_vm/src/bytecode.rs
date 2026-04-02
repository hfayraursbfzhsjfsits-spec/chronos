use std::fmt;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Value — VM'deki runtime değerler
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Float32(f32),
    Float64(f64),
    Bool(bool),
    Char(char),
    StringVal(String),
    Void,
    /// Path değeri — ExitCode::Success gibi
    Path(Vec<String>),
    /// Struct instance
    Struct {
        type_name: String,
        fields: Vec<(String, Value)>,
    },
    /// Null/None placeholder
    None,
}

impl Value {
    pub fn type_name(&self) -> &str {
        match self {
            Value::Int8(_)      => "Int8",
            Value::Int16(_)     => "Int16",
            Value::Int32(_)     => "Int32",
            Value::Int64(_)     => "Int64",
            Value::UInt8(_)     => "UInt8",
            Value::UInt16(_)    => "UInt16",
            Value::UInt32(_)    => "UInt32",
            Value::UInt64(_)    => "UInt64",
            Value::Float32(_)   => "Float32",
            Value::Float64(_)   => "Float64",
            Value::Bool(_)      => "Bool",
            Value::Char(_)      => "Char",
            Value::StringVal(_) => "String",
            Value::Void         => "Void",
            Value::Path(_)      => "Path",
            Value::Struct { type_name, .. } => type_name,
            Value::None         => "None",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int32(n) => *n != 0,
            Value::Int64(n) => *n != 0,
            Value::StringVal(s) => !s.is_empty(),
            Value::None | Value::Void => false,
            _ => true,
        }
    }

    pub fn to_i64(&self) -> Option<i64> {
        match self {
            Value::Int8(n)   => Some(*n as i64),
            Value::Int16(n)  => Some(*n as i64),
            Value::Int32(n)  => Some(*n as i64),
            Value::Int64(n)  => Some(*n),
            Value::UInt8(n)  => Some(*n as i64),
            Value::UInt16(n) => Some(*n as i64),
            Value::UInt32(n) => Some(*n as i64),
            Value::UInt64(n) => Some(*n as i64),
            _ => None,
        }
    }

    pub fn to_f64(&self) -> Option<f64> {
        match self {
            Value::Float32(n) => Some(*n as f64),
            Value::Float64(n) => Some(*n),
            _ => self.to_i64().map(|n| n as f64),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int8(n)      => write!(f, "{}i8", n),
            Value::Int16(n)     => write!(f, "{}i16", n),
            Value::Int32(n)     => write!(f, "{}i32", n),
            Value::Int64(n)     => write!(f, "{}i64", n),
            Value::UInt8(n)     => write!(f, "{}u8", n),
            Value::UInt16(n)    => write!(f, "{}u16", n),
            Value::UInt32(n)    => write!(f, "{}u32", n),
            Value::UInt64(n)    => write!(f, "{}u64", n),
            Value::Float32(n)   => write!(f, "{}f32", n),
            Value::Float64(n)   => write!(f, "{}f64", n),
            Value::Bool(b)      => write!(f, "{}", b),
            Value::Char(c)      => write!(f, "'{}'", c),
            Value::StringVal(s) => write!(f, "{}", s),
            Value::Void         => write!(f, "Void"),
            Value::Path(p)      => write!(f, "{}", p.join("::")),
            Value::Struct { type_name, fields } => {
                let fs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                write!(f, "{}{{ {} }}", type_name, fs.join(", "))
            }
            Value::None => write!(f, "None"),
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  OpCode — bytecode komutları
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    /// Sabit değeri stack'e koy
    PushConst(Value),

    /// Değişkeni stack'e yükle
    Load(String),

    /// Stack'ten değeri değişkene kaydet
    Store(String),

    /// Mutable değişken tanımla
    StoreMut(String),

    // ── Aritmetik ──
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Negate,

    // ── Karşılaştırma ──
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,

    // ── Mantıksal ──
    Not,
    And,
    Or,

    // ── Kontrol Akışı ──
    /// Koşulsuz jump
    Jump(usize),
    /// Koşullu jump — stack'teki değer false ise atla
    JumpIfFalse(usize),
    /// Koşullu jump — stack'teki değer true ise atla
    JumpIfTrue(usize),

    // ── Fonksiyon ──
    /// Built-in fonksiyon çağır (isim, argüman sayısı)
    CallBuiltin(String, usize),
    /// Kullanıcı fonksiyon çağır (isim, argüman sayısı)
    Call(String, usize),
    /// Method çağır (method_name, arg_count)
    CallMethod(String, usize),
    /// Fonksiyondan dön
    Return,

    // ── Stack ──
    /// Stack'in tepesini at
    Pop,
    /// Stack'in tepesini kopyala
    Dup,

    // ── Struct ──
    /// Struct oluştur (type_name, field_count)
    MakeStruct(String, usize),
    /// Field'a eriş
    GetField(String),
    /// Field'a ata
    SetField(String),

    // ── Özel ──
    /// Programı durdur
    Halt,
    /// Hiçbir şey yapma
    Nop,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Chunk — bytecode komutları listesi
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone)]
pub struct Chunk {
    pub name: String,
    pub params: Vec<String>,
    pub code: Vec<OpCode>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new(name: String) -> Self {
        Self {
            name,
            params: Vec::new(),
            code: Vec::new(),
            lines: Vec::new(),
        }
    }
    
    pub fn with_params(name: String, params: Vec<String>) -> Self {
    Self {
        name,
        params,
        code: Vec::new(),
        lines: Vec::new(),
    }
} 

    pub fn emit(&mut self, op: OpCode, line: usize) -> usize {
        let idx = self.code.len();
        self.code.push(op);
        self.lines.push(line);
        idx
    }

    pub fn patch_jump(&mut self, idx: usize, target: usize) {
        match &mut self.code[idx] {
            OpCode::Jump(ref mut t) => *t = target,
            OpCode::JumpIfFalse(ref mut t) => *t = target,
            OpCode::JumpIfTrue(ref mut t) => *t = target,
            _ => panic!("Cannot patch non-jump instruction"),
        }
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }

    pub fn disassemble(&self) -> String {
        let mut output = format!(
    "═══ Chunk: {} | params: ({}) ═══\n",
    self.name,
    self.params.join(", ")
);
        for (i, op) in self.code.iter().enumerate() {
            let line = self.lines.get(i).unwrap_or(&0);
            output.push_str(&format!("  {:04}  L{:<4}  {:?}\n", i, line, op));
        }
        output
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  CompiledProgram — tüm fonksiyonların bytecode'u
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone)]
pub struct CompiledProgram {
    pub chunks: Vec<Chunk>,
    pub entry_point: Option<String>,
}

impl CompiledProgram {
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            entry_point: None,
        }
    }

    pub fn add_chunk(&mut self, chunk: Chunk) {
        self.chunks.push(chunk);
    }

    pub fn find_chunk(&self, name: &str) -> Option<&Chunk> {
        self.chunks.iter().find(|c| c.name == name)
    }

    pub fn disassemble_all(&self) -> String {
        let mut output = String::new();
        for chunk in &self.chunks {
            output.push_str(&chunk.disassemble());
            output.push('\n');
        }
        output
    }
}
