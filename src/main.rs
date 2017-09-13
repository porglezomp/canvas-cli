#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;
extern crate reqwest;
extern crate hyper;
extern crate chrono;
extern crate clap;

use clap::{App, Arg, SubCommand};
use chrono::{DateTime, Utc};
use hyper::header::{Authorization, Bearer};
use std::io::Read;

// @Todo: Use the Link header to get the rel=next links to handle pagination


#[derive(Deserialize, Debug)]
struct Course {
    id: u64,
    uuid: String,
    name: String,
    course_code: String,
    workflow_state: WorkflowState,
    enrollment_term_id: u64,
    start_at: Option<DateTime<Utc>>,
    end_at: Option<DateTime<Utc>>,
    is_public: Option<bool>,
    public_description: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum WorkflowState {
    Unpublished,
    Available,
    Completed,
    Deleted,
}

#[derive(Deserialize, Debug)]
struct Folder {
    id: u64,
    folders_url: String,
    files_url: String,
    name: String,
    full_name: String,
}

#[derive(Deserialize, Debug)]
struct File {
    id: u64,
    display_name: String,
    url: String,
}

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
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author("C Jones <code@calebjones.net>")
        .about("An app for interacting with Canvas")
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("ls")
                .about("List files or courses")
                .arg(
                    Arg::with_name("course")
                        .help("The course to list files in. If this is omitted, then ls will return a list of courses")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("path")
                        .help("The path to examine. Defaults to /")
                        .takes_value(true)
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("download")
                .about("Download files")
                .arg(
                    Arg::with_name("course")
                        .help("The course to download files from")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("path")
                        .help("The path to download")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .get_matches();

    let config = get_config()?;
    let client = reqwest::Client::new().map_err(|err| {
        format!("Failed to make HTTP client ({})", err)
    })?;
    let courses = get_course_list(&config, &client)?;
    match matches.subcommand() {
        ("ls", Some(ls)) => {
            if let Some(course_name) = ls.value_of("course") {
                let mut view_course = None;
                for course in courses {
                    if course.name.starts_with(course_name) {
                        if view_course.is_none() {
                            view_course = Some(course);
                        } else {
                            return Err(format!("Multiple courses start with \"{}\"", course_name));
                        }
                    }
                }
                match view_course {
                    Some(course) => {
                        let dir = match ls.value_of("path") {
                            Some("/") | None => {
                                get_course_root_folder(&config, &client, course.id)?
                            }
                            Some(path) => get_course_folder(&config, &client, course.id, &path)?,
                        };
                        let (files, folders) = get_files_and_folders(&config, &client, &dir)?;
                        for folder in folders {
                            println!("{}/", folder.name);
                        }
                        for file in files {
                            println!("{}", file.display_name);
                        }
                    }
                    None => return Err(format!("No course starts with \"{}\"", course_name)),
                }
            } else {
                for course in courses {
                    println!("{}", course.name);
                }
            }
        }
        ("download", Some(dl)) => {
            // @Todo: Download folders https://canvas.instructure.com/doc/api/content_exports.html
            if let (Some(_course), Some(_path)) = (dl.value_of("course"), dl.value_of("path")) {
                unimplemented!();
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn get_url_json<T: serde::de::DeserializeOwned>(
    config: &Config,
    client: &reqwest::Client,
    url: &str,
) -> Result<T, String> {
    let mut response = client
        .get(url)
        .map_err(|err| format!("Failed to make GET request ({})", err))?
        .header(Authorization(Bearer { token: config.api.key.clone() }))
        .send()
        .map_err(|err| format!("Failed to request ({})", err))?;
    if response.status().is_success() {
        response.json().map_err(|err| {
            format!("Failed to load folder list ({})", err)
        })
    } else {
        Err(format!(
            "Failed to fetch {}: HTTP status {}",
            response.url(),
            response.status()
        ))
    }
}

fn get_course_list(config: &Config, client: &reqwest::Client) -> Result<Vec<Course>, String> {
    let url = format!("https://{}/api/v1/courses?per_page=32", config.api.url);
    get_url_json(config, client, &url)
}

fn get_course_folder(
    config: &Config,
    client: &reqwest::Client,
    course_id: u64,
    path: &str,
) -> Result<Folder, String> {
    let url = format!(
        "https://{}/api/v1/courses/{}/folders/by_path/{}",
        config.api.url,
        course_id,
        path.trim_left_matches('/')
    );
    let mut folders: Vec<_> = get_url_json(config, client, &url)?;
    folders.pop().ok_or_else(
        || format!("No files at path {}", path),
    )
}

fn get_course_root_folder(
    config: &Config,
    client: &reqwest::Client,
    course_id: u64,
) -> Result<Folder, String> {
    let url = format!(
        "https://{}/api/v1/courses/{}/folders/root/",
        config.api.url,
        course_id
    );
    get_url_json(config, client, &url)
}

fn get_files_and_folders(
    config: &Config,
    client: &reqwest::Client,
    folder: &Folder,
) -> Result<(Vec<File>, Vec<Folder>), String> {
    let files = get_url_json(config, client, &folder.files_url)?;
    let folders = get_url_json(config, client, &folder.folders_url)?;
    Ok((files, folders))
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
