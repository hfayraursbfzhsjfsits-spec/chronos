use chronos_vm::Value;

pub struct ChronosMath;

impl ChronosMath {
    pub fn checked_add(left: &Value, right: &Value) -> Value {
        match (left, right) {
            (Value::Int32(a), Value::Int32(b)) => {
                match a.checked_add(*b) {
                    Some(result) => Value::Int32(result),
                    None => Value::None, // overflow
                }
            }
            (Value::Int64(a), Value::Int64(b)) => {
                match a.checked_add(*b) {
                    Some(result) => Value::Int64(result),
                    None => Value::None,
                }
            }
            _ => Value::None,
        }
    }

    /// Absolute value
    pub fn abs(val: &Value) -> Value {
        match val {
            Value::Int32(n) => Value::Int32(n.abs()),
            Value::Int64(n) => Value::Int64(n.abs()),
            Value::Float32(n) => Value::Float32(n.abs()),
            Value::Float64(n) => Value::Float64(n.abs()),
            _ => Value::None,
        }
    }

    /// Power
    pub fn pow(base: &Value, exp: &Value) -> Value {
        match (base.to_f64(), exp.to_f64()) {
            (Some(b), Some(e)) => Value::Float64(b.powf(e)),
            _ => Value::None,
        }
    }

    /// Square root
    pub fn sqrt(val: &Value) -> Value {
        match val.to_f64() {
            Some(n) if n >= 0.0 => Value::Float64(n.sqrt()),
            _ => Value::None,
        }
    }

    /// Min of two values
    pub fn min(a: &Value, b: &Value) -> Value {
        match (a.to_f64(), b.to_f64()) {
            (Some(x), Some(y)) => {
                if x <= y {
                    a.clone()
                } else {
                    b.clone()
                }
            }
            _ => Value::None,
        }
    }

    /// Max of two values
    pub fn max(a: &Value, b: &Value) -> Value {
        match (a.to_f64(), b.to_f64()) {
            (Some(x), Some(y)) => {
                if x >= y {
                    a.clone()
                } else {
                    b.clone()
                }
            }
            _ => Value::None,
        }
    }
}
