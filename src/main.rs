#![allow(clippy::redundant_closure)] // blame pest

mod ast;

use pest::Parser;
use pest_derive::*;

#[derive(Parser)]
#[grammar = "ron.pest"]
struct RonParser;

fn main() {
    let file = std::fs::read_to_string("test.ron").expect("unable to read file");
    let ron = RonParser::parse(Rule::ron_file, &file)
        .expect("unable to parse RON")
        .next()
        .unwrap();

    println!("{}", ast::Node::from(ron));
}
