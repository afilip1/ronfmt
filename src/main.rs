mod ast;
mod config;
mod parser;

use std::fs;

fn main() {
    let config = config::get_config();
    let input = fs::read_to_string(&config.target_file_path)
        .expect("unable to read file");

    let ron_ast = parser::parse_ron(&input);
    let ron_formatted = ron_ast.pretty_print(&config);

    if config.format_in_place {
        if config.with_backup {
            create_backup(&config.target_file_path);
        }

        fs::write(&config.target_file_path, ron_formatted)
            .expect("unable to overwrite target file");
    } else {
        println!("{}", ron_formatted);
    }
}

fn create_backup(target_file_path: &str) {
    fs::copy(target_file_path, format!("{}.bak", target_file_path))
        .expect("unable to create backup file");
}
