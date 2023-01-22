use clap::{arg, command, value_parser, ArgAction, ArgMatches};
use std::path::PathBuf;

pub fn get_matches() -> ArgMatches {
    command!()
        .arg(arg!(output: <OUTPUT_DIRECTORY> "Sets the directory for uploaded files").value_parser(validate_dir))
        .arg(arg!(-p --port [NUMBER] "Sets the port number to listen on").default_value("54298").value_parser(value_parser!(u16).range(1..)))
        .arg(arg!(-u --unix [FILE] "Sets the unix socket to listen on (Unix only)").value_parser(value_parser!(PathBuf)))
        .arg(arg!(-d --database [DIRECTORY] "Sets the directory for the SQLite database").default_value("./").value_parser(validate_dir))
        .arg(arg!(--password [TEXT] "Sets a password for API requests").env("DEKINAI_PASSWORD").value_parser(validate_password))
        .arg(arg!(-b --blacklist [FILE_EXTENSIONS] ... "Sets a list of disallowed file extensions\nUsage: --blacklist asp html php").num_args(1..))
        .arg(arg!(--"disable-port" "Disables port listening (Unix only)").requires("unix").conflicts_with("port").action(ArgAction::SetTrue))
        .get_matches()
}

fn validate_dir(value: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(value);

    if path.is_dir() {
        Ok(path)
    } else {
        Err(format!("Cannot access directory \"{value}\""))
    }
}

fn validate_password(value: &str) -> Result<String, String> {
    if value.is_ascii() {
        Ok(value.to_owned())
    } else {
        Err("Password needs to contain only ASCII characters".to_owned())
    }
}
