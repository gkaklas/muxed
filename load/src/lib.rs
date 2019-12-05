extern crate dirs;
extern crate libc;
#[cfg(test)]
extern crate rand;
extern crate yaml_rust;

extern crate common;

pub mod command;
pub mod project;
pub mod tmux;

use args::Args;
use command::Commands;
use common::{args, first_run};
use project::parser;
use tmux::config::Config;

#[cfg(not(test))]
use dirs::home_dir;
use std::path::PathBuf;

static MUXED_FOLDER: &str = "muxed";

pub fn exec(args: Args) -> Result<(), String> {
    // FIXME: If -p flag isn't set there's no default?
    let home = homedir().expect("Can't find home dir");
    let default_dir = format!("{}/.{}", home.display(), MUXED_FOLDER);
    let muxed_dir = match args.flag_p {
        Some(ref x) => Some(x.as_str()),
        _ => Some(default_dir.as_str()),
    };

    let yaml = project::read(&args.arg_project, &muxed_dir).unwrap();
    let project_name = &yaml[0]["name"]
        .as_str()
        .unwrap_or(&args.arg_project)
        .to_string();

    let commands: Vec<Commands>;
    match project::session_exists(project_name) {
        Some(c) => {
            commands = vec![c];
        }
        None => {
            let config = Config::from_string(tmux::get_config());
            commands = parser::call(&yaml, project_name, args.flag_d, &config)
                .expect("Couldn't parse commands");
        }
    };

    if args.flag_debug {
        println!("{:?}", &commands);
    };

    for command in &commands {
        command
            .as_trait()
            .call(args.flag_debug)
            .map_err(|e| format!("Had a problem running commands for tmux {}", e))
            .unwrap();
    }

    Ok(())
}

/// Return the users homedir as a string.
#[cfg(not(test))]
fn homedir() -> Result<PathBuf, String> {
    match home_dir() {
        Some(dir) => Ok(dir),
        None => Err(String::from("We couldn't find your home directory.")),
    }
}

/// Return the temp dir as the users home dir during testing.
#[cfg(test)]
fn homedir() -> Result<PathBuf, String> {
    Ok(PathBuf::from("/tmp"))
}
