use std::env;

pub struct Config {
    pub ref_file: String,
    pub mongodb_uri: String,
}

impl Config {
    pub fn new() -> Config {
        let ref_file = match env::var("REF_FILE") {
            Ok(rf) => rf,
            Err(err) => panic!("Problem retrieving REF_FILE env var: {}", err)
        };

        let mongodb_uri = match env::var("MONGODB_URI") {
            Ok(mu) => mu,
            Err(err) => panic!("Problem retrieving MONGODB_URI env var: {}", err)
        };

        Config {
            ref_file,
            mongodb_uri,
        }
    }
}