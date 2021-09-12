use std::env;
use log::warn;

pub struct Config {
    pub ref_file: String,
    pub mongodb_uri: String,
    pub prices_max_len: usize,
}

impl Config {
    pub fn new() -> Config {
        let ref_file = match env::var("REF_FILE") {
            Ok(rf) => rf,
            Err(err) => panic!("Problem retrieving REF_FILE env var: {}", err),
        };

        let mongodb_uri = match env::var("MONGODB_URI") {
            Ok(mu) => mu,
            Err(err) => panic!("Problem retrieving MONGODB_URI env var: {}", err),
        };

        let prices_max_len = match env::var("PRICES_MAX_LEN") {
            Ok(pml) => pml.parse::<usize>().unwrap(),
            Err(err) => {
                warn!("{}", err);
                2
            },
        };

        Config {
            ref_file,
            mongodb_uri,
            prices_max_len,
        }
    }
}