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
    List(Vec<Node>),
    Map(Vec<(Node, Node)>),
    TupleType(Option<String>, Vec<Node>),
    FieldsType(Option<String>, Vec<(String, Node)>),
}

impl<'a> From<Pair<'a, Rule>> for Node {
    fn from(pair: Pair<Rule>) -> Node {
        match pair.as_rule() {
            Rule::ron_file => {
                let mut iter = pair.into_inner();
                let extensions = iter
                    .take_while_ref(|item| item.as_rule() == Rule::extension)
                    .flat_map(Pair::into_inner)
                    .map(|ext_name| ext_name.as_str().into())
                    .collect();
                let value = iter.next().map(Node::from).unwrap();

                debug_assert!(iter.next().unwrap().as_rule() == Rule::EOI);

                Node(0, Kind::RonFile(extensions, Box::new(value)))
            }

            // atomics
            Rule::bool
            | Rule::char
            | Rule::string
            | Rule::signed_int
            | Rule::float
            | Rule::unit_type => {
                let atom = pair.as_str().to_string();
                Node(atom.len(), Kind::Atom(atom))
            }

            // collections
            Rule::list => {
                let values: Vec<_> = pair.into_inner().map(Node::from).collect();

                // N elements -> N-1 ", " + "[]" -> +2 chars per element
                let len = values.iter().map(|n| n.0 + 2).sum();

                Node(len, Kind::List(values))
            }

            Rule::map => {
                let entries: Vec<_> = pair
                    .into_inner()
                    .map(|entry| {
                        let mut kv_iter = entry.into_inner();
                        let (key, value) = (kv_iter.next().unwrap(), kv_iter.next().unwrap());
                        (Node::from(key), Node::from(value))
                    })
                    .collect();

                // N entries -> N ": " + N-1 ", " + "{}" -> +4 chars per entry
                let len = entries.iter().map(|(k, v)| k.0 + v.0 + 4).sum();

                Node(len, Kind::Map(entries))
            }

            Rule::tuple_type => {
                let mut iter = pair.into_inner().peekable();
                let ident = match iter.peek().unwrap().as_rule() {
                    Rule::ident => Some(iter.next().unwrap().as_str().to_string()),
                    _ => None,
                };

                let values: Vec<_> = iter.map(Node::from).collect();

                // N elements -> N-1 ", " + "()" -> +2 chars per element
                let len = values.iter().map(|n| n.0 + 2).sum::<usize>()
                    + ident.as_ref().map_or(0, |i| i.len());

                Node(len, Kind::TupleType(ident, values))
            }

            Rule::fields_type => {
                let mut iter = pair.into_inner().peekable();
                let ident = match iter.peek().unwrap().as_rule() {
                    Rule::ident => Some(iter.next().unwrap().as_str().to_string()),
                    _ => None,
                };

                let fields: Vec<_> = iter
                    .map(|field| {
                        let mut field_iter = field.into_inner();
                        let (field_name, field_value) =
                            (field_iter.next().unwrap(), field_iter.next().unwrap());
                        (field_name.as_str().to_string(), Node::from(field_value))
                    })
                    .collect();

                // N entries -> N ": " + N-1 ", " + "()" -> +4 chars per entry
                let len: usize = fields.iter().map(|(k, v)| k.len() + v.0 + 4).sum::<usize>()
                    + ident.as_ref().map_or(0, |i| i.len());

                Node(len, Kind::FieldsType(ident, fields))
            }

            // intermediates and aggregates
            Rule::value => Node::from(pair.into_inner().next().unwrap()),

            // handled in other rules
            _ => unreachable!(),
        }
    }
}
