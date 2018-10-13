use pest::{iterators::Pair, Parser};
use pest_derive::*;
use std::collections::BTreeMap;

#[derive(Parser)]
#[grammar = "ron.pest"]
struct RonParser;

fn main() {
    let file = std::fs::read_to_string("test.ron").expect("unable to read file");
    let ron = RonParser::parse(Rule::ron_file, &file)
        .expect("unable to parse RON")
        .next()
        .unwrap();

    Ast::parse_from(ron).format(0, true);
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
enum Ast {
    Ron(Vec<String>, Box<Ast>),
    Atom(String), // atomic types: bool, char, str, int, float
    Tuple(Vec<Ast>),
    List(Vec<Ast>),
    Map(BTreeMap<Ast, Ast>),
    NamedTypeUnit(String),
    NamedTypeTuple(String, Vec<Ast>),
    NamedTypeFields(String, BTreeMap<String, Ast>),
    AnonymousTypeFields(BTreeMap<String, Ast>),
}

impl Ast {
    fn parse_from(value: Pair<Rule>) -> Ast {
        let rule = value.as_rule();
        match rule {
            // atomics
            Rule::bool | Rule::char | Rule::string | Rule::signed_int | Rule::float => {
                Ast::Atom(value.as_str().into())
            }
            // collections
            Rule::tuple | Rule::list => {
                let comma_separated_values = value.into_inner().next().unwrap();
                let values: Vec<_> = comma_separated_values
                    .into_inner()
                    .map(|val| Ast::parse_from(val))
                    .collect();

                match rule {
                    Rule::tuple => Ast::Tuple(values),
                    Rule::list => Ast::List(values),
                    _ => unreachable!(),
                }
            }
            Rule::map => {
                if let Some(map_inner) = value.into_inner().next() {
                    let entries: BTreeMap<Ast, Ast> = map_inner
                        .into_inner()
                        .map(|entry| {
                            let mut kv_iter = entry.into_inner();
                            let (key, value) = (kv_iter.next().unwrap(), kv_iter.next().unwrap());
                            (Ast::parse_from(key), Ast::parse_from(value))
                        })
                        .collect();
                    Ast::Map(entries)
                } else {
                    Ast::Map(Default::default())
                }
            }
            // named types
            Rule::named_type_unit => Ast::NamedTypeUnit(value.as_str().into()),
            Rule::named_type_tuple => {
                let mut iter = value.into_inner();
                let (ident, tuple) = (iter.next().unwrap(), iter.next().unwrap());
                match Ast::parse_from(tuple) {
                    Ast::Tuple(inner) => Ast::NamedTypeTuple(ident.as_str().into(), inner),
                    _ => unreachable!(),
                }
            }
            Rule::named_type_fields => {
                let mut iter = value.into_inner();
                let (ident, fields_iter) = (iter.next().unwrap(), iter.next().unwrap());
                let fields: BTreeMap<String, Ast> = fields_iter
                    .into_inner()
                    .map(|field| {
                        let mut field_iter = field.into_inner();
                        let (field_name, field_value) =
                            (field_iter.next().unwrap(), field_iter.next().unwrap());
                        (field_name.as_str().into(), Ast::parse_from(field_value))
                    })
                    .collect();
                Ast::NamedTypeFields(ident.as_str().into(), fields)
            }
            Rule::anonymous_type_fields => {
                let fields_iter = value.into_inner().next().unwrap();
                let fields: BTreeMap<String, Ast> = fields_iter
                    .into_inner()
                    .map(|field| {
                        let mut field_iter = field.into_inner();
                        let (field_name, field_value) =
                            (field_iter.next().unwrap(), field_iter.next().unwrap());
                        (field_name.as_str().into(), Ast::parse_from(field_value))
                    })
                    .collect();
                Ast::AnonymousTypeFields(fields)
            }
            Rule::ron_file => {
                let ron = value.into_inner();
                let mut extensions = vec![];
                let mut value = Ast::Atom("".into());
                for item in ron {
                    match item.as_rule() {
                        Rule::extension => {
                            for ext_name in item.into_inner() {
                                extensions.push(ext_name.as_str().into());
                            }
                        }
                        Rule::value => value = Ast::parse_from(item),
                        _ => {}
                    }
                }
                Ast::Ron(extensions, Box::new(value))
            }
            // intermediates and aggregates
            Rule::value | Rule::named_type => Ast::parse_from(value.into_inner().next().unwrap()),
            // handled in other rules
            _ => unreachable!(),
        }
    }

    fn format(&self, indent_level: usize, indent_first_line: bool) {
        fn indent(level: usize) -> String {
            const TAB_SIZE: usize = 4;
            " ".repeat(TAB_SIZE * level)
        }

        if indent_first_line {
            print!("{}", indent(indent_level));
        }

        match self {
            Ast::Ron(extensions, value) => {
                if !extensions.is_empty() {
                    let mut sorted_exts = extensions.clone();
                    sorted_exts.sort_unstable();

                    print!("#![enable({})]\n\n", sorted_exts.join(", "));
                }

                value.format(0, true);
            }

            Ast::Atom(atom) => print!("{}", atom),

            Ast::Tuple(elements) => {
                print!("(\n");
                for elem in elements {
                    elem.format(indent_level + 1, true);
                    print!(",\n");
                }
                print!("{indent})", indent = indent(indent_level));
            },

            Ast::List(elements) => {
                print!("[\n");
                for elem in elements {
                    elem.format(indent_level + 1, true);
                    print!(",\n");
                }
                print!("{indent}]", indent = indent(indent_level));
            }

            Ast::Map(entries) => {
                print!("{{\n");
                for (key, value) in entries {
                    key.format(indent_level + 1, true);
                    print!(": ");
                    value.format(indent_level + 1, false);
                    print!(",\n");
                }
                print!("{indent}}}", indent = indent(indent_level));
            }

            Ast::NamedTypeUnit(ident) => print!("{}", ident),

            Ast::NamedTypeTuple(ident, elements) => {
                print!("{}(\n", ident);
                for elem in elements {
                    elem.format(indent_level + 1, true);
                    print!(",\n");
                }
                print!("{indent})", indent = indent(indent_level));
            }

            Ast::NamedTypeFields(ident, fields) => {
                print!("{} (\n", ident);
                for (field_name, field_value) in fields {
                    print!("{indent}{}: ", field_name, indent = indent(indent_level + 1));
                    field_value.format(indent_level + 1, false);
                    print!(",\n");
                }
                print!("{indent})", indent = indent(indent_level));
            }

            Ast::AnonymousTypeFields(fields) => {
                print!("(\n");
                for (field_name, field_value) in fields {
                    print!("{indent}{}: ", field_name, indent = indent(indent_level + 1));
                    field_value.format(indent_level + 1, false);
                    print!(",\n");
                }
                print!("{indent})", indent = indent(indent_level));
            }
        }
    }
}
