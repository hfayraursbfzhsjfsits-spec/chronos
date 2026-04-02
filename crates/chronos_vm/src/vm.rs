use crate::bytecode::*;
use crate::errors::VMError;
use std::collections::HashMap;

pub struct VM {
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    call_stack: Vec<CallFrame>,
    output: Vec<String>,
    halted: bool,
}

#[derive(Debug, Clone)]
struct CallFrame {
    chunk_name: String,
    ip: usize,
    locals: HashMap<String, Variable>,
}

#[derive(Debug, Clone)]
struct Variable {
    value: Value,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            globals: HashMap::new(),
            call_stack: Vec::new(),
            output: Vec::new(),
            halted: false,
        }
    }

    pub fn run(&mut self, program: &CompiledProgram) -> Result<Value, VMError> {
        let entry = program.entry_point.as_deref().unwrap_or("main");

        let entry_chunk = program
            .find_chunk(&format!("Main::{}", entry))
            .or_else(|| program.find_chunk(entry))
            .ok_or_else(|| VMError::ChunkNotFound {
                name: entry.to_string(),
            })?;

        self.call_stack.push(CallFrame {
            chunk_name: entry_chunk.name.clone(),
            ip: 0,
            locals: HashMap::new(),
        });

        self.execute_loop(program)
    }

    fn execute_loop(&mut self, program: &CompiledProgram) -> Result<Value, VMError> {
        let mut iterations: u64 = 0;
        let limit: u64 = 1_000_000;

        while !self.halted && !self.call_stack.is_empty() {
            iterations += 1;
            if iterations > limit {
                return Err(VMError::InfiniteLoop);
            }

            let frame = self.call_stack.last().unwrap();
            let chunk_name = frame.chunk_name.clone();
            let ip = frame.ip;

            let chunk = match program.find_chunk(&chunk_name) {
                Some(c) => c,
                None => {
                    self.call_stack.pop();
                    continue;
                }
            };

            if ip >= chunk.code.len() {
                self.call_stack.pop();
                continue;
            }

            let op = chunk.code[ip].clone();
            self.call_stack.last_mut().unwrap().ip += 1;

            self.execute_op(op, program)?;
        }

        if !self.stack.is_empty() {
            Ok(self.safe_pop())
        } else {
            Ok(Value::Void)
        }
    }

    fn execute_op(&mut self, op: OpCode, program: &CompiledProgram) -> Result<(), VMError> {
        match op {
            OpCode::PushConst(val) => {
                self.stack.push(val);
            }

            OpCode::Load(name) => {
                let val = self.load_variable(&name);
                self.stack.push(val);
            }

            OpCode::Store(name) => {
                let val = self.safe_pop();
                self.store_variable(name, val);
            }

            OpCode::StoreMut(name) => {
                let val = self.safe_pop();
                self.store_variable(name, val);
            }

            OpCode::Add => {
                let right = self.safe_pop();
                let left = self.safe_pop();
                self.stack.push(self.arithmetic(&left, &right, "add"));
            }

            OpCode::Sub => {
                let right = self.safe_pop();
                let left = self.safe_pop();
                self.stack.push(self.arithmetic(&left, &right, "sub"));
            }

            OpCode::Mul => {
                let right = self.safe_pop();
                let left = self.safe_pop();
                self.stack.push(self.arithmetic(&left, &right, "mul"));
            }

            OpCode::Div => {
                let right = self.safe_pop();
                let left = self.safe_pop();
                self.stack.push(self.arithmetic(&left, &right, "div"));
            }

            OpCode::Mod => {
                let right = self.safe_pop();
                let left = self.safe_pop();
                self.stack.push(self.arithmetic(&left, &right, "mod"));
            }

            OpCode::Negate => {
                let val = self.safe_pop();
                let result = match val {
                    Value::Int32(n) => Value::Int32(-n),
                    Value::Int64(n) => Value::Int64(-n),
                    Value::Float32(n) => Value::Float32(-n),
                    Value::Float64(n) => Value::Float64(-n),
                    other => other,
                };
                self.stack.push(result);
            }

            OpCode::Equal => {
                let right = self.safe_pop();
                let left = self.safe_pop();
                self.stack.push(Value::Bool(left == right));
            }

            OpCode::NotEqual => {
                let right = self.safe_pop();
                let left = self.safe_pop();
                self.stack.push(Value::Bool(left != right));
            }

            OpCode::LessThan => {
                let right = self.safe_pop();
                let left = self.safe_pop();
                self.stack.push(Value::Bool(self.compare(&left, &right, "lt")));
            }

            OpCode::LessEqual => {
                let right = self.safe_pop();
                let left = self.safe_pop();
                self.stack.push(Value::Bool(self.compare(&left, &right, "le")));
            }

            OpCode::GreaterThan => {
                let right = self.safe_pop();
                let left = self.safe_pop();
                self.stack.push(Value::Bool(self.compare(&left, &right, "gt")));
            }

            OpCode::GreaterEqual => {
                let right = self.safe_pop();
                let left = self.safe_pop();
                self.stack.push(Value::Bool(self.compare(&left, &right, "ge")));
            }

            OpCode::Not => {
                let val = self.safe_pop();
                self.stack.push(Value::Bool(!val.is_truthy()));
            }

            OpCode::And => {
                let right = self.safe_pop();
                let left = self.safe_pop();
                self.stack.push(Value::Bool(left.is_truthy() && right.is_truthy()));
            }

            OpCode::Or => {
                let right = self.safe_pop();
                let left = self.safe_pop();
                self.stack.push(Value::Bool(left.is_truthy() || right.is_truthy()));
            }

            OpCode::Jump(target) => {
                self.call_stack.last_mut().unwrap().ip = target;
            }

            OpCode::JumpIfFalse(target) => {
                let val = self.safe_pop();
                if !val.is_truthy() {
                    self.call_stack.last_mut().unwrap().ip = target;
                }
            }

            OpCode::JumpIfTrue(target) => {
                let val = self.safe_pop();
                if val.is_truthy() {
                    self.call_stack.last_mut().unwrap().ip = target;
                }
            }

            OpCode::CallBuiltin(name, arg_count) => {
                let args = self.pop_n(arg_count);
                let result = self.exec_builtin(&name, args);
                self.stack.push(result);
            }

            OpCode::Call(name, arg_count) => {
                let args = self.pop_n(arg_count);

                if self.is_builtin(&name) {
                    let result = self.exec_builtin(&name, args);
                    self.stack.push(result);
                } else if let Some(chunk) = program.find_chunk(&name) {
                    let mut locals = HashMap::new();

                    for (param_name, arg_value) in chunk.params.iter().cloned().zip(args.into_iter()) {
                        locals.insert(param_name, Variable { value: arg_value });
                    }

                    self.call_stack.push(CallFrame {
                        chunk_name: name,
                        ip: 0,
                        locals,
                    });
                } else {
                    self.stack.push(Value::Void);
                }
            }

            OpCode::CallMethod(method, arg_count) => {
                let args = self.pop_n(arg_count);
                let object = self.safe_pop();
                let result = self.exec_method(&object, &method, args);
                self.stack.push(result);
            }

            OpCode::Return => {
                let return_val = if !self.stack.is_empty() {
                    self.safe_pop()
                } else {
                    Value::Void
                };

                self.call_stack.pop();

                if self.call_stack.is_empty() {
                    self.halted = true;
                    self.stack.push(return_val);
                    return Ok(());
                }

                self.stack.push(return_val);
            }

            OpCode::Pop => {
                self.safe_pop();
            }

            OpCode::Dup => {
                let val = self.stack.last().cloned().unwrap_or(Value::Void);
                self.stack.push(val);
            }

            OpCode::MakeStruct(name, field_count) => {
                let values = self.pop_n(field_count);
                let fields: Vec<(String, Value)> = values
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| (format!("field_{}", i), v))
                    .collect();

                self.stack.push(Value::Struct {
                    type_name: name,
                    fields,
                });
            }

            OpCode::GetField(field) => {
                let obj = self.safe_pop();
                match obj {
                    Value::Struct { fields, .. } => {
                        let val = fields
                            .iter()
                            .find(|(k, _)| k == &field)
                            .map(|(_, v)| v.clone())
                            .unwrap_or(Value::None);
                        self.stack.push(val);
                    }
                    _ => self.stack.push(Value::None),
                }
            }

            OpCode::SetField(field) => {
                let val = self.safe_pop();
                let obj = self.safe_pop();
                match obj {
                    Value::Struct { type_name, mut fields } => {
                        if let Some(entry) = fields.iter_mut().find(|(k, _)| k == &field) {
                            entry.1 = val;
                        }
                        self.stack.push(Value::Struct { type_name, fields });
                    }
                    _ => self.stack.push(obj),
                }
            }

            OpCode::Halt => {
                self.halted = true;
            }

            OpCode::Nop => {}
        }

        Ok(())
    }

    fn is_builtin(&self, name: &str) -> bool {
        name.contains("::")
            || matches!(name, "panic!" | "assert" | "print" | "println")
    }

    fn exec_builtin(&mut self, name: &str, args: Vec<Value>) -> Value {
        match name {
            "StreamWriter::acquire" => {
                Value::Struct {
                    type_name: "StreamWriter".to_string(),
                    fields: vec![("target".to_string(), Value::StringVal("StdOut".to_string()))],
                }
            }

            "StreamWriter::release" => Value::Void,

            "String::from" => {
                args.into_iter().next().unwrap_or(Value::StringVal(String::new()))
            }

            "String::format" => {
                if let Some(Value::StringVal(template)) = args.first() {
                    let mut result = template.clone();
                    for arg in args.iter().skip(1) {
                        if let Some(pos) = result.find("{}") {
                            result.replace_range(pos..pos + 2, &format!("{}", arg));
                        }
                    }
                    Value::StringVal(result)
                } else {
                    Value::StringVal(String::new())
                }
            }

            "ExitCode::Success" => {
                let code = args.first().and_then(|v| v.to_i64()).unwrap_or(0);
                Value::Path(vec!["ExitCode".to_string(), format!("Success({})", code)])
            }

            "ExitCode::Failure" => {
                let code = args.first().and_then(|v| v.to_i64()).unwrap_or(1);
                Value::Path(vec!["ExitCode".to_string(), format!("Failure({})", code)])
            }

            _ => Value::Void,
        }
    }

    fn exec_method(&mut self, _object: &Value, method: &str, args: Vec<Value>) -> Value {
        match method {
            "emit" => {
                for arg in &args {
                    let text = match arg {
                        Value::StringVal(s) => s.clone(),
                        other => format!("{}", other),
                    };
                    self.output.push(text);
                }
                Value::Void
            }

            "acquire" => {
                Value::Struct {
                    type_name: "StreamWriter".to_string(),
                    fields: vec![],
                }
            }

            "release" => Value::Void,

            "clone" => args.into_iter().next().unwrap_or(Value::Void),

            "length" | "len" => Value::UInt64(0),

            "push" => Value::Void,

            "get" => Value::Int64(0),

            "from" => {
                args.into_iter().next().unwrap_or(Value::Void)
            }

            "format" => {
                if let Some(Value::StringVal(template)) = args.first() {
                    let mut result = template.clone();
                    for arg in args.iter().skip(1) {
                        if let Some(pos) = result.find("{}") {
                            result.replace_range(pos..pos + 2, &format!("{}", arg));
                        }
                    }
                    Value::StringVal(result)
                } else {
                    Value::StringVal(String::new())
                }
            }

            "new" | "with_capacity" => {
                Value::Struct {
                    type_name: "Collection".to_string(),
                    fields: vec![],
                }
            }

            "Success" => {
                let code = args.first().and_then(|v| v.to_i64()).unwrap_or(0);
                Value::Path(vec!["ExitCode".to_string(), format!("Success({})", code)])
            }

            "Failure" => {
                let code = args.first().and_then(|v| v.to_i64()).unwrap_or(1);
                Value::Path(vec!["ExitCode".to_string(), format!("Failure({})", code)])
            }

            _ => Value::Void,
        }
    }

    fn safe_pop(&mut self) -> Value {
        self.stack.pop().unwrap_or(Value::Void)
    }

    fn pop_n(&mut self, n: usize) -> Vec<Value> {
        let mut result = Vec::with_capacity(n);
        for _ in 0..n {
            result.push(self.safe_pop());
        }
        result.reverse();
        result
    }

    fn load_variable(&self, name: &str) -> Value {
        for frame in self.call_stack.iter().rev() {
            if let Some(var) = frame.locals.get(name) {
                return var.value.clone();
            }
        }

        if let Some(val) = self.globals.get(name) {
            return val.clone();
        }

        Value::Path(vec![name.to_string()])
    }

    fn store_variable(&mut self, name: String, value: Value) {
        if let Some(frame) = self.call_stack.last_mut() {
            frame.locals.insert(name, Variable { value });
        } else {
            self.globals.insert(name, value);
        }
    }

    fn arithmetic(&self, left: &Value, right: &Value, op: &str) -> Value {
        if let (Value::Int32(l), Value::Int32(r)) = (left, right) {
            return match op {
                "add" => Value::Int32(l.wrapping_add(*r)),
                "sub" => Value::Int32(l.wrapping_sub(*r)),
                "mul" => Value::Int32(l.wrapping_mul(*r)),
                "div" => {
                    if *r != 0 { Value::Int32(l / r) } else { Value::Int32(0) }
                }
                "mod" => {
                    if *r != 0 { Value::Int32(l % r) } else { Value::Int32(0) }
                }
                _ => Value::Int32(0),
            };
        }

        if let (Some(l), Some(r)) = (left.to_i64(), right.to_i64()) {
            return match op {
                "add" => Value::Int64(l.wrapping_add(r)),
                "sub" => Value::Int64(l.wrapping_sub(r)),
                "mul" => Value::Int64(l.wrapping_mul(r)),
                "div" => {
                    if r != 0 { Value::Int64(l / r) } else { Value::Int64(0) }
                }
                "mod" => {
                    if r != 0 { Value::Int64(l % r) } else { Value::Int64(0) }
                }
                _ => Value::Int64(0),
            };
        }

        if let (Some(l), Some(r)) = (left.to_f64(), right.to_f64()) {
            return match op {
                "add" => Value::Float64(l + r),
                "sub" => Value::Float64(l - r),
                "mul" => Value::Float64(l * r),
                "div" => Value::Float64(l / r),
                "mod" => Value::Float64(l % r),
                _ => Value::Float64(0.0),
            };
        }

        if let (Value::StringVal(l), Value::StringVal(r)) = (left, right) {
            if op == "add" {
                return Value::StringVal(format!("{}{}", l, r));
            }
        }

        Value::Void
    }

    fn compare(&self, left: &Value, right: &Value, op: &str) -> bool {
        if let (Some(l), Some(r)) = (left.to_f64(), right.to_f64()) {
            return match op {
                "lt" => l < r,
                "le" => l <= r,
                "gt" => l > r,
                "ge" => l >= r,
                _ => false,
            };
        }

        false
    }

    pub fn get_output(&self) -> &[String] {
        &self.output
    }
}
