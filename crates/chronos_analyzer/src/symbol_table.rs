use crate::types::ChronosType;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: ChronosType,
    pub kind: SymbolKind,
    pub mutable: bool,
    pub initialized: bool,
    pub used: bool,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable,
    Parameter,
    Function,
    Method,
    Field,
    Contract,
    Enumeration,
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub params: Vec<(String, ChronosType)>,
    pub return_type: ChronosType,
    pub annotations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ContractInfo {
    pub name: String,
    pub traits: Vec<String>,
    pub fields: Vec<(String, ChronosType)>,
    pub methods: Vec<FunctionSignature>,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub symbols: HashMap<String, Symbol>,
    pub scope_type: ScopeType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScopeType {
    Global,
    Contract(String),
    Function(String),
    Block,
    Loop,
}

impl Scope {
    pub fn new(scope_type: ScopeType) -> Self {
        Self {
            symbols: HashMap::new(),
            scope_type,
        }
    }

    pub fn define(&mut self, symbol: Symbol) -> Option<Symbol> {
        self.symbols.insert(symbol.name.clone(), symbol)
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        self.symbols.get_mut(name)
    }
}

#[derive(Debug)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
    pub contracts: HashMap<String, ContractInfo>,
    pub functions: HashMap<String, FunctionSignature>,
}

impl SymbolTable {
    pub fn new() -> Self {
        let global = Scope::new(ScopeType::Global);
        Self {
            scopes: vec![global],
            contracts: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn push_scope(&mut self, scope_type: ScopeType) {
        self.scopes.push(Scope::new(scope_type));
    }

    pub fn pop_scope(&mut self) -> Option<Scope> {
        if self.scopes.len() > 1 {
            self.scopes.pop()
        } else {
            None
        }
    }

    pub fn define(&mut self, symbol: Symbol) -> Option<Symbol> {
        if let Some(scope) = self.scopes.last_mut() {
            scope.define(symbol)
        } else {
            None
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.lookup(name) {
                return Some(sym);
            }
        }
        None
    }

    pub fn lookup_current_scope(&self, name: &str) -> Option<&Symbol> {
        self.scopes.last()?.lookup(name)
    }

    pub fn mark_used(&mut self, name: &str) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(sym) = scope.lookup_mut(name) {
                sym.used = true;
                return;
            }
        }
    }

    pub fn current_scope_type(&self) -> &ScopeType {
        &self.scopes.last().unwrap().scope_type
    }

    pub fn enclosing_function(&self) -> Option<&str> {
        for scope in self.scopes.iter().rev() {
            if let ScopeType::Function(name) = &scope.scope_type {
                return Some(name.as_str());
            }
        }
        None
    }

    pub fn enclosing_contract(&self) -> Option<&str> {
        for scope in self.scopes.iter().rev() {
            if let ScopeType::Contract(name) = &scope.scope_type {
                return Some(name.as_str());
            }
        }
        None
    }

    pub fn register_contract(&mut self, info: ContractInfo) {
        self.contracts.insert(info.name.clone(), info);
    }

    pub fn get_contract(&self, name: &str) -> Option<&ContractInfo> {
        self.contracts.get(name)
    }

    pub fn register_function(&mut self, sig: FunctionSignature) {
        self.functions.insert(sig.name.clone(), sig);
    }

    pub fn find_unused_symbols(&self) -> Vec<&Symbol> {
        let mut unused = Vec::new();
        for scope in &self.scopes {
            for sym in scope.symbols.values() {
                if !sym.used
                    && sym.kind == SymbolKind::Variable
                    && !sym.name.starts_with('_')
                {
                    unused.push(sym);
                }
            }
        }
        unused
    }

    pub fn depth(&self) -> usize {
        self.scopes.len()
    }

    pub fn is_inside_loop(&self) -> bool {
        for scope in self.scopes.iter().rev() {
            if matches!(scope.scope_type, ScopeType::Loop) {
                return true;
            }
        }
        false
    }
}
