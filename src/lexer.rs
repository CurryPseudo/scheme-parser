use chumsky::prelude::*;

use crate::*;

const EXTENDED_IDENTIFIER_CHARS: &str = "!$%&*+-/:<=>?@^_~";
#[derive(Debug, Clone)]
pub enum Token {
    Integer(i64),
    Ident(String),
    Keyword(&'static str),
    KeywordChar(char),
}

pub fn lexer() -> impl Parser<char, Vec<Spanned<Token>>, Error = Simple<char>> {
    macro_rules! keyword {
        ($s: expr) => {
            just($s).map(|_| Token::Keyword($s)).labelled($s)
        };
    }

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
    let keyword = keyword!("(").or(keyword!(")"));
    let comment = just(';')
        .then(take_until(text::newline().or(end())))
        .ignored()
        .labelled("comment");
    let token = keyword.or(num).or(ident).labelled("token");
    token
        .padded()
        .padded_by(comment.repeated())
        .map_with_span(|token, span| (token, span))
        .repeated()
        .then_ignore(end())
}

pub fn tokenize(
    source: &str,
    source_path: &str,
) -> (Option<Vec<Spanned<Token>>>, Option<ParseError>) {
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
                ..Default::default()
            })
        },
    )
}
