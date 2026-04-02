use std::fmt;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  CHRONOS Type System
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone, PartialEq)]
pub enum ChronosType {
    // ── Primitive Types ──
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    Bool,
    Char,
    StringType,
    Void,

    // ── Complex Types ──
    /// Vector<T>, Optional<T>, etc.
    Generic {
        base: String,
        type_args: Vec<ChronosType>,
    },

    /// Result<T, E>
    Result {
        ok_type: Box<ChronosType>,
        err_type: Box<ChronosType>,
    },

    /// Optional<T>
    Optional {
        inner: Box<ChronosType>,
    },

    /// &T
    Reference {
        mutable: bool,
        inner: Box<ChronosType>,
    },

    /// Tuple<A, B, C>
    Tuple {
        elements: Vec<ChronosType>,
    },

    /// Closure<(A, B) -> R>
    Closure {
        params: Vec<ChronosType>,
        return_type: Box<ChronosType>,
    },

    /// User-defined contract type
    Contract(String),

    /// User-defined enum type
    Enum(String),

    /// Self keyword — contract içinde
    SelfType,

    /// Henüz çözümlenmemiş tip
    Unresolved(String),

    /// Hata durumunda kullanılan placeholder
    Error,
}

impl ChronosType {
    /// Primitive mi?
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            ChronosType::Int8 | ChronosType::Int16 |
            ChronosType::Int32 | ChronosType::Int64 |
            ChronosType::UInt8 | ChronosType::UInt16 |
            ChronosType::UInt32 | ChronosType::UInt64 |
            ChronosType::Float32 | ChronosType::Float64 |
            ChronosType::Bool | ChronosType::Char |
            ChronosType::StringType | ChronosType::Void
        )
    }

    /// Numeric mi?
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            ChronosType::Int8 | ChronosType::Int16 |
            ChronosType::Int32 | ChronosType::Int64 |
            ChronosType::UInt8 | ChronosType::UInt16 |
            ChronosType::UInt32 | ChronosType::UInt64 |
            ChronosType::Float32 | ChronosType::Float64
        )
    }

    /// Integer mi?
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            ChronosType::Int8 | ChronosType::Int16 |
            ChronosType::Int32 | ChronosType::Int64 |
            ChronosType::UInt8 | ChronosType::UInt16 |
            ChronosType::UInt32 | ChronosType::UInt64
        )
    }

    /// Float mi?
    pub fn is_float(&self) -> bool {
        matches!(self, ChronosType::Float32 | ChronosType::Float64)
    }

    /// İki tip uyumlu mu?
    pub fn is_assignable_from(&self, other: &ChronosType) -> bool {
        // Error tipi her şeyle uyumlu (hata zaten raporlandı)
        if matches!(self, ChronosType::Error) || matches!(other, ChronosType::Error) {
            return true;
        }

        // Birebir aynı tip
        if self == other {
            return true;
        }

        // Unresolved tip — henüz çözümlenmemiş, kabul et
        if matches!(self, ChronosType::Unresolved(_))
            || matches!(other, ChronosType::Unresolved(_))
        {
            return true;
        }

        // Contract tipi — isim eşleşmesi
        match (self, other) {
            (ChronosType::Contract(a), ChronosType::Contract(b)) => a == b,
            (ChronosType::Enum(a), ChronosType::Enum(b)) => a == b,
            (ChronosType::Generic { base: a, type_args: args_a },
             ChronosType::Generic { base: b, type_args: args_b }) => {
                a == b
                    && args_a.len() == args_b.len()
                    && args_a.iter().zip(args_b.iter()).all(|(x, y)| x.is_assignable_from(y))
            }
            (ChronosType::Reference { mutable: m1, inner: i1 },
             ChronosType::Reference { mutable: m2, inner: i2 }) => {
                // &mut T, &T'ye atanabilir ama tersi olmaz
                (*m1 || !*m2) && i1.is_assignable_from(i2)
            }
            _ => false,
        }
    }

    /// String'den tipe dönüştür
    pub fn from_name(name: &str) -> ChronosType {
        match name {
            "Int8"    => ChronosType::Int8,
            "Int16"   => ChronosType::Int16,
            "Int32"   => ChronosType::Int32,
            "Int64"   => ChronosType::Int64,
            "UInt8"   => ChronosType::UInt8,
            "UInt16"  => ChronosType::UInt16,
            "UInt32"  => ChronosType::UInt32,
            "UInt64"  => ChronosType::UInt64,
            "Float32" => ChronosType::Float32,
            "Float64" => ChronosType::Float64,
            "Bool"    => ChronosType::Bool,
            "Char"    => ChronosType::Char,
            "String"  => ChronosType::StringType,
            "Void"    => ChronosType::Void,
            "Self"    => ChronosType::SelfType,
            other     => ChronosType::Unresolved(other.to_string()),
        }
    }
}

impl fmt::Display for ChronosType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChronosType::Int8       => write!(f, "Int8"),
            ChronosType::Int16      => write!(f, "Int16"),
            ChronosType::Int32      => write!(f, "Int32"),
            ChronosType::Int64      => write!(f, "Int64"),
            ChronosType::UInt8      => write!(f, "UInt8"),
            ChronosType::UInt16     => write!(f, "UInt16"),
            ChronosType::UInt32     => write!(f, "UInt32"),
            ChronosType::UInt64     => write!(f, "UInt64"),
            ChronosType::Float32    => write!(f, "Float32"),
            ChronosType::Float64    => write!(f, "Float64"),
            ChronosType::Bool       => write!(f, "Bool"),
            ChronosType::Char       => write!(f, "Char"),
            ChronosType::StringType => write!(f, "String"),
            ChronosType::Void       => write!(f, "Void"),
            ChronosType::SelfType   => write!(f, "Self"),
            ChronosType::Error      => write!(f, "<error>"),
            ChronosType::Contract(name) => write!(f, "{}", name),
            ChronosType::Enum(name) => write!(f, "{}", name),
            ChronosType::Unresolved(name) => write!(f, "{}", name),
            ChronosType::Generic { base, type_args } => {
                write!(f, "{}<{}>", base,
                    type_args.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", "))
            }
            ChronosType::Result { ok_type, err_type } => {
                write!(f, "Result<{}, {}>", ok_type, err_type)
            }
            ChronosType::Optional { inner } => {
                write!(f, "Optional<{}>", inner)
            }
            ChronosType::Reference { mutable, inner } => {
                if *mutable { write!(f, "&mut {}", inner) }
                else { write!(f, "&{}", inner) }
            }
            ChronosType::Tuple { elements } => {
                write!(f, "Tuple<{}>",
                    elements.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", "))
            }
            ChronosType::Closure { params, return_type } => {
                write!(f, "Closure<({}) -> {}>",
                    params.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", "),
                    return_type)
            }
        }
    }
}
