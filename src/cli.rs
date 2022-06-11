use clap::{arg, command, ArgMatches};
use std::ops::RangeInclusive;
use std::path::Path;

const PORT_RANGE: RangeInclusive<usize> = 1..=65535;

pub fn get_matches() -> ArgMatches {
    command!()
        .arg(arg!(output: <OUTPUT_DIR> "Sets the directory for uploaded files").validator(validate_dir))
        .arg(
            arg!(-p --port <NUMBER> "Sets the port number to listen on")
                .required(false)
                .default_value("54298")
                .validator(|value| {
                    value
                        .parse::<usize>()
                        .map(|port| PORT_RANGE.contains(&port))
                        .map_err(|err| err.to_string())
                        .and_then(|result| if result { Ok(()) } else { Err(format!(
                            "Port needs to be a number between {} and {}",
                            PORT_RANGE.start(),
                            PORT_RANGE.end()
                        )) })
                }),
        )
        .arg(arg!(-u --unix <FILE> "Sets the unix socket to listen on (Unix only)").required(false))
        .arg(
            arg!(-d --database <DATABASE_DIR> "Sets the directory for the SQLite database")
                .required(false)
                .default_value("./")
                .validator(validate_dir)
        )
        .arg(arg!(--password <TEXT> "Sets a password for API requests").required(false).env("DEKINAI_PASSWORD").validator(|value| {
            if value.is_ascii() {
                Ok(())
            } else {
                Err("Password needs to contain only ASCII characters")
            }
        }))
        .arg(arg!(-b --blacklist <FILE_EXTENSIONS> "Sets a list of disallowed file extensions\nUsage: --blacklist asp html php").required(false).multiple_values(true))
        .arg(arg!(--"disable-port" "Disables port listening (Unix only)").required(false).requires("unix").conflicts_with("port"))
        .get_matches()
}

fn validate_dir(path: &str) -> Result<(), String> {
    if Path::new(path).is_dir() {
        Ok(())
    } else {
        Err(format!("Cannot access directory \"{}\"", path))
    }
}
