use std::fmt::Display;

use crate::*;
use derive_more::From;
#[derive(From, Debug, PartialEq, Eq, Clone, Hash)]
pub enum Datum {
    Error,
    Keyword(&'static str),
    Primitive(Primitive),
    List(Vec<Spanned<Datum>>),
}

impl Display for Datum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Datum::Error => write!(f, "<error>"),
            Datum::Keyword(keyword) => write!(f, "{}", keyword),
            Datum::Primitive(p) => write!(f, "{}", p),
            Datum::List(_) => write!(f, "( ... )"),
        }
    }
}

pub trait IntoTokens {
    fn into_tokens(self) -> Vec<Spanned<Token>>;
}

impl IntoTokens for Spanned<Datum> {
    fn into_tokens(self) -> Vec<Spanned<Token>> {
        todo!()
    }
}

impl IntoTokens for Vec<Spanned<Datum>> {
    fn into_tokens(self) -> Vec<Spanned<Token>> {
        todo!()
    }
}
