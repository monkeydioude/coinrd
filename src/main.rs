pub mod config;
pub mod gecko;
pub mod provider;

use config::Config;
use core::time;
use std::thread;

fn main() {
    let config = Config::new();
    let mut s= 61u64;

    let providers = provider::list_from_toml(config.ref_file.clone()).unwrap();

    let coingecko = match providers.get("coingecko") {
        Some(p) => p,
        None => panic!("coingecko config in {} file must be provided", config.ref_file.clone()),
    };
    let mut previous_p: f32 = 0.0;

    loop {
        let curr_p ;
        match gecko::simple_price(coingecko) {
            Some(p) => curr_p = p,
            None => continue
        };

        if curr_p != previous_p {
            s = s - 1;
        } else {
            s = s + 1;
        }
        previous_p = curr_p;
        println!("going for a siesta for {}s", s);
        thread::sleep(time::Duration::from_secs(s));
    }
}
