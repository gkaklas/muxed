use std::path::posix::Path;
use std::io::fs::PathExtensions;
use std::io::process::Command;

mod io;

static TEMPLATE: &'static str = include_str!("template.toml");

pub fn main(path: Path) {
    if !path.exists() {
        let filename = path.filename_str().unwrap();
        let template = modified_template(TEMPLATE, filename);

        try_or_err!(io::create(&path, template.as_slice()), "Failed to create project file");

        match is_default_editor_set() {
            true  => io::open(&path),
            false => println!("Default editor is not set. Your config has been created and can be found in ~/.muxed/. Please define $EDITOR in your ~/.bashrc or similar file.")
        }
    } else {
        println!("Project already exists.");
    }
}

/// Run `which $EDITOR` to see if a default editor is defined on the system.
fn is_default_editor_set() -> bool {
  let output = match Command::new("which").arg("$EDITOR").output() {
      Ok(output) => output.output.to_string(),
      Err(e)     => fail!("failed to execute process: {}", e),
  };

  !output.is_empty()
}

fn modified_template(template: &str, project_name: &str) -> String {
    template.replace("{file_name}", project_name)
}

#[test]
fn populates_template_placeholders() {
    let value  = modified_template(TEMPLATE, "muxed project");
    let result = value.as_slice().contains("muxed project");
    assert!(result);
}

#[test]
fn removes_template_placeholders() {
    let value  = modified_template(TEMPLATE, "muxed project");
    let result = !value.as_slice().contains("{file_name}");
    assert!(result);
}