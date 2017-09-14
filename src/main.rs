#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;
extern crate reqwest;
extern crate hyper;
extern crate chrono;
#[macro_use]
extern crate clap;

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
                if let Some(description) = assignment.description {
                    // @Todo: Handle HTML
                    if description.len() > 72 {
                        println!("  {}...", description.chars().take(72).collect::<String>());
                    } else {
                        println!("  {}", description);
                    }
                }
            }
        }
        ("info", Some(_info_matches)) => unimplemented!(),
        ("submit", Some(_submit_matches)) => unimplemented!(),
        _ => unreachable!(),
    }
    Ok(())
}

fn config_subcommand(_matches: &clap::ArgMatches) -> Result<(), String> {
    unimplemented!()
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
