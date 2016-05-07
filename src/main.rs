//! Muxed. A tmux project manager with no runtime dependencies.
extern crate getopts;
extern crate libc;
extern crate yaml_rust;
extern crate rand;

use std::env;

mod tmux;
mod command;
mod project;

use project::parser;
use project::processor;

/// The main execution method.
/// Verify all the arguments and options passed are valid for the application.
fn main() {
    let args: Vec<String> = env::args().collect();
    //let program = args[0].clone();
    let input = args[1].clone();

    let yaml     = project::main(&input);
    let commands = parser::main(&yaml, &input);
    processor::main(&commands)
}
