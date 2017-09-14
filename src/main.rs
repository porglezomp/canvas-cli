#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;
extern crate reqwest;
extern crate hyper;
extern crate chrono;
#[macro_use]
extern crate clap;

use std::io::{Read, Write};

mod config;
mod canvas;


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
    let matches = app().get_matches();

    match matches.subcommand() {
        ("course", Some(course_matches)) => course_subcommand(course_matches),
        ("file", Some(file_matches)) => file_subcommand(file_matches),
        ("assignment", Some(assignment_matches)) => assignment_subcommand(assignment_matches),
        ("config", Some(config_matches)) => config_subcommand(config_matches),
        _ => unreachable!(),
    }
}

fn course_subcommand(matches: &clap::ArgMatches) -> Result<(), String> {
    let config = config::get_config()?;
    let client = config::get_client()?;
    match matches.subcommand() {
        ("ls", Some(_ls_matches)) => {
            let courses = canvas::get_course_list(&config, &client)?;
            for course in courses {
                println!("{:<10} {}", format!("({})", course.id), course.name);
            }
        }
        ("info", Some(_info_matches)) => unimplemented!(),
        _ => unreachable!(),
    }
    Ok(())
}

fn file_subcommand(matches: &clap::ArgMatches) -> Result<(), String> {
    let config = config::get_config()?;
    let client = config::get_client()?;
    match matches.subcommand() {
        ("ls", Some(ls_matches)) => {
            let course_id =
                canvas::find_course_id(&config, &client, &ls_matches.value_of("course").unwrap())?;

            let dir = match ls_matches.value_of("path") {
                Some("/") | None => canvas::get_course_root_folder(&config, &client, course_id)?,
                Some(path) => canvas::get_course_folder(&config, &client, course_id, &path)?,
            };

            let (files, folders) = canvas::get_files_and_folders(&config, &client, &dir)?;
            for folder in folders {
                println!("{}/", folder.name);
            }
            for file in files {
                println!("{}", file.display_name);
            }
        }
        ("info", Some(_info_matches)) => unimplemented!(),
        ("download", Some(_download_matches)) => unimplemented!(),
        _ => unreachable!(),
    }
    Ok(())
}

fn assignment_subcommand(matches: &clap::ArgMatches) -> Result<(), String> {
    let config = config::get_config()?;
    let client = config::get_client()?;
    match matches.subcommand() {
        ("ls", Some(ls_matches)) => {
            let course_id =
                canvas::find_course_id(&config, &client, &ls_matches.value_of("course").unwrap())?;

            let assignments = canvas::get_assignments(&config, &client, course_id)?;
            for assignment in assignments {
                if let Some(due) = assignment.due_at {
                    println!("{} (due {:?})", assignment.name, due);
                } else {
                    println!("{}", assignment.name);
                }
                // @Todo: Print the description
            }
        }
        ("info", Some(_info_matches)) => unimplemented!(),
        ("submit", Some(_submit_matches)) => unimplemented!(),
        _ => unreachable!(),
    }
    Ok(())
}

fn config_subcommand(_matches: &clap::ArgMatches) -> Result<(), String> {
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

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)
        .map_err(|err| format!("Error opening file ({})", err))?;
    file.write(toml::to_string_pretty(&toml_doc).unwrap().as_bytes())
        .map_err(|err| format!("Error writing new file ({})", err))?;

    Ok(())
}

fn app<'a, 'b>() -> clap::App<'a, 'b> {
    clap_app!(canvas =>
        (version: env!("CARGO_PKG_VERSION"))
        (author: "C Jones <code@calebjones.net>")
        (about: "An app for interacting with Canvas")
        (setting: clap::AppSettings::ArgRequiredElseHelp)
        (@subcommand course =>
            (about: "List courses and view course information")
            (@subcommand ls =>
                (about: "List courses")
            )
            (@subcommand info =>
                (about: "Display information about a course")
                (@arg course: +required "A course title or numeric ID")
            )
        )
        (@subcommand file =>
            (about: "List, inspect, or download files")
            (@subcommand ls =>
                (about: "List files")
                (@arg course: +required "A course title or numeric ID")
                (@arg path: "The directory to examine. Defaults to /")
            )
            (@subcommand info =>
                (about: "Display information about a file")
                (@arg course: +required "A course title or numeric ID")
                (@arg path: +required "The file or directory to examine")
            )
            (@subcommand download =>
                (about: "Download a file")
                (@arg course: +required "A course title or numeric ID")
                (@arg path: +required "The file or directory to download")
            )
        )
        (@subcommand assignment =>
            (about: "List, inspect, or submit assignments")
            (@subcommand ls =>
                (about: "List assignments")
                (@arg course: +required "A course title or numeric ID")
            )
            (@subcommand info =>
                (about: "Display information about an assignment")
                (@arg course: +required "A course title or numeric ID")
                (@arg id: +required "An assignment ID")
            )
            (@subcommand submit =>
                (about: "Submit files for an assignment")
                (@arg course: +required "A course title or numeric ID")
                (@arg id: +required "An assignment ID")
                (@arg file: +required +multiple "The file to submit")
            )
        )
        (@subcommand config =>
            (about: "Edit the user config")
        )
    )
}
