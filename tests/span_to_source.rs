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
            .field(&self.replace(&self.0 .0))
            .field(&self.replace(&self.0 .1))
            .finish()
    }
}

impl<'a> Debug for SpanToSource<'a, Expression> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Expression::ProcedureCall { operator, args } => f
                .debug_struct("ProcedureCall")
                .field("operator", &self.replace(operator))
                .field("args", &self.replace(args))
                .finish(),
            Expression::Procedure { args, body } => f
                .debug_struct("Procedure")
                .field("args", &self.replace(args))
                .field("body", &self.replace(body.as_ref()))
                .finish(),
            Expression::Assignment(ident, expr) => f
                .debug_tuple("Assignment")
                .field(&self.replace(ident))
                .field(&self.replace(expr))
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

impl<'a> Debug for SpanToSource<'a, Datum> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Datum::Primitive(_) | Datum::Error | Datum::Keyword(_) => self.0.fmt(f),
            Datum::List(list) => f.debug_tuple("List").field(&self.replace(list)).finish(),
        }
    }
}

#[derive(Debug)]
struct Source<T>(T);

macro_rules! impl_non_leaf {
    ($t: ty) => {
        impl<'a> Debug for SpanToSource<'a, Spanned<$t>> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple("")
                    .field(&self.replace(&self.0 .0))
                    .field(&Source(&self.1[self.0 .1.clone()]) as &dyn Debug)
                    .finish()
            }
        }
    };
}
macro_rules! impl_leaf {
    ($t: ty) => {
        impl<'a> Debug for SpanToSource<'a, Spanned<$t>> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple("")
                    .field(&&self.0 .0)
                    .field(&Source(&self.1[self.0 .1.clone()]) as &dyn Debug)
                    .finish()
            }
        }
    };
}

impl_non_leaf!(Definition);
impl_non_leaf!(Expression);
impl_non_leaf!(Datum);
impl_leaf!(Token);
impl_leaf!(String);
