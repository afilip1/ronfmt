#![allow(clippy::redundant_closure)] // blame pest

mod ast;

use pest::Parser;
use pest_derive::*;

#[derive(Parser)]
#[grammar = "ron.pest"]
struct RonParser;

fn main() {
    let target_path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: ronfmt <path_to_file>");
        std::process::exit(1);
    });

    let file = std::fs::read_to_string(&target_path).expect("unable to read file");
    std::fs::copy(&target_path, format!("{}.bak", &target_path))
        .expect("unable to create backup file");

    let ron = RonParser::parse(Rule::ron_file, &file)
        .expect("unable to parse RON")
        .next()
        .unwrap();

    std::fs::write(&target_path, format!("{}", ast::RonFile::parse_from(ron)))
        .expect("unable to overwrite target file");
}
