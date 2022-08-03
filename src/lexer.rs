use crate::*;

pub(super) mod chumsky {
    use crate::span::*;
    use crate::token::*;
    use chumsky::{prelude::*, text::Character};
    use num::{traits::Pow, BigInt, Signed, Zero};
    const EXTENDED_IDENTIFIER_CHARS: &str = "!$%&*+-/:<=>?@^_~";
    fn keyword(keyword: &'static str) -> impl Parser<char, Token, Error = Simple<char>> {
        filter(|c: &char| c.to_char().is_ascii_alphabetic() || c.to_char() == '_')
            .map(Some)
            .chain::<char, Vec<_>, _>(
                filter(|c: &char| {
                    c.to_char().is_ascii_alphanumeric() || c.to_char() == '_' || c.to_char() == '!'
                })
                .repeated(),
            )
            .collect()
            .try_map(move |s: String, span| {
                if s == keyword {
                    Ok(())
                } else {
                    Err(Simple::<char>::expected_input_found(span, None, None))
                }
            })
            .map(move |_| Token::Keyword(keyword))
            .labelled(keyword)
    }

    pub fn lexer() -> impl Parser<char, Vec<Spanned<Token>>, Error = Simple<char>> {
        macro_rules! char {
            ($s: expr) => {
                just($s).map(|_| Token::Keyword($s)).labelled($s)
            };
        }

        let keyword = char!("(")
            .or(char!(")"))
            .or(keyword("define"))
            .or(keyword("lambda"))
            .or(keyword("if"))
            .or(keyword("set!"));
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
                Real { numer, denom_log10 }.into()
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
}

pub fn tokenize(source: &str, source_path: &str) -> Result<Vec<Spanned<Token>>, ParseError<char>> {
    use ::chumsky::Parser;
    chumsky::lexer()
        .parse(source)
        .map_err(|e| ParseError::new(source.to_owned(), source_path.to_owned(), e, "char"))
}
