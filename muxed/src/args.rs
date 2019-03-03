//! The struct managing cli args

#[derive(Debug, RustcDecodable)]
pub struct Args {
    pub flag_d: bool,
    pub flag_v: bool,
    pub flag_p: Option<String>,
    pub arg_project: String,
    pub cmd_new: bool,
}
