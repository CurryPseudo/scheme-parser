use std::fmt::Display;

use derive_more::From;
use num::{rational::Ratio, traits::Pow, BigInt, Integer, ToPrimitive};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Real {
    pub numer: BigInt,
    pub denom_log10: u32,
}

impl Real {
    fn denom(&self) -> BigInt {
        BigInt::from(10).pow(self.denom_log10)
    }
}

impl ToPrimitive for Real {
    fn to_i64(&self) -> Option<i64> {
        Ratio::new(self.numer.clone(), self.denom()).to_i64()
    }

    fn to_u64(&self) -> Option<u64> {
        Ratio::new(self.numer.clone(), self.denom()).to_u64()
    }

    fn to_f64(&self) -> Option<f64> {
        Ratio::new(self.numer.clone(), self.denom()).to_f64()
    }
}

impl Display for Real {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let denom = self.denom();
        let int = self.numer.div_floor(&denom);
        let frac = self.numer.mod_floor(&denom);
        write!(f, "{}.{}", int, frac)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, From)]
pub enum Primitive {
    Integer(BigInt),
    Bool(bool),
    Real(Real),
    Ident(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, From)]
pub enum Token {
    #[from(forward)]
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
            Primitive::Real(real) => {
                write!(f, "{}", real)
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
