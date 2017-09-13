#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate reqwest;
extern crate hyper;
extern crate chrono;

use chrono::{DateTime, Utc};
use hyper::header::{Authorization, Bearer};
use std::io::Read;


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
    let client = reqwest::Client::new().map_err(|err| {
        format!("Failed to make HTTP client ({})", err)
    })?;
    let courses = get_course_list(&config, &client)?;
    for course in courses {
        println!("{}", course.name);
        println!("  Id:\t{}", course.id);
        println!("  Code:\t{}", course.course_code);
        if let Some(desc) = course.public_description {
            println!("  Desc:\t{}", desc);
        }
    }
    Ok(())
}

fn get_course_list(config: &Config, client: &reqwest::Client) -> Result<Vec<Course>, String> {
    let mut response = client
        .get(&format!("https://{}/api/v1/courses", config.api.url))
        .map_err(|err| format!("Failed to make GET request ({})", err))?
        .header(Authorization(Bearer { token: config.api.key.clone() }))
        .send()
        .map_err(|err| format!("Failed to request ({})", err))?;
    if response.status().is_success() {
        response.json().map_err(|err| {
            format!("Failed to load course list ({})", err)
        })
    } else {
        Err(format!(
            "Failed to fetch {}: HTTP status {}",
            response.url(),
            response.status()
        ))
    }
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
