mod display;

use super::Rule;
use itertools::Itertools;
use pest::iterators::{Pair, Pairs};
use std::collections::BTreeSet;

pub struct RonFile(BTreeSet<String>, Box<Value>);

pub struct Value(usize, Kind);

pub struct Commented {
    value: Value,
    eol: Option<String>,
    pre: Option<Vec<String>>,
    post: Option<Vec<String>>,
}

impl std::ops::Deref for Commented {
    type Target = Value;
    fn deref(&self) -> &Value {
        &self.value
    }
}

impl std::ops::DerefMut for Commented {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

pub enum Kind {
    Atom(String), // atomic types: bool, char, str, int, float, unit type
    List(Vec<Commented>),
    Map(Vec<(Value, Commented)>),
    // LineComment(String),
    // BlockComment(String),
    TupleType(Option<String>, Vec<Commented>),
    FieldsType(Option<String>, Vec<(String, Commented)>),
}

impl RonFile {
    pub fn parse_from(pair: Pair<Rule>) -> RonFile {
        if pair.as_rule() != Rule::ron_file {
            panic!("expected ron_file pair");
        }

        let mut iter = pair.into_inner();
        let extensions = iter
            .take_while_ref(|item| item.as_rule() == Rule::extension)
            .flat_map(Pair::into_inner)
            .map(|ext_name| ext_name.as_str().into())
            .collect();
        let value = iter.next().map(Value::from).unwrap();

        debug_assert!(iter.next().unwrap().as_rule() == Rule::EOI);

        RonFile(extensions, Box::new(value))
    }
}

fn emptynull<T>(v: Vec<T>) -> Option<Vec<T>> {
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

impl Commented {
    fn from<'a, T: std::iter::IntoIterator<Item = Pair<'a, Rule>>>(pairs: T) -> Vec<Commented> {
        let mut res = vec![];

        let mut pre = vec![];
        let mut last = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::COMMENT => {
                    let a = pair.as_str().to_string();
                    pre.push(a)
                }
                _ => {
                    match last {
                        None => (),
                        Some(c) => res.push(c),
                    }
                    let c = Commented {
                        value: Value::from(pair),
                        pre: emptynull(pre),
                        eol: None,
                        post: None,
                    };
                    pre = vec![];
                    last = Some(c);
                }
            }
        }

        match last {
            None => (), // we might be dropping comments
            Some(mut c) => {
                c.post = emptynull(pre);
                res.push(c);
            }
        }

        res
    }

    fn keyed(pairs: Pairs<Rule>) -> Vec<(Value, Commented)> {
        let mut res = vec![];

        let mut pre = vec![];
        let mut last = None;
        // let mut key = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::COMMENT => {
                    let a = pair.as_str().to_string();
                    pre.push(a)
                }
                Rule::map_entry => {
                    match last {
                        None => (),
                        Some(last) => res.push(last),
                    }
                    let mut k = None;
                    let mut v = None;
                    for inner in pair.into_inner() {
                        match inner.as_rule() {
                            Rule::COMMENT => pre.push(inner.as_str().to_string()),
                            Rule::value => {
                                if k.is_none() {
                                    k = Some(Value::from(inner))
                                } else {
                                    v = Some(Value::from(inner))
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    let c = Commented {
                        value: v.unwrap(),
                        pre: emptynull(pre),
                        eol: None,
                        post: None,
                    };
                    last = Some((k.unwrap(), c));
                    pre = vec![];
                }
                _ => unreachable!(),
            }
        }

        match last {
            None => (), // we might be dropping comments
            Some(mut c) => {
                c.1.post = emptynull(pre);
                res.push(c);
            }
        }

        res
    }

    fn str_keyed<'a, T: std::iter::IntoIterator<Item = Pair<'a, Rule>>>(
        pairs: T,
    ) -> Vec<(String, Commented)> {
        let mut res = vec![];

        let mut pre = vec![];
        let mut last = None;

        for pair in pairs {
            match pair.as_rule() {
                Rule::COMMENT => {
                    let a = pair.as_str().to_string();
                    pre.push(a)
                }
                _ => {
                    match last {
                        None => (),
                        Some(last) => res.push(last),
                    }
                    let mut k = None;
                    let mut v = None;
                    for inner in pair.into_inner() {
                        match inner.as_rule() {
                            Rule::COMMENT => pre.push(inner.as_str().to_string()),
                            Rule::ident => k = Some(inner.as_str().to_string()),
                            Rule::value => v = Some(Value::from(inner)),
                            _ => unreachable!(),
                        }
                    }
                    let c = Commented {
                        value: v.unwrap(),
                        pre: emptynull(pre),
                        eol: None,
                        post: None,
                    };
                    last = Some((k.unwrap(), c));
                    pre = vec![];
                }
            }
        }

        match last {
            None => (), // we might be dropping comments
            Some(mut c) => {
                c.1.post = emptynull(pre);
                res.push(c);
            }
        }

        res
    }
}

impl Value {
    fn from(pair: Pair<Rule>) -> Value {
        match pair.as_rule() {
            Rule::bool
            | Rule::char
            | Rule::string
            | Rule::signed_int
            | Rule::COMMENT
            | Rule::line_comment
            | Rule::block_comment
            | Rule::float
            | Rule::unit_type => {
                let a = pair.as_str().to_string();
                // let ident = match iter.peek().map(|p| p.as_rule()) {
                //     Some(Rule::COMMENT) => Some(iter.next().unwrap().as_str().to_string()),
                //     _ => None,
                // };
                Value(a.len(), Kind::Atom(a))
            }

            Rule::list => {
                let values: Vec<_> = Commented::from(pair.into_inner());
                // .map(Value::from).collect();
                let len = values.iter().map(|n| n.0 + 2).sum(); // N elements -> N-1 ", " + "[]" -> +2 chars per element

                Value(len, Kind::List(values))
            }

            Rule::map => {
                let entries: Vec<_> = Commented::keyed(pair.into_inner());
                // .map(|entry| {
                //     let mut kv_iter = entry.into_inner();
                //     let (k, v) = (kv_iter.next().unwrap(), kv_iter.next().unwrap());
                //     (Value::from(k), Value::from(v))
                // })
                // .collect();
                let len = entries.iter().map(|(k, v)| k.0 + v.0 + 4).sum(); // N entries -> N ": " + N-1 ", " + "{}" -> +4 chars per entry

                Value(len, Kind::Map(entries))
            }

            Rule::tuple_type => {
                let mut iter = pair.into_inner().peekable();
                let ident = match iter.peek().map(|p| p.as_rule()) {
                    Some(Rule::ident) => Some(iter.next().unwrap().as_str().to_string()),
                    _ => None,
                };

                let values: Vec<_> = Commented::from(iter);
                let len = ident.as_ref().map_or(0, |i| i.len())
                    + values.iter().map(|n| n.0 + 2).sum::<usize>(); // N elements -> N-1 ", " + "()" -> +2 chars per element

                Value(len, Kind::TupleType(ident, values))
            }

            Rule::fields_type => {
                let mut iter = pair.into_inner().peekable();
                let ident = match iter.peek().unwrap().as_rule() {
                    Rule::ident => Some(iter.next().unwrap().as_str().to_string()),
                    _ => None,
                };

                let fields: Vec<_> = Commented::str_keyed(iter);
                // .map(|field| {
                //     let mut kv_iter = field.into_inner();
                //     let (k, v) = (kv_iter.next().unwrap(), kv_iter.next().unwrap());
                //     (k.as_str().to_string(), Value::from(v))
                // })
                // .collect();
                let len = ident.as_ref().map_or(0, |i| i.len())
                    + fields.iter().map(|(k, v)| k.len() + v.0 + 4).sum::<usize>(); // N fields -> N ": " + N-1 ", " + "()" -> +4 chars per field

                Value(len, Kind::FieldsType(ident, fields))
            }

            Rule::value => Value::from(pair.into_inner().next().unwrap()),

            // Rule::COMMENT => {
            //     let a = pair.as_str().to_string();
            //     Value(a.len(), Kind::BlockComment(a))
            // }

            // handled in other rules
            _ => {
                let a = pair.as_str().to_string();
                println!("{}", a);
                // let ident = match iter.peek().map(|p| p.as_rule()) {
                //     Some(Rule::COMMENT) => Some(iter.next().unwrap().as_str().to_string()),
                //     _ => None,
                // };
                panic!("Unreachacle")
            } // _ => unreachable!(),
        }
    }
}
