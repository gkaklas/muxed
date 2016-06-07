//! The YAML parser. Here is where we convert the yaml in to commands to be
/// processed later.

use yaml_rust::Yaml;
use command::{Command, Session, SendKeys, Split, Layout, Window, Attach};

#[cfg(test)] use yaml_rust::{YamlLoader};

/// Here was pass in the parsed yaml and project name. The purpose of this main
/// loop is to build the stack of commands that are run to setup a users tmux
/// session.
///
/// yaml_string: The parsed yaml from the config file.
/// project_name: The name of the project.
pub fn main(yaml_string: &Vec<Yaml>, project_name: &String, daemonize: bool) -> Result<Vec<Command>, String> {
    let mut commands: Vec<Command> = vec!();

    // There should only be one doc but it's a vec so loop it.
    for doc in yaml_string {
        let root = match doc["root"].as_str() {
            Some(x) => Some(x.to_string()),
            None    => None
        };

        let pre = match doc["pre"].as_str() {
            Some(x) => Some(x.to_string()),
            None    => None
        };

        // A clojure used to capture the current local root and pre Options.
        // This way we can call the clojure to create common SendKeys command
        // like changing the directory or executing a system command from the
        // `pre` option.
        let common_commands = |target: String| -> Vec<Command> {
            let mut commands2 = vec!();

            // SendKeys to change to the `root` directory
            if root.is_some() {
                let r = root.clone().unwrap();
                commands2.push(Command::SendKeys(SendKeys{
                    target: target.clone(),
                    exec: format!("cd {}", r)
                }));
            };

            // SendKeys for the Pre option
            if pre.is_some() {
                let p = pre.clone().unwrap();
                commands2.push(Command::SendKeys(SendKeys{
                    target: target.clone(),
                    exec: p
                }));
            };

            commands2
        };

        let windows = doc["windows"].as_vec().expect("No Windows have been defined.");

        for window in windows.iter() {
            match window {
                &Yaml::Hash(ref h)  => {
                    for (k, v) in h {
                        if v.as_hash().is_some() {
                            commands.push(Command::Window(Window{
                                session_name: project_name.clone(),
                                name: k.as_str().unwrap().to_string()
                            }));

                            if root.is_some() {
                              let r = root.clone().unwrap();
                              commands.push(Command::SendKeys(SendKeys{
                                  target: format!("{}:{}", project_name, k.as_str().unwrap().to_string()).to_string(),
                                  exec: format!("cd {}", r)
                              }));
                            };

                            commands.append(&mut try!(pane_matcher(&project_name, v, k.as_str().unwrap().to_string(), &pre, &root)));
                        } else {
                            commands.push(Command::Window(Window{
                                session_name: project_name.clone(),
                                name: try!(k.as_str().ok_or_else(|| "Windows require being named in your config.").map(|x| x.to_string()))
                            }));

                            let t = format!("{}:{}", project_name, k.as_str().unwrap().to_string()).to_string();
                            commands.append(&mut common_commands(t.to_string()));

                            // SendKeys for the exec command
                            if v.as_str().is_some() {
                                let ex = v.as_str().unwrap();
                                if !ex.is_empty() {
                                    commands.push(Command::SendKeys(SendKeys{
                                        target: format!("{}:{}", project_name, k.as_str().unwrap().to_string()).to_string(),
                                        exec: v.as_str().unwrap().to_string()
                                    }));
                                };
                            }
                        }
                    }
                },
                &Yaml::String(ref s) => {
                    commands.push(Command::Window(Window{
                        session_name: project_name.clone(),
                        name: s.clone()
                    }));

                    let t = format!("{}:{}", &project_name, &s);
                    commands.append(&mut common_commands(t.to_string()));
                },
                &Yaml::Integer(ref s) => {
                    commands.push(Command::Window(Window{
                        session_name: project_name.clone(),
                        name: s.to_string()
                    }));

                    let t = format!("{}:{}", &project_name, &s);
                    commands.append(&mut common_commands(t.to_string()));
                },
                _ => panic!("Muxed config file formatting isn't recognized.")
            };
        };
    };

    if !daemonize { commands.push(Command::Attach(Attach{name: project_name.clone()})) };

    let (first, commands) = commands.split_first().unwrap();
    let mut remains = commands.to_vec();

    match first {
        &Command::Window(ref w) => {
            remains.insert(0, Command::Session(Session{
                name: project_name.clone(),
                window_name: w.name.clone()
            }));
        },
        _ => {}
    };

    Ok(remains)
}

/// Pane matcher is for breaking apart the panes. Splitting windows when needed
/// and executing commands as needed.
fn pane_matcher(session: &String, window: &Yaml, window_name: String, pre: &Option<String>, root: &Option<String>) -> Result<Vec<Command>, String> {
    let mut commands = vec!();
    let panes = window["panes"].as_vec().expect("Something is wrong with panes.");

    for (i, pane) in panes.iter().enumerate() {
        // For every pane, we need one less split.
        // ex. An existing window to become 2 panes, needs 1 split.
        if i < (panes.len()-1) {
            commands.push(Command::Split(Split{
                target: format!("{}:{}.{}", session, window_name, i).to_string()
            }));
        };

        // SendKeys for the Pre option
        if pre.is_some() {
            let p = pre.clone().unwrap();
            commands.push(Command::SendKeys(SendKeys{
                target: format!("{}:{}.{}", session, window_name, i).to_string(),
                exec: p
            }));
        };

        // Execute given commands in each new pane after all splits are
        // complete.
        if pane.as_str().is_some() {
            let p = pane.as_str().unwrap();
            if !p.is_empty() {
                commands.push(Command::SendKeys(SendKeys{
                    target: format!("{}:{}.{}", session, window_name, i).to_string(),
                    exec: p.to_string()
                }));
            };
        };
    };

    // After all panes are split select the layout for the window
    if window["layout"].as_str().is_some() {
        let err = format!("A problem with the specified layout for the window: {}", &window_name);
        let layout = window["layout"].as_str().expect(err.as_str()).to_string();
        commands.push(Command::Layout(Layout{
            target: format!("{}:{}", session, window_name).to_string(),
            layout: layout
        }));
    };

    Ok(commands)
}

#[test]
pub fn expect_1_session() {
    let s = "---
windows: ['cargo', 'vim', 'git']
";
    let yaml = YamlLoader::load_from_str(s).unwrap();
    let remains: Vec<Command> = main(&yaml, &"muxed".to_string(), false).unwrap().into_iter().filter(|x| match x {
        &Command::Session(_) => true,
        _ => false
    }).collect();

    assert_eq!(remains.len(), 1)
}

#[test]
pub fn expect_2_windows_from_array() {
    let s = "---
windows: ['cargo', 'vim', 'git']
";
    let yaml = YamlLoader::load_from_str(s).unwrap();
    let remains: Vec<Command> = main(&yaml, &"muxed".to_string(), false).unwrap().into_iter().filter(|x| match x {
        &Command::Window(_) => true,
        _ => false
    }).collect();

    assert_eq!(remains.len(), 2)
}

#[test]
pub fn expect_1_attach() {
    let s = "---
windows: ['cargo', 'vim', 'git']
";
    let yaml = YamlLoader::load_from_str(s).unwrap();
    let remains: Vec<Command> = main(&yaml, &"muxed".to_string(), false).unwrap().into_iter().filter(|x| match x {
        &Command::Attach(_) => true,
        _ => false
    }).collect();

    assert_eq!(remains.len(), 1)
}

#[test]
pub fn expect_2_windows_with_mixed_type_names() {
    let s = "---
windows: [1, 'vim', 3]
";
    let yaml = YamlLoader::load_from_str(s).unwrap();
    let remains: Vec<Command> = main(&yaml, &"muxed".to_string(), false).unwrap().into_iter().filter(|x| match x {
        &Command::Window(_) => true,
        _ => false
    }).collect();
    assert_eq!(remains.len(), 2)
}

#[test]
pub fn expect_2_windows_from_list() {
    let s = "---
windows:
  - cargo: ''
  - vim: ''
  - git: ''
";
    let yaml = YamlLoader::load_from_str(s).unwrap();
    let remains: Vec<Command> = main(&yaml, &"muxed".to_string(), false).unwrap().into_iter().filter(|x| match x {
        &Command::Window(_) => true,
        _ => false
    }).collect();
    assert_eq!(remains.len(), 2)
}

#[test]
pub fn expect_ok_with_empty_syscommands() {
    let s = "---
windows:
  - editor:
";
    let yaml = YamlLoader::load_from_str(s).unwrap();
    let result = main(&yaml, &"muxed".to_string(), false);
    assert!(result.is_ok())
}

#[test]
pub fn expect_no_send_keys_commands() {
    let s = "---
windows:
  - editor:
";

    let yaml = YamlLoader::load_from_str(s).unwrap();
    let remains: Vec<Command> = main(&yaml, &"muxed".to_string(), false).unwrap().into_iter().filter(|x| match x {
        &Command::SendKeys(_) => true,
        _ => false
    }).collect();

    assert_eq!(remains.len(), 0)
}

#[test]
pub fn expect_err_with_nameless_window() {
    let s = "---
windows:
  - : ls
";
    let yaml = YamlLoader::load_from_str(s).unwrap();
    let result = main(&yaml, &"muxed".to_string(), false);
    assert!(result.is_err())
}

#[test]
pub fn expect_ok_with_empty_panes_syscommands() {
    let s = "---
windows:
  - cargo:
      layout: 'main-vertical'
      panes:
        -
";
    let yaml = YamlLoader::load_from_str(s).unwrap();
    let result = main(&yaml, &"muxed".to_string(), false);
    assert!(result.is_ok())
}

#[test]
pub fn expect_no_send_keys_with_empty_panes_syscommands() {
    let s = "---
windows:
  - editor:
      panes:
        -
";

    let yaml = YamlLoader::load_from_str(s).unwrap();
    let remains: Vec<Command> = main(&yaml, &"muxed".to_string(), false).unwrap().into_iter().filter(|x| match x {
        &Command::SendKeys(_) => true,
        _ => false
    }).collect();

    assert_eq!(remains.len(), 0)
}

#[test]
pub fn expect_1_split_window() {
    let s = "---
windows:
  - editor:
      layout: 'main-vertical'
      panes: ['vim', 'guard']
";

    let yaml = YamlLoader::load_from_str(s).unwrap();
    let remains: Vec<Command> = main(&yaml, &"muxed".to_string(), false).unwrap().into_iter().filter(|x| match x {
        &Command::Split(_) => true,
        _ => false
    }).collect();

    assert_eq!(remains.len(), 1)
}

#[test]
pub fn expect_1_layout() {
    let s = "---
windows:
  - editor:
      layout: 'main-vertical'
      panes: ['vim', 'guard']
";

    let yaml = YamlLoader::load_from_str(s).unwrap();
    let remains: Vec<Command> = main(&yaml, &"muxed".to_string(), false).unwrap().into_iter().filter(|x| match x {
        &Command::Layout(_) => true,
        _ => false
    }).collect();

    assert_eq!(remains.len(), 1)
}

#[test]
pub fn expect_1_session_with_panes_array() {
    let s = "---
windows:
  - editor:
      layout: 'main-vertical'
      panes: ['vim', 'guard']
";

    let yaml = YamlLoader::load_from_str(s).unwrap();
    let remains: Vec<Command> = main(&yaml, &"muxed".to_string(), false).unwrap().into_iter().filter(|x| match x {
        &Command::Session(_) => true,
        _ => false
    }).collect();

    assert_eq!(remains.len(), 1)
}

#[test]
pub fn expect_no_layout() {
    let s = "---
windows:
  - editor:
      panes: ['vim', 'guard']
";

    let yaml = YamlLoader::load_from_str(s).unwrap();
    let remains: Vec<Command> = main(&yaml, &"muxed".to_string(), false).unwrap().into_iter().filter(|x| match x {
        &Command::Layout(_) => true,
        _ => false
    }).collect();

    assert_eq!(remains.len(), 0)
}

#[test]
pub fn expect_three_send_keys_commands_from_pre() {
    let s = "---
pre: 'ls'
windows:
  - editor:
      panes:
        -
        -
  - logs:
";
    let yaml = YamlLoader::load_from_str(s).unwrap();
    let remains: Vec<Command> = main(&yaml, &"muxed".to_string(), false).unwrap().into_iter().filter(|x| match x {
        &Command::SendKeys(_) => true,
        _ => false
    }).collect();

    assert_eq!(remains.len(), 3)
}

#[test]
pub fn expect_no_send_keys_with_blank_panes() {
    let s = "---
windows:
  - editor:
      panes: ['','','']
";

    let yaml = YamlLoader::load_from_str(s).unwrap();
    let remains: Vec<Command> = main(&yaml, &"muxed".to_string(), false).unwrap().into_iter().filter(|x| match x {
        &Command::SendKeys(_) => true,
        _ => false
    }).collect();

    assert_eq!(remains.len(), 0)
}

#[test]
pub fn expect_no_send_keys_with_blank_window() {
    let s = "---
windows:
  - editor: ''
";

    let yaml = YamlLoader::load_from_str(s).unwrap();
    let remains: Vec<Command> = main(&yaml, &"muxed".to_string(), false).unwrap().into_iter().filter(|x| match x {
        &Command::SendKeys(_) => true,
        _ => false
    }).collect();

    assert_eq!(remains.len(), 0)
}
