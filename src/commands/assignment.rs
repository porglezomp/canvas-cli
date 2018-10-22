use clap;

use super::super::canvas;
use config;


pub fn subcommand(matches: &clap::ArgMatches) -> Result<(), String> {
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
