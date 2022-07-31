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
        let (datum, span) = self;
        match datum {
            Datum::Error => unreachable!(),
            Datum::Keyword(keyword) => vec![(Token::Keyword(keyword), span)],
            Datum::Primitive(primitive) => vec![(Token::Primitive(primitive), span)],
            Datum::List(datums) => {
                let mut tokens = Vec::new();
                tokens.push((Token::Keyword("("), span.start..span.start + 1));
                tokens.extend(datums.into_tokens().into_iter());
                tokens.push((Token::Keyword(")"), span.end - 1..span.end));
                tokens
            }
        }
    }
}

impl<T> IntoTokens for Vec<Spanned<T>>
where
    Spanned<T>: IntoTokens,
{
    fn into_tokens(self) -> Vec<Spanned<Token>> {
        self.into_iter().flat_map(|t| t.into_tokens()).collect()
    }
}
