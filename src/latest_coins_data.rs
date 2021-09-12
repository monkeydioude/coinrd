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
}

pub fn get_coin_latest_data(id: String, db: &impl Collection) -> Option<LatestCoinData> {
    db.find_one::<LatestCoinData>(id)
}

impl LatestCoinData {
    pub fn save(&self, db: impl Collection) {
        db.save(self.id.to_owned(), self);
    }
    
    pub fn new(id: String, symbol: String) -> LatestCoinData {
        LatestCoinData {
            id,
            updated_at: 0,
            symbol,
            prices: vec![],
        }
    }

    pub fn update_with_coin(&mut self, coin: Coin, prices_max_len: usize) {
        if self.prices.len() >= prices_max_len {
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
        let mut lcd = LatestCoinData::new("test_first".into(), "tf".into());
        let c = Coin {
            id: "test_first".into(),
            symbol: "tf".into(),
            prices: HashMap::new(),
        };
        lcd.update_with_coin(c.clone(), 1);
        assert_eq!(c.prices, lcd.prices[0]);
    }
}