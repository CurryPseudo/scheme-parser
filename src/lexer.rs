use std::fmt::Display;

use crate::*;
use chumsky::prelude::*;
use derive_more::From;
pub use error::*;
use num::{traits::Pow, BigInt, Integer, Signed, Zero};

const EXTENDED_IDENTIFIER_CHARS: &str = "!$%&*+-/:<=>?@^_~";

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

pub fn lexer() -> impl Parser<char, Vec<Spanned<Token>>, Error = Simple<char>> {
    macro_rules! char {
        ($s: expr) => {
            just($s).map(|_| Token::Keyword($s)).labelled($s)
        };
    }
    macro_rules! keyword {
        ($s: expr) => {
            text::keyword($s).map(|_| Token::Keyword($s)).labelled($s)
        };
    }

    let keyword = char!("(").or(char!(")")).or(keyword!("define"));
    let boolean = choice((
        just("#t").to(Primitive::Bool(true).into()),
        just("#f").to(Primitive::Bool(false).into()),
    ));
    let sign = (just('+').or(just('-'))).or_not();
    let integer_raw = sign.chain::<char, _, _>(text::int(10)).collect::<String>();
    let integer = integer_raw
        .map(|n| Primitive::Integer(n.parse().unwrap()).into())
        .labelled("number");
    let real = integer_raw
        .then_ignore(just("."))
        .then(text::int(10))
        .map(|(int, frac)| {
            let mut numer: BigInt = int.parse().unwrap();
            let sign = numer.sign();
            numer = numer.abs();

            let frac: BigInt = frac.parse().unwrap();
            let mut denom_log10 = 0u32;
            {
                let mut frac: BigInt = frac.clone();
                while !frac.is_zero() {
                    frac /= 10;
                    denom_log10 += 1;
                }
            };
            numer = numer * BigInt::from(10).pow(denom_log10) + frac;
            numer = match sign {
                num::bigint::Sign::Minus => -numer,
                _ => numer,
            };
            Primitive::Real { numer, denom_log10 }.into()
        });
    let ident =
        filter(|c: &char| c.is_ascii_alphanumeric() || EXTENDED_IDENTIFIER_CHARS.contains(*c))
            .repeated()
            .at_least(1)
            .collect()
            .map(|s| Primitive::Ident(s).into())
            .labelled("ident");
    let comment = just(';')
        .then(take_until(text::newline().or(end())))
        .ignored()
        .labelled("comment");
    let token = keyword
        .or(boolean)
        .or(real)
        .or(integer)
        .or(ident)
        .labelled("token");
    token
        .map_with_span(|token, span| (token, span))
        .padded_by(comment.repeated().padded())
        .padded()
        .repeated()
        .then_ignore(end())
}

pub fn tokenize<'a>(
    source: &'a str,
    source_path: &'a str,
) -> (Option<Vec<Spanned<Token>>>, Option<ParseError<'a, char>>) {
    let (tokens, e) = lexer().parse_recovery(source);
    (
        tokens,
        if e.is_empty() {
            None
        } else {
            Some(ParseError {
                source,
                source_path,
                simple: e,
                type_name: "char",
                colorful: false,
                display_every_expected: false,
            })
        },
    )
}
