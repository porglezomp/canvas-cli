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
    let matches = App::new("canvas")
        .version(env!("CARGO_PKG_VERSION"))
        .author("C Jones <code@calebjones.net>")
        .about("An app for interacting with Canvas")
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("course")
                .about("List courses and view course information")
                .subcommand(SubCommand::with_name("ls").about("List courses"))
                .subcommand(
                    SubCommand::with_name("info")
                        .about("Display information about a course")
                        .arg(
                            Arg::with_name("course")
                                .help("A course title or numeric ID")
                                .takes_value(true)
                                .required(true),
                        ),
                ),
        )
        .subcommand(
            SubCommand::with_name("file")
                .about("List, inspect, or download files")
                .subcommand(
                    SubCommand::with_name("ls")
                        .about("List files")
                        .arg(
                            Arg::with_name("course")
                                .help("A course title or numeric ID")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("path")
                                .help("The directory to examine. Defaults to /")
                                .takes_value(true)
                                .required(false),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("info")
                        .about("Display information about a file")
                        .arg(
                            Arg::with_name("course")
                                .help("A course title or numeric ID")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("path")
                                .help("The file or directory to examine")
                                .takes_value(true)
                                .required(true),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("download")
                        .about("Download a file")
                        .arg(
                            Arg::with_name("course")
                                .help("A course title or numeric ID")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("path")
                                .help("The file or directory to download")
                                .takes_value(true)
                                .required(true),
                        ),
                ),
        )
        .subcommand(
            SubCommand::with_name("assignment")
                .about("List, inspect, or submit assignments")
                .subcommand(
                    SubCommand::with_name("ls").about("List assignments").arg(
                        Arg::with_name("course")
                            .help("A course title or numeric ID")
                            .takes_value(true)
                            .required(true),
                    ),
                )
                .subcommand(
                    SubCommand::with_name("info")
                        .about("Display information about an assignment")
                        .arg(
                            Arg::with_name("course")
                                .help("A course title or numeric ID")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("id")
                                .help("An assignment ID")
                                .takes_value(true)
                                .required(true),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("submit")
                        .about("Submit files for an assignment")
                        .arg(
                            Arg::with_name("course")
                                .help("A course title or numeric ID")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("id")
                                .help("An assignment ID")
                                .takes_value(true)
                                .required(true),
                        )
                        .arg(
                            Arg::with_name("file")
                                .help("The file to submit")
                                .takes_value(true)
                                .multiple(true)
                                .required(true),
                        ),
                ),
        )
        .subcommand(SubCommand::with_name("config").about(
            "Edit the user config",
        ))
        .get_matches();

    match matches.subcommand() {
        ("course", Some(course_matches)) => course_subcommand(course_matches)?,
        ("file", Some(file_matches)) => file_subcommand(file_matches)?,
        ("assignment", Some(assignment_matches)) => assignment_subcommand(assignment_matches)?,
        ("config", Some(config_matches)) => config_subcommand(config_matches)?,
        _ => unreachable!(),
    }

    Ok(())
}

fn get_client() -> Result<reqwest::Client, String> {
    reqwest::Client::new().map_err(|err| format!("Failed to make HTTP client ({})", err))
}

fn course_subcommand(matches: &clap::ArgMatches) -> Result<(), String> {
    let config = get_config()?;
    let client = get_client()?;
    match matches.subcommand() {
        ("ls", Some(_ls_matches)) => {
            let courses = get_course_list(&config, &client)?;
            for course in courses {
                println!("{:<10} {}", format!("({})", course.id), course.name);
            }
        }
        ("info", Some(_info_matches)) => unimplemented!(),
        _ => unreachable!(),
    }
    Ok(())
}

fn find_course_id(
    config: &Config,
    client: &reqwest::Client,
    course_id: &str,
) -> Result<u64, String> {
    let mut view_course = None;
    let courses = get_course_list(config, client)?;
    for course in courses {
        if course.name.starts_with(course_id) {
            if view_course.is_none() {
                view_course = Some(course);
            } else {
                return Err(format!("Multiple courses start with \"{}\"", course_id));
            }
        }
    }
    match view_course {
        Some(course) => Ok(course.id),
        None => Err(format!("No course starts with \"{}\"", course_id)),
    }
}

fn file_subcommand(matches: &clap::ArgMatches) -> Result<(), String> {
    let config = get_config()?;
    let client = get_client()?;
    match matches.subcommand() {
        ("ls", Some(ls_matches)) => {
            let course_id =
                find_course_id(&config, &client, &ls_matches.value_of("course").unwrap())?;
            let dir = match ls_matches.value_of("path") {
                Some("/") | None => get_course_root_folder(&config, &client, course_id)?,
                Some(path) => get_course_folder(&config, &client, course_id, &path)?,
            };
            let (files, folders) = get_files_and_folders(&config, &client, &dir)?;
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
    let _config = get_config()?;
    let _client = get_client()?;
    match matches.subcommand() {
        ("ls", Some(_ls_matches)) => unimplemented!(),
        ("info", Some(_info_matches)) => unimplemented!(),
        ("submit", Some(_submit_matches)) => unimplemented!(),
        _ => unreachable!(),
    }
}

fn config_subcommand(_matches: &clap::ArgMatches) -> Result<(), String> {
    unimplemented!()
}


// @Todo: Download folders https://canvas.instructure.com/doc/api/content_exports.html

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
