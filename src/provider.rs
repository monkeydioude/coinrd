use serde::Deserialize;
use std::collections::HashMap;
use std::{fs, io::Error};

// Provide defines a Provider behavior
pub trait Provide {
    fn get_name(&self) -> &String;
    fn get_coins(&self) -> &Vec<String>;
    fn get_coins_string(&self) -> String {
        self.get_coins().join(",")
    }
    fn get_base_route(&self) -> &String;
    fn get_routes(&self) -> &HashMap<String, String>;
    fn get_uri(&self, route: &str) -> Option<String> {
        match self.get_routes().get(route) {
            Some(r) => Some(self.get_base_route().clone() + r),
            None => None,
        }
    }
    fn get_currencies(&self) -> &Vec<String>;
    fn get_currencies_string(&self) -> String {
        self.get_currencies().join(",")
    }
}

// Provider is the definition of a service that should be
// deserialized from config
#[derive(Deserialize)]
pub struct Provider {
    name: String,
    coins: Vec<String>,
    base_route: String,
    routes: HashMap<String, String>,
    currencies: Vec<String>,
}

impl Provide for Provider {
    fn get_name(&self) -> &String {
        &self.name
    }

    fn get_base_route(&self) -> &String {
        &self.base_route
    }

    fn get_coins(&self) -> &Vec<String> {
        &self.coins
    }

    fn get_routes(&self) -> &HashMap<String, String> {
        &self.routes
    }

    fn get_currencies(&self) -> &Vec<String> {
        &self.currencies
    }
}

#[derive(Deserialize)]
struct Providers {
    providers: HashMap<String, Provider>
}

// list_from_toml generates a providers list from
// a config toml file.
pub fn list_from_toml(filepath: String) -> Result<HashMap<String, Provider>, Error> {
    let content = match fs::read_to_string(filepath) {
        Ok(c) => c,
        Err(err) => return Err(err),
    };

    let plist: Providers = toml::from_str(&content).unwrap();
    Ok(plist.providers)
}

#[cfg(test)]
mod tests {
    #[test]
    fn i_should_trigger_error_on_unknown_file() {
        match super::list_from_toml("pouet".into()) {
            Ok(_) => panic!("list_from_file should not return Ok()"),
            _ => "ok",
        };
    }

    #[test]
    fn i_should_parse_providers_file() {
        let plist = match super::list_from_toml("./test/providers-test.toml".into()) {
            Ok(plist) => plist,
            Err(err) => panic!("i_should_parse_providers_file should return Ok: {}", err) ,
        };

        use super::Provide;

        assert_eq!(plist.get("coingecko").unwrap().routes.get("ping").unwrap(), "/ping");
        assert_eq!(plist.get("coingecko").unwrap().get_uri("ping").unwrap(), "https://api.coingecko.com/api/v3/ping");
    }
}