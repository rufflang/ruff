// File: src/ast.rs
//
// Abstract Syntax Tree (AST) definitions for the Ruff programming language.
// Defines the structure of parsed Ruff programs.
//
// The AST represents the syntactic structure of Ruff code after parsing.
// Expressions (Expr) represent values and computations, while Statements (Stmt)
// represent actions and control flow.

use crate::errors::SourceLocation;

/// Special method names for operator overloading
/// These methods can be defined on structs to customize operator behavior
pub mod operator_methods {
    // Arithmetic operators
    pub const ADD: &str = "op_add";
    pub const SUB: &str = "op_sub";
    pub const MUL: &str = "op_mul";
    pub const DIV: &str = "op_div";
    pub const MOD: &str = "op_mod";

    // Comparison operators
    pub const EQ: &str = "op_eq";
    pub const NE: &str = "op_ne";
    pub const LT: &str = "op_lt";
    pub const GT: &str = "op_gt";
    pub const LE: &str = "op_le";
    pub const GE: &str = "op_ge";

    // Unary operators
    pub const NEG: &str = "op_neg";
    pub const NOT: &str = "op_not";

    /// Maps binary operators to their corresponding method names
    pub fn binary_op_method(op: &str) -> Option<&'static str> {
        match op {
            "+" => Some(ADD),
            "-" => Some(SUB),
            "*" => Some(MUL),
            "/" => Some(DIV),
            "%" => Some(MOD),
            "==" => Some(EQ),
            "!=" => Some(NE),
            "<" => Some(LT),
            ">" => Some(GT),
            "<=" => Some(LE),
            ">=" => Some(GE),
            _ => None,
        }
    }

    /// Maps unary operators to their corresponding method names
    pub fn unary_op_method(op: &str) -> Option<&'static str> {
        match op {
            "-" => Some(NEG),
            "!" => Some(NOT),
            _ => None,
        }
    }
}

/// Represents parts of an interpolated string in the AST
#[derive(Debug, Clone)]
pub enum InterpolatedStringPart {
    Text(String),    // Plain text
    Expr(Box<Expr>), // Expression to evaluate
}

/// Type annotations for variables and functions
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum TypeAnnotation {
    Int,
    Float,
    String,
    Bool,
    Function { params: Vec<TypeAnnotation>, return_type: Box<TypeAnnotation> },
    Enum(String),
    Union(Vec<TypeAnnotation>),
    Any, // For gradual typing - no type checking
}

impl TypeAnnotation {
    /// Returns true if this type matches another type (allowing Any to match anything)
    pub fn matches(&self, other: &TypeAnnotation) -> bool {
        match (self, other) {
            (TypeAnnotation::Any, _) | (_, TypeAnnotation::Any) => true,
            (TypeAnnotation::Int, TypeAnnotation::Int) => true,
            (TypeAnnotation::Float, TypeAnnotation::Float) => true,
            (TypeAnnotation::String, TypeAnnotation::String) => true,
            (TypeAnnotation::Bool, TypeAnnotation::Bool) => true,
            (TypeAnnotation::Enum(a), TypeAnnotation::Enum(b)) => a == b,
            (TypeAnnotation::Union(types), other) | (other, TypeAnnotation::Union(types)) => {
                types.iter().any(|t| t.matches(other))
            }
            _ => false,
        }
    }
}

/// Represents an expression in Ruff - something that evaluates to a value
#[derive(Debug, Clone)]
pub enum Expr {
    Identifier(String),
    Int(i64),   // Integer literal like 42
    Float(f64), // Float literal like 3.14
    String(String),
    InterpolatedString(Vec<InterpolatedStringPart>), // String with expressions
    Bool(bool),
    Function {
        params: Vec<String>,
        param_types: Vec<Option<TypeAnnotation>>,
        return_type: Option<TypeAnnotation>,
        body: Vec<Stmt>,
    },
    UnaryOp {
        op: String,
        operand: Box<Expr>,
    },
    #[allow(dead_code)]
    BinaryOp {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
    },
    #[allow(dead_code)]
    Call {
        function: Box<Expr>,
        args: Vec<Expr>,
    },
    Tag(String, Vec<Expr>), // for enum variant constructors like Result::Ok(...)
    StructInstance {
        name: String,
        fields: Vec<(String, Expr)>,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    ArrayLiteral(Vec<Expr>),
    DictLiteral(Vec<(Expr, Expr)>), // key-value pairs
    IndexAccess {
        object: Box<Expr>,
        index: Box<Expr>,
    },
}

impl Expr {
    /// Returns a source location for this expression if available.
    /// Currently returns unknown location - will be enhanced when parser tracks locations.
    #[allow(dead_code)]
    pub fn location(&self) -> SourceLocation {
        SourceLocation::unknown()
    }
}

/// Represents a statement in Ruff - an action or declaration
#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        value: Expr,
        #[allow(dead_code)]
        mutable: bool,
        type_annotation: Option<TypeAnnotation>,
    },
    Const {
        name: String,
        value: Expr,
        type_annotation: Option<TypeAnnotation>,
    },
    #[allow(dead_code)]
    Assign {
        target: Expr, // Can be Identifier or IndexAccess
        value: Expr,
    },
    FuncDef {
        name: String,
        params: Vec<String>,
        param_types: Vec<Option<TypeAnnotation>>,
        return_type: Option<TypeAnnotation>,
        body: Vec<Stmt>,
    },
    EnumDef {
        name: String,
        variants: Vec<String>,
    },
    Match {
        value: Expr,
        cases: Vec<(String, Vec<Stmt>)>,
        default: Option<Vec<Stmt>>,
    },
    ExprStmt(Expr),
    Return(Option<Expr>),
    #[allow(dead_code)]
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    Loop {
        condition: Option<Expr>,
        body: Vec<Stmt>,
    },
    For {
        var: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Break,
    Continue,
    TryExcept {
        try_block: Vec<Stmt>,
        except_var: String,
        except_block: Vec<Stmt>,
    },
    #[allow(dead_code)]
    Block(Vec<Stmt>),
    /// Import statement: import module or from module import symbol1, symbol2
    Import {
        module: String,
        symbols: Option<Vec<String>>, // None means import whole module, Some means specific symbols
    },
    /// Export statement: marks a statement as exported from a module
    Export {
        stmt: Box<Stmt>,
    },
    StructDef {
        name: String,
        fields: Vec<(String, Option<TypeAnnotation>)>,
        methods: Vec<Stmt>, // FuncDef statements
    },
    /// Spawn statement: run a block of code in a background thread
    Spawn {
        body: Vec<Stmt>,
    },
}

impl Stmt {
    /// Returns a source location for this statement if available.
    /// Currently returns unknown location - will be enhanced when parser tracks locations.
    #[allow(dead_code)]
    pub fn location(&self) -> SourceLocation {
        SourceLocation::unknown()
    }
}
