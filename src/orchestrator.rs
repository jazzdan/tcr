use colored::*;
use itertools::Itertools;
use notify::Event;
use notify::EventKind;
use std::io::{self, Error, ErrorKind};

use crate::ignore::Checker;
use crate::log::VerboseLogger;

#[derive(Clone)]
pub struct FileChangeEvent {
    pub paths: std::vec::Vec<std::path::PathBuf>,
    pub is_dir: bool,
}

impl FileChangeEvent {
    pub fn new(event: Event) -> FileChangeEvent {
        let is_directory = match event.kind {
            EventKind::Create(e) => match e {
                notify::event::CreateKind::Folder => true,
                _ => false,
            },
            EventKind::Modify(_e) => false,
            EventKind::Remove(e) => match e {
                notify::event::RemoveKind::Folder => true,
                _ => false,
            },
            _ => false,
        };
        return FileChangeEvent {
            paths: event.paths,
            is_dir: is_directory,
        };
    }
}

#[mockall::automock]
pub trait Runner {
    fn run(&mut self) -> io::Result<std::process::Output>;
}

pub struct Orchestrator<'a> {
    ignore: Checker,
    build: &'a mut dyn Runner,
    test: &'a mut dyn Runner,
    commit: &'a mut dyn Runner,
    revert: &'a mut dyn Runner,
    logger: &'a VerboseLogger,
}

fn print_output(out: &std::process::Output) {
    let utf_string = std::str::from_utf8(&out.stdout);
    match utf_string {
        Ok(s) => {
            if s.len() != 0 {
                println!("{}", s)
            }
        }
        Err(e) => eprintln!("Error handlng process stdout: {:#?}", e),
    }

    let utf_string = std::str::from_utf8(&out.stderr);
    match utf_string {
        Ok(s) => {
            if s.len() != 0 {
                println!("{}", s)
            }
        }
        Err(e) => eprintln!("Error handlng process stderr: {:#?}", e),
    }
}

fn handle_output(
    output: std::result::Result<std::process::Output, std::io::Error>,
) -> Option<std::io::Error> {
    match output {
        Ok(res) => {
            print_output(&res);

            if !res.status.success() {
                return Some(Error::new(
                    ErrorKind::Other,
                    "cmd returned non-zero exit code",
                ));
            }
            return None;
        }
        Err(e) => {
            return Some(e);
        }
    }
}

impl Orchestrator<'_> {
    pub fn new<'a>(
        ignore: Checker,
        build: &'a mut dyn Runner,
        test: &'a mut dyn Runner,
        commit: &'a mut dyn Runner,
        revert: &'a mut dyn Runner,
        logger: &'a VerboseLogger,
    ) -> Orchestrator<'a> {
        return Orchestrator {
            ignore,
            build,
            test,
            commit,
            revert,
            logger,
        };
    }
    pub fn handle_event(
        &mut self,
        event: FileChangeEvent,
    ) -> std::result::Result<(), std::io::Error> {
        let paths_str: String = event
            .paths
            .iter()
            .map(|p| p.to_str().unwrap())
            .intersperse(", ")
            .collect();
        if self.ignore.is_ignored(event) {
            self.logger
                .log(format!("{} {}", "Files are ignored: ".yellow(), paths_str));
            return Ok(());
        } else {
            println!("{}: {}", "Saw file changes".yellow(), paths_str);
        }

        println!("Running build..");
        let build = self.build.run();
        match handle_output(build) {
            Some(err) => {
                println!("{}: {:?}", "Build failed".red(), err);
                let res = self.run_revert();
                if res.is_err() {
                    let err = res.err();
                    return Err(err.unwrap());
                }
                return Ok(());
            }
            None => println!("{}", "Build succeeded".green()),
        }
        let test = self.test.run();
        match handle_output(test) {
            Some(err) => {
                println!("{}: {:?}", "Test failed".red(), err);
                let res = self.run_revert();
                if res.is_err() {
                    let err = res.err();
                    return Err(err.unwrap());
                }
                return Ok(());
            }
            None => println!("{}", "Tests passed".green()),
        }

        let commit = self.commit.run();
        match commit {
            Ok(_res) => println!("{}", "Changes committed".green()),
            Err(e) => {
                eprintln!("{}", "Error comitting changes".red());
                return Err(e);
            }
        }
        return Ok(());
    }

    fn run_revert(&mut self) -> io::Result<std::process::Output> {
        let revert_res = self.revert.run();
        match revert_res {
            Ok(out) => Ok(out),
            Err(e) => {
                println!("Error reverting: {:?}", e);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> std::path::PathBuf {
        return std::path::PathBuf::from(r"/home/stuff");
    }

    fn ok_event() -> FileChangeEvent {
        return FileChangeEvent {
            paths: vec![std::path::PathBuf::from(r"/home/stuff/hi")],
            is_dir: false,
        };
    }

    #[test]
    fn test_mock_runner() {
        let mut mock = MockRunner::default();
        mock.expect_run()
            .returning(|| std::process::Command::new("true").output());
        mock.run().expect("not to fail");
    }

    fn fail() -> MockRunner {
        let mut fail = MockRunner::default();
        fail.expect_run()
            .times(1)
            .returning(|| std::process::Command::new("false").output());

        return fail;
    }

    fn not_called() -> MockRunner {
        let mut fail = MockRunner::default();
        fail.expect_run().never();

        return fail;
    }

    fn succeed() -> MockRunner {
        let mut success = MockRunner::default();
        success
            .expect_run()
            .times(1)
            .returning(|| std::process::Command::new("true").output());

        return success;
    }

    fn called() -> MockRunner {
        return succeed();
    }

    fn logger() -> VerboseLogger {
        return VerboseLogger::new(false);
    }

    #[test]
    fn test_orchestrator_build_fails() {
        let mut build = fail();

        let mut test = not_called();
        let mut commit = not_called();
        let mut revert = called();

        let mut orc = Orchestrator {
            ignore: Checker::new(root(), None),
            build: &mut build,
            test: &mut test,
            commit: &mut commit,
            revert: &mut revert,
            logger: &logger(),
        };

        orc.handle_event(ok_event()).expect("This shouldn't error");
    }

    #[test]
    fn test_orchestrator_build_succeeds_test_fails() {
        let mut build = succeed();
        let mut test = fail();

        let mut commit = not_called();
        let mut revert = called();

        let mut orc = Orchestrator {
            ignore: Checker::new(root(), None),
            build: &mut build,
            test: &mut test,
            commit: &mut commit,
            revert: &mut revert,
            logger: &logger(),
        };

        orc.handle_event(ok_event()).expect("This shouldn't error");
    }

    #[test]
    fn ignore_git_directory() {
        let mut build = not_called();
        let mut test = not_called();
        let mut commit = not_called();
        let mut revert = not_called();

        let mut orc = Orchestrator {
            ignore: Checker::new(root(), None),
            build: &mut build,
            test: &mut test,
            commit: &mut commit,
            revert: &mut revert,
            logger: &logger(),
        };

        let event = FileChangeEvent {
            paths: vec![std::path::PathBuf::from(r"/home/stuff/.git")],
            is_dir: true,
        };

        orc.handle_event(event).expect("This shouldn't error");
    }
}
