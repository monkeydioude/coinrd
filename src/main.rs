pub mod config;
pub mod gecko;
pub mod provider;
pub mod coin;
pub mod latest_coins_data;
pub mod database;

use config::Config;
use coin::Stack;
use database::Collection;
use latest_coins_data::get_coin_latest_data;
use core::time;
use std::thread;
use mongodb::{bson::to_bson, sync::{Client}};
use log::{info, warn, error};
use chrono::Utc;

use crate::latest_coins_data::LatestCoinData;
use crate::database::MongoDB;
const F: u32 = 4;
const S: u64 = 64;

static LATEST_ENTRIES_COLL: &str = "latest_entries";

fn db_connection (config: &Config) -> MongoDB {
    let client = match Client::with_uri_str(config.mongodb_uri.as_str()) {
        Ok(c) => c,
        Err(err) => {
            error!("Could not establish connection to DB: {}", err);
            panic!("{}", err);
        },
    };
    
    MongoDB::new(client.database("coins"))
}

fn save_coins_stack(coins: &Stack, db: &MongoDB) {
    let doc = to_bson(coins).unwrap().as_document().unwrap().to_owned();
    match db.to_mongo_db().collection("price_history").insert_one(doc, None) {
        Err(err) => error!("Err save_coins_stack: {}", err),
        _ => (),
    };
}

fn save_latest_entries(coins: &Stack, db: &MongoDB, prices_max_len: usize) {
    let coll = db.new_collection(LATEST_ENTRIES_COLL);

    for c in coins.coins.iter() {
        let coin = c.1.to_owned();
        let mut latest_coins = match get_coin_latest_data(coin.id.to_owned(), &coll) {
            Some(b) => b,
            None => LatestCoinData::new(coin.id.to_owned(), coin.symbol.to_owned()),
        };

        latest_coins.updated_at = Utc::now().timestamp_millis();
        latest_coins.update_with_coin(coin, prices_max_len);
        &coll.save::<LatestCoinData>(latest_coins.id.to_owned(), latest_coins);
        // latest_coins.save(db);
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
                    save_latest_entries(&trimmed_coins, &db, config.prices_max_len)
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