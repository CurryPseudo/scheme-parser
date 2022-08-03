use crate::*;
use derive_more::From;

#[derive(Debug)]
pub struct ProcedureBody {
    pub defs: Vec<Spanned<Definition>>,
    pub exprs: Vec<Spanned<Expression>>,
    pub last_expr: Spanned<Expression>,
}

pub type Program = ProcedureBody;

#[derive(Debug)]
pub struct Definition(pub Spanned<String>, pub Spanned<Expression>);

#[derive(Debug, From)]
pub enum Expression {
    ProcedureCall {
        operator: Box<Spanned<Expression>>,
        args: Vec<Spanned<Expression>>,
    },
    #[from(forward)]
    Primitive(Primitive),
    Procedure {
        /// Arguments
        args: Vec<Spanned<String>>,
        /// Procedure Body
        body: Box<ProcedureBody>,
    },
    Conditional {
        /// Test
        test: Box<Spanned<Expression>>,
        /// Consequent
        conseq: Box<Spanned<Expression>>,
        /// Alternative
        alter: Option<Box<Spanned<Expression>>>,
    },
    Assignment(Spanned<String>, Box<Spanned<Expression>>),
    Error,
}
