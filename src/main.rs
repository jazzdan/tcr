use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::io::{self};
use std::path::Path;
use std::process::Command;

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

    let mut orc = orchestrator::Orchestrator::new(builder, committer, tester, reverter);

    for res in rx {
        match res {
            Ok(event) => {
                println!("changed: {:?}", event);
                let result = orc.handle_event();
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

// TODO(dmiller): this should take a configuration. CLI, convention, toml file?
fn main() {
    let path = get_path().expect("Unable to get path");

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
