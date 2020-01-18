use crate::ast;

use pest::Parser;
use pest_derive::*;

#[derive(Parser)]
#[grammar = "ron.pest"]
struct RonParser;

pub fn parse_ron(input: &str) -> ast::FileText {
    let pest_ir = RonParser::parse(Rule::ron_file, &input)
        .expect("unable to parse RON")
        .next()
        .unwrap(); // never fails according to pest docs

    ast::FileText::parse_from(pest_ir)
}
