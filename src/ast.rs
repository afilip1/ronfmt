mod display;

use super::Rule;
use itertools::Itertools;
use pest::iterators::Pair;
use std::collections::BTreeSet;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Ast {
    RonFile(BTreeSet<String>, Box<Ast>),
    Atom(String), // atomic types: bool, char, str, int, float, unit type
    Tuple(Vec<Ast>),
    List(Vec<Ast>),
    Map(Vec<(Ast, Ast)>),
    NamedTypeTuple(String, Vec<Ast>),
    NamedTypeFields(String, Vec<(String, Ast)>),
    AnonymousTypeFields(Vec<(String, Ast)>),
}

impl<'a> From<Pair<'a, Rule>> for Ast {
    fn from(value: Pair<Rule>) -> Ast {
        match value.as_rule() {
            Rule::ron_file => {
                let mut iter = value.into_inner();
                let extensions = iter
                    .take_while_ref(|item| item.as_rule() == Rule::extension)
                    .map(|exts| {
                        exts.into_inner()
                            .map(|ext| ext.as_str().to_string())
                            .collect()
                    })
                    .next()
                    .unwrap_or_default();
                let ast = iter.next().map(Ast::from).unwrap();
                debug_assert!(iter.next().unwrap().as_rule() == Rule::EOI);

                Ast::RonFile(extensions, Box::new(ast))
            }

            // atomics
            Rule::bool
            | Rule::char
            | Rule::string
            | Rule::signed_int
            | Rule::float
            | Rule::named_type_unit => Ast::Atom(value.as_str().into()),

            // collections
            Rule::tuple => {
                let comma_separated_values = value.into_inner().next().unwrap();
                let values = comma_separated_values.into_inner().map(Ast::from).collect();

                Ast::Tuple(values)
            }

            Rule::list => {
                let comma_separated_values = value.into_inner().next().unwrap();
                let values = comma_separated_values.into_inner().map(Ast::from).collect();

                Ast::List(values)
            }

            Rule::map => {
                let map_inner = value.into_inner().next();
                let entries = map_inner
                    .map(|mi| {
                        mi.into_inner()
                            .map(|entry| {
                                let mut kv_iter = entry.into_inner();
                                let (key, value) =
                                    (kv_iter.next().unwrap(), kv_iter.next().unwrap());
                                (Ast::from(key), Ast::from(value))
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                Ast::Map(entries)
            }

            Rule::named_type_tuple => {
                let mut iter = value.into_inner();
                let (ident, tuple) = (iter.next().unwrap(), iter.next().unwrap());

                match Ast::from(tuple) {
                    Ast::Tuple(inner) => Ast::NamedTypeTuple(ident.as_str().into(), inner),
                    _ => unreachable!(),
                }
            }

            Rule::named_type_fields => {
                let mut iter = value.into_inner();
                let (ident, fields_iter) = (iter.next().unwrap(), iter.next().unwrap());
                let fields: Vec<(String, Ast)> = fields_iter
                    .into_inner()
                    .map(|field| {
                        let mut field_iter = field.into_inner();
                        let (field_name, field_value) =
                            (field_iter.next().unwrap(), field_iter.next().unwrap());
                        (field_name.as_str().into(), Ast::from(field_value))
                    })
                    .collect();

                Ast::NamedTypeFields(ident.as_str().into(), fields)
            }

            Rule::anonymous_type_fields => {
                let fields_iter = value.into_inner().next().unwrap();
                let fields: Vec<(String, Ast)> = fields_iter
                    .into_inner()
                    .map(|field| {
                        let mut field_iter = field.into_inner();
                        let (field_name, field_value) =
                            (field_iter.next().unwrap(), field_iter.next().unwrap());
                        (field_name.as_str().into(), Ast::from(field_value))
                    })
                    .collect();

                Ast::AnonymousTypeFields(fields)
            }

            // intermediates and aggregates
            Rule::value | Rule::named_type => Ast::from(value.into_inner().next().unwrap()),

            // handled in other rules
            _ => unreachable!(),
        }
    }
}
