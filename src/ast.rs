mod display;

use super::Rule;
use itertools::Itertools;
use pest::iterators::Pair;
use std::collections::BTreeSet;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Node(usize, Kind);

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Kind {
    RonFile(BTreeSet<String>, Box<Node>),
    Atom(String), // atomic types: bool, char, str, int, float, unit type
    Tuple(Vec<Node>),
    List(Vec<Node>),
    Map(Vec<(Node, Node)>),
    NamedTypeTuple(String, Vec<Node>),
    NamedTypeFields(String, Vec<(String, Node)>),
    AnonymousTypeFields(Vec<(String, Node)>),
}

impl<'a> From<Pair<'a, Rule>> for Node {
    fn from(value: Pair<Rule>) -> Node {
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
                let ast = iter.next().map(Node::from).unwrap();
                debug_assert!(iter.next().unwrap().as_rule() == Rule::EOI);

                Node(0, Kind::RonFile(extensions, Box::new(ast)))
            }

            // atomics
            Rule::bool
            | Rule::char
            | Rule::string
            | Rule::signed_int
            | Rule::float
            | Rule::named_type_unit => Node(0, Kind::Atom(value.as_str().into())),

            // collections
            Rule::tuple => {
                let comma_separated_values = value.into_inner().next().unwrap();
                let values = comma_separated_values.into_inner().map(Node::from).collect();

                Node(0, Kind::Tuple(values))
            }

            Rule::list => {
                let comma_separated_values = value.into_inner().next().unwrap();
                let values = comma_separated_values.into_inner().map(Node::from).collect();

                Node(0, Kind::List(values))
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
                                (Node::from(key), Node::from(value))
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                Node(0, Kind::Map(entries))
            }

            Rule::named_type_tuple => {
                let mut iter = value.into_inner();
                let (ident, tuple) = (iter.next().unwrap(), iter.next().unwrap());

                if let Node(_, Kind::Tuple(inner)) = Node::from(tuple) {
                    return Node(0, Kind::NamedTypeTuple(ident.as_str().into(), inner))
                }

                unreachable!();
            }

            Rule::named_type_fields => {
                let mut iter = value.into_inner();
                let (ident, fields_iter) = (iter.next().unwrap(), iter.next().unwrap());
                let fields: Vec<(String, Node)> = fields_iter
                    .into_inner()
                    .map(|field| {
                        let mut field_iter = field.into_inner();
                        let (field_name, field_value) =
                            (field_iter.next().unwrap(), field_iter.next().unwrap());
                        (field_name.as_str().into(), Node::from(field_value))
                    })
                    .collect();

                Node(0, Kind::NamedTypeFields(ident.as_str().into(), fields))
            }

            Rule::anonymous_type_fields => {
                let fields_iter = value.into_inner().next().unwrap();
                let fields: Vec<(String, Node)> = fields_iter
                    .into_inner()
                    .map(|field| {
                        let mut field_iter = field.into_inner();
                        let (field_name, field_value) =
                            (field_iter.next().unwrap(), field_iter.next().unwrap());
                        (field_name.as_str().into(), Node::from(field_value))
                    })
                    .collect();

                Node(0, Kind::AnonymousTypeFields(fields))
            }

            // intermediates and aggregates
            Rule::value | Rule::named_type => Node::from(value.into_inner().next().unwrap()),

            // handled in other rules
            _ => unreachable!(),
        }
    }
}
