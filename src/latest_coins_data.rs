use std::collections::HashMap;
use serde::{Serialize, Deserialize};

static PRICES_MAX_LEN: usize = 2;
static LATEST_ENTRIES_COLL: &str = "latest_entries_test";

use crate::coin::Coin;
use crate::database::Database;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LatestCoinData {
    pub id: String,
    pub symbol: String,
    pub prices: Vec<HashMap<String, f32>>,
    pub updated_at: i64,
    #[serde(skip_deserializing)]
    pub prices_diff_with_next: HashMap<String, f32>,
    #[serde(skip_deserializing)]
    pub prices_diff_with_last: HashMap<String, f32>,
}

pub fn get_coin_latest_data(id: String, db: impl Database) -> Option<LatestCoinData> {
    db.find_one::<LatestCoinData>(id, LATEST_ENTRIES_COLL)
}

impl LatestCoinData {
    pub fn save(&self, db: impl Database) {
        db.save(self.id.to_owned(), self, LATEST_ENTRIES_COLL);
    }
    
    pub fn new(id: String, symbol: String) -> LatestCoinData {
        return LatestCoinData {
            id,
            updated_at: 0,
            symbol,
            prices_diff_with_next: HashMap::new(),
            prices_diff_with_last: HashMap::new(),
            prices: vec![],
        }
    }

    pub fn update_with_coin(&mut self, coin: Coin) {
        if self.prices.len() >= PRICES_MAX_LEN {
            self.prices.remove(0);
        }
        self.prices.push(coin.prices);
    }
}