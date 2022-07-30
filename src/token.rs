use std::fmt::Display;

use derive_more::From;
use num::{traits::Pow, BigInt, Integer};
#[derive(Debug, Clone, PartialEq, Eq, Hash, From)]
pub enum Primitive {
    Integer(BigInt),
    Bool(bool),
    Real { numer: BigInt, denom_log10: u32 },
    Ident(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, From)]
pub enum Token {
    Primitive(Primitive),
    Keyword(&'static str),
}

impl Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Primitive::Integer(i) => write!(f, "{}", i),
            Primitive::Ident(ident) => write!(f, "{}", ident),
            Primitive::Bool(b) => {
                if *b {
                    write!(f, "#t")
                } else {
                    write!(f, "#f")
                }
            }
            Primitive::Real { numer, denom_log10 } => {
                let denom = BigInt::from(10).pow(*denom_log10);
                let int = numer.div_floor(&denom);
                let frac = numer.mod_floor(&denom);
                write!(f, "{}.{}", int, frac)
            }
        }
    }
}
impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Keyword(keyword) => write!(f, "{}", keyword),
            Token::Primitive(primitive) => write!(f, "{}", primitive),
        }
    }
}
