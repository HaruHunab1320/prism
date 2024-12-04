use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literals
    Float(f64),
    Integer(i64),
    String(String),
    Identifier(String),
    
    // Confidence operations
    ConfidenceValue {
        value: Box<Expr>,
    },
    ConfidenceFlow {
        source: Box<Expr>,
        target: Box<Expr>,
    },
    ReverseConfidenceFlow {
        target: Box<Expr>,
        source: Box<Expr>,
    },
    
    // Control flow
    UncertainIf {
        condition: Box<Expr>,
        high_confidence: Vec<Stmt>,
        medium_confidence: Option<Vec<Stmt>>,
        low_confidence: Option<Vec<Stmt>>,
    },
    
    // Context operations
    ContextBlock {
        context_name: String,
        body: Vec<Stmt>,
    },
    ContextShift {
        from: String,
        to: String,
        body: Vec<Stmt>,
    },
    
    // Verification
    Verify {
        sources: Vec<String>,
        threshold: f64,
        body: Vec<Stmt>,
    },
    
    // Binary operations
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
    },
    
    // Function-related
    FunctionCall {
        name: String,
        arguments: Vec<Expr>,
    },
    Return(Option<Box<Expr>>),
    
    // Array operations
    Array(Vec<Expr>),
    ArrayAccess {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    ArraySlice {
        array: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
    ArrayLength(Box<Expr>),
    ArrayPush {
        array: Box<Expr>,
        value: Box<Expr>,
    },
    ArrayPop(Box<Expr>),
    
    // Iterator-related expressions
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        step: Option<Box<Expr>>,
    },
    
    // Pattern matching
    Match {
        value: Box<Expr>,
        cases: Vec<MatchCase>,
    },
    
    // Error handling
    Try(Box<Expr>),
    Throw {
        error: Box<Expr>,
        confidence: Option<Box<Expr>>,
    },
    Error {
        message: String,
        code: Option<String>,
        confidence: Option<f64>,
        context: Option<String>,
    },
    
    // Struct-related expressions
    StructInstantiation {
        name: String,
        fields: Vec<(String, Box<Expr>)>,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    MethodCall {
        object: Box<Expr>,
        method: String,
        arguments: Vec<Expr>,
    },
    This,
    
    // Trait-related expressions
    DynamicMethodCall {
        object: Box<Expr>,
        trait_name: String,
        method: String,
        arguments: Vec<Expr>,
    },
    
    // Async-related expressions
    Await {
        expr: Box<Expr>,
        confidence: Option<f64>,
    },
    AsyncCall {
        function: Box<Expr>,
        arguments: Vec<Expr>,
        confidence: Option<f64>,
    },
    Promise {
        value: Box<Expr>,
        confidence: Option<f64>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    Declaration {
        name: String,
        value: Expr,
    },
    Assignment {
        target: String,
        value: Expr,
    },
    FunctionDefinition {
        name: String,
        parameters: Vec<String>,
        body: Vec<Stmt>,
        confidence_level: Option<f64>,
    },
    
    // Loop statements
    ForLoop {
        initializer: Option<Box<Stmt>>,
        condition: Option<Box<Expr>>,
        increment: Option<Box<Expr>>,
        body: Vec<Stmt>,
    },
    ForInLoop {
        variable: String,
        iterator: Box<Expr>,
        body: Vec<Stmt>,
    },
    WhileLoop {
        condition: Box<Expr>,
        body: Vec<Stmt>,
    },
    Break,
    Continue,
    
    // Error handling
    TryCatch {
        body: Vec<Stmt>,
        catches: Vec<CatchClause>,
        finally: Option<Vec<Stmt>>,
    },
    
    // Module-related statements
    Import(Import),
    Export(Export),
    Module {
        name: String,
        confidence: Option<f64>,
        body: Vec<Stmt>,
    },
    
    // Struct-related statements
    StructDefinition {
        name: String,
        type_parameters: Vec<TypeParameter>,
        confidence: Option<f64>,
        fields: Vec<StructField>,
        methods: Vec<StructMethod>,
    },
    
    // Trait-related statements
    TraitDefinition {
        name: String,
        type_parameters: Vec<TypeParameter>,
        confidence: Option<f64>,
        methods: Vec<TraitMethod>,
    },
    ImplTrait {
        trait_name: String,
        trait_type_args: Vec<Type>,
        struct_name: String,
        struct_type_args: Vec<Type>,
        confidence: Option<f64>,
        methods: Vec<StructMethod>,
    },
    
    // Async-related statements
    AsyncFunctionDefinition {
        name: String,
        type_parameters: Vec<TypeParameter>,
        parameters: Vec<(String, Type)>,
        return_type: Option<Type>,
        confidence: Option<f64>,
        body: Vec<Stmt>,
    },
    AsyncBlock {
        body: Vec<Stmt>,
        confidence: Option<f64>,
    },
    
    // Operator overloading
    OperatorDefinition {
        operator: Operator,
        lhs_type: Type,
        rhs_type: Option<Type>,
        return_type: Type,
        confidence: Option<f64>,
        body: Vec<Stmt>,
    },
    
    // Macro definitions
    MacroDefinition {
        name: String,
        parameters: Vec<String>,
        body: Vec<MacroToken>,
        confidence: Option<f64>,
    },
    
    // Macro invocation
    MacroInvoke {
        name: String,
        arguments: Vec<Expr>,
        confidence: Option<f64>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    ConfidenceAnd,
    ConfidenceOr,
    Add,
    Subtract,
    Multiply,
    Divide,
    ArrayConcat,
    ArrayEquals,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(Value),
    Variable(String),
    Array(Vec<Pattern>),
    Rest(Box<Pattern>),
    Confidence {
        value: Box<Pattern>,
        range: (f64, f64),
    },
    Context {
        name: String,
        pattern: Box<Pattern>,
    },
    Or(Vec<Pattern>),
    And(Vec<Pattern>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchCase {
    pattern: Pattern,
    guard: Option<Box<Expr>>,
    body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatchClause {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,
    pub body: Vec<Stmt>,
    pub confidence_adjustment: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub name: String,
    pub confidence: Option<f64>,
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Import {
    All {
        from: String,
        confidence: Option<f64>,
    },
    Named {
        names: Vec<String>,
        from: String,
        confidence: Option<f64>,
    },
    Aliased {
        name: String,
        as_name: String,
        from: String,
        confidence: Option<f64>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Export {
    Named(String),
    All,
    Default(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub type_annotation: Type,
    pub confidence: Option<f64>,
    pub default_value: Option<Box<Expr>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructMethod {
    pub name: String,
    pub type_parameters: Vec<TypeParameter>,
    pub parameters: Vec<(String, Type)>,
    pub return_type: Option<Type>,
    pub confidence: Option<f64>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethod {
    pub name: String,
    pub confidence: Option<f64>,
    pub parameters: Vec<String>,
    pub body: Option<Vec<Stmt>>,  // None for abstract methods, Some for default implementations
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeParameter {
    pub name: String,
    pub bounds: Vec<TraitBound>,
    pub confidence_bound: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitBound {
    pub trait_name: String,
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Simple(String),
    Generic {
        base: String,
        type_args: Vec<Type>,
    },
    Array(Box<Type>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add,      // +
    Sub,      // -
    Mul,      // *
    Div,      // /
    Eq,       // ==
    NotEq,    // !=
    Lt,       // <
    Gt,       // >
    LtEq,     // <=
    GtEq,     // >=
    And,      // &&
    Or,       // ||
    Not,      // !
    Neg,      // - (unary)
    ConfFlow, // ~>
}

#[derive(Debug, Clone, PartialEq)]
pub enum MacroToken {
    // Literal tokens
    Literal(String),
    
    // Parameter substitution
    Parameter(String),
    
    // Special macro operators
    Concat,           // ##
    Stringify,        // #
    
    // Confidence-related tokens
    ConfidenceOf(String),  // confidence($param)
    ConfidenceFlow,        // ~>
    
    // Control flow in macros
    If {
        condition: Box<MacroExpr>,
        then_tokens: Vec<MacroToken>,
        else_tokens: Option<Vec<MacroToken>>,
    },
    
    // Repetition
    Repeat {
        pattern: Vec<MacroToken>,
        separator: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MacroExpr {
    // Literal values
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    
    // Parameter reference
    Parameter(String),
    
    // Operators
    BinaryOp {
        op: MacroOperator,
        left: Box<MacroExpr>,
        right: Box<MacroExpr>,
    },
    
    // Type checking
    TypeCheck {
        value: Box<MacroExpr>,
        type_name: String,
    },
    
    // Confidence comparison
    ConfidenceCheck {
        value: Box<MacroExpr>,
        threshold: f64,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MacroOperator {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
    Lt,
    Gt,
    And,
    Or,
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Float(val) => write!(f, "{}", val),
            Expr::Integer(val) => write!(f, "{}", val),
            Expr::String(val) => write!(f, "\"{}\"", val),
            Expr::Identifier(name) => write!(f, "{}", name),
            Expr::ConfidenceValue { value } => write!(f, "conf {}", value),
            Expr::ConfidenceFlow { source, target } => write!(f, "{} ~> {}", source, target),
            Expr::ReverseConfidenceFlow { target, source } => write!(f, "{} <~ {}", target, source),
            Expr::FunctionCall { name, arguments } => {
                write!(f, "{}(", name)?;
                for (i, arg) in arguments.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            },
            Expr::Return(Some(expr)) => write!(f, "return {}", expr),
            Expr::Return(None) => write!(f, "return"),
            Expr::Array(elements) => {
                write!(f, "[")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, "]")
            },
            Expr::ArrayAccess { array, index } => write!(f, "{}[{}]", array, index),
            Expr::ArraySlice { array, start, end } => {
                write!(f, "{}[", array)?;
                if let Some(start) = start {
                    write!(f, "{}", start)?;
                }
                write!(f, ":")?;
                if let Some(end) = end {
                    write!(f, "{}", end)?;
                }
                write!(f, "]")
            },
            Expr::ArrayLength(array) => write!(f, "{}.length", array),
            Expr::ArrayPush { array, value } => write!(f, "{}.push({})", array, value),
            Expr::ArrayPop(array) => write!(f, "{}.pop()", array),
            Expr::Range { start, end, step } => {
                write!(f, "{}:{}", start, end)?;
                if let Some(step) = step {
                    write!(f, ":{}", step)?;
                }
                Ok(())
            },
            Expr::Match { value, cases } => {
                writeln!(f, "match {} {{", value)?;
                for case in cases {
                    write!(f, "    {} ", case.pattern)?;
                    if let Some(guard) = &case.guard {
                        write!(f, "if {} ", guard)?;
                    }
                    writeln!(f, "=> {{")?;
                    for stmt in &case.body {
                        writeln!(f, "        {}", stmt)?;
                    }
                    writeln!(f, "    }},")?;
                }
                write!(f, "}}")
            },
            Expr::Try(expr) => write!(f, "try {}", expr),
            Expr::Throw { error, confidence } => {
                write!(f, "throw ")?;
                if let Some(conf) = confidence {
                    write!(f, "~{} ", conf)?;
                }
                write!(f, "{}", error)
            },
            Expr::Error { message, code, confidence, context } => {
                write!(f, "error(\"{}\"")?;
                if let Some(code) = code {
                    write!(f, ", code: {}", code)?;
                }
                if let Some(conf) = confidence {
                    write!(f, ", confidence: {}", conf)?;
                }
                if let Some(ctx) = context {
                    write!(f, ", context: {}", ctx)?;
                }
                write!(f, ")")
            },
            Expr::StructInstantiation { name, fields } => {
                write!(f, "{} {{ ", name)?;
                for (i, (field_name, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", field_name, value)?;
                }
                write!(f, " }}")
            },
            Expr::FieldAccess { object, field } => {
                write!(f, "{}.{}", object, field)
            },
            Expr::MethodCall { object, method, arguments } => {
                write!(f, "{}.{}(", object, method)?;
                for (i, arg) in arguments.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            },
            Expr::This => write!(f, "this"),
            Expr::DynamicMethodCall { object, trait_name, method, arguments } => {
                write!(f, "{}::{}.{}(", trait_name, object, method)?;
                for (i, arg) in arguments.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            },
            Expr::Await { expr, confidence } => {
                write!(f, "await ")?;
                if let Some(conf) = confidence {
                    write!(f, "~{} ", conf)?;
                }
                write!(f, "{}", expr)
            },
            Expr::AsyncCall { function, arguments, confidence } => {
                write!(f, "async ")?;
                if let Some(conf) = confidence {
                    write!(f, "~{} ", conf)?;
                }
                write!(f, "{}(", function)?;
                for (i, arg) in arguments.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            },
            Expr::Promise { value, confidence } => {
                write!(f, "promise ")?;
                if let Some(conf) = confidence {
                    write!(f, "~{} ", conf)?;
                }
                write!(f, "{}", value)
            },
            _ => write!(f, "{:?}", self),
        }
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Expression(expr) => write!(f, "{}", expr),
            Stmt::Declaration { name, value } => write!(f, "conf {} = {}", name, value),
            Stmt::Assignment { target, value } => write!(f, "{} = {}", target, value),
            Stmt::FunctionDefinition { name, parameters, confidence_level, .. } => {
                if let Some(conf) = confidence_level {
                    write!(f, "fn {}({}) ~{}", name, parameters.join(", "), conf)
                } else {
                    write!(f, "fn {}({})", name, parameters.join(", "))
                }
            },
            Stmt::ForLoop { initializer, condition, increment, .. } => {
                write!(f, "for (")?;
                if let Some(init) = initializer {
                    write!(f, "{}", init)?;
                }
                write!(f, "; ")?;
                if let Some(cond) = condition {
                    write!(f, "{}", cond)?;
                }
                write!(f, "; ")?;
                if let Some(inc) = increment {
                    write!(f, "{}", inc)?;
                }
                write!(f, ")")
            },
            Stmt::ForInLoop { variable, iterator, .. } => {
                write!(f, "for {} in {}", variable, iterator)
            },
            Stmt::WhileLoop { condition, .. } => {
                write!(f, "while {}", condition)
            },
            Stmt::Break => write!(f, "break"),
            Stmt::Continue => write!(f, "continue"),
            Stmt::TryCatch { body, catches, finally } => {
                writeln!(f, "try {{")?;
                for stmt in body {
                    writeln!(f, "    {}", stmt)?;
                }
                writeln!(f, "}}")?;
                
                for catch in catches {
                    write!(f, "catch {} ", catch.pattern)?;
                    if let Some(guard) = &catch.guard {
                        write!(f, "if {} ", guard)?;
                    }
                    if let Some(conf) = catch.confidence_adjustment {
                        write!(f, "~{} ", conf)?;
                    }
                    writeln!(f, "{{")?;
                    for stmt in &catch.body {
                        writeln!(f, "    {}", stmt)?;
                    }
                    writeln!(f, "}}")?;
                }
                
                if let Some(finally_block) = finally {
                    writeln!(f, "finally {{")?;
                    for stmt in finally_block {
                        writeln!(f, "    {}", stmt)?;
                    }
                    writeln!(f, "}}")?;
                }
                Ok(())
            },
            Stmt::Import(import) => write!(f, "{}", import),
            Stmt::Export(export) => write!(f, "{}", export),
            Stmt::Module { name, confidence, body } => {
                write!(f, "module {} ", name)?;
                if let Some(conf) = confidence {
                    write!(f, "~{} ", conf)?;
                }
                writeln!(f, "{{")?;
                for stmt in body {
                    writeln!(f, "    {}", stmt)?;
                }
                write!(f, "}}")
            },
            Stmt::StructDefinition { name, type_parameters, confidence, fields, methods } => {
                write!(f, "struct {}", name)?;
                if !type_parameters.is_empty() {
                    write!(f, "<")?;
                    for (i, param) in type_parameters.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", param)?;
                    }
                    write!(f, ">")?;
                }
                if let Some(conf) = confidence {
                    write!(f, " ~{}", conf)?;
                }
                writeln!(f, " {{")?;
                
                // Fields
                for field in fields {
                    write!(f, "    {}: {}", field.name, field.type_annotation)?;
                    if let Some(conf) = field.confidence {
                        write!(f, " ~{}", conf)?;
                    }
                    if let Some(default) = &field.default_value {
                        write!(f, " = {}", default)?;
                    }
                    writeln!(f, ",")?;
                }
                
                if !fields.is_empty() && !methods.is_empty() {
                    writeln!(f)?;
                }
                
                // Methods
                for method in methods {
                    write!(f, "    fn {}", method.name)?;
                    if !method.type_parameters.is_empty() {
                        write!(f, "<")?;
                        for (i, param) in method.type_parameters.iter().enumerate() {
                            if i > 0 {
                                write!(f, ", ")?;
                            }
                            write!(f, "{}", param)?;
                        }
                        write!(f, ">")?;
                    }
                    write!(f, "(")?;
                    for (i, (param, ty)) in method.parameters.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}: {}", param, ty)?;
                    }
                    write!(f, ")")?;
                    if let Some(ret_ty) = &method.return_type {
                        write!(f, " -> {}", ret_ty)?;
                    }
                    if let Some(conf) = method.confidence {
                        write!(f, " ~{}", conf)?;
                    }
                    writeln!(f, " {{")?;
                    for stmt in &method.body {
                        writeln!(f, "        {}", stmt)?;
                    }
                    writeln!(f, "    }}")?;
                }
                
                write!(f, "}}")
            },
            Stmt::TraitDefinition { name, type_parameters, confidence, methods } => {
                write!(f, "trait {}", name)?;
                if !type_parameters.is_empty() {
                    write!(f, "<")?;
                    for (i, param) in type_parameters.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", param)?;
                    }
                    write!(f, ">")?;
                }
                if let Some(conf) = confidence {
                    write!(f, " ~{}", conf)?;
                }
                writeln!(f, " {{")?;
                
                for method in methods {
                    write!(f, "    fn {}", method.name)?;
                    if !method.type_parameters.is_empty() {
                        write!(f, "<")?;
                        for (i, param) in method.type_parameters.iter().enumerate() {
                            if i > 0 {
                                write!(f, ", ")?;
                            }
                            write!(f, "{}", param)?;
                        }
                        write!(f, ">")?;
                    }
                    write!(f, "(")?;
                    for (i, (param, ty)) in method.parameters.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}: {}", param, ty)?;
                    }
                    write!(f, ")")?;
                    if let Some(ret_ty) = &method.return_type {
                        write!(f, " -> {}", ret_ty)?;
                    }
                    if let Some(conf) = method.confidence {
                        write!(f, " ~{}", conf)?;
                    }
                    
                    if let Some(body) = &method.body {
                        writeln!(f, " {{")?;
                        for stmt in body {
                            writeln!(f, "        {}", stmt)?;
                        }
                        writeln!(f, "    }}")?;
                    } else {
                        writeln!(f, ";")?;
                    }
                }
                
                write!(f, "}}")
            },
            Stmt::ImplTrait { trait_name, trait_type_args, struct_name, struct_type_args, confidence, methods } => {
                write!(f, "impl {}", trait_name)?;
                if !trait_type_args.is_empty() {
                    write!(f, "<")?;
                    for (i, arg) in trait_type_args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", arg)?;
                    }
                    write!(f, ">")?;
                }
                write!(f, " for {}", struct_name)?;
                if !struct_type_args.is_empty() {
                    write!(f, "<")?;
                    for (i, arg) in struct_type_args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", arg)?;
                    }
                    write!(f, ">")?;
                }
                if let Some(conf) = confidence {
                    write!(f, " ~{}", conf)?;
                }
                writeln!(f, " {{")?;
                
                for method in methods {
                    write!(f, "    fn {}", method.name)?;
                    if !method.type_parameters.is_empty() {
                        write!(f, "<")?;
                        for (i, param) in method.type_parameters.iter().enumerate() {
                            if i > 0 {
                                write!(f, ", ")?;
                            }
                            write!(f, "{}", param)?;
                        }
                        write!(f, ">")?;
                    }
                    write!(f, "(")?;
                    for (i, (param, ty)) in method.parameters.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}: {}", param, ty)?;
                    }
                    write!(f, ")")?;
                    if let Some(ret_ty) = &method.return_type {
                        write!(f, " -> {}", ret_ty)?;
                    }
                    if let Some(conf) = method.confidence {
                        write!(f, " ~{}", conf)?;
                    }
                    writeln!(f, " {{")?;
                    for stmt in &method.body {
                        writeln!(f, "        {}", stmt)?;
                    }
                    writeln!(f, "    }}")?;
                }
                
                write!(f, "}}")
            },
            Stmt::AsyncFunctionDefinition { name, type_parameters, parameters, return_type, confidence, body } => {
                write!(f, "async fn {}", name)?;
                if !type_parameters.is_empty() {
                    write!(f, "<")?;
                    for (i, param) in type_parameters.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", param)?;
                    }
                    write!(f, ">")?;
                }
                write!(f, "(")?;
                for (i, (param, ty)) in parameters.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", param, ty)?;
                }
                write!(f, ")")?;
                if let Some(ret_ty) = return_type {
                    write!(f, " -> {}", ret_ty)?;
                }
                if let Some(conf) = confidence {
                    write!(f, " ~{}", conf)?;
                }
                writeln!(f, " {{")?;
                for stmt in body {
                    writeln!(f, "    {}", stmt)?;
                }
                write!(f, "}}")
            },
            Stmt::AsyncBlock { body, confidence } => {
                write!(f, "async")?;
                if let Some(conf) = confidence {
                    write!(f, " ~{}", conf)?;
                }
                writeln!(f, " {{")?;
                for stmt in body {
                    writeln!(f, "    {}", stmt)?;
                }
                write!(f, "}}")
            },
            Stmt::OperatorDefinition { operator, lhs_type, rhs_type, return_type, confidence, body } => {
                write!(f, "operator {} for {} ", operator, lhs_type)?;
                if let Some(rhs) = rhs_type {
                    write!(f, "{} ", rhs)?;
                }
                write!(f, "-> {}", return_type)?;
                if let Some(conf) = confidence {
                    write!(f, " ~{}", conf)?;
                }
                writeln!(f, " {{")?;
                for stmt in body {
                    writeln!(f, "    {}", stmt)?;
                }
                write!(f, "}}")
            },
            Stmt::MacroDefinition { name, parameters, body, confidence } => {
                write!(f, "macro {} ", name)?;
                if !parameters.is_empty() {
                    write!(f, "<")?;
                    for (i, param) in parameters.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", param)?;
                    }
                    write!(f, ">")?;
                }
                if let Some(conf) = confidence {
                    write!(f, " ~{}", conf)?;
                }
                writeln!(f, " {{")?;
                for token in body {
                    writeln!(f, "    {}", token)?;
                }
                write!(f, "}}")
            },
            Stmt::MacroInvoke { name, arguments, confidence } => {
                write!(f, "{} ", name)?;
                if !arguments.is_empty() {
                    write!(f, "(")?;
                    for (i, arg) in arguments.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", arg)?;
                    }
                    write!(f, ")")?;
                }
                if let Some(conf) = confidence {
                    write!(f, " ~{}", conf)?;
                }
                Ok(())
            },
        }
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pattern::Literal(val) => write!(f, "{}", val),
            Pattern::Variable(name) => write!(f, "{}", name),
            Pattern::Array(patterns) => {
                write!(f, "[")?;
                for (i, p) in patterns.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, "]")
            },
            Pattern::Rest(pattern) => write!(f, "...{}", pattern),
            Pattern::Confidence { value, range } => {
                write!(f, "{} ~{{{}, {}}}", value, range.0, range.1)
            },
            Pattern::Context { name, pattern } => {
                write!(f, "in {} {}", name, pattern)
            },
            Pattern::Or(patterns) => {
                for (i, p) in patterns.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", p)?;
                }
                Ok(())
            },
            Pattern::And(patterns) => {
                for (i, p) in patterns.iter().enumerate() {
                    if i > 0 {
                        write!(f, " & ")?;
                    }
                    write!(f, "{}", p)?;
                }
                Ok(())
            },
        }
    }
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "module {} ", self.name)?;
        if let Some(conf) = self.confidence {
            writeln!(f, "~{}", conf)?;
        }
        writeln!(f, "{{")?;
        
        for import in &self.imports {
            writeln!(f, "    {}", import)?;
        }
        
        if !self.imports.is_empty() {
            writeln!(f)?;
        }
        
        for export in &self.exports {
            writeln!(f, "    {}", export)?;
        }
        
        if !self.exports.is_empty() {
            writeln!(f)?;
        }
        
        for stmt in &self.body {
            writeln!(f, "    {}", stmt)?;
        }
        
        write!(f, "}}")
    }
}

impl fmt::Display for Import {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Import::All { from, confidence } => {
                write!(f, "import * from \"{}\"", from)?;
                if let Some(conf) = confidence {
                    write!(f, " ~{}", conf)?;
                }
                Ok(())
            },
            Import::Named { names, from, confidence } => {
                write!(f, "import {{ {} }} from \"{}\"", names.join(", "), from)?;
                if let Some(conf) = confidence {
                    write!(f, " ~{}", conf)?;
                }
                Ok(())
            },
            Import::Aliased { name, as_name, from, confidence } => {
                write!(f, "import {} as {} from \"{}\"", name, as_name, from)?;
                if let Some(conf) = confidence {
                    write!(f, " ~{}", conf)?;
                }
                Ok(())
            },
        }
    }
}

impl fmt::Display for Export {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Export::Named(name) => write!(f, "export {}", name),
            Export::All => write!(f, "export *"),
            Export::Default(expr) => write!(f, "export default {}", expr),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Simple(name) => write!(f, "{}", name),
            Type::Generic { base, type_args } => {
                write!(f, "{}<", base)?;
                for (i, arg) in type_args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ">")
            },
            Type::Array(elem_type) => write!(f, "[{}]", elem_type),
        }
    }
}

impl fmt::Display for TypeParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.bounds.is_empty() {
            write!(f, ": ")?;
            for (i, bound) in self.bounds.iter().enumerate() {
                if i > 0 {
                    write!(f, " + ")?;
                }
                write!(f, "{}", bound)?;
            }
        }
        if let Some(conf) = self.confidence_bound {
            write!(f, " ~{}", conf)?;
        }
        Ok(())
    }
}

impl fmt::Display for TraitBound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.trait_name)?;
        if let Some(conf) = self.confidence {
            write!(f, " ~{}", conf)?;
        }
        Ok(())
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operator::Add => write!(f, "+"),
            Operator::Sub => write!(f, "-"),
            Operator::Mul => write!(f, "*"),
            Operator::Div => write!(f, "/"),
            Operator::Eq => write!(f, "=="),
            Operator::NotEq => write!(f, "!="),
            Operator::Lt => write!(f, "<"),
            Operator::Gt => write!(f, ">"),
            Operator::LtEq => write!(f, "<="),
            Operator::GtEq => write!(f, ">="),
            Operator::And => write!(f, "&&"),
            Operator::Or => write!(f, "||"),
            Operator::Not => write!(f, "!"),
            Operator::Neg => write!(f, "-"),
            Operator::ConfFlow => write!(f, "~>"),
        }
    }
}

impl fmt::Display for MacroToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MacroToken::Literal(s) => write!(f, "{}", s),
            MacroToken::Parameter(name) => write!(f, "${}", name),
            MacroToken::Concat => write!(f, "##"),
            MacroToken::Stringify => write!(f, "#"),
            MacroToken::ConfidenceOf(param) => write!(f, "confidence({})", param),
            MacroToken::ConfidenceFlow => write!(f, "~>"),
            MacroToken::If { condition, then_tokens, else_tokens } => {
                write!(f, "if {} {{", condition)?;
                for token in then_tokens {
                    write!(f, " {}", token)?;
                }
                if let Some(else_tokens) = else_tokens {
                    write!(f, " }} else {{")?;
                    for token in else_tokens {
                        write!(f, " {}", token)?;
                    }
                }
                write!(f, " }}")
            },
            MacroToken::Repeat { pattern, separator } => {
                write!(f, "repeat(")?;
                for (i, token) in pattern.iter().enumerate() {
                    if i > 0 {
                        if let Some(sep) = separator {
                            write!(f, "{}", sep)?;
                        }
                    }
                    write!(f, "{}", token)?;
                }
                write!(f, ")")
            },
        }
    }
}

impl fmt::Display for MacroExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MacroExpr::Integer(i) => write!(f, "{}", i),
            MacroExpr::Float(x) => write!(f, "{}", x),
            MacroExpr::String(s) => write!(f, "\"{}\"", s),
            MacroExpr::Boolean(b) => write!(f, "{}", b),
            MacroExpr::Parameter(name) => write!(f, "${}", name),
            MacroExpr::BinaryOp { op, left, right } => write!(f, "({} {} {})", left, op, right),
            MacroExpr::TypeCheck { value, type_name } => write!(f, "typeof({}) == \"{}\"", value, type_name),
            MacroExpr::ConfidenceCheck { value, threshold } => write!(f, "confidence({}) > {}", value, threshold),
        }
    }
}

impl fmt::Display for MacroOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MacroOperator::Add => write!(f, "+"),
            MacroOperator::Sub => write!(f, "-"),
            MacroOperator::Mul => write!(f, "*"),
            MacroOperator::Div => write!(f, "/"),
            MacroOperator::Eq => write!(f, "=="),
            MacroOperator::NotEq => write!(f, "!="),
            MacroOperator::Lt => write!(f, "<"),
            MacroOperator::Gt => write!(f, ">"),
            MacroOperator::And => write!(f, "&&"),
            MacroOperator::Or => write!(f, "||"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_flow_expr() {
        let expr = Expr::ConfidenceFlow {
            source: Box::new(Expr::Identifier("x".to_string())),
            target: Box::new(Expr::Float(0.7)),
        };
        assert_eq!(expr.to_string(), "x ~> 0.7");
    }

    #[test]
    fn test_declaration_stmt() {
        let stmt = Stmt::Declaration {
            name: "x".to_string(),
            value: Expr::Float(0.8),
        };
        assert_eq!(stmt.to_string(), "conf x = 0.8");
    }

    #[test]
    fn test_function_definition() {
        let stmt = Stmt::FunctionDefinition {
            name: "test".to_string(),
            parameters: vec!["x".to_string(), "y".to_string()],
            body: vec![],
            confidence_level: Some(0.8),
        };
        assert_eq!(stmt.to_string(), "fn test(x, y) ~0.8");
    }

    #[test]
    fn test_function_call() {
        let expr = Expr::FunctionCall {
            name: "test".to_string(),
            arguments: vec![Expr::Float(0.5), Expr::Integer(42)],
        };
        assert_eq!(expr.to_string(), "test(0.5, 42)");
    }
} 