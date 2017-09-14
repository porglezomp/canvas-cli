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
mod commands;


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
        ("assignment", Some(matches)) => commands::assignment::subcommand(matches),
        ("config", Some(matches)) => commands::config::subcommand(matches),
        ("course", Some(matches)) => commands::course::subcommand(matches),
        ("file", Some(matches)) => commands::file::subcommand(matches),
        _ => unreachable!(),
    }
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
