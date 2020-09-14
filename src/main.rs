use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

// TODO(dmiller): handle ignoring files
// TODO(dmiller): run build maybe
// TODO(dmiller): run test maybe
// TODO(dmiller): run commit maybe
// TODO(dmiller): otherwise revert
fn handle_change_file() {
    let result = Command::new("ls")
        .output()
        .expect("ls command failed to start");

    io::stdout().write_all(&result.stdout).unwrap();
    io::stderr().write_all(&result.stderr).unwrap();
}

fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| tx.send(res).unwrap())?;

    watcher.watch(path, RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                println!("changed: {:?}", event);
                handle_change_file();
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

    println!("watching {}", path.to_str().expect("unable to convert path to string"));
    if let Err(e) = watch(path) {
        println!("error: {:?}", e)
    }
}
