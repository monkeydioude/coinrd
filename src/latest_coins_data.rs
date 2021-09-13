use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::coin::Coin;
use crate::database::Collection;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LatestCoinData {
    pub id: String,
    pub symbol: String,
    pub prices: Vec<HashMap<String, f32>>,
    pub updated_at: i64,
    #[serde(skip)]
    prices_max_len: usize,
}

pub fn get_coin_latest_data(id: String, db: &impl Collection<LatestCoinData>) -> Option<LatestCoinData> {
    db.find_one(id)
}

impl LatestCoinData {
    pub fn new(id: String, symbol: String, prices_max_len: usize) -> LatestCoinData {
        LatestCoinData {
            id,
            updated_at: 0,
            symbol,
            prices: vec![],
            prices_max_len,
        }
    }

    pub fn update_with_coin(&mut self, coin: Coin) {
        if self.prices_max_len == 0 {
            return;
        }
        if self.prices.len() >= self.prices_max_len {
            self.prices.remove(0);
        }
        self.prices.push(coin.prices);
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::coin::Coin;
    use super::LatestCoinData;

    #[test]
    pub fn i_can_update_with_coin_if_empty_vec_prices() {
        let mut lcd = LatestCoinData::new("test1".into(), "t1".into(), 1);
        let c = Coin {
            id: "test1".into(),
            symbol: "t1".into(),
            prices: HashMap::new(),
        };
        lcd.update_with_coin(c.clone());
        assert_eq!(c.prices, lcd.prices[0]);
        assert_eq!(lcd.prices.len(), 1);
    }

    #[test]
    pub fn i_can_replace_vec_prices_if_len_is_0() {
        let mut lcd = LatestCoinData::new("test2".into(), "t2".into(), 0);
        let c = Coin {
            id: "test2".into(),
            symbol: "t2".into(),
            prices: HashMap::new(),
        };
        lcd.update_with_coin(c.clone());
        assert_eq!(lcd.prices.len(), 0);
    }

    #[test]
    pub fn i_can_replace_vec_prices_if_len_is_greater_or_equal() {
        let mut lcd = LatestCoinData::new("test3".into(), "t3".into(), 2);
        let c = Coin {
            id: "test3_1".into(),
            symbol: "t3_1".into(),
            prices: HashMap::new(),
        };
        lcd.update_with_coin(c.clone());
        let c = Coin {
            id: "test3_2".into(),
            symbol: "t3_2".into(),
            prices: HashMap::new(),
        };
        lcd.update_with_coin(c.clone());
        let c = Coin {
            id: "test3_3".into(),
            symbol: "t3_3".into(),
            prices: HashMap::new(),
        };
        lcd.update_with_coin(c.clone());
        assert_eq!(c.prices, lcd.prices[1]);
        assert_eq!(lcd.prices.len(), 2);
    }
}