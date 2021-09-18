pub mod config;
pub mod gecko;
pub mod provider;
pub mod coin;
pub mod latest_coins_data;
pub mod database;
pub mod coin_info;

use config::Config;
use coin::Stack;
use database::{Collection};
use latest_coins_data::get_coin_latest_data;
use provider::{Provide, Provider};
use core::time;
use std::thread;
use mongodb::sync::{Client};
use log::{info, warn, error};
use chrono::Utc;

use crate::coin_info::CoinInfo;
use crate::latest_coins_data::LatestCoinData;
use crate::database::MongoDB;
const F: u32 = 4;
const S: u64 = 64;

// db_connection returns a MongoDB struct wrapping
// around Mongo DB connector.
fn db_connection (mongodb_uri: String) -> MongoDB {
    let client = match Client::with_uri_str(&mongodb_uri) {
        Ok(c) => c,
        Err(err) => {
            error!("Could not establish connection to DB: {}", err);
            panic!("{}", err);
        },
    };
    
    MongoDB::new(client.database("coins"))
}

// save_coins_stack stores a batch of trimmed coins in a single new
// MongoDB Document
fn save_coins_stack(coins: &Stack, db: &MongoDB) {
    db.new_collection::<Stack>("price_history").insert(coins);
}

// save_latest_entries stores the x lasts (x = prices_max_len) into a single document
// organized by currency id 
fn save_latest_entries(coins: &Stack, db: &MongoDB, prices_max_len: usize) {
    let coll = db.new_collection::<LatestCoinData>("latest_entries");

    for c in coins.coins.iter() {
        let coin = c.1.to_owned();
        let mut latest_coins = match get_coin_latest_data(coin.id.to_owned(), &coll) {
            Some(mut lcd) => {
                lcd.set_prices_max_len(prices_max_len);
                lcd
            },
            None => continue,
        };

        latest_coins.updated_at = Utc::now().timestamp_millis();
        latest_coins.update_with_coin(coin);
        coll.save(latest_coins.id.to_owned(), &latest_coins);
    }
}

fn should_update_providers(c_f: u32) -> bool {
    c_f == F
}

fn update_coingecko_list_provider_routine(ref_file: &str, collection: impl Collection<CoinInfo>) -> Result<Provider, String> {
    let coingecko = match provider::update_provider(ref_file, "coingecko") {
        Some(c) => c,
        _ => return Err("Could not find matching provider".to_owned()),
    };

    for coin in coingecko.get_coins() {
        collection.save(coin.0.to_owned(), &CoinInfo::new(coin.0, coin.1));
    }

    Ok(coingecko)
}

fn main() {
    // so should_update_providers triggers straight away
    let mut cur_f = 4;
    let config = Config::parse();
    
    let db = db_connection(config.mongodb_uri);

    let mut coingecko = match provider::update_provider(&config.ref_file, "coingecko") {
        Some(c) => c,
        _ => panic!("coingecko config in {} file must be provided", &config.ref_file),
    };

    let mut coins_cache = Stack::new();

    loop {
        if should_update_providers(cur_f) {
            coingecko = match update_coingecko_list_provider_routine(
                &config.ref_file,
                db.new_collection::<CoinInfo>("coin_info"),
            ) {
                Ok(fresh_gecko) => fresh_gecko,
                Err(err) => {
                    warn!("{}", err);
                    coingecko
                },
            };
            cur_f = 0;
        }

        match gecko::simple_price(&coingecko) {
            Ok(coins) => {
                let trimmed_coins = coin::trim_nonupdated_coins(&coins_cache, &coins);
                info!("{:?}", &trimmed_coins);

                if trimmed_coins.coins.len() > 0 {
                    save_coins_stack(&trimmed_coins, &db);
                    save_latest_entries(&trimmed_coins, &db, config.prices_max_len)
                }
                coins_cache = coins;
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