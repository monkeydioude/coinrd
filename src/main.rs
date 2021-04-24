pub mod config;
pub mod gecko;
pub mod provider;
pub mod coin;

use config::Config;
use coin::Stack;
use core::time;
use std::thread;
use mongodb::{
    bson::{to_bson},
    sync::{Client, Collection},
};

const F: u32 = 4;
const S: u64 = 60;

fn db_connection (config: &Config) -> Collection {
    let client = match Client::with_uri_str(config.mongodb_uri.as_str()) {
        Ok(c) => c,
        Err(err) => panic!("{}", err),
    };
    let db = client.database("coins");
    db.collection("price_history")
}

fn save_coins_data(coins: &Stack, collection: &Collection) {
    let doc = match to_bson(&coins) {
        Ok(bs) => match bs.as_document() {
            Some(d) => d.to_owned(),
            None => return println!("Could not convert into document"),
        },
        Err(err) => return println!("{}", err),
    };
    match collection.insert_one(doc.to_owned(), None) {
        Err(err) => println!("{}", err),
        _ => (),
    };
}

fn should_update_providers(c_f: u32) -> bool {
    c_f == F
}

fn main() {
    let mut cur_f = 0;
    let config = Config::new();
    
    let collection = db_connection(&config);

    let mut coingecko = match provider::update_provider(&config.ref_file, "coingecko") {
        Some(c) => c,
        _ => panic!("coingecko config in {} file must be provided", config.ref_file.clone()),
    };

    let mut coins_cache = Stack::new();

    loop {
        if should_update_providers(cur_f) {
            coingecko = match provider::update_provider(&config.ref_file, "coingecko") {
                Some(c) => c,
                _ => coingecko,
            };
            cur_f = 0;
        }

        match gecko::simple_price(&coingecko) {
            Ok(coins) => {
                let (trimmed_coins, cache) = coin::trim_nonupdated_coins(&coins_cache, &coins);
                println!("[INFO] {:?}", &trimmed_coins);

                if trimmed_coins.coins.len() > 0 {
                    save_coins_data(&trimmed_coins, &collection);
                }
                coins_cache = cache;
            },
            Err(err) => {
                print!("{}", err);
                continue
            }
        };

        println!("[INFO] going for a siesta for {}s", S);
        thread::sleep(time::Duration::from_secs(S));
        cur_f = cur_f + 1;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn should_test_should_update_providers() {
        let mut it = 0;
        let mut trial = false;

        while it < super::F {
            trial = super::should_update_providers(it);
            it = it + 1;
        }
        assert_eq!(trial, false);

        it = 0;
        while it <= super::F {
            trial = super::should_update_providers(it);
            it = it + 1;
        }
        assert_eq!(trial, true);

        it = 0;
        while it <= 7 {
            trial = super::should_update_providers(it);
            it = it + 1;
        }
        assert_eq!(trial, false);
    }
}