use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

use ariadne::{sources, Color, Fmt, Label, Report, ReportKind};
use chumsky::prelude::Simple;

#[derive(Debug, Default)]
pub struct ParseError<T: Hash> {
    pub(crate) simple: Vec<Simple<T>>,
    pub(crate) source: String,
    pub(crate) source_path: String,
    pub(crate) colorful: bool,
    pub(crate) type_name: &'static str,
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
            let report = Report::build(
                ReportKind::Error,
                self.source_path.clone(),
                error.span().start,
            );
            let report = match error.reason() {
                chumsky::error::SimpleReason::Unexpected => report
                    .with_message(format!(
                        "{}{}, expected {}",
                        if error.found().is_some() {
                            format!("Unexpected {} in input", self.type_name)
                        } else {
                            "Unexpected end of input".to_string()
                        },
                        if let Some(label) = error.label() {
                            format!(" while parsing {}", label)
                        } else {
                            "".to_string()
                        },
                        if error.expected().len() == 0 {
                            "something else".to_string()
                        } else {
                            let mut expected = error
                                .expected()
                                .map(|expected| match expected {
                                    Some(expected) => expected.to_string(),
                                    None => "end of input".to_string(),
                                })
                                .collect::<Vec<_>>();
                            expected.sort();
                            expected.join(", ")
                        }
                    ))
                    .with_label(with_color!(
                        Label::new((self.source_path.clone(), error.span())).with_message(format!(
                            "Unexpected {}",
                            fg!(
                                error
                                    .found()
                                    .map(|c| format!("{} {}", self.type_name, c))
                                    .unwrap_or_else(|| "end of file".to_string()),
                                Color::Red
                            )
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
                        Label::new((self.source_path.clone(), span.clone())).with_message(format!(
                            "Unclosed delimiter {}",
                            fg!(delimiter, Color::Yellow)
                        )),
                        Color::Yellow
                    ))
                    .with_label(with_color!(
                        Label::new((self.source_path.clone(), error.span())).with_message(format!(
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
                        Label::new((self.source_path.clone(), error.span()))
                            .with_message(format!("{}", fg!(msg, Color::Red))),
                        Color::Red
                    )),
            };
            let mut content = Vec::new();
            if report
                .with_config(ariadne::Config::default().with_color(self.colorful))
                .finish()
                .write(
                    sources(vec![(self.source_path.clone(), &self.source)]),
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

impl<T: Hash> ParseError<T> {
    /// Should display with color or not, default: false
    pub fn with_color(self, colorful: bool) -> Self {
        ParseError { colorful, ..self }
    }
}
