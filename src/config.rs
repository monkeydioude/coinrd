use std::env;

pub struct Config {
    pub ref_file: String,
}

impl Config {
    pub fn new() -> Config {
        let ref_file = match env::var("REF_FILE") {
            Ok(ref_file) => ref_file,
            Err(err) => panic!("Problem retrieving REF_FILE env var: {}", err)
        };

        Config {
            ref_file: ref_file.into(),
        }
    }
}