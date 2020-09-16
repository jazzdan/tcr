use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

mod orchestrator;
mod runner;

use runner::Runner;

// TODO(dmiller): handle ignoring files
// TODO(dmiller): run build maybe
// TODO(dmiller): run test maybe
// TODO(dmiller): run commit maybe
// TODO(dmiller): otherwise revert
struct CmdRunner<'a> {
    cmd: &'a String,
}

impl runner::Runner for CmdRunner<'_> {
    fn run(&self) -> io::Result<std::process::Output> {
        return Command::new(self.cmd).output();
    }
}

fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    // TODO(dmiller): uhh this doesn't actually watch recursively??
    let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| tx.send(res).unwrap())?;

    watcher.watch(path, RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                println!("changed: {:?}", event);
                let c = CmdRunner {
                    cmd: &String::from("ls"),
                };
                let result = c.run().expect("ls command failed to start");
                io::stdout().write_all(&result.stdout).unwrap();
                io::stderr().write_all(&result.stderr).unwrap();
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

// build.sh
// build.bash
// build.py
// tcr --build build.sh
