use crate::provider::{Provide, Provider};
use crate::coin::{Coin, Stack};
use reqwest::blocking;
use std::collections::HashMap;
use chrono::Utc;

// format_coin_data transforms a gecko api response
// into a HashMap of Coin
fn format_coin_data(
    mut coins_data: HashMap<String, HashMap<String, f32>>,
    coins_config: &HashMap<String, String>
) -> Option<HashMap<String, Coin>> {
    let mut coins: HashMap<String, Coin> = HashMap::new();

    for (id, prices) in coins_data.drain() {
        let symbol = match coins_config.get(&id) {
            Some(l ) => l.to_owned(),
            None => continue,
        };

        coins.insert(id.clone(), Coin {
            id,
            symbol,
            prices,
        });
    };

    if coins.len() == 0 {
        return None
    }
    Some(coins)
}

// simple_price gives price in specified currencies for spcific cryptocurrencies
pub fn simple_price(provider: &Provider) -> Result<Stack, String> {
    let uri = format!(
        "{}?ids={}&vs_currencies={}",
        provider.get_uri("simple_price").unwrap(),
        provider.get_coins_string(),
        provider.get_currencies_string(),
    );

    // match response
    let response_string: String = match blocking::get(uri) {
        // match render of content
        Ok(response) => 
            match response.text() {
                Ok(res) => res,
                Err(err) => return Err(err.to_string()),
            },
        Err(err) => return Err(err.to_string()),
    };

    let coins_data = match serde_json::from_str(response_string.as_str()) {
        Ok(data) => format_coin_data(data, provider.get_coins()),
        Err(err) =>  return Err(err.to_string()),
    };

    match coins_data {
        Some(coins) => Ok(Stack {
            coins,
            created_at: Utc::now().timestamp_millis(),
        }),
        None => Err(String::from("Could not retrieve any coin data")),
    }
}