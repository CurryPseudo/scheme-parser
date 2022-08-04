use crate::{transformer::Transformer, Datum, Primitive};

pub struct Begin;

impl Transformer for Begin {
    fn transform(&self, datum: &mut crate::Spanned<Datum>) {
        let mut is_matched = false;
        let (datum, span) = datum;
        let mut lambda_list = Vec::new();
        if let Datum::List(list) = datum {
            let mut begin_span = 0..0;
            if let Some((Datum::Primitive(Primitive::Ident(ident)), span)) = list.first() {
                if ident == "begin" {
                    is_matched = true;
                    begin_span = span.clone();
                }
            }
            if is_matched {
                let list = list.drain(1..);
                lambda_list.push((Datum::Keyword("lambda"), begin_span.clone()));
                lambda_list.push((Datum::List(vec![]), begin_span));
                lambda_list.extend(list);
            }
        }
        if is_matched {
            *datum = Datum::List(vec![(Datum::List(lambda_list), span.clone())])
        }
    }
}
