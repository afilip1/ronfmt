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

    let mut extensions = vec![];

    for item in ron.into_inner() {
        match item.as_rule() {
            Rule::extension => {
                for ext_name in item.into_inner() {
                    extensions.push(ext_name.as_str());
                }
            }
            Rule::value => {}
            Rule::EOI => {}
            _ => unreachable!(),
        }
    }

    println!("#![enable({})]", extensions.join(", "));
}
