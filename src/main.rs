use pest_derive::*;

use pest::Parser;

#[derive(Parser)]
#[grammar = "ron.pest"]
struct RonParser;

fn main() {
    let file = std::fs::read_to_string("test.ron").unwrap();
    let ron = RonParser::parse(Rule::ron_file, &file);
    println!("{:#?}", ron);
}
