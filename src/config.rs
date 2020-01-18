use clap::clap_app;

pub struct Config {
    pub soft_tab_width: usize,
    pub max_line_width: usize,
    pub target_file_path: String,
    pub format_in_place: bool,
    pub with_backup: bool,
}

impl Config {
    fn new(target_file_path: &str) -> Config {
        Config {
            target_file_path: target_file_path.to_string(),
            soft_tab_width: 4,
            max_line_width: 40,
            format_in_place: false,
            with_backup: true,
        }
    }
}

pub fn get_config() -> Config {
    let matches = get_clap_matches();

    let target_file_path = matches.value_of("INPUT").unwrap();
    let mut config = Config::new(target_file_path);

    if let Some(max_line_width) = matches.value_of("MAX_LINE_WIDTH") {
        config.max_line_width = str::parse(max_line_width).unwrap();
    }

    if let Some(soft_tab_width) = matches.value_of("TAB_SIZE") {
        config.soft_tab_width = str::parse(soft_tab_width).unwrap();
    }

    if matches.is_present("inplace") {
        config.format_in_place = true;

        if matches.is_present("nobackup") {
            config.with_backup = false;
        }
    }

    config
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
