use clap::{load_yaml, value_t_or_exit, App};

#[derive(Debug, Clone)]
pub struct CliOptions {
    pub file: String,
    pub url: String,
    pub offset: (u32, u32),
    pub alpha_color: [u8; 6],
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
        alpha_color: if let Some(mut s) = matches.value_of("alpha-color") {
            if s.starts_with('#') {
                s = &s[1..];
            }
            if s.len() < 6 {
                *b"ffffff"
            } else {
                use std::convert::TryInto;
                s.as_bytes()[0..6].try_into().unwrap()
            }
        } else {
            *b"ffffff"
        },
    }
}
