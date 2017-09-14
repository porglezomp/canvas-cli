#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;
extern crate reqwest;
extern crate hyper;
extern crate chrono;
extern crate clap;

use clap::{App, Arg, SubCommand};

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
