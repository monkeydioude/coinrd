use std::collections::HashMap;

pub struct Coin {
    id: String,
    symbol: String,
    prices: HashMap<String, f32>
}