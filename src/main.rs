pub mod config;
pub mod gecko;
pub mod provider;
pub mod coin;

use config::Config;
use coin::{Stack};
use core::time;
use std::thread;
use mongodb::{bson::{to_bson, doc}, options::ReplaceOptions, sync::{Client, Database}};
use log::{info, warn, error};
use chrono::Utc;

const F: u32 = 4;
const S: u64 = 64;

fn db_connection (config: &Config) -> Database {
    let client = match Client::with_uri_str(config.mongodb_uri.as_str()) {
        Ok(c) => c,
        Err(err) => {
            error!("Could not establish connection to DB: {}", err);
            panic!("{}", err);
        },
    };
    client.database("coins")
}

fn save_coins_stack(coins: &Stack, db: &Database) {
    let doc = to_bson(coins).unwrap().as_document().unwrap().to_owned();
    match db.collection("price_history").insert_one(doc, None) {
        Err(err) => error!("Err save_coins_stack: {}", err),
        _ => (),
    };
}

fn save_latest_entries(coins: &Stack, db: &Database) {
    let collection = db.collection("latest_entries");

    for c in coins.coins.iter() {
        let mut coin = c.1.to_owned();
        coin.updated_at = Some(Utc::now().timestamp_millis());

        match collection.replace_one(
            doc!{"id": coin.id.to_owned()},
            to_bson(&coin).unwrap().as_document().unwrap().to_owned(), 
            ReplaceOptions::builder().upsert(true).build(),
        ) {
            Err(err) => error!("Err save_latest_entries: {}", err),
            _ => (),
        };
    }
}

fn should_update_providers(c_f: u32) -> bool {
    c_f == F
}

fn main() {
    let mut cur_f = 0;
    let config = Config::new();
    
    let db = db_connection(&config);

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
                info!("{:?}", &trimmed_coins);

                if trimmed_coins.coins.len() > 0 {
                    save_coins_stack(&trimmed_coins, &db);
                    save_latest_entries(&trimmed_coins, &db)
                }
                coins_cache = cache;
            },
            Err(err) => {
                warn!("{}", err);
                continue
            }
        };

        info!("Going for a siesta for {}s", S);
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