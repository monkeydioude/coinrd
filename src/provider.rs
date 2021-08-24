use serde::Deserialize;
use std::collections::HashMap;
use std::{fs, io::Error};
use log::{info, error};

// Provide defines a Provider behavior
pub trait Provide {
    fn get_name(&self) -> &String;
    fn get_coins(&self) -> &HashMap<String, String>;
    fn get_coins_string(&self) -> String {
        self.get_coins()
        .keys()
        .map(|s: &String| s.to_owned())
        .collect::<Vec<String>>()
        .join(",")
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
#[derive(Deserialize, Clone)]
pub struct Provider {
    name: String,
    coins: HashMap<String, String>,
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

    fn get_coins(&self) -> &HashMap<String, String> {
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

    let plist: Providers = match toml::from_str(&content) {
        Ok(r) => r,
        Err(err) => return Err(err.into()),
    };
    Ok(plist.providers)
}

pub fn update_provider(ref_file: &str, provider_name: &str) -> Option<Provider> {
    info!("Update provider {} requested", provider_name);
    match list_from_toml(ref_file.to_string()) {
        Ok(p) => match p.get(provider_name) {
            Some(mp) => return Some(mp.to_owned()),
            _ => None,
        },
        Err(err) => {
            error!("update_provider: {:?}", err);
            None
        },
    }
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
        let trial = match super::list_from_toml("./test/providers-test-1.toml".to_string()) {
            Ok(plist) => plist,
            Err(err) => panic!("{}", err) ,
        };

        use super::Provide;

        assert_eq!(trial.get("test1").unwrap().routes.get("ping").unwrap(), "/ping");
        assert_eq!(trial.get("test1").unwrap().get_uri("ping").unwrap(), "https://api.coingecko.com/api/v3/ping");
    }

    #[test]
    fn i_should_update_provider_multiple_times() {
        let mut trial = super::update_provider("./test/providers-test-1.toml", "test1").unwrap();
        assert_eq!(trial.name , "test1");
        assert_eq!(trial.routes.get("ping").unwrap(), "/ping");

        trial = super::update_provider("./test/providers-test-2.toml", "test2").unwrap();
        assert_eq!(trial.name , "test2");
        assert_eq!(trial.routes.get("pong").unwrap(), "/pong");
    }
}