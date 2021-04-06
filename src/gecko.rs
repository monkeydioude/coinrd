use crate::provider::{Provide, Provider};
use reqwest::blocking::{Response, get};
use std::collections::HashMap;

pub fn simple_price(provider: &Provider) -> Option<f32> {
    let uri = format!(
        "{}?ids={}&vs_currencies={}",
        provider.get_uri("simple_price").unwrap(),
        provider.get_coins_string(),
        provider.get_currencies_string(),
    );

    let response: Response;
    
    match get(uri) {
        Ok(res) => response = res,
        Err(err) => {
            println!("{}", err);
            return None
        },
    };

    let coins: HashMap<String, HashMap<String, f32>>;
    coins = serde_json::from_str(response.text().unwrap().as_str()).unwrap();

    println!("{:?}", coins.get("storm").unwrap());
    Some(coins.get("storm").unwrap().get("usd").unwrap().clone())
}