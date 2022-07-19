use std::fmt::Debug;

use scheme_parser::*;

// This is a helper struct to print the source code of a span.
pub struct SpanToSource<'a, T>(pub &'a T, pub &'a str);
impl<'a, T> SpanToSource<'a, T> {
    fn replace<U>(&self, u: &'a U) -> SpanToSource<'a, U> {
        SpanToSource(u, self.1)
    }
}

impl<'a> Debug for SpanToSource<'a, Program> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcedureBody")
            .field("defs", &self.replace(&self.0.defs))
            .field("exprs", &self.replace(&self.0.exprs))
            .field("last_expr", &self.replace(&self.0.last_expr))
            .finish()
    }
}

impl<'a, T: Debug> Debug for SpanToSource<'a, Box<T>>
where
    SpanToSource<'a, T>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.replace(self.0.as_ref()).fmt(f)
    }
}

impl<'a, T: Debug> Debug for SpanToSource<'a, Option<T>>
where
    SpanToSource<'a, T>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(t) => f.debug_tuple("Some").field(&self.replace(t)).finish(),
            None => f.debug_struct("None").finish(),
        }
    }
}

impl<'a, T: Debug> Debug for SpanToSource<'a, Vec<T>>
where
    SpanToSource<'a, T>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(|element| self.replace(element)))
            .finish()
    }
}

impl<'a> Debug for SpanToSource<'a, Definition> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Definition")
            .field(&self.0 .0)
            .field(&self.replace(&self.0 .1))
            .finish()
    }
}

impl<'a> Debug for SpanToSource<'a, Spanned<Expression>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("")
            .field(&self.replace(&self.0 .0))
            .field(&&self.1[self.0 .1.clone()] as &dyn Debug)
            .finish()
    }
}

impl<'a> Debug for SpanToSource<'a, Expression> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Expression::ProcedureCall(exprs) => f
                .debug_tuple("ProcedureCall")
                .field(&self.replace(exprs))
                .finish(),
            Expression::Procedure { args, body } => f
                .debug_tuple("Procedure")
                .field(&self.replace(args))
                .field(&self.replace(body.as_ref()))
                .finish(),
            Expression::Primitive(_) | Expression::Error => self.0.fmt(f),
            Expression::Conditional {
                test,
                conseq,
                alter,
            } => f
                .debug_struct("Conditional")
                .field("test", &self.replace(test))
                .field("conseq", &self.replace(conseq))
                .field("alter", &self.replace(alter))
                .finish(),
        }
    }
}

trait Root {}

impl<'a, T: Debug + Root> Debug for SpanToSource<'a, Spanned<T>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("")
            .field(&self.0 .0)
            .field(&&self.1[self.0 .1.clone()] as &dyn Debug)
            .finish()
    }
}

impl Root for Token {}
impl Root for String {}
