use clap;

use canvas;
use config;

pub fn subcommand(matches: &clap::ArgMatches) -> Result<(), String> {
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
