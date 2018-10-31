use std;
use std::io::Read;

use toml;
use xdg;

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

    pub fn key(&self) -> &str {
        &self.api.key
    }
}

pub fn config_path() -> Result<std::path::PathBuf, String> {
    xdg::BaseDirectories::with_prefix("canvas-cli")
        .map_err(|_err| "Unable to find config directory".to_string())?
        .place_config_file("config.toml")
        .map_err(|_err| "Unable to create config directory".to_string())
}

pub fn config_file() -> Result<std::fs::File, String> {
    let path = config_path()?;
    std::fs::File::open(path)
        .map_err(|_err| format!("Config file doesn't appear to exist, try running {} config",
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
