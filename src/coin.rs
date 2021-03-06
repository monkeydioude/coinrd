use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Coin {
    pub id: String,
    pub symbol: String,
    pub prices: HashMap<String, f32>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Stack {
    pub coins: HashMap<String, Coin>,
    pub created_at: i64,
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            coins: HashMap::new(),
            created_at: 0,
        }
    }
}


// trim_nonupdated_coins compare with previously retrieved coins and filters out
// ones that price hasn't change
pub fn trim_nonupdated_coins(cache: &Stack, future: &Stack) -> Stack {
    let mut trimmed = Stack::new();
    trimmed.created_at = future.created_at;

    for (name, coin) in future.coins.iter() {
        match cache.coins.get(name) {
            Some(pcoin) => {
                let (p_usd, f_usd) = match (pcoin.prices.get("usd"), coin.prices.get("usd")) {
                    (Some(p), Some(f)) => (*p, *f),
                    _ => continue,
                };
                if p_usd == f_usd {
                    continue
                }
                trimmed.coins.insert(name.to_string(), coin.to_owned());
            },
            None => {
                trimmed.coins.insert(name.to_string(), coin.to_owned());
            },
        };
    }

    trimmed
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::{Stack, Coin, trim_nonupdated_coins};

    fn gen_hashmap<'a, T>(keys: Vec<&str>, items: Vec<T>) -> HashMap<String, T>
    where T: Clone {
        let mut hmap: HashMap<String, T> = HashMap::new();

        if keys.len() != items.len() {
            return hmap;
        }
        for it in 0..keys.len() {
            hmap.insert(keys[it].to_string(), items[it].clone());
        }

        hmap
    }

    #[test]
    fn trim_should_return_empty_coins_hashmap() {
        let trial = Stack {
            coins: gen_hashmap(
                vec!["coinoyaro"], 
                vec![
                    Coin {
                        id: "coinoyaro".to_string(),
                        symbol: "con".to_string(),
                        prices: gen_hashmap(vec!["wsh"], vec![4.20f32]),
                    }
                ]),
            created_at: 0,
        };
        let cache = trial.clone();
        let goal = trim_nonupdated_coins(&cache, &trial);
        assert_eq!(goal.coins.len(), 0);
        assert_eq!(cache.coins.get("coinoyaro").unwrap().id, "coinoyaro");
        assert_eq!(
            cache.coins.get("coinoyaro").unwrap().prices.get("wsh").unwrap().to_owned(),
            4.20f32
        );
    }

    #[test]
    fn trim_should_return_n1_coins_hashmap() {
        let cached_coin = Coin {
            id: "cached1".to_string(),
            symbol: "cac".to_string(),
            prices: gen_hashmap(vec!["wsh"], vec![4.20f32]),
        };
        let og_coin = Coin {
            id: "og1".to_string(),
            symbol: "og".to_string(),
            prices: gen_hashmap(vec!["wsh"], vec![6.969f32]),
        };

        let trial = Stack {
            coins: gen_hashmap(
                vec!["cached1", "og1"],
                vec![
                    cached_coin.clone(),
                    og_coin.clone(),
                ]
            ),
            created_at: 0,
        };

        let cache = Stack {
            coins: gen_hashmap(vec!["cached1"], vec![cached_coin]),
            created_at: 0,
        };

        let goal = trim_nonupdated_coins(&cache, &trial);
        assert_eq!(goal.coins.len(), 1);
        assert_eq!(goal.coins.get("og1").unwrap().id, "og1");
        assert_eq!(cache.coins.get("cached1").unwrap().id, "cached1");
        assert_eq!(trial.coins.get("og1").unwrap().id, "og1");
    }
}