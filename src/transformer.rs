pub mod builtin;
use ::chumsky::prelude::*;
use chumsky::combinator::MapWithSpan;
use std::{hash::Hash, ops::Range};

use crate::{Datum, IntoTokens, ParseError, Spanned, Token};
pub trait Transformer {
    fn transform(&self, datum: &mut Spanned<Datum>);
}

fn enclosed<T>(
    parser: impl chumsky::Parser<Token, T, Error = Simple<Token>>,
) -> impl chumsky::Parser<Token, T, Error = Simple<Token>> {
    parser.delimited_by(just(Token::Keyword("(")), just(Token::Keyword(")")))
}
fn spanned<S: Clone + Hash + Eq, T, I: chumsky::Parser<S, T, Error = Simple<S>>>(
    parser: I,
) -> MapWithSpan<I, impl Fn(T, Range<usize>) -> (T, Range<usize>) + Clone + Copy, T> {
    parser.map_with_span(|t, span| (t, span))
}
pub fn datums() -> impl Parser<Token, Vec<Spanned<Datum>>, Error = Simple<Token>> {
    macro_rules! select {
        ($category: literal, {$($p:pat $(if $guard:expr)? => $out:expr),+ $(,)?}) => ({
            filter_map(move |span: std::ops::Range<usize>, x| match x {
                $($p $(if $guard)? => ::core::result::Result::Ok($out)),+,
                other => Err(Simple::expected_input_found(
                    span,
                    Some(Some(Token::Keyword($category))),
                    Some(other),
                )),
            })
        });
    }
    recursive(|datum| {
            let keyword = select! {
                "<keyword>",
                {
                    Token::Keyword(keyword)  if keyword != "(" && keyword != ")" => Datum::Keyword(keyword),
                }
            };
            let primitive = select! {
                "<primitive>",
                {
                    Token::Primitive(p) => Datum::Primitive(p),
                }
            };
            let list = enclosed(datum.repeated())
                .map(Datum::List)
                .labelled("( ... )");
            spanned(
                list.or(keyword)
                    .or(primitive)
                    .recover_with(nested_delimiters(
                        Token::Keyword("("),
                        Token::Keyword(")"),
                        [],
                        |_| Datum::Error,
                    )),
            )
        })
        .repeated()
        .then_ignore(end())
}
pub fn datumize(
    tokens: &[Spanned<Token>],
    source: &str,
    source_path: &str,
) -> Result<Vec<Spanned<Datum>>, ParseError<Token>> {
    use ::chumsky::Parser as _;
    let len = source.len();
    datums()
        .parse(::chumsky::Stream::from_iter(
            len..len + 1,
            tokens.iter().cloned(),
        ))
        .map_err(|e| ParseError::new(source.to_owned(), source_path.to_owned(), e, "token"))
}

pub fn expansion(
    transformers: &mut Vec<Box<dyn Transformer>>,
    tokens: &[Spanned<Token>],
    source: &str,
    source_path: &str,
) -> Result<Vec<Spanned<Token>>, ParseError<Token>> {
    let mut datums = datumize(tokens, source, source_path)?;
    for transformer in transformers {
        for datum in &mut datums {
            transformer.transform(datum);
        }
    }
    Ok(datums.into_tokens())
}
