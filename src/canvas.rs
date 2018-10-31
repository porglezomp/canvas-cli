use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::de::DeserializeOwned;

use config::Config;

type Result<T> = ::std::result::Result<T, String>;

// @Todo: Use the Link header to get the rel=next links to handle pagination


/// https://canvas.instructure.com/doc/api/courses.html
#[derive(Deserialize, Debug)]
pub struct Course {
    pub id: u64,
    pub uuid: String,
    pub name: String,
    pub course_code: String,
    pub workflow_state: WorkflowState,
    pub enrollment_term_id: u64,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
    pub is_public: Option<bool>,
    pub public_description: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowState {
    Unpublished,
    Available,
    Completed,
    Deleted,
}

/// https://canvas.instructure.com/doc/api/files.html
#[derive(Deserialize, Debug)]
pub struct Folder {
    pub id: u64,
    pub folders_url: String,
    pub files_url: String,
    pub name: String,
    pub full_name: String,
}

/// https://canvas.instructure.com/doc/api/files.html
#[derive(Deserialize, Debug)]
pub struct File {
    pub id: u64,
    pub display_name: String,
    pub url: String,
}

/// https://canvas.instructure.com/doc/api/assignments.html
#[derive(Deserialize, Debug)]
pub struct Assignment {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
}



// @Todo: Download folders https://canvas.instructure.com/doc/api/content_exports.html

pub fn find_course_id(config: &Config, client: &Client, course_id: &str) -> Result<u64> {
    // If we have an integer ID, then we return that instead of making a network request
    if let Ok(id) = course_id.parse() {
        return Ok(id);
    }

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

    // @Improvement: match on some of the other identifiers?

    match view_course {
        Some(course) => Ok(course.id),
        None => Err(format!("No course starts with \"{}\"", course_id)),
    }
}

pub fn get_url_json<T: DeserializeOwned>(config: &Config, client: &Client, url: &str) -> Result<T> {
    let mut response = client
        .get(url)
        .bearer_auth(config.key())
        .send()
        .map_err(|err| format!("Failed to request ({})", err))?;
    if response.status().is_success() {
        response.json().map_err(
            |err| format!("Failed to load ({})", err),
        )
    } else {
        Err(format!(
            "Failed to fetch {}: HTTP status {}",
            response.url(),
            response.status()
        ))
    }
}

pub fn get_course_list(config: &Config, client: &Client) -> Result<Vec<Course>> {
    let url = format!("{}/courses?per_page=32", config.api_url());
    get_url_json(config, client, &url)
}

pub fn get_course_folder(
    config: &Config,
    client: &Client,
    course_id: u64,
    path: &str,
) -> Result<Folder> {
    let url = format!(
        "{}/courses/{}/folders/by_path/{}",
        config.api_url(),
        course_id,
        path.trim_left_matches('/')
    );
    let mut folders: Vec<_> = get_url_json(config, client, &url)?;
    folders.pop().ok_or_else(
        || format!("No files at path {}", path),
    )
}

pub fn get_course_root_folder(config: &Config, client: &Client, course_id: u64) -> Result<Folder> {
    let url = format!("{}/courses/{}/folders/root/", config.api_url(), course_id);
    get_url_json(config, client, &url)
}

pub fn get_files_and_folders(
    config: &Config,
    client: &Client,
    folder: &Folder,
) -> Result<(Vec<File>, Vec<Folder>)> {
    let files = get_url_json(config, client, &folder.files_url)?;
    let folders = get_url_json(config, client, &folder.folders_url)?;
    Ok((files, folders))
}

pub fn get_assignments(
    config: &Config,
    client: &Client,
    course_id: u64,
) -> Result<Vec<Assignment>> {
    let url = format!("{}/courses/{}/assignments/", config.api_url(), course_id);
    let assignments = get_url_json(config, client, &url)?;
    Ok(assignments)
}
