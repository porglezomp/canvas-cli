use std;
use clap;
use toml;

use config;

pub fn subcommand(_matches: &clap::ArgMatches) -> Result<(), String> {
    fn confirm(prompt: &str) -> bool {
        loop {
            println!("{}", prompt);
            let mut response = String::new();
            std::io::stdin().read_line(&mut response).unwrap();
            match response.to_lowercase().trim() {
                "y" | "yes" => return true,
                "n" | "no" => return false,
                _ => {}
            }
        }
    }

    fn get_value(prompt: &str) -> String {
        let mut buf = String::new();
        while buf.is_empty() {
            println!("{}", prompt);
            std::io::stdin().read_line(&mut buf).unwrap();
            buf = buf.trim().into();
        }
        buf
    }

    const EDIT_PROMPT: &str = "Do you want to edit it? This will remove all comments. (y/n)";
    const URL_PROMPT: &str = "Enter your Canvas domain name (for example, canvas.instructure.com or umich.instructure.com):";
    const KEY_PROMPT: &str = "Enter your access token. You can generate an access token in Canvas by going to Account > Settings, then scroll down to find \"+ New Access Token\". Paste it here.";

    let mut config_url = String::new();
    let mut config_key = String::new();

    let mut toml_doc = match config::config_file() {
        Ok(mut file) => {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).unwrap();
            match toml::from_slice::<toml::Value>(&buf) {
                Ok(toml_doc) => {
                    println!("A config file already exists");
                    if !confirm(EDIT_PROMPT) {
                        return Ok(());
                    }

                    if let Some(&toml::Value::Table(ref api)) = toml_doc.get("api") {
                        match api.get("url") {
                            Some(&toml::Value::String(ref url)) => {
                                config_url = url.clone();
                                println!("api.url is set to \"{}\"", url);
                                if confirm("Do you want to overwrite this value? (y/n)") {
                                    config_url = get_value(URL_PROMPT);
                                }
                            }
                            Some(url) => {
                                println!("api.url has an invalid type, is is: {:?}", url);
                                if confirm("Do you want to overwrite this value? (y/n)") {
                                    config_url = get_value(URL_PROMPT);
                                }
                            }
                            None => {
                                println!("api.url is not set");
                                config_url = get_value(URL_PROMPT);
                            }
                        }

                        match api.get("key") {
                            Some(&toml::Value::String(ref key)) => {
                                config_key = key.clone();
                                println!("api.key is set to \"{}\"", key);
                                if confirm("Do you want to overwrite this value? (y/n)") {
                                    config_key = get_value(KEY_PROMPT);
                                }
                            }
                            Some(key) => {
                                println!("api.key has an invalid type, it is: {:?}", key);
                                if confirm("Do you want to overwrite this value? (y/n)") {
                                    config_key = get_value(KEY_PROMPT);
                                }
                            }
                            None => {
                                println!("api.key is not set");
                                config_key = get_value(KEY_PROMPT);
                            }
                        }
                    } else {
                        println!("Warning: No [api] section found");
                        config_url = get_value(URL_PROMPT);
                        config_key = get_value(KEY_PROMPT);
                    }
                    toml_doc
                }
                Err(err) => {
                    return Err(format!(
                        "A config file already exists, but can't be parsed. You should fix it by hand, or rename it so a new config file can be generated. (Parse error: {})",
                        err
                    ));
                }
            }
        }
        Err(_) => {
            config_url = get_value(URL_PROMPT);
            config_key = get_value(KEY_PROMPT);
            toml::Value::Table(toml::value::Table::new())
        }
    };

    if let toml::Value::Table(ref mut doc) = toml_doc {
        let api = doc.entry("api".into()).or_insert_with(|| {
            toml::Value::Table(toml::value::Table::new())
        });
        if let &mut toml::Value::Table(ref mut api) = api {
            api.insert("url".into(), toml::Value::String(config_url));
            api.insert("key".into(), toml::Value::String(config_key));
        }
    }

    let path = config::config_path()?;
    let mut backup_path = path.clone();
    let mut extension = backup_path.extension().unwrap().to_os_string();
    extension.push(".bk");
    backup_path.set_extension(extension);
    // We have to allow this to fail in case it doesn't already exist
    let _ = std::fs::rename(&path, &backup_path);

    std::fs::create_dir_all(&path.parent().unwrap())
        .map_err(|err| format!("Error creating config dir ({})", err))?;

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)
        .map_err(|err| format!("Error opening file ({})", err))?;
    file.write(toml::to_string_pretty(&toml_doc).unwrap().as_bytes())
        .map_err(|err| format!("Error writing new file ({})", err))?;

    Ok(())
}
