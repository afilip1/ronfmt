use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
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
        .unwrap()
        .into_inner();

    let mut extensions = get_extensions(ron.clone());
    extensions.sort();
    println!("#![enable({})]\n", extensions.join(", "));

    for value in ron {
        if let Rule::value = value.as_rule() {
            println!("{:?}", parse_into_ast(value));
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
enum Ast {
    Atom(String), // atomic types: bool, char, str, int, float
    Tuple(Vec<Ast>),
    List(Vec<Ast>),
    Map(BTreeMap<Ast, Ast>),
    NamedTypeUnit(String),
    NamedTypeTuple(String, Vec<Ast>),
    NamedTypeWithFields(String, BTreeMap<String, Ast>),
}

fn parse_into_ast(value: Pair<Rule>) -> Ast {
    match value.as_rule() {
        // atomics
        Rule::bool | Rule::char | Rule::string | Rule::signed_int | Rule::float => {
            Ast::Atom(value.as_str().into())
        }
        // collections
        Rule::tuple => {
            let comma_separated_values = value.into_inner().next().unwrap();
            let values: Vec<_> = comma_separated_values
                .into_inner()
                .map(|val| parse_into_ast(val))
                .collect();
            Ast::Tuple(values)
        }
        Rule::list => {
            let comma_separated_values = value.into_inner().next().unwrap();
            let values: Vec<_> = comma_separated_values
                .into_inner()
                .map(|val| parse_into_ast(val))
                .collect();
            Ast::List(values)
        }
        Rule::map => {
            let map_inner = value.into_inner().next().unwrap();
            let entries: BTreeMap<Ast, Ast> = map_inner
                .into_inner()
                .map(|entry| {
                    let mut kv_iter = entry.into_inner();
                    let (key, value) = (kv_iter.next().unwrap(), kv_iter.next().unwrap());
                    (parse_into_ast(key), parse_into_ast(value))
                })
                .collect();
            Ast::Map(entries)
        }
        // named types

        // intermediates
        Rule::value => parse_into_ast(value.into_inner().next().unwrap()),
        Rule::comma_separated_values | Rule::map_inner | Rule::map_entry => unreachable!(),
        _ => Ast::Atom("<invalid>".into()),
    }
}

fn get_extensions(ron: Pairs<Rule>) -> Vec<&str> {
    let mut extensions = vec![];
    for item in ron {
        if let Rule::extension = item.as_rule() {
            for ext_name in item.into_inner() {
                extensions.push(ext_name.as_str());
            }
        }
    }
    extensions
}
