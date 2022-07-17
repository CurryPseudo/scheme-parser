use std::fmt::Display;

use crate::*;
use chumsky::prelude::*;
pub use error::*;

const EXTENDED_IDENTIFIER_CHARS: &str = "!$%&*+-/:<=>?@^_~";
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Integer(i64),
    Bool(bool),
    Ident(String),
    Keyword(&'static str),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Integer(i) => write!(f, "{}", i),
            Token::Ident(ident) => write!(f, "{}", ident),
            Token::Keyword(keyword) => write!(f, "{}", keyword),
            Token::Bool(b) => {
                if *b {
                    write!(f, "#t")
                } else {
                    write!(f, "#f")
                }
            }
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
        just("#t").to(Token::Bool(true)),
        just("#f").to(Token::Bool(false)),
    ));
    let num = just('-')
        .or_not()
        .chain::<char, _, _>(text::int(10))
        .collect::<String>()
        .map(|n| Token::Integer(n.parse::<i64>().unwrap()))
        .labelled("number");
    let ident =
        filter(|c: &char| c.is_ascii_alphanumeric() || EXTENDED_IDENTIFIER_CHARS.contains(*c))
            .repeated()
            .at_least(1)
            .collect()
            .map(Token::Ident)
            .labelled("ident");
    let comment = just(';')
        .then(take_until(text::newline().or(end())))
        .ignored()
        .labelled("comment");
    let token = keyword.or(boolean).or(num).or(ident).labelled("token");
    token
        .map_with_span(|token, span| (token, span))
        .padded_by(comment.repeated().padded())
        .padded()
        .repeated()
        .then_ignore(end())
}

pub fn tokenize(
    source: &str,
    source_path: &str,
) -> (Option<Vec<Spanned<Token>>>, Option<ParseError<char>>) {
    let (tokens, e) = lexer().parse_recovery(source);
    (
        tokens,
        if e.is_empty() {
            None
        } else {
            Some(ParseError {
                source: source.to_string(),
                source_path: source_path.to_string(),
                simple: e,
                type_name: "char",
                colorful: false,
                display_every_expected: false,
            })
        },
    )
}
