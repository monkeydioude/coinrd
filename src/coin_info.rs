use chrono::Utc;
use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize)]
pub struct CoinInfo {
  id: String,
  symbol: String,
  created_at: i64,
}


impl CoinInfo {
  pub fn new(id: &str, symbol: &str) -> Self {
      Self {
        id: id.to_string(),
        symbol: symbol.to_string(),
        created_at: Utc::now().timestamp_millis(),
      }
  }
}