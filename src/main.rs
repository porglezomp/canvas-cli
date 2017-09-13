#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::io::Read;

#[derive(Deserialize, Debug)]
struct Config {
    api: Api,
}

#[derive(Deserialize, Debug)]
struct Api {
    key: String,
    url: String,
}

fn main() {
    std::process::exit(match run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    });
}

fn run() -> Result<(), String> {
    let config = get_config()?;
    println!("{}: {}", config.api.url, config.api.key);
    Ok(())
}

fn get_config() -> Result<Config, String> {
    let path = std::env::home_dir()
        .ok_or_else(|| String::from("Missing home directory"))?
        .join(".config/canvas-cli/config.toml");
    let mut data = Vec::new();
    let mut file = std::fs::File::open(path).map_err(|err| {
        format!("Cannot open config file ({})", err)
    })?;
    file.read_to_end(&mut data).map_err(|err| {
        format!("Cannot read config file ({})", err)
    })?;
    toml::from_slice(&data).map_err(|err| format!("Cannot parse config ({})", err))
}
