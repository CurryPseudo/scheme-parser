use std::{
    fmt::{Debug, Display, Write},
    hash::Hash,
};

use ariadne::{Color, Fmt, FnCache, Label, Report, ReportKind, Source};
use chumsky::prelude::Simple;

#[derive(Debug, Default)]
pub struct ParseError<T: Hash> {
    simple: Vec<Simple<T>>,
    source: String,
    source_path: String,
    type_name: &'static str,
    colorful: bool,
    display_every_expected: bool,
}

impl<T: Hash> ParseError<T> {
    pub fn new(
        source: String,
        source_path: String,
        e: Vec<Simple<T>>,
        type_name: &'static str,
    ) -> ParseError<T> {
        ParseError {
            source,
            source_path,
            simple: e,
            type_name,
            colorful: true,
            display_every_expected: true,
        }
    }
    pub fn display_every_expected(self, enable: bool) -> Self {
        Self {
            display_every_expected: enable,
            ..self
        }
    }
}
impl<T: Hash + Eq + Display> std::fmt::Display for ParseError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! with_color {
            ($a: expr, $b: expr) => {
                if self.colorful {
                    $a.with_color($b)
                } else {
                    $a
                }
            };
        }
        macro_rules! fg {
            ($a: expr, $b: expr) => {
                if self.colorful {
                    Box::new($a.fg($b)) as Box<dyn std::fmt::Display>
                } else {
                    Box::new($a) as Box<dyn std::fmt::Display>
                }
            };
        }
        for error in &self.simple {
            let report = Report::build(ReportKind::Error, &self.source_path, error.span().start);
            let report = match error.reason() {
                chumsky::error::SimpleReason::Unexpected => report
                    .with_message({
                        let mut msg = String::new();
                        if error.found().is_some() {
                            write!(msg, "Unexpected {} in input", self.type_name).unwrap();
                        } else {
                            write!(msg, "Unexpected end of input").unwrap();
                        }
                        if let Some(label) = error.label() {
                            write!(msg, " while parsing {}", label).unwrap();
                        }
                        if self.display_every_expected && error.expected().len() > 0 {
                            write!(msg, ", expected ").unwrap();
                            let mut expected = error
                                .expected()
                                .map(|expected| {
                                    fg!(
                                        match expected {
                                            Some(expected) => expected.to_string(),
                                            None => "end of input".to_string(),
                                        },
                                        Color::Yellow
                                    )
                                    .to_string()
                                })
                                .collect::<Vec<_>>();
                            expected.sort();
                            write!(msg, "{}", expected.join(", ")).unwrap();
                        }
                        msg
                    })
                    .with_label(with_color!(
                        Label::new((&self.source_path, error.span())).with_message(format!(
                            "Unexpected {}",
                            error
                                .found()
                                .map(|c| format!("{} {}", self.type_name, fg!(c, Color::Red)))
                                .unwrap_or_else(
                                    || fg!("end of file".to_string(), Color::Red).to_string()
                                ),
                        )),
                        Color::Red
                    )),
                chumsky::error::SimpleReason::Unclosed { span, delimiter } => report
                    .with_message(format!(
                        "Unclosed delimiter {}{}",
                        fg!(delimiter, Color::Yellow),
                        if let Some(label) = error.label() {
                            format!(" while parsing {}", label)
                        } else {
                            "".to_string()
                        },
                    ))
                    .with_label(with_color!(
                        Label::new((&self.source_path, span.clone())).with_message(format!(
                            "Unclosed delimiter {}",
                            fg!(delimiter, Color::Yellow)
                        )),
                        Color::Yellow
                    ))
                    .with_label(with_color!(
                        Label::new((&self.source_path, error.span())).with_message(format!(
                            "Must be closed before this {}",
                            fg!(
                                error
                                    .found()
                                    .map(|c| c.to_string())
                                    .unwrap_or_else(|| "end of file".to_string()),
                                Color::Red
                            )
                        )),
                        Color::Red
                    )),
                chumsky::error::SimpleReason::Custom(msg) => report
                    .with_message(format!(
                        "{}{}",
                        msg,
                        if let Some(label) = error.label() {
                            format!(" while parsing {}", label)
                        } else {
                            "".to_string()
                        },
                    ))
                    .with_label(with_color!(
                        Label::new((&self.source_path, error.span()))
                            .with_message(format!("{}", fg!(msg, Color::Red))),
                        Color::Red
                    )),
            };
            let mut content = Vec::new();
            if report
                .with_config(ariadne::Config::default().with_color(self.colorful))
                .finish()
                .write(
                    FnCache::new(
                        (move |id| Err(Box::new(format!("Failed to fetch source '{}'", id)) as _))
                            as fn(&_) -> _,
                    )
                    .with_sources(
                        vec![(
                            &self.source_path,
                            if self.source.is_empty() {
                                " "
                            } else {
                                &self.source
                            },
                        )]
                        .into_iter()
                        .map(|(id, s)| (id, Source::from(s)))
                        .collect(),
                    ),
                    &mut content,
                )
                .is_err()
            {
                return Err(std::fmt::Error);
            }
            write!(f, "{}", String::from_utf8_lossy(&content))?;
        }
        Ok(())
    }
}

impl<T: Hash + Eq + Display + Debug> std::error::Error for ParseError<T> {}

impl<T: Hash> ParseError<T> {
    /// Should display with color or not, default: true
    pub fn with_color(self, colorful: bool) -> Self {
        ParseError { colorful, ..self }
    }
}
