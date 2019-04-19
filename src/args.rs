//! The struct managing cli args

/// The args struct for taking arguments passed in from the command line
/// and making it easier to pass around.
#[derive(Debug, RustcDecodable)]
pub struct Args {
    pub flag_d: bool,
    pub flag_v: bool,
    pub flag_f: bool,
    pub flag_p: Option<String>,
    pub flag_t: Option<String>,
    pub arg_project: String,
    pub cmd_new: bool,
    pub cmd_snapshot: bool,
}
