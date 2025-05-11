// File: src/ast.rs

#[derive(Debug, Clone)]
pub enum Expr {
    Identifier(String),
    Number(f64),
    String(String),
    BinaryOp {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
    },
    Call {
        function: Box<Expr>,
        args: Vec<Expr>,
    },
    Tag(String, Vec<Expr>), // for enum variant constructors like Result::Ok(...)
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        value: Expr,
        mutable: bool,
    },
    Const {
        name: String,
        value: Expr,
    },
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
    Block(Vec<Stmt>),
}
