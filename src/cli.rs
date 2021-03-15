use clap::{load_yaml, value_t_or_exit, App};

#[derive(Debug, Clone)]
pub struct CliOptions {
    pub file: String,
    pub url: String,
    pub offset: (u32, u32),
    pub similarity: u32,
    pub shuffle: bool,
    pub time_factor: u32,
}

pub fn get_options() -> CliOptions {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    CliOptions {
        file: matches.value_of("file").unwrap().into(),
        url: matches.value_of("url").unwrap().into(),
        offset: (
            if matches.is_present("offset_x") {
                value_t_or_exit!(matches, "offset_x", u32)
            } else {
                0
            },
            if matches.is_present("offset_y") {
                value_t_or_exit!(matches, "offset_y", u32)
            } else {
                0
            },
        ),
        similarity: if matches.is_present("similarity") {
            value_t_or_exit!(matches, "similarity", u32)
        } else {
            0
        },
        shuffle: if matches.is_present("shuffle") {
            match matches.value_of("shuffle").unwrap() {
                "yes" => true,
                "false" => false,
                _ => panic!("Invalid value for cli argument 'shuffle'"),
            }
        } else {
            true
        },
        time_factor: if matches.is_present("time_factor") {
            value_t_or_exit!(matches, "time_factor", u32)
        } else {
            10
        },
    }
}
