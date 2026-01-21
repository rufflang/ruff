// File: src/ast.rs
//
// Abstract Syntax Tree (AST) definitions for the Ruff programming language.
// Defines the structure of parsed Ruff programs.
//
// The AST represents the syntactic structure of Ruff code after parsing.
// Expressions (Expr) represent values and computations, while Statements (Stmt)
// represent actions and control flow.

/// Represents an expression in Ruff - something that evaluates to a value
#[derive(Debug, Clone)]
pub enum Expr {
    Identifier(String),
    Number(f64),
    String(String),
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
}

/// Represents a statement in Ruff - an action or declaration
#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        value: Expr,
        #[allow(dead_code)]
        mutable: bool,
    },
    Const {
        name: String,
        value: Expr,
    },
    #[allow(dead_code)]
    Assign {
        name: String,
        value: Expr,
    },
    FuncDef {
        name: String,
        params: Vec<String>,
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
    TryExcept {
        try_block: Vec<Stmt>,
        except_var: String,
        except_block: Vec<Stmt>,
    },
    #[allow(dead_code)]
    Block(Vec<Stmt>),
}
