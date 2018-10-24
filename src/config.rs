use hyper::header::{Authorization, Bearer};
use std;
use std::io::Read;

use toml;
use reqwest;

#[derive(Deserialize, Debug)]
pub struct Config {
    api: Api,
}

#[derive(Deserialize, Debug)]
pub struct Api {
    key: String,
    url: String,
}

impl Config {
    pub fn api_url(&self) -> String {
        format!("https://{}/api/v1", self.api.url)
    }

    pub fn auth(&self) -> Authorization<Bearer> {
        Authorization(Bearer { token: self.api.key.clone() })
    }
}

pub fn config_path() -> Result<std::path::PathBuf, String> {
    let path = std::env::home_dir()
        .ok_or_else(|| String::from("Missing home directory"))?
        .join(".config/canvas-cli/config.toml");
    Ok(path)
}

pub fn config_file() -> Result<std::fs::File, String> {
    let path = config_path()?;
    std::fs::File::open(path)
        .map_err(|err| format!("Config file doesn't appear to exist, try running {} config",
                                std::env::current_exe().unwrap().to_string_lossy()))
}

pub fn get_config() -> Result<Config, String> {
    let mut file = config_file()?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).map_err(|err| {
        format!("Cannot read config file ({})", err)
    })?;
    toml::from_slice(&data).map_err(|err| format!("Cannot parse config ({})", err))
}

pub fn get_client() -> Result<reqwest::Client, String> {
    reqwest::Client::new().map_err(|err| format!("Failed to make HTTP client ({})", err))
}
