use clap::Clap;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::{self};
use std::path::Path;
use std::process::Command;
use ::ignore::gitignore::Gitignore as Gitignore;

mod ignore;
mod orchestrator;

#[derive(Clap)]
#[clap(version = "0.1", author = "Dan Miller <dan@dmiller.dev>")]
struct Opts {
    #[clap(short, long)]
    config: Option<String>,
    #[clap(short, long)]
    root: Option<String>,
}

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
            return Err("Expected cmd to not be empty");
        }
    }

    let mut command = Command::new(cmd);
    command.args(iter);

    return Ok(command);
}

struct CmdRunner {
    cmd: std::process::Command,
}

impl orchestrator::Runner for CmdRunner {
    fn run(&mut self) -> io::Result<std::process::Output> {
        return self.cmd.output();
    }
}

fn watch_and_run<P: AsRef<Path>>(path: P, config: Config) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    // TODO(dmiller): uhh this doesn't actually watch recursively on WSL?
    // TODO we need to put some debouncing on this or something
    let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| tx.send(res).unwrap())?;

    watcher.watch(path, RecursiveMode::Recursive)?;

    let builder = &mut CmdRunner {
        cmd: cmd_from_string(config.build_cmd).unwrap(),
    };
    let committer = &mut CmdRunner {
        cmd: cmd_from_string(config.commit_cmd).unwrap(),
    };
    let tester = &mut CmdRunner {
        cmd: cmd_from_string(config.revert_cmd).unwrap(),
    };
    let reverter = &mut CmdRunner {
        cmd: cmd_from_string(config.test_cmd).unwrap(),
    };

    let root = std::env::current_dir().unwrap();
    let gitignore_path = root.join(".gitignore").to_path_buf();
    // TODO should I handle the error here? Weird syntax.
    let (gitignore, _) =  Gitignore::new(&gitignore_path);
    let checker = ignore::Checker::new(root, Some(gitignore));

    let mut orc = orchestrator::Orchestrator::new(checker, builder, tester, committer, reverter);

    for res in rx {
        match res {
            Ok(event) => {
                // TODO make this gated on a verbose flag
                println!("changed: {:?}", event);
                let fce = orchestrator::FileChangeEvent::new(event); 
                let result = orc.handle_event(fce);
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

// TODO if not specified find where the config file is, and run from there
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

fn get_config(path: std::path::PathBuf) -> io::Result<Config> {
    let contents = std::fs::read_to_string(path);
    match contents {
        Ok(file_contents) => {
            let c: Config = serde_json::from_str(file_contents.as_ref()).unwrap();
            return Ok(c);
        }
        Err(e) => return Err(e),
    }
}

fn main() {
    let opts: Opts = Opts::parse();
    let root = match opts.root {
        Some(p) => std::path::PathBuf::from(p),
        None => get_path().expect("Unable to get path"),
    };
    let config = match opts.config {
        Some(c) => get_config(std::path::PathBuf::from(c)),
        None => get_config(root.join(".tcr")),
    };

    match config {
        Ok(c) => {
            println!("We read the config! {:?}", c);
            println!(
                "watching {}",
                root.to_str().expect("unable to convert path to string")
            );
            if let Err(e) = watch_and_run(root, c) {
                println!("error: {:?}", e)
            }
        }
        Err(e) => {
            println!(
                "Error reading config at path {:?}: {:?}.\n TODO help user make config",
                root, e
            );
            std::process::exit(1);
        }
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
