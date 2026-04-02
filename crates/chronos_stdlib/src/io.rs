use chronos_vm::Value;

pub struct ChronosIO;

impl ChronosIO {
    /// writer.emit() — değeri string'e çevirip döndürür
    pub fn emit(args: &[Value]) -> Value {
        let mut parts: Vec<String> = Vec::new();
        for arg in args {
            let text = Self::value_to_string(arg);
            parts.push(text);
        }
        Value::StringVal(parts.join(""))
    }

    /// String::format(template, args...) implementasyonu
    pub fn format(template: &str, args: &[Value]) -> Value {
        let mut result = template.to_string();
        for arg in args {
            if let Some(pos) = result.find("{}") {
                let replacement = Self::value_to_string(arg);
                result.replace_range(pos..pos + 2, &replacement);
            }
        }
        Value::StringVal(result)
    }

    /// Herhangi bir Value'yu okunabilir string'e çevir
    pub fn value_to_string(val: &Value) -> String {
        match val {
            Value::Int8(n)      => format!("{}", n),
            Value::Int16(n)     => format!("{}", n),
            Value::Int32(n)     => format!("{}", n),
            Value::Int64(n)     => format!("{}", n),
            Value::UInt8(n)     => format!("{}", n),
            Value::UInt16(n)    => format!("{}", n),
            Value::UInt32(n)    => format!("{}", n),
            Value::UInt64(n)    => format!("{}", n),
            Value::Float32(n)   => format!("{}", n),
            Value::Float64(n)   => format!("{}", n),
            Value::Bool(b)      => format!("{}", b),
            Value::Char(c)      => format!("{}", c),
            Value::StringVal(s) => s.clone(),
            Value::Void         => "Void".to_string(),
            Value::None         => "None".to_string(),
            Value::Path(p)      => p.join("::"),
            Value::Struct { type_name, fields } => {
                let fs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, Self::value_to_string(v)))
                    .collect();
                format!("{} {{ {} }}", type_name, fs.join(", "))
            }
        }
    }

    /// Tip bilgisini string olarak döndür
    pub fn type_of(val: &Value) -> Value {
        Value::StringVal(val.type_name().to_string())
    }

    /// Debug representation
    pub fn debug_repr(val: &Value) -> Value {
        Value::StringVal(format!("{:?}", val))
    }
}
