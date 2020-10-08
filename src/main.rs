use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::{self};
use std::path::Path;
use std::process::Command;

mod ignore;
mod orchestrator;

// "ls -al" => Command::new("ls").arg("-al");
fn cmd_from_string(s: String) -> Result<std::process::Command, &'static str> {
    let mut iter = s.split_ascii_whitespace();
    let cmd;
    let first;

    first = iter.nth(0);
    match first {
        Some(c) => {
            cmd = c;
        }
        None => {
            // Not a great err msg...
            return Err("Expected there to be at least one thing in the command");
        }
    }

    let mut command = Command::new(cmd);
    command.args(iter);

    return Ok(command);
}

// TODO(dmiller): this should be a vector of arguments?
struct CmdRunner {
    cmd: std::process::Command,
}

impl orchestrator::Runner for CmdRunner {
    fn run(&mut self) -> io::Result<std::process::Output> {
        return self.cmd.output();
    }
}

fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    // TODO(dmiller): uhh this doesn't actually watch recursively??
    // TODO we need to put some debouncing on this or something
    let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| tx.send(res).unwrap())?;

    watcher.watch(path, RecursiveMode::Recursive)?;

    let builder = &mut CmdRunner {
        cmd: cmd_from_string(String::from("cargo build")).unwrap(),
    };
    let committer = &mut CmdRunner {
        cmd: cmd_from_string(String::from("git commit -am 'working'")).unwrap(),
    };
    let tester = &mut CmdRunner {
        cmd: cmd_from_string(String::from("cargo test")).unwrap(),
    };
    let reverter = &mut CmdRunner {
        cmd: cmd_from_string(String::from("git reset HEAD --hard")).unwrap(),
    };

    let root = std::env::current_dir().unwrap();
    let checker = ignore::Checker::new(root, None);

    let mut orc = orchestrator::Orchestrator::new(checker, builder, tester, committer, reverter);

    for res in rx {
        match res {
            Ok(event) => {
                println!("changed: {:?}", event);
                let paths = event.paths;
                let result = orc.handle_event(orchestrator::FileChangeEvent { paths: paths });
                match result {
                    Ok(_) => {}
                    Err(err) => {
                        println!("Error: {:?}", err);
                    }
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

// TODO
// find where the config file is, and run from there
fn get_path() -> io::Result<std::path::PathBuf> {
    match std::env::args().nth(1) {
        Some(p) => {
            let path = std::path::PathBuf::from(p);
            return Ok(path);
        }
        None => match std::env::current_dir() {
            Ok(p) => {
                return Ok(p);
            }
            Err(e) => {
                return Err(e);
            }
        },
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    build_cmd: String,
    test_cmd: String,
    revert_cmd: String,
    commit_cmd: String,
}

fn read_config(path: std::path::PathBuf) -> io::Result<Config> {
    let contents = std::fs::read_to_string(path);
    match contents {
        Ok(file_contents) => {
            let c: Config = serde_json::from_str(file_contents.as_ref()).unwrap();
            return Ok(c);
        },
        Err(e) => return Err(e),
    }
}

// TODO(dmiller): this should take a configuration. CLI, convention, toml/yaml/json file?
fn main() {
    let path = get_path().expect("Unable to get path");

    // TODO make this CLI configurable
    let config = read_config(path.join(".tcr"));
    match config {
        Ok(_) => {
            println!("We read the config!");
        },
        Err(e) => {
            println!("Error reading config at path {:?}: {:?}.\n TODO help user make config", path, e);
            std::process::exit(1);
        }
    }

    println!(
        "watching {}",
        path.to_str().expect("unable to convert path to string")
    );
    if let Err(e) = watch(path) {
        println!("error: {:?}", e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_cmd_from_string() {
        let output = cmd_from_string(String::from("ls -al")).unwrap();
        // there might be a better way to test this
        assert_eq!(format!("{:?}", output), "\"ls\" \"-al\"");

        cmd_from_string(String::from("")).expect_err("Expected this to fail");
    }
}
