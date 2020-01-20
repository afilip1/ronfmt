use clap::clap_app;

pub struct Config {
    pub soft_tab_width: usize,
    pub max_line_width: usize,
    pub target_file_path: String,
    pub format_in_place: bool,
    pub with_backup: bool,
}

impl Config {
    const SOFT_TAB_WIDTH: usize = 4;
    const MAX_LINE_WIDTH: usize = 40;
}

pub fn get_config() -> Config {
    let matches = get_clap_matches();

    let target_file_path = matches.value_of("INPUT").unwrap().to_string();

    let soft_tab_width = matches
        .value_of("TAB_SIZE")
        .map(|stw| stw.parse().expect("invalid tab size"))
        .unwrap_or(Config::SOFT_TAB_WIDTH);

    let max_line_width = matches
        .value_of("MAX_LINE_WIDTH")
        .map(|mlw| mlw.parse().expect("invalid line width"))
        .unwrap_or(Config::MAX_LINE_WIDTH);

    let format_in_place = matches.is_present("inplace");

    let with_backup = !matches.is_present("nobackup");

    Config {
        target_file_path,
        soft_tab_width,
        max_line_width,
        format_in_place,
        with_backup,
    }
}

fn get_clap_matches<'a>() -> clap::ArgMatches<'a> {
    clap_app!(app =>
        (version: "0.1.0")
        (author: "Anton F. <afilip@fastmail.com>")
        (about: "Utility for autoformatting RON files.")
        (@arg INPUT: +required "Sets which file to format")
        (@arg MAX_LINE_WIDTH: -w +takes_value "Sets soft max line width for formatting heuristics")
        (@arg TAB_SIZE: -t +takes_value "Sets indentation size in spaces")
        (@arg inplace: -i "Formats target file in-place.")
        (@arg nobackup: --("no-backup") "Prevents creation of a backup of the target file when formatting in-place; does nothing when formatting to stdout")
    )
    .get_matches()
}
